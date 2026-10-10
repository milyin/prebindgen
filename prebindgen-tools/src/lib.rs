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
//! the parameters, return values and fields of the generated elements, typed
//! by a closed set the adapter owns ([`WireType`]).
//!
//! ## Decisions, then code
//!
//! How a value crosses is decided before any code is written, and the
//! decision is a tree the model already has the shape of: the type.
//!
//! 1. **Ways** ([`Ways`]): every way a type *may* cross, registered once.
//!    A [`Way`] holds the model item that fixes what lies below it — a
//!    struct crosses [`Way::Fields`] with one part per field, a type built by
//!    a function [`Way::Constructed`] with one part per parameter — so a way
//!    cannot be registered with parts its item does not have.
//! 2. **Choices** ([`Choices`]): which way each *occurrence* takes, in one
//!    direction — a default per type, replaced at chosen [`Place`]s. A
//!    [`WayId`] is typed by the directions its way can serve, so a
//!    constructor cannot be chosen for a value leaving Rust.
//! 3. **Crossing** ([`resolve`]): one occurrence's type walked layer by
//!    layer, the chosen way installed at each node, down to the wires — a
//!    [`Crossing`]. Its [`Node`]s are built only by resolving, so their
//!    children are the type's: a [`Fields`] node has a crossing per field of
//!    its struct, an [`Optional`] one of the option's inner type. A
//!    crossing is typed by its [`Direction`], so a node only one direction
//!    has cannot occur in the other. What the model leaves open — which
//!    values cross whole, on which wire, how an option tells absence, which
//!    wires carry a sequence — the adapter answers through [`Lower`].
//! 4. **Code**: [`Crossing::decode`] turns an [`In`] crossing into the Rust
//!    that rebuilds the value from its wires ([`Input`]), and
//!    [`Crossing::encode`] an [`Out`] crossing into the Rust that produces
//!    them ([`Output`]). The adapter writes only what one wire holds — with
//!    the wire itself, as a [`Whole`] — and a sequence's loop
//!    ([`Decode`], [`Encode`]); every composition is the crossing's.
//!
//! The same crossing serves both sides of the boundary: its
//! [`wires`](Crossing::wires) are the generated element's parameters, and
//! the foreign-side writer reads its nodes to produce or unpack the same
//! wires in the same order.
//!
//! ## Example
//!
//! A struct `Point` registered three ways — as its fields, whole as a
//! handle, and built by `point_new` — crossing each way at a different
//! parameter:
//!
//! ```
//! use prebindgen::SourceLocation;
//! use prebindgen_flat::{flat::{ScalarKind, Type, TypeRef}, Flat};
//! use prebindgen_tools::{
//!     resolve, Choices, Code, Crossing, Decode, In, Input, Lower, Place, Presence, Seg,
//!     Sequence, Shape, Ways, Whole, Wire, WireType,
//! };
//! use proc_macro2::TokenStream;
//! use quote::quote;
//!
//! // The adapter's wire types.
//! #[derive(Clone, Debug)]
//! enum W { Long, Handle }
//! impl WireType for W {
//!     fn rust(&self) -> TokenStream {
//!         match self { W::Long => quote!(i64), W::Handle => quote!(*mut ::core::ffi::c_void) }
//!     }
//!     fn placeholder(&self) -> TokenStream {
//!         match self { W::Long => quote!(0), W::Handle => quote!(::core::ptr::null_mut()) }
//!     }
//! }
//!
//! let source = syn::parse_file("
//!     pub struct Point { pub x: i64, pub y: i64 }
//!     pub fn point_new(x: i64) -> Point { Point { x, y: 0 } }
//!     pub fn draw(p: Point) {}
//!     pub fn keep(p: Point) {}
//!     pub fn build(p: Point) {}
//! ").unwrap();
//! let location = SourceLocation { crate_name: Some("src".into()), ..Default::default() };
//! let flat = Flat::builder()
//!     .items(source.items.into_iter().map(|i| (i, location.clone())))
//!     .build().unwrap();
//! let Some(Type::Struct(point)) = flat.declared_type("Point") else { unreachable!() };
//!
//! // 1. Ways: every way `Point` may cross, each checked against the model.
//! let mut ways = Ways::new();
//! let fields = ways.fields(point);
//! let handle = ways.whole(point.type_ref(), ());
//! let built = ways.constructed(flat.function("point_new").unwrap()).unwrap();
//!
//! // 2. Choices: the fields everywhere, but a handle for `keep`'s `p` and
//! //    the constructor for `build`'s. (`built` is a `WayId<In>`: it cannot
//! //    be chosen for a `Choices<Out>`.)
//! let param = |f: &str| Place::new(f).at(Seg::Param("p".into()));
//! let mut choices = Choices::<In>::new();
//! choices.choose(&ways, fields).unwrap();
//! choices.choose_at(param("keep"), handle);
//! choices.choose_at(param("build"), built);
//!
//! // 3. What the model leaves to the boundary: which values cross whole,
//! //    on which wire, read by which code.
//! struct Boundary<'a, 'f> { ways: &'a Ways<'f, ()>, choices: &'a Choices<In> }
//! impl<'f> Lower<'f, In> for Boundary<'_, 'f> {
//!     type Wire = W;
//!     type Leaf = ();
//!     type Error = String;
//!     fn ways(&self) -> &Ways<'f, ()> { self.ways }
//!     fn choices(&self) -> &Choices<In> { self.choices }
//!     fn whole(&self, _: &TypeRef, shape: Shape<'_, &()>, place: &Place)
//!         -> Result<Option<Whole<W, In>>, String>
//!     {
//!         let wire = |t| Wire::new(place.ident(), t);
//!         Ok(match shape {
//!             Shape::Scalar(ScalarKind::I64) => {
//!                 Some(Whole::<W, In>::new(wire(W::Long), |w| Code::new(quote!(#w))))
//!             }
//!             // `Point`, where its chosen way is whole.
//!             Shape::Declared { .. } => Some(Whole::<W, In>::new(wire(W::Handle), |w| {
//!                 Code::new(quote!(*::std::boxed::Box::from_raw(#w as *mut src::Point)))
//!             })),
//!             _ => None,
//!         })
//!     }
//!     fn presence(&self, _: &Crossing<'f, W, In>, place: &Place)
//!         -> Result<Presence<W, In>, String> { Err(format!("{place}: no options")) }
//!     fn sequence(&self, _: &Crossing<'f, W, In>, place: &Place)
//!         -> Result<Vec<Wire<W>>, String> { Err(format!("{place}: no sequences")) }
//!     fn tag(&self, place: &Place) -> Result<Wire<W>, String> {
//!         Err(format!("{place}: no sums"))
//!     }
//! }
//!
//! // 4. The code the crossing leaves to the adapter: here, none.
//! struct Codec;
//! impl<'f> Decode<'f, W> for Codec {
//!     type Error = String;
//!     fn sequence(&self, at: &Crossing<'f, W, In>, _: &Sequence<'f, W, In>, _: Input)
//!         -> Result<Code, String> { Err(format!("{}: no sequences", at.place())) }
//! }
//!
//! // Resolve each function's `p`, then read its wires and its Rust.
//! let boundary = Boundary { ways: &ways, choices: &choices };
//! let cross = |f: &str| {
//!     let p = &flat.function(f).unwrap().params[0];
//!     let crossing = resolve(&boundary, &p.ty, param(f)).unwrap();
//!     let wires: Vec<String> = crossing.wires().iter().map(|w| w.name.to_string()).collect();
//!     (wires, crossing.decode(&Codec).unwrap().expr().to_string())
//! };
//! let (wires, rust) = cross("draw");
//! assert_eq!(wires, ["p_x", "p_y"]);
//! assert_eq!(rust, quote!(src::Point { x: p_x, y: p_y }).to_string());
//! let (wires, _) = cross("keep");
//! assert_eq!(wires, ["p"]);
//! let (wires, rust) = cross("build");
//! assert_eq!(wires, ["p_x"]);
//! assert_eq!(rust, quote!(src::point_new(p_x)).to_string());
//! ```
//!
//! `tests/toy_adapter.rs` is a complete adapter in this style, with both
//! directions, options, sums, conversions and the foreign side.
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
/// adapter does it once, while planning, and registers the
/// [`ResolvedConversion`] as a way for the source type
/// ([`Ways::converted`]). A crossing of that type is then a
/// [`Converted`] node over the crossing of the representation.
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
/// [`Crossing::decode`] and [`Crossing::encode`] use it for [`Fields`] and
/// [`Arm`]s; an adapter writing its own mirror does too.
pub mod record {
    pub use crate::api::record::Record;
}

