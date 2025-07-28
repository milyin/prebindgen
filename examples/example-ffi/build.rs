use std::io::Write;

fn generate_for_target(target: &str, filename: &str) {
    let arch = if target.contains("x86_64") {
        "x86_64"
    } else if target.contains("aarch64") {
        "aarch64"
    } else {
        panic!("Unsupported architecture: {target}");
    };
    
    let bar = format!("#[prebindgen(\"structs\")]\n#[cfg(target_arch = \"{arch}\")]\n#[repr(C)]\n#[derive(Copy, Clone, Debug, PartialEq)]\npub struct Bar {{ pub {arch}_field: u64 }}\n");
    
    prebindgen::trace!("Generating {filename} for target: {target}");
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .unwrap()
        .write_all(bar.as_bytes())
        .unwrap();
}

fn main() {
    prebindgen::init_prebindgen_out_dir();

    // The code below is usually not needed in the typical project. It illustrates the specific case when
    // build.rs generates the source code for specific the target architecture.
    //
    // Simulate the case when part of the source code is generated in example-ffi/build.rs and this code
    // depends on the target architecture.
    // In this case the example-ffi/build.rs is called twice:
    // - once for the host platform as a dependency for example-cbindgen/build.rs, The goal: generate the OUT_DIR/prebindgen.rs file
    // - once for the target platform as a dependency for example-ffi itself. The goal: build the binding library for the target platform.
    // The problem is that on the host platform call the target architecture is unknown, but example-ffi/build.rs should generate the source code for it.
    // To make things work correctly the target architecture must be passed by some independent from cargo environment variable.
    // ( PREBINDGEN_TARGET in this case ).
    // E.g. to cross-build for x86_64-unknown-linux-gnu run
    // ```sh
    // PREBINDGEN_TARGET=x86_64-unknown-linux-gnu cargo build --target x86_64-unknown-linux-gnu
    // ```
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let bar_rs = format!("{out_dir}/bar.rs");
    
    let current_target = std::env::var("TARGET").unwrap();
    let prebindgen_target = std::env::var("PREBINDGEN_TARGET").ok();
    
    std::fs::remove_file(&bar_rs).ok();
    
    if let Some(ref target) = prebindgen_target {
        if target != &current_target {
            generate_for_target(target, &bar_rs);
        }
    }
    generate_for_target(&current_target, &bar_rs);
}
