use elf::{
    ElfBytes,
    abi::{ET_DYN, ET_EXEC, PF_R, PF_W, PF_X, PT_LOAD, R_X86_64_RELATIVE, SHT_RELA},
    endian::LittleEndian,
};
use errors::{Errno, Result};

use crate::vm::{MMUFlags, PAGE_SIZE, Vmar, Vmo};

const R_LARCH_RELATIVE: u32 = 3;

pub fn load_elf(vmar: &Vmar, elf_data: &[u8]) -> Result<usize> {
    let elf = ElfBytes::<LittleEndian>::minimal_parse(elf_data)
        .map_err(|_| Errno::InvArg.no_message())?;

    if !matches!(elf.ehdr.e_type, ET_EXEC | ET_DYN) {
        return Err(Errno::InvArg.no_message());
    }

    let load_segments = || -> Result<_> {
        Ok(elf
            .segments()
            .ok_or(Errno::InvArg.no_message())?
            .into_iter()
            .filter(|segment| segment.p_type == PT_LOAD))
    };

    let min_vaddr = load_segments()?
        .map(|segment| segment.p_vaddr)
        .min()
        .unwrap() as usize;
    let aligned_min_vaddr = min_vaddr.div_floor(PAGE_SIZE);
    let size = load_segments()?
        .map(|segment| segment.p_vaddr + segment.p_memsz)
        .max()
        .unwrap() as usize
        - min_vaddr;
    let aligned_size = min_vaddr % PAGE_SIZE + size.div_ceil(PAGE_SIZE);

    let region = if elf.ehdr.e_type == ET_EXEC {
        vmar.allocate_at(aligned_min_vaddr, aligned_size)?
    } else {
        vmar.allocate(aligned_size)?
    };

    let load_base = if elf.ehdr.e_type == ET_EXEC {
        0
    } else {
        region.base()
    };
    let segment_offset_base = if elf.ehdr.e_type == ET_EXEC {
        region.base()
    } else {
        0
    };

    for segment in load_segments()? {
        let offset = segment.p_vaddr as usize - segment_offset_base;
        let aligned_offset = offset.div_floor(PAGE_SIZE);
        let page_offset = offset - aligned_offset;
        let memsz = segment.p_memsz as usize;

        let page_count = (memsz + page_offset).div_ceil(PAGE_SIZE);
        let vmo = Vmo::allocate(page_count)?;

        let file_data = elf
            .segment_data(&segment)
            .map_err(|_| Errno::InvArg.no_message())?;
        vmo.write(page_offset, file_data)?;
        if file_data.len() < memsz {
            vmo.write(
                file_data.len() + page_offset,
                &alloc::vec![0u8; memsz - file_data.len()],
            )
            .unwrap();
        }

        let flags = {
            let mut flags = MMUFlags::empty();

            let raw_flags = segment.p_flags;
            if raw_flags & PF_R != 0 {
                flags |= MMUFlags::READ;
            }
            if raw_flags & PF_W != 0 {
                flags |= MMUFlags::WRITE;
            }
            if raw_flags & PF_X != 0 {
                flags |= MMUFlags::EXECUTE;
            }

            flags
        };
        region.map(aligned_offset, &vmo, flags)?;
    }

    if elf.ehdr.e_type == ET_DYN
        && let Some(sections) = elf.section_headers()
    {
        let relas = sections
            .into_iter()
            .filter(|s| s.sh_type == SHT_RELA)
            .flat_map(|s| elf.section_data_as_relas(&s))
            .flatten();
        for rela in relas {
            let target = rela.r_offset as usize + load_base;
            let value = (rela.r_addend + load_base as i64) as usize;
            match rela.r_type {
                R_X86_64_RELATIVE | R_LARCH_RELATIVE => {
                    vmar.write_val(target, &value)?;
                }
                _ => unimplemented!(),
            }
        }
    }

    Ok(elf.ehdr.e_entry as usize + load_base)
}
