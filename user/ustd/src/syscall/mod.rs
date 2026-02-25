use errors::{Errno, Error, Result};
use protocol::{ReadBuffer, WriteBuffer};

mod r#impl;

macro_rules! gen_syscall {
    ($(fn $name: ident ( $id: expr ) ( $( $arg: ident : $ty: ty ),* $(,)? ); )+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe fn $name( $( $arg : $ty ),* ) -> Result<usize> {
                let raw_result: isize = crate::do_syscall!($id $( ,$arg )*);
                if raw_result < 0 {
                    Err(Error::try_from(raw_result as i32).unwrap_or(Errno::InvSyscall.no_message()))
                } else {
                    Ok(raw_result as usize)
                }
            }
        )+
    };
}

gen_syscall! {
    fn sys_debug (0usize) (ptr: *const u8, len: usize);

    fn sys_remove_handle (1usize) (handle: u32);
    fn sys_duplicate_handle (19usize) (handle: u32, new_handle: *mut u32);

    fn sys_new_channel (2usize) (handle0_ptr: *mut u32, handle1_ptr: *mut u32);
    fn sys_read_channel (3usize) (
        handle: u32,
        data_buffer: *const ReadBuffer,
        handle_buffer: *const ReadBuffer,
    );
    fn sys_write_channel (4usize) (
        handle: u32,
        data_buffer: *const WriteBuffer,
        handle_buffer: *const WriteBuffer,
    );

    fn sys_allocate_vmar (5usize) (
        handle: u32,
        size: usize,
        child_handle: *mut u32,
    );
    fn sys_allocate_vmar_at (6usize) (
        handle: u32,
        address: usize,
        size: usize,
        child_handle: *mut u32,
    );

    fn sys_map_vmar (7usize) (
        handle: u32,
        offset: usize,
        vmo_handle: u32,
        flags: u32,
    );
    fn sys_unmap_vmar (8usize) (handle: u32, addr: usize, size: usize);
    fn sys_protect_vmar (9usize) (
        handle: u32,
        addr: usize,
        size: usize,
        flags: u32,
    );

    fn sys_get_vmar_base (22usize) (handle: u32);
    fn sys_get_vmar_size (23usize) (handle: u32);

    fn sys_allocate_vmo (10usize) (count: usize, handle: *mut u32);
    fn sys_read_vmo (20usize) (handle: u32, offset: usize, buffer: *mut u8, size: usize);
    fn sys_write_vmo (21usize) (handle: u32, offset: usize, buffer: *const u8, size: usize);

    fn sys_exit (11usize) (exit_code: i32);
    fn sys_new_process (12usize) (
        handle: *mut u32,
        vmar_handle: *mut u32,
        base: *mut usize,
        size: *mut usize,
    );
    fn sys_start_process (13usize) (
        handle: u32,
        thread: u32,
        boot_handle: u32,
        entry: usize,
        stack: usize,
        start_info_addr: usize,
    );
    fn sys_kill_process (17usize) (process: u32);

    fn sys_new_thread (14usize) (process: u32, handle: *mut u32);
    fn sys_start_thread (15usize) (
        handle: u32,
        entry: usize,
        stack: usize,
        first_arg: usize,
    );
    fn sys_exit_thread (16usize) ();
    fn sys_kill_thread (18usize) (thread: u32);
}
