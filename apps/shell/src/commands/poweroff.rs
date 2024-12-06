use alloc::{string::String, vec::Vec};

pub fn poweroff(args: Vec<String>) {
    if args.len() != 0 {
        raca_std::println!("Usage: poweroff");
        return;
    }

    raca_std::kernel::poweroff();
}
