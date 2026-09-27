// Every public implementation item must be selected by the facade below.
#![deny(unreachable_pub)]

//! # prebindgen-tools
//!
//! Building blocks for a prebindgen **language adapter**.
//!
//! An adapter reads the flat model of what `#[prebindgen]` captured
//! ([`flat`], re-exported from `prebindgen-flat`), turns its build script's
//! declarations into a plan of the elements to bind, and then writes each
//! element: the Rust wrapper through a generator here, and whatever the
//! destination language needs beside it (Kotlin text, for the JNI adapter)
//! itself. This crate holds no plan and no state across elements: every
//! generator writes one element from what it is given.
//!
//! ## Generators
//!
//! One generator per kind of Rust item. Each takes the flat element it
//! wraps and header customizations through a builder, and asks for the
//! boundary of each value through a small callback trait:
//!
//! | Generator | Writes | Callbacks |
//! |---|---|---|
//! | [`FunctionWriter`] | an exported wrapper around a [`Function`](flat::flat::Function) | [`FunctionCallbacks`]: the result's return type, out-wires and body; each parameter's wires and wire→value [`Input`]; the failure fragment |
//! | [`StructWriter`] | a mirror struct of a [`Struct`](flat::flat::Struct), with both conversions | [`FieldCallbacks`]: each field's wires, both directions |
//! | [`SumWriter`] | a mirror enum of a [`Variant`](flat::flat::Variant), with both conversions | [`FieldCallbacks`] |
//! | [`ClosureWriter`] | an `impl Fn(..)` closure over a foreign callback | [`ClosureCallbacks`]: each argument's wires ([`Output`]) |
//!
//! The generators expect **final** wire types: how a `Vec<Payload>` becomes
//! a pointer and a length, or a count and a column per field, is the
//! adapter's answer in its callback.
//!
//! ## Recursion
//!
//! A callback answers for a whole type by recursing over its structure.
//! [`shape()`] reads one layer — an `Option`, a borrow, a sequence, a named
//! type with the adapter's own setting for it — so every adapter's recursion
//! is one `match` over [`Shape`]. The answers compose: [`Input`] and
//! [`Output`] carry an expression plus the wires it reads or produces;
//! [`record_in`] / [`record_out`] take a struct or a sum alternative apart
//! into its fields; [`Input::optional`], [`Input::combine`] and
//! [`Output::concat`] build the rest; [`Stage::decode`] / [`Stage::encode`]
//! wrap a representation in a declared conversion. A conversion that can
//! fail uses `?` on `Result<_, String>`, and whoever places it decides where
//! the error goes.
//!
//! ## Everything else
//!
//! * [`Qualifier`] — names a flat item from generated code
//!   (`Payload` → `perftest_flat::Payload`).
//! * [`names`] — case conversion and identifier helpers.
//! * [`mod@convert`] — the `convert!` vocabulary: a source type that crosses as
//!   another type through functions or `From`/`TryFrom` impls.
//! * [`RustFile`] — the generated file: items in order, formatted, written
//!   only when changed.

// Implementation modules are private. The public modules below select every
// exported item explicitly, independently of the implementation layout.
mod api;

/// The flat source model read by language adapters.
pub use prebindgen_flat as flat;
// Exported macros must reach syn through this crate, including when an adapter
// re-exports them. Keep this dependency export here with the rest of the API.
#[doc(hidden)]
pub use syn as __syn;

/// The closure writer: a Rust `impl Fn(..)` built over a foreign callback.
///
/// A source function taking `impl Fn(A, B) + Send + Sync + 'static` needs a
/// Rust closure that, on every call, turns `A` and `B` into wires and hands
/// them to the foreign side. The shape is fixed; the adapter supplies the
/// pieces:
///
/// ```text
/// {
///     <setup>                                  // once, when the closure is built
///     move |__a0: A, __a1: B| {
///         let __res = (|| -> Result<(), String> {
///             <invoke>                         // binds each argument's wires, calls the foreign side
///             Ok(())
///         })();
///         if let Err(__err) = __res { <on_error> }
///     }
/// }
/// ```
///
/// `invoke` receives the arguments' bindings rather than having them placed
/// ahead of it, so an adapter can run them inside a scope of its own — a
/// JNI frame whose environment the conversions use.
pub mod closure {
    pub use crate::api::closure::{ClosureCallbacks, ClosureWriter};
}

