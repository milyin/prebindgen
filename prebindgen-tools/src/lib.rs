// Every public implementation item must be selected by the facade below.
#![deny(unreachable_pub)]

//! # prebindgen-tools
//!
//! Building blocks for a prebindgen **language adapter**. Applications that
//! only want bindings use the `prebindgen-c` or `prebindgen-jni` adapter.
//!
//! An adapter writes *generated elements* — functions, types, constants —
//! around the elements of the [`Flat`](prebindgen_flat::Flat) model, plus the
//! foreign code that uses them. Every generated element speaks only in the
//! adapter's **wire types**: a closed set, one enum per adapter
//! ([`WireType`]). The tools here are what every adapter needs to get a
//! source type onto wires and back.
//!
//! ## The model
//!
//! A source type at a [`Place`] — parameter `p` of `send`, field `x` of it —
//! crosses in one of a few ways ([`FormKind`]):
//!
//! * **directly**, as one wire ([`Input::wire`]);
//! * **converted** to another source type that crosses instead
//!   ([`ResolvedConversion::decode`], [`ResolvedConversion::encode`]);
//! * **taken apart** into parts that cross on their own: a record's fields
//!   ([`Input::record`]), an optional's presence and value
//!   ([`Input::optional`]), a sum's tag and alternatives ([`Input::sum`]),
//!   a sequence ([`Input::seq`]).
//!
//! Lowering a value builds an [`Input`] (wires → value) or an [`Output`]
//! (value → wires): the Rust expression *and* the [`Form`] tree recording
//! which way each layer crossed. The foreign-side writer walks that tree,
//! so it follows the decisions made — defaults and overrides alike — without
//! making them again; what a single wire means is in the wire's own type.
//!
//! ## Writing an element
//!
//! 1. For each part of the element (parameter, result, field), take its
//!    [`Place`].
//! 2. Look up the decision with [`Overrides::get`]: the place's override if
//!    the build script gave one, else the type's default. [`shape()`] does it
//!    layer by layer when given that lookup.
//! 3. Lower the part by recursion over [`Shape`], one combinator per layer,
//!    extending the place as it descends ([`Place::at`]).
//! 4. Assemble the element from the parts' wires and expressions.
//!
//! [`Qualifier`] names source items from the generated crate; [`RustFile`]
//! collects and writes the generated Rust.
//!
//! Fallible conversions use `Result<_, String>`: an expression may use `?`,
//! and the adapter decides where the error goes ([`Input::result`]).

// Implementation modules are private. The public modules below select every
// exported item explicitly, independently of the implementation layout.
mod api;

// Exported macros must reach syn through this crate, including when an adapter
// re-exports them. Keep this dependency export here with the rest of the API.
#[doc(hidden)]
pub use syn as __syn;

/// Custom conversions: a source type that crosses as another type.
///
/// `convert!(Millis).input(fun!(millis_from_raw)).output(fun!(millis_to_raw))`
/// says that `Millis` never crosses as itself: it crosses as `u64` — the
/// **representation** — and these two functions convert.
///
/// Each direction is a function (a flat item, or a binding-local path with a
/// stated signature) or a standard trait impl (`From`, `Into`, `TryFrom`,
/// `TryInto`). A function returning `Result` makes its direction fallible;
/// the error is reported through its `Display`.
///
/// [`Conversion::resolve`] checks the declaration against the model. The
/// adapter lowers the representation like any other type, inside the closure
/// it passes to [`ResolvedConversion::decode`] or
/// [`ResolvedConversion::encode`]; the result's [`Form`] is
/// [`FormKind::Via`] over the representation's form.
pub mod convert {
    pub use crate::api::convert::{Conversion, FnRef, ResolvedConversion, Via};
}

/// The generated Rust file.
///
/// [`RustFile`] preserves item order, adds source feature guards with
/// [`RustFile::guards`], formats parseable Rust with [`RustFile::render`],
/// and writes through [`RustFile::write`]. The write operation compares the
/// current bytes before replacing a file so a build script does not trigger
/// another Cargo build merely by regenerating identical output.
///
/// ```
/// use prebindgen_tools::RustFile;
/// use quote::quote;
///
/// let mut file = RustFile::new();
/// file.push(quote!(pub fn answer() -> u32 { 42 }));
/// assert!(file.render().contains("pub fn answer() -> u32"));
/// // In build.rs: file.write("generated.rs")?;
/// ```
pub mod file {
    pub use crate::api::file::{resolve_out_path, write_if_changed, RustFile};
}

