use errors::{Errno, Result};
use goblin::elf::{
    Elf,
    header::ET_EXEC,
    program_header::{PF_R, PF_W, PF_X, PT_LOAD},
};

use crate::vm::{MMUFlags, PAGE_SIZE, Vmar, Vmo};

pub fn load_elf(vmar: &Vmar, elf_data: &[u8]) -> Result<usize> {
    let elf = Elf::parse(elf_data).map_err(|_| Errno::InvArg.no_message())?;

    if !matches!(elf.header.e_type, ET_EXEC) {
        return Err(Errno::InvArg.no_message());
    }

    let load_segments = || -> Result<_> {
        Ok(elf
            .program_headers
            .iter()
            .filter(|segment| segment.p_type == PT_LOAD))
    };

    for segment in load_segments()? {
        let vaddr = segment.p_vaddr as usize;
        let memsz = segment.p_memsz as usize;

        let page_offset = vaddr % PAGE_SIZE;
        let aligned_vaddr = vaddr - page_offset;

        let aligned_memsz = (memsz + page_offset).div_ceil(PAGE_SIZE) * PAGE_SIZE;

        let vmo = Vmo::allocate(aligned_memsz / PAGE_SIZE)?;

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
        let region = vmar.allocate_at(aligned_vaddr, aligned_memsz)?;
        region.map(0, &vmo, flags)?;

        let file_data = &elf_data[segment.file_range()];
        vmo.write(page_offset, file_data)?;
        if file_data.len() < memsz {
            vmo.write(
                file_data.len() + page_offset,
                &alloc::vec![0u8; memsz - file_data.len()],
            )?;
        }
    }

    Ok(elf.entry as usize)
}
