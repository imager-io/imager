#![allow(unused)]

use std::path::PathBuf;
use std::string::ToString;
use flate2::read::GzDecoder;
use tar::Archive;

///////////////////////////////////////////////////////////////////////////////
// BUILD
///////////////////////////////////////////////////////////////////////////////

fn is_release_mode() -> bool {
    let value = std::env::var("PROFILE")
        .expect("missing PROFILE")
        .to_lowercase();
    &value == "release"
}

fn is_debug_mode() -> bool {
    let value = std::env::var("PROFILE")
        .expect("missing PROFILE")
        .to_lowercase();
    &value == "debug"
}

fn get_webp_output_dir() -> PathBuf {
    let out_dir = std::env::var("OUT_DIR").expect("missing OUT_DIR");
    let out_dir = PathBuf::from(out_dir).join("webp");
    std::fs::create_dir_all(&out_dir).expect("unable to add webp dir under OUT_DIR");
    out_dir
}

fn run_make(source_path: &PathBuf) {
    let result = std::process::Command::new("make")
        .arg("-C")
        .arg(source_path)
        .args(&["-f", "makefile.unix"])
        .output()
        .expect(&format!("make -C {:?} failed", source_path));
}


#[derive(Debug, Clone)]
pub struct WebpFiles {
    release_dir: PathBuf,
    
    libwebp_a: PathBuf,
    libwebpdemux_a: PathBuf,


    decode_header_file: PathBuf,
    encode_header_file: PathBuf,
    types_header_file: PathBuf,

    imageio_image_dec_h: PathBuf,
    imageio_image_enc_h: PathBuf,
    imageio_imageio_util_h: PathBuf,
    imageio_jpegdec_h: PathBuf,
    imageio_metadata_h: PathBuf,
    imageio_pngdec_h: PathBuf,
    imageio_pnmdec_h: PathBuf,
    imageio_tiffdec_h: PathBuf,
    imageio_webpdec_h: PathBuf,
    imageio_wicdec_h: PathBuf,

    imageio_libimagedec_a: PathBuf,
    imageio_libimageenc_a: PathBuf,
    imageio_libimageio_util_a: PathBuf,
}

