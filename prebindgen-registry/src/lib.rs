//! # prebindgen-registry
//!
//! Registry-based, **language-agnostic** converter pipeline for
//! [`prebindgen`](https://docs.rs/prebindgen). This crate turns a stream of
//! `#[prebindgen]` items — read through [`prebindgen::Source`] and parsed by
//! [`flat::Flat`] — into generated Rust FFI bindings plus a fully resolved
//! table of type converters. It has no knowledge of any particular
//! destination language — C, JNI/Kotlin, Swift, Python, etc. all plug in the
//! same way.
//!
//! It also re-exports the flat model (`flat`, `shape`, `types_util`) from
//! [`prebindgen`], so a language adapter names one crate root for the whole
//! pipeline rather than reaching back into `prebindgen::core` for half of it.
//!
//! # The plug-in point
//!
//! Write one generator per destination language. It does two things:
//!
//! * **Says how the language represents Rust types on the wire** — it builds a
//!   [`ConverterImpl`] (a generated converter fn plus its wire type) for each
//!   crossing the registry hands it, and gives them all back through
//!   `RegistryBuilder::convert_with`.
//! * **Emits the wrapper code per item** — `on_function` / `on_struct` /
//!   `on_enum` / `on_const` on the [`Prebindgen`] trait.
//!
//! Everything language-specific that must travel through the pipeline rides in
//! the back-end's chosen [`Metadata`](Prebindgen::Metadata) type (a JNI
//! back-end's Kotlin class names and exception info, a C back-end's header
//! names, …). It is set on each converter, propagated into the registry's
//! [`TypeEntry`], and read back by the back-end's own emitter — no side
//! channels. Back-ends needing no extras leave it at the default `()`.
//!
//! # Flow
//!
//! A build script sees one type — the generator — and never names a `Flat` or a
//! `Registry`:
//!
//! ```ignore
//! let jni = JniGen::builder()
//!     .package(package!("io.zenoh"))
//!     .fun(fun!(session_open))
//!     .source(zenoh_flat::PREBINDGEN_OUT_DIR)
//!     .build()?;
//! jni.write_rust(&rust_dest)?;
//! jni.write_kotlin(&kotlin_root)?;
//! ```
//!
//! Inside `build()`, the generator does what it alone knows how to do:
//!
//! 1. [`flat::Flat::builder`] parses the declared sources into the model, and
//!    [`Registry::builder`] starts describing a binding over it.
//! 2. The generator states that binding, then [`Registry::crossings`] hands
//!    over every crossing needing a conversion — inner types first, so each one
//!    can be built from those already done. `convert_with` answers them and
//!    `build` names any gap.
//! 3. The resolved registry becomes a field of the built generator, whose
//!    `write_*` methods emit the artifacts — Rust wrappers, and whatever else
//!    that language needs (a C header, Kotlin sources, …).
//!
//! # Universality, by example
//!
//! The same machinery serves very different languages:
//!
//! * **C / cbindgen back-end** (the separate `prebindgen-c` crate): wire types
//!   are raw pointers and primitive C types; converters are thin transmutes;
//!   `pre_stages` are usually empty (errors surface as return codes).
//! * **JNI / Kotlin back-end** (the separate `prebindgen-jni` crate): wire
//!   types are JNI handles (`jlong`, `JObject`); converters marshal across the
//!   JVM boundary; `pre_stages` carry fallible steps whose `Err` arms throw
//!   JVM exceptions (the exception info lives in that back-end's `Metadata`).
//!
//! # Macros
//!
//! The declaration surface is built almost entirely from exported macros. This
//! crate defines the language-neutral ones — the domain vocabulary shared by
//! every adapter, plus the syntax helpers they're built from:
//!
//! - Members & constants: [`fun!`](crate::fun)
//! - Conversions: [`convert!`](crate::convert), [`from!`](crate::from),
//!   [`try_from!`](crate::try_from), [`into!`](crate::into),
//!   [`try_into!`](crate::try_into)
//! - Boundary expansion: [`expand_param!`](crate::expand_param),
//!   [`expand_return!`](crate::expand_return), [`fields!`](crate::fields)
//!
//! **Syntax helpers** produce a bare `syn` node — `Type` / `Path` / `Expr` /
//! `Signature` / `Ident` — to hand to a declaration method that requires one.
//! They exist only to sidestep `syn::parse_quote!`'s type-inference ambiguity
//! (E0283) in a generic argument position, not to express a domain concept:
//! [`ty!`](crate::ty), [`path!`](crate::path), [`expr!`](crate::expr),
//! [`sig!`](crate::sig), [`ident!`](crate::ident).
//!
//! The JNI/Kotlin-specific declaration macros — `package!`, `ptr_class!`,
//! `data_class!`, `enum_class!`, `sealed_class!`, `variant!`, `constant!` —
//! construct a typed `*Decl` for the Kotlin surface and live in the separate
//! `prebindgen-jni` crate, which hands the result to its `JniGenBuilder`.

