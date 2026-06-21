//! Build script generating Kotlin/JNI bindings for `perftest-flat` using
//! prebindgen's [`prebindgen::lang::JniGen`] adapter. It produces:
//!   * `src/generated_bindings.rs` — the Rust-side JNI wrappers (included by
//!     `src/lib.rs`), and
//!   * `kotlin/generated/**` — the matching typed Kotlin classes.
//!
//! The example contrasts two ways of bringing a `Payload` across the JNI
//! boundary, benchmarked in `kotlin/.../Bench.kt`:
//!   * **one-crossing composition** — `payload_get()` returns a Kotlin
//!     `data class Payload` composed on the Kotlin side via a generated
//!     `fromParts(...)` factory in a single JNI crossing (`.data_class`);
//!   * **naive baseline** — `payload_stored_*()` fetch each field with a
//!     separate JNI call (N crossings), then build `Payload`.
//!
//! `Payload.label` is `Option<Box<String>>` (an opaque-pointer string field);
//! JniGen maps `Box<String>` → Kotlin `String` and `Option<Box<String>>` →
//! `String?` automatically.

use prebindgen::core::Registry;
use prebindgen::lang::JniGen;
use syn::parse_quote as pq;

fn main() {
    let source = prebindgen::Source::new(perftest_flat::PREBINDGEN_OUT_DIR);

    let jni = JniGen::new()
        .source_module(pq!(perftest_flat))
        .package_prefix("io.prebindgen.perftest")
        // Trigger native-library loading from the generated `JNINative` static
        // init (the single choke point through which every JNI call routes).
        .jni_native_init("io.prebindgen.perftest.NativeLibrary.ensureLoaded()")
        // `Payload` as a Kotlin `data class` with a `fromParts` companion factory:
        // returning/accepting it crosses all fields in ONE JNI call and composes
        // the object on the Kotlin side.
        .data_class(pq!(Payload))
        // `Storage` as an opaque Kotlin handle class (`NativeHandle`, closeable);
        // the functions read/write the payload it owns.
        .ptr_class(pq!(Storage))
        .package("storage")
        // One-crossing composition path. Only the value/ref-input put forms map to
        // JNI: `storage_put_by_take` (by-value `Payload`) and `storage_put_by_read`
        // (`&Payload`). The `&mut Payload` / `&mut MaybeUninit<Payload>` out-param
        // forms (`storage_put_by_read_and_update`, `storage_get_into_*`) are C-only —
        // a Kotlin `data class` is an immutable value with no out-param/uninit
        // semantics — so they are left undeclared here.
        //
        // (`storage_callback(s, impl Fn(&Payload))` is also NOT declared: a
        // `data_class` supports by-value / `&T` *input* and by-value *output*
        // (composed via `fromParts`), but not a borrowed data-class delivered to a
        // callback — that needs a `ptr_class` + `flatten_output` builder. The
        // `storage_get` → `fromParts` path already shows Kotlin-side composition.)
        .fun(pq!(storage_new))
        .fun(pq!(storage_get))
        .fun(pq!(storage_put_by_take))
        .fun(pq!(storage_put_by_read))
        // Naive per-field baseline (one JNI call each).
        .fun(pq!(storage_get_id))
        .fun(pq!(storage_get_seq))
        .fun(pq!(storage_get_value))
        .fun(pq!(storage_get_flag))
        .fun(pq!(storage_get_label));

    let mut registry = Registry::from_items(source.items_all()).expect("scan prebindgen items");

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // Rust JNI wrappers → src/generated_bindings.rs (committed; included by lib.rs).
    let rust_dest = std::path::Path::new(&crate_dir)
        .join("src")
        .join("generated_bindings.rs");
    let rust_path = registry
        .write_rust(&jni, &rust_dest)
        .expect("write_rust failed");
    println!(
        "cargo:warning=Generated bindings at: {}",
        rust_path.display()
    );

    // Kotlin classes → kotlin/generated/** (picked up by the Gradle source set).
    let kotlin_root = std::path::Path::new(&crate_dir)
        .join("kotlin")
        .join("generated");
    if let Err(err) = std::fs::remove_dir_all(&kotlin_root) {
        if err.kind() != std::io::ErrorKind::NotFound {
            panic!("cleanup kotlin/generated failed: {err}");
        }
    }
    for path in jni
        .write_kotlin(&registry, &kotlin_root)
        .expect("write_kotlin failed")
    {
        println!("cargo:warning=Wrote {}", path.display());
    }
}
