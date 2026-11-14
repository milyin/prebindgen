//! Reproducible source-shape and generation checks for payload enum support.
use prebindgen_jni::prelude::*;
mod zenoh;

use prebindgen_flat::{Flat, Result};
use prebindgen_jni::{callback, enum_class, package, sealed_class, JniGen};
use prebindgen_jni::{fun, JniInputConfig, JniOutputConfig};
fn items(source: &str) -> Vec<(syn::Item, prebindgen::SourceLocation)> {
    syn::parse_file(source)
        .unwrap()
        .items
        .into_iter()
        .map(|item| (item, Default::default()))
        .collect()
}
fn rejected<T>(label: &str, result: Result<T>, expected: &str) {
    let error = result
        .err()
        .expect("this case requires a separate prerequisite");
    assert!(error.contains(expected), "{label}: {error}");
    println!("{label}: {error}");
}
fn main() -> Result<()> {
    for source in [
        "pub enum Event{Idle,Count(i64)}",
        "pub enum Event{Idle,Packet{id:u64,bytes:Vec<u8>}}",
        "pub enum Event{Unit,Tuple(),Named{}}",
    ] {
        let flat = Flat::<()>::parse(items(source))?;
        assert_eq!(flat.payload_enums.len(), 1);
    }
    println!("payload source model: unit, tuple, named and empty constructor forms succeed");
    let source = "pub enum State{Idle=4,Ready=9}pub fn echo(value:State)->State{value}";
    let c = prebindgen_c::Cbindgen::new(Flat::parse(items(source))?)
        .enum_type("state", syn::parse_quote!(State))
        .function(prebindgen_c::fun!(echo))
        .build()?;
    syn::parse_file(&c.rust).map_err(|e| e.to_string())?;
    let jni = JniGen::builder()
        .items(items(source))
        .set_package_prefix("example")
        .package(package!().class(enum_class!(State)).fun(fun!(echo)))
        .build()?;
    assert!(jni.kotlin["example/State.kt"].contains("enum class State"));
    let source = "pub fn register(cb:impl Fn(Result<(),String>)->Result<i64,String>){}";
    let jni = JniGen::builder()
        .items(items(source))
        .set_package_prefix("example")
        .package(
            package!()
                .class(sealed_class!(Result<(),String>).name("Notice"))
                .class(sealed_class!(Result<i64,String>).name("Decision"))
                .class(
                    callback!(impl Fn(Result<(), String>) -> Result<i64, String>).name("Handler"),
                )
                .fun(fun!(register)),
        )
        .build()?;
    syn::parse_file(&jni.rust).map_err(|e| e.to_string())?;
    println!("callback Result arguments and returns: ordinary sealed values succeed");
    let source = "pub fn echo(value:Result<i64,String>)->Result<i64,String>{value}";
    let c = prebindgen_c::Cbindgen::new(Flat::parse(items(source))?)
        .tagged_union("decision", syn::parse_quote!(Result<i64,String>))
        .function(prebindgen_c::fun!(echo).result_as_value())
        .panic()
        .build()?;
    syn::parse_file(&c.rust).map_err(|e| e.to_string())?;
    let jni = JniGen::builder()
        .items(items(source))
        .set_package_prefix("example")
        .package(
            package!().fun(
                fun!(echo)
                    .result_as_value()
                    .expand_param("value", JniInputConfig::enum_fields([]))
                    .expand_return(JniOutputConfig::enum_fields([])),
            ),
        )
        .build()?;
    assert!(!jni.kotlin.contains_key("example/Decision.kt"));
    assert!(jni.kotlin["example/PayloadTags.kt"].contains("_OK_TAG"));
    println!("Result as data: C carrier and fields-only JNI need no error routing");
    zenoh::run()?;
    Ok(())
}
