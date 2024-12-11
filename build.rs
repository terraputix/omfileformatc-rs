use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Re-run build script if these files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");

    const SUBMODULE: &str = "open-meteo/Sources/OmFileFormatC";
    const LIB_NAME: &str = "omfileformatc";

    println!("cargo:rerun-if-changed={:}", SUBMODULE);
    // Determine the target and arch
    let target = env::var("TARGET").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    // Get sysroot from environment if it is set
    // This might be required for cross-compilation
    // compare: https://github.com/rust-lang/rust-bindgen/issues/1229
    let sysroot = env::var("SYSROOT").ok();

    let is_windows = target.contains("windows");

    // Check for MARCH_SKYLAKE environment variable
    let use_skylake = env::var("MARCH_SKYLAKE")
        .map(|v| v == "TRUE")
        .unwrap_or(false);

    let mut build = cc::Build::new();

    // We try to use clang if it is available
    let clang_available = Command::new("clang")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if clang_available {
        build.compiler("clang");
    }

    // respect CC env variable if set
    let _ = env::var("CC").map(|cc| build.compiler(cc));

    let compiler = build.get_compiler();
    println!("cargo:compiler={:?}", compiler.path());

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
        .flag("-O3");

    println!("cargo:warning=blub...");
    // --- Architecture-specific flags ---
    match arch.as_str() {
        "aarch64" => {
            println!("cargo:warning=Using -march=armv8-a");
            // ARM64
            // build.flag("-march=armv8-a+simd+crypto");
            build.flag("-march=armv8-a+simd");
            build.flag("-mavx2");
            build.flag("-mbmi2");
            // build.flag("-mssse3");
            // build.flag("-msse2");
            // build.flag("-msse4.1");
            // build.define("SIMDE_ENABLE_NATIVE_ALIASES", None);

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
                // build.flag("/arch:AVX");
                // build.flag("/arch:AVX2");
                // build.flag("/arch:SSE2");

                // Define __SSE2__ manually for MSVC
                // build.define("__SSE2__", None);
            } else {
                build.flag("-mavx2");
                build.flag("-mbmi2");
                // build.flag("-mssse3");
                // build.flag("-msse2");
                // build.flag("-msse4.1");
                build.define("SIMDE_ENABLE_NATIVE_ALIASES", None);

                // Choose between skylake and native based on environment variable
                // if use_skylake {
                //     build.flag("-march=skylake");
                //     println!("cargo:warning=Using -march=skylake");
                // } else {
                //     build.flag("-march=native");
                //     println!("cargo:warning=Using -march=native");
                // }
            }
        }
        _ => {
            // Handle other architectures if necessary
        }
    }

    // Set sysroot if specified
    if let Some(sysroot_path) = &sysroot {
        build.flag(&format!("--sysroot={}", sysroot_path));
    }

    // Compile the library
    build.warnings(false);
    build.compile(LIB_NAME);

    // Generate bindings using bindgen
    let bindings = bindgen::Builder::default()
        // Set sysroot for bindgen if specified (for cross compilation)
        .clang_arg(sysroot.map_or("".to_string(), |s| format!("--sysroot={}", s)))
        .clang_arg(format!("-I{}/include", SUBMODULE))
        .header(format!("{}/include/om_file_format.h", SUBMODULE))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link the static library
    println!("cargo:rustc-link-lib=static={}", LIB_NAME);
}