/// Identifier and case helpers shared by every adapter.
pub mod names {
    pub use crate::api::names::{bare, camel, ident, join, mangle, pascal, snake};
}

/// Naming source items from generated Rust code.
///
/// The flat model identifies a source item by its flat name, such as
/// `Payload` or `make`. A binding's generated Rust usually lives in a
/// different crate, where those names are not automatically in scope.
/// [`Qualifier`] uses each item's source location to spell a path such as
/// `source_crate::Payload` or `source_crate::make`. Use it when a wrapper
/// calls a source function or a conversion names a source type.
///
/// ## Choosing a path
///
/// [`Qualifier::new`] borrows the model. For each known item,
/// [`Qualifier::module_of`] reads the crate name recorded in its
/// [`SourceLocation`](prebindgen::SourceLocation), converting hyphens to
/// underscores for a Rust path. Items captured from different crates can
/// therefore receive different prefixes from the same qualifier.
///
/// [`Qualifier::with_default_module`] supplies a fallback for **known items
/// without a recorded crate name**, for example a model assembled directly
/// from parsed syntax in a test. A recorded crate name takes precedence.
/// A name absent from the model is left unchanged even when a fallback is
/// configured; a known item with neither a crate name nor a fallback also
/// keeps its bare name. The generated crate must be able to resolve the
/// emitted paths: qualification does not add dependencies or imports, or
/// discover Cargo dependency aliases.
///
/// ## Item paths and type expressions
///
/// * [`Qualifier::path`] qualifies one item name, such as the function the
///   adapter will call.
/// * [`Qualifier::ty`] walks a complete model type. For example,
///   `Option<Vec<Payload>>` becomes
///   `::core::option::Option<::std::vec::Vec<source_crate::Payload>>`.
///   It qualifies named types inside generic arguments and containers, and
///   named constants used as array lengths. Standard containers receive
///   absolute `::core` or `::std` paths; scalar names such as `u32` stay as is.
/// * [`Qualifier::ty_elided`] provides the same naming for positions such as
///   closure arguments that cannot refer to a source lifetime parameter.
///   Its method documentation lists which shapes it elides.
///
/// These methods return Rust tokens for the source value's type or path.
/// The adapter still chooses the wire representation: qualifying
/// `Payload` does not turn it into a pointer, handle, or mirror struct.
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_flat::Flat;
/// use prebindgen_tools::Qualifier;
/// use quote::quote;
///
/// let source = syn::parse_file(
///     "pub struct Payload; pub fn make() -> Option<Vec<Payload>> { None }",
/// ).unwrap();
/// let location = SourceLocation {
///     crate_name: Some("source-crate".into()), ..Default::default()
/// };
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|item| (item, location.clone())))
///     .build().unwrap();
/// let q = Qualifier::new(&flat);
/// let function = flat.function("make").unwrap();
/// assert_eq!(q.path(&function.name).to_string(), quote!(source_crate::make).to_string());
/// let expected: syn::Type = syn::parse_quote!(
///     ::core::option::Option<::std::vec::Vec<source_crate::Payload>>
/// );
/// assert_eq!(syn::parse2::<syn::Type>(q.ty(&function.ret)).unwrap(), expected);
///
/// // The fallback does not override the recorded source crate.
/// let q = q.with_default_module(Some(syn::parse_quote!(crate::source)));
/// assert_eq!(q.path(&function.name).to_string(), quote!(source_crate::make).to_string());
/// // A binding-local helper absent from the model stays unqualified.
/// assert_eq!(q.path(&syn::parse_quote!(local_helper)).to_string(), "local_helper");
/// ```
pub mod qualify {
    pub use crate::api::qualify::Qualifier;
}

/// Records — structs and the alternatives of a sum — taken apart into their
/// fields and put back together, with the delimiters the source wrote.
/// [`Input::record`] and [`Output::record`] use it; an adapter writing its own
/// mirror does too.
pub mod record {
    pub use crate::api::record::Record;
}

/// Places in an element, and the decisions made at them.
pub mod place {
    pub use crate::api::place::{Overrides, Place, Seg};
}

