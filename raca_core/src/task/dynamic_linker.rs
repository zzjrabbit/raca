use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};
use goblin::elf::{header::EM_SE_C17, Elf};
use spin::Mutex;
use x86_64::{
    structures::paging::{page_table, OffsetPageTable, PhysFrame},
    VirtAddr,
};

use crate::memory::{ExtendedPageTable, MappingType, MemoryManager};

pub static DYNAMIC_LIBRARIES: Mutex<BTreeMap<String, DynamicLibrary>> = Mutex::new(BTreeMap::new());

pub struct DynamicLinker {
    base: usize,
}

#[derive(Clone)]
pub struct DynamicLibrary {
    data: Vec<u8>,
    symbols: BTreeMap<String, usize>,
    /// the offset of the symbol in its section ( with section id )
    symbol_offset: BTreeMap<usize, (usize, usize)>,
    /// size offset( in file )
    sections: BTreeMap<usize, (usize, usize)>,
    /// offset section symbol addend
    relocations: Vec<(usize, usize, String, isize)>,
}

const DYNAMIC_LIBRARY_BASE: usize = 0x114514;

impl DynamicLinker {
    pub const fn new() -> Self {
        Self {
            base: DYNAMIC_LIBRARY_BASE,
        }
    }

    pub fn link(&mut self, page_table: &mut OffsetPageTable<'static>, name: &str) -> Option<()> {
        let library = DYNAMIC_LIBRARIES.lock().get(name)?.clone();

        let mut section_addresses = Vec::new();

        for (&size, _) in library.sections.iter() {
            MemoryManager::alloc_range(
                VirtAddr::new(self.base as u64),
                size as u64,
                MappingType::UserCode.flags(),
                page_table,
            )
            .ok()?;

            section_addresses.push(self.base);
            self.base += size;
        }

        for (section_id, &address) in section_addresses.iter().enumerate() {
            let &(size, offset_in_file) = library.sections.get(&section_id).unwrap();

            page_table.write_to_mapped_address(
                &library.data[offset_in_file..offset_in_file + size],
                VirtAddr::new(address as u64),
            );
        }

        let symbol_addresses = library
            .symbol_offset
            .iter()
            .map(|(&symbol_id, &(offset, section_id))| {
                (symbol_id, section_addresses[section_id] + offset)
            })
            .collect::<BTreeMap<_, _>>();

        for (offset, section, symbol, addend) in library.relocations {
            let symbol_id = library.symbols.get(&symbol).unwrap();
            let symbol_address = *symbol_addresses.get(symbol_id).unwrap();

            let target_address = VirtAddr::new(if addend >= 0 {
                symbol_address + addend as usize
            } else {
                symbol_address - (-addend as usize)
            } as u64);

            let relocation_address = section_addresses[section] + offset;

            page_table.write_to_mapped_address(&relocation_address.to_le_bytes(), target_address);
        }

        Some(())
    }
}

impl DynamicLibrary {
    pub fn new(data: &[u8]) -> Self {
        let binary = Elf::parse(data).unwrap();

        let mut symbols = BTreeMap::new();
        let mut symbol_offset = BTreeMap::new();
        let mut sections = BTreeMap::new();
        let mut relocations = Vec::new();

        for (id, symbol) in binary.syms.iter().enumerate() {
            symbols.insert(
                binary.strtab.get_at(symbol.st_name).unwrap().to_string(),
                id,
            );
            symbol_offset.insert(id, (symbol.st_value as usize, symbol.st_shndx as usize));
        }

        for (id, section) in binary.section_headers.iter().enumerate() {
            if section.is_relocation() {
                continue;
            }

            sections.insert(id, (section.sh_size as usize, section.sh_offset as usize));
        }

        for (section_index, relocation_section) in binary.shdr_relocs {
            let relocationed_section = binary.section_headers.get(section_index).unwrap();

            for relocation in relocation_section.iter() {
                let symbol = binary.syms.get(relocation.r_sym).unwrap();

                let section = relocationed_section.sh_info;
                let offset = relocation.r_offset;
                let addend = relocation.r_addend.unwrap_or(0);

                relocations.push((
                    offset as usize,
                    section as usize,
                    binary.strtab.get_at(symbol.st_name).unwrap().to_string(),
                    addend as isize,
                ));
            }
        }

        Self {
            data: data.to_vec(),
            symbols,
            symbol_offset,
            sections,
            relocations,
        }
    }
}
