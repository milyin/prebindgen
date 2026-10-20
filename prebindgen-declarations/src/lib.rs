//! Declaration language and composition for preparing conversion slots.
//!
//! [`Declarations::fill`], [`bind_input`] and [`bind_output`] build conversions
//! directly in a [`Bindings`] model. Recipes and codec input data are used only
//! during construction; slots retain wires and code needed by wrapper writers.
//! Generation is owned by the language generator;
//! this crate owns only declaration resolution and slot preparation.
//! Each [`Wire`] carries the adapter's own description in [`Wire::kind`].
//! Reusable value and boxed-allocation leaves live in `prebindgen-conversions`.
#![warn(missing_docs)]
use prebindgen_conversions::{InputPlan, OutputPlan};
use prebindgen_flat::{same_type, type_key, Flat, Result, Wire};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;
mod condition;
pub use condition::Condition;
mod component;
pub use component::{ComponentPath, ComponentSegment};

/// A recipe for converting boundary inputs into one source argument.
///
/// Binding expands the recipe into wires and Rust conversion code.
#[derive(Clone, Debug)]
pub enum Input {
    /// Convert the occurrence type itself with [`Codec::input_leaf`].
    Leaf,
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
    /// `fields` holds one recipe per field of the source record, in declaration
    /// order. Binding verifies the count against the record.
    pub fn fields(fields: impl IntoIterator<Item = Input>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    /// Call a named source constructor with the supplied argument recipes.
    ///
    /// Binding checks its result type and argument count. Argument validation and
    /// ownership acquisition precede the source call.
    ///
    /// # Arguments
    ///
    /// * `name` — unqualified name of a source function whose result type is the
    ///   input's type.
    /// * `args` — one recipe per parameter of that function, in order.
    pub fn constructor(name: impl Into<String>, args: impl IntoIterator<Item = Input>) -> Self {
        Self::Constructor {
            function: name.into(),
            args: args.into_iter().collect(),
        }
    }
    /// Choose between owned representations of the same input type.
    ///
    /// `arms` holds one recipe per alternative; each must produce the input's
    /// type. The caller passes the zero-based index of the chosen alternative in an
    /// extra wire ahead of all arms' wires.
    ///
    /// Requires at least one arm. Each arm retains its own wires; this does not
    /// merge equivalent wire parameters or generate destination-language overloads.
    pub fn selector(arms: impl IntoIterator<Item = Input>) -> Self {
        Self::Selector(arms.into_iter().collect())
    }
}
/// A result conversion recipe, interpreted together with its source type and codec input data.
///
/// Field projections are observations rather than moves. Adapters decide how to
/// copy or otherwise expose a foreign value observed through a shared reference.
#[derive(Clone, Debug)]
pub enum Output {
    /// Convert the occurrence type itself with [`Codec::output_leaf`].
    Leaf,
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
    ///
    /// `fields` holds one recipe per field of the source record, in declaration order.
    pub fn fields(fields: impl IntoIterator<Item = Output>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    /// Observe a value through a source getter and convert the getter's result.
    ///
    /// The getter must take exactly `&T` for the current type `T`, and return an
    /// owned value. Binding rejects mismatched signatures.
    ///
    /// # Arguments
    ///
    /// * `name` — unqualified name of the source getter.
    /// * `output` — recipe for the getter's result.
    pub fn getter(name: impl Into<String>, output: Output) -> Self {
        Self::Getter {
            function: name.into(),
            output: Box::new(output),
        }
    }
    /// Produce multiple outputs from one value.
    ///
    /// `parts` holds one recipe per view of the value; their wires appear in this
    /// order. Binding rejects an empty list or more than one consuming part. The
    /// framework uses [`Codec::consumes`] to order observations before a move.
    pub fn combine(parts: impl IntoIterator<Item = Output>) -> Self {
        Self::Combine(parts.into_iter().collect())
    }
    fn consumes<C: Codec>(&self, ty: &Type, extra: &C::Extra, codec: &C) -> bool {
        match self {
            Self::Leaf => codec.consumes(ty, extra),
            Self::Combine(p) => p.iter().any(|p| p.consumes(ty, extra, codec)),
            _ => false,
        }
    }
}

/// Language-specific leaf conversions. Recursive composition stays in this declaration compiler.
///
/// A leaf is a source type the adapter converts itself: a value copied across
/// the boundary or an object kept in Rust. `prebindgen-conversions` provides helpers
/// for both representations. The
/// adapter chooses by type; a recipe only says where the leaves are.
///
/// Input leaves must validate without consuming in `validation`, then
/// acquire ownership without failure or source calls in `acquire`. Each acquired
/// value needs an ordinary Rust owner so later failures drop it. `statements`
/// can construct arguments from those owners. Every input wire must retain
/// the supplied activation condition. Output leaves keep ownership in Rust
/// locals until their infallible `commit` runs after successful delivery.
///
/// # Implementing a codec
///
/// This minimal codec uses Rust values as its carriers. It illustrates converter
/// construction without a foreign ABI or object storage. A language adapter
/// additionally writes wrappers and chooses native and destination-language
/// carrier types for its wire kinds.
///
/// ```
/// use prebindgen_flat::{Result, validate_bindings};
/// use prebindgen_conversions::{InputPlan, OutputPlan};
/// use prebindgen_conversions::value;
/// use prebindgen_declarations::{
///     Codec, Declarations,
///     Bindings, fun, expand_param, expand_return, bind_input, Input,
/// };
/// use proc_macro2::TokenStream;
/// use quote::quote;
/// use syn::{Ident, Type};
///
/// use prebindgen_declarations::Condition;
/// fn carrier(kind: value::Value) -> Type {
///     match kind {
///         value::Value::Scalar(p) => p.rust_type(),
///         value::Value::Text => syn::parse_quote!(String),
///         value::Value::Bytes => syn::parse_quote!(Vec<u8>),
///     }
/// }
/// struct RustCarriers;
/// impl Codec for RustCarriers {
///     type Extra = ();
///     type Context = ();
///     type Input = InputPlan<Self::Wire>;
///     type Output = OutputPlan<Self::Wire>;
///     type Wire = Type; // The Rust carrier type is the complete wire description.
///
///     fn selector(&self, _: &()) -> Self::Wire { syn::parse_quote!(i32) }
///     fn consumes(&self, _: &Type, _: &()) -> bool { false }
///     fn input_leaf(
///         &self, ty: &Type, _: &(), wire: Ident, var: Ident, active: &Condition,
///     ) -> Result<InputPlan<Self::Wire, Self::Context>> {
///         let kind = value::Value::of(ty).ok_or("only values cross this codec")?;
///         let decode = quote!(#wire.clone());
///         value::input(ty, wire, var, &active.rust(), decode, carrier(kind))
///     }
///     fn output_leaf(
///         &self, ty: &Type, _: &(), wire: Ident, v: TokenStream, borrowed: bool,
///     ) -> Result<OutputPlan<Self::Wire, Self::Context>> {
///         let kind = value::Value::of(ty).ok_or("only values cross this codec")?;
///         value::output(ty, wire, v, borrowed, |v| v, carrier(kind))
///     }
/// }
///
/// let source = syn::parse_file(r#"
///     pub fn from_i32(value: i32) -> i64 { value as i64 }
///     pub fn to_i32(value: &i64) -> i32 { *value as i32 }
///     pub fn echo(value: i64) -> i64 { value }
/// "#)?;
/// let mut flat = Bindings::<Type>::parse(
///     source.items.into_iter().map(|item| (item, Default::default())),
/// )?;
/// let defaults = Declarations::new(&flat, vec![
///     expand_param!(i64).variant(fun!(from_i32)).variant_self().into(),
///     expand_return!(i64).field(fun!(to_i32)).field_self().into(),
/// ])?;
/// defaults.fill(&mut flat, &RustCarriers, &fun!(echo), &())?;
/// let function = flat.function("echo")?;
/// // Input: selector, constructor argument, direct value. Output: two values.
/// assert_eq!(function.params[0].ty.slot.as_ref().unwrap().wires.len(), 3);
/// assert_eq!(function.ret.slot.as_ref().unwrap().wires.len(), 2);
/// // Replace just this parameter's converter; later default filling preserves it.
/// bind_input(&mut flat, &RustCarriers, "echo", "value", &Input::Leaf, &())?;
/// defaults.fill(&mut flat, &RustCarriers, &fun!(echo), &())?;
/// validate_bindings(&flat)?;
/// let input = flat.function("echo")?.params[0].ty.slot.as_ref().unwrap();
/// assert_eq!(input.wires.len(), 1);
/// assert_eq!(prebindgen_flat::type_key(&input.wires[0].kind), "i64");
/// assert!(input.resources.is_empty()); // No foreign ownership in this codec.
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait Codec {
    /// Input data borrowed while building one occurrence's conversion; use `()` if unused.
    /// The same value is passed through nested conversions. It is not stored in Flat;
    /// hooks retain only the data needed for emission in their code and wires.
    type Extra;
    /// Context supplied by the language generator when emitting stored templates.
    type Context: prebindgen_conversions::TemplateContext + 'static;
    /// Final self-contained input converter installed after preparation.
    type Input: prebindgen_flat::Conversion + From<InputPlan<Self::Wire, Self::Context>>;
    /// Final self-contained output converter installed after preparation.
    type Output: prebindgen_flat::Conversion + From<OutputPlan<Self::Wire, Self::Context>>;
    /// Preserve a logical path in private preparation metadata when the codec needs it.
    /// The final converter decides how to use this mapping; Flat never interprets it.
    /// Called from the innermost component outward; prepend each segment to its path.
    fn prefix_component(&self, _wire: &mut Self::Wire, _segment: ComponentSegment) {}

    /// Private preparation metadata for a leaf wire. Final slot conversion may
    /// replace it with a transport-only description before exposing wires to a writer.
    type Wire: prebindgen_flat::WireKind + Clone + std::fmt::Debug;
    /// Describe the `i32` arm index wire that the declaration compiler creates for [`Input::Selector`].
    ///
    /// `extra` is the occurrence data of the conversion that contains the selector.
    fn selector(&self, extra: &Self::Extra) -> Self::Wire;
    /// Whether a leaf output consumes the current source value.
    ///
    /// This must agree with [`Self::output_leaf`]. It governs move ordering and
    /// double-consumption rejection in [`Output::Combine`], not input ownership.
    /// A non-consuming leaf receives a shared reference with `borrowed` set.
    ///
    /// # Arguments
    ///
    /// * `ty` — owned source type of the value being delivered.
    /// * `extra` — occurrence data of the conversion, as passed to [`bind_output`].
    fn consumes(&self, ty: &Type, extra: &Self::Extra) -> bool;
    /// Prepare a leaf input of source type `ty` using occurrence data `extra`.
    ///
    /// All fallible checks belong in [`InputPlan::validation`]; do not
    /// consume values there. Return a generation error when this type or access
    /// mode is unsupported.
    ///
    /// # Arguments
    ///
    /// * `ty` — the source parameter type at this position, as written: `T`,
    ///   `&T` or `&mut T`.
    /// * `extra` — occurrence data of the conversion, as passed to [`bind_input`]
    ///   or [`Declarations::fill`].
    /// * `wire` — fresh identifier for the boundary parameter this leaf adds.
    ///   A leaf that needs more wires or locals derives their names from it.
    /// * `var` — fresh identifier for the Rust local that holds the converted value.
    ///   [`InputPlan::argument`] normally refers to it.
    /// * `active` — the selector arms under which this leaf is used. Copy it to every
    ///   returned wire and resource, and guard validation and acquisition with it.
    fn input_leaf(
        &self,
        ty: &Type,
        extra: &Self::Extra,
        wire: syn::Ident,
        var: syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<Self::Wire, Self::Context>>;
    /// Complete a composed parameter plan before it is stored in a slot.
    /// Called once per bound parameter, including records and selectors.
    fn finish_input(
        &self,
        _plan: &mut InputPlan<Self::Wire, Self::Context>,
        _extra: &Self::Extra,
    ) -> Result<()> {
        Ok(())
    }
    /// Complete a composed result plan, including unit and multi-wire results.
    /// Install call-level delivery here, after all output expressions are known.
    fn finish_output(
        &self,
        _plan: &mut OutputPlan<Self::Wire, Self::Context>,
        _extra: &Self::Extra,
    ) -> Result<()> {
        Ok(())
    }
    /// Prepare delivery of the Rust expression `value` as one or more wires.
    ///
    /// Keep unpublished allocations under Rust ownership and transfer them only in
    /// [`OutputPlan::commit`]. Return a generation error for unsupported
    /// conversions. Install final delivery in [`Codec::finish_output`], rather
    /// than on nested leaf plans; recursive composition rejects leaf delivery.
    ///
    /// # Arguments
    ///
    /// * `ty` — the owned source type of the value.
    /// * `extra` — occurrence data of the conversion, as passed to [`bind_output`]
    ///   or [`Declarations::fill`].
    /// * `wire` — fresh identifier for the delivered value. A leaf that needs more
    ///   wires or locals derives their names from it.
    /// * `value` — Rust expression producing the value to deliver.
    /// * `borrowed` — `true` when `value` is a shared reference to `ty`, such as a
    ///   record field observed in place; do not move out of it. `false` when
    ///   `value` is owned.
    fn output_leaf(
        &self,
        ty: &Type,
        extra: &Self::Extra,
        wire: syn::Ident,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<Self::Wire, Self::Context>>;
}
/// A flat model whose slots contain only conversion data for wrapper emission.
///
/// Unbound source helpers have `None` slots. Bound functions need `Some` for their
/// result and every parameter; [`prebindgen_flat::validate_bindings`] checks this before emission.
/// Record fields retain only names and source types. Recipes and [`Codec::Extra`]
/// are construction inputs, not stored in this model.
///
/// Output conversion data cannot be attached to an input position:
/// ```compile_fail
/// use prebindgen_declarations::Bindings;
/// fn swap(flat: &mut Bindings<syn::Type>) {
///     let function = flat.function_mut("foo").unwrap();
///     function.params[0].ty.slot = function.ret.slot.clone();
/// }
/// ```
/// Nor can a recipe be stored in a slot:
/// ```compile_fail
/// use prebindgen_declarations::{Bindings, Input};
/// fn configure(flat: &mut Bindings<syn::Type>) {
///     flat.function_mut("foo").unwrap().params[0].ty.slot = Some(Input::Leaf);
/// }
/// ```
pub type Bindings<W, C = ()> = Flat<InputPlan<W, C>, OutputPlan<W, C>>;

/// Build and attach a converter to a named function parameter.
///
/// Resolves the occurrence's source type from `flat`, checks recipe compatibility,
/// and recursively composes wires and code. Nested conversions use the supplied
/// `extra`, not data from referenced constructors or getters. Recipes and `extra`
/// are borrowed only for this call. A failure leaves the existing slot unchanged.
/// Rebinding an occurrence replaces its converter without changing other slots.
/// Generated names are scoped by parameter position, so separate calls cannot
/// collide within the wrapper. Keep the function signature fixed after binding.
/// Check completeness with [`prebindgen_flat::validate_bindings`].
///
/// # Arguments
///
/// * `flat` — the model holding the function; the parameter's slot is replaced.
/// * `codec` — the adapter's leaf conversions.
/// * `function` — unqualified name of the source function.
/// * `parameter` — name of the parameter, as written in the source.
/// * `recipe` — how to build the argument from boundary inputs.
/// * `extra` — occurrence data passed to every codec call for this conversion.
///
/// Input binding rejects output recipes:
/// ```compile_fail
/// use prebindgen_declarations::{bind_input, Codec, Output};
/// use prebindgen_flat::Flat;
/// fn wrong<C: Codec>(flat: &mut Flat<C::Input, C::Output>, codec: &C, extra: &C::Extra) {
///     bind_input(flat, codec, "foo", "x", &Output::Leaf, extra).unwrap();
/// }
/// ```
pub fn bind_input<C: Codec>(
    flat: &mut Flat<C::Input, C::Output>,
    codec: &C,
    function: &str,
    parameter: &str,
    recipe: &Input,
    extra: &C::Extra,
) -> Result<()> {
    let f = flat.function(function)?;
    let index = f
        .params
        .iter()
        .position(|p| p.name == parameter)
        .ok_or_else(|| format!("`{function}`: unknown parameter `{parameter}`"))?;
    let mut conversion = Compiler::new(flat, codec, format!("p{index}")).input(
        &f.params[index].ty.ty,
        recipe,
        extra,
        &Condition::default(),
    )?;
    codec.finish_input(&mut conversion, extra)?;
    flat.function_mut(function)?.params[index].ty.slot = Some(conversion.into());
    Ok(())
}

/// Build and attach a converter to a function result, replacing any existing one.
///
/// Uses the same construction and failure rules as [`bind_input`]. Output code
/// uses [`prebindgen_conversions::source_result_expr`] for the source call's result local.
/// The recipe is checked for compatible types and signatures, and for repeated
/// consumption of the same value. Unit results use `Output::fields([])`.
///
/// # Arguments
///
/// * `flat` — the model holding the function; the result's slot is replaced.
/// * `codec` — the adapter's leaf conversions.
/// * `function` — unqualified name of the source function.
/// * `recipe` — how to deliver the result as boundary outputs.
/// * `extra` — occurrence data passed to every codec call for this conversion.
///
/// Output binding rejects input recipes:
/// ```compile_fail
/// use prebindgen_declarations::{bind_output, Codec, Input};
/// use prebindgen_flat::Flat;
/// fn wrong<C: Codec>(flat: &mut Flat<C::Input, C::Output>, codec: &C, extra: &C::Extra) {
///     bind_output(flat, codec, "foo", &Input::Leaf, extra).unwrap();
/// }
/// ```
pub fn bind_output<C: Codec>(
    flat: &mut Flat<C::Input, C::Output>,
    codec: &C,
    function: &str,
    recipe: &Output,
    extra: &C::Extra,
) -> Result<()> {
    let mut conversion = Compiler::new(flat, codec, "ret".into()).output(
        &flat.function(function)?.ret.ty,
        recipe,
        extra,
        prebindgen_conversions::source_result_expr(),
        false,
    )?;
    codec.finish_output(&mut conversion, extra)?;
    flat.function_mut(function)?.ret.slot = Some(conversion.into());
    Ok(())
}

struct Compiler<'a, C: Codec> {
    flat: &'a Flat<C::Input, C::Output>,
    codec: &'a C,
    prefix: String,
    next: std::cell::Cell<usize>,
}
impl<'a, C: Codec> Compiler<'a, C> {
    fn new(flat: &'a Flat<C::Input, C::Output>, codec: &'a C, prefix: String) -> Self {
        Self {
            flat,
            codec,
            prefix,
            next: std::cell::Cell::new(0),
        }
    }
    fn name(&self) -> syn::Ident {
        let next = self.next.get();
        let n = format_ident!("{}_w{next}", self.prefix);
        self.next.set(next + 1);
        n
    }
    fn prefix(&self, wires: &mut [Wire<C::Wire>], segment: ComponentSegment) {
        for wire in wires {
            self.codec.prefix_component(&mut wire.kind, segment.clone());
        }
    }
    fn input(
        &self,
        ty: &Type,
        recipe: &Input,
        extra: &C::Extra,
        active: &Condition,
    ) -> Result<InputPlan<C::Wire, C::Context>> {
        let var = self.name();
        let mut wires = Vec::new();
        let mut validation = TokenStream::new();
        let mut acquire = TokenStream::new();
        let mut resources = Vec::new();
        let mut acknowledgement = None;
        let mut statements = TokenStream::new();
        let argument = quote!(#var);
        match recipe {
            Input::Leaf => {
                let w = self.name();
                return self.codec.input_leaf(ty, extra, w, var, active);
            }
            Input::Fields(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(InputPlan {
                        context: Default::default(),
                        acknowledgement,
                        wires,
                        validation,
                        acquire,
                        resources,
                        statements,
                        argument: quote!(()),
                    });
                }
                let r = self.flat.record(ty)?;
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
                    let mut p = self.input(&field.ty, child, extra, active)?;
                    self.prefix(
                        &mut p.wires,
                        ComponentSegment::Field(field.name.to_string()),
                    );
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
                    merge_acknowledgement(&mut acknowledgement, p.acknowledgement)?;
                    statements.extend(p.statements);
                    wires.extend(p.wires);
                    let name = &field.name;
                    let arg = p.argument;
                    fields.push(quote!(#name:#arg));
                }
                statements.extend(quote!(let #var:#ty=#ty{#(#fields),*};));
            }
            Input::Constructor { function, args } => {
                let f = self.flat.function(function)?;
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
                    let mut p = self.input(&param.ty.ty, child, extra, active)?;
                    self.prefix(
                        &mut p.wires,
                        ComponentSegment::Argument(param.name.to_string()),
                    );
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
                    merge_acknowledgement(&mut acknowledgement, p.acknowledgement)?;
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
                wires.push(Wire {
                    name: tag.clone(),
                    kind: {
                        let mut kind = self.codec.selector(extra);
                        self.codec
                            .prefix_component(&mut kind, ComponentSegment::Selector);
                        kind
                    },
                });
                let count = arms.len() as i32;
                let invalid = active.and(quote!(!(0..#count).contains(&#tag)));
                validation.extend(quote!(if #invalid {return Err("invalid selector".into());}));
                let mut cases = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    let mut p = self.input(ty, arm, extra, &active.arm(tag.clone(), i as i32))?;
                    self.prefix(&mut p.wires, ComponentSegment::Alternative(i));
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
                    merge_acknowledgement(&mut acknowledgement, p.acknowledgement)?;
                    let body = p.statements;
                    let value = p.argument;
                    wires.extend(p.wires);
                    let i = i as i32;
                    cases.push(quote!(#i=>{#body #value}));
                }
                statements = quote!(let #var:#ty=match #tag{#(#cases,)* _=>unreachable!("validated selector")};);
            }
        }
        Ok(InputPlan {
            context: Default::default(),
            acknowledgement,
            wires,
            validation,
            acquire,
            resources,
            statements,
            argument,
        })
    }
    fn output(
        &self,
        ty: &Type,
        recipe: &Output,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<C::Wire, C::Context>> {
        let mut out = OutputPlan {
            context: Default::default(),
            delivery: Default::default(),
            commit: TokenStream::new(),
            wires: Vec::new(),
            statements: TokenStream::new(),
            values: Vec::new(),
        };
        match recipe {
            Output::Leaf => {
                let w = self.name();
                return self.codec.output_leaf(ty, extra, w, value, borrowed);
            }
            Output::Fields(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(out);
                }
                let r = self.flat.record(ty)?;
                if r.fields.len() != children.len() {
                    return Err("wrong output field count".into());
                }
                for (field, child) in r.fields.iter().zip(children) {
                    let name = &field.name;
                    let mut p =
                        self.output(&field.ty, child, extra, quote!(&(#value).#name), true)?;
                    self.prefix(&mut p.wires, ComponentSegment::Field(name.to_string()));
                    append(&mut out, p)?;
                }
            }
            Output::Getter { function, output } => {
                let f = self.flat.function(function)?;
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
                let mut child = self.output(&f.ret.ty, output, extra, quote!(#v), false)?;
                self.prefix(&mut child.wires, ComponentSegment::Getter(function.clone()));
                append(&mut out, child)?;
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
                for (index, child) in parts.iter().enumerate() {
                    let consuming = child.consumes(ty, extra, self.codec);
                    let arg = if borrowed || consuming {
                        value.clone()
                    } else {
                        quote!(&#value)
                    };
                    let mut p = self.output(ty, child, extra, arg, borrowed || !consuming)?;
                    self.prefix(&mut p.wires, ComponentSegment::View(index));
                    bodies.push((consuming, std::mem::take(&mut p.statements)));
                    append(&mut out, p)?;
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
fn append<W: prebindgen_flat::WireKind, C>(
    to: &mut OutputPlan<W, C>,
    from: OutputPlan<W, C>,
) -> Result<()> {
    if !from.delivery.statements.is_empty()
        || from.delivery.result.to_string() != quote!(()).to_string()
    {
        return Err("composed output delivery must be installed by Codec::finish_output".into());
    }
    to.commit.extend(from.commit);
    to.wires.extend(from.wires);
    to.statements.extend(from.statements);
    to.values.extend(from.values);
    Ok(())
}

#[cfg(test)]
mod tests;

pub mod declarations;
pub use declarations::{
    Declarations, ExpandDecl, ExpandParamDecl, ExpandReturnDecl, FieldsDecl, FunctionDecl,
};

/// Parse one Rust identifier used by a declaration.
///
/// Returns an error for invalid syntax, including paths or multiple identifiers.
pub fn ident(name: &str) -> Result<syn::Ident> {
    syn::parse_str(name).map_err(|_| format!("invalid identifier `{name}`"))
}

mod representation;
pub use representation::{Component, Representation, RepresentationId, Representations};

fn merge_acknowledgement(
    target: &mut Option<TokenStream>,
    incoming: Option<TokenStream>,
) -> Result<()> {
    if let Some(code) = incoming {
        if let Some(previous) = target {
            if previous.to_string() != code.to_string() {
                return Err("conflicting input acknowledgements".into());
            }
        } else {
            *target = Some(code);
        }
    }
    Ok(())
}
