use std::sync::{Arc, Mutex};
use image::{DynamicImage, GenericImage, GenericImageView};
use dcv_color_primitives::{convert_image, ColorSpace, ImageFormat, PixelFormat};

fn ensure_even_reslution(source: &DynamicImage) -> DynamicImage {
    let (width, height) = source.dimensions();
    // ENSURE EVEN
    let even_width = (width % 2) == 0;
    let even_height = (height % 2) == 0;
    if (!even_width) || (!even_height) {
        let new_width = {
            if !even_width {
                width - 1
            } else {
                width
            }
        };
        let new_height = {
            if !even_height {
                height - 1
            } else {
                height
            }
        };
        let new_image = source
            .clone()
            .crop(0, 0, new_width, new_height);
        new_image
    } else {
        source.clone()
    }
}


pub fn to_nv12(source: &DynamicImage) -> (Vec<u8>, usize, usize) {
    // ENSURE VALID INPUT IMAGE
    let source = ensure_even_reslution(source);
    // SETUP
    dcv_color_primitives::initialize();
    let (mut width, height) = source.dimensions();
    
    // ALLOCATE INPUT
    let source_buffer: Vec<u8> = {
        source
            .to_bgra()
            .pixels()
            .flat_map(|px: &::image::Bgra<u8>| vec![
                px.0[0],
                px.0[1],
                px.0[2],
                px.0[3],
            ])
            .collect::<Vec<u8>>()
    };
    let input_data: &[&[u8]] = &[&source_buffer[..]];
    let src_format = ImageFormat {
        pixel_format: PixelFormat::Bgra,
        color_space: ColorSpace::Lrgb,
        num_planes: 1,
    };
    
    // ALLOCATE OUTPUT
    let dst_size: usize = 3 * (width as usize) * (height as usize) / 2;
    let mut output_buffer = vec![0u8; dst_size];
    let output_data: &mut [&mut [u8]] = &mut [&mut output_buffer[..]];
    let dst_format = ImageFormat {
        pixel_format: PixelFormat::Nv12,
        color_space: ColorSpace::Bt601,
        num_planes: 1,
    };

    // GO!
    convert_image(
        width,
        height,
        &src_format,
        None,
        input_data,
        &dst_format,
        None,
        output_data,
    ).expect("convert rgba source to nv12");
    
    // DONE
    assert!(output_data.len() == 1);
    (
        output_data[0].to_owned(),
        source.width() as usize,
        source.height() as usize,
    )
}

pub fn to_yuv420p(source: &DynamicImage) -> ([Vec<u8>; 3], usize, usize) {
    use itertools::Itertools;
    let (mut nv12, width, height) = to_nv12(source);
    std::fs::write("test.nv12.yuv", &nv12);
    let y_size: usize = {
        (width * height) as usize
    };
    let uv_size_interleaved: usize = {
        (width * height / 2) as usize
    };
    let uv_size_planar: usize = {
        (width * height / 4) as usize
    };
    // println!("nv12: {}", nv12.len());
    // println!("y_size: {}", y_size);
    // println!("uv_size_interleaved: {}", uv_size_interleaved);
    assert!(nv12.len() == y_size + uv_size_interleaved);
    let y = nv12
        .drain(0 .. y_size)
        .collect::<Vec<_>>();
    assert!(nv12.len() == uv_size_interleaved);
    let (u, v) = nv12
        .into_iter()
        .chunks(2)
        .into_iter()
        .map(|uv| {
            let uv = uv.collect::<Vec<u8>>();
            assert!(uv.len() == 2);
            let u = uv[0];
            let v = uv[1];
            (u, v)
        })
        .unzip::<_, _, Vec<_>, Vec<_>>();
    assert!(u.len() + v.len() == uv_size_interleaved);
    assert!(u.len() == uv_size_planar);
    assert!(v.len() == uv_size_planar);
    ([y, u, v], width, height)
}


// pub fn to_y4m(source: &DynamicImage) -> y4m::Frame {
//     let [y, u, v] = to_yuv420p(source);
//     let frame = y4m::Frame::new([&y, &u, &v], None);
//     frame
// }