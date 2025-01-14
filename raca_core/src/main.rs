#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use core::panic::PanicInfo;
use limine::BaseRevision;
use raca_core::{
    fs::{Path, ROOT, operation::kernel_open},
    task::Process,
};

#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::with_revision(1);

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    raca_core::init();

    let bin_folder = ROOT.read().get_child("bin").unwrap();
    let init_file = bin_folder.read().get_child("init").unwrap();

    let mut data = vec![0; init_file.read().len()];

    init_file.read().read_at(0, &mut data);

    let terminal = kernel_open(Path::new("/dev/terminal")).unwrap();

    Process::new_user_process("init", data.leak(), terminal.clone(), terminal.clone()).unwrap();

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
