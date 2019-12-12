#![allow(unused)]
pub mod mux;
pub mod dec;
pub mod demux;

fn main() {
    demux::run();
}
