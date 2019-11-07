// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::path::PathBuf;
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use structopt::StructOpt;
use indicatif::{ProgressBar, ProgressStyle};
use crate::opt;
use crate::data::{
    OutputFormat,
    OutputSize,
    Resolution,
};

#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
#[structopt(name = "imager", about = "The imager cli interface.")]
pub struct OptCommand {
    /// Input file paths (supports file globs).
    #[structopt(short, long, required = true, min_values = 1)]
    input: Vec<String>,
    /// Output directory.
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
    /// Output format.
    #[structopt(short, long, default_value = "jpeg")]
    format: OutputFormat,
    /// Output image size (resolution).
    /// 
    /// To target a specific resolution (say 100x100) use `--size 100x100`.
    /// This will always preserve aspect ratio and only downscales when necessary.
    /// 
    /// To preserve the original resolution use `--size full`.
    #[structopt(short, long, default_value = "full")]
    size: OutputSize,
}


impl OptCommand {
    pub fn run(&self) {
        let inputs = self.input
            .clone()
            .into_iter()
            .filter_map(|x| glob::glob(&x).ok())
            .map(|x| x.collect::<Vec<_>>())
            .flatten()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        let to_out_path = |input_path: &PathBuf| -> PathBuf {
            let filename = input_path
                .file_name()
                .expect("file name from path")
                .to_str()
                .expect("str path");
            let mut output_path = self.output.clone();
            std::fs::create_dir_all(&output_path)
                .expect("create output dir if missing");
            output_path.push(&filename);
            match self.format {
                OutputFormat::Jpeg => output_path.set_extension("jpeg")
            };
            output_path
        };
        let progress_bar = ProgressBar::new(inputs.len() as u64);
        progress_bar.tick();
        inputs
            .par_iter()
            .for_each(|input_path| {
                let resize = self.size.clone();
                let source = opt::Source::open(input_path, resize).expect("load source");
                let (output, opt_meta) = source.run_search();
                let output_path = to_out_path(input_path);
                std::fs::write(&output_path, output).expect("write output file");
                progress_bar.inc(1);
            });
        progress_bar.finish();
    }
}
