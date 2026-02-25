#![no_std]
#![no_main]

use core::slice::from_raw_parts;

use alloc::vec::Vec;
use goblin::elf::program_header::{PF_R, PF_W, PF_X, PT_LOAD};
use kernel_hal::{
    mem::{CachePolicy, MMUFlags, PageProperty, Privilege, virt_to_phys},
    task::launch_multitask,
};
use limine::{
    BaseRevision,
    modules::InternalModule,
    request::{FramebufferRequest, ModuleRequest, StackSizeRequest},
};
use object::{
    ipc::{Channel, MessagePacket},
    mem::{PAGE_SIZE, Vmo, align_up_by_page_size},
    object::{Handle, Rights},
    task::Process,
};
use protocol::{
    BOOT_DATA_CNT, BOOT_FB_HANDLE_IDX, BOOT_HANDLE_CNT, BOOT_TERM_HANDLE_IDX, FB_HEIGHT_IDX,
    FB_WIDTH_IDX, FIRST_HANDLE, PROC_HANDLE_IDX, PROC_START_HANDLE_CNT, ProcessStartInfo,
    TERM_SIZE_IDX, VMAR_HANDLE_IDX,
};
use syscall::syscall_handler;

use crate::stack::{new_user_stack, push_stack};

extern crate alloc;

mod stack;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(4);

#[used]
#[unsafe(link_section = ".requests")]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(128 * 1024);

#[used]
#[unsafe(link_section = ".requests")]
static FRAME_BUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static USER_BOOT_REQUEST: ModuleRequest = ModuleRequest::new().with_internal_modules(&[
    &InternalModule::new().with_path(c"user_boot"),
    &InternalModule::new().with_path(c"terminal"),
]);

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    kernel_hal::init();
    object::init();
    log::info!("kernel initialized");

    let process = Process::new();
    let vmar = process.root_vmar();

    let files = USER_BOOT_REQUEST.get_response().unwrap().modules();
    let user_boot_file = files[0];
    let terminal_file = files[1];

    let user_boot_data =
        unsafe { from_raw_parts(user_boot_file.addr(), user_boot_file.size() as usize) };
    let terminal_data =
        unsafe { from_raw_parts(terminal_file.addr(), terminal_file.size() as usize) };

    let user_boot = goblin::elf::Elf::parse(user_boot_data).unwrap();

    for segment in user_boot
        .program_headers
        .iter()
        .filter(|s| s.p_type == PT_LOAD)
    {
        let vaddr = segment.p_vaddr as usize;
        let memsz = segment.p_memsz as usize;
        let flags = segment.p_flags;

        let page_offset = vaddr % PAGE_SIZE;
        let aligned_vaddr = vaddr - page_offset;

        let aligned_memsz = align_up_by_page_size(memsz + page_offset);

        let vmo = Vmo::allocate_ram(aligned_memsz / PAGE_SIZE).unwrap();
        let file_data = &user_boot_data[segment.file_range()];
        vmo.write_bytes(page_offset, file_data).unwrap();
        if file_data.len() < memsz {
            vmo.write_bytes(
                file_data.len() + page_offset,
                &alloc::vec![0u8; memsz - file_data.len()],
            )
            .unwrap();
        }

        let mut mmu_flags = MMUFlags::empty();
        if flags & PF_R != 0 {
            mmu_flags |= MMUFlags::READ;
        }
        if flags & PF_W != 0 {
            mmu_flags |= MMUFlags::WRITE;
        }
        if flags & PF_X != 0 {
            mmu_flags |= MMUFlags::EXECUTE;
        }

        let region = vmar.create_child(aligned_vaddr, aligned_memsz).unwrap();

        region
            .map(
                0,
                &vmo,
                PageProperty::new(mmu_flags, CachePolicy::CacheCoherent, Privilege::User),
                false,
            )
            .unwrap();
    }

    let entry_point = user_boot.entry as usize;
    log::debug!("entry: {:#x}", entry_point);

    let stack = new_user_stack(vmar.clone()).unwrap();
    let mut stack_ptr = stack.end();

    let terminal_region = vmar
        .allocate_child(align_up_by_page_size(terminal_data.len()))
        .unwrap();
    let terminal_vmo = Vmo::allocate_ram(terminal_region.page_count()).unwrap();
    terminal_region
        .map(0, &terminal_vmo, PageProperty::user_data(), false)
        .unwrap();
    terminal_vmo.write_bytes(0, terminal_data).unwrap();

    log::debug!("pushing handles");

    let (kernel_endpoint, user_endpoint) = Channel::new();
    let channel = process.add_handle(Handle::new(user_endpoint, Rights::READ));
    assert_eq!(channel.as_raw(), FIRST_HANDLE);

    let process_handle = Handle::new(process.clone(), Rights::PROCESS);
    let vmar_handle = Handle::new(vmar.clone(), Rights::VMAR);

    let proc_info = ProcessStartInfo {
        vmar_base: vmar.base(),
        vmar_size: vmar.size(),
    };
    let proc_info_ptr = push_stack(stack, &mut stack_ptr, &proc_info);

    log::info!("Staring user boot with info: {:#x?}", proc_info);

    let thread = process.new_thread();
    process.start(
        thread.clone(),
        entry_point,
        stack_ptr,
        |ctx| {
            ctx.set_first_arg(proc_info_ptr);
        },
        syscall_handler,
    );

    let mut handles =
        alloc::vec![Handle::new(process.clone(), Rights::empty()); PROC_START_HANDLE_CNT];
    handles[PROC_HANDLE_IDX] = process_handle;
    handles[VMAR_HANDLE_IDX] = vmar_handle;

    kernel_endpoint
        .write(MessagePacket {
            data: Vec::new(),
            handles,
        })
        .unwrap();

    let frame_buffer = FRAME_BUFFER_REQUEST
        .get_response()
        .take()
        .unwrap()
        .framebuffers()
        .next()
        .unwrap();

    let fb_len = frame_buffer.width() as usize
        * frame_buffer.height() as usize
        * (frame_buffer.bpp() as usize / 8);
    let fb_vmo = Vmo::acquire_iomem(virt_to_phys(frame_buffer.addr() as usize), fb_len).unwrap();

    let mut data = alloc::vec![0usize; BOOT_DATA_CNT];
    data[TERM_SIZE_IDX] = terminal_data.len();
    data[FB_WIDTH_IDX] = frame_buffer.width() as usize;
    data[FB_HEIGHT_IDX] = frame_buffer.height() as usize;
    let data = data
        .iter()
        .flat_map(|d| d.to_le_bytes().into_iter())
        .collect();

    let mut handles = alloc::vec![Handle::new(process.clone(), Rights::empty()); BOOT_HANDLE_CNT];
    handles[BOOT_TERM_HANDLE_IDX] = Handle::new(terminal_region, Rights::VMAR);
    handles[BOOT_FB_HANDLE_IDX] = Handle::new(fb_vmo, Rights::VMO);

    kernel_endpoint
        .write(MessagePacket { data, handles })
        .unwrap();

    launch_multitask();

    kernel_hal::platform::idle_loop();
}
