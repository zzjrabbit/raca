use alloc::sync::{Arc, Weak};
pub use process::*;
use spin::Once;
pub use thread::*;

mod exception;
mod process;
mod thread;

static IDLE: Once<Arc<Thread>> = Once::new();

pub fn init() {
    exception::init();

    let idle = Thread::new(Weak::new());
    idle.start(|| {});
    IDLE.call_once(|| idle.clone());
}