/// Places in an element, and the decisions made at them.
pub mod place {
    pub use crate::api::place::{Place, Seg};
}

/// One level of a type's structure, with any adapter declaration for that type.
///
/// [`resolve`] reads each layer of a type with it, and hands the layer to the
/// adapter's [`Lower::whole`] to ask whether the value crosses whole there.
/// [`shape()`] answers the same way for every adapter. It first asks the adapter whether the complete type has a
/// declaration, then returns a built-in shape when it does not. Each adapter
/// recurses with one `match` over [`Shape`].
///
/// For example, `shape(&payload_type, shape_declared)` returns
/// [`Shape::Declared`] when the callback returns a declaration for `Payload`.
/// For `Option<Payload>`, it returns [`Shape::Option`] when the callback has
/// no declaration for the whole option; the adapter then calls [`shape()`]
/// on its inner type.
///
/// A shape describes source structure; the adapter uses it to choose its
/// wire for a value that crosses whole. An adapter can declare a type such as `Vec<u8>` as a whole, while an undeclared
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

/// The hand-composed builders the adapters used before [`Crossing`]: kept
/// until both adapters resolve crossings, then removed.
#[doc(hidden)]
pub mod legacy {
    pub use crate::api::legacy::{Form, FormKind, Input, Output};
}