/// Custom conversions: a source type that crosses as another type.
///
/// `convert!(Millis).input(fun!(millis_from_raw)).output(fun!(millis_to_raw))`
/// says that `Millis` never crosses as itself: it crosses as `u64` — the
/// **representation** — and these two functions convert. The adapter lowers
/// the representation like any other type and wraps the resulting wires in
/// the conversion's stages ([`Stage::apply`]).
///
/// A stage is a function (a flat item, or a binding-local path with a
/// stated signature) or a standard trait impl (`From`, `Into`, `TryFrom`,
/// `TryInto`). A function returning `Result` makes its stage fallible; the
/// error is reported through its `Display`.
pub mod convert {
    pub use crate::api::convert::{Conversion, FnRef, ResolvedConversion, Stage, Via};
}

/// The generated Rust file.
pub mod file {
    pub use crate::api::file::{resolve_out_path, write_if_changed, RustFile};
}

/// The function wrapper writer.
///
/// A wrapper is one exported function that receives wires, converts them to
/// the source function's arguments, calls it, and hands the result back as
/// wires:
///
/// ```text
/// <attrs>
/// pub unsafe extern "C" fn <name><generics>(<leading>, <param wires>, <return wires>, <trailing>) -> <return type> {
///     <prologue>
///     let <param> = <param input>;          // on failure: <fail>
///     ...
///     let __result = <callee>(<params>);
///     <return body>
/// }
/// ```
///
/// The writer owns the shape; the adapter owns every decision, through
/// [`FunctionCallbacks`]: which wires a parameter becomes and how they turn
/// back into the argument, which wires the result leaves on, and what the
/// wrapper does when a conversion fails. The builder methods customize the
/// header and the fixed parts of the body.
pub mod function {
    pub use crate::api::function::{
        error_ident, result_ident, FunctionCallbacks, FunctionWriter, Return, ERROR, RESULT,
    };
}

/// Identifier and case helpers shared by every adapter.
pub mod names {
    pub use crate::api::names::{bare, camel, ident, join, mangle, pascal, snake};
}

/// Naming source items from generated code.
///
/// The flat model spells every type in the flat namespace — `Payload`,
/// `Option<Vec<Payload>>` — because that is what the source crates wrote. The
/// generated file lives in a different crate, so every item it names has to
/// be reached through the crate that declared it: `perftest_flat::Payload`.
/// [`Qualifier`] does that rewriting, for types and for item paths.
pub mod qualify {
    pub use crate::api::qualify::Qualifier;
}

/// Records — structs and the alternatives of a sum — taken apart into their
/// fields and put back together.
///
/// Two uses, one mechanism:
///
/// * **Decomposition** ([`record_in`], [`record_out`]): the record crosses as
///   the concatenation of its fields' wires, with no type of its own on the
///   boundary. Recursing into a field that is itself a record gives the
///   leaf-by-leaf crossing a JVM binding uses.
/// * **Mirrors** ([`StructWriter`], [`SumWriter`]): the record crosses as a
///   generated type whose fields are the wires — a `#[repr(C)]` struct or
///   enum for C — plus the two conversions between it and the source type.
///
/// Either way the adapter decides, per field, through [`FieldCallbacks`];
/// the helpers here only handle the Rust shape of the record: named or
/// positional fields, the delimiters the source wrote, the order.
pub mod record {
    pub use crate::api::record::{
        record_in, record_out, FieldCallbacks, Record, StructMirror, StructWriter, SumMirror,
        SumWriter,
    };
}

/// One level of a type's structure, read against the adapter's own settings.
///
/// An adapter lowers a type by recursion: it looks at the outermost layer,
/// decides what that layer becomes on its boundary, and recurses into what
/// the layer holds. [`shape()`] answers the first question the same way for
/// every adapter — which layer this is, and, for a named type, the setting
/// the adapter gave that type — so each adapter's recursion is one `match`
/// over [`Shape`].
pub mod shape {
    pub use crate::api::shape::{shape, Access, Holding, Shape};
}

