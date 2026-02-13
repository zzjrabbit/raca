use alloc::sync::Arc;
use elf::{
    ElfBytes,
    abi::{PF_X, R_X86_64_RELATIVE},
    endian::LittleEndian,
};
use kernel_hal::mem::{CachePolicy, MMUFlags, PageProperty, Privilege, VirtAddr};

use crate::{
    mem::{PAGE_SIZE, Vmar, Vmo, align_up_by_page_size},
    task::Process,
};

static VDSO: &[u8] = include_bytes!(env!("VDSO_DYLIB_PATH"));

impl Process {
    pub fn map_vdso(vmar: Arc<Vmar>) -> (VirtAddr, Arc<Vmar>) {
        let vdso_len = VDSO.len();
        let aligned_vdso_len = align_up_by_page_size(vdso_len);
        let vdso_elf = vmar.allocate_child(aligned_vdso_len).unwrap();
        let vmo = Vmo::allocate_ram(vdso_elf.page_count()).unwrap();
        vmo.write_bytes(0, VDSO).unwrap();
        vdso_elf
            .map(0, &vmo, PageProperty::user_data(), false)
            .unwrap();

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

        use elf::abi::SHT_RELA;

        let relas = vdso
            .section_headers()
            .unwrap()
            .into_iter()
            .filter(|s| s.sh_type == SHT_RELA)
            .map(|s| vdso.section_data_as_relas(&s))
            .flatten()
            .flatten();

        let (vdso_symtab, vdso_strtab) = vdso.dynamic_symbol_table().unwrap().unwrap();

        for rela in relas {
            let real_addr = if rela.r_type == R_X86_64_RELATIVE {
                (region.base() as i64 + rela.r_addend) as usize
            } else {
                #[cfg(not(feature = "libos"))]
                panic!("Unknown relocation type found in vDSO!");
                #[cfg(feature = "libos")]
                {
                    use kernel_hal::arch::task::syscall_fn_entry;

                    (syscall_fn_entry as *const () as i64 + rela.r_addend) as usize
                }
            };
            let offset = rela.r_offset;
            region
                .write_val(offset as usize + region.base(), &real_addr)
                .unwrap();

            let symbol = vdso_symtab.get(rela.r_sym as usize).unwrap();
            let symbol_name = vdso_strtab.get(symbol.st_name as usize).unwrap();

            log::debug!(
                "plt: offset={:#x} addr={:#x} name={}",
                offset,
                real_addr,
                symbol_name
            );
        }

        (region.base(), vdso_elf)
    }
}
