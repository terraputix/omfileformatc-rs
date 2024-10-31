use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Re-run build script if these files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");

    const SUBMODULE: &str = "open-meteo/Sources/OmFileFormatC";
    const LIB_NAME: &str = "omfileformatc";

    // Determine the target and arch
    let target = env::var("TARGET").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    // Get sysroot from environment if it is set
    // This might be required for cross-compilation
    // compare: https://github.com/rust-lang/rust-bindgen/issues/1229
    let sysroot = env::var("SYSROOT").ok();

    let is_windows = target.contains("windows");

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

    // if cfg!(target_family = "wasm") {
    if let Some(libc) = std::env::var_os("DEP_WASM32_UNKNOWN_UNKNOWN_OPENBSD_LIBC_INCLUDE") {
        build.include(libc);
        println!("cargo::rustc-link-lib=wasm32-unknown-unknown-openbsd-libc");
    }
    // }

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

    // --- Architecture-specific flags ---
    match arch.as_str() {
        "ppc64le" => {
            // PowerPC 64 Little Endian
            build.define("__SSSE3__", None);
            build.define("__SSE2__", None);
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
                // // MSVC-specific flags for SSE and AVX
                // // Note: MSVC does not support /arch:SSE4.1 directly
                // // Using /arch:AVX instead, which includes SSE4.1
                // build.flag("/arch:AVX");
                // build.flag("/arch:AVX2");
                // build.flag("/arch:SSE2");

                // // // Define __SSE2__ manually for MSVC
                // build.define("__SSE2__", None);
            } else {
                // For now just build for the native architecture
                // This can be changed to a common baseline if necessary
                build.flag("-march=native");
            }
        }
        "wasm32" => {
            build.flag("-msimd128");
            build.flag("-sMEMORY64");
            build.flag("-mssse3");
            build.define("__SSSE3__", None);
            build.define("__SSE2__", None);
            build.define("__SSE__", None);

            // WebAssembly
            // build.flag("-msimd128");
            // build.flag("-msse2");
            // build.flag("-msse");
            // build.flag("-msse3");
            // build.flag("-mssse3");
            // build.flag("-msse4.1");
            // build.flag("-msse4.2");
            // build.flag("-mavx");
            // build.flag("-mavx2");
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
        .header(format!("{}/include/vp4.h", SUBMODULE))
        .header(format!("{}/include/fp.h", SUBMODULE))
        .header(format!("{}/include/om_decoder.h", SUBMODULE))
        .header(format!("{}/include/delta2d.h", SUBMODULE))
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
