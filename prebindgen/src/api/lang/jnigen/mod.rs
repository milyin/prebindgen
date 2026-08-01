//! JNI / Kotlin language adapter — the [`JniGenBuilder`] back-end.
//!
//! Sibling of [`crate::api::lang::cbindgen`]: it implements the
//! language-agnostic [`crate::api::core::prebindgen::Prebindgen`] trait to
//! turn a flat `#[prebindgen]` library into a Rust file of JNI `extern "C"`
//! wrappers plus a fan-out of generated Kotlin sources.
//!
//! Pipeline:
//!   1. [`crate::api::core::registry::Registry::builder`] describes a binding over a model built from
//!      `(syn::Item, SourceLocation)` (typically `source.items_all()`).
//!   2. [`crate::api::core::registry::Registry::write_rust`] resolves every
//!      required type via a configured [`JniGenBuilder`] and writes the generated
//!      Rust bindings file.
//!   3. [`jni::JniGenBuilder::write_kotlin`] walks the resolved registry to emit the
//!      secondary Kotlin artifacts (typed-handle classes, data/enum classes,
//!      exception classes, the centralized `JNINative` holder).
//!
//! # Fixed-width unsigned integers
//!
//! JniGenBuilder exposes Rust's fixed-width unsigned scalars without narrowing their
//! domain at the Kotlin boundary:
//!
//! | Rust | Kotlin surface | JNI wire |
//! |------|----------------|----------|
//! | `u8` | `Int` | `jint` |
//! | `u16` | `Int` | `jint` |
//! | `u32` | `Long` | `jlong` |
//! | `u64` | `ULong` | `jlong` / `Long` bit pattern |
//!
//! Inputs for `u8`, `u16`, and `u32` are range-checked and report a
//! [`JniBindingError`] through the generated binding-error handler. `u64`
//! uses Kotlin's bit-preserving `ULong.toLong()` / `Long.toULong()` bridge.
//! These mappings compose through nullable/result outputs, generated data
//! classes, callbacks, const getters, and supported output collections.

pub mod jni;
pub(crate) mod util;

#[cfg(feature = "unstable-cbindgen")]
pub(crate) use jni::ConvertSpec;
pub use jni::{
    box_jboolean, box_jbyte, box_jchar, box_jdouble, box_jfloat, box_jint, box_jlong, box_jshort,
    decode_byte_array, decode_string, encode_byte_array, encode_string, matching, null_byte_array,
    null_string, CachedIfaceMethod, ClassDecl, ConstDecl, ConvertDecl, ConvertSourceDecl,
    DataClassDecl, Declarations, EnumClassDecl, ExpandDecl, ExpandParamDecl, ExpandReturnDecl,
    FieldsDecl, FunctionDecl, IgnoreDecl, JniBindingError, JniGen, JniGenBuilder, PackageDecl,
    PtrClassDecl, SealedClassDecl, VariantDecl,
};

// Kotlin emission types now live in the standalone generator module
// (`api::gen::kotlin`); re-exported here so the public `lang::` surface is
// unchanged (`KotlinFile` aliases the model's `KtFile`).
pub use crate::api::gen::kotlin::KtFile as KotlinFile;
pub use crate::api::gen::kotlin::WriteKotlinError;

#[cfg(test)]
mod spelling_census {
    //! A committed census of every place this adapter asks a **spelling** what
    //! a type's layers are, instead of asking the model.
    //!
    //! `is_option_type` is `path_tail_is(ty, "Option")`, and
    //! `option_inner_type`/`vec_inner_type` peel by last path segment. The model
    //! **erases** transparent wrappers — `Box<T>` *is* `T`, and so is
    //! `Cow<'_, T>` — so every one of these answers "no layer" for a type the
    //! model says is `Optional` or `Sequence`.
    //!
    //! That is how #273 happened: Kotlin nullability was decided this way, so a
    //! `Box<Option<String>>` **parameter** rendered non-null while the
    //! identical-meaning `Option<String>` rendered `String?` — and a non-null
    //! parameter for an optional value makes the absent case unexpressible.
    //! `Conversions::{is_optional, optional_inner, sequence_elem,
    //! is_optional_borrow}` ask the model, and every site with a registry in
    //! scope should use those.
    //!
    //! ## What this is for
    //!
    //! The counts go **down**. A file at zero has been migrated and must not
    //! regress; a file above zero is remaining work, and #229's L4 "layer
    //! questions" is where it is tracked. Either way a NEW call cannot appear
    //! without moving a number, which is what stops the next site from reaching
    //! for the spelling because it was the easiest thing in scope.
    //!
    //! It does not say the remaining calls are wrong *today* — some may be
    //! legitimate spelling questions, the way `decoded_vec_satisfies` and
    //! `is_unsized_spelling` are. It says each one is a decision someone made,
    //! and moving the number is what puts it in front of review.
    //!
    //! ## Why it walks tokens
    //!
    //! A text scan is the wrong instrument for anything with more than one
    //! spelling: `option_inner_type(..)`,
    //! `types_util::option_inner_type(..)` and a `use`-aliased call are the same
    //! call. #271's first guard matched text and missed a bare `Some(..)` that
    //! turned out to be a live bug, so this counts **call expressions by callee
    //! name**, whatever path qualifies them.

