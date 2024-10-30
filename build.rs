use std::env;
use std::path::PathBuf;

fn main() {
    // Re-run build script if these files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");

    const SUBMODULE: &str = "open-meteo/Sources/OmFileFormatC";
    const LIB_NAME: &str = "ic";

    // Determine the target and arch
    let target = env::var("TARGET").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    // Set sysroot for cross compilation directly
    let sysroot = if target == "aarch64-unknown-linux-gnu" {
        Some("/usr/aarch64-linux-gnu")
    } else {
        None
    };

    let mut build = cc::Build::new();

    // Include directories
    build.include(format!("{}/include", SUBMODULE));
    // Add all .c files from the submodule's src directory
    let src_path = format!("{}/src", SUBMODULE);
    for entry in std::fs::read_dir(&src_path).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("c") {
            build.file(path);
        }
    }

    // Basic compiler flags
    build
        // .flag("-Wall")
        .flag("-O2");

    // Add -fPIC for non-Windows targets
    if !target.contains("windows") {
        build.flag("-fPIC");
    }

    if arch == "x86_64" {
        // Add SSE flags
        build.flag("-msse4.1");

        // Optionally add AVX2 flags if AVX2=1 is set
        if env::var("AVX2").unwrap_or_default() == "1" {
            build.flag("-mavx2");
        }
    } else if arch == "aarch64" {
        build.flag("-march=armv8-a");
    } else if arch.contains("iphone") {
        build.flag("-DHAVE_MALLOC_MALLOC");
    }

    // Set sysroot if specified
    if let Some(sysroot_path) = sysroot {
        build.flag(&format!("--sysroot={}", sysroot_path));
    }

    // Compile the library
    build.compile(LIB_NAME);

    // Generate bindings using bindgen
    let bindings = bindgen::Builder::default()
        // Set sysroot for bindgen if specified
        .clang_arg(sysroot.map_or("".to_string(), |s| format!("--sysroot={}", s)))
        .clang_arg(format!("-I{}/include", SUBMODULE))
        .header(format!("{}/include/vp4.h", SUBMODULE))
        .header(format!("{}/include/fp.h", SUBMODULE))
        .header(format!("{}/include/om_decoder.h", SUBMODULE))
        .header(format!("{}/include/delta2d.h", SUBMODULE))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link the static library
    println!("cargo:rustc-link-lib=static={}", LIB_NAME);
}