/// Wires and the conversions between wires and source values.
///
/// A **wire** is one parameter, field or return slot of the generated
/// boundary, typed in the generated Rust file: `jlong`, `*const c_char`,
/// `payload_t`. A source value crosses as zero or more wires, and which
/// ones is the adapter's decision; these types only carry the decision and
/// compose it.
///
/// * [`Input`] turns wires into a source value — a parameter on its way into
///   the source function, a field of a struct arriving from the foreign side.
/// * [`Output`] turns a source value into wires — a return value, a callback
///   argument, a field of a struct leaving.
///
/// Both carry an expression. A **fallible** one may use `?` on a
/// `Result<_, String>`; whoever places it decides where the error goes (see
/// [`Input::result`]). Generated conversion helpers follow the same
/// convention, so a converter written for one position composes into another.
pub mod wire {
    pub use crate::api::wire::{result_expr, Input, Output, Wire};
}

pub use crate::{
    api::check_supported,
    closure::{ClosureCallbacks, ClosureWriter},
    convert::{Conversion, FnRef, ResolvedConversion, Stage, Via},
    file::RustFile,
    function::{FunctionCallbacks, FunctionWriter, Return},
    qualify::Qualifier,
    record::{
        record_in, record_out, FieldCallbacks, Record, StructMirror, StructWriter, SumMirror,
        SumWriter,
    },
    shape::{shape, Access, Holding, Shape},
    wire::{Input, Output, Wire},
};

/// `fun!(name)` / `fun!(crate::local)` — a function reference.
#[macro_export]
macro_rules! fun {
    ($($path:tt)+) => {
        $crate::convert::FnRef::new($crate::__syn::parse_quote!($($path)+))
    };
}

/// `convert!(Type)` — start a conversion declaration.
#[macro_export]
macro_rules! convert {
    ($($ty:tt)+) => {
        $crate::convert::Conversion::new($crate::__syn::parse_quote!($($ty)+))
    };
}

/// `from!(Repr)` — the input stage is `From<Repr>` on the source type.
#[macro_export]
macro_rules! from {
    ($($ty:tt)+) => { $crate::convert::Via::From($crate::__syn::parse_quote!($($ty)+)) };
}

/// `into!(Repr)` — the output stage is `Into<Repr>` on the source type.
#[macro_export]
macro_rules! into {
    ($($ty:tt)+) => { $crate::convert::Via::Into($crate::__syn::parse_quote!($($ty)+)) };
}

/// `try_from!(Repr)` — the input stage is `TryFrom<Repr>` on the source type.
#[macro_export]
macro_rules! try_from {
    ($($ty:tt)+) => { $crate::convert::Via::TryFrom($crate::__syn::parse_quote!($($ty)+)) };
}

/// `try_into!(Repr)` — the output stage is `TryInto<Repr>` on the source type.
#[macro_export]
macro_rules! try_into {
    ($($ty:tt)+) => { $crate::convert::Via::TryInto($crate::__syn::parse_quote!($($ty)+)) };
}

/// `sig!((a: A, b: B) -> R)` — a signature, for a binding-local function.
#[macro_export]
macro_rules! sig {
    (($($args:tt)*) -> $($ret:tt)+) => {
        $crate::__syn::parse_quote!(fn __sig($($args)*) -> $($ret)+)
    };
    (($($args:tt)*)) => {
        $crate::__syn::parse_quote!(fn __sig($($args)*))
    };
}

/// `ty!(T)` — a `syn::Type`.
#[macro_export]
macro_rules! ty {
    ($($t:tt)+) => { { let __t: $crate::__syn::Type = $crate::__syn::parse_quote!($($t)+); __t } };
}

/// `path!(a::b)` — a `syn::Path`.
#[macro_export]
macro_rules! path {
    ($($t:tt)+) => { { let __p: $crate::__syn::Path = $crate::__syn::parse_quote!($($t)+); __p } };
}

/// `expr!(e)` — a `syn::Expr`.
#[macro_export]
macro_rules! expr {
    ($($t:tt)+) => { { let __e: $crate::__syn::Expr = $crate::__syn::parse_quote!($($t)+); __e } };
}

/// `ident!(name)` — a `syn::Ident`.
#[macro_export]
macro_rules! ident {
    ($i:ident) => {{
        let __i: $crate::__syn::Ident = $crate::__syn::parse_quote!($i);
        __i
    }};
}

#[cfg(test)]
mod tests;
