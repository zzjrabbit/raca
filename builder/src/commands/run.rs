use std::process::{Command, Stdio};

use anyhow::Result;
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

use crate::{RunArgs, StorageDevice, commands::do_build};

pub fn do_run(args: RunArgs) -> Result<()> {
    let RunArgs {
        build_args,
        whpx,
        cores,
        serial,
        storage,
        debug,
    } = args;
    let arch = build_args.arch.clone();

    let (kernel_path, img_path) = do_build(build_args)?;
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
    if debug {
        cmd.arg("-s").arg("-S");
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

    if !debug {
        cmd.spawn()?.wait()?.exit_ok()?;
    } else {
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        let mut qemu = cmd.spawn()?;

        let mut lldb = Command::new("rust-lldb");
        lldb.arg(kernel_path.to_str().unwrap());
        lldb.arg("--one-line")
            .arg(&format!("gdb-remote localhost:1234"));
        let mut lldb = lldb.spawn()?;

        lldb.wait()?.exit_ok()?;
        qemu.kill()?;
    }
    Ok(())
}
