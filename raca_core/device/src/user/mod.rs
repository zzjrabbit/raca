use alloc::sync::Arc;
use framework::{
    arch::apic::get_lapic_id,
    task::{process::ProcessId, scheduler::SCHEDULERS, Process, Thread},
};
use spin::RwLock;

pub mod syscall;

pub fn get_current_thread() -> Arc<RwLock<Thread>> {
    SCHEDULERS
        .lock()
        .get(&get_lapic_id())
        .unwrap()
        .current_thread
        .clone()
}

pub fn get_current_process() -> Arc<RwLock<Process>> {
    get_current_thread().read().process.upgrade().unwrap()
}

pub fn get_current_process_id() -> ProcessId {
    get_current_process().read().id
}