fn download_and_build_webp() -> Result<WebpFiles, String> {
    let out_dir = get_webp_output_dir();
    let download_dir = out_dir.join("download");
    let source_dir = out_dir.join("source");
    let release_dir = out_dir.join("release");
    // OUTPUT FILES


    let libwebp_a = release_dir.join("libwebp.a");
    let libwebpdemux_a = release_dir.join("libwebpdemux.a");
    let decode_header_file = release_dir.join("decode.h");
    let encode_header_file = release_dir.join("encode.h");
    let types_header_file = release_dir.join("types.h");

    let imageio_image_dec_h = release_dir.join("imageio/image_dec.h");
    let imageio_image_enc_h = release_dir.join("imageio/image_enc.h");
    let imageio_imageio_util_h = release_dir.join("imageio/imageio_util.h");
    let imageio_jpegdec_h = release_dir.join("imageio/jpegdec.h");
    let imageio_metadata_h = release_dir.join("imageio/metadata.h");
    let imageio_pngdec_h = release_dir.join("imageio/pngdec.h");
    let imageio_pnmdec_h = release_dir.join("imageio/pnmdec.h");
    let imageio_tiffdec_h = release_dir.join("imageio/tiffdec.h");
    let imageio_webpdec_h = release_dir.join("imageio/webpdec.h");
    let imageio_wicdec_h = release_dir.join("imageio/wicdec.h");

    let imageio_libimagedec_a = release_dir.join("libimagedec.a");
    let imageio_libimageenc_a = release_dir.join("libimageenc.a");
    let imageio_libimageio_util_a = release_dir.join("libimageio_util.a");

    // CHECKS
    if is_debug_mode() {
        // Let’s not re-download this every time someone (or their dev tools)
        // builds the project. Unless in release mode.
        let all_exists =
            libwebp_a.exists() &&
            libwebpdemux_a.exists() &&
            decode_header_file.exists() &&
            encode_header_file.exists() &&
            types_header_file.exists() &&
            imageio_image_dec_h.exists() &&
            imageio_image_enc_h.exists() &&
            imageio_imageio_util_h.exists() &&
            imageio_jpegdec_h.exists() &&
            imageio_metadata_h.exists() &&
            imageio_pngdec_h.exists() &&
            imageio_pnmdec_h.exists() &&
            imageio_tiffdec_h.exists() &&
            imageio_webpdec_h.exists() &&
            imageio_wicdec_h.exists() &&
            imageio_libimagedec_a.exists() &&
            imageio_libimageenc_a.exists() &&
            imageio_libimageio_util_a.exists();
        if all_exists {
            return Ok(WebpFiles{
                release_dir,
                
                libwebp_a,
                libwebpdemux_a,
                
                decode_header_file,
                encode_header_file,
                types_header_file,

                imageio_image_dec_h,
                imageio_image_enc_h,
                imageio_imageio_util_h,
                imageio_jpegdec_h,
                imageio_metadata_h,
                imageio_pngdec_h,
                imageio_pnmdec_h,
                imageio_tiffdec_h,
                imageio_webpdec_h,
                imageio_wicdec_h,

                imageio_libimagedec_a,
                imageio_libimageenc_a,
                imageio_libimageio_util_a,
            });
        }
    }
    // CLEAN
    std::fs::remove_dir_all(&out_dir).map_err(|x| x.to_string())?;
    // SETUP
    std::fs::create_dir_all(&download_dir).map_err(|x| x.to_string())?;
    // DOWNLOAD
    let url = "https://github.com/webmproject/libwebp/tarball/1.0.3";
    let tar_reply = reqwest::get(url).expect("unable to get webp tar file from github");
    let tar = GzDecoder::new(tar_reply);
    let mut archive = Archive::new(tar);
    // UNPACK ARCHIVE
    let tmp_source_dir: Option<PathBuf> = {
        archive
            .unpack(&download_dir)
            .map_err(|x| format!(
                "failed to unpack webp tar payload from github to {:?}: {:?}",
                download_dir,
                x
            ))?;
        let xs = std::fs::read_dir(&download_dir)
            .expect(&format!("unable to read dir {:?}", download_dir))
            .filter_map(Result::ok)
            .filter(|file| {
                file.file_type()
                    .map(|x| x.is_dir())
                    .unwrap_or(false)
            })
            .collect::<Vec<std::fs::DirEntry>>();
        match &xs[..] {
            [x] => Some(x.path()),
            _ => None,
        }
    };
    // MOVE TO STD SOURCE DIR
    let tmp_source_dir = tmp_source_dir.expect("unexpected tar output from github");
    std::fs::rename(&tmp_source_dir, &source_dir)
        .map_err(|x| format!(
            "unable to rename from {:?} to {:?}: {}",
            tmp_source_dir,
            source_dir,
            x,
        ))?;
    // COMPILE SOURCE
    run_make(&source_dir);
    // TO RELEASE DIR
    std::fs::create_dir_all(&release_dir).map_err(|x| x.to_string())?;
    std::fs::create_dir_all(release_dir.join("imageio")).map_err(|x| x.to_string())?;
    

    let cpy = |src: PathBuf, dest: &PathBuf| {
        std::fs::copy(&src, dest).expect(&format!(
            "unable to cpy from {:?} to {:?}",
            src,
            dest,
        ));
    };
    cpy(source_dir.join("src/libwebp.a"), &libwebp_a);
    cpy(source_dir.join("src/webp/decode.h"), &decode_header_file);
    cpy(source_dir.join("src/webp/encode.h"), &encode_header_file);
    cpy(source_dir.join("src/webp/types.h"), &types_header_file);
    cpy(source_dir.join("imageio/image_dec.h"), &imageio_image_dec_h);
    cpy(source_dir.join("imageio/image_enc.h"), &imageio_image_enc_h);
    cpy(source_dir.join("imageio/imageio_util.h"), &imageio_imageio_util_h);
    cpy(source_dir.join("imageio/jpegdec.h"), &imageio_jpegdec_h);
    cpy(source_dir.join("imageio/metadata.h"), &imageio_metadata_h);
    cpy(source_dir.join("imageio/pngdec.h"), &imageio_pngdec_h);
    cpy(source_dir.join("imageio/pnmdec.h"), &imageio_pnmdec_h);
    cpy(source_dir.join("imageio/tiffdec.h"), &imageio_tiffdec_h);
    cpy(source_dir.join("imageio/webpdec.h"), &imageio_webpdec_h);
    cpy(source_dir.join("imageio/wicdec.h"), &imageio_wicdec_h);
    cpy(source_dir.join("imageio/libimagedec.a"), &imageio_libimagedec_a);
    cpy(source_dir.join("imageio/libimageenc.a"), &imageio_libimageenc_a);
    cpy(source_dir.join("imageio/libimageio_util.a"), &imageio_libimageio_util_a);
    // CLEANUP
    std::fs::remove_dir_all(&download_dir).map_err(|x| x.to_string())?;
    std::fs::remove_dir_all(&source_dir).map_err(|x| x.to_string())?;
    // DONE
    Ok(WebpFiles{
        release_dir,
        
        libwebp_a,
        libwebpdemux_a,
        
        decode_header_file,
        encode_header_file,
        types_header_file,
        imageio_image_dec_h,
        imageio_image_enc_h,
        imageio_imageio_util_h,
        imageio_jpegdec_h,
        imageio_metadata_h,
        imageio_pngdec_h,
        imageio_pnmdec_h,
        imageio_tiffdec_h,
        imageio_webpdec_h,
        imageio_wicdec_h,

        imageio_libimagedec_a,
        imageio_libimageenc_a,
        imageio_libimageio_util_a,
    })
}


