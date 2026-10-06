//! Generation checks over a small source. Runtime behavior — every value
//! shape crossing both ways on a real JVM — is covered by
//! `examples/covertest-kotlin`; these pin the Kotlin surface and the
//! adapter's refusals.

use prebindgen::SourceLocation;

use crate::{
    data_class, enum_class, expand_param, expand_return, fun, package, ptr_class, sealed_class,
    Generation, JniGen, JniGenBuilder,
};

const SRC: &str = r#"
    pub type Store = inner::Store;
    pub type Sum = inner::Sum;
    pub struct Item { pub id: i64, pub name: String, pub tag: Option<u64> }
    pub enum Level { Low = 0, High = 5 }
    pub enum Event { Idle, Moved(i32), Named { name: String } }
    pub fn store_new() -> Store { todo!() }
    pub fn store_put(s: &mut Store, item: Item) { todo!() }
    pub fn store_items(s: &Store) -> Vec<Item> { todo!() }
    pub fn store_first(s: &Store) -> Option<Item> { todo!() }
    pub fn store_level(s: &Store) -> Level { todo!() }
    pub fn store_events(s: &Store, f: impl Fn(Event) + Send + Sync + 'static) { todo!() }
    pub fn store_sum(s: &Store) -> Sum { todo!() }
    pub fn sum_new(count: i64) -> Sum { todo!() }
    pub fn sum_count(s: &Sum) -> i64 { todo!() }
    pub fn sum_use(s: Sum) -> i64 { todo!() }
    pub type Error = inner::Error;
    pub type Blob = inner::Blob;
    pub fn error_message(e: &Error) -> String { todo!() }
    pub fn sum_try(count: i64) -> Result<Sum, Error> { todo!() }
    pub fn store_sum_raw(s: &Store) -> Sum { todo!() }
    pub fn blob_new(bytes: Vec<u8>) -> Blob { todo!() }
    pub fn blob_put(s: &Store, b: Blob, extra: Option<Blob>) { todo!() }
    pub fn store_watch(s: &Store, on_close: impl Fn() + Send + Sync + 'static) { todo!() }
    pub const LIMIT: i64 = 3;
"#;

fn builder() -> JniGenBuilder {
    let loc = SourceLocation {
        crate_name: Some("src_crate".to_string()),
        ..Default::default()
    };
    let items = syn::parse_file(SRC)
        .unwrap()
        .items
        .into_iter()
        .map(|i| (i, loc.clone()));
    JniGen::builder()
        .items(items)
        .set_package_prefix("io.test")
        .package(
            package!()
                .class(ptr_class!(Store).method(fun!(store_level)))
                .class(data_class!(Item))
                .class(enum_class!(Level))
                .class(sealed_class!(Event))
                .class(ptr_class!(Sum))
                .fun(fun!(store_new))
                .fun(fun!(store_put))
                .fun(fun!(store_items))
                .fun(fun!(store_first))
                .fun(fun!(store_events)),
        )
}

fn kotlin(g: &Generation) -> String {
    g.kotlin()
        .iter()
        .map(|(_, t)| t.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn kotlin_surface() {
    let g = builder().build().unwrap();
    let kt = kotlin(&g);
    assert!(kt.contains("public class Store(initialPtr: Long) : io.test.NativeHandle(initialPtr)"));
    assert!(
        kt.contains("public data class Item(val id: Long, val name: String, val tag: ULong?)"),
        "{kt}"
    );
    assert!(kt.contains("public enum class Level(public val value: Int)"));
    assert!(kt.contains("    LOW(0),\n    HIGH(5);"));
    assert!(kt.contains("public sealed interface Event"));
    assert!(kt.contains("public data object Idle : Event"));
    assert!(kt.contains("public data class Moved(public val v0: Int) : Event"));
    assert!(kt.contains(
        "public fun storeItems(s: io.test.Store, onError: io.test.JniErrorHandler<List<io.test.Item>>): List<io.test.Item>"
    ), "{kt}");
    assert!(kt.contains("public fun storeFirst(s: io.test.Store, onError: io.test.JniErrorHandler<io.test.Item?>): io.test.Item?"));
    // A member keeps its receiver as `this`.
    assert!(kt.contains(
        "public fun storeLevel(onError: io.test.JniErrorHandler<io.test.Level>): io.test.Level"
    ));
    // A callback gets a typed interface and its raw twin.
    assert!(kt.contains("public fun interface EventCallback"));
    assert!(kt.contains("public fun interface EventCallbackRaw"));
    // The native holder carries every extern.
    assert!(kt.contains("internal object JNINative"));
}

#[test]
fn rust_externs_are_named_by_jni_rules() {
    let g = builder().build().unwrap();
    let rs = g.render_rust();
    assert!(
        rs.contains("fn Java_io_test_JNINative_storeNew<'a>("),
        "{rs}"
    );
    assert!(rs.contains("fn Java_io_test_Store_freePtr("));
    // A data class crosses as its fields: no object is read on the Rust side.
    assert!(rs.contains("item_id: ::prebindgen_jni_runtime::jni::sys::jlong"));
    assert!(rs.contains("item_tag__present: ::prebindgen_jni_runtime::jni::sys::jboolean"));
}

#[test]
fn expansions_shape_the_surface() {
    let g = builder()
        .expand(expand_return!(Sum).field(fun!(sum_count)))
        .expand(expand_param!(Sum).variant(fun!(sum_new)).variant_self())
        .package(
            package!()
                .fun(fun!(store_sum))
                .fun(fun!(sum_use).split_on_param("s")),
        )
        .build()
        .unwrap();
    let kt = kotlin(&g);
    // An expanded result goes to a caller-supplied builder.
    assert!(kt.contains("public fun <R> storeSum(s: io.test.Store, onError: io.test.JniErrorHandler<R>, build: io.test.SumBuilder<R>): R"), "{kt}");
    assert!(kt.contains("public fun interface SumBuilder<out R>"));
    // An expanded parameter takes a selector, plus one overload per variant.
    assert!(kt.contains("public fun sumUse(sSel: Int, s0: Long?, s1: io.test.Sum?, onError: io.test.JniErrorHandler<Long>): Long"), "{kt}");
    assert!(kt.contains("public fun sumUse(count: Long, onError: io.test.JniErrorHandler<Long>): Long =\n    sumUse(0, count, null, onError)"), "{kt}");
    assert!(kt.contains(
        "public fun sumUse(s: io.test.Sum, onError: io.test.JniErrorHandler<Long>): Long ="
    ));
}

#[test]
fn undeclared_types_are_refused() {
    let loc = SourceLocation::default();
    let items = syn::parse_file(SRC)
        .unwrap()
        .items
        .into_iter()
        .map(move |i| (i, loc.clone()));
    let err = JniGen::builder()
        .items(items)
        .set_package_prefix("io.test")
        .package(package!().fun(fun!(store_new)))
        .build()
        .err()
        .unwrap();
    assert!(err.0.contains("`Store` is not declared"), "{}", err.0);
}

#[test]
fn zenoh_shaped_declarations() {
    let g = builder()
        // A type-level expansion: `sum_count` reads its field, so it is an
        // accessor and returns the value itself; so does a fallible
        // function returning the type.
        .expand(expand_return!(Sum).field(fun!(sum_count)).field_self())
        // An error type with no class: its handler is in the base package.
        .expand(expand_return!(Error).field(fun!(error_message).name("message")))
        // A single constructor and no handle variant: no selector.
        .expand(expand_param!(Blob).variant(fun!(blob_new)))
        .package(
            package!()
                .class(ptr_class!(Blob))
                .fun(fun!(sum_count))
                .fun(fun!(sum_try))
                .fun(fun!(blob_put))
                .fun(fun!(store_watch))
                .constant(crate::ConstDecl::named("LIMIT").name("MAX")),
        )
        .build()
        .unwrap();
    let kt = kotlin(&g);
    assert!(
        kt.contains(
            "public fun sumCount(s: io.test.Sum, onError: io.test.JniErrorHandler<Long>): Long"
        ),
        "{kt}"
    );
    assert!(kt.contains("public fun sumTry(count: Long, onBindingError: io.test.JniErrorHandler<io.test.Sum>, onError: io.test.ErrorHandler<io.test.Sum>): io.test.Sum"), "{kt}");
    assert!(kt.contains("public fun interface ErrorHandler<out R>"));
    assert!(kt.contains("public fun blobPut(s: io.test.Store, b: ByteArray, extra: ByteArray?, onError: io.test.JniErrorHandler<Unit>): Unit"), "{kt}");
    // A callback with no arguments.
    assert!(kt.contains("public fun interface VoidCallback"));
    // A constant built at run time, renamed.
    assert!(kt.contains("public val MAX: Long by lazy"), "{kt}");
}

#[test]
fn value_form_overrides_name_real_fields() {
    let src = r#"
        pub type Store = inner::Store;
        pub struct Parts { pub id: i64, pub name: String }
        pub fn store_parts(s: &Store) -> Parts { todo!() }
        pub fn store_open() -> Store { todo!() }
    "#;
    let items = syn::parse_file(src)
        .unwrap()
        .items
        .into_iter()
        .map(|i| (i, SourceLocation::default()));
    let build = |form: crate::FieldsDecl| {
        JniGen::builder()
            .items(items.clone())
            .set_package_prefix("io.test")
            .package(package!().class(ptr_class!(Store)).fun(fun!(store_open)))
            .expand(expand_return!(Store).fields(form))
            .build()
            .err()
            .map(|e| e.0)
    };
    let unknown = build(crate::fields!(store_parts).name("title", "t")).expect("refused");
    assert!(
        unknown.contains("the struct has no field `title`"),
        "{unknown}"
    );
    let twice =
        build(crate::fields!(store_parts).name("id", "a").name("id", "b")).expect("refused");
    assert!(twice.contains("declared twice"), "{twice}");
    assert!(build(crate::fields!(store_parts).name("id", "key")).is_none());
    // A constructor's own expansion is checked too.
    let ctor = JniGen::builder()
        .items(items.clone())
        .set_package_prefix("io.test")
        .package(package!().class(
            ptr_class!(Store).constructor(fun!(store_open).expand_return(
                expand_return!(Store).fields(crate::fields!(store_parts).name("typo", "t")),
            )),
        ))
        .build()
        .err()
        .expect("refused")
        .0;
    assert!(ctor.contains("the struct has no field `typo`"), "{ctor}");
}

#[test]
fn sequence_inputs_preserve_supported_access() {
    for (ty, accepted) in [
        ("Vec<u8>", true),
        ("&Vec<u8>", true),
        ("&[u8]", true),
        ("&mut Vec<u8>", false),
        ("&mut [u8]", false),
        ("[u8]", false),
        ("String", true),
        ("&String", true),
        ("&str", true),
        ("&mut String", false),
        ("&mut str", false),
        ("str", false),
        ("Cow<'static, str>", true),
        ("Cow<'static, [u8]>", true),
    ] {
        let source = syn::parse_file(&format!("pub fn inspect(x: {ty}) {{}}")).unwrap();
        let result = JniGen::builder()
            .items(
                source
                    .items
                    .into_iter()
                    .map(|item| (item, SourceLocation::default())),
            )
            .set_package_prefix("io.test")
            .package(package!().fun(fun!(inspect)))
            .build();
        assert_eq!(result.is_ok(), accepted, "{ty}: {:?}", result.err());
    }
}

#[test]
fn methods_over_the_jvm_slot_limit_are_refused() {
    // Each `i64` is a Kotlin `Long`: two JVM slots. Every generated method
    // has a receiver, one more.
    let fields = |n: usize, prefix: &str| -> String {
        (0..n)
            .map(|i| format!("pub {prefix}{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let getters = |n: usize| -> String {
        (0..n)
            .map(|i| format!("pub fn row_g{i}(r: &Row) -> i64 {{ todo!() }}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let src = format!(
        "pub struct Half64 {{ {} }} pub struct Half63 {{ {} }} \
         pub struct Fit {{ pub a: Half64, pub b: Half63 }} \
         pub struct Wide {{ pub a: Half64, pub b: Half64 }} \
         pub fn take_fit({}, last: i32) -> i64 {{ todo!() }} \
         pub fn take_over({}) -> i64 {{ todo!() }} \
         pub fn make_fit() -> Fit {{ todo!() }} pub fn make_wide() -> Wide {{ todo!() }} \
         pub type Row = inner::Row; {} pub fn row_count(r: &Row) -> i32 {{ todo!() }} \
         pub fn rows() -> Vec<Row> {{ todo!() }}",
        fields(64, "f"),
        fields(63, "f"),
        fields(126, "a").replace("pub ", ""),
        fields(127, "a").replace("pub ", ""),
        getters(127),
    );
    let build = |f: crate::FunctionDecl, row_getters: usize, row_count: bool| {
        let items = syn::parse_file(&src).unwrap().items.into_iter();
        let mut row = expand_return!(Row);
        for i in 0..row_getters {
            row = row.field(crate::FunctionDecl::new(
                syn::parse_str(&format!("row_g{i}")).unwrap(),
            ));
        }
        if row_count {
            row = row.field(fun!(row_count));
        }
        JniGen::builder()
            .items(items.map(|i| (i, SourceLocation::default())))
            .set_package_prefix("io.test")
            .expand(row)
            .package(
                package!()
                    .class(data_class!(Half64))
                    .class(data_class!(Half63))
                    .class(data_class!(Fit))
                    .class(data_class!(Wide))
                    .class(ptr_class!(Row))
                    .fun(f),
            )
            .build()
    };
    let refused = |r: Result<Generation, crate::Error>, what: &str| {
        let e = r.err().expect("refused").0;
        assert!(e.contains(what), "{e}");
    };

    // The native method: the receiver, the parameters, and only the sinks
    // it has — here just the error sink. 1 + 252 + 1 + 1 = 255.
    assert!(build(fun!(take_fit), 0, false).is_ok());
    // 1 + 254 + 1 = 256.
    refused(
        build(fun!(take_over), 0, false),
        "`JNINative.takeOver` would take 256 JVM argument slots",
    );

    // A result sink takes every leaf: 1 + 254 = 255, then 1 + 256 = 257.
    assert!(build(fun!(make_fit), 0, false).is_ok());
    refused(
        build(fun!(make_wide), 0, false),
        "`__Sink_Wide.run` would take 257 JVM argument slots",
    );

    // A typed folder takes the accumulator and every field, while its
    // columns interface takes one array per leaf: 1 + 1 + 252 + 1 = 255 ...
    assert!(build(fun!(rows), 126, true).is_ok());
    // ... then 1 + 1 + 254 = 256, though the columns need only 129.
    refused(
        build(fun!(rows), 127, false),
        "Folder.run` would take 256 JVM argument slots",
    );
}
