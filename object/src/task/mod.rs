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
    idle.start(|| {
        #[cfg(feature = "libos")]
        {
            extern crate std;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        #[cfg(not(feature = "libos"))]
        {
            kernel_hal::platform::idle_ins();
        }
    });
    IDLE.call_once(|| idle.clone());
}
