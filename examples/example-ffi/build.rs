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
    let generate_for_target = |target: &str| {
        let bar = if target.contains("x86_64") {
            "#[prebindgen(\"structs\")]\n#[repr(C)]\npub struct Bar { pub x86_64_field: u64 }".to_string()
        } else if target.contains("aarch64") {
            "#[prebindgen(\"structs\")]\n#[repr(C)]\npub struct Bar { pub aarch64_field: u64 }".to_string()
        } else {
            panic!("Unsupported architecture: {}", target);
        };
        // write with append
        let bar_rs = format!("{}/bar.rs", out_dir);
        prebindgen::trace!("Generating {bar_rs} for target: {target}");
        std::fs::write(bar_rs, bar).unwrap();
    };
    if let Ok(target) = std::env::var("PREBINDGEN_TARGET") {
        generate_for_target(&target);
    }
    generate_for_target(&std::env::var("TARGET").unwrap());
}
