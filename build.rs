//! Make sure to clone both WFA and Edlib to the sibling directory.
//! See
//! $ make wfa
//! and
//! $ make edlib

extern crate bindgen;

#[allow(unused)]
fn wfa() {
    use std::env;
    use std::path::PathBuf;

    // 1. Link instructions for Cargo.

    // The directory of the WFA libraries, added to the search path.
    println!("cargo:rustc-link-search=../wfa2/lib");
    // Link the `wfa-lib` library.
    println!("cargo:rustc-link-lib=wfa");
    // Also link `omp`.
    println!("cargo:rustc-link-lib=omp");
    // Invalidate the built crate whenever the linked library changes.
    println!("cargo:rerun-if-changed=../wfa2/lib/libwfa.a");

    // 2. Generate bindings.

    let bindings = bindgen::Builder::default()
        // Generate bindings for this header file.
        .header("../wfa2/wavefront/wavefront_align.h")
        // Add this directory to the include path to find included header files.
        .clang_arg("-I../wfa2")
        // Generate bindings for all functions starting with `wavefront_`.
        .allowlist_function("wavefront_.*")
        // Generate bindings for all variables starting with `wavefront_`.
        .allowlist_var("wavefront_.*")
        // Invalidate the built crate whenever any of the included header files
        // changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings_wfa.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings_wfa.rs"))
        .expect("Couldn't write bindings!");
}

#[allow(unused)]
fn edlib() {
    use std::env;
    use std::path::PathBuf;

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=../edlib/meson-build");
    println!("cargo:rustc-link-lib=edlib");
    // Edlib depends on c++ libraries.
    println!("cargo:rustc-link-lib=stdc++");
    // Rebuild when the edlib library changes.
    println!("cargo:rerun-if-changed=../edlib/meson-build/libedlib.a");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("../edlib/edlib/include/edlib.h")
        .clang_arg("-I../edlib/edlib")
        .allowlist_function("edlibAlign|edlibFreeAlignResult|edlibDefaultAlignConfig")
        .allowlist_var("EDLIB_STATUS_OK")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings_edlib.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(feature = "wfa")]
    wfa();
    #[cfg(feature = "edlib")]
    edlib();
}
