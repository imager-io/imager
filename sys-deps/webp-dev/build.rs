#![allow(unused)]

use std::path::PathBuf;
use std::string::ToString;
use flate2::read::GzDecoder;
use tar::Archive;

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
    lib_file: PathBuf,
    decode_header_file: PathBuf,
    encode_header_file: PathBuf,
    types_header_file: PathBuf,
}

fn download_and_build_webp() -> Result<WebpFiles, String> {
    let out_dir = get_webp_output_dir();
    let download_dir = out_dir.join("download");
    let source_dir = out_dir.join("source");
    let release_dir = out_dir.join("release");
    // OUTPUT FILES
    let lib_file = release_dir.join("libwebp.a");
    let decode_header_file = release_dir.join("decode.h");
    let encode_header_file = release_dir.join("encode.h");
    let types_header_file = release_dir.join("types.h");
    // CHECKS
    if is_debug_mode() {
        // Let’s not re-download this every time someone (or their dev tools)
        // builds the project. Unless in release mode.
        let all_exists =
            lib_file.exists() &&
            decode_header_file.exists() &&
            encode_header_file.exists() &&
            types_header_file.exists();
        if all_exists {
            return Ok(WebpFiles{
                release_dir,
                lib_file,
                decode_header_file,
                encode_header_file,
                types_header_file,
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
    let cpy = |src: PathBuf, dest: &PathBuf| {
        std::fs::copy(&src, dest).expect(&format!(
            "unable to cpy from {:?} to {:?}",
            src,
            dest,
        ));
    };
    cpy(source_dir.join("src/libwebp.a"), &lib_file);
    cpy(source_dir.join("src/webp/decode.h"), &decode_header_file);
    cpy(source_dir.join("src/webp/encode.h"), &encode_header_file);
    cpy(source_dir.join("src/webp/types.h"), &types_header_file);
    // CLEANUP
    std::fs::remove_dir_all(&download_dir).map_err(|x| x.to_string())?;
    std::fs::remove_dir_all(&source_dir).map_err(|x| x.to_string())?;
    // DONE
    Ok(WebpFiles{
        release_dir,
        lib_file,
        decode_header_file,
        encode_header_file,
        types_header_file,
    })
}


///////////////////////////////////////////////////////////////////////////////
// ENTRYPOINT
///////////////////////////////////////////////////////////////////////////////

fn build_all() {
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
    println!("cargo:rustc-link-lib=static=webp");
    
    // BUILD RUST FFI CODE
    bindgen::Builder::default()
        .header(webp_files.decode_header_file.to_str().expect("PathBuf as str"))
        .header(webp_files.encode_header_file.to_str().expect("PathBuf as str"))
        .header(webp_files.types_header_file.to_str().expect("PathBuf as str"))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn build_docs_only() {
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    // BUILD RUST FFI CODE
    bindgen::Builder::default()
        .header("include/webp/decode.h")
        .header("include/webp/encode.h")
        .header("include/webp/types.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}


fn main() {
    #[cfg(feature="buildtype-docs-only")]
    build_docs_only();

    #[cfg(not(feature="buildtype-docs-only"))]
    build_all();
}