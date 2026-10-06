// Every public implementation item must be selected by the facade below.
#![deny(unreachable_pub)]

//! # prebindgen-tools
//!
//! Building blocks for a prebindgen **language adapter**. Applications that
//! only want bindings use the `prebindgen-c` or `prebindgen-jni` adapter.
//!
//! An adapter writes *generated elements* — functions, types, constants —
//! around the elements of the [`Flat`](prebindgen_flat::Flat) model, plus the
//! foreign code that uses them. Values cross between the two on **wires**:
//! the parameters, return values and fields of the generated elements. Wire
//! types form a closed set, one enum per adapter ([`WireType`]). The tools
//! here are what every adapter needs to get a source type onto wires and
//! back.
//!
//! ## The model
//!
//! A value crosses in one of two directions:
//!
//! * an [`Input`] turns wires into a source value — for example a parameter
//!   on its way into the source function;
//! * an [`Output`] turns a source value into wires — for example a result on
//!   its way back.
//!
//! Each holds the Rust expression that turns one into the other *and* a
//! [`Form`]: a tree recording how each layer of the type crossed.
//!
//! A source type crosses in one of the ways below; its form records which,
//! as a [`FormKind`].
//! For each way, the table names the function that builds the [`Input`] and
//! the one that builds the [`Output`]:
//!
//! | The value crosses… | [`Input`] | [`Output`] |
//! |---|---|---|
//! | as one wire | [`Input::wire`] | [`Output::wire`] |
//! | converted to another source type, which crosses instead | [`Input::via`] | [`Output::via`] |
//! | as a record's fields | [`Input::record`] | [`Output::record`] |
//! | as an option's presence flag and value | [`Input::optional`] | [`Output::optional`] |
//! | as a sum's tag and alternatives | [`Input::sum`] | [`Output::sum`] |
//! | as a sequence | [`Input::seq`] | [`Output::seq`] |
//! | as any other list of parts | [`Input::parts`] | [`Output::parts`] |
//!
//! Some take a description of the type: `record` takes the flat model's
//! struct and `sum` its enum, while `via` takes the conversion the build
//! script declared with `convert!`, resolved once into a
//! [`ResolvedConversion`].
//!
//! The foreign-side writer walks the form, so it follows what the adapter
//! decided for each layer without deciding again. What a single wire means
//! is in the wire's own type.
//!
//! ## Using an Input or Output
//!
//! An [`Input`] or [`Output`] holds everything the element needs for one
//! value, on both sides of the boundary:
//!
//! * **The element's signature.** [`Input::wires`] and [`Output::wires`]
//!   list the value's wires in order. [`Wire::decl`] writes one wire as
//!   `name: Type` for a parameter list; [`WireType::rust`] gives its Rust
//!   type alone, for a return type or a struct field.
//! * **The Rust code.** An input's `expr` evaluates to the source value:
//!   place it where the value is needed, such as an argument of the source
//!   call. An output's `expr` evaluates to the wire values, one value or a
//!   tuple in [`Output::wires`] order; [`Output::bind`] writes
//!   `let <wires> = expr;`, so each wire becomes a local of its own name.
//! * **Errors.** `fallible` says whether `expr` uses `?` on a
//!   `Result<_, String>`. Place such an expression in a function returning
//!   that type, or take [`Input::result`] / [`Output::result`], a `Result`
//!   value, and route the error yourself.
//! * **The foreign code.** `form` tells the foreign-side writer how to
//!   produce the same wires (for an input) or read them (for an output), in
//!   the same order. What a single wire means is in its [`WireType`]
//!   variant; records, options, sums and sequences are their [`FormKind`].
//!
//! For example, a C-style wrapper for `pub fn twice(n: u64) -> u64`, where
//! both values cross as an `i64` wire:
//!
//! ```
//! use prebindgen::SourceLocation;
//! use prebindgen_flat::Flat;
//! use prebindgen_tools::{Input, Output, Wire, WireType};
//! use quote::{format_ident, quote};
//!
//! #[derive(Clone, Debug)]
//! struct Long;
//! impl WireType for Long {
//!     fn rust(&self) -> proc_macro2::TokenStream { quote!(i64) }
//!     fn placeholder(&self) -> proc_macro2::TokenStream { quote!(0) }
//! }
//!
//! let source = syn::parse_file("pub fn twice(n: u64) -> u64 { n * 2 }").unwrap();
//! let flat = Flat::builder()
//!     .items(source.items.into_iter().map(|i| (i, SourceLocation::default())))
//!     .build().unwrap();
//! let f = flat.function("twice").unwrap();
//!
//! // Build: one input for the parameter, on a wire named like it, and one
//! // output for the result, on a wire named `ret`.
//! let n = &f.params[0];
//! let n_name = &n.name;
//! let input = Input::wire(&n.ty, Wire::new(n_name.clone(), Long), quote!(#n_name as u64));
//! let output = Output::wire(&f.ret, Wire::new(format_ident!("ret"), Long), quote!(__value as i64));
//!
//! // Use: the wires give the signature, the expressions the body.
//! let params = input.wires().iter().map(|w| w.decl()).collect::<Vec<_>>();
//! let ret_ty = output.wires()[0].ty.rust();
//! // The model names the source function by its qualified path.
//! let callee = &f.name;
//! let (arg, value) = (&input.expr, &output.expr);
//! let wrapper = quote! {
//!     pub extern "C" fn twice_wrapper(#(#params),*) -> #ret_ty {
//!         let __value = #callee(#arg);
//!         #value
//!     }
//! };
//! let wrapper: syn::ItemFn = syn::parse2(wrapper).unwrap();
//! assert_eq!(
//!     quote!(#wrapper).to_string(),
//!     quote! {
//!         pub extern "C" fn twice_wrapper(n: i64) -> i64 {
//!             let __value = twice(n as u64);
//!             __value as i64
//!         }
//!     }.to_string(),
//! );
//! ```
//!
//! ## Deciding how each layer crosses
//!
//! The example decided by itself that a `u64` crosses as one wire. A real
//! adapter decides per type, following the build script's declarations, and
//! a type such as `Option<Point>` needs one decision per layer: the
//! `Option`, then the `Point`, then each of its fields.
//!
//! So the adapter builds an input or output by recursion over the type.
//! [`shape()`] reads the outermost layer as a [`Shape`] — a scalar, an
//! `Option`, a borrow, a type the adapter declared, and so on. The adapter
//! calls the function the table above gives for that layer, and recurses
//! into what the layer holds.
//!
//! The adapter keeps its declarations in [`Overrides`]: a default for each
//! type, and the replacements the build script declared for single places.
//! A [`Place`] names where a part sits in the element, such as parameter
//! `p` of `send`, or field `x` of that parameter. The adapter starts from
//! the place of each parameter or result and extends it as it recurses
//! ([`Place::at`]). At each layer it asks [`Overrides::get`] with the
//! current place and type — passing that lookup to [`shape()`] does it —
//! and names the layer's wires after the place ([`Place::ident`]). The place
//! only steers the building; it is not stored in the [`Input`] or
//! [`Output`].
//!
//! ## Other tools
//!
//! Generated code names a source item by its
//! [`ItemName`](prebindgen_flat::flat::ItemName), which the flat model
//! qualifies when it is built: splicing a function's `name` writes
//! `source_crate::make`. A whole source type is never spelled by the adapter,
//! except through [`callback_arg_types`]. [`RustFile`] collects and writes the
//! generated Rust.

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
/// [`Conversion::resolve`] checks the declaration against the model; the
/// adapter does it once, while planning, and keeps the
/// [`ResolvedConversion`] as its declaration for the source type. When it
/// builds a value of that type, it passes the conversion to [`Input::via`]
/// or [`Output::via`], the same way [`Input::record`] takes the flat model's
/// struct. The adapter builds the representation's [`Input`] or [`Output`]
/// as for any other type, inside the closure it passes; the result's
/// [`Form`] is [`FormKind::Via`] over the representation's form.
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

