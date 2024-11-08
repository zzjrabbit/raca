#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use alloc::vec;
use limine::BaseRevision;
use raca_core::{fs::ROOT, task::Process};

#[used]
#[link_section = ".requests"]
pub static BASE_REVISION: BaseRevision = BaseRevision::with_revision(1);

#[no_mangle]
pub extern "C" fn main() -> ! {
    raca_core::init();

    let bin_folder = ROOT.read().get_child("bin").unwrap();
    let init_file = bin_folder.read().get_child("init.rae").unwrap();

    let mut data = vec![0; init_file.read().len()];

    init_file.read().read_at(0, &mut data);
    Process::new_user_process("init", data.leak());

    log::info!("racaOS v{} init program started!",core::env!("CARGO_PKG_VERSION"));

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
