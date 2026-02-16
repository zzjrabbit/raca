#![no_std]
#![no_main]

use elf::{
    ElfBytes,
    abi::{PF_R, PF_W, PF_X, PT_LOAD, SHT_RELA},
    endian::LittleEndian,
};
use kernel_hal::{
    mem::{CachePolicy, MMUFlags, PageProperty, Privilege},
    task::launch_multitask,
};
use limine::BaseRevision;
use object::{
    mem::{PAGE_SIZE, Vmo, align_up_by_page_size},
    task::Process,
};
use syscall::syscall_handler;

use crate::stack::new_user_stack;

extern crate alloc;

mod stack;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(4);

static USER_BOOT: &[u8] = include_bytes!(env!("USER_BOOT_PATH"));

const R_LARCH_RELATIVE: u32 = 3;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    kernel_hal::init();
    object::init();
    log::info!("kernel initialized");

    let process = Process::new();
    let vmar = process.root_vmar();

    let user_boot = ElfBytes::<LittleEndian>::minimal_parse(USER_BOOT).unwrap();
    let user_boot_size = align_up_by_page_size(
        user_boot
            .segments()
            .unwrap()
            .into_iter()
            .filter(|s| s.p_type == PT_LOAD)
            .map(|s| s.p_vaddr + s.p_memsz)
            .max()
            .unwrap() as usize,
    );
    let user_boot_region = vmar.allocate_child(user_boot_size).unwrap();
    let load_base = user_boot_region.base();

    log::debug!(
        "user boot: base={:#x} size={:#x}",
        load_base,
        user_boot_size
    );

    for segment in user_boot
        .segments()
        .unwrap()
        .into_iter()
        .filter(|s| s.p_type == PT_LOAD)
    {
        let vaddr = segment.p_vaddr as usize + load_base;
        let memsz = segment.p_memsz as usize;
        let flags = segment.p_flags;

        let page_offset = vaddr % PAGE_SIZE;
        let aligned_vaddr = vaddr - page_offset;

        let aligned_memsz = align_up_by_page_size(memsz + page_offset);

        let vmo = Vmo::allocate_ram(aligned_memsz / PAGE_SIZE).unwrap();
        vmo.write_bytes(page_offset, user_boot.segment_data(&segment).unwrap())
            .unwrap();

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

        user_boot_region
            .map(
                aligned_vaddr - load_base,
                &vmo,
                PageProperty::new(mmu_flags, CachePolicy::CacheCoherent, Privilege::User),
                false,
            )
            .unwrap();
    }

    let relas = user_boot
        .section_headers()
        .unwrap()
        .into_iter()
        .filter_map(|s| (s.sh_type == SHT_RELA).then_some(user_boot.section_data_as_relas(&s)))
        .flatten()
        .flatten();

    for rela in relas {
        if rela.r_type != R_LARCH_RELATIVE {
            log::debug!("SKIP RELA");
            continue;
        }
        let address = rela.r_offset as usize + load_base;
        let value = (load_base as i64 + rela.r_addend) as usize;
        user_boot_region.write_val(address, &value).unwrap();
    }

    let entry_point = user_boot.ehdr.e_entry as usize + load_base;
    log::debug!("entry: {:#x}", entry_point);

    let stack = new_user_stack(process.root_vmar().clone()).unwrap();

    let thread = process.new_thread();
    process.start(
        thread.clone(),
        entry_point,
        stack.end(),
        |_| {},
        syscall_handler,
    );

    launch_multitask();

    kernel_hal::platform::idle_loop();
}
