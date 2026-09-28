// Every public implementation item must be selected by the facade below.
#![deny(unreachable_pub)]

//! # prebindgen-tools
//!
//! Building blocks for a prebindgen **language adapter**. Applications that
//! only want bindings normally use the `prebindgen-c` or `prebindgen-jni`
//! adapter crate instead.
//!
//! An adapter reads the captured [`Flat`](prebindgen_flat::Flat) model,
//! chooses representations, and generates its own Rust wrappers and foreign
//! declarations. This crate supplies type inspection and conversion composition;
//! the adapter owns signatures, layouts, calls, and error handling.
//!
//! * [`shape()`] classifies one layer of a type, including adapter declarations.
//!   The adapter decides which children to visit recursively.
//! * [`Qualifier`] names source types and items in generated Rust.
//! * [`Input`] and [`Output`] carry wires and conversion expressions.
//!   [`Input::combine`], [`Input::optional`] and [`Output::concat`] compose them.
//! * [`Record`], [`record_in`] and [`record_out`] construct and destructure
//!   source structs and enum alternatives while preserving field order.
//! * [`mod@convert`] resolves declared conversions; [`Stage::decode`] and
//!   [`Stage::encode`] compose them with their representation's conversion.
//! * [`RustFile`] collects, formats and writes the adapter's generated items.
//!
//! Fallible conversions use `Result<_, String>`. The adapter places the
//! expressions in an error scope and chooses how to report failures. Source
//! function errors remain part of the source return value and are handled
//! separately by the adapter.

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
/// **representation** — and these two functions convert. The adapter lowers
/// the representation like any other type and wraps the resulting wires in
/// the conversion's stages ([`Stage::apply`]).
///
/// A stage is a function (a flat item, or a binding-local path with a
/// stated signature) or a standard trait impl (`From`, `Into`, `TryFrom`,
/// `TryInto`). A function returning `Result` makes its stage fallible; the
/// error is reported through its `Display`.
///
/// For example, given source functions `millis_from_raw(u64) -> Millis` and
/// `millis_to_raw(&Millis) -> Result<u64, String>`, an adapter can resolve
/// both directions before lowering a use of `Millis`:
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_flat::Flat;
/// use prebindgen_tools::{convert, fun, Input, Output, Qualifier, Wire, ident};
/// use quote::quote;
/// let source = syn::parse_file(r#"
///     pub struct Millis(pub u64);
///     pub fn millis_from_raw(raw: u64) -> Millis { Millis(raw) }
///     pub fn millis_to_raw(value: &Millis) -> Result<u64, String> { Ok(value.0) }
/// "#).unwrap();
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|item| (item, SourceLocation::default())))
///     .build().unwrap();
/// let conversion = convert!(Millis)
///     .input(fun!(millis_from_raw))
///     .output(fun!(millis_to_raw))
///     .resolve(&flat).unwrap();
/// let input = conversion.input.as_ref().unwrap();
/// let output = conversion.output.as_ref().unwrap();
/// // Each stage's `repr` is u64; `output.fallible` is true.
/// assert_eq!(input.repr.spell().to_string(), "u64");
/// assert!(output.fallible);
/// let q = Qualifier::new(&flat);
/// let decoded = input.decode(&q, &conversion.target,
///     Input::identity(Wire::new(ident!(raw), quote!(u64))),
///     &ident!(r), quote!());
/// assert_eq!(decoded.wires.len(), 1);
/// let encoded = output.encode(&q, &conversion.target, &quote!(value),
///     &ident!(r), quote!(),
///     Output::single(Wire::new(ident!(raw), quote!(u64)), quote!(r)));
/// assert!(encoded.fallible);
/// ```
///
/// An [`Input`] for `u64` is wrapped with [`Stage::decode`]
/// before the source call; an [`Output`] for `u64` is wrapped
/// with [`Stage::encode`] after it. A binding-local function can be named
/// with [`fun!`](crate::fun!) plus [`FnRef::sig`]. See [`Via`] for trait-based alternatives.
pub mod convert {
    pub use crate::api::convert::{Conversion, FnRef, ResolvedConversion, Stage, Via};
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
/// * [`Qualifier::path`] qualifies one item name, such as the function a
///   the adapter will call.
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
/// fields and put back together.
///
/// The adapter chooses a representation independently of the source shape.
/// These helpers construct and destructure source values, preserving named or
/// positional fields, delimiters, and order. Each field may occupy several wires.
///
/// ## Example: decompose a record
///
/// [`record_in`] combines field inputs in source order and reconstructs the
/// source's named or tuple form. An adapter normally produces the field
/// inputs while recursing through [`shape()`](crate::shape()).
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_flat::{Flat, flat::Type};
/// use prebindgen_tools::{Input, Record, Wire,
///     ident, record_in};
/// use quote::quote;
///
/// let item = syn::parse_quote!(pub struct Point { pub x: i32, pub y: i32 });
/// let flat = Flat::builder()
///     .items([(item, SourceLocation::default())])
///     .build().unwrap();
/// let Some(Type::Struct(point)) = flat.declared_type("Point") else { panic!() };
/// let fields = vec![
///     Input::identity(Wire::new(ident!(x_wire), quote!(i32))),
///     Input::identity(Wire::new(ident!(y_wire), quote!(i32))),
/// ];
/// let input = record_in(Record::Struct(point), &quote!(Point), fields);
/// assert_eq!(input.wires.len(), 2);
/// assert_eq!(input.expr.to_string(), "Point { x : x_wire , y : y_wire }");
///
/// ```
pub mod record {
    pub use crate::api::record::{record_in, record_out, Record};
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
///
/// ## Example: one value on two wires
///
/// A `u64` can arrive as two `u32` words. [`Input`] keeps the expression together with
/// the two [`Wire`] declarations so the adapter
/// can place and evaluate them in the right order:
///
/// ```
/// use prebindgen_tools::{Input, Output, Wire, ident};
/// use quote::quote;
///
/// let hi = Wire::new(ident!(hi), quote!(u32));
/// let lo = Wire::new(ident!(lo), quote!(u32));
/// let input = Input::new(vec![hi, lo], quote!(((hi as u64) << 32) | lo as u64));
/// assert_eq!(input.wires.len(), 2);
/// assert!(input.result().to_string().contains("Ok"));
///
/// let output = Output::new(
///     vec![Wire::new(ident!(hi), quote!(u32)),
///          Wire::new(ident!(lo), quote!(u32))],
///     quote!(((value >> 32) as u32, value as u32)),
/// );
/// assert_eq!(output.pattern().to_string(), "(hi , lo)");
/// ```
///
/// For a fallible step, use [`Input::and_then`] or [`Output::fallible`]
/// and place [`Input::result`] or [`Output::result`] at the point whose
/// error policy applies. The conversion error type is `String`; the adapter
/// supplies the fallback or error delivery code.
pub mod wire {
    pub use crate::api::wire::{result_expr, Input, Output, Wire};
}

pub use crate::{
    api::check_supported,
    convert::{Conversion, FnRef, ResolvedConversion, Stage, Via},
    file::RustFile,
    qualify::Qualifier,
    record::{record_in, record_out, Record},
    shape::{shape, Access, SequenceKind, Shape, TextKind},
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
