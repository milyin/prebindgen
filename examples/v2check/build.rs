//! Runs the v2 engine over `src/source.rs` for both targets, so **rustc** gets
//! to judge what it emitted.
//!
//! This is `docs/v2`'s worked example, executed: the same two declarations, the
//! same two targets, the same expected wrappers. The engine plans; the adapters
//! in `src/adapters.rs` answer the local questions; the common Rust writer
//! renders. What lands in `OUT_DIR` is compiled by `src/lib.rs`, and the tests
//! there check it against the specification pages themselves.
//!
//! Like `emitcheck`, this crate has no captured source crate: it parses one
//! file and hands its items to `Flat::builder().items(..)`, which is the entry
//! point the unit-test fixtures use.

// One copy of the adapters, compiled both here and into the library.
#[path = "src/adapters.rs"]
mod adapters;

use std::path::{Path, PathBuf};

use adapters::{CPolicy, CTarget, JniClasses, JniPolicy, JniTarget};
use prebindgen_flat::flat::Flat;
use prebindgen_registry_v2::{generate, BindingRequests, DeclaredElement, ElementKind, Generation};

/// The crate name stamped on every item.
const SOURCE_CRATE: &str = "v2check";

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest dir");
    let source = PathBuf::from(&manifest_dir).join("src").join("source.rs");
    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-changed=src/adapters.rs");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));

    let c = generate(model(&source), &CTarget, c_requests(), SOURCE_CRATE)
        .expect("the C binding plans");
    // One table names the Kotlin classes; the declarations and the signatures
    // that mention them both read it.
    let classes = JniClasses::in_package("example").with("Stamp", "Stamp");
    let jni = generate(
        model(&source),
        &JniTarget::new(classes.clone()),
        jni_requests(&classes),
        SOURCE_CRATE,
    )
    .expect("the JNI binding plans");

    let c_path = c.write_rust(out_dir.join("c.rs")).expect("write c.rs");
    let jni_path = jni
        .write_rust(out_dir.join("kotlin.rs"))
        .expect("write kotlin.rs");
    let kotlin_path = out_dir.join("Bindings.kt");
    std::fs::write(&kotlin_path, adapters::write_kotlin(&jni)).expect("write Bindings.kt");

    report(&c, &out_dir, "c");
    report(&jni, &out_dir, "jni");

    println!("cargo:rustc-env=V2CHECK_C={}", c_path.display());
    println!("cargo:rustc-env=V2CHECK_JNI={}", jni_path.display());
    println!("cargo:rustc-env=V2CHECK_KOTLIN={}", kotlin_path.display());
}

/// The source model, built from the one file this crate compiles.
fn model(source: &Path) -> Flat {
    let location = prebindgen::SourceLocation {
        crate_name: Some(SOURCE_CRATE.to_string()),
        ..Default::default()
    };
    let text = std::fs::read_to_string(source).expect("read src/source.rs");
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = syn::parse_file(&text)
        .expect("src/source.rs parses")
        .items
        .into_iter()
        .map(|item| (item, location.clone()))
        .collect();
    Flat::builder()
        .items(items)
        .build()
        .expect("the fixture builds a model")
}

/// The C binding: `Stamp` as a by-value aggregate, `stamp_sum` as an entry
/// point. Both keep the source name, because nothing renamed them.
fn c_requests() -> BindingRequests<CPolicy> {
    let mut requests = BindingRequests::new("c", syn::parse_quote!(source), CPolicy::Scalar);
    let stamp = requests.policy(CPolicy::DataStruct {
        c_name: "Stamp".to_string(),
    });
    requests.type_policies.insert("Stamp".to_string(), stamp);
    let sum = requests.policy(CPolicy::Function {
        symbol: "stamp_sum".to_string(),
    });
    let delta = requests.policy(CPolicy::Function {
        symbol: "stamp_delta".to_string(),
    });
    requests.output(
        DeclaredElement::new(ElementKind::Type, "Stamp", "Stamp", "data_struct"),
        stamp,
    );
    requests.output(
        DeclaredElement::new(ElementKind::Function, "stamp_sum", "stamp_sum", "function"),
        sum,
    );
    requests.output(
        DeclaredElement::new(
            ElementKind::Function,
            "stamp_delta",
            "stamp_delta",
            "function",
        ),
        delta,
    );
    let show = requests.policy(CPolicy::Function {
        symbol: "stamp_show".to_string(),
    });
    requests.output(
        DeclaredElement::new(
            ElementKind::Function,
            "stamp_show",
            "stamp_show",
            "function",
        ),
        show,
    );
    // Positional fields: the aggregate this adapter declares names its members,
    // so it refuses rather than reading `.0` out of a struct whose C form has
    // no such member.
    let pair = requests.policy(CPolicy::DataStruct {
        c_name: "Pair".to_string(),
    });
    requests.type_policies.insert("Pair".to_string(), pair);
    requests.output(
        DeclaredElement::new(ElementKind::Type, "Pair", "Pair", "data_struct"),
        pair,
    );
    requests
}

/// The JNI binding: `Stamp` as an `example.Stamp` object whose properties are
/// read, `stamp_sum` as `example.Bindings.sum`.
fn jni_requests(classes: &JniClasses) -> BindingRequests<JniPolicy> {
    let mut requests = BindingRequests::new("jni", syn::parse_quote!(source), JniPolicy::Scalar);
    let stamp = requests.policy(JniPolicy::DataClass {
        rust: "Stamp".to_string(),
    });
    requests.type_policies.insert("Stamp".to_string(), stamp);
    // A record whose field neither adapter can carry, declared so the report
    // says so rather than the example quietly not mentioning it.
    let reading = requests.policy(JniPolicy::DataClass {
        rust: "Reading".to_string(),
    });
    requests
        .type_policies
        .insert("Reading".to_string(), reading);
    let show = requests.policy(JniPolicy::Function {
        placement: "example.Bindings.show".to_string(),
    });
    let _ = classes;
    let sum = requests.policy(JniPolicy::Function {
        placement: "example.Bindings.sum".to_string(),
    });
    // A second method on the same Kotlin object: one `object Bindings`, two
    // declarations in it.
    let delta = requests.policy(JniPolicy::Function {
        placement: "example.Bindings.delta".to_string(),
    });
    requests.output(
        DeclaredElement::new(ElementKind::Type, "Stamp", "example.Stamp", "data_class"),
        stamp,
    );
    requests.output(
        DeclaredElement::new(
            ElementKind::Function,
            "stamp_sum",
            "example.Bindings.sum",
            "function",
        ),
        sum,
    );
    requests.output(
        DeclaredElement::new(
            ElementKind::Function,
            "stamp_delta",
            "example.Bindings.delta",
            "function",
        ),
        delta,
    );
    // Delivers nothing: its failure route has to terminate without a value.
    requests.output(
        DeclaredElement::new(
            ElementKind::Function,
            "stamp_show",
            "example.Bindings.show",
            "function",
        ),
        show,
    );
    requests.output(
        DeclaredElement::new(
            ElementKind::Type,
            "Reading",
            "example.Reading",
            "data_class",
        ),
        reading,
    );
    requests
}

/// The report is published beside the code, as it is for a real binding.
fn report<P>(generation: &Generation<P>, out_dir: &Path, target: &str) {
    std::fs::write(
        out_dir.join(format!("{target}-report.json")),
        generation.report().to_json(),
    )
    .expect("write the report");
    std::fs::write(
        out_dir.join(format!("{target}-report.md")),
        generation.report().to_markdown(),
    )
    .expect("write the report");
}
