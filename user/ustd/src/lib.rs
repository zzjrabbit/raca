#![no_std]
#![feature(rustc_private)]

use core::{ffi::c_int, slice::from_raw_parts};

use elf::{
    ElfBytes,
    abi::{R_X86_64_JUMP_SLOT, SHT_RELA, STB_GLOBAL, STV_DEFAULT},
    endian::LittleEndian,
};
use protocol::ProcStartInfo;

extern crate compiler_builtins;

pub fn debug(msg: &str) {
    unsafe {
        vdso::sys_debug(msg.as_ptr(), msg.len());
    }
}

unsafe extern "C" {
    fn main() -> i32;
}

#[unsafe(no_mangle)]
extern "C" fn _start(info: *const ProcStartInfo) -> ! {
    let ProcStartInfo {
        vdso_load_base,
        vdso_base,
        vdso_size,
        elf_base,
        elf_size,
        load_base,
    } = unsafe { info.read() };

    let vdso = unsafe { from_raw_parts(vdso_base as *const u8, vdso_size) };
    let elf = unsafe { from_raw_parts(elf_base as *const u8, elf_size) };

    let vdso = ElfBytes::<LittleEndian>::minimal_parse(vdso).unwrap();
    let elf = ElfBytes::<LittleEndian>::minimal_parse(elf).unwrap();

    let relas = elf
        .section_headers()
        .unwrap()
        .into_iter()
        .filter(|s| s.sh_type == SHT_RELA)
        .map(|s| elf.section_data_as_relas(&s))
        .flatten()
        .flatten();
    let (elf_symtab, elf_strtab) = elf.dynamic_symbol_table().unwrap().unwrap();
    let (vdso_symtab, vdso_strtab) = vdso.dynamic_symbol_table().unwrap().unwrap();

    for rela in relas {
        if rela.r_type != R_X86_64_JUMP_SLOT {
            continue;
        }

        let symbol = elf_symtab.get(rela.r_sym as usize).unwrap();
        let name = elf_strtab.get(symbol.st_name as usize).unwrap();

        let Some(vdso_symbol) = vdso_symtab
            .iter()
            .filter(|s| s.st_bind() == STB_GLOBAL && s.st_vis() == STV_DEFAULT)
            .map(|s| (vdso_strtab.get(s.st_name as usize).unwrap(), s.st_value))
            .find(|(n, _)| n == &name)
            .map(|(_, value)| value + vdso_load_base as u64)
        else {
            panic!()
        };

        let vdso_symbol = (vdso_symbol as i64 + rela.r_addend) as usize;
        unsafe {
            ((rela.r_offset as usize + load_base) as *mut usize).write(vdso_symbol);
        }
    }

    unsafe {
        main();
    }

    loop {}
}

pub fn dummy() {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memset(s: *mut u8, c: c_int, n: usize) -> *mut u8 {
    unsafe { compiler_builtins::mem::memset(s, c, n) }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe { compiler_builtins::mem::memcpy(dest, src, n) }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe { compiler_builtins::mem::memcmp(s1, s2, n) }
}
