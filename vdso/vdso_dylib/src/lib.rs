#![no_std]

mod regs;

macro_rules! gen_syscall {
    ($(fn $name: ident ( $id: expr ) ( $( $arg: ident : $ty: ty ),* ); )+) => {
        $(
            #[cfg(feature = "libos")]
            #[unsafe(no_mangle)]
            pub extern "C" fn $name( $( $arg : $ty ),* ) -> usize {
                crate::do_syscall!($id $( ,$arg )*)
            }

            #[cfg(not(feature = "libos"))]
            #[unsafe(no_mangle)]
            pub extern "C" fn $name( $( $arg : $ty ),* ) -> usize {
                0
            }
        )+
    };
}

include!(concat!(env!("OUT_DIR"), "/syscalls.rs"));

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
