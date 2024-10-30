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

    let is_windows = target.contains("windows");

    // Set sysroot for cross compilation directly
    let sysroot = if target == "aarch64-unknown-linux-gnu" {
        Some("/usr/aarch64-linux-gnu")
    } else {
        None
    };

    let mut build = cc::Build::new();

    let compiler = build.get_compiler();

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

    // --- Platform-specific flags ---
    if !is_windows {
        build.flag("-fPIC");
    }
    if target.contains("iphone") {
        build.flag("-DHAVE_MALLOC_MALLOC");
    }

    // --- Architecture-specific flags ---
    match arch.as_str() {
        "ppc64le" => {
            // PowerPC 64 Little Endian
            build.define("__SSSE3__", None);
            build.flag("-mcpu=power9");
            build.flag("-mtune=power9");
        }
        "aarch64" => {
            // ARM64
            build.flag("-march=armv8-a");

            // if compiler.is_like_clang() {
            //     build.flag("-fomit-frame-pointer");
            //     // Uncomment the following line if you need to set the macro limit for Clang
            //     // build.flag("-fmacro-backtrace-limit=0");
            // }
        }
        "x86_64" => {
            // x86_64 Architecture
            if is_windows && compiler.is_like_msvc() {
                // MSVC-specific flags for SSE and AVX
                // Note: MSVC does not support /arch:SSE4.1 directly
                // Using /arch:AVX instead, which includes SSE4.1
                build.flag("/arch:AVX");
                build.flag("/arch:AVX2");
                build.flag("/arch:SSE2");

                // // Define __SSE2__ manually for MSVC
                build.define("__SSE2__", None);
            } else {
                // For now just build for the native architecture
                // This can be changed to a common baseline if necessary
                build.flag("-march=native");
            }
        }
        _ => {
            // Handle other architectures if necessary
        }
    }

    // Set sysroot if specified
    if let Some(sysroot_path) = sysroot {
        build.flag(&format!("--sysroot={}", sysroot_path));
    }

    // Compile the library
    build.warnings(false);
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
