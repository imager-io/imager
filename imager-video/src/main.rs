#![allow(unused)]
pub mod codec;
pub mod format;
pub mod data;
pub mod tool;

use data::{VideoBuffer, Yuv420P};

fn format() {
    let path = "assets/samples/test.h264";
    let video = VideoBuffer::open(path).expect("decode video file");
    println!("video frames: {}", video.as_frames().len());
    video
        .as_frames()
        .get(50)
        .expect("get frame")
        .save("assets/output/test.yuv");
}

fn main() {
    codec::h264::run();
}
