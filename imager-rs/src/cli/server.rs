// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::path::PathBuf;
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use structopt::StructOpt;
use crate::server;

#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
pub struct ServerCommand {
    #[structopt(short, long)]
    address: String,
}

impl ServerCommand {
    pub fn run(&self) {
        server::run(&self.address);
    }
}



