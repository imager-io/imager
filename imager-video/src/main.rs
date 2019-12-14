#![allow(unused)]
pub mod codec;
pub mod format;
pub mod data;
pub mod tool;

use data::{VideoBuffer, Yuv420P};

fn format() {
    let path = "assets/samples/test.h264";
    let video = VideoBuffer::open_video(path).expect("decode video file");
    println!("video frames: {}", video.as_frames().len());
    video
        .as_frames()
        .get(50)
        .expect("get frame")
        .save("assets/output/test.yuv");
}

fn encode_from_dir() {
    let path = "assets/samples/dump-2";
    let stream = VideoBuffer::open_image_dir(path).expect("load source dir");
    let output = unsafe {codec::h264::encode(stream, 0.0).expect("encode to h264")};
    std::fs::write("assets/output/dump2.h264", &output);
}

fn main() {
    // codec::h264::run();
    // encode_from_dir();
    tool::vmaf::run();
}
