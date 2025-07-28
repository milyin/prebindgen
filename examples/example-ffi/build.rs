use std::io::Write;

fn generate_for_target(target: &str, filename: &str, prebindgen: Option<bool>) {
    let arch = if target.contains("x86_64") {
        "x86_64"
    } else if target.contains("aarch64") {
        "aarch64"
    } else {
        panic!("Unsupported architecture: {target}");
    };

    let mut bar= if let Some(skip) = prebindgen {
        format!("#[prebindgen(\"structs\", skip = {skip})]\n").into()
    } else {
        String::new()
    };
    
    bar.push_str(&format!(
        "#[repr(C)]\n#[derive(Copy, Clone, Debug, PartialEq)]\npub struct Bar {{ pub {arch}_field: u64 }}\n"
    ));

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
    //
    // Specially for this situation the "skip" attribute is supported in the `#[prebindgen]` macro.
    // It allows to produce 2 variants of the generated code: one for host platform (std::env::var("TARGET") in build.rs)
    // not intrumented with `#[prebindgen]` macro and one for the target platform (std::env::var("CROSS_TARGET") in build.rs)
    // which is instrumented with `#[prebindgen]` macro but skipped in the host build.
    //
    // E.g. to cross-build for x86_64-unknown-linux-gnu run
    // ```sh
    // CROSS_TARGET=x86_64-unknown-linux-gnu cargo build --target x86_64-unknown-linux-gnu
    // ```
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let bar_rs = format!("{out_dir}/bar.rs");

    let current_target = std::env::var("TARGET").unwrap();
    let cross_target = std::env::var("CROSS_TARGET")
        .ok()
        .filter(|s| s != &current_target);

    std::fs::remove_file(&bar_rs).ok();
    if let Some(ref cross_target) = cross_target {
        // Bar definition for the host, wihtout prebindgen generation
        generate_for_target(&current_target, &bar_rs, None);
        // Bar definition for the target, skipped on the host but generated prebindgen data for the target
        generate_for_target(cross_target, &bar_rs, Some(true));
    } else {
        generate_for_target(&current_target, &bar_rs, Some(false));
    }
}
