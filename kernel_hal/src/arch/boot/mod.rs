use core::{alloc::Layout, arch::global_asm};

use alloc::alloc::alloc;
use bit_field::BitField;
use loongarch64::registers::{IpiEnabled, IpiSend, MailSend};

use crate::{
    arch::idle_loop,
    mem::{PageSize, VirtualAddress, VmSpace},
};

pub fn init() {
    IpiEnabled.write(u32::MAX as u64);
    for i in 1..8 {
        boot_ap(i);
    }
}

#[repr(C, packed)]
struct BootData {
    stack_address: u64,
    cpu_id: u64,
}

#[used]
#[unsafe(no_mangle)]
static mut BOOT_DATA: BootData = BootData {
    stack_address: 0,
    cpu_id: 0,
};

#[unsafe(no_mangle)]
extern "C" fn ap_rust_entry() -> ! {
    idle_loop();
}

unsafe extern "C" {
    fn ap_asm_entry();
}

fn boot_ap(cpu_id: u64) {
    log::info!("booting cpu {}!", cpu_id);

    const AP_STACK_SIZE: usize = 16 * 1024;

    let stack = unsafe { alloc(Layout::from_size_align(AP_STACK_SIZE, 4096).unwrap()) };

    unsafe {
        BOOT_DATA.cpu_id = cpu_id;
        BOOT_DATA.stack_address = stack as u64;
    }

    let vaddr = ap_asm_entry as *const () as VirtualAddress;
    let aligned_vaddr = PageSize::Size4K.align_down(vaddr);
    let page_offset = vaddr - aligned_vaddr;

    let kernel_vm_space = unsafe { VmSpace::kernel() };
    let (pm, _) = kernel_vm_space
        .cursor(aligned_vaddr)
        .unwrap()
        .query()
        .unwrap();
    let paddr = pm.get_start_address_of_frame(0).unwrap() + page_offset;

    let entry = paddr;
    log::info!("entry: {:x}", entry);

    let lower_half_data = entry.get_bits(0..32) as u32;
    let upper_half_data = entry.get_bits(32..64) as u32;

    MailSend.send_data(cpu_id, 0, 0, 0, true);

    MailSend.send_data(cpu_id, lower_half_data, 0, 0, true);
    MailSend.send_data(cpu_id, upper_half_data, 1, 0, true);

    IpiSend.send_ipi(cpu_id, 0, true);
}

global_asm!(
    r#"
.extern BOOT_DATA
.extern ap_rust_entry

.align 16
ap_asm_entry:
    bl ap_asm_entry

    li.w $r12, 0x11
    csrwr $r12, 0x180

    li.d $r12, 0
    li.d $r13, 1 << 8
    csrxchg	$r12, $r13, 0x80
    
    li.w $r12, 0xb0
    csrwr $r12, 0x00
    li.w $r12, 0x04
    csrwr $r12, 0x01
    li.w $r12, 0x00
    csrwr $r12, 0x02
    
    la.pcrel $r12, BOOT_DATA
    ld.d $r3, $r12, 0
    ld.d $r2, $r12, 8
    
    bl ap_rust_entry
"#
);
