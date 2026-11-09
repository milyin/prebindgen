use prebindgen_declarations::{expand_param, expand_return, fun};
use prebindgen_jni::{package, ptr_class, JniGen};
fn main() {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let generated = JniGen::builder()
        .source(type_group_flat::PREBINDGEN_OUT_DIR)
        .set_package_prefix("bench.types")
        .set_jni_native_init(r#"System.loadLibrary("type_group_bench")"#)
        .package(
            package!()
                .class(ptr_class!(A))
                .fun(fun!(a_new))
                .fun(fun!(a_read))
                .class(ptr_class!(B))
                .fun(fun!(b_new))
                .fun(fun!(b_read))
                .class(ptr_class!(C))
                .fun(fun!(c_new))
                .fun(fun!(c_read))
                .class(ptr_class!(D))
                .fun(fun!(d_new))
                .fun(fun!(d_read))
                .class(ptr_class!(Token))
                .fun(fun!(token_new))
                .fun(fun!(token_read))
                .fun(fun!(control))
                .fun(fun!(live))
                .fun(
                    fun!(read_one)
                        .expand_param("values", expand_param!(i64).variant(fun!(a_read)).bulk()),
                )
                .fun(
                    fun!(mut_one)
                        .expand_param("values", expand_param!(i64).variant(fun!(a_mut)).bulk()),
                )
                .fun(
                    fun!(mut_nested)
                        .expand_param("values", expand_param!(i64).variant(fun!(a_mut)).bulk()),
                )
                .fun(
                    fun!(mixed_shared)
                        .expand_param("a", expand_param!(i64).variant(fun!(a_read)).bulk())
                        .expand_param("b", expand_param!(i64).variant(fun!(b_read)).bulk())
                        .expand_param("c", expand_param!(i64).variant(fun!(c_read)).bulk())
                        .expand_param("d", expand_param!(i64).variant(fun!(d_read)).bulk()),
                )
                .fun(
                    fun!(mixed_mut)
                        .expand_param("a", expand_param!(i64).variant(fun!(a_mut)).bulk())
                        .expand_param("b", expand_param!(i64).variant(fun!(b_mut)).bulk())
                        .expand_param("c", expand_param!(i64).variant(fun!(c_mut)).bulk())
                        .expand_param("d", expand_param!(i64).variant(fun!(d_mut)).bulk()),
                )
                .fun(
                    fun!(mixed_same)
                        .expand_param("reads", expand_param!(i64).variant(fun!(a_read)).bulk())
                        .expand_param("writes", expand_param!(i64).variant(fun!(a_mut)).bulk()),
                )
                .fun(
                    fun!(owned_echo)
                        .expand_param("values", expand_param!(A).variant_self().bulk())
                        .expand_return(expand_return!(A).field_self_into().bulk()),
                ),
        )
        .build()
        .expect("generate type benchmark");
    generated.write_rust(out.join("bindings.rs")).unwrap();
    generated.write_kotlin(&out.join("kotlin")).unwrap();
}
