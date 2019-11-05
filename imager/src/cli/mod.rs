// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
pub mod opt;

use serde::{Serialize, Deserialize};
use structopt::StructOpt;


/// The imager CLI interface.
/// 
/// Currently just supports the optimization sub-command.
/// See `imager opt --help` for details.
#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
#[structopt(name = "imager", rename_all = "kebab-case")]
pub enum Command {
    /// Optimize the given media for distribution on the web.
    /// 
    /// Performs a brute force ‘rate control’ search using ML based metrics;
    /// essentially does what you should otherwise be manually doing for
    /// media distribution on the web.
    /// 
    /// E.g. `imager opt -i assets/**/*.jpeg -o assets/output/ -s 900x900`.
    Opt(opt::Command),
}

impl Command {
    pub fn run(&self) {
        match self {
            Command::Opt(cmd) => cmd.run(),
        }
    }
}
