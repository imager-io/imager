use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use rand::prelude::*;
use image::{GenericImage, GenericImageView, ImageBuffer, DynamicImage};
use image::{Luma, Rgb, Pixel};
use imageproc::region_labelling::{
    connected_components,
    Connectivity
};
use imageproc::definitions::Image;
use imageproc::distance_transform::Norm;
use imageproc::definitions::HasBlack;
use image::GrayImage;
use rayon::prelude::*;
use serde::{Serialize, Deserialize};

use crate::color::palette::{self, ToPrettyRgbPalette};


pub fn quantizer(image: &DynamicImage) -> DynamicImage {
    let image = image.resize_exact(600, 600, ::image::FilterType::Lanczos3);
    let image = image.unsharpen(1.2, 4);
    let image = crate::color::quant::reduce_palette(&image, 64);
    let image = image.to_luma();
    let image = ::imageproc::map::map_pixels(&image, |x, y, mut px| {
        if px.0[0] == 0 {
            px.0[0] = 1;
        }
        px
    });
    let image = imageproc::region_labelling::connected_components(
        &image,
        Connectivity::Eight,
        Luma::black()
    );
    let image = palette::set_region(&image, Luma([std::u32::MAX]), |_, count| count > (120 * 120));
    
    // DONE
    let image = image.to_pretty_rgb_palette();
    DynamicImage::ImageRgb8(image)
}

pub fn preprocess(image: &DynamicImage, class: Class) -> DynamicImage {
    let image = quantizer(&image);
    image
}


///////////////////////////////////////////////////////////////////////////////
// CLASSIFY
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Class {
    HiBasic,
    Hi,
    ExLo,
    Lo,
}

impl Class {
    pub fn all_variants() -> Vec<Self> {
        vec![
            Class::HiBasic,
            Class::Hi,
            Class::ExLo,
            Class::Lo,
        ]
    }
    pub fn id(&self) -> u8 {
        match self {
            Class::HiBasic => 0,
            Class::Hi => 1,
            Class::ExLo => 2,
            Class::Lo => 3,
        }
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Class::HiBasic => "hi-basic",
            Class::Hi => "hi",
            Class::ExLo => "ex-lo",
            Class::Lo => "lo",
        };
        write!(f, "{}", value)
    }
}



pub fn train() {
    let load_group = |pattern: &str| -> Vec<DynamicImage> {
        glob::glob(pattern)
            .expect("input glob")
            .filter_map(Result::ok)
            .map(|input_path| {
                let image = ::image::open(input_path).expect("load input image");
                let image = quantizer(&image);
                image
            })
            .collect::<Vec<_>>()
    };
    let dataset = vec![
        (
            load_group("assets/samples/high/**/*.jpeg"),
            Class::Hi,
        ),
        (
            load_group("assets/samples/low/**/*.jpeg"),
            Class::Lo,
        ),
        (
            load_group("assets/samples/extra-low/**/*.jpeg"),
            Class::ExLo,
        ),
        (
            load_group("assets/samples/high-basic/**/*.jpeg"),
            Class::HiBasic,
        ),
    ];
    for (media, class) in dataset {

    }
}


///////////////////////////////////////////////////////////////////////////////
// MAIN
///////////////////////////////////////////////////////////////////////////////

pub fn process(input_path: &str, output_path: &str, class: Class) {
    // RUN
    let image = ::image::open(input_path).expect("open source image");
    let debug_image = preprocess(&image, class);

    // FILE PATHS
    let base_path = PathBuf::from(output_path)
        .parent()
        .expect("parent path")
        .to_owned();
    std::fs::create_dir_all(&base_path);
    
    let debug_output_path = {
        let mut path = PathBuf::from(output_path)
            .file_name()
            .map(|x| PathBuf::from(x.clone()))
            .expect("file name");
        path.set_extension("debug.jpeg");
        base_path.join(path)
    };

    // SAVE
    // image.save(output_path);
    debug_image.save(debug_output_path);
    // let compressed = unsafe {
    //     crate::codec::jpeg::encode(&image, 4)
    // };
    // std::fs::write(debug_output_path, compressed);
}

