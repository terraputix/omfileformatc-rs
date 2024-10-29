use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

const SUBMODULE: &str = "open-meteo/Sources/OmFileFormatC";
const LIB: &str = "ic";

fn run_command<I, S>(prog: &str, args: I, envs: Option<HashMap<&str, &str>>)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(prog);
    command.args(args);

    if let Some(env_vars) = envs {
        for (key, value) in env_vars {
            command.env(key, value);
        }
    }

    let output = command.output().expect("failed to start command");
    assert!(output.status.success(), "{:?}", output);
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=Makefile");

    let target = env::var("TARGET").unwrap();
    let (sysroot, env): (Option<&str>, Option<HashMap<&str, &str>>) = match target.as_str() {
        "aarch64-unknown-linux-gnu" => (Some("/usr/aarch64-linux-gnu"), None),
        // "aarch64-pc-windows-msvc" => (
        //     None,
        //     Some(HashMap::from([
        //         ("CLANG_TARGET", "aarch64-pc-windows-msvc"),
        //         ("ARCH", "aarch64"),
        //     ])),
        // ),
        _ => (None, None),
    };

    let bindings = bindgen::Builder::default()
        // fix strange cross compilation error from bindgen
        // https://github.com/rust-lang/rust-bindgen/issues/1229
        // for some reason setting sysroot to anything just works!?
        .clang_arg(sysroot.map_or("".to_string(), |s| format!("--sysroot={}", s)))
        .header(format!("{}/include/vp4.h", SUBMODULE))
        .header(format!("{}/include/fp.h", SUBMODULE))
        .header(format!("{}/include/om_decoder.h", SUBMODULE))
        .header(format!("{}/include/delta2d.h", SUBMODULE))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let out_path_str = out_path.to_str().unwrap();

    // We do the build in the out dir instead of in source, since it pollutes
    // the lib directory.
    // Copy the source files to the build directory and build the library.
    // First we copy headers and C files from the submodule.
    for item in [
        format!("{}/{}", SUBMODULE, "src"),
        format!("{}/{}", SUBMODULE, "include"),
        "Makefile".to_string(),
    ] {
        run_command(
            "cp",
            vec!["-R".to_string(), item, out_path_str.to_string()],
            None,
        );
    }
    run_command("make", vec!["-C", out_path_str, "libic.a"], env);

    println!("cargo:rustc-link-search=native={}", out_path_str);
    println!("cargo:rustc-link-lib=static={}", LIB);
}
