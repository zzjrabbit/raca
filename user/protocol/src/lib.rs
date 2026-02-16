#![no_std]

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProcStartInfo {
    pub elf_base: usize,
    pub elf_size: usize,
    pub load_base: usize,
}
