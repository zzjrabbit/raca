use core::alloc::Layout;

use syscall_macro::syscall;
use talc::{OomHandler, Span, Talc, Talck};

pub fn malloc(address: usize, len: usize) {
    const MALLOC_SYSCALL_ID: u64 = 2;

    let _ = syscall!(MALLOC_SYSCALL_ID, address, len);
}

const HEAP_START: usize = 0x19198100000;
const ONCE_ALLOCATION_SIZE: usize = 128 * 1024;

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, OomHandlerImpl> =
    Talc::new(OomHandlerImpl::default()).lock();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Kernel heap allocation error: {:?}", layout)
}

struct OomHandlerImpl(Span);

impl OomHandlerImpl {
    const fn default() -> Self {
        OomHandlerImpl(Span::from_base_size(HEAP_START as *mut u8, 0))
    }
}

impl OomHandler for OomHandlerImpl {
    fn handle_oom(talc: &mut Talc<Self>, _layout: Layout) -> Result<(), ()> {
        let current_heap = talc.oom_handler.0;

        if current_heap.is_empty() {
            malloc(HEAP_START, ONCE_ALLOCATION_SIZE);
            let new_heap = Span::from_base_size(HEAP_START as *mut u8, ONCE_ALLOCATION_SIZE);
            unsafe { talc.claim(new_heap).unwrap() };
            talc.oom_handler.0 = new_heap;
        } else {
            let (_, current_end) = current_heap.get_base_acme().unwrap();
            malloc(current_end as usize, ONCE_ALLOCATION_SIZE);
            let new_heap = current_heap.extend(0, ONCE_ALLOCATION_SIZE);
            talc.oom_handler.0 = unsafe { talc.extend(current_heap, new_heap) };
        }

        Ok(())
    }
}
