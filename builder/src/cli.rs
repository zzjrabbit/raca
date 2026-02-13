use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

use crate::{RunArgs, StorageDevice, cargo::CargoOpts, image};

fn target_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
}

pub fn do_run(args: RunArgs) -> Result<()> {
    let target_dir = target_dir();

    let RunArgs {
        whpx,
        cores,
        serial,
        arch,
        storage,
        release,
    } = args;

    let vdso_dylib_path = build_vdso(&target_dir, &arch, release, false)?;
    let user_boot_path = build_user_boot(&target_dir, &vdso_dylib_path, &arch, release)?;

    let kernel_target = format!("{}-unknown-none", arch);
    let mut kernel = CargoOpts::new("kernel".into());
    kernel.target(kernel_target.clone());
    if release {
        kernel.release();
    }
    kernel.env("VDSO_DYLIB_PATH", vdso_dylib_path.to_str().unwrap());
    kernel.env("USER_BOOT_PATH", user_boot_path.to_str().unwrap());
    kernel.done();
    let kernel_path = target_dir
        .join(kernel_target)
        .join(if release { "release" } else { "debug" })
        .join("kernel");

    let img_path = image::build(&kernel_path)?;
    println!("Image path: {img_path:?}");

    let mut cmd = Command::new(format!("qemu-system-{}", arch));
    cmd.arg("-machine").arg("virt");
    cmd.arg("-m").arg("512m");
    cmd.arg("-smp").arg(format!("cores={}", cores));
    cmd.arg("-cpu").arg("la464");

    if whpx {
        cmd.arg("-accel").arg("whpx");
    }
    if serial {
        cmd.arg("-serial").arg("stdio");
    }

    cmd.arg("-device").arg("qemu-xhci,id=xhci");
    cmd.args(["-device", "usb-kbd", "-device", "usb-mouse"]);

    if let Some(backend) = match std::env::consts::OS {
        "linux" => Some("pa"),
        "macos" => Some("coreaudio"),
        "windows" => Some("dsound"),
        _ => None,
    } {
        cmd.arg("-audiodev").arg(format!("{backend},id=sound"));
        cmd.arg("-device").arg("intel-hda");
        cmd.arg("-device").arg("hda-output,audiodev=sound");
    }

    match storage {
        StorageDevice::Ahci => {
            cmd.arg("-device").arg("ahci,id=ahci");
            cmd.arg("-device").arg("ide-hd,drive=disk,bus=ahci.0");
        }
        StorageDevice::Nvme => {
            cmd.arg("-device").arg("nvme,drive=disk,serial=deadbeef");
        }
        StorageDevice::Virtio => {
            cmd.arg("-device").arg("virtio-blk-pci,drive=disk");
        }
    }

    let param = "if=none,format=raw,id=disk";
    cmd.args(["-drive", &format!("{param},file={}", img_path.display())]);

    let param = "if=pflash,format=raw";
    let ovmf_path = Prebuilt::fetch(Source::LATEST, "target/ovmf")
        .expect("failed to update prebuilt")
        .get_file(Arch::LoongArch64, FileType::Code);
    cmd.args(["-drive", &format!("{param},file={}", ovmf_path.display())]);

    cmd.args(["-device", "ramfb"]);

    cmd.spawn()?.wait()?.exit_ok()?;
    Ok(())
}

pub fn do_test() -> Result<()> {
    let target_dir = target_dir();
    let arch = "x86_64";

    let vdso_dylib_path = build_vdso(&target_dir, arch, false, true)?;

    let mut object = CargoOpts::new("object".into());
    object.action("test");
    object.env("VDSO_DYLIB_PATH", vdso_dylib_path.to_str().unwrap());
    object.feature("libos");
    object.done();

    let mut kernel_hal = CargoOpts::new("kernel_hal".into());
    kernel_hal.action("test");
    kernel_hal.feature("libos");
    kernel_hal.done();

    Ok(())
}

pub fn do_clippy() -> Result<()> {
    let target_dir = target_dir();
    let arch = "x86_64";

    let vdso_dylib_path = build_vdso(&target_dir, arch, false, true)?;
    let user_boot_path = build_user_boot(&target_dir, &vdso_dylib_path, arch, false)?;

    let run_clippy = |kcrate: CargoOpts| {
        let mut crate1 = kcrate.clone();
        crate1.target("loongarch64-unknown-none".into());
        crate1.done();

        let mut crate2 = kcrate.clone();
        crate2.feature("libos");
        crate2.done();
    };

    let mut object = CargoOpts::new("object".into());
    object.action("clippy");
    object.env("VDSO_DYLIB_PATH", vdso_dylib_path.to_str().unwrap());
    run_clippy(object);

    let mut kernel_hal = CargoOpts::new("kernel_hal".into());
    kernel_hal.action("clippy");
    run_clippy(kernel_hal);

    let mut kernel = CargoOpts::new("kernel".into());
    kernel.action("clippy");
    kernel.target("loongarch64-unknown-none".into());
    kernel.env("VDSO_DYLIB_PATH", vdso_dylib_path.to_str().unwrap());
    kernel.env("USER_BOOT_PATH", user_boot_path.to_str().unwrap());
    kernel.done();

    Ok(())
}

fn user_target(arch: &str) -> &str {
    match arch {
        "x86_64" => "x86_64-unknown-linux-none",
        "loongarch64" => "loongarch64-unknown-linux-musl",
        _ => unimplemented!(),
    }
}

fn build_vdso(target_dir: &Path, arch: &str, release: bool, libos: bool) -> Result<PathBuf> {
    let user_target = user_target(arch);

    let mut vdso_dylib = CargoOpts::new("vdso_dylib".into());
    vdso_dylib.build_std();
    vdso_dylib.target(user_target.into());

    if libos {
        vdso_dylib.feature("libos");
    }
    if release {
        vdso_dylib.release();
    }

    vdso_dylib.done();

    Ok(target_dir
        .join(user_target)
        .join(if release { "release" } else { "debug" })
        .join("libvdso_dylib.so"))
}

fn build_user_boot(
    target_dir: &Path,
    vdso_dylib_path: &Path,
    arch: &str,
    release: bool,
) -> Result<PathBuf> {
    let user_target = user_target(arch);

    let mut user_boot = CargoOpts::new("user_boot".into());
    user_boot.build_std();
    user_boot.target(user_target.into());

    if release {
        user_boot.release();
    }

    user_boot.env("VDSO_DYLIB_PATH", vdso_dylib_path.to_str().unwrap());

    user_boot.done();

    Ok(target_dir
        .join(user_target)
        .join(if release { "release" } else { "debug" })
        .join("user_boot"))
}
