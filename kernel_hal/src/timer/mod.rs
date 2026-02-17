use alloc::{boxed::Box, vec::Vec};
use spin::Mutex;

use crate::arch::trap::TrapFrame;

type TimerFn = Box<dyn Fn() + Sync + Send>;

static TIMER_CALLBACKS: Mutex<Vec<TimerFn>> = Mutex::new(Vec::new());

pub fn register_callback_on_cpu<F>(func: F)
where
    F: Fn() + Sync + Send + 'static,
{
    let mut callbacks = TIMER_CALLBACKS.lock();
    callbacks.push(Box::new(func));
}

#[allow(unused)]
pub(crate) fn call_timer_callback_functions(_: &TrapFrame) {
    let callbacks = TIMER_CALLBACKS.lock();
    for callback in callbacks.iter() {
        (callback)();
    }
}