/// Wires: the slots of the generated boundary, typed by the adapter.
pub mod wire {
    pub use crate::api::wire::{Wire, WireType};
}

/// The ways each type may cross, and which one each occurrence takes.
pub mod ways {
    pub use crate::api::ways::{Both, Choices, Way, WayId, Ways};
}

/// How one occurrence of a type crosses: the decision tree, resolved from
/// the model and the choices.
pub mod crossing {
    pub use crate::api::crossing::{
        resolve, resolve_arm, Alternatives, Arm, Constructed, Converted, Crossing, Direction,
        Fields, In, Lower, Never, Node, Optional, Out, Presence, Sequence, Whole, Wrapped, Wrapper,
    };
}

/// The Rust a crossing becomes, with the adapter's code for its wires.
pub mod code {
    pub use crate::api::code::{result_expr, ArmOutput, Code, Decode, Encode, Input, Output};
}

pub use crate::{
    api::check_supported,
    callback::callback_arg_types,
    code::{result_expr, ArmOutput, Code, Decode, Encode, Input, Output},
    convert::{Conversion, FnRef, ResolvedConversion, Via},
    crossing::{
        resolve, resolve_arm, Alternatives, Arm, Constructed, Converted, Crossing, Direction,
        Fields, In, Lower, Never, Node, Optional, Out, Presence, Sequence, Whole, Wrapped, Wrapper,
    },
    file::RustFile,
    place::{Place, Seg},
    record::Record,
    shape::{shape, Access, SequenceKind, Shape, TextKind},
    ways::{Both, Choices, Way, WayId, Ways},
    wire::{Wire, WireType},
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
