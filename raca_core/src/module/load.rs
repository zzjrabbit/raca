use alloc::vec::Vec;

use super::*;

impl Module {
    pub fn load(data: &[u8]) -> Arc<Self> {
        let binary = Elf::parse(data).unwrap();

        let mut section_addresses = BTreeMap::new();

        let page_table = KERNEL_PAGE_TABLE.lock();

        let mut memory_sections = Vec::new();

        for (id, section) in binary.section_headers.iter().enumerate() {
            if section.is_relocation() {
                continue;
            }

            let addr = unsafe {
                MODULE_ALLOCATOR
                    .lock()
                    .malloc(Layout::from_size_align(section.sh_size as usize, 4096).unwrap())
                    .unwrap()
                    .addr()
            };

            memory_sections.push((addr.get() as u64, section.sh_size as usize));

            section_addresses.insert(id, addr.get() as u64);
        }

        for (section_index, relocation_section) in binary.shdr_relocs {
            let relocationed_section = binary.section_headers.get(section_index).unwrap();
            let relocationed_section = binary
                .section_headers
                .get(relocationed_section.sh_info as usize)
                .unwrap();

            for relocation in relocation_section.iter() {
                let symbol = binary.syms.get(relocation.r_sym).unwrap();

                let section_address = if symbol.is_import() {
                    let symbol_name = binary.strtab.get_at(symbol.st_name).unwrap();
                    if let Some(address) = KERNEL_SYMBOL_TABLE.get(symbol_name) {
                        *address
                    } else {
                        panic!("unknow symbol {}!", symbol_name);
                    }
                } else {
                    *section_addresses.get(&symbol.st_shndx).unwrap() + symbol.st_value
                };
                let offset = relocation.r_offset + relocationed_section.sh_offset;
                let addend = relocation.r_addend.unwrap_or(0);

                let target_address = VirtAddr::new(if addend >= 0 {
                    section_address + addend as u64
                } else {
                    section_address - (-addend as u64)
                });

                unsafe {
                    ((data.as_ptr() as u64 + offset as u64) as *mut u64)
                        .write(target_address.as_u64());
                }
            }
        }

        for (&section_id, &section_address) in section_addresses.iter() {
            let section = &binary.section_headers[section_id];

            page_table.write_to_mapped_address(
                &data[section.sh_offset as usize
                    ..section.sh_offset as usize + section.sh_size as usize],
                VirtAddr::new(section_address),
            );
        }

        let symbol_addresses = binary
            .syms
            .iter()
            .filter(|symbol| section_addresses.get(&symbol.st_shndx).is_some())
            .map(|symbol| {
                (
                    binary.strtab.get_at(symbol.st_name).unwrap().to_string(),
                    *section_addresses.get(&symbol.st_shndx).unwrap() + symbol.st_value,
                )
            })
            .collect::<BTreeMap<_, _>>();

        let init_address = *symbol_addresses.get("module_init").unwrap();
        let exit_address = *symbol_addresses.get("module_exit").unwrap();

        let info_address = *symbol_addresses.get("MODULE_INFO").unwrap();
        let module_info = unsafe { &mut *(info_address as *mut InfoStruct) };
        let name = module_info.name;

        let module = Self {
            name: name.into(),
            license: module_info.license.into(),
            memory_sections,
            symbol_addresses,
            init_address,
            exit_address,
        };

        log::info!(
            "module {} loaded ( license {} )",
            module.get_name(),
            module.license
        );

        Arc::new(module)
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_function_address(&self, name: &str) -> u64 {
        self.symbol_addresses.get(name).unwrap_or(&0).clone()
    }

    pub fn init(&self) -> usize {
        let init: extern "C" fn() -> usize = unsafe { transmute(self.init_address) };
        init()
    }
}
