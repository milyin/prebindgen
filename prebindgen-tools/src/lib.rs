//! Conversion composition. Adapters supply leaf codecs; wrappers consume prepared slots.
//!
//! Fill [`Slot`] recipes using [`Declarations`] or manual configuration, then call
//! [`prepare`] with a [`Codec`]. Wrappers consume [`Prepared`] slots and follow the
//! validation/acquisition and delivery/commit ordering documented on those types.
//! Source types remain in the flat model; recipes and adapter metadata are separate.
//! [`WireKind`] describes a common role while [`Wire::extra`] holds optional
//! adapter-defined information. [`boxed`] is an opt-in storage policy.
#![warn(missing_docs)]
use prebindgen_flat::{same_type, type_key, Flat, Primitive, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;

/// Inputs are recipes, not wire layouts. A selector denotes a logical choice.
#[derive(Clone, Debug)]
pub enum Input {
    /// Decode the occurrence's Rust primitive using the adapter's scalar codec.
    Scalar,
    /// Delegate this source type and occurrence data to [`Codec::input_foreign`].
    Foreign,
    /// Decode `String` or shared string input using the adapter's text codec.
    Text,
    /// Decode `Vec<u8>` or shared byte input using the adapter's byte codec.
    Bytes,
    /// Construct a named record from children in field order; an empty list builds unit.
    Fields(Vec<Input>),
    /// Call a source function after preparing its arguments with the child recipes.
    Constructor {
        /// Unqualified source function name; its result must match the occurrence type.
        function: String,
        /// Recipes in the constructor's parameter order.
        args: Vec<Input>,
    },
    /// Choose an owned input using an `i32` arm index followed by all arms' wires.
    /// Inactive arms are neither validated nor acquired; indices are zero-based.
    Selector(Vec<Input>),
}
impl Input {
    /// Construct a record from field recipes, or unit from an empty list.
    ///
    /// Preparation verifies the child count against the source record.
    pub fn fields(fields: impl IntoIterator<Item = Input>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    /// Call a named source constructor with the supplied argument recipes.
    ///
    /// Preparation checks its result type and argument count. Argument validation and
    /// ownership acquisition precede the source call.
    pub fn constructor(name: impl Into<String>, args: impl IntoIterator<Item = Input>) -> Self {
        Self::Constructor {
            function: name.into(),
            args: args.into_iter().collect(),
        }
    }
    /// Choose between owned representations of the same input type.
    ///
    /// Requires at least one arm. Each arm retains its own wires; this does not
    /// merge equivalent wire parameters or generate destination-language overloads.
    pub fn selector(arms: impl IntoIterator<Item = Input>) -> Self {
        Self::Selector(arms.into_iter().collect())
    }
}
/// A result conversion recipe, interpreted together with its source type and slot data.
///
/// Field projections are observations rather than moves. Adapters decide how to
/// copy or otherwise expose a foreign value observed through a shared reference.
#[derive(Clone, Debug)]
pub enum Output {
    /// Encode the occurrence's Rust primitive using the adapter's scalar codec.
    Scalar,
    /// Delegate to [`Codec::output_foreign`]; the adapter owns representation and ownership.
    Foreign,
    /// Move an owned `String`, or clone it when observing a borrowed value.
    Text,
    /// Move an owned `Vec<u8>`, or clone it when observing a borrowed value.
    Bytes,
    /// Observe named fields in declaration order; an empty list represents unit.
    Fields(Vec<Output>),
    /// A shared-borrow accessor, followed by the output conversion of its result.
    Getter {
        /// Source getter taking exactly one shared reference to the current type.
        function: String,
        /// Conversion of the getter's owned result; borrowed results are unsupported.
        output: Box<Output>,
    },
    /// Produce several views of one value. At most one child may consume it.
    /// Observations run before consumption, while wire order follows this list.
    Combine(Vec<Output>),
}
impl Output {
    /// Observe record fields in declaration order; use an empty list for unit.
    pub fn fields(fields: impl IntoIterator<Item = Output>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    /// Observe a value through a source getter and convert the getter's result.
    ///
    /// The getter must take exactly `&T` for the current type `T`, and return an
    /// owned value. Preparation rejects mismatched signatures.
    pub fn getter(name: impl Into<String>, output: Output) -> Self {
        Self::Getter {
            function: name.into(),
            output: Box::new(output),
        }
    }
    /// Produce multiple outputs from one value.
    ///
    /// Preparation rejects an empty list or more than one consuming branch. The
    /// framework uses [`Codec::consumes`] to order observations before a move.
    pub fn combine(parts: impl IntoIterator<Item = Output>) -> Self {
        Self::Combine(parts.into_iter().collect())
    }
    fn consumes<C: Codec>(&self, ty: &Type, extra: &C::Extra, codec: &C) -> bool {
        match self {
            Self::Foreign => codec.consumes(ty, extra),
            Self::Combine(p) => p.iter().any(|p| p.consumes(ty, extra, codec)),
            _ => false,
        }
    }
}

/// The direction and structure of a conversion assigned to a flat occurrence.
#[derive(Clone, Debug, Default)]
pub enum Recipe {
    /// No conversion assigned. An entirely empty function is not exported.
    #[default]
    Empty,
    /// Conversion from foreign inputs to a source argument.
    Input(Input),
    /// Conversion from a source result to foreign outputs.
    Output(Output),
}

/// An occurrence's recipe and adapter data are independent. The same `extra`
/// is passed through every nested conversion in this occurrence's recipe.
#[derive(Clone, Debug, Default)]
pub struct Slot<E> {
    /// Conversion structure, shared by all adapters.
    pub recipe: Recipe,
    /// Adapter data for this occurrence, preserved when declarations replace the recipe.
    pub extra: E,
}

/// A wire's role remains available to both sides, even when carriers coincide.
#[derive(Clone, Debug)]
pub enum WireKind {
    /// A source primitive; the adapter chooses its ABI carrier.
    Scalar(Primitive),
    /// An adapter-defined value; this imposes no metadata or ownership requirement.
    Foreign,
    /// Text in the adapter's buffer or object representation.
    Text,
    /// Bytes in the adapter's buffer or array representation.
    Bytes,
    /// Zero-based input alternative index, carried as an `i32` by current adapters.
    Selector,
}
/// One prepared boundary parameter or result with adapter-owned metadata.
///
/// The framework does not inspect `W`. Metadata can be attached to every kind,
/// or omitted by choosing `()`, including for [`WireKind::Foreign`].
///
/// ```
/// use prebindgen_tools::{Condition, Wire, WireKind};
/// let wire = Wire {
///     name: syn::parse_quote!(value),
///     kind: WireKind::Foreign,
///     extra: (),
///     active: Condition::default(),
/// };
/// assert!(matches!(wire.kind, WireKind::Foreign));
/// ```
#[derive(Clone, Debug)]
pub struct Wire<W> {
    /// Generated identifier used by conversion code and wrapper signatures.
    pub name: syn::Ident,
    /// Common role used by native and destination-language emission.
    pub kind: WireKind,
    /// Adapter-owned data for any wire kind; `()` is valid, including for foreign wires.
    pub extra: W,
    /// Input selector path; outputs normally use the always-active default.
    pub active: Condition,
}

/// Language-specific terminal operations. Recursive composition stays in tools.
///
/// Foreign input leaves must validate without consuming in `validation`, then
/// acquire ownership without failure or source calls in `acquire`. Each acquired
/// value needs an ordinary Rust owner so later failures drop it. `statements`
/// can construct arguments from those owners. Every foreign wire must retain
/// the supplied activation condition. Output leaves keep ownership in Rust
/// locals until their infallible `commit` runs after successful delivery.
///
/// # Implementing a codec
///
/// This minimal codec uses Rust primitives, `String` and `Vec<u8>` as its wire
/// carriers. It illustrates preparation without a foreign ABI or object storage.
/// A language adapter additionally writes wrappers and chooses corresponding
/// native and destination-language carrier types. Unsupported foreign leaves
/// return generation errors.
///
/// ```
/// use prebindgen_flat::{Flat, Primitive, Result};
/// use prebindgen_tools::{
///     Codec, Condition, Declarations, Prepared, PreparedInput, PreparedOutput,
///     Slot, WireKind, fun, prepare,
/// };
/// use proc_macro2::TokenStream;
/// use quote::quote;
/// use syn::{Ident, Type};
///
/// struct RustCarriers;
/// impl Codec for RustCarriers {
///     type Extra = ();
///     type Wire = (); // No wire metadata, even though the trait allows it.
///
///     fn wire_extra(&self, _: &Type, _: &WireKind, _: &()) {}
///     fn consumes(&self, _: &Type, _: &()) -> bool { false }
///     fn input_foreign(
///         &self, _: &Type, _: &(), _: Ident, _: Ident, _: &Condition,
///     ) -> Result<PreparedInput<()>> {
///         Err("this codec supports only scalars, text and bytes".into())
///     }
///     fn output_foreign(
///         &self, _: &Type, _: &(), _: Ident, _: TokenStream, _: bool,
///     ) -> Result<PreparedOutput<()>> {
///         Err("this codec supports only scalars, text and bytes".into())
///     }
///     fn decode_scalar(&self, _: Primitive, wire: &Ident, _: &()) -> TokenStream {
///         quote!(#wire)
///     }
///     fn encode_scalar(&self, _: Primitive, value: TokenStream, _: &()) -> TokenStream {
///         value
///     }
///     fn decode_text(&self, wire: &Ident, _: &Type, _: &()) -> TokenStream {
///         quote!(#wire.clone())
///     }
///     fn decode_bytes(&self, wire: &Ident, _: &Type, _: &()) -> TokenStream {
///         quote!(#wire.clone())
///     }
/// }
///
/// let source = syn::parse_file("pub fn echo(value: i64) -> i64 { value }")?;
/// let mut flat = Flat::<Slot<()>>::parse(
///     source.items.into_iter().map(|item| (item, Default::default())),
/// )?;
/// Declarations::new(&flat, vec![])?.fill(&mut flat, &fun!(echo))?;
/// let prepared = prepare(flat, &RustCarriers)?;
/// let Prepared::Input(input) = &prepared.function("echo")?.params[0].ty.slot else {
///     panic!("expected prepared input");
/// };
/// assert!(matches!(input.wires[0].kind, WireKind::Scalar(Primitive::I64)));
/// assert!(input.resources.is_empty()); // No foreign ownership in this codec.
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait Codec {
    /// Occurrence data stored beside a recipe in [`Slot`]; `()` means no extra data.
    /// This same value is passed through all nested conversions of that occurrence.
    type Extra: Clone + std::fmt::Debug;
    /// Metadata stored in [`Wire::extra`]. Tools neither inspect it nor require
    /// `Default`; [`Self::wire_extra`] supplies it for common conversions.
    type Wire: Clone + std::fmt::Debug;
    /// Data for wires emitted by common conversions. Foreign converters supply
    /// their own wires and data. Tools impose no metadata requirements by kind.
    fn wire_extra(&self, ty: &Type, kind: &WireKind, extra: &Self::Extra) -> Self::Wire;
    /// Whether a foreign output consumes the current source value.
    ///
    /// This must agree with [`Self::output_foreign`]. It governs move ordering and
    /// double-consumption rejection in [`Output::Combine`], not input ownership.
    fn consumes(&self, ty: &Type, extra: &Self::Extra) -> bool;
    /// Prepare a foreign input of source type `ty` using occurrence data `extra`.
    ///
    /// `wire` and `var` are fresh identifiers for a boundary input and source local.
    /// Preserve `active` on every returned wire and resource. All fallible checks
    /// belong in [`PreparedInput::validation`]; do not consume values there. Return
    /// a generation error when this type or access mode is unsupported.
    fn input_foreign(
        &self,
        ty: &Type,
        extra: &Self::Extra,
        wire: syn::Ident,
        var: syn::Ident,
        active: &Condition,
    ) -> Result<PreparedInput<Self::Wire>>;
    /// Prepare delivery of the Rust expression `value` as one or more foreign wires.
    ///
    /// When `borrowed` is true, the expression is a shared reference to `ty`; do not
    /// move out of it. Keep unpublished allocations under Rust ownership and transfer
    /// them only in [`PreparedOutput::commit`]. Return a generation error for
    /// unsupported conversions. The source value is owned when `borrowed` is false.
    fn output_foreign(
        &self,
        ty: &Type,
        extra: &Self::Extra,
        wire: syn::Ident,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<PreparedOutput<Self::Wire>>;
    /// Emit a Rust expression decoding one carrier to the requested primitive.
    ///
    /// It runs during active-input validation, before object acquisition, and may
    /// use `?` to return a `String` error. It must not consume foreign objects.
    fn decode_scalar(
        &self,
        primitive: Primitive,
        wire: &syn::Ident,
        extra: &Self::Extra,
    ) -> TokenStream;
    /// Emit a Rust expression encoding an owned primitive value for output.
    ///
    /// The adapter's wrapper writer must use the matching carrier for the resulting
    /// [`WireKind::Scalar`] wire.
    fn encode_scalar(
        &self,
        primitive: Primitive,
        value: TokenStream,
        extra: &Self::Extra,
    ) -> TokenStream;
    /// Emit text decoding for `String` or a shared string input.
    ///
    /// The result must own or safely borrow text for the entire source call. Tools
    /// pass it by shared reference when `ty` is a reference. The expression executes
    /// only for an active input, may return a `String` error with `?`, and must not
    /// consume foreign objects.
    fn decode_text(&self, wire: &syn::Ident, ty: &Type, extra: &Self::Extra) -> TokenStream;
    /// Emit byte decoding for `Vec<u8>` or a shared byte input.
    ///
    /// The result must remain valid for the entire source call. Tools pass it by
    /// shared reference when `ty` is a reference. Like [`Self::decode_text`], this
    /// runs during validation and must not acquire foreign ownership.
    fn decode_bytes(&self, wire: &syn::Ident, ty: &Type, extra: &Self::Extra) -> TokenStream;
}
/// An input conversion split into call-wide validation, acquisition and construction.
///
/// Wrappers must validate **all** active inputs and [`Resource`] conflicts before
/// executing any acquisition, then acquire all consuming inputs before evaluating
/// argument construction. This preserves caller ownership on validation failure.
#[derive(Clone, Debug)]
pub struct PreparedInput<W> {
    /// Flattened boundary parameters in signature order.
    pub wires: Vec<Wire<W>>,
    /// Validate and prepare every active leaf before any ownership transfer.
    pub validation: TokenStream,
    /// Infallible acquisition of all owned inputs, before constructors can run.
    pub acquire: TokenStream,
    /// Active foreign resources whose identities must be checked across all arguments.
    pub resources: Vec<Resource>,
    /// Argument construction after all acquisitions, including source constructor calls.
    pub statements: TokenStream,
    /// Rust expression passed to the source function after `statements`.
    pub argument: TokenStream,
}
/// Output preparation whose owned temporaries survive until successful delivery.
///
/// Wrappers evaluate `statements`, deliver `values` in `wires` order, then execute
/// `commit` only after every fallible delivery operation succeeds.
#[derive(Clone, Debug)]
pub struct PreparedOutput<W> {
    /// Boundary results in delivery order; one entry per expression in `values`.
    pub wires: Vec<Wire<W>>,
    /// Transfer ownership only after all foreign delivery operations succeed.
    pub commit: TokenStream,
    /// Code producing owned output temporaries and evaluating observations before moves.
    pub statements: TokenStream,
    /// Delivery expressions in wire order; they must remain valid until commit.
    pub values: Vec<TokenStream>,
}
/// A slot after recursive conversion preparation; wrapper writers consume this stage.
#[derive(Clone, Debug, Default)]
pub enum Prepared<W> {
    /// Unconfigured occurrence, such as a function used only as a constructor.
    #[default]
    Empty,
    /// Prepared source argument.
    Input(PreparedInput<W>),
    /// Prepared source result.
    Output(PreparedOutput<W>),
}

/// Compile recipes into prepared slots, rejecting incomplete bound functions.
///
/// Each conversion receives source type, recipe and occurrence data separately.
/// Nested recipes inherit their boundary occurrence's data, not the slots of
/// referenced constructors, getters or record declarations. Output expressions
/// refer to a wrapper-local `__result` containing the source function's result.
/// Source guards and locations are preserved.
///
/// Errors include missing/wrong-direction slots, incompatible recipe/type pairs,
/// invalid constructor/getter signatures, and repeated output consumption.
/// A function is exported iff its result has an output slot, including `fields([])` for unit.
pub fn prepare<C: Codec>(flat: Flat<Slot<C::Extra>>, codec: &C) -> Result<Flat<Prepared<C::Wire>>> {
    for f in flat.functions.values() {
        match &f.ret.slot.recipe {
            Recipe::Output(_) => {
                for p in &f.params {
                    if !matches!(p.ty.slot.recipe, Recipe::Input(_)) {
                        return Err(format!(
                            "`{}`: parameter `{}` has no input converter",
                            f.name, p.name
                        ));
                    }
                }
            }
            Recipe::Empty
                if f.params
                    .iter()
                    .all(|p| matches!(p.ty.slot.recipe, Recipe::Empty)) => {}
            _ => {
                return Err(format!(
                    "`{}`: incomplete or wrong-direction conversion slots",
                    f.name
                ))
            }
        }
    }
    let source = flat.clone();
    let mut compiler = Compiler {
        flat: &source,
        codec,
        next: 0,
    };
    flat.map_slots(|ty, slot| {
        Ok(match slot.recipe {
            Recipe::Empty => Prepared::Empty,
            Recipe::Input(recipe) => {
                Prepared::Input(compiler.input(ty, &recipe, &slot.extra, &Condition::default())?)
            }
            Recipe::Output(recipe) => Prepared::Output(compiler.output(
                ty,
                &recipe,
                &slot.extra,
                quote!(__result),
                false,
            )?),
        })
    })
}
struct Compiler<'a, C: Codec> {
    flat: &'a Flat<Slot<C::Extra>>,
    codec: &'a C,
    next: usize,
}
impl<C: Codec> Compiler<'_, C> {
    fn wire(
        &self,
        name: syn::Ident,
        ty: &Type,
        kind: WireKind,
        extra: &C::Extra,
        active: &Condition,
    ) -> Wire<C::Wire> {
        Wire {
            name,
            extra: self.codec.wire_extra(ty, &kind, extra),
            kind,
            active: active.clone(),
        }
    }
    fn name(&mut self) -> syn::Ident {
        let n = format_ident!("w{}", self.next);
        self.next += 1;
        n
    }
    fn input(
        &mut self,
        ty: &Type,
        recipe: &Input,
        extra: &C::Extra,
        active: &Condition,
    ) -> Result<PreparedInput<C::Wire>> {
        let var = self.name();
        let mut wires = Vec::new();
        let mut validation = TokenStream::new();
        let mut acquire = TokenStream::new();
        let mut resources = Vec::new();
        let condition = active.rust();
        let mut statements = TokenStream::new();
        let mut argument = quote!(#var);
        match recipe {
            Input::Scalar => {
                let primitive =
                    Primitive::from_type(ty).ok_or("scalar input needs a primitive type")?;
                let w = self.name();
                let decode = self.codec.decode_scalar(primitive, &w, extra);
                wires.push(self.wire(w, ty, WireKind::Scalar(primitive), extra, active));
                validation = quote!(let #var = if #condition {Some(#decode)} else {None};);
                statements = quote!(let #var:#ty=#var.expect("active scalar"););
            }
            Input::Foreign => {
                let w = self.name();
                return self.codec.input_foreign(ty, extra, w, var, active);
            }
            Input::Text | Input::Bytes => {
                let text = matches!(recipe, Input::Text);
                if matches!(ty, Type::Reference(r) if r.mutability.is_some()) {
                    return Err(
                        "mutable text/byte views require an adapter-specific converter".into(),
                    );
                }
                let (core, borrowed) = match ty {
                    Type::Reference(r) => (&*r.elem, true),
                    _ => (ty, false),
                };
                let key = type_key(core);
                if !(if text {
                    key == "String" || (borrowed && key == "str")
                } else {
                    key == "Vec < u8 >" || (borrowed && key == "[u8]")
                }) {
                    return Err(format!(
                        "{} input cannot produce `{}`",
                        if text { "text" } else { "bytes" },
                        type_key(ty)
                    ));
                }
                let w = self.name();
                wires.push(self.wire(
                    w.clone(),
                    ty,
                    if text {
                        WireKind::Text
                    } else {
                        WireKind::Bytes
                    },
                    extra,
                    active,
                ));
                let decode = if text {
                    self.codec.decode_text(&w, ty, extra)
                } else {
                    self.codec.decode_bytes(&w, ty, extra)
                };
                validation = quote!(let #var = if #condition {Some(#decode)} else {None};);
                statements = quote!(let #var=#var.expect("active buffer"););
                if borrowed {
                    argument = quote!(&#var);
                }
            }
            Input::Fields(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(PreparedInput {
                        wires,
                        validation,
                        acquire,
                        resources,
                        statements,
                        argument: quote!(()),
                    });
                }
                let r = self.flat.record(ty)?.clone();
                if r.fields.len() != children.len() {
                    return Err(format!(
                        "`{}`: expected {} field converters, got {}",
                        type_key(ty),
                        r.fields.len(),
                        children.len()
                    ));
                }
                let mut fields = Vec::new();
                for (field, child) in r.fields.iter().zip(children) {
                    let p = self.input(&field.ty.ty, child, extra, active)?;
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
                    statements.extend(p.statements);
                    wires.extend(p.wires);
                    let name = &field.name;
                    let arg = p.argument;
                    fields.push(quote!(#name:#arg));
                }
                statements.extend(quote!(let #var:#ty=#ty{#(#fields),*};));
            }
            Input::Constructor { function, args } => {
                let f = self.flat.function(function)?.clone();
                if !same_type(&f.ret.ty, ty) {
                    return Err(format!(
                        "constructor `{function}` returns `{}`, expected `{}`",
                        type_key(&f.ret.ty),
                        type_key(ty)
                    ));
                }
                if f.params.len() != args.len() {
                    return Err(format!("constructor `{function}`: wrong argument count"));
                }
                let mut call_args = Vec::new();
                for (param, child) in f.params.iter().zip(args) {
                    let p = self.input(&param.ty.ty, child, extra, active)?;
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
                    statements.extend(p.statements);
                    wires.extend(p.wires);
                    call_args.push(p.argument);
                }
                let path = &f.path;
                statements.extend(quote!(let #var:#ty=#path(#(#call_args),*);));
            }
            Input::Selector(arms) => {
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                if matches!(ty, Type::Reference(_)) {
                    return Err("selector results must be owned in this subset".into());
                }
                let tag = self.name();
                wires.push(self.wire(tag.clone(), ty, WireKind::Selector, extra, active));
                let count = arms.len() as i32;
                let invalid = active.and(quote!(!(0..#count).contains(&#tag)));
                validation.extend(quote!(if #invalid {return Err("invalid selector".into());}));
                let mut cases = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    let p = self.input(ty, arm, extra, &active.arm(tag.clone(), i as i32))?;
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
                    let body = p.statements;
                    let value = p.argument;
                    wires.extend(p.wires);
                    let i = i as i32;
                    cases.push(quote!(#i=>{#body #value}));
                }
                statements = quote!(let #var:#ty=match #tag{#(#cases,)* _=>unreachable!("validated selector")};);
            }
        }
        Ok(PreparedInput {
            wires,
            validation,
            acquire,
            resources,
            statements,
            argument,
        })
    }
    fn output(
        &mut self,
        ty: &Type,
        recipe: &Output,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<PreparedOutput<C::Wire>> {
        let mut out = PreparedOutput {
            commit: TokenStream::new(),
            wires: Vec::new(),
            statements: TokenStream::new(),
            values: Vec::new(),
        };
        match recipe {
            Output::Scalar => {
                let primitive =
                    Primitive::from_type(ty).ok_or("scalar output needs a primitive type")?;
                let value = if borrowed {
                    let reference = self.name();
                    out.statements
                        .extend(quote!(let #reference: &#ty = #value;));
                    quote!(*#reference)
                } else {
                    value
                };
                let expr = self.codec.encode_scalar(primitive, value, extra);
                let w = self.name();
                out.statements.extend(quote!(let #w=#expr;));
                out.values.push(quote!(#w));
                out.wires.push(self.wire(
                    w,
                    ty,
                    WireKind::Scalar(primitive),
                    extra,
                    &Condition::default(),
                ));
            }
            Output::Foreign => {
                let w = self.name();
                return self.codec.output_foreign(ty, extra, w, value, borrowed);
            }
            Output::Text | Output::Bytes => {
                let text = matches!(recipe, Output::Text);
                if type_key(ty) != if text { "String" } else { "Vec < u8 >" } {
                    return Err(format!("wrong output type `{}`", type_key(ty)));
                }
                let expr = if borrowed {
                    quote!(::core::clone::Clone::clone(#value))
                } else {
                    value
                };
                let w = self.name();
                out.statements = quote!(let #w = #expr;);
                out.values.push(quote!(#w));
                out.wires.push(self.wire(
                    w,
                    ty,
                    if text {
                        WireKind::Text
                    } else {
                        WireKind::Bytes
                    },
                    extra,
                    &Condition::default(),
                ));
            }
            Output::Fields(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(out);
                }
                let r = self.flat.record(ty)?.clone();
                if r.fields.len() != children.len() {
                    return Err("wrong output field count".into());
                }
                for (field, child) in r.fields.iter().zip(children) {
                    let name = &field.name;
                    let p =
                        self.output(&field.ty.ty, child, extra, quote!(&(#value).#name), true)?;
                    append(&mut out, p);
                }
            }
            Output::Getter { function, output } => {
                let f = self.flat.function(function)?.clone();
                let expected: Type = syn::parse_quote!(&#ty);
                if f.params.len() != 1 || !same_type(&f.params[0].ty.ty, &expected) {
                    return Err(format!(
                        "getter `{function}` must take exactly `&{}`",
                        type_key(ty)
                    ));
                }
                if matches!(f.ret.ty, Type::Reference(_)) {
                    return Err("borrowed getter results are outside this subset".into());
                }
                let v = self.name();
                let path = &f.path;
                let arg = if borrowed { value } else { quote!(&#value) };
                out.statements = quote!(let #v=#path(#arg););
                append(
                    &mut out,
                    self.output(&f.ret.ty, output, extra, quote!(#v), false)?,
                );
            }
            Output::Combine(parts) => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                if parts
                    .iter()
                    .filter(|p| p.consumes(ty, extra, self.codec))
                    .count()
                    > 1
                {
                    return Err("a combination cannot consume the same value twice".into());
                }
                let mut bodies = Vec::new();
                for child in parts {
                    let consuming = child.consumes(ty, extra, self.codec);
                    let arg = if borrowed || consuming {
                        value.clone()
                    } else {
                        quote!(&#value)
                    };
                    let mut p = self.output(ty, child, extra, arg, borrowed || !consuming)?;
                    bodies.push((consuming, std::mem::take(&mut p.statements)));
                    append(&mut out, p);
                }
                // Wire order follows the declaration. Observations execute before the move.
                bodies.sort_by_key(|(consumes, _)| *consumes);
                for (_, body) in bodies {
                    out.statements.extend(body);
                }
            }
        }
        Ok(out)
    }
}
fn append<W>(to: &mut PreparedOutput<W>, from: PreparedOutput<W>) {
    to.commit.extend(from.commit);
    to.wires.extend(from.wires);
    to.statements.extend(from.statements);
    to.values.extend(from.values);
}

/// Format generated tokens as a Rust source file.
///
/// Returns a syntax error if the tokens do not form a file. This does not
/// resolve names or type-check generated code; the binding crate must compile it.
pub fn rust_source(tokens: TokenStream) -> Result<String> {
    let file = syn::parse2(tokens).map_err(|e| format!("generated Rust: {e}"))?;
    Ok(prettyplease::unparse(&file))
}

/// Parse one Rust identifier used by a declaration.
///
/// Returns an error for invalid syntax, including paths or multiple identifiers.
pub fn ident(name: &str) -> Result<syn::Ident> {
    syn::parse_str(name).map_err(|_| format!("invalid identifier `{name}`"))
}

#[cfg(test)]
mod tests;

pub mod declarations;
pub use declarations::{
    Declarations, ExpandDecl, ExpandParamDecl, ExpandReturnDecl, FieldsDecl, FunctionDecl,
};

/// A leaf is active only on its selected path. Adapters use the same condition
/// for foreign-side resource locking and native preparation.
#[derive(Clone, Debug, Default)]
pub struct Condition(
    /// Conjunction of `(selector identifier, required arm index)` terms.
    /// An empty vector is always active.
    pub Vec<(syn::Ident, i32)>,
);
impl Condition {
    /// Return a new condition additionally requiring one selector arm.
    ///
    /// Does not modify the original condition; no additional runtime flag is generated.
    pub fn arm(&self, tag: syn::Ident, index: i32) -> Self {
        let mut result = self.clone();
        result.0.push((tag, index));
        result
    }
    /// Emit this condition AND a Rust predicate, short-circuiting the predicate
    /// when the selector path is inactive.
    pub fn and(&self, predicate: TokenStream) -> TokenStream {
        if self.0.is_empty() {
            predicate
        } else {
            let condition = self.rust();
            quote!((#condition) && (#predicate))
        }
    }
    /// Emit the conjunction as a Rust boolean expression; an empty condition emits `true`.
    pub fn rust(&self) -> TokenStream {
        self.0
            .iter()
            .map(|(tag, i)| quote!(#tag == #i))
            .reduce(|a, b| quote!((#a) && (#b)))
            .unwrap_or_else(|| quote!(true))
    }
}
/// Access to one foreign resource during an exported call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    /// Read-only borrow; multiple active shared accesses may alias.
    Shared,
    /// Mutable borrow; conflicts with every other active use of the resource.
    Exclusive,
    /// Ownership transfer; conflicts with every other active use of the resource.
    Move,
}
/// The adapter supplies identity; tools only enforce call-wide access conflicts.
#[derive(Clone, Debug)]
pub struct Resource {
    /// Side-effect-free Rust expression identifying a live resource, such as an address.
    /// All identities compared within a call must share an equality-compatible domain.
    pub identity: TokenStream,
    /// Borrow or ownership requirement imposed by this input conversion.
    pub access: Access,
    /// Path on which this resource is accessed; inactive resources do not conflict.
    pub active: Condition,
}
/// Emit pairwise alias checks across all active input resources.
///
/// Run this code after leaf validation and before **any** ownership acquisition.
/// Shared/shared pairs and mutually exclusive selector arms do not conflict.
/// Conflicts return `Err("conflicting input aliases".into())` from the enclosing
/// wrapper. This does not validate pointer liveness, synchronize threads, or
/// acquire resources; those remain adapter/caller responsibilities.
pub fn validate_resources(resources: &[Resource]) -> TokenStream {
    let mut result = TokenStream::new();
    for (i, a) in resources.iter().enumerate() {
        for b in &resources[i + 1..] {
            if a.access == Access::Shared && b.access == Access::Shared {
                continue;
            }
            let x = &a.identity;
            let y = &b.identity;
            // Different alternatives of the same selector cannot conflict.
            if a.active
                .0
                .iter()
                .any(|(tag, arm)| b.active.0.iter().any(|(t, i)| t == tag && i != arm))
            {
                continue;
            }
            let mut active = a.active.clone();
            for term in &b.active.0 {
                if !active.0.contains(term) {
                    active.0.push(term.clone());
                }
            }
            let conflict = active.and(quote!((#x) == (#y)));
            result.extend(quote!(if #conflict {return Err("conflicting input aliases".into());}));
        }
    }
    result
}

/// Reusable storage-specific conversions; not a required object model.
pub mod boxed;
