#![allow(unused)]
pub mod tool;
pub mod utils;
pub mod opt;
pub mod cli;
pub mod data;

use structopt::StructOpt;
use cli::Command;

fn main() {
    let cmd = Command::from_args();
    cmd.run();
}
