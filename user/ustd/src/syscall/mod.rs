mod r#impl;

macro_rules! gen_syscall {
    ($(fn $name: ident ( $id: expr ) ( $( $arg: ident : $ty: ty ),* ); )+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name( $( $arg : $ty ),* ) -> usize {
                crate::do_syscall!($id $( ,$arg )*)
            }
        )+
    };
}

gen_syscall! {
    fn sys_debug (0usize) (ptr: *const u8, len: usize);
    fn sys_empty (1usize) ();
}
