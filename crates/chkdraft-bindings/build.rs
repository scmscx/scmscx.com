extern crate bindgen;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let chkdraft_dir = Path::new(&manifest_dir).join("Chkdraft");
    let map_gfx_utils_dir = chkdraft_dir.join("src/map_gfx_utils");
    let build_dir = out_dir.join("chkdraft_build");

    // Create build directory
    std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");

    println!("cargo:rerun-if-changed=src/chkdraft_wrapper.h");
    println!("cargo:rerun-if-changed=src/chkdraft_wrapper.cpp");
    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");

    // Watch all files in the Chkdraft directory for changes
    watch_directory_recursively(&chkdraft_dir);

    // Find vcpkg installation
    let vcpkg_root = env::var("VCPKG_ROOT").ok().or_else(|| {
        let home = env::var("HOME").unwrap_or_default();
        let candidates = [
            format!("{}/vcpkg", home),
            "/usr/local/vcpkg".to_string(),
            "/opt/vcpkg".to_string(),
        ];
        candidates.into_iter().find(|c| Path::new(c).exists())
    });

    // Configure cmake
    let mut cmake_args = vec![
        "-S".to_string(),
        map_gfx_utils_dir.to_str().unwrap().to_string(),
        "-B".to_string(),
        ".".to_string(),
        "-DCMAKE_BUILD_TYPE=Release".to_string(),
        "-DCMAKE_POSITION_INDEPENDENT_CODE=ON".to_string(),
    ];

    let vcpkg_root = vcpkg_root.unwrap_or_else(|| {
        panic!(
            "vcpkg not found. Set VCPKG_ROOT or install vcpkg to ~/vcpkg:\n\
             \n  git clone https://github.com/microsoft/vcpkg ~/vcpkg\
             \n  ~/vcpkg/bootstrap-vcpkg.sh\
             \n  export VCPKG_ROOT=~/vcpkg"
        );
    });

    let cmake_toolchain = format!("{}/scripts/buildsystems/vcpkg.cmake", vcpkg_root);
    if Path::new(&cmake_toolchain).exists() {
        cmake_args.push(format!("-DCMAKE_TOOLCHAIN_FILE={}", cmake_toolchain));
        eprintln!("Using vcpkg toolchain: {}", cmake_toolchain);
    }

    // Run cmake configure
    eprintln!("Running cmake configure in {:?}", build_dir);
    let mut cmake_cmd = Command::new("cmake");
    cmake_cmd
        .current_dir(&build_dir)
        .args(&cmake_args)
        .env("VCPKG_ROOT", &vcpkg_root);

    let cmake_output = cmake_cmd.output().expect("Failed to run cmake configure");

    if !cmake_output.status.success() {
        eprintln!("=== CMAKE CONFIGURE FAILED ===");
        eprintln!("stdout: {}", String::from_utf8_lossy(&cmake_output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&cmake_output.stderr));
        eprintln!("");
        eprintln!("=== BUILD REQUIREMENTS ===");
        eprintln!("The chkdraft-bindings crate requires vcpkg with several C++ libraries.");
        eprintln!("");
        eprintln!("  git clone https://github.com/microsoft/vcpkg ~/vcpkg");
        eprintln!("  ~/vcpkg/bootstrap-vcpkg.sh");
        eprintln!("  export VCPKG_ROOT=~/vcpkg");
        panic!("cmake configure failed");
    }

    // Build MapGfxUtils
    eprintln!("Running cmake build...");
    let build_output = Command::new("cmake")
        .current_dir(&build_dir)
        .args(["--build", ".", "--config", "Release", "-j"])
        .output()
        .expect("Failed to run cmake build");

    if !build_output.status.success() {
        eprintln!("=== CMAKE BUILD FAILED ===");
        eprintln!("stdout: {}", String::from_utf8_lossy(&build_output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&build_output.stderr));
        panic!("cmake build failed for MapGfxUtils");
    }

    // Parse cmake-generated build info
    let build_info_path = build_dir.join("rust_build_info.txt");
    let build_info = parse_build_info(&build_info_path);

    for dir in &build_info.lib_dirs {
        println!("cargo:rustc-link-search=native={}", dir.display());
    }
    for lib in &build_info.static_libs {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    // Compile the wrapper
    let mut cc_build = cc::Build::new();
    cc_build
        .cpp(true)
        .std("c++20")
        .file("src/chkdraft_wrapper.cpp")
        .include(chkdraft_dir.join("src"))
        .include(chkdraft_dir.join("src/mapping_core/opengl"))
        .define("_UNICODE", None)
        .define("UNICODE", None)
        .define("NOMINMAX", None)
        .define("CASCLIB_UNICODE", None)
        .define("STORMLIB_NO_AUTO_LINK", None)
        .opt_level(3)
        .define("NDEBUG", None)
        // Suppress warnings from vendored Chkdraft code
        .flag_if_supported("-Wno-unknown-pragmas")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-reorder")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-overloaded-virtual")
        .flag_if_supported("-Wno-parentheses");

    // Add include paths from cmake-generated build info
    for dir in &build_info.include_dirs {
        cc_build.include(dir);
    }

    cc_build.compile("chkdraft_wrapper");

    for lib in &build_info.system_libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("src/chkdraft_wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("chk_.*")
        .allowlist_type("Chk.*")
        .allowlist_var("CHK_.*")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    eprintln!("chkdraft-bindings build completed successfully!");
}

