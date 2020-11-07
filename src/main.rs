#[macro_use]
extern crate bitfield;


use std::ffi::OsStr;
use std::path::PathBuf;
use structopt::StructOpt;

mod fs;
use fs::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "sfs", about = "a tool for creating sfs images and fuse mounting them")]
struct CLIOptions {
    #[structopt(subcommand)]
    command: CLICommands,
}

#[derive(Debug, StructOpt)]
enum CLICommands {
    New(CLINew),
    Mount(CLIMount),
}

#[derive(Debug, StructOpt)]
struct CLINew {
    #[structopt(parse(from_os_str))]
    image: PathBuf,
    size: String,
}

#[derive(Debug, StructOpt)]
struct CLIMount {
    #[structopt(parse(from_os_str))]
    image: PathBuf,

    mount_point: String,
}

fn main() {
    env_logger::init();

    let opt = CLIOptions::from_args();

    match opt.command {
        CLICommands::Mount(s) => {
            let options = ["-o", "ro", "-o", "fsname=sfs"]
                .iter()
                .map(|o| o.as_ref())
                .collect::<Vec<&OsStr>>();
            fuse::mount(SFS::default(), &s.mount_point, &options).unwrap();
        },
        _ => panic!("image creation not ready"),
    }
}
