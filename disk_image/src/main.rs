use std::{
    convert::TryFrom,
    fs::{self, File},
    io::{self, Seek},
    path::{Path, PathBuf},
};
use flate2::{Compression, GzBuilder};

fn main() {
    // take efi file path as command line argument
    let mut args = std::env::args();
    let _exe_name = args.next().unwrap();
    let efi_path = PathBuf::from(
        args.next()
            .expect("path to `.efi` files must be given as argument"),
    );

    let kernel_path = PathBuf::from(
        args.next()
            .expect("path to kernel file must be given as argument"),
    );

    let compressed_kernel_path = PathBuf::from(
        "core.sys"
    );

    let mut compressed_kernel = GzBuilder::new().read(std::fs::File::open(kernel_path.clone()).unwrap(), Compression::best());
    
    
    io::copy(&mut compressed_kernel, &mut std::fs::File::create(compressed_kernel_path.clone()).unwrap()).unwrap();

    let config_path = PathBuf::from(
        args.next()
            .expect("path to config file must be given as argument"),
    );

    let fat_path = PathBuf::from("boot_img.fat");
    let disk_path = PathBuf::from("boot_img.img");

    create_fat_filesystem(&fat_path, &efi_path, &compressed_kernel_path,  &config_path);
    create_gpt_disk(&disk_path, &fat_path);
}

fn create_fat_filesystem(
    fat_path: &Path,
    efi_file: &Path,
    kernel_file: &Path,
    config_file: &Path,
) {
    // retrieve size of `.efi` file and round it up
    let efi_size = fs::metadata(&efi_file).unwrap().len();
    let kernel_size = fs::metadata(&kernel_file).unwrap().len();
    let config_size = fs::metadata(&config_file).unwrap().len();
    let mb = 1024 * 1024; // size of a megabyte
                          // round it to next megabyte
    let efi_size_rounded = ((efi_size + kernel_size + config_size - 4) / mb + 1) * mb;

    // create new filesystem image file at the given path and set its length
    let fat_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&fat_path)
        .unwrap();
    fat_file.set_len(efi_size_rounded).unwrap();

    // create new FAT file system and open it
    let format_options = fatfs::FormatVolumeOptions::new();
    fatfs::format_volume(&fat_file, format_options).unwrap();
    let filesystem = fatfs::FileSystem::new(&fat_file, fatfs::FsOptions::new()).unwrap();

    // copy EFI file to FAT filesystem
    let root_dir = filesystem.root_dir();
    root_dir.create_dir("efi").unwrap();
    root_dir.create_dir("efi/boot").unwrap();
    root_dir.create_dir("RACA").unwrap();
    root_dir.create_dir("RACA/system64").unwrap();

    let mut bootx64 = root_dir.create_file("efi/boot/bootx64.efi").unwrap();
    bootx64.truncate().unwrap();
    io::copy(&mut fs::File::open(&efi_file).unwrap(), &mut bootx64).unwrap();

    let mut config = root_dir.create_file("limine.cfg").unwrap();
    config.truncate().unwrap();
    io::copy(&mut fs::File::open(&config_file).unwrap(), &mut config).unwrap();

    let mut kernel = root_dir.create_file("RACA/system64/core.sys").unwrap();
    kernel.truncate().unwrap();
    io::copy(&mut fs::File::open(&kernel_file).unwrap(), &mut kernel).unwrap();
}

fn create_gpt_disk(disk_path: &Path, fat_image: &Path) {
    // create new file
    let mut disk = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&disk_path)
        .unwrap();

    // set file size
    let partition_size: u64 = fs::metadata(&fat_image).unwrap().len();
    let disk_size = partition_size + 1024 * 64; // for GPT headers
    disk.set_len(disk_size).unwrap();

    // create a protective MBR at LBA0 so that disk is not considered
    // unformatted on BIOS systems
    let mbr = gpt::mbr::ProtectiveMBR::with_lb_size(
        u32::try_from((disk_size / 512) - 1).unwrap_or(0xFF_FF_FF_FF),
    );
    mbr.overwrite_lba0(&mut disk).unwrap();

    // create new GPT structure
    let block_size = gpt::disk::LogicalBlockSize::Lb512;
    let mut gpt = gpt::GptConfig::new()
        .writable(true)
        .initialized(false)
        .logical_block_size(block_size)
        .create_from_device(Box::new(&mut disk), None)
        .unwrap();
    gpt.update_partitions(Default::default()).unwrap();

    // add new EFI system partition and get its byte offset in the file
    let partition_id = gpt
        .add_partition("boot", partition_size, gpt::partition_types::EFI, 0)
        .unwrap();
    let partition = gpt.partitions().get(&partition_id).unwrap();
    let start_offset = partition.bytes_start(block_size).unwrap();

    // close the GPT structure and write out changes
    gpt.write().unwrap();

    // place the FAT filesystem in the newly created partition
    disk.seek(io::SeekFrom::Start(start_offset)).unwrap();
    io::copy(&mut File::open(&fat_image).unwrap(), &mut disk).unwrap();
}
