#![allow(unused)]
pub mod vmaf;
pub mod encoder;
pub mod yuv420p;
pub mod stream;
pub mod search;

fn main() {
    encoder::run();
    // vmaf::run();
    // for i in 0..3 {
    //     vmaf::run();
    // }
}
