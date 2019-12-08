#![allow(unused)]
pub mod vmaf;
pub mod yuv420p;
pub mod stream;
pub mod enc;

fn main() {
    enc::run();
}
