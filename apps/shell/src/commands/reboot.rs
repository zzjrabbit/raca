use alloc::{string::String, vec::Vec};

pub fn reboot(args: Vec<String>) {
    if args.len() != 0 {
        raca_std::println!("Usage: reboot");
        return;
    }

    raca_std::kernel::reboot();
}
