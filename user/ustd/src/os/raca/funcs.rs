extern crate compiler_builtins;

#[unsafe(no_mangle)]
unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe { compiler_builtins::mem::memcpy(dst, src, n) }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memcmp(lhs: *const u8, rhs: *const u8, n: usize) -> i32 {
    unsafe { compiler_builtins::mem::memcmp(lhs, rhs, n) }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memset(dst: *mut u8, c: i32, n: usize) -> *mut u8 {
    unsafe { compiler_builtins::mem::memset(dst, c, n) }
}
