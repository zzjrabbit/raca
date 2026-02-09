use alloc::sync::Arc;
use elf::{ElfBytes, abi::PF_X, endian::LittleEndian};
use kernel_hal::mem::{CachePolicy, MMUFlags, PageProperty, Privilege, VirtAddr};

use crate::{
    mem::{PAGE_SIZE, Vmar, Vmo, align_up_by_page_size},
    task::Process,
};

static VDSO: &[u8] = include_bytes!(concat!("../../../../", env!("VDSO_PATH")));

impl Process {
    pub fn map_vdso(vmar: Arc<Vmar>) -> Arc<Vmar> {
        let vdso = ElfBytes::<LittleEndian>::minimal_parse(VDSO).unwrap();

        let region = vmar
            .allocate_child(align_up_by_page_size(
                vdso.segments()
                    .unwrap()
                    .into_iter()
                    .map(|s| s.p_vaddr + s.p_memsz)
                    .max()
                    .unwrap_or(0) as usize,
            ))
            .unwrap();

        for segment in vdso.segments().unwrap() {
            let data = vdso.segment_data(&segment).unwrap();

            let vmo =
                Vmo::allocate_ram(align_up_by_page_size(segment.p_memsz as usize) / PAGE_SIZE)
                    .unwrap();
            region
                .map(
                    segment.p_vaddr as VirtAddr,
                    &vmo,
                    PageProperty::new(
                        if segment.p_flags & PF_X != 0 {
                            MMUFlags::EXECUTE
                        } else {
                            MMUFlags::empty()
                        } | MMUFlags::READ,
                        CachePolicy::CacheCoherent,
                        Privilege::User,
                    ),
                    false,
                )
                .unwrap();
            vmo.write_bytes(0, data).unwrap();
        }

        #[cfg(feature = "libos")]
        {
            use elf::abi::SHT_RELA;

            let rela_header = vdso
                .section_headers()
                .unwrap()
                .into_iter()
                .find(|s| s.sh_type == SHT_RELA)
                .unwrap();
            let relas = vdso.section_data_as_relas(&rela_header).unwrap();

            for rela in relas {
                use kernel_hal::arch::task::syscall_fn_entry;

                let real_addr = (syscall_fn_entry as *const () as i64 + rela.r_addend) as usize;
                let offset = rela.r_offset;
                region
                    .write_val(offset as usize + region.base(), &real_addr)
                    .unwrap();
            }
        }

        region
    }
}