/// One level of a type's structure, with any adapter declaration for that type.
///
/// An adapter lowers a type by recursion: it looks at the outermost layer,
/// decides what that layer becomes on its boundary, and recurses into what
/// the layer holds. [`shape()`] answers the first question the same way for
/// every adapter. It first asks the adapter whether the complete type has a
/// declaration, then returns a built-in shape when it does not. Each adapter
/// recurses with one `match` over [`Shape`].
///
/// For example, `shape(&payload_type, shape_declared)` returns
/// [`Shape::Declared`] when the callback returns a declaration for `Payload`.
/// For `Option<Payload>`, it returns [`Shape::Option`] when the callback has
/// no declaration for the whole option; the adapter then calls [`shape()`]
/// on its inner type.
///
/// A shape describes source structure; the adapter uses it to build an
/// [`Input`] or [`Output`] for its chosen wire representation. An adapter
/// can declare a type such as `Vec<u8>` as a whole, while an undeclared
/// container exposes child types for further calls.
/// [`Access`] describes by-value use or borrowing; [`SequenceKind`] and
/// [`TextKind`] identify the source container independently of access.
/// `Cow` remains an explicit wrapper around its inner type.
///
/// [`shape()`] documents declaration precedence, the handling of common
/// borrows, and which unsupported forms return errors.
/// A successful call only classifies the current layer: an adapter must
/// still handle undeclared names and errors encountered in its children.
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_flat::{Flat, flat::{TypeKind, TypeRef}};
/// use prebindgen_tools::{shape, Access, Shape};
/// let source = syn::parse_file("pub struct Payload; pub fn send(value: Option<&Payload>) {}").unwrap();
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|item| (item, SourceLocation::default())))
///     .build().unwrap();
/// let ty = &flat.function("send").unwrap().params[0].ty;
/// // The callback has no declaration for the whole Option.
/// let Shape::Option(inner) = shape::<&str>(ty, |_| None)
///     .unwrap() else { panic!("expected Option") };
/// // The callback finds Payload inside the borrow; Declared retains &Payload.
/// assert!(matches!(
///     shape(inner, |candidate: &TypeRef| match candidate.kind() {
///         TypeKind::Named { id, .. } if id.name == "Payload" => Some("handle"),
///         _ => None,
///     }).unwrap(),
///     Shape::Declared { ty, declaration: "handle" }
///         if std::ptr::eq(ty, inner) && Access::of(ty) == Access::Shared
/// ));
/// // Missing adapter configuration is a shape the adapter must handle.
/// assert!(matches!(
///     shape::<()>(inner, |_| None).unwrap(),
///     Shape::Undeclared("Payload")
/// ));
/// ```
pub mod shape {
    pub use crate::api::shape::{shape, Access, SequenceKind, Shape, TextKind};
}

/// Wires, the forms values take on them, and the conversions between.
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_flat::Flat;
/// use prebindgen_tools::{Input, Place, Seg, Wire, WireType};
/// use quote::quote;
///
/// // The adapter's closed set of wire types.
/// #[derive(Clone, Debug)]
/// enum W { Long { unsigned: bool } }
/// impl WireType for W {
///     fn rust(&self) -> proc_macro2::TokenStream { quote!(i64) }
///     fn placeholder(&self) -> proc_macro2::TokenStream { quote!(0) }
/// }
///
/// let source = syn::parse_file("pub fn send(n: u64) {}").unwrap();
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|i| (i, SourceLocation::default())))
///     .build().unwrap();
/// let n = &flat.function("send").unwrap().params[0];
/// let place = Place::new("send").at(Seg::Param("n".into()));
/// let w = place.ident("");
/// let input = Input::wire(&n.ty, Wire::new(w.clone(), W::Long { unsigned: true }),
///     quote!(#w as u64));
/// assert_eq!(input.wires()[0].decl().to_string(), "n : i64");
/// ```
pub mod wire {
    pub use crate::api::wire::{result_expr, Form, FormKind, Input, Output, Wire, WireType};
}

pub use crate::{
    api::check_supported,
    convert::{Conversion, FnRef, ResolvedConversion, Via},
    file::RustFile,
    place::{Overrides, Place, Seg},
    qualify::Qualifier,
    record::Record,
    shape::{shape, Access, SequenceKind, Shape, TextKind},
    wire::{Form, FormKind, Input, Output, Wire, WireType},
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
