//! The shape matrix: which value shapes cross the boundary, in which position,
//! for each target language.
//!
//! # Why it exists
//!
//! Whether a given Rust shape survives the trip to C or to Kotlin —
//! `Option<&T>`, `Vec<Option<T>>`, an enum with payloads inside a struct — is
//! otherwise knowable only by reading the generators, or by writing the Rust
//! and finding out. So a user discovers a gap by hitting it, a reviewer cannot
//! tell whether a change closed a hole or moved it, and a regression that
//! quietly drops support for a shape looks like nothing at all.
//!
//! This crate enumerates those combinations, runs each one through the **real**
//! generators, and writes down the answer that came back. It is deliberately not
//! a second opinion about what should work: there is no table of expected
//! support anywhere in here, because a second authority on legality can disagree
//! with the first, and then neither can be trusted.
//!
//! # How the axes are built
//!
//! * **Types** come from the model, not from a list in this crate. The accepted
//!   forms are a closed enum ([`prebindgen_flat::flat::TypeKind`]), and
//!   [`tag::tag_of`] is an exhaustive match over it — so adding a form to the
//!   language breaks this crate until the form has a fixture.
//! * **Declarations** are a separate axis, because the model records a field's
//!   type as *"a named type called `Rec`"* and stops. A cell about a struct
//!   field has to emit a struct. This axis has the same kind of gate:
//!   [`run::kind_of`] is exhaustive over the JNI adapter's own
//!   [`ClassDecl`](prebindgen_jni::ClassDecl), so a fifth class kind also stops
//!   this crate compiling. The C build-script API has no closed kind vocabulary
//!   to gate against and is due to be reworked in the JNI style (#192), so
//!   [`run::to_c`] translates this axis onto it and nothing else here is shaped
//!   around C's current spelling.
//! * **Positions** are parameter, return, struct field and enum payload.
//! * **Targets** are enumerated separately and never merged — C and Kotlin
//!   legitimately answer differently, and one combined verdict would hide
//!   exactly the gaps this exists to find.
//!
//! Every cell that produces Rust is then handed to rustc ([`check`]), because
//! "the generator produced a file" and "the file compiles" are different claims
//! and only the second is worth much. For the C target there is a third:
//! [`header`] asks cbindgen whether that file becomes a header actually
//! declaring the wrapper, since a header is what a C caller is given.
//!
//! What a cell reached is then held as a floor ([`guarantees`]): rising is free,
//! falling fails a test that names the cell. The committed report says *"an
//! answer moved"*; the floor says *"an answer moved **down**"*, which is the
//! half that does not need a reviewer to catch it.
//!
//! Run it with `cargo run -p shape-matrix`, which rewrites `REPORT.md`; add
//! `-- --update-guarantees` to raise the floors to what the run achieved.

pub mod check;
pub mod corpus;
pub mod guarantees;
pub mod header;
pub mod report;
pub mod run;
pub mod tag;

use corpus::{Need, Shape, SHAPES};
use prebindgen::SourceLocation;
use prebindgen_flat::flat::Flat;
use tag::{tags_in, TypeTag};

/// Where the committed report lives, relative to this crate.
pub const REPORT_PATH: &str = "REPORT.md";

/// A model holding every supporting declaration the corpus can refer to, so a
/// shape's spelling can be classified without building its whole fixture.
fn corpus_model() -> Flat {
    let loc = SourceLocation {
        crate_name: Some("probe".to_string()),
        ..Default::default()
    };
    let sources = [
        Need::Record,
        Need::Handle,
        Need::Sum,
        Need::UnitEnum,
        Need::Error,
    ];
    // `parse_file`, not `parse_str::<Item>`: a supporting declaration may be
    // more than one item — an error type also declares the accessor that
    // renders it.
    let items = sources.iter().flat_map(|need| {
        syn::parse_file(need.source())
            .expect("supporting declaration parses")
            .items
            .into_iter()
            .map(|item| (item, loc.clone()))
            .collect::<Vec<_>>()
    });
    Flat::builder()
        .items(items)
        .build()
        .expect("supporting declarations index")
}

/// The model's reading of one shape's spelling, or `None` if the model refuses
/// the spelling outright.
pub fn classify_one(shape: &Shape) -> Option<prebindgen_flat::flat::TypeRef> {
    let ty: syn::Type = syn::parse_str(shape.spelling).ok()?;
    corpus_model().classify(&ty).ok()
}

/// The model's reading of one spelling, whatever wrote it.
pub fn corpus_model_classify(ty: &syn::Type) -> Option<prebindgen_flat::flat::TypeRef> {
    corpus_model().classify(ty).ok()
}

/// Every shape paired with the type forms it writes.
pub fn classify_corpus() -> Vec<(&'static Shape, Vec<TypeTag>)> {
    let model = corpus_model();
    SHAPES
        .iter()
        .map(|shape| {
            let tags = syn::parse_str::<syn::Type>(shape.spelling)
                .ok()
                .and_then(|ty| model.classify(&ty).ok())
                .map(|reading| tags_in(&reading))
                .unwrap_or_default();
            (shape, tags)
        })
        .collect()
}

#[cfg(test)]
mod tests;
