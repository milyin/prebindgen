include!("../declarations.rs");
fn main() {
    println!("cargo:rerun-if-changed=../declarations.rs");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let mut builder = prebindgen_c::Cbindgen::builder()
        .source(demo_flat::PREBINDGEN_OUT_DIR)
        .prefix("demo")
        .handle("key", syn::parse_quote!(KeyExpr))
        .handle("payload", syn::parse_quote!(Payload))
        .handle("bytes", syn::parse_quote!(Vec<u8>));
    for expansion in expansions() {
        builder = builder.expand(expansion);
    }
    for function in functions() {
        builder = builder.function(function);
    }
    let generation = builder.build().unwrap();
    generation.write_rust(output.join("bindings.rs")).unwrap();
    generation.write_header(output.join("demo.h")).unwrap();
    println!(
        "cargo:warning=C header: {}",
        output.join("demo.h").display()
    );
}
