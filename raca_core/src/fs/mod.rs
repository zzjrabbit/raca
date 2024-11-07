pub mod cache;
pub mod dev;
pub mod ramfs;
pub mod vfs;

use core::{ffi::CStr, slice::from_raw_parts};

use limine::request::ModuleRequest;
use ramfs::create_ramfs_from_cpio;
use spin::Lazy;
pub use vfs::*;

const fn create_string<const N: usize>(s: &[u8; N]) -> [u8; N + 1] {
    let mut res = [0; N + 1];
    let mut i = 0;
    while i < N {
        res[i] = s[i];
        i += 1;
    }
    res[N] = 0;
    res
}

static INITRAMFS_MODULE: limine::modules::InternalModule = limine::modules::InternalModule::new()
    .with_path(unsafe { CStr::from_bytes_with_nul_unchecked(&create_string(b"/boot/initramfs")) });

#[used]
#[link_section = ".requests"]
static INITRAMFS: ModuleRequest = ModuleRequest::new().with_internal_modules(&[&INITRAMFS_MODULE]);

pub static ROOT: Lazy<FileRef> = Lazy::new(|| {
    let initramfs = INITRAMFS.get_response().unwrap().modules()[0];
    let address = initramfs.addr();
    let len = initramfs.size();

    let initramfs = unsafe { from_raw_parts(address, len as usize) };

    log::info!(
        "Initramfs loaded, address: {:x}, len: {:x}",
        initramfs.as_ptr() as u64,
        initramfs.len()
    );

    create_ramfs_from_cpio::<1024>(Path::new("/"), initramfs)
});

pub fn init() {

    log::info!("root children: {:#?}",ROOT.read().get_children());

    log::info!(
        "root path: {} haha children: {:#?}",
        ROOT.read().get_file_path(),
        ROOT.read().get_child("haha").unwrap().read().get_children(),
    );

    let mut data = [0; 50];
    ROOT.read()
        .get_child("haha")
        .unwrap()
        .read().get_child("test.txt").unwrap().read().read_at(0, &mut data);

    log::info!("test.txt : {}", core::str::from_utf8(&data).unwrap())
}
