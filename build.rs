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

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=../wfa2/lib");
    println!("cargo:rustc-link-lib=wfa");
    println!("cargo:rustc-link-lib=omp");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=../wfa2/lib/libwfa.a");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("../wfa2/wavefront/wavefront_align.h")
        .clang_arg("-I../wfa2")
        .allowlist_function("wavefront_.*")
        .allowlist_var("wavefront_.*")
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
    #[cfg(feature = "wfa")]
    wfa();
    #[cfg(feature = "edlib")]
    edlib();
}
