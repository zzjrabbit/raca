use alloc::collections::btree_map::BTreeMap;
use spin::Lazy;
use x86_64::instructions::port::PortReadOnly;
//use x86_64::instructions::port::PortReadOnly;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;

use super::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::arch::apic::LAPIC;
use crate::device::terminal::TERMINAL;
use crate::fs::dev::KEYBOARD_INPUT;
use crate::task::SCHEDULER;

const INTERRUPT_INDEX_OFFSET: u8 = 32;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = INTERRUPT_INDEX_OFFSET,
    ApicError,
    ApicSpurious,
//    Keyboard,
//    Mouse,
}

const BASE: u8 = InterruptIndex::ApicSpurious as u8 + 1;

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint);
    idt.segment_not_present.set_handler_fn(segment_not_present);
    idt.invalid_opcode.set_handler_fn(invalid_opcode);
    idt.page_fault.set_handler_fn(page_fault);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault);

    idt[InterruptIndex::Timer as u8].set_handler_fn(timer_interrupt);
    idt[InterruptIndex::ApicError as u8].set_handler_fn(lapic_error);
    idt[InterruptIndex::ApicSpurious as u8].set_handler_fn(spurious_interrupt);
    //idt[InterruptIndex::Keyboard as u8].set_handler_fn(keyboard_interrupt);
    //idt[InterruptIndex::Mouse as u8].set_handler_fn(mouse_interrupt);
    
    macro_rules!  other_int_regist {
        ($id: expr) => {
            {
                extern "x86-interrupt" fn handler(frame: InterruptStackFrame) {
                    other_interrupt($id, frame);
                }
                
                idt[$id + BASE].set_handler_fn(handler);
            }
        }
    }
    
    other_int_regist!(0);
    other_int_regist!(1);
    other_int_regist!(2);
    other_int_regist!(3);
    other_int_regist!(4);
    other_int_regist!(5);
    other_int_regist!(6);
    other_int_regist!(7);
    other_int_regist!(8);
    other_int_regist!(9);
    other_int_regist!(10);
    other_int_regist!(11);
    other_int_regist!(12);
    other_int_regist!(13);
    other_int_regist!(14);
    other_int_regist!(15);
    other_int_regist!(16);
    other_int_regist!(17);
    other_int_regist!(18);
    other_int_regist!(19);
    other_int_regist!(20);
    other_int_regist!(21);
    other_int_regist!(22);
    other_int_regist!(23);
    other_int_regist!(24);
    other_int_regist!(25);
    other_int_regist!(26);
    other_int_regist!(27);
    other_int_regist!(28);
    other_int_regist!(29);
    other_int_regist!(30);
    other_int_regist!(31);
    other_int_regist!(32);
    other_int_regist!(33);
    other_int_regist!(34);
    other_int_regist!(35);
    other_int_regist!(36);
    other_int_regist!(37);
    other_int_regist!(38);
    other_int_regist!(39);
    other_int_regist!(40);
    other_int_regist!(41);
    other_int_regist!(42);
    other_int_regist!(43);
    other_int_regist!(44);
    other_int_regist!(45);
    other_int_regist!(46);
    other_int_regist!(47);
    other_int_regist!(48);
    other_int_regist!(49);
    other_int_regist!(50);
    other_int_regist!(51);
    other_int_regist!(52);
    other_int_regist!(53);
    other_int_regist!(54);
    other_int_regist!(55);
    other_int_regist!(56);
    other_int_regist!(57);
    other_int_regist!(58);
    other_int_regist!(59);
    other_int_regist!(60);
    other_int_regist!(61);
    other_int_regist!(62);
    other_int_regist!(63);
    
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
    }

    return idt;
});

use spin::*;

type HandlerFunction = fn(frame: InterruptStackFrame);
static HANDLERS: Mutex<BTreeMap<u8, HandlerFunction>> = Mutex::new(BTreeMap::new());

fn other_interrupt(int: u8,frame: InterruptStackFrame) {
    HANDLERS.lock().get(&int).unwrap()(frame);
    //log::info!("int {} handled", int);
}

pub fn add_interrupt_handler(handler: HandlerFunction) -> u8 {
    let mut interrupt_handlers = HANDLERS.lock();
    let index = interrupt_handlers.len() as u8;
    interrupt_handlers.insert(index, handler);
    let vector = index + BASE;
    //log::info!("added int handler for int {}", vector);
    vector
}

#[naked]
extern "x86-interrupt" fn timer_interrupt(_frame: InterruptStackFrame) {
    fn timer_handler(context: VirtAddr) -> VirtAddr {
        let context = SCHEDULER.lock().schedule(context);
        super::apic::end_of_interrupt();
        context
    }

    unsafe {
        core::arch::naked_asm!(
            "cli",
            crate::push_context!(),
            "mov rdi, rsp",
            "call {timer_handler}",
            "mov rsp, rax",
            crate::pop_context!(),
            "sti",
            "iretq",
            timer_handler = sym timer_handler,
        );
    }
}

extern "x86-interrupt" fn lapic_error(_frame: InterruptStackFrame) {
    log::error!("Local APIC error!");
    super::apic::end_of_interrupt();
}

extern "x86-interrupt" fn spurious_interrupt(_frame: InterruptStackFrame) {
    log::debug!("Received spurious interrupt!");
    super::apic::end_of_interrupt();
}

extern "x86-interrupt" fn segment_not_present(frame: InterruptStackFrame, error_code: u64) {
    log::error!("Exception: Segment Not Present\n{:#?}", frame);
    log::error!("Error Code: {:#x}", error_code);
    panic!("Unrecoverable fault occured, halting!");
}

extern "x86-interrupt" fn general_protection_fault(frame: InterruptStackFrame, error_code: u64) {
    log::error!("Exception: General Protection Fault\n{:#?}", frame);
    log::error!("Error Code: {:#x}", error_code);
    x86_64::instructions::hlt();
}

extern "x86-interrupt" fn invalid_opcode(frame: InterruptStackFrame) {
    log::error!("Exception: Invalid Opcode\n{:#?}", frame);
    x86_64::instructions::hlt();
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    log::debug!("Exception: Breakpoint\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, error_code: u64) -> ! {
    log::error!("Exception: Double Fault\n{:#?}", frame);
    log::error!("Error Code: {:#x}", error_code);
    panic!("Unrecoverable fault occured, halting!");
}

/*extern "x86-interrupt" fn keyboard_interrupt(_frame: InterruptStackFrame) {
    let scancode: u8 = unsafe { PortReadOnly::new(0x60).read() };
    let string_option = TERMINAL.lock().handle_keyboard(scancode).clone();
    if let Some(string) = string_option {
        let mut keyboard_input = KEYBOARD_INPUT.lock();
        keyboard_input.push_str(&string);
    }
    super::apic::end_of_interrupt();
}

extern "x86-interrupt" fn mouse_interrupt(_frame: InterruptStackFrame) {
    //let packet = unsafe { PortReadOnly::new(0x60).read() };
    //crate::device::mouse::MOUSE.lock().process_packet(packet);
    super::apic::end_of_interrupt();
}*/

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    log::warn!("Processor: {}", unsafe { LAPIC.lock().id() });
    log::warn!("Exception: Page Fault\n{:#?}", frame);
    log::warn!("Error Code: {:#?}", error_code);
    match Cr2::read() {
        Ok(address) => {
            log::warn!("Fault Address: {:#x}", address);
        }
        Err(error) => {
            log::warn!("Invalid virtual address: {:?}", error);
        }
    }
    x86_64::instructions::hlt();
}
