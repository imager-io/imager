#![allow(unused)]
pub mod classifier;
pub mod codec;
pub mod vmaf;
pub mod data;
pub mod api;

use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use structopt::StructOpt;
use structopt::clap::ArgGroup;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;

use crate::data::{
    OutputFormat,
    OutputFormats,
    Resolution,
};

///////////////////////////////////////////////////////////////////////////////
// CLI FRONTEND - INTERNAL HELPER TYPES
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
enum OutputType {
    Dir(PathBuf),
    File(PathBuf),
    Replace,
}

impl OutputType {
    pub fn is_dir(&self) -> bool {
        match self {
            OutputType::Dir(_) => true,
            _ => false
        }
    }
    pub fn is_file(&self) -> bool {
        match self {
            OutputType::File(_) => true,
            _ => false
        }
    }
    pub fn is_replace(&self) -> bool {
        match self {
            OutputType::Replace => true,
            _ => false
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CLI FRONTEND
///////////////////////////////////////////////////////////////////////////////

/// The Imager CLI Interface
/// 
/// Output type much be one of: `--output-file`, `--output-dir`, or `--replace`.
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "imager",
    // rename_all = "kebab-case",
    group = (ArgGroup::with_name("output_type").required(true)),
)]
pub struct Command {
    /// Input file(s) path.
    #[structopt(short, long, required = true, min_values = 1)]
    inputs: Vec<String>,
    
    /// Save the result to this file path.
    /// 
    /// Save the optimized file to this path.
    /// Only works for single input/output files.
    #[structopt(short="o", long, parse(from_os_str), group = "output_type")]
    output_file: Option<PathBuf>,

    /// Save results under this directory.
    /// 
    /// Dump results to this directory.
    /// Files will have the same name as the input file. 
    /// Valid for multiple input/output files.
    #[structopt(short="O", long, parse(from_os_str), group = "output_type")]
    output_dir: Option<PathBuf>,

    /// Replace input files with their optimized results.
    /// 
    /// Valid for multiple input/output files.
    #[structopt(long, group = "output_type")]
    replace: bool,
    
    /// Output format(s).
    /// 
    /// Multiple output formats may be specified, e.g. `--formats webp jpeg`.
    /// The saved results will have their file extension updated if different
    /// from the original.
    #[structopt(short, long, default_value = "jpeg webp")]
    formats: Vec<OutputFormats>,
    
    /// Resize or downscale images if their resolution exceeds the given size.
    #[structopt(long)]
    max_size: Option<Resolution>,

    /// Internal. No stability guarantees.
    #[structopt(long, parse(from_os_str))]
    log_file: Option<PathBuf>,

    /// Internal. No stability guarantees.
    #[structopt(long)]
    extreme: bool,
}


impl Command {
    pub fn run(&self) {
        let inputs = self.inputs
            .clone()
            .into_iter()
            .filter_map(|x| glob::glob(&x).ok())
            .map(|x| x.collect::<Vec<_>>())
            .flatten()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        if inputs.len() > 1 && self.output_file.is_some() {
            panic!("Output file isn’t valid for multiple input file paths, maybe use `--output-dir`?");
        }
        let output = match (self.output_file.clone(), self.output_dir.clone(), self.replace) {
            (Some(x), None, false) => OutputType::File(x),
            (None, Some(x), false) => OutputType::Dir(x),
            (None, None, true) => OutputType::Replace,
            _ => panic!("invalid output type")
        };
        if output.is_replace() {
            eprintln!("[warning] replacing input files");
            eprintln!("[note] imager only works for original images, i.e. your highest quality versions")
        }
        let entries = inputs
            .clone()
            .into_iter()
            .flat_map(|input_path| {
                self.formats
                    .clone()
                    .into_iter()
                    .flat_map(|f| f.0)
                    .map(|f| (input_path.clone(), f))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let progress_bar = ProgressBar::new(entries.len() as u64);
        progress_bar.tick();
        if entries.is_empty() {
            eprintln!("[warning] no (or missing) input files given");
        }
        let entries_len = entries.len();
        let process = |input_path: PathBuf, output_format: OutputFormat| -> api::OutMeda {
            let mut opt_job = crate::api::OptJob::open(&input_path).expect("open input file path");
            opt_job.output_format(output_format.clone());
            if let Some(max_size) = self.max_size.clone() {
                opt_job.max_size(max_size);
            }
            let (encoded, mut out_meta) = opt_job.run(self.extreme).expect("opt job failed");
            out_meta.input_path = Some(input_path.clone());
            out_meta.output_path = None;
            let different_format = {
                OutputFormat::infer_from_path(&input_path)
                    .map(|src| src != output_format.clone())
                    .unwrap_or(true)
            };
            let file_name = input_path
                .file_name()
                .expect("file name")
                .to_str()
                .expect("OsStr to str");
            let output_ext = match output_format {
                OutputFormat::Jpeg => "jpeg",
                OutputFormat::Png => "png",
                OutputFormat::Webp => "webp",
            };
            match output.clone() {
                OutputType::Dir(path) => {
                    if !path.exists() {
                        std::fs::create_dir_all(&path).expect("create parent dir");
                    }
                    let mut output_path = path.join(file_name);
                    if different_format {
                        output_path.set_extension(output_ext);
                    }
                    out_meta.output_path = Some(output_path.clone());
                    std::fs::write(output_path, encoded).expect("failed to write output file");
                }
                OutputType::File(mut output_path) => {
                    let parent_dir = output_path
                        .parent()
                        .expect("get parent path");
                    if !parent_dir.exists() {
                        std::fs::create_dir_all(&parent_dir).expect("create parent dir");
                    }
                    if different_format {
                        output_path.set_extension(output_ext);
                    }
                    out_meta.output_path = Some(output_path.clone());
                    std::fs::write(output_path, encoded).expect("failed to write output file");
                }
                OutputType::Replace => {
                    let mut output_path = input_path.clone();
                    if different_format {
                        output_path.set_extension(output_ext);
                    }
                    out_meta.output_path = Some(output_path.clone());
                    std::fs::write(output_path, encoded).expect("failed to write output file");
                }
            }
            out_meta
        };
        let output_log = entries
            .into_par_iter()
            .map(|(input_path, output_format)| {
                let out_meta = process(input_path, output_format);
                // DONE
                progress_bar.inc(1);
                out_meta
            })
            .collect::<Vec<api::OutMeda>>();
        // SAVE LOG FILE
        if let Some(log_path) = self.log_file.clone() {
            let output_log = serde_json::to_string_pretty(&output_log).expect("to json str failed");
            std::fs::write(log_path, output_log);
        }
        // DONE
        progress_bar.finish();
    }
}


///////////////////////////////////////////////////////////////////////////////
// MAIN
///////////////////////////////////////////////////////////////////////////////



fn main() {
    let cmd = Command::from_args();
    cmd.run();
}