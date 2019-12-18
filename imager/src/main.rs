//! NOTE:
//! * Just used for development.
//! * Imager CLI tools are under the imager-tools GitHub repo.
#![allow(unused)]
pub mod classifier;
pub mod codec;
pub mod vmaf;
pub mod data;

fn main() {
    codec::jpeg::run();
}
