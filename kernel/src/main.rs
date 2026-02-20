#![no_std]
#![no_main]

use alloc::vec::Vec;
use elf::{
    ElfBytes,
    abi::{PF_R, PF_W, PF_X, PT_LOAD},
    endian::LittleEndian,
};
use kernel_hal::{
    mem::{CachePolicy, MMUFlags, PageProperty, Privilege},
    task::launch_multitask,
};
use limine::{BaseRevision, request::StackSizeRequest};
use object::{
    ipc::{Channel, MessagePacket},
    mem::{PAGE_SIZE, Vmo, align_up_by_page_size},
    object::{Handle, Rights},
    task::Process,
};
use protocol::{
    FIRST_HANDLE, PROC_HANDLE_IDX, PROC_START_HANDLE_CNT, ProcessStartInfo, VMAR_HANDLE_IDX,
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

static USER_BOOT: &[u8] = include_bytes!(env!("USER_BOOT_PATH"));

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    kernel_hal::init();
    object::init();
    log::info!("kernel initialized");

    let process = Process::new();
    let vmar = process.root_vmar();

    let user_boot = ElfBytes::<LittleEndian>::minimal_parse(USER_BOOT).unwrap();
    let load_segments = || {
        user_boot
            .segments()
            .unwrap()
            .into_iter()
            .filter(|s| s.p_type == PT_LOAD)
    };

    let min_vaddr = load_segments().map(|s| s.p_vaddr).min().unwrap() as usize;
    let max_vaddr = load_segments()
        .map(|s| s.p_vaddr + s.p_memsz)
        .max()
        .unwrap() as usize;
    let size = max_vaddr - min_vaddr;
    let region = vmar
        .create_child(min_vaddr, align_up_by_page_size(size))
        .unwrap();

    for segment in user_boot
        .segments()
        .unwrap()
        .into_iter()
        .filter(|s| s.p_type == PT_LOAD)
    {
        let vaddr = segment.p_vaddr as usize;
        let memsz = segment.p_memsz as usize;
        let flags = segment.p_flags;

        let page_offset = vaddr % PAGE_SIZE;
        let aligned_vaddr = vaddr - page_offset;

        let aligned_memsz = align_up_by_page_size(memsz + page_offset);

        let vmo = Vmo::allocate_ram(aligned_memsz / PAGE_SIZE).unwrap();
        let file_data = user_boot.segment_data(&segment).unwrap();
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

        region
            .map(
                aligned_vaddr - min_vaddr,
                &vmo,
                PageProperty::new(mmu_flags, CachePolicy::CacheCoherent, Privilege::User),
                false,
            )
            .unwrap();
    }

    let entry_point = user_boot.ehdr.e_entry as usize;
    log::debug!("entry: {:#x}", entry_point);

    let stack = new_user_stack(process.root_vmar().clone()).unwrap();
    let mut stack_ptr = stack.end();

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
    kernel_endpoint
        .write(MessagePacket {
            data: Vec::from(b"Hello, World"),
            handles: Vec::new(),
        })
        .unwrap();

    launch_multitask();

    kernel_hal::platform::idle_loop();
}
