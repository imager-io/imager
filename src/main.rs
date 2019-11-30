#![allow(unused)]
pub mod png;
pub mod jpeg;
pub mod jpeg_opt;
pub mod vmaf;
pub mod classifier;
pub mod utils;
pub mod data;

fn main() {
    png::run();
}
