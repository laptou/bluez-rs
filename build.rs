extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // tell cargo to tell rustc to link the bluez shared library
    println!("cargo:rustc-link-lib=bluetooth");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.
        write_to_file(out_path.join("bt.rs"))
        .expect("Couldn't write bindings!");
}
