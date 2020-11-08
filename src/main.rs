#[macro_use]
extern crate bitfield;


use std::ffi::OsStr;
use std::fs::File;
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
    size: u32,
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
        CLICommands::New(s) => {
            create_fs(s)
        }
    }
}

fn create_fs(fs: CLINew) {
    // size of the fs in KB
    let kb_size = fs.size * 1024 * 1024;
    let inode_count = kb_size;
    let block_count = kb_size/4;
    let sfs = fs::SFS::new(inode_count, block_count);
    let mut file = File::create(fs.image).expect("Failed to open file");
    sfs.dump(&mut file).expect("failed to write file");
}
