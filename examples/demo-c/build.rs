include!("../configure.rs");
fn main() {
    println!("cargo:rerun-if-changed=../configure.rs");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let mut flat =
        Flat::parse(prebindgen::Source::new(demo_flat::PREBINDGEN_OUT_DIR).items_all()).unwrap();
    configure(&mut flat).unwrap();
    let generation = prebindgen_c::Cbindgen::new(flat)
        .prefix("demo")
        .handle("key", syn::parse_quote!(KeyExpr))
        .handle("payload", syn::parse_quote!(Payload))
        .handle("bytes", syn::parse_quote!(Vec<u8>))
        .build()
        .unwrap();
    generation.write_rust(output.join("bindings.rs")).unwrap();
    generation.write_header(output.join("demo.h")).unwrap();
    println!(
        "cargo:warning=C header: {}",
        output.join("demo.h").display()
    );
}
