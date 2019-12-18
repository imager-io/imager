#![allow(unused)]
pub mod server;

use serde::{Serialize, Deserialize};
use structopt::StructOpt;

///////////////////////////////////////////////////////////////////////////////
// CLI FRONTEND
///////////////////////////////////////////////////////////////////////////////

/// The Imager Server Interface
#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
#[structopt(
    name = "imager-server",
    rename_all = "kebab-case"
)]
pub struct Command {
    #[structopt(short, long)]
    address: String,
}

impl Command {
    pub fn run(&self) {
        server::run(&self.address);
    }
}


///////////////////////////////////////////////////////////////////////////////
// MAIN
///////////////////////////////////////////////////////////////////////////////

fn main() {
    let cmd = Command::from_args();
    cmd.run();
}
