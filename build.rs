// ! make sure that astar-pairwaise-aligner and WFA2-lib are on the same level of directories
// ! to compile WFA2-lib do
// ! $ cd WFA2-lib
// ! $ make lib_wfa
// ! make sure you have make package installed

extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=../WFA2-lib/lib");
    println!("cargo:rustc-link-lib=wfa");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("../WFA2-lib/wavefront/wavefront_align.h")
        .clang_arg("-I../WFA2-lib")
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
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
