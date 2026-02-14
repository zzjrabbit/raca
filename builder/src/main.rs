#![feature(exit_status_error)]

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::commands::*;

mod cargo;
mod commands;
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
    /// Build the kernel.
    Build(BuildArgs),
    /// Run the kernel.
    Run(RunArgs),
    /// Test object and kernel_hal
    Test,
    /// Run cargo clippy for object.
    Clippy,
}

#[derive(Args)]
struct BuildArgs {
    /// Build with release
    #[clap(long)]
    release: bool,

    /// Set target arch.
    #[clap(long)]
    #[clap(default_value = "loongarch64")]
    arch: String,
}

#[derive(Args)]
struct RunArgs {
    #[command(flatten)]
    build_args: BuildArgs,
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

    #[clap(short, long)]
    #[clap(default_value_t = false)]
    debug: bool,

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
        SubCommands::Build(args) => do_build(args).map(|_| ()),
        SubCommands::Run(args) => do_run(args),
        SubCommands::Test => do_test(),
        SubCommands::Clippy => do_clippy(),
    }
}
