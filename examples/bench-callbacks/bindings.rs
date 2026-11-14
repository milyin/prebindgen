use prebindgen_jni::{data_class, package, ptr_class, JniGen};
use DECLARATIONS_CRATE::fun;
fn main() {
    let generated = JniGen::builder()
        .source(bench_flat::PREBINDGEN_OUT_DIR)
        .set_package_prefix("bench")
        .set_jni_native_init(r#"System.loadLibrary("callback_bench")"#)
        .package(
            package!()
                .class(ptr_class!(ScalarHandler))
                .class(ptr_class!(RecordHandler))
                .class(ptr_class!(Owned))
                .class(ptr_class!(OwnedHandler))
                .class(data_class!(Record))
                .fun(fun!(scalar_handler_new))
                .fun(fun!(scalar_fire))
                .fun(fun!(record_handler_new))
                .fun(fun!(record_fire))
                .fun(fun!(owned_new))
                .fun(fun!(owned_value))
                .fun(fun!(owned_handler_new))
                .fun(fun!(owned_fire)),
        )
        .build()
        .expect("generate");
    generated
        .write_rust(std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .unwrap();
    generated
        .write_kotlin(&std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("kotlin"))
        .unwrap();
}
