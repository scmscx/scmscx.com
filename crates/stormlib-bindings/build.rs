extern crate bindgen;

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    assert!(Command::new("cmake")
        .current_dir("StormLib")
        .args(&["CMakeLists.txt"])
        .status()
        .expect("failed to cmake")
        .success());

    assert!(Command::new("make")
        .current_dir("StormLib")
        // .env("LUA_DIR", lua_dir)
        .status()
        .expect("failed to make!")
        .success());

    assert!(Command::new("cmake")
        .current_dir("bzip2")
        .args(&["CMakeLists.txt", "-DENABLE_STATIC_LIB=1"])
        .status()
        .expect("failed to cmake")
        .success());

    assert!(Command::new("make")
        .current_dir("bzip2")
        // .env("LUA_DIR", lua_dir)
        .status()
        .expect("failed to make!")
        .success());

    assert!(Command::new("bash")
        .current_dir("zlib")
        .args(&["./configure"])
        .status()
        .expect("failed to configure")
        .success());

    assert!(Command::new("make")
        .current_dir("zlib")
        .status()
        .expect("failed to make!")
        .success());

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/wrapper.hpp");
    // println!("cargo:rerun-if-changed=StormLib"); this causes incremental build to not work, it's overly sensitive.
    println!("cargo:rerun-if-changed=StormLib/libstorm.a");

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&dir).join("StormLib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&dir).join("bzip2").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&dir).join("zlib").display()
    );
    println!("cargo:rustc-link-search=native=vendor");
    println!("cargo:rustc-link-lib=static=storm");
    // println!("cargo:rustc-link-lib=static=bz2_static");
    // println!("cargo:rustc-link-lib=static=z");
    println!("cargo:rustc-link-lib=stdc++");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/wrapper.hpp");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/wrapper.hpp")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
