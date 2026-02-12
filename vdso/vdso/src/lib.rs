#![no_std]

macro_rules! gen_syscall {
    ($(fn $name: ident ( $id: expr ) ( $( $arg: ident : $ty: ty ),* ) $(-> $ret: ty)? ; )+) => {
        unsafe extern "C" {
            $(pub fn $name( $( $arg : $ty ),* ) $(-> $ret)?;)+
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/syscalls.rs"));

unsafe extern "C" {
    pub fn syscall() -> usize;
}
