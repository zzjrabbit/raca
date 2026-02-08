#![no_std]

#[cfg(feature = "libos")]
unsafe extern "C" {
    static SYSCALL_ENTRY: extern "C" fn();
}

macro_rules! gen_syscall {
    ($(fn $name: ident ( $id: expr ) ( $( $arg: ident : $ty: ty ),* ) $(-> $ret: ty)? ; )+) => {
        $(
            #[cfg(feature = "libos")]
            #[unsafe(no_mangle)]
            pub extern "C" fn $name( $( $arg : $ty ),* ) $(-> $ret)? {
                unsafe {
                    SYSCALL_ENTRY();
                }
            }

            #[cfg(not(feature = "libos"))]
            #[unsafe(no_mangle)]
            pub extern "C" fn $name( $( $arg : $ty ),* ) $(-> $ret)? {}
        )+
    };
}

include!(concat!(env!("OUT_DIR"), "/syscalls.rs"));

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
