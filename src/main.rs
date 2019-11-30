#![allow(unused)]
pub mod png;
pub mod vmaf;
pub mod jpeg;
pub mod classifier;
pub mod utils;

fn main() {
    png::run();
}
