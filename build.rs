use std::{env, path::PathBuf};
extern crate bindgen;

fn write_wrapper(wrapper_path: &PathBuf) {
    std::fs::write(wrapper_path, "#include \"edlib.h\"").unwrap();
}

fn main() {
    // Ensure sane macOS min version for all child toolchains (clang, cmake, bindgen clang)
    #[cfg(target_os = "macos")]
    env::set_var("MACOSX_DEPLOYMENT_TARGET", "13.0");

    // Configure and build edlib
    let mut cfg = cmake::Config::new("edlib-c");
    cfg.profile("Release")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("CMAKE_POLICY_VERSION_MINIMUM", "3.5");
    #[cfg(target_os = "macos")]
    {
        cfg.define("CMAKE_OSX_ARCHITECTURES", "arm64")
           .define("CMAKE_OSX_DEPLOYMENT_TARGET", "13.0");
    }
    let dst = cfg.build(); // installs into {dst}/include and {dst}/lib

    // Link the static library that CMake installed
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=edlib");
    if cfg!(target_os = "macos") { println!("cargo:rustc-link-lib=c++"); }
    else { println!("cargo:rustc-link-lib=stdc++"); }

    // Generate wrapper and bindings
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wrapper = out.join("wrapper.h");
    write_wrapper(&wrapper);

    // >>> This is the important change: include the CMake install include dir
    let edlib_include = dst.join("include");

    let bindings = bindgen::Builder::default()
        .header(wrapper.to_str().unwrap())
        .clang_arg(format!("-I{}", edlib_include.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    println!("cargo:rerun-if-changed=edlib-c/include/edlib.h");
    println!("cargo:rerun-if-changed=edlib-c/CMakeLists.txt");
    println!("cargo:rerun-if-changed=build.rs");

    bindings
        .write_to_file(out.join("bindings.rs"))
        .expect("Could not write bindings");
}
