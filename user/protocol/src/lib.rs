#![no_std]

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProcStartInfo {
    pub vdso_load_base: usize,
    pub vdso_base: usize,
    pub vdso_size: usize,
    pub elf_base: usize,
    pub elf_size: usize,
    pub load_base: usize,
}
