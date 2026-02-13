#![feature(exit_status_error)]

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::cli::{do_run, do_test};

mod cargo;
mod cli;
mod image;

/// racaOS kernel builder, tester and runner
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    /// Run the kernel.
    Run(RunArgs),
    /// Test object and kernel_hal
    Test,
}

#[derive(Args)]
struct RunArgs {
    /// use Hyper-V acceleration
    #[clap(short, long)]
    whpx: bool,

    /// number of CPU cores
    #[clap(short, long)]
    #[clap(default_value_t = 4)]
    cores: usize,

    /// redirect serial to stdio
    #[clap(short, long)]
    serial: bool,

    /// Build with release
    #[clap(long)]
    #[clap(default_value_t = false)]
    release: bool,

    /// Set target arch.
    #[clap(long)]
    #[clap(default_value = "loongarch64")]
    arch: String,

    /// boot device
    #[clap(long)]
    #[clap(default_value_t = StorageDevice::Nvme)]
    #[arg(value_enum)]
    storage: StorageDevice,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum StorageDevice {
    #[default]
    Nvme,
    Ahci,
    Virtio,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        SubCommands::Run(args) => do_run(args),
        SubCommands::Test => do_test(),
    }
}
