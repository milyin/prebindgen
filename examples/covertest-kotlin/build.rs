include!("../declarations.rs");
use prebindgen_jni::{package, ptr_class, JniGen};
fn main() {
    println!("cargo:rerun-if-changed=../declarations.rs");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let mut package = package!()
        .class(ptr_class!(KeyExpr))
        .class(ptr_class!(Payload))
        .class(ptr_class!(Vec<u8>).name(String::from("Bytes")));
    for f in functions() {
        package = package.fun(f);
    }
    package = package
        .fun(fun!(echo_key))
        .fun(
            fun!(echo_key_raw)
                .name(String::from("echoRaw"))
                .expand_param(String::from("k"), expand_param!(KeyExpr).variant_self())
                .expand_return(expand_return!(KeyExpr).field_self()),
        )
        .fun(
            fun!(keyexpr_pair).expand_return(
                expand_return!(KeyExpr)
                    .field_self()
                    .field(fun!(keyexpr_to_string)),
            ),
        );
    let mut builder = JniGen::builder()
        .source(demo_flat::PREBINDGEN_OUT_DIR)
        .package(package)
        .package(package!("extra").fun(fun!(noop_extra).name("doNothing")));
    for expansion in expansions() {
        builder = builder.expand(expansion);
    }
    // Settings after declarations also exercise order independence.
    let generation: JniGen = builder
        .set_package_prefix(String::from("io.prebindgen.demo"))
        .set_jni_native_init(String::from("System.loadLibrary(\"covertest_kotlin\")"))
        .build()
        .unwrap();
    let written = generation.write_rust("bindings.rs").unwrap();
    assert_eq!(written, output.join("bindings.rs"));
    let files = generation.write_kotlin(&output.join("kotlin")).unwrap();
    assert!(!files.is_empty());
    println!(
        "cargo:warning=Kotlin sources: {}",
        output.join("kotlin").display()
    );
}
