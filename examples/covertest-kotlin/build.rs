include!("../configure.rs");
use prebindgen_jni::{expand_param, expand_return, fun, package, ptr_class, JniGen};
fn main() {
    println!("cargo:rerun-if-changed=../configure.rs");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let generation = JniGen::builder()
        .source(demo_flat::PREBINDGEN_OUT_DIR)
        .configure(configure)
        .set_package_prefix("io.prebindgen.demo")
        .set_jni_native_init("System.loadLibrary(\"covertest_kotlin\")")
        .expand(
            expand_param!(KeyExpr)
                .variant(fun!(keyexpr_from_string))
                .variant_self(),
        )
        .expand(expand_return!(KeyExpr).field(fun!(keyexpr_to_string)))
        .package(
            package!()
                .class(ptr_class!(KeyExpr))
                .class(ptr_class!(Payload))
                .class(ptr_class!(Vec<u8>).name("Bytes"))
                .fun(fun!(foo))
                .fun(fun!(keyexpr_from_string))
                .fun(fun!(keyexpr_text))
                .fun(fun!(bytes_from_slice))
                .fun(fun!(bytes_read))
                .fun(fun!(payload_from_string))
                .fun(fun!(payload_from_slice))
                .fun(fun!(payload_text))
                .fun(fun!(noop))
                .fun(fun!(echo_key))
                .fun(
                    fun!(echo_key_raw)
                        .name("echoRaw")
                        .expand_param("k", expand_param!(KeyExpr).variant_self())
                        .expand_return(expand_return!(KeyExpr).field_self()),
                )
                .fun(
                    fun!(keyexpr_pair).expand_return(
                        expand_return!(KeyExpr)
                            .field_self()
                            .field(fun!(keyexpr_to_string)),
                    ),
                ),
        )
        .package(package!("extra").fun(fun!(noop_extra).name("doNothing")))
        .build()
        .unwrap();
    generation.write_rust(output.join("bindings.rs")).unwrap();
    generation.write_kotlin(output.join("kotlin")).unwrap();
    println!(
        "cargo:warning=Kotlin sources: {}",
        output.join("kotlin").display()
    );
}
