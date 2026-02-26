use core::arch::asm;

use crate::{VirtAddr, registers::Asid};

pub fn flush(addr: VirtAddr) {
    let asid = Asid.read_asid();
    unsafe {
        asm!(
            "invtlb 0x05, {}, {}",
            in(reg) asid,
            in(reg) addr.as_u64()
        );
    }
}

pub fn flush_all() {
    unsafe {
        asm!("invtlb 0, $zero, $zero");
    }
}
