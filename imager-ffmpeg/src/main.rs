#![allow(unused)]
pub mod mux;
pub mod dec;
pub mod demux;
pub mod format;

fn main() {
    format::decode::run();
}
