#![allow(unused)]
pub mod classifier;
pub mod codec;
pub mod vmaf;
pub mod data;
pub mod api;

use std::path::PathBuf;
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use structopt::StructOpt;
use structopt::clap::ArgGroup;
use indicatif::{ProgressBar, ProgressStyle};
use imager::data::{
    OutputFormat,
    OutputSize,
    Resolution,
};

///////////////////////////////////////////////////////////////////////////////
// CLI FRONTEND
///////////////////////////////////////////////////////////////////////////////

/// The Imager CLI Interface
/// 
/// Output type much be one of: `--output-file`, `--output-dir`, or `--replace`.
#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
#[structopt(
    name = "imager",
    // rename_all = "kebab-case",
    group = (ArgGroup::with_name("output_type").required(true)),
)]
pub struct Command {
    /// Input file(s) path.
    #[structopt(short, long, required = true, min_values = 1)]
    inputs: Vec<String>,
    
    /// Output file path.
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
    #[structopt(long, parse(from_os_str), group = "output_type")]
    output_dir: Option<PathBuf>,

    /// Replace input files with their optimized results.
    /// Valid for multiple input/output files.
    #[structopt(long, group = "output_type")]
    replace: bool,
    
    /// Output format(s).
    /// 
    /// Multiple output formats may be specified, e.g. `--formats webp jpeg`.
    /// The saved results will have their file extension updated if different
    /// from the original.
    #[structopt(short, long, default_value = "jpeg")]
    formats: Vec<OutputFormat>,
    
    /// Output image size (resolution).
    /// 
    /// To target a specific resolution (say 100x100) use `--resize 100x100`.
    /// This will always preserve aspect ratio and only downscales when necessary.
    /// 
    /// To preserve the original resolution use `--resize full`.
    #[structopt(short, long)]
    resize: Option<OutputSize>,
}


// impl Command {
//     pub fn run(&self) {
//         let inputs = self.input
//             .clone()
//             .into_iter()
//             .filter_map(|x| glob::glob(&x).ok())
//             .map(|x| x.collect::<Vec<_>>())
//             .flatten()
//             .filter_map(Result::ok)
//             .collect::<Vec<_>>();
//         let to_out_path_for = |input_path: &PathBuf| -> PathBuf {
//             let filename = input_path
//                 .file_name()
//                 .expect("file name from path")
//                 .to_str()
//                 .expect("str path");
//             let mut output_path = self.output.clone();
//             std::fs::create_dir_all(&output_path)
//                 .expect("create output dir if missing");
//             output_path.push(&filename);
//             match self.format {
//                 OutputFormat::Jpeg => output_path.set_extension("jpeg")
//             };
//             output_path
//         };
//         if self.single {
//             if inputs.len() > 1 {
//                 panic!("The single flag is incompatible with multiple inputs.");
//             }
//         }
//         let progress_bar = ProgressBar::new(inputs.len() as u64);
//         progress_bar.tick();
//         inputs
//             .par_iter()
//             .for_each(|input_path| {
//                 let resize = self.size.clone();
//                 let source = opt::Source::open(input_path, resize).expect("load source");
//                 let (output, opt_meta) = source.run_search();
//                 let output_path = if self.single {
//                     self.output
//                         .parent()
//                         .map(|parent| {
//                             std::fs::create_dir_all(parent)
//                                 .expect("create missing parent directory");
//                         });
//                     self.output.clone()
//                 } else {
//                     to_out_path_for(input_path)
//                 };
//                 std::fs::write(&output_path, output).expect("write output file");
//                 progress_bar.inc(1);
//             });
//         progress_bar.finish();
//     }
// }


///////////////////////////////////////////////////////////////////////////////
// MAIN
///////////////////////////////////////////////////////////////////////////////



fn main() {
    let cmd = Command::from_args();
    println!("output: {:#?}", cmd);
    // cmd.run();
}