use std::{
    pin::Pin,
    sync::{Arc, LazyLock, Mutex},
    task::{Context, Poll},
};

use tokio::runtime::Runtime;

pub use crate::arch::task::syscall_fn_entry;
use crate::task::ThreadState;

static TOKIO_RT: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

#[derive(Default)]
pub struct HwThread {
    state: Mutex<ThreadState>,
}

impl HwThread {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn spawn(self: &Arc<Self>, mut f: impl FnMut() + Send + 'static) {
        let ctx = self.clone();
        TOKIO_RT.spawn(async move {
            Box::pin(ThreadFuture::new(ctx.clone())).await;
            ctx.set_state(ThreadState::Running);
            f();
            if ctx.state().running() {
                ctx.set_state(ThreadState::Ready);
            }
        });
    }

    pub fn state(&self) -> ThreadState {
        *self.state.lock().unwrap()
    }

    pub fn set_state(&self, state: ThreadState) {
        *self.state.lock().unwrap() = state;
    }
}

struct ThreadFuture {
    ctx: Arc<HwThread>,
}

impl ThreadFuture {
    fn new(ctx: Arc<HwThread>) -> Self {
        Self { ctx }
    }
}

impl Future for ThreadFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.ctx.state().can_run() {
            true => Poll::Ready(()),
            false => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::*;

    #[test]
    fn test_task_context() {
        let ctx = Arc::new(HwThread::new());

        ctx.spawn(|| {
            println!("Task run");
        });

        sleep(Duration::from_millis(100));
        ctx.set_state(ThreadState::Blocked);
    }
}