struct BuildInfo {
    include_dirs: Vec<PathBuf>,
    lib_dirs: Vec<PathBuf>,
    static_libs: Vec<String>,
    system_libs: Vec<String>,
}

/// Parse the cmake-generated rust_build_info.txt file.
fn parse_build_info(path: &Path) -> BuildInfo {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

    let mut info = BuildInfo {
        include_dirs: Vec::new(),
        lib_dirs: Vec::new(),
        static_libs: Vec::new(),
        system_libs: Vec::new(),
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(dir) = line.strip_prefix("INCLUDE_DIR=") {
            let dir = dir.trim();
            if !dir.is_empty() && Path::new(dir).exists() {
                info.include_dirs.push(PathBuf::from(dir));
            }
        } else if let Some(dir) = line.strip_prefix("LIB_DIR=") {
            let dir = dir.trim();
            if !dir.is_empty() && Path::new(dir).exists() {
                info.lib_dirs.push(PathBuf::from(dir));
            }
        } else if let Some(lib) = line.strip_prefix("STATIC_LIB=") {
            let lib = lib.trim();
            if !lib.is_empty() {
                info.static_libs.push(lib.to_string());
            }
        } else if let Some(lib) = line.strip_prefix("SYSTEM_LIB=") {
            let lib = lib.trim();
            if !lib.is_empty() {
                info.system_libs.push(lib.to_string());
            }
        }
    }

    info
}

/// Recursively watch all files in a directory for changes.
/// This ensures cargo rebuilds when any source file, header, CMakeLists.txt,
/// or library file in the Chkdraft directory changes.
fn watch_directory_recursively(dir: &Path) {
    if !dir.exists() {
        return;
    }

    // Watch the directory itself
    println!("cargo:rerun-if-changed={}", dir.display());

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // Skip build directories and hidden directories
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if dir_name.starts_with('.') || dir_name == "build" || dir_name == "out" {
                continue;
            }
            watch_directory_recursively(&path);
        } else if path.is_file() {
            // Watch source files, headers, cmake files, and libraries
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext {
                    "cpp" | "c" | "h" | "hpp" | "hxx" | "cxx" | "cc" |
                    "cmake" | "txt" | "json" |  // CMakeLists.txt, vcpkg.json, etc.
                    "a" => {
                        println!("cargo:rerun-if-changed={}", path.display());
                    }
                    _ => {}
                }
            }
            // Also watch CMakeLists.txt specifically (no extension check needed)
            if path.file_name().and_then(|n| n.to_str()) == Some("CMakeLists.txt") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