    use proc_macro2::{Delimiter, TokenTree};

    /// The helpers that read a spelling where the model has the answer.
    const SPELLING_HELPERS: &[&str] = &[
        "is_option_type",
        "is_option_ref",
        "option_inner_type",
        "vec_inner_type",
        "peel_ref_option_vec",
    ];

    /// `(file, call count)` — every `.rs` under `api/lang/jnigen`, checked
    /// against the directory tree so a new module cannot sit outside the census.
    const CENSUS: &[(&str, usize)] = &[
        // The L4 "layer questions" remainder — #229. Not migrated here because
        // it is a separate consumer and bundling it would make one review of
        // both impossible.
        ("jni/emit/flat_input.rs", 20),
        ("jni/emit/struct_out.rs", 2),
        ("jni/emit/vec_build.rs", 1),
        ("jni/emit/wrapper.rs", 2),
        ("jni/fold.rs", 1),
        ("jni/iface.rs", 2),
        ("jni/kotlin_emit.rs", 1),
        ("jni/trait_impl.rs", 4),
        // Down from 2: the nullability decisions now ask the model. The one
        // left probes for an enum through its layers.
        ("jni/fn_plan.rs", 1),
    ];

    /// Count `name(` call expressions, ignoring how the path is qualified.
    fn count(ts: proc_macro2::TokenStream, n: &mut usize) {
        let toks: Vec<TokenTree> = ts.into_iter().collect();
        for (i, t) in toks.iter().enumerate() {
            if let TokenTree::Ident(id) = t {
                let called = matches!(
                    toks.get(i + 1),
                    Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis
                );
                if called && SPELLING_HELPERS.iter().any(|h| id == h) {
                    *n += 1;
                }
            }
            if let TokenTree::Group(g) = t {
                count(g.stream(), n);
            }
        }
    }

    fn rs_files(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).expect("jnigen dir") {
            let p = e.expect("dir entry").path();
            if p.is_dir() {
                rs_files(&p, root, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(
                    p.strip_prefix(root)
                        .expect("under jnigen")
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }

    #[test]
    fn spelling_helper_calls_are_accounted_for() {
        let root =
            std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/api/lang/jnigen"));
        let mut files = Vec::new();
        rs_files(root, root, &mut files);
        files.sort();

        let mut drift: Vec<String> = Vec::new();
        for f in &files {
            // Tests may exercise the helpers directly.
            if f.contains("tests") {
                continue;
            }
            let src = std::fs::read_to_string(root.join(f)).expect("read source");
            let ts: proc_macro2::TokenStream =
                src.parse().unwrap_or_else(|e| panic!("tokenize {f}: {e}"));
            let mut found = 0usize;
            count(ts, &mut found);
            let expected = CENSUS
                .iter()
                .find(|(name, _)| name == f)
                .map(|(_, n)| *n)
                .unwrap_or(0);
            if found != expected {
                drift.push(format!("  {f}: {expected} -> {found}"));
            }
        }
        assert!(
            drift.is_empty(),
            "SPELLING-CENSUS DRIFT:\n{}\n\n\
             These helpers read a type's layers off its SPELLING, which the model \
             erases wrappers from — see this module's docs. A count going DOWN is \
             the goal: drop the row (or lower it) in the same commit. A count going \
             UP needs a reason in review: prefer `Conversions::{{is_optional, \
             optional_inner, sequence_elem, is_optional_borrow}}`, which ask the \
             model, wherever a registry is in scope.",
            drift.join("\n"),
        );
    }
}
