use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use argh::FromArgs;
use cpio::newc::ModeFileType;
use cpio::{write_cpio, NewcBuilder};

mod image_builder;

#[derive(FromArgs)]
#[argh(description = "racaOS bootloader and kernel builder")]
struct Args {
    #[argh(switch, short = 'b')]
    #[argh(description = "boot the constructed image")]
    boot: bool,

    #[argh(switch, short = 'k')]
    #[argh(description = "use KVM acceleration")]
    kvm: bool,

    #[argh(switch, short = 'h')]
    #[argh(description = "use HAXM acceleration")]
    haxm: bool,

    #[argh(option, short = 'c')]
    #[argh(default = "2")]
    #[argh(description = "number of CPU cores")]
    cores: usize,

    #[argh(switch, short = 's')]
    #[argh(description = "redirect serial to stdio")]
    serial: bool,
}

fn build_module(name: &str, images_path: PathBuf, module_name: Option<String>) {
    let mut cmd = Command::new("cargo");
    cmd.current_dir("modules");
    cmd.arg("build");
    cmd.arg("--package").arg(name);
    cmd.arg("--release");
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();

    let module_path = PathBuf::from("target/target/release/".to_string() + name);
    let mut module_src = File::open(module_path).unwrap();
    let name = if let Some(name) = module_name {
        name
    } else {
        name.to_string()
    };
    let mut module_dest = File::create(images_path.join(name + ".km")).unwrap();

    io::copy(&mut module_src, &mut module_dest).unwrap();
}

fn build_user_program(name: &str, images_path: PathBuf, user_program_name: Option<String>) {
    let mut cmd = Command::new("cargo");
    cmd.current_dir("apps");
    cmd.arg("build");
    cmd.arg("--package").arg(name);
    cmd.arg("--release");
    cmd.arg("--target").arg("x86_64-unknown-none");
    
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();

    let module_path = PathBuf::from("target/x86_64-unknown-none/release/".to_string() + name);
    let mut module_src = File::open(module_path).unwrap();
    let name = if let Some(name) = user_program_name {
        name
    } else {
        name.to_string()
    };
    let mut module_dest = File::create(images_path.join("bin").join(name + ".rae")).unwrap();

    io::copy(&mut module_src, &mut module_dest).unwrap();
}

fn build_initramfs() {
    let mut initramfs_file = File::create("esp/boot/initramfs").unwrap();
    let mut inputs = Vec::new();

    for entry in walkdir::WalkDir::new("initramfs") {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                let mut path = entry.path().to_str().unwrap().to_string();
                for _ in 0..("initramfs".len() + 1) {
                    path.remove(0);
                }

                #[cfg(target_os = "windows")]
                let path = path.replace("\\", "/");

                println!("{} {}", path, entry.path().to_path_buf().display());

                let path_out = entry.path().to_path_buf().clone();

                inputs.push((path, path_out));
            }
        }
    }

    write_cpio(
        inputs.iter().map(|(path_in, path)| {
            (
                NewcBuilder::new(path_in).set_mode_file_type(ModeFileType::Regular),
                File::open(path).unwrap(),
            )
        }),
        &mut initramfs_file,
    )
    .unwrap();
}

fn build_image_from_dir(dir: &str, image_file: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let img_path = manifest_dir.parent().unwrap().join(image_file);

    let mut files = BTreeMap::new();

    for entry in walkdir::WalkDir::new(dir) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                let mut path = entry.path().to_str().unwrap().to_string();
                for _ in 0..(dir.len() + 1) {
                    path.remove(0);
                }
                let path = path.replace("\\", "/");

                files.insert(path.clone(), entry.path().to_path_buf());
            }
        }
    }

    image_builder::ImageBuilder::build(files, &img_path).unwrap();
}

fn main() {
    let mut config_file = File::open("config.toml").unwrap();
    let mut config_data = String::new();
    config_file.read_to_string(&mut config_data).unwrap();
    let config = toml::Table::from_str(&config_data).unwrap();

    let esp_config = config.get("esp").unwrap().as_table().unwrap();
    let initramfs_config = config.get("initramfs").unwrap().as_table().unwrap();

    let raca_core_path = PathBuf::from(env!("CARGO_BIN_FILE_RACA_CORE_raca_core"));
    println!("RacaCore Path: {}", raca_core_path.display());
    let mut raca_core_src = File::open(raca_core_path).unwrap();

    let images_path = PathBuf::from("esp");

    let mut raca_core_dest = File::create(images_path.join("core.so")).unwrap();

    io::copy(&mut raca_core_src, &mut raca_core_dest).unwrap();

    for module in esp_config.get("modules").unwrap().as_array().unwrap() {
        build_module(
            module.as_str().unwrap(),
            images_path.clone().join("modules"),
            None,
        );
    }

    let initramfs_path = PathBuf::from("initramfs");

    for module in initramfs_config.get("modules").unwrap().as_array().unwrap() {
        build_module(module.as_str().unwrap(), initramfs_path.clone(), None);
    }

    for user_program in initramfs_config.get("users").unwrap().as_array().unwrap() {
        let name = user_program.as_str().unwrap();
        build_user_program(name, initramfs_path.clone(), None);
    }

    let init_name = initramfs_config.get("init").unwrap().as_str().unwrap();
    build_user_program(init_name, initramfs_path, Some("init".into()));

    //build_image_from_dir("initrd", "esp/initrd");
    build_initramfs();

    build_image_from_dir("esp", "racaOS.img");

    let args: Args = argh::from_env();

    if args.boot {
        let mut cmd = Command::new("qemu-system-x86_64");
        let drive_config = format!("format=raw,file=racaOS.img,if=none,id=boot_disk",);

        cmd.arg("-device").arg("ahci,id=ahci");
        cmd.arg("-machine").arg("q35");
        cmd.arg("-m").arg("4g");
        cmd.arg("-pflash").arg("ovmf/x86_64.fd");
        cmd.arg("-drive").arg(drive_config);
        cmd.arg("-device").arg("ide-hd,drive=boot_disk,bus=ahci.0");
        cmd.arg("-smp").arg(format!("cores={}", args.cores));
        cmd.arg("-cpu").arg("qemu64,+x2apic");
        cmd.arg("-usb");
        cmd.arg("-device").arg("qemu-xhci,id=xhci");
        /*cmd.arg("-drive")
            .arg("format=raw,file=disk.img,if=none,id=disk1");
        cmd.arg("-device").arg("ide-hd,drive=disk1,bus=ahci.2");
        cmd.arg("-drive")
            .arg("format=raw,file=data.img,if=none,id=disk2");
        cmd.arg("-device").arg("nvme,drive=disk2,serial=1234");*/
        cmd.arg("-net").arg("nic");

        if args.kvm {
            cmd.arg("--enable-kvm");
        }
        if args.haxm {
            cmd.arg("-accel").arg("hax");
        }
        if args.serial {
            cmd.arg("-serial").arg("stdio");
        }

        let mut child = cmd.spawn().unwrap();
        child.wait().unwrap();
    }
}
