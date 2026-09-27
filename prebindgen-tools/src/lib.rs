// Every public implementation item must be selected by the facade below.
#![deny(unreachable_pub)]

//! # prebindgen-tools
//!
//! Building blocks for a prebindgen **language adapter**. Applications that
//! only want bindings normally use the `prebindgen-c` or `prebindgen-jni`
//! adapter crate instead.
//!
//! An adapter reads the flat model of what `#[prebindgen]` captured
//! ([`flat`], re-exported from `prebindgen-flat`), turns its build script's
//! declarations into a plan of the elements to bind, and then writes each
//! element: the Rust wrapper through a generator here, and whatever the
//! destination language needs beside it (Kotlin text, for the JNI adapter)
//! itself. The generators hold no shared plan: each writes one element from
//! what it is given. [`RustFile`] only collects the resulting Rust items.
//! The adapter decides which source items to bind, their names, wire types,
//! and error policy.
//!
//! ## Follow one element
//!
//! 1. Read the captured model as [`flat::Flat`]. Reject unsupported source
//!    items with [`check_supported`]. Use [`Qualifier`] when generated Rust
//!    refers back to a source item.
//! 2. For each declared item, choose a generator below. Its callback trait
//!    asks the adapter how values cross the boundary. [`shape()`] and
//!    [`Shape`] help a callback walk a type one layer at a time.
//! 3. Describe incoming values with [`Input`] and outgoing values with
//!    [`Output`]. Each contains its [`Wire`] slots and conversion expression.
//!    [`Stage`] can wrap either direction in a declared conversion.
//! 4. Push the generated Rust items into [`RustFile`], then
//!    [`RustFile::write`] them. The adapter writes any destination-language
//!    files itself.
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
//! adapter's answer in its callback. Start at a generator's module page for
//! the shape it writes and a worked example: [`function`], [`record`],
//! [`closure`].
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
//!
//! ## Small complete example
//!
//! This toy adapter keeps scalar parameters unchanged. Real C and JNI
//! adapters provide different [`FunctionCallbacks`] and handle more
//! [`Shape`] variants. The example uses an in-memory model so the path from
//! a flat function to generated Rust is visible without a build script.
//!
//! ```
//! use prebindgen::SourceLocation;
//! use prebindgen_tools::{flat::Flat, FunctionCallbacks, FunctionWriter,
//!     Input, Qualifier, Return, RustFile, Wire, check_supported};
//! use prebindgen_tools::flat::flat::TypeRef;
//! use proc_macro2::TokenStream;
//! use quote::quote;
//!
//! let source = syn::parse_file("pub fn double(n: i64) -> i64 { n * 2 }").unwrap();
//! let location = SourceLocation {
//!     crate_name: Some("source_crate".into()),
//!     ..Default::default()
//! };
//! let flat = Flat::builder()
//!     .items(source.items.into_iter().map(|item| (item, location.clone())))
//!     .build().unwrap();
//! check_supported(&flat).unwrap();
//!
//! struct Scalar;
//! impl FunctionCallbacks for Scalar {
//!     type Error = String;
//!     fn param(&mut self, name: &syn::Ident, ty: &TypeRef) -> Result<Input, String> {
//!         Ok(Input::identity(Wire::new(name.clone(), ty.spell())))
//!     }
//!     fn ret(&mut self, ret: &TypeRef) -> Result<Return, String> {
//!         Ok(Return { ty: Some(ret.spell()), wires: vec![], body: quote!(__result) })
//!     }
//!     fn fail(&mut self, _: &Return) -> TokenStream {
//!         quote!(panic!("{__err}"))
//!     }
//! }
//!
//! let function = flat.function("double").unwrap();
//! let qualifier = Qualifier::new(&flat);
//! let wrapper = FunctionWriter::new(function, qualifier.path(&function.name))
//!     .abi(None).unsafety(false)
//!     .write(&mut Scalar).unwrap();
//! let mut file = RustFile::new();
//! file.push(wrapper);
//! let generated = file.render();
//! assert!(generated.contains("fn double(n: i64) -> i64"));
//! assert!(generated.contains("source_crate::double(n)"));
//! ```

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
///
/// For instance, a callback taking `i64` can use
/// [`Output::single`](crate::Output::single) to pass one integer wire. The
/// adapter's `invoke` closure then emits the foreign call using that wire;
/// the writer supplies the Rust `move` closure, per-call error scope, and
/// [`ClosureWriter::on_error`] path.
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_tools::{flat::{Flat, flat::{TypeKind, TypeRef}},
///     ClosureCallbacks, ClosureWriter, Output, Qualifier, Wire};
/// use proc_macro2::TokenStream;
/// use quote::{format_ident, quote};
///
/// let source = syn::parse_file(
///     "pub fn register(callback: impl Fn(i64) + Send + Sync + 'static) {}"
/// ).unwrap();
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|item| (item, SourceLocation::default())))
///     .build().unwrap();
/// let ty = &flat.function("register").unwrap().params[0].ty;
/// let TypeKind::Callback { args } = ty.kind() else { panic!("expected callback") };
///
/// struct Scalar;
/// impl ClosureCallbacks for Scalar {
///     type Error = String;
///     fn arg(&mut self, index: usize, _: &TypeRef, value: &TokenStream)
///         -> Result<Output, String> {
///         Ok(Output::single(Wire::new(format_ident!("w{index}"), quote!(i64)), value))
///     }
/// }
/// let closure = ClosureWriter::new(args)
///     .on_error(quote!(panic!("{__err}")))
///     .write(&Qualifier::new(&flat), &mut Scalar, |bindings, outputs| {
///         let wire = &outputs[0].wires[0].name;
///         quote!(#bindings foreign_callback(#wire);)
///     }).unwrap();
/// assert!(closure.to_string().contains("foreign_callback (w0)"));
/// ```
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
///
/// For example, given source functions `millis_from_raw(u64) -> Millis` and
/// `millis_to_raw(&Millis) -> Result<u64, String>`, an adapter can resolve
/// both directions before lowering a use of `Millis`:
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_tools::{convert, fun, flat::Flat, Input, Output,
///     Qualifier, Wire, ident};
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
///
/// The [crate example](crate#small-complete-example) builds one wrapper from
/// an in-memory [`flat::Flat`]. In an adapter, the
/// callback's [`FunctionCallbacks::param`] can call
/// [`shape()`](crate::shape()), convert its result into an [`Input`], and let
/// [`FunctionWriter::write`] place the wires and conversion at the call site.
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
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_tools::{flat::Flat, Qualifier};
/// let source = syn::parse_file("pub struct Payload; pub fn make() -> Payload { Payload }").unwrap();
/// let location = SourceLocation {
///     crate_name: Some("source_crate".into()), ..Default::default()
/// };
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|item| (item, location.clone())))
///     .build().unwrap();
/// let q = Qualifier::new(&flat);
/// let function = flat.function("make").unwrap();
/// assert_eq!(q.path(&function.name).to_string(), "source_crate :: make");
/// assert_eq!(q.ty(&function.ret).to_string(), "source_crate :: Payload");
/// ```
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
///
/// ## Example: decompose a record
///
/// [`record_in`] combines field inputs in source order and reconstructs the
/// source's named or tuple form. A callback would normally produce the field
/// inputs while recursing through [`shape()`](crate::shape()).
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_tools::{flat::Flat, flat::flat::{Field, Type},
///     FieldCallbacks, Input, Output, Record, StructWriter, Wire, ident, record_in};
/// use proc_macro2::TokenStream;
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
/// // A mirror uses the same field decisions to define a boundary struct
/// // and the two conversions. Each conversion expression reads a local `v`.
/// struct Scalar;
/// impl FieldCallbacks for Scalar {
///     type Error = String;
///     fn field_in(&mut self, field: &Field) -> Result<Input, String> {
///         Ok(Input::identity(Wire::new(field.name.clone().unwrap(), quote!(i32))))
///     }
///     fn field_out(&mut self, field: &Field, value: &TokenStream)
///         -> Result<Output, String> {
///         Ok(Output::single(
///             Wire::new(field.name.clone().unwrap(), quote!(i32)), value
///         ))
///     }
/// }
/// let mirror = StructWriter::new(point, quote!(Point), ident!(PointWire))
///     .attr(quote!(#[repr(C)]))
///     .write(&mut Scalar).unwrap();
/// assert_eq!(mirror.wires.len(), 2);
/// assert!(mirror.def.to_string().contains("struct PointWire"));
/// assert!(mirror.input.expr.to_string().contains("let PointWire"));
/// ```
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
///
/// For example, if the adapter has declared a binding for `Payload`, then
/// `shape(&payload_type, |name| settings.get(name))` returns
/// [`Shape::Declared`] with that setting. For `Option<Payload>`, it returns
/// [`Shape::Option`] with the inner type; the adapter calls [`shape()`] again
/// for that inner type. This keeps the adapter's policy in one recursive
/// match rather than in the flat model.
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_tools::{flat::Flat, shape, Access, Shape};
/// let source = syn::parse_file("pub struct Payload; pub fn send(value: Option<&Payload>) {}").unwrap();
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|item| (item, SourceLocation::default())))
///     .build().unwrap();
/// let ty = &flat.function("send").unwrap().params[0].ty;
/// let Shape::Option(inner) = shape(ty, |name| (name == "Payload").then_some("handle"))
///     .unwrap() else { panic!("expected Option") };
/// assert!(matches!(
///     shape(inner, |name| (name == "Payload").then_some("handle")).unwrap(),
///     Shape::Declared { setting: "handle", access: Access::Shared, .. }
/// ));
/// ```
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
///
/// ## Example: one value on two wires
///
/// A `u64` can arrive as two `u32` words. [`Input`] keeps the expression together with
/// the two [`Wire`] declarations so [`FunctionWriter`]
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
/// error policy applies. The writer only understands `Result<_, String>`;
/// the adapter supplies the actual fallback in
/// [`FunctionCallbacks::fail`].
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
