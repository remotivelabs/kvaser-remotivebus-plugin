use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    if target_os == "linux" {
        println!("cargo:rerun-if-changed=ffi-codegen/kvaser.h");

        let arch = match target_arch.as_str() {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            _ => {
                println!("cargo:warning=Unsupported architecture: {target_arch}");
                return;
            }
        };

        // Priority: KVASER_LIB_PATH (direct override) > OUT_LIB_BASE/<arch> (Makefile output) > default
        let lib_path = env::var("KVASER_LIB_PATH").unwrap_or_else(|_| {
            let lib_base = env::var("OUT_LIB_BASE").unwrap_or_else(|_| "build/lib".to_string());
            format!("{}/{}", lib_base, arch)
        });
        let include_path = format!("{}/include", lib_path);

        println!("cargo:rustc-link-lib=dylib=linlib");
        println!("cargo:rustc-link-lib=dylib=canlib");
        println!("cargo:rustc-link-search=native={}", lib_path);

        // Only set rpath for development builds, not for packages
        // Packages should rely on ldconfig and system library paths
        if env::var("CARGO_PROFILE").as_deref() != Ok("release") {
            println!("cargo:rustc-link-arg=-Wl,-rpath={}", lib_path);
        }

        let bindings = bindgen::Builder::default()
            .header("ffi-codegen/kvaser.h")
            .clang_arg(format!("-I{}", include_path))
            .raw_line("#![cfg_attr(rustfmt, rustfmt_skip)]")
            .raw_line("#![allow(clippy::all)]")
            .raw_line("#![allow(non_snake_case)]")
            .raw_line("#![allow(non_camel_case_types)]")
            .raw_line("#![allow(dead_code)]")
            .raw_line("#![allow(unused_imports)]")
            .raw_line("#![allow(unused_variables)]")
            .raw_line("#![allow(deref_nullptr)]")
            .raw_line("#![allow(unsafe_op_in_unsafe_fn)]")
            .raw_line("#![allow(improper_ctypes)]")
            .raw_line("#![allow(improper_ctypes_definitions)]")
            .raw_line("#![allow(unused_mut)]")
            .raw_line("#![allow(non_upper_case_globals)]")
            .raw_line("#![allow(clippy::too_many_lines)]")
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file("src/kvaser_raw_binding.rs")
            .expect("Couldn't write bindings!");
    } else {
        println!(
            "cargo:warning=Skipping bindgen generation on {target_arch}-{target_os} (only supported on Linux)"
        );
    }
}