pub fn run() {
    let extra_low1 = vec![
        ("assets/samples/focus/extra-low/0000.jpeg", "assets/output/extra-low/0000.jpeg"),
        ("assets/samples/focus/extra-low/7s-RwkZ.jpeg", "assets/output/extra-low/7s-RwkZ.jpeg"),
        ("assets/samples/focus/extra-low/9w-10TO.jpeg", "assets/output/extra-low/9w-10TO.jpeg"),
        ("assets/samples/focus/extra-low/NN4-OebzQ7N2R7.jpeg", "assets/output/extra-low/NN4-OebzQ7N2R7.jpeg"),
        ("assets/samples/focus/extra-low/VNy-32PXbRWXlj.jpeg", "assets/output/extra-low/VNy-32PXbRWXlj.jpeg"),
        ("assets/samples/focus/extra-low/nGY-9nAdDR0NVM.jpeg", "assets/output/extra-low/nGY-9nAdDR0NVM.jpeg"),
        ("assets/samples/focus/extra-low/qW2-91VVN7mRD4.jpeg", "assets/output/extra-low/qW2-91VVN7mRD4.jpeg"),
    ];
    
    let low_sources1 = vec![
        ("assets/samples/focus/low/0003.jpeg", "assets/output/low/0003.jpeg"),
        ("assets/samples/focus/low/0005.jpeg", "assets/output/low/0005.jpeg"),
        ("assets/samples/focus/low/0041.jpeg", "assets/output/low/0041.jpeg"),
        ("assets/samples/focus/low/04-RO1j.jpeg", "assets/output/low/04-RO1j.jpeg"),
        ("assets/samples/focus/low/07-cIIF.jpeg", "assets/output/low/07-cIIF.jpeg"),
        ("assets/samples/focus/low/5k-IoGP.jpeg", "assets/output/low/5k-IoGP.jpeg"),
        ("assets/samples/focus/low/7J-7Tkf.jpeg", "assets/output/low/7J-7Tkf.jpeg"),
        ("assets/samples/focus/low/7p-WH0J.jpeg", "assets/output/low/7p-WH0J.jpeg"),
        ("assets/samples/focus/low/9X-hbpj.jpeg", "assets/output/low/9X-hbpj.jpeg"),
        ("assets/samples/focus/low/AJ-pR2p.jpeg", "assets/output/low/AJ-pR2p.jpeg"),
        ("assets/samples/focus/low/An-zDad.jpeg", "assets/output/low/An-zDad.jpeg"),
        ("assets/samples/focus/low/Nr-2JdU.jpeg", "assets/output/low/Nr-2JdU.jpeg"),
        ("assets/samples/focus/low/Uf-dIAI.jpeg", "assets/output/low/Uf-dIAI.jpeg"),
        ("assets/samples/focus/low/aC-49dv.jpeg", "assets/output/low/aC-49dv.jpeg"),
        ("assets/samples/focus/low/aO-j21N.jpeg", "assets/output/low/aO-j21N.jpeg"),
        ("assets/samples/focus/low/cX-AEXN.png", "assets/output/low/cX-AEXN.png"),
        ("assets/samples/focus/low/dO-J24k.jpeg", "assets/output/low/dO-J24k.jpeg"),
        ("assets/samples/focus/low/dn-Gku1.jpeg", "assets/output/low/dn-Gku1.jpeg"),
        ("assets/samples/focus/low/nd-G8ak.jpeg", "assets/output/low/nd-G8ak.jpeg"),
    ];

    let high_basic_sources1 = vec![
        ("assets/samples/focus/high-basic/2o-e6ft.jpeg", "assets/output/high-basic/2o-e6ft.jpeg"),
        ("assets/samples/focus/high-basic/Az-GMpY.jpeg", "assets/output/high-basic/Az-GMpY.jpeg"),
        ("assets/samples/focus/high-basic/Cl-PBPF.jpeg", "assets/output/high-basic/Cl-PBPF.jpeg"),
        ("assets/samples/focus/high-basic/Ju-7Bj3.jpeg", "assets/output/high-basic/Ju-7Bj3.jpeg"),
        ("assets/samples/focus/high-basic/RQ-4jpi.jpeg", "assets/output/high-basic/RQ-4jpi.jpeg"),
        ("assets/samples/focus/high-basic/sQ-hDXB.jpeg", "assets/output/high-basic/sQ-hDXB.jpeg"),
        ("assets/samples/focus/high-basic/xM-AhIP.jpeg", "assets/output/high-basic/xM-AhIP.jpeg"),
    ];

    let high_basic_sources2 = vec![
        ("assets/samples/focus/high-basic/5U-4oIc.jpeg", "assets/output/high-basic/5U-4oIc.jpeg"),
    ];

    let high1 = vec![
        ("assets/samples/focus/high/0J-uIfz.jpeg", "assets/output/high/0J-uIfz.jpeg"),
        ("assets/samples/focus/high/1s-93Jf.jpeg", "assets/output/high/1s-93Jf.jpeg"),
        ("assets/samples/focus/high/2F-CESC.jpeg", "assets/output/high/2F-CESC.jpeg"),
        ("assets/samples/focus/high/5W-7GcR.jpeg", "assets/output/high/5W-7GcR.jpeg"),
        ("assets/samples/focus/high/6L-RcpD.jpeg", "assets/output/high/6L-RcpD.jpeg"),
        ("assets/samples/focus/high/Aq-hR1z.jpeg", "assets/output/high/Aq-hR1z.jpeg"),
        ("assets/samples/focus/high/CK-tgYJ.jpeg", "assets/output/high/CK-tgYJ.jpeg"),
        ("assets/samples/focus/high/Dy-b6Y3.jpeg", "assets/output/high/Dy-b6Y3.jpeg"),
        ("assets/samples/focus/high/QN-9ZmY.jpeg", "assets/output/high/QN-9ZmY.jpeg"),
        ("assets/samples/focus/high/eH-vWUx.jpeg", "assets/output/high/eH-vWUx.jpeg"),
        ("assets/samples/focus/high/ec-WIuG.jpeg", "assets/output/high/ec-WIuG.jpeg"),
    ];

    let high2: Vec<(&str, &str)> = vec![

    ];

    let f = move |xs: Vec<(&'static str, &'static str)>, class: Class| {
        xs
            .into_par_iter()
            .map(|(s, d)| std::thread::spawn({
                let class = class.clone();
                move || process(s, d, class)
            }))
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|x| {
                x.join();
            });
    };


    f(high_basic_sources1, Class::HiBasic);
    f(extra_low1, Class::ExLo);
    f(high1, Class::Hi);
    f(low_sources1, Class::Lo);
}
