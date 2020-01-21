#![allow(unused)]
pub mod debug;
pub mod process;
pub mod codec;
pub mod quant;
pub mod auto;

fn main() {
    // process::run();
    auto::run();
}
