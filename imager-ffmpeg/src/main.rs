#![allow(unused)]
mod mux;
mod dec;
mod format;
mod codec;

fn main() {
    format::decode::run();
}
