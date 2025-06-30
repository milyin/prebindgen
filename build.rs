// This is a build script that would be used in a project that depends on prebindgen
// It demonstrates how to include the generated prebindgen.rs file

use std::env;
use std::path::Path;

fn main() {
    // The prebindgen.rs file will be created by the prebindgen macro during compilation
    let out_dir = env::var("OUT_DIR").unwrap();
    let prebindgen_path = Path::new(&out_dir).join("prebindgen.rs");
    
    println!("cargo:rerun-if-changed=src/");
    
    // Tell cargo to rerun if the generated file changes
    if prebindgen_path.exists() {
        println!("cargo:rerun-if-changed={}", prebindgen_path.display());
    }
}