/// The parameter types of a callback closure.
///
/// An adapter builds a Rust closure for an `impl Fn(..)` parameter, and Rust
/// needs its parameters annotated to give it the borrowed signature the callee
/// expects. [`callback_arg_types`] spells those types from the model:
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_flat::{flat::TypeKind, Flat};
/// use prebindgen_tools::callback_arg_types;
/// use quote::quote;
///
/// let source = syn::parse_file(
///     "pub struct Payload; pub fn on(f: impl Fn(&Payload) + Send + Sync + 'static) {}",
/// ).unwrap();
/// let location = SourceLocation { crate_name: Some("src".into()), ..Default::default() };
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|i| (i, location.clone())))
///     .build().unwrap();
/// let TypeKind::Callback { args } = flat.function("on").unwrap().params[0].ty.kind() else {
///     panic!("a callback parameter")
/// };
/// let types = callback_arg_types(&flat, args);
/// assert_eq!(types[0].to_string(), quote!(&src::Payload).to_string());
/// ```
pub mod callback {
    pub use crate::api::qualify::callback_arg_types;
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
/// An adapter builds a type's [`Input`] or [`Output`] by recursion: it looks
/// at the outermost layer,
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
/// let w = place.ident();
/// let input = Input::wire(&n.ty, Wire::new(w.clone(), W::Long { unsigned: true }),
///     quote!(#w as u64));
/// assert_eq!(input.wires()[0].decl().to_string(), "n : i64");
/// ```
pub mod wire {
    pub use crate::api::wire::{result_expr, Form, FormKind, Input, Output, Wire, WireType};
}

pub use crate::{
    api::check_supported,
    callback::callback_arg_types,
    convert::{Conversion, FnRef, ResolvedConversion, Via},
    file::RustFile,
    place::{Overrides, Place, Seg},
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