pub mod decl;
mod destination;
pub mod diagnostics;
pub mod domain;
pub mod expand;
pub mod niches;
pub mod prebindgen;
pub mod registry;
pub(crate) mod resolve;
#[cfg(test)]
pub(crate) mod test_util;
pub mod unfold;
pub mod write;

/// The flat model itself lives in `prebindgen::core` — re-exported here so
/// an adapter names one crate root for the whole pipeline.
pub use ::prebindgen::core::{flat, shape, types_util};
pub use ::prebindgen::core::{Element, Emit, Flat};

pub use self::{
    decl::{
        ConvertDecl, ConvertSourceDecl, ConvertSpec, ExpandDecl, ExpandParamDecl, ExpandReturnDecl,
        FieldsDecl, FunctionDecl, LocalField, LocalVariant,
    },
    diagnostics::{warn_unclaimed, Claimed},
    domain::{DomainScalar, RepresentationDomain, ScalarValue},
    niches::{NicheSlot, Niches},
    prebindgen::{ConverterImpl, NamePredicate, Prebindgen, Stage},
    registry::{
        Building, Conversions, Crossing, Decompositions, Direction, DuplicateNameError,
        NotExpressibleEntry, Registry, RegistryBuilder, ScanError, TypeEntry, TypeKey,
        TypeKeyParseError, WriteRustError,
    },
};

/// Not part of the public API — referenced by the [`ident!`] macro expansion
/// so callers don't need their own `proc-macro2` dependency just to build a
/// `Span`, by this crate's own decl macros (`fun!`, `convert!`, …), and by the
/// `prebindgen-jni` crate's JNI/Kotlin decl macros (`ptr_class!`, `package!`,
/// …) to parse a bare type token into a concrete `syn::Type`. `pub` (rather
/// than `pub(crate)`) for exactly that cross-crate macro-expansion reason,
/// despite `#[doc(hidden)]`.
#[doc(hidden)]
pub mod __macro_support {
    pub use proc_macro2;

    pub fn parse_type(s: &str) -> ::syn::Type {
        ::syn::parse_str(s).unwrap_or_else(|e| panic!("prebindgen: invalid type `{s}`: {e}"))
    }

    pub fn parse_path(s: &str) -> ::syn::Path {
        ::syn::parse_str(s).unwrap_or_else(|e| panic!("prebindgen: invalid path `{s}`: {e}"))
    }

    pub fn parse_expr(s: &str) -> ::syn::Expr {
        ::syn::parse_str(s).unwrap_or_else(|e| panic!("prebindgen: invalid expression `{s}`: {e}"))
    }

    /// Parse a `sig!((params) -> Ret)` body: `s` is the token text between
    /// the macro's outer parens plus the optional `-> Ret` tail, e.g.
    /// `"(s: & Summary, verbose: bool) -> String"`. Wrapped into a full fn
    /// item signature under a placeholder name (replaced by the declaring
    /// decl's fn ident at synthesis time).
    pub fn parse_signature(s: &str) -> ::syn::Signature {
        let full = format!("fn __sig {s}");
        ::syn::parse_str::<::syn::ItemFn>(&format!("{full} {{ unimplemented!() }}"))
            .map(|f| f.sig)
            .unwrap_or_else(|e| panic!("prebindgen: invalid signature `sig!({s})`: {e}"))
    }
}

/// Build a `syn::Ident` from a bare identifier token. Unlike
/// `syn::parse_quote!`, this always yields the concrete type `syn::Ident` —
/// there's no external context needed to infer it — so it can be passed
/// directly into a generic `impl Into<T>` parameter without hitting rustc's
/// "type annotations needed" ambiguity. `syn::parse_quote!`'s output type
/// has to be pinned by a *concrete* parameter type to infer successfully; a
/// generic `impl Into<T>` bound doesn't give it anything to unify against.
///
/// This is what powers the [`fun!`](crate::fun) decl macro — see that macro
/// (and the `prebindgen-jni` crate's `ptr_class!`/`enum_class!`/`data_class!`,
/// which apply the same trick to `syn::Type`) for the primary way this
/// crate's builders are fed bare Rust names today.
///
/// ```
/// let _: syn::Ident = prebindgen_registry::ident!(z_thing_name);
/// ```
#[macro_export]
macro_rules! ident {
    ($name:ident) => {
        ::syn::Ident::new(
            stringify!($name),
            $crate::__macro_support::proc_macro2::Span::call_site(),
        )
    };
}
