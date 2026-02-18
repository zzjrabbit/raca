use errors::{Errno, Error, Result};
use protocol::{ReadBuffer, WriteBuffer};

mod r#impl;

macro_rules! gen_syscall {
    ($(fn $name: ident ( $id: expr ) ( $( $arg: ident : $ty: ty ),* ); )+) => {
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
    fn sys_new_channel (2usize) (handle0_ptr: *mut u32, handle1_ptr: *mut u32);
    fn sys_read_channel (3usize) (handle: u32, data_buffer: *const ReadBuffer, handle_buffer: *const ReadBuffer);
    fn sys_write_channel (4usize) (handle: u32, data_buffer: *const WriteBuffer, handle_buffer: *const WriteBuffer);
    fn sys_allocate_vmar (5usize) (handle: u32, size: usize, child_handle: *mut u32);
    fn sys_allocate_vmar_at (6usize) (handle: u32, address: usize, size: usize, child_handle: *mut u32);
    fn sys_map_vmar (7usize) (handle: u32, offset: usize, vmo_handle: u32, flags: u32);
    fn sys_unmap_vmar (8usize) (handle: u32, addr: usize, size: usize);
    fn sys_protect_vmar (9usize) (handle: u32, addr: usize, size: usize, flags: u32);
    fn sys_allocate_vmo (10usize) (count: usize, handle: *mut u32);
}