///////////////////////////////////////////////////////////////////////////////
// DEPENDENCIES
///////////////////////////////////////////////////////////////////////////////

fn build_dependencies() {
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // DOWNLOAD & BUILD VMAF
    let webp_files = match download_and_build_webp() {
        Ok(x) => x,
        Err(x) => panic!("{}", x),
    };
    
    // LINK TO STATIC LIB
    println!("cargo:rustc-link-search=native={}", {
        webp_files.release_dir
            .to_str()
            .expect("unable to get str")
    });
    println!("cargo:rustc-link-search=native={}", {
        webp_files.release_dir
            .join("imageio")
            .to_str()
            .expect("unable to get str")
    });
    println!("cargo:rustc-link-lib=static=webp");
    println!("cargo:rustc-link-lib=static=imagedec");
    println!("cargo:rustc-link-lib=static=imageenc");
    println!("cargo:rustc-link-lib=static=imageio_util");
    
    // BUILD RUST FFI CODE - WEBP
    bindgen::Builder::default()
        .header(webp_files.decode_header_file.to_str().expect("PathBuf as str"))
        .header(webp_files.encode_header_file.to_str().expect("PathBuf as str"))
        .header(webp_files.types_header_file.to_str().expect("PathBuf as str"))
        .generate_comments(true)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings_webp.rs"))
        .expect("Couldn't write bindings!");
    
    // BUILD RUST FFI CODE - IMAGE-IO
    bindgen::Builder::default()
        .header(webp_files.imageio_image_dec_h.to_str().expect("PathBuf as str"))
        .header(webp_files.imageio_image_enc_h.to_str().expect("PathBuf as str"))
        .header(webp_files.imageio_imageio_util_h.to_str().expect("PathBuf as str"))
        .header(webp_files.imageio_jpegdec_h.to_str().expect("PathBuf as str"))
        .header(webp_files.imageio_metadata_h.to_str().expect("PathBuf as str"))
        .header(webp_files.imageio_pngdec_h.to_str().expect("PathBuf as str"))
        .header(webp_files.imageio_pnmdec_h.to_str().expect("PathBuf as str"))
        .header(webp_files.imageio_tiffdec_h.to_str().expect("PathBuf as str"))
        .header(webp_files.imageio_webpdec_h.to_str().expect("PathBuf as str"))
        .header(webp_files.imageio_wicdec_h.to_str().expect("PathBuf as str"))
        .generate_comments(true)
        .generate()
        .expect("Unable to generate bindings (image_io)")
        .write_to_file(out_path.join("bindings_imageio.rs"))
        .expect("Couldn't write bindings (imageio)!");
}


///////////////////////////////////////////////////////////////////////////////
// CBITS
///////////////////////////////////////////////////////////////////////////////

pub fn compile_cbits() {
    cc::Build::new()
        .include("include")
        .file("cbits/encoder.c")
        .file("cbits/decoder.c")
        .file("cbits/utils.c")
        .compile("cbits");
}

///////////////////////////////////////////////////////////////////////////////
// MAIN
///////////////////////////////////////////////////////////////////////////////

fn main() {
    build_dependencies();
    compile_cbits();
}
