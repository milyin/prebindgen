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
    assert!(kt.contains("public fun sumUse(sSel: Int, s00: Long?, s1: io.test.Sum?, onError: io.test.JniErrorHandler<Long>): Long"));
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
