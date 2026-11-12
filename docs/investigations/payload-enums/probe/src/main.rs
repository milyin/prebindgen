//! Reproduce the current source-model and callback-Result restrictions.
use prebindgen_declarations::{fun, Input, Output};
use prebindgen_flat::{Flat, Result};
use prebindgen_jni::{callback, enum_class, package, JniGen};

mod zenoh;

fn items<L: Default>(source: &str) -> Vec<(syn::Item, L)> {
    syn::parse_file(source)
        .unwrap()
        .items
        .into_iter()
        .map(|item| (item, Default::default()))
        .collect()
}

fn rejected<T>(label: &str, result: Result<T>, expected: &str) {
    let error = result.err().expect("the baseline should reject this case");
    assert!(error.contains(expected), "{label}: {error}");
    println!("{label}: {error}");
}

fn main() -> Result<()> {
    for (label, source) in [
        ("tuple payload", "pub enum Event { Idle, Count(i64) }"),
        (
            "named payload",
            "pub enum Event { Idle, Packet { id: u64, bytes: Vec<u8> } }",
        ),
    ] {
        rejected(
            label,
            Flat::<()>::parse(items(source)),
            "only fieldless enum variants are supported",
        );
    }

    let source = "pub enum State { Idle = 4, Ready = 9 }
                  pub fn echo(value: State) -> State { value }";
    let c = prebindgen_c::Cbindgen::new(Flat::parse(items(source))?)
        .enum_type("state", syn::parse_quote!(State))
        .function(fun!(echo))
        .build()?;
    syn::parse_file(&c.rust).map_err(|e| e.to_string())?;
    let jni = JniGen::builder()
        .items(items(source))
        .set_package_prefix("example")
        .package(package!().class(enum_class!(State)).fun(fun!(echo)))
        .build()?;
    syn::parse_file(&jni.rust).map_err(|e| e.to_string())?;
    assert!(jni.kotlin["example/State.kt"].contains("enum class State"));
    println!("fieldless control: C and Kotlin generation succeeds");

    let argument = JniGen::builder()
        .items(items(
            "pub fn register(cb: impl Fn(Result<i64, String>)) {}",
        ))
        .set_package_prefix("example")
        .package(
            package!()
                .class(callback!(impl Fn(Result<i64, String>)).name("Handler"))
                .fun(fun!(register)),
        )
        .build();
    rejected(
        "callback argument Result",
        argument,
        "Result-valued callback arguments require an explicit value representation",
    );

    let explicit_argument = JniGen::builder()
        .items(items(
            "pub fn register(cb: impl Fn(Result<i64, String>)) {}",
        ))
        .set_package_prefix("example")
        .package(
            package!()
                .class(
                    callback!(impl Fn(Result<i64, String>))
                        .name("Handler")
                        .expand_param(0, Output::value()),
                )
                .fun(fun!(register)),
        )
        .build();
    rejected(
        "explicit callback argument Result",
        explicit_argument,
        "Result-valued callback arguments require an explicit value representation",
    );

    let returned = JniGen::builder()
        .items(items(
            "pub fn register(cb: impl Fn() -> Result<i64, String>) {}",
        ))
        .set_package_prefix("example")
        .package(
            package!()
                .class(callback!(impl Fn() -> Result<i64, String>).name("Handler"))
                .fun(fun!(register)),
        )
        .build();
    rejected(
        "callback return Result",
        returned,
        "Result inputs require an explicit data representation",
    );

    let input = JniGen::builder()
        .items(items("pub fn consume(value: Result<i64, String>) {}"))
        .set_package_prefix("example")
        .package(package!().fun(fun!(consume).expand_param("value", Input::value())))
        .build();
    rejected(
        "explicit Result value input",
        input,
        "Result inputs require an explicit data representation",
    );

    let ordinary = JniGen::builder()
        .items(items(
            "pub fn produce() -> Result<i64, String> { Ok(1) }
             pub fn message(error: &String) -> String { error.clone() }",
        ))
        .set_package_prefix("example")
        .package(package!().fun(fun!(produce).on_error(fun!(message))))
        .build()?;
    syn::parse_file(&ordinary.rust).map_err(|e| e.to_string())?;
    println!("outer function Result control: existing error routing succeeds");
    zenoh::run()?;
    Ok(())
}
