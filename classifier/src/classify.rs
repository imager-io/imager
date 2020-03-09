use std::io::Cursor;
use av_metrics::video::*;
use image::{DynamicImage, GenericImage, GenericImageView};
// use clap::{App, Arg};
// use console::style;
// use maplit::hashmap;
// use serde::Serialize;
// use std::collections::HashMap;
// use std::error::Error;
// use std::fs::File;
// use std::path::Path;
// use std::process::exit;



pub fn run() {
    let input_path = "assets/samples/pexels-photo-1153655.jpeg";
    let quality = 10;
    
    // SOURCE-1
    let source1 = ::image::open(input_path).expect("source image");

    // SOURCE-2
    let source2 = ::image::open(input_path).expect("source image");
    let source2 = unsafe {crate::codec::jpeg::encode(&source2, quality)};
    std::fs::write("test.jpeg", &source2);
    let source2 = ::image::load_from_memory_with_format(&source2, ::image::ImageFormat::Jpeg).expect("decode jpeg buffer");

    // RESIZE
    let source1 = source1.resize(300, 300, ::image::imageops::FilterType::Lanczos3);
    let source2 = source2.resize(300, 300, ::image::imageops::FilterType::Lanczos3);

    // START METRICS
    println!("computing metrics: {}", quality);

    // RUN SSIM
    let to_encoded = |source: &DynamicImage| {
        let encoded = {
            let ([y, u, v], width, height) = crate::color::format::to_yuv420p(&source);
            let y4m_frame = y4m::Frame::new([&y, &u, &v], None);
            let encoder = y4m::encode(
                width,
                height,
                y4m::Ratio::new(0, 1),
            );
            let mut buffer = Vec::<u8>::new();
            {
                let encoder = encoder.with_colorspace(y4m::Colorspace::C420);
                let mut encoder = encoder.write_header(&mut buffer).expect("y4m encoder init");
                encoder.write_frame(&y4m_frame).expect("write y4m frame");
            }
            buffer
        };
        std::io::Cursor::new(encoded)
    };

    let ssim = {
        let mut decoder1 = to_encoded(&source1);
        let mut decoder2 = to_encoded(&source2);
        
        let mut decoder1: y4m::Decoder<Cursor<Vec<u8>>> = y4m::Decoder::new(&mut decoder1).expect("init y4m decoder");
        let mut decoder2: y4m::Decoder<Cursor<Vec<u8>>> = y4m::Decoder::new(&mut decoder2).expect("init y4m decoder");
        av_metrics::video::ssim::calculate_video_ssim(
            &mut decoder1,
            &mut decoder2,
            None,
        ).expect("calculate_video_ssim")
    };

    let msssim = {
        let mut decoder1 = to_encoded(&source1);
        let mut decoder2 = to_encoded(&source2);
        
        let mut decoder1: y4m::Decoder<Cursor<Vec<u8>>> = y4m::Decoder::new(&mut decoder1).expect("init y4m decoder");
        let mut decoder2: y4m::Decoder<Cursor<Vec<u8>>> = y4m::Decoder::new(&mut decoder2).expect("init y4m decoder");
        av_metrics::video::ssim::calculate_video_msssim(
            &mut decoder1,
            &mut decoder2,
            None,
        ).expect("calculate_video_msssim")
    };

    let ciede = {
        let mut decoder1 = to_encoded(&source1);
        let mut decoder2 = to_encoded(&source2);
        
        let mut decoder1: y4m::Decoder<Cursor<Vec<u8>>> = y4m::Decoder::new(&mut decoder1).expect("init y4m decoder");
        let mut decoder2: y4m::Decoder<Cursor<Vec<u8>>> = y4m::Decoder::new(&mut decoder2).expect("init y4m decoder");
        av_metrics::video::ciede::calculate_video_ciede(
            &mut decoder1,
            &mut decoder2,
            None,
        ).expect("calculate_video_ciede")
    };

    let psnr_hvs = {
        let mut decoder1 = to_encoded(&source1);
        let mut decoder2 = to_encoded(&source2);
        
        let mut decoder1: y4m::Decoder<Cursor<Vec<u8>>> = y4m::Decoder::new(&mut decoder1).expect("init y4m decoder");
        let mut decoder2: y4m::Decoder<Cursor<Vec<u8>>> = y4m::Decoder::new(&mut decoder2).expect("init y4m decoder");
        av_metrics::video::psnr_hvs::calculate_video_psnr_hvs(
            &mut decoder1,
            &mut decoder2,
            None,
        ).expect("calculate_video_psnr_hvs")
    };

    println!("ssim: {:#?}", ssim);
    println!("msssim: {:#?}", msssim);
    println!("ciede: {:#?}", ciede);
    println!("psnr_hvs: {:#?}", psnr_hvs);
}
