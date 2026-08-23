//! Shared converter-chain composition for recipe shapes.
//!
//! Adapters choose an intermediate representation and implement its bridge
//! operations. The registry owns the source-value walk and child-call/error
//! propagation. Source [`TypeRef`]s remain opaque until [`Chain::render`] is
//! called by the final Rust writer with [`Emit`].

use proc_macro2::TokenStream;

use crate::{
    flat::{Alternative, TypeRef},
    recipe::{Direction, Mode},
    Emit,
};

/// The callable contract of one generated converter.
#[derive(Clone)]
pub struct Call {
    ident: syn::Ident,
    fallible: bool,
    unsafe_: bool,
}

impl Call {
    /// Record a callable contract without retaining its rendered signature.
    pub fn new(ident: syn::Ident, fallible: bool, unsafe_: bool) -> Self {
        Self {
            ident,
            fallible,
            unsafe_,
        }
    }

    /// Recover a legacy function's callable contract during migration.
    pub fn complete(function: &syn::ItemFn) -> Self {
        Self::new(
            function.sig.ident.clone(),
            matches!(
                &function.sig.output,
                syn::ReturnType::Type(_, ty)
                    if crate::types_util::result_parts(ty).is_some()
            ),
            function.sig.unsafety.is_some(),
        )
    }

    /// Generated function identifier.
    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    /// Whether invoking the converter produces a `Result`.
    pub fn fallible(&self) -> bool {
        self.fallible
    }

    /// Whether invoking the converter requires an unsafe context.
    pub fn unsafe_(&self) -> bool {
        self.unsafe_
    }
}

/// Adapter policy for spelling and transparently wrapping a source value.
///
/// These operations run only during final emission. Planning stores the
/// source-side type as a [`TypeRef`] and adapter-owned wrapper policy as data.
pub trait Source: Clone {
    /// Spell the exact source type in its final generated context.
    fn spell(&self, source: &TypeRef, emit: &Emit) -> syn::Type;

    /// Turn a canonical shape value into the exact source spelling.
    fn build(&self, canonical: TokenStream) -> TokenStream {
        canonical
    }

    /// Expose the canonical shape value held by the exact source spelling.
    fn read(&self, source: TokenStream) -> TokenStream {
        source
    }

    /// Read one named Product field from the exact source spelling.
    ///
    /// The default keeps the common unwrapped access free of redundant
    /// parentheses. Wrapper policies override this when their read expression
    /// needs grouping before field access.
    fn field(&self, source: TokenStream, name: &syn::Ident) -> TokenStream {
        quote::quote!(#source.#name)
    }
}

/// One already-planned child converter chain.
///
/// `invoke` returns the raw call expression. The shared composer applies `?`
/// exactly when [`Self::call`] says the child is fallible.
pub trait Child: Clone {
    /// Callable facts needed by parents and exported wrappers.
    fn call(&self) -> &Call;

    /// Invoke this child chain for one intermediate/source value.
    fn invoke(&self, value: TokenStream) -> TokenStream;
}

impl Child for Call {
    fn call(&self) -> &Call {
        self
    }

    fn invoke(&self, value: TokenStream) -> TokenStream {
        let ident = self.ident();
        quote::quote!(#ident(#value))
    }
}

fn child_value<C: Child>(child: &C, value: TokenStream) -> TokenStream {
    let call = child.invoke(value);
    if child.call().fallible() {
        quote::quote!(#call?)
    } else {
        call
    }
}

/// Adapter-selected representation protocol for one Product shape.
pub trait ProductBridge: Clone {
    /// The one intermediate Rust type assigned to this fragment.
    fn intermediate(&self) -> syn::Type;

    /// Read one child intermediate value from `value`.
    fn part(&self, value: TokenStream, index: usize, name: &syn::Ident) -> TokenStream;

    /// Construct the intermediate value from converted children.
    fn build(&self, parts: &[(syn::Ident, TokenStream)]) -> TokenStream;
}

/// Registry-owned tuple representation for Product intermediates.
///
/// Adapters declare only the ordered child intermediate types. Tuple indexing
/// and construction follow from `Shape::Product`, so they belong to the shared
/// composer rather than to each language adapter.
#[derive(Clone)]
pub struct TupleProduct {
    /// One intermediate type per Product position.
    pub parts: Vec<syn::Type>,
}

impl ProductBridge for TupleProduct {
    fn intermediate(&self) -> syn::Type {
        let parts = &self.parts;
        syn::parse_quote!((#(#parts,)*))
    }

    fn part(&self, value: TokenStream, index: usize, _name: &syn::Ident) -> TokenStream {
        let index = syn::Index::from(index);
        quote::quote!((#value).#index)
    }

    fn build(&self, parts: &[(syn::Ident, TokenStream)]) -> TokenStream {
        let values = parts.iter().map(|(_, value)| value);
        quote::quote!((#(#values,)*))
    }
}

/// One Product position and its resolved child chain.
#[derive(Clone)]
pub struct ProductPart<C> {
    /// Source field name and stable representation position label.
    pub name: syn::Ident,
    /// Child converter selected by the recipe driver.
    pub child: C,
    /// How the part is reached through its containing source value.
    pub mode: Mode,
    /// Wrap the converted child in `MaybeUninit` in the intermediate value.
    pub hold_uninit: bool,
}

/// Registry-composed converter plan for a Product recipe.
#[derive(Clone)]
pub struct Product<S, B, C> {
    /// Exact source spelling, opaque until rendering.
    pub source: TypeRef,
    /// Direction of the source/intermediate relation.
    pub direction: Direction,
    /// Source spelling and transparent-wrapper policy.
    pub source_policy: S,
    /// Adapter intermediate representation.
    pub bridge: B,
    /// Ordered source parts and resolved child chains.
    pub parts: Vec<ProductPart<C>>,
}

/// Adapter-selected representation protocol for one Optional shape.
pub trait OptionalBridge: Clone {
    /// The one intermediate Rust type assigned to this fragment.
    fn intermediate(&self) -> syn::Type;

    /// Whether an inbound intermediate named `v` represents absence.
    ///
    /// The composer binds its input as `v` before splicing this predicate,
    /// matching the invariant carried by adapter niche predicates.
    fn is_absent(&self) -> TokenStream;

    /// Extract the child intermediate from a present inbound value.
    fn present(&self, value: TokenStream) -> TokenStream;

    /// Construct the outbound representation of absence.
    fn build_absent(&self) -> TokenStream;

    /// Construct the outbound representation of a converted present child.
    fn build_present(&self, child: TokenStream) -> TokenStream;
}

/// Registry-composed converter plan for an Optional recipe.
#[derive(Clone)]
pub struct Optional<S, B, C> {
    /// Exact source spelling, opaque until rendering.
    pub source: TypeRef,
    /// Direction of the source/intermediate relation.
    pub direction: Direction,
    /// Source spelling and transparent-wrapper policy.
    pub source_policy: S,
    /// Adapter intermediate representation.
    pub bridge: B,
    /// Resolved child converter chain.
    pub child: C,
}

/// Adapter-selected representation protocol for one Sequence shape.
///
/// The shared composer owns the source collection, its element loop and the
/// child converter call. The adapter supplies only operations on its one
/// intermediate collection value. Operation snippets use the local names
/// documented on each method and are rendered only during final emission.
///
/// The composer reserves `__sequence_values`, `__sequence_part`,
/// `__sequence_source`, and `__sequence_element`. Bridge snippets must not
/// bind or shadow those names; they may refer to a name only where its method
/// documents that name as an argument.
pub trait SequenceBridge: Clone {
    /// The one intermediate Rust type assigned to the Sequence fragment.
    fn intermediate(&self) -> syn::Type;

    /// Prepare to read elements from the inbound intermediate value.
    ///
    /// The returned statements may bind adapter state. The next operation is
    /// rendered in the same scope.
    fn begin(&self, value: TokenStream) -> TokenStream;

    /// Produce the next child intermediate, or None when input is exhausted.
    ///
    /// This expression is rendered as the condition of the registry-owned
    /// while-let loop.
    fn next(&self) -> TokenStream;

    /// Prepare one outbound intermediate before source elements are visited.
    /// `source` names the canonical source collection before it is consumed,
    /// so an adapter may inspect its length or other representation metadata.
    ///
    /// The returned statements may bind adapter state. Push and finish are
    /// rendered in the same scope.
    fn begin_output(&self, source: TokenStream) -> TokenStream;

    /// Append one converted child intermediate to the outbound representation.
    fn push(&self, value: TokenStream) -> TokenStream;

    /// Produce the completed outbound intermediate.
    fn finish(&self) -> TokenStream;

    /// Whether the representation operations can fail independently of the
    /// child converter.
    fn fallible(&self) -> bool;
}

/// Registry-composed converter plan for a Sequence recipe.
#[derive(Clone)]
pub struct Sequence<S, B, C> {
    /// Exact source spelling, opaque until rendering.
    pub source: TypeRef,
    /// Exact element spelling, opaque until rendering.
    pub element: TypeRef,
    /// Direction of the source/intermediate relation.
    pub direction: Direction,
    /// Source spelling and transparent-wrapper policy.
    pub source_policy: S,
    /// Adapter collection representation operations.
    pub bridge: B,
    /// Resolved element converter chain.
    pub child: C,
}

/// One source field inside one [`Choice`] arm.
#[derive(Clone)]
pub struct ChoicePart<C> {
    /// Child converter selected by the recipe driver.
    pub child: C,
    /// How the part is reached through its containing source value.
    pub mode: Mode,
    /// Wrap the converted child in `MaybeUninit` in the arm intermediate.
    pub hold_uninit: bool,
}

/// One already-composed arm of a [`Choice`] recipe.
#[derive(Clone)]
pub struct ChoiceArm<B, C> {
    /// The Flat alternative, including its field addresses and delimiters.
    ///
    /// It remains model data until [`Chain::render`] hands it to [`Emit`].
    pub alternative: Alternative,
    /// Adapter tag pattern selecting this arm.
    pub tag: syn::Pat,
    /// Product representation of this arm's intermediate parts.
    pub bridge: B,
    /// Ordered payload parts and their resolved child chains.
    pub parts: Vec<ChoicePart<C>>,
}

/// Adapter-selected representation protocol for one Choice shape.
pub trait ChoiceBridge: Clone {
    /// The one intermediate Rust type assigned to the Choice fragment.
    fn intermediate(&self) -> syn::Type;

    /// Read the selector from an inbound intermediate value.
    fn tag(&self, value: TokenStream) -> TokenStream;

    /// Prepare a validated inbound Choice representation for arm access.
    ///
    /// The selector is read and matched before this operation. This lets an
    /// adapter validate raw storage before turning it into its intermediate
    /// type, and guarantees the whole value is prepared at most once.
    fn prepare(&self, value: TokenStream) -> TokenStream {
        value
    }

    /// Read one arm's Product intermediate from an inbound value.
    fn arm(&self, emit: &Emit, value: TokenStream, index: usize) -> TokenStream;

    /// Construct the outbound Choice intermediate with `active` selected.
    ///
    /// Inactive arm storage is representation policy. Implementations must not
    /// manufacture source or child-intermediate values merely to fill it.
    fn build(&self, emit: &Emit, active: usize, value: TokenStream) -> TokenStream;

    /// Finish the whole outbound representation after its active arm is built.
    fn finish(&self, value: TokenStream) -> TokenStream {
        value
    }

    /// Error returned when an inbound selector names no arm.
    fn invalid_tag(&self, tag: TokenStream) -> TokenStream;
}

/// Registry-owned tuple representation for Choice intermediates.
///
/// Position zero is the selector. Every remaining position holds one arm's
/// Product intermediate. Inactive values are adapter-provided ABI-safe storage,
/// not values of the source type.
#[derive(Clone)]
pub struct TupleChoice {
    /// Selector type.
    pub tag: syn::Type,
    /// One Product intermediate type per arm.
    pub arms: Vec<syn::Type>,
    /// Selector value emitted for each arm.
    pub tags: Vec<syn::Expr>,
    /// ABI-safe inactive storage for each arm.
    pub inactive: Vec<TokenStream>,
    /// Adapter error expression for an invalid inbound selector.
    pub invalid: TokenStream,
}

impl ChoiceBridge for TupleChoice {
    fn intermediate(&self) -> syn::Type {
        let tag = &self.tag;
        let arms = &self.arms;
        syn::parse_quote!((#tag, #(#arms,)*))
    }

    fn tag(&self, value: TokenStream) -> TokenStream {
        quote::quote!((#value).0)
    }

    fn arm(&self, _emit: &Emit, value: TokenStream, index: usize) -> TokenStream {
        let index = syn::Index::from(index + 1);
        quote::quote!((#value).#index)
    }

    fn build(&self, _emit: &Emit, active: usize, value: TokenStream) -> TokenStream {
        let tag = &self.tags[active];
        let arms = self.inactive.iter().enumerate().map(|(index, inactive)| {
            if index == active {
                value.clone()
            } else {
                inactive.clone()
            }
        });
        quote::quote!((#tag, #(#arms,)*))
    }

    fn invalid_tag(&self, tag: TokenStream) -> TokenStream {
        let invalid = &self.invalid;
        quote::quote!({
            let __invalid_tag = #tag;
            #invalid
        })
    }
}

/// Registry-composed converter plan for a Choice recipe.
#[derive(Clone)]
pub struct Choice<S, B, P, C> {
    /// Exact source spelling, opaque until rendering.
    pub source: TypeRef,
    /// Direction of the source/intermediate relation.
    pub direction: Direction,
    /// Source spelling and transparent-wrapper policy.
    pub source_policy: S,
    /// Adapter intermediate representation.
    pub bridge: B,
    /// Every source alternative, already composed from child chains.
    ///
    /// Outbound child conversion may be fallible. An adapter whose generated
    /// boundary has no error channel must refuse that crossing before it
    /// creates this plan; the shared composer propagates every accepted error.
    pub arms: Vec<ChoiceArm<P, C>>,
}

/// One callback argument after its adapter-owned delivery has been rendered.
///
/// A source callback argument may occupy any number of target-language call
/// arguments. The registry owns their ordering and the lifetime of the
/// preparation/cleanup around the foreign invocation; the adapter owns only
/// how its intermediate values are represented.
pub struct RenderedInvokePart {
    /// Statements that turn one source callback argument into intermediates.
    pub prepare: TokenStream,
    /// Target-language call arguments contributed by this source argument.
    pub arguments: Vec<TokenStream>,
    /// Statements run after the target-language invocation.
    pub cleanup: TokenStream,
}

/// Adapter plan for delivering one argument of an [`Invoke`] shape.
///
/// The source [`TypeRef`] is deliberately absent from this hook. The shared
/// composer spells and binds source arguments only during final rendering and
/// hands the adapter the resulting local identifier.
pub trait InvokePart: Clone {
    /// Render target-side delivery for one already-bound source argument.
    fn render(&self, value: &syn::Ident, index: usize, emit: &Emit) -> RenderedInvokePart;
}

/// Adapter-selected callable and invocation protocol for one [`Invoke`] shape.
///
/// `capture` runs once while the source callback is constructed. `surround`
/// runs for every invocation and receives the three ordered phases assembled
/// by the registry. This keeps target-specific guards, local frames and error
/// routes in the adapter without giving it ownership of the source-value walk.
///
/// Tokens returned by `capture`, `invoke`, and `surround` share one generated
/// lexical scope. An adapter that coordinates those hooks through local names
/// therefore owns those names as a reserved-identifier contract. Likewise, an
/// adapter that prepares an [`InvokePart`] against [`Self::argument_name`] must
/// derive both uses from one helper; the composer supplies that same identifier
/// to [`InvokePart::render`] so the contract can be checked at final emission.
pub trait InvokeBridge: Clone {
    /// The one intermediate callable type accepted by the converter.
    fn intermediate(&self) -> syn::Type;

    /// Stable local name for the inbound target callable.
    fn value_name(&self) -> syn::Ident {
        syn::Ident::new("v", proc_macro2::Span::call_site())
    }

    /// Stable local name for one source callback argument.
    fn argument_name(&self, index: usize) -> syn::Ident {
        syn::Ident::new(
            &format!("__invoke_arg{index}"),
            proc_macro2::Span::call_site(),
        )
    }

    /// Capture the target callable and produce the source callback value.
    fn capture(&self, value: TokenStream, closure: TokenStream) -> TokenStream;

    /// Invoke the captured target callable with the flattened arguments.
    fn invoke(&self, arguments: &[TokenStream]) -> TokenStream;

    /// Bracket one invocation with adapter-specific control flow.
    fn surround(
        &self,
        prepare: TokenStream,
        invoke: TokenStream,
        cleanup: TokenStream,
    ) -> TokenStream {
        quote::quote!({ #prepare #invoke #cleanup })
    }

    /// Whether constructing the source callback can fail.
    fn fallible(&self) -> bool;
}

/// Registry-composed converter plan for an Invoke recipe.
#[derive(Clone)]
pub struct Invoke<S, B, P> {
    /// Exact callback spelling, opaque until rendering.
    pub source: TypeRef,
    /// Exact callback argument spellings, opaque until rendering.
    pub arguments: Vec<TypeRef>,
    /// Source spelling and transparent-wrapper policy.
    pub source_policy: S,
    /// Adapter callable/call-site protocol.
    pub bridge: B,
    /// Ordered adapter delivery plans, one per callback argument.
    pub parts: Vec<P>,
}

/// A rendered chain body and the types needed to put a function around it.
pub struct Rendered {
    /// Exact source type, spelled only at final emission.
    pub source: syn::Type,
    /// Adapter-selected intermediate type.
    pub intermediate: syn::Type,
    /// Function-body expression.
    pub body: syn::Expr,
    /// Whether any child call can fail.
    pub fallible: bool,
}

/// A shape chain whose recursive source-value walk is owned by the registry.
pub trait Chain {
    /// Render the already-planned chain at the final emission boundary.
    fn render(&self, emit: &Emit) -> Rendered;
}

impl<S, B, C> Chain for Product<S, B, C>
where
    S: Source,
    B: ProductBridge,
    C: Child,
{
    fn render(&self, emit: &Emit) -> Rendered {
        let source = self.source_policy.spell(&self.source, emit);
        let intermediate = self.bridge.intermediate();
        let fallible = self.parts.iter().any(|part| part.child.call().fallible());
        let body = match self.direction {
            Direction::Construct => {
                let canonical_source = self.source_policy.spell(self.source.unwrapped(), emit);
                let fields: Vec<_> = self
                    .parts
                    .iter()
                    .enumerate()
                    .map(|(index, part)| {
                        let name = part.name.clone();
                        let value = self.bridge.part(quote::quote!(v), index, &name);
                        let value = child_value(&part.child, value);
                        (name, value)
                    })
                    .collect();
                let names = fields.iter().map(|(name, _)| name);
                let values = fields.iter().map(|(_, value)| value);
                let canonical = quote::quote!(#canonical_source { #(#names: #values),* });
                let built = self.source_policy.build(canonical);
                syn::parse2(built).expect("a Product source constructor is a valid expression")
            }
            Direction::Deconstruct => {
                let fields: Vec<_> = self
                    .parts
                    .iter()
                    .map(|part| {
                        let name = part.name.clone();
                        let field = self.source_policy.field(quote::quote!(v), &name);
                        let field = match part.mode {
                            Mode::Owned => field,
                            Mode::Shared => quote::quote!(&(#field)),
                            Mode::Exclusive => quote::quote!(&mut (#field)),
                        };
                        let child = child_value(&part.child, field);
                        let child = if part.hold_uninit {
                            quote::quote!(::core::mem::MaybeUninit::new(#child))
                        } else {
                            child
                        };
                        (name, child)
                    })
                    .collect();
                let built = self.bridge.build(&fields);
                syn::parse2(built)
                    .expect("a Product intermediate constructor is a valid expression")
            }
        };
        Rendered {
            source,
            intermediate,
            body,
            fallible,
        }
    }
}

impl<S, B, C> Chain for Optional<S, B, C>
where
    S: Source,
    B: OptionalBridge,
    C: Child,
{
    fn render(&self, emit: &Emit) -> Rendered {
        let source = self.source_policy.spell(&self.source, emit);
        let intermediate = self.bridge.intermediate();
        let fallible = self.child.call().fallible();
        let body = match self.direction {
            Direction::Construct => {
                let absent = self.bridge.is_absent();
                let present = self.bridge.present(quote::quote!(v));
                let child = child_value(&self.child, quote::quote!(__present));
                let canonical = quote::quote!({
                    if #absent {
                        ::core::option::Option::None
                    } else {
                        let __present = #present;
                        ::core::option::Option::Some(#child)
                    }
                });
                let built = self.source_policy.build(canonical);
                syn::parse2(built).expect("an Optional source constructor is a valid expression")
            }
            Direction::Deconstruct => {
                let value = self.source_policy.read(quote::quote!(v));
                let absent = self.bridge.build_absent();
                let child = child_value(&self.child, quote::quote!(__value));
                let present = self.bridge.build_present(child);
                syn::parse2(quote::quote!({
                    match #value {
                        ::core::option::Option::Some(__value) => #present,
                        ::core::option::Option::None => #absent,
                    }
                }))
                .expect("an Optional intermediate constructor is a valid expression")
            }
        };
        Rendered {
            source,
            intermediate,
            body,
            fallible,
        }
    }
}

impl<S, B, C> Chain for Sequence<S, B, C>
where
    S: Source,
    B: SequenceBridge,
    C: Child,
{
    fn render(&self, emit: &Emit) -> Rendered {
        let source = self.source_policy.spell(&self.source, emit);
        let element = self.source_policy.spell(&self.element, emit);
        let intermediate = self.bridge.intermediate();
        let body = match self.direction {
            Direction::Construct => {
                let begin = self.bridge.begin(quote::quote!(v));
                let next = self.bridge.next();
                let child = child_value(&self.child, quote::quote!(__sequence_part));
                let canonical = quote::quote!({
                    #begin
                    let mut __sequence_values: ::std::vec::Vec<#element> =
                        ::std::vec::Vec::new();
                    while let ::core::option::Option::Some(__sequence_part) = #next {
                        __sequence_values.push(#child);
                    }
                    __sequence_values
                });
                let built = self.source_policy.build(canonical);
                syn::parse2(built).expect("a Sequence source constructor is a valid expression")
            }
            Direction::Deconstruct => {
                let value = self.source_policy.read(quote::quote!(v));
                let begin = self.bridge.begin_output(quote::quote!(__sequence_source));
                let child = child_value(&self.child, quote::quote!(__sequence_element));
                let push = self.bridge.push(quote::quote!(__sequence_part));
                let finish = self.bridge.finish();
                syn::parse2(quote::quote!({
                    let __sequence_source = #value;
                    #begin
                    for __sequence_element in __sequence_source.into_iter() {
                        let __sequence_part = #child;
                        #push
                    }
                    #finish
                }))
                .expect("a Sequence intermediate constructor is a valid expression")
            }
        };
        Rendered {
            source,
            intermediate,
            body,
            fallible: self.bridge.fallible() || self.child.call().fallible(),
        }
    }
}

impl<S, B, P, C> Chain for Choice<S, B, P, C>
where
    S: Source,
    B: ChoiceBridge,
    P: ProductBridge,
    C: Child,
{
    fn render(&self, emit: &Emit) -> Rendered {
        let source = self.source_policy.spell(&self.source, emit);
        let intermediate = self.bridge.intermediate();
        let child_fallible = self
            .arms
            .iter()
            .flat_map(|arm| &arm.parts)
            .any(|part| part.child.call().fallible());
        let body = match self.direction {
            Direction::Construct => {
                let canonical_source = self.source_policy.spell(self.source.unwrapped(), emit);
                let tag = self.bridge.tag(quote::quote!(v));
                let arms = self.arms.iter().enumerate().map(|(arm_index, arm)| {
                    let tag_pattern = &arm.tag;
                    let variant = &arm.alternative.name;
                    let fields: Vec<_> = arm
                        .parts
                        .iter()
                        .enumerate()
                        .map(|(part_index, part)| {
                            let member = arm.alternative.fields[part_index].member();
                            let name = match &member {
                                syn::Member::Named(name) => name.clone(),
                                syn::Member::Unnamed(index) => {
                                    syn::Ident::new(&format!("v{}", index.index), index.span)
                                }
                            };
                            let value = arm.bridge.part(quote::quote!(__arm), part_index, &name);
                            child_value(&part.child, value)
                        })
                        .collect();
                    let shaped: Vec<_> = arm
                        .alternative
                        .fields
                        .iter()
                        .zip(&fields)
                        .map(|(field, value)| field.bind(value))
                        .collect();
                    let canonical = emit.shape_alternative(
                        &arm.alternative,
                        quote::quote!(#canonical_source::#variant),
                        &shaped,
                    );
                    let built = self.source_policy.build(canonical);
                    if arm.parts.is_empty() {
                        quote::quote!(#tag_pattern => #built)
                    } else {
                        let prepared = self.bridge.prepare(quote::quote!(v));
                        let arm_value = self.bridge.arm(emit, quote::quote!(__choice), arm_index);
                        quote::quote!(#tag_pattern => {
                            let __choice = #prepared;
                            let __arm = #arm_value;
                            #built
                        })
                    }
                });
                let invalid = self.bridge.invalid_tag(quote::quote!(__tag));
                syn::parse2(quote::quote!({
                    let __tag = #tag;
                    match __tag {
                        #(#arms,)*
                        _ => return ::core::result::Result::Err(#invalid),
                    }
                }))
                .expect("a Choice source constructor is a valid expression")
            }
            Direction::Deconstruct => {
                let canonical_source = self.source_policy.spell(self.source.unwrapped(), emit);
                let value = self.source_policy.read(quote::quote!(v));
                let arms = self.arms.iter().enumerate().map(|(arm_index, arm)| {
                    let variant = &arm.alternative.name;
                    let bindings: Vec<_> = arm
                        .parts
                        .iter()
                        .enumerate()
                        .map(|(index, _)| {
                            syn::Ident::new(&format!("__part{index}"), variant.span())
                        })
                        .collect();
                    let bound: Vec<_> = arm
                        .alternative
                        .fields
                        .iter()
                        .zip(&bindings)
                        .map(|(field, binding)| field.bind(binding))
                        .collect();
                    let pattern = emit.shape_alternative(
                        &arm.alternative,
                        quote::quote!(#canonical_source::#variant),
                        &bound,
                    );
                    let parts: Vec<_> = arm
                        .parts
                        .iter()
                        .zip(&bindings)
                        .zip(&arm.alternative.fields)
                        .map(|((part, binding), field)| {
                            let value = match part.mode {
                                Mode::Owned => quote::quote!(#binding),
                                Mode::Shared => quote::quote!(&*#binding),
                                Mode::Exclusive => quote::quote!(&mut *#binding),
                            };
                            let value = child_value(&part.child, value);
                            let value = if part.hold_uninit {
                                quote::quote!(::core::mem::MaybeUninit::new(#value))
                            } else {
                                value
                            };
                            let name = match field.member() {
                                syn::Member::Named(name) => name.clone(),
                                syn::Member::Unnamed(index) => {
                                    syn::Ident::new(&format!("v{}", index.index), index.span)
                                }
                            };
                            (name, value)
                        })
                        .collect();
                    let arm_value = arm.bridge.build(&parts);
                    let built = self.bridge.build(emit, arm_index, arm_value);
                    quote::quote!(#pattern => #built)
                });
                let built = quote::quote!({ match #value { #(#arms,)* } });
                let finished = self.bridge.finish(built);
                syn::parse2(finished)
                    .expect("a Choice intermediate constructor is a valid expression")
            }
        };
        Rendered {
            source,
            intermediate,
            body,
            fallible: child_fallible || self.direction == Direction::Construct,
        }
    }
}

impl<S, B, P> Chain for Invoke<S, B, P>
where
    S: Source,
    B: InvokeBridge,
    P: InvokePart,
{
    fn render(&self, emit: &Emit) -> Rendered {
        assert_eq!(
            self.arguments.len(),
            self.parts.len(),
            "an Invoke plan has one delivery plan per callback argument"
        );
        let source = self.source_policy.spell(&self.source, emit);
        let intermediate = self.bridge.intermediate();
        let value_name = self.bridge.value_name();
        let names: Vec<_> = (0..self.arguments.len())
            .map(|index| self.bridge.argument_name(index))
            .collect();
        let types: Vec<_> = self
            .arguments
            .iter()
            .map(|argument| self.source_policy.spell(argument, emit))
            .collect();
        let rendered: Vec<_> = self
            .parts
            .iter()
            .zip(&names)
            .enumerate()
            .map(|(index, (part, name))| part.render(name, index, emit))
            .collect();
        let prepare = rendered.iter().map(|part| &part.prepare);
        let call_arguments: Vec<_> = rendered
            .iter()
            .flat_map(|part| part.arguments.iter().cloned())
            .collect();
        let cleanup = rendered.iter().map(|part| &part.cleanup);
        let invocation = self.bridge.invoke(&call_arguments);
        let surrounded = self.bridge.surround(
            quote::quote!(#(#prepare)*),
            invocation,
            quote::quote!(#(#cleanup)*),
        );
        let closure = quote::quote!(move |#(#names: #types),*| #surrounded);
        let body = self.bridge.capture(quote::quote!(#value_name), closure);
        Rendered {
            source,
            intermediate,
            body: syn::parse2(body).expect("an Invoke callback constructor is a valid expression"),
            fallible: self.bridge.fallible(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use quote::{quote, ToTokens};

    use super::*;
    use crate::flat::{Alternative, Field, Origin, ScalarKind};

    fn alternative(syntax: syn::Variant, index: usize) -> Alternative {
        let location: Rc<prebindgen::SourceLocation> = Rc::new(Default::default());
        let fields = syntax
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| Field {
                name: field.ident.clone(),
                index,
                ty: TypeRef::scalar(ScalarKind::I64),
                origin: Origin::new(field.clone(), location.clone()),
            })
            .collect();
        Alternative {
            name: syntax.ident.clone(),
            index,
            fields,
            origin: Origin::new(syntax, location),
        }
    }

    #[derive(Clone)]
    struct TestSource {
        spells: Rc<Cell<usize>>,
    }

    impl Source for TestSource {
        fn spell(&self, source: &TypeRef, emit: &Emit) -> syn::Type {
            self.spells.set(self.spells.get() + 1);
            emit.spell_ty(source)
        }
    }

    #[derive(Clone)]
    struct TestOptional;

    impl OptionalBridge for TestOptional {
        fn intermediate(&self) -> syn::Type {
            syn::parse_quote!(i64)
        }

        fn is_absent(&self) -> TokenStream {
            quote!(*v == 0)
        }

        fn present(&self, value: TokenStream) -> TokenStream {
            quote!(*#value)
        }

        fn build_absent(&self) -> TokenStream {
            quote!(0)
        }

        fn build_present(&self, child: TokenStream) -> TokenStream {
            child
        }
    }

    #[test]
    fn optional_spells_only_at_final_render_and_owns_child_error_propagation() {
        let spells = Rc::new(Cell::new(0));
        let plan = Optional {
            source: TypeRef::scalar(ScalarKind::I64).optional(),
            direction: Direction::Construct,
            source_policy: TestSource {
                spells: spells.clone(),
            },
            bridge: TestOptional,
            child: Call::new(syn::parse_quote!(decode), true, false),
        };

        assert_eq!(spells.get(), 0, "planning must not spell the TypeRef");
        let rendered = plan.render(&Emit::for_test());
        assert_eq!(spells.get(), 1);
        assert_eq!(
            rendered.source.to_token_stream().to_string(),
            "Option < i64 >"
        );
        let body = rendered.body.to_token_stream().to_string();
        assert!(body.contains("Option :: None"), "{body}");
        assert!(
            body.contains("Option :: Some (decode (__present) ?)"),
            "{body}"
        );
        assert!(rendered.fallible);
    }

    #[derive(Clone)]
    struct TestSequence;

    impl SequenceBridge for TestSequence {
        fn intermediate(&self) -> syn::Type {
            syn::parse_quote!(Vec<i32>)
        }

        fn begin(&self, value: TokenStream) -> TokenStream {
            quote!(let mut __test_sequence = (#value).into_iter();)
        }

        fn next(&self) -> TokenStream {
            quote!(__test_sequence.next())
        }

        fn begin_output(&self, _source: TokenStream) -> TokenStream {
            quote!(let mut __test_sequence = Vec::new();)
        }

        fn push(&self, value: TokenStream) -> TokenStream {
            quote!(__test_sequence.push(#value);)
        }

        fn finish(&self) -> TokenStream {
            quote!(__test_sequence)
        }

        fn fallible(&self) -> bool {
            false
        }
    }

    #[test]
    fn sequence_owns_both_loops_and_child_error_propagation() {
        for direction in [Direction::Construct, Direction::Deconstruct] {
            let spells = Rc::new(Cell::new(0));
            let child_name = match direction {
                Direction::Construct => "decode",
                Direction::Deconstruct => "encode",
            };
            let plan = Sequence {
                source: TypeRef::scalar(ScalarKind::I64),
                element: TypeRef::scalar(ScalarKind::I64),
                direction,
                source_policy: TestSource {
                    spells: spells.clone(),
                },
                bridge: TestSequence,
                child: Call::new(
                    syn::Ident::new(child_name, proc_macro2::Span::call_site()),
                    true,
                    false,
                ),
            };

            assert_eq!(spells.get(), 0);
            let rendered = plan.render(&Emit::for_test());
            assert_eq!(spells.get(), 2);
            let body = rendered.body.to_token_stream().to_string();
            match direction {
                Direction::Construct => {
                    assert!(
                        body.contains("while let :: core :: option :: Option :: Some"),
                        "{body}"
                    );
                    assert!(body.contains("decode (__sequence_part) ?"), "{body}");
                }
                Direction::Deconstruct => {
                    assert!(body.contains("for __sequence_element in"), "{body}");
                    assert!(body.contains("encode (__sequence_element) ?"), "{body}");
                }
            }
            assert!(rendered.fallible);
        }
    }
    fn choice_plan(
        direction: Direction,
        spells: Rc<Cell<usize>>,
    ) -> Choice<TestSource, TupleChoice, TupleProduct, Call> {
        let child = Call::new(
            syn::Ident::new(
                match direction {
                    Direction::Construct => "decode",
                    Direction::Deconstruct => "encode",
                },
                proc_macro2::Span::call_site(),
            ),
            true,
            false,
        );
        Choice {
            source: TypeRef::scalar(ScalarKind::I64),
            direction,
            source_policy: TestSource { spells },
            bridge: TupleChoice {
                tag: syn::parse_quote!(i32),
                arms: vec![syn::parse_quote!(()), syn::parse_quote!((i64,))],
                tags: vec![syn::parse_quote!(0), syn::parse_quote!(1)],
                inactive: vec![quote!(()), quote!((0,))],
                invalid: quote!("invalid tag"),
            },
            arms: vec![
                ChoiceArm {
                    alternative: alternative(syn::parse_quote!(A), 0),
                    tag: syn::parse_quote!(0),
                    bridge: TupleProduct { parts: Vec::new() },
                    parts: Vec::new(),
                },
                ChoiceArm {
                    alternative: alternative(syn::parse_quote!(B(i64)), 1),
                    tag: syn::parse_quote!(1),
                    bridge: TupleProduct {
                        parts: vec![syn::parse_quote!(i64)],
                    },
                    parts: vec![ChoicePart {
                        child,
                        mode: Mode::Owned,
                        hold_uninit: false,
                    }],
                },
            ],
        }
    }

    #[test]
    fn choice_spells_only_at_render_and_owns_arm_control_flow() {
        for direction in [Direction::Construct, Direction::Deconstruct] {
            let spells = Rc::new(Cell::new(0));
            let plan = choice_plan(direction, spells.clone());

            assert_eq!(spells.get(), 0, "planning must not spell the TypeRef");
            let rendered = plan.render(&Emit::for_test());
            assert_eq!(spells.get(), 2);
            let body = rendered.body.to_token_stream().to_string();
            match direction {
                Direction::Construct => {
                    assert!(body.contains("let __tag = (v) . 0"), "{body}");
                    assert!(body.contains("match __tag"), "{body}");
                    assert!(body.contains("i64 :: B (decode ((__arm) . 0) ?)"), "{body}");
                    assert!(
                        body.contains("return :: core :: result :: Result :: Err"),
                        "{body}"
                    );
                }
                Direction::Deconstruct => {
                    assert!(body.contains("match v"), "{body}");
                    assert!(body.contains("i64 :: B (__part0)"), "{body}");
                    assert!(body.contains("encode (__part0) ?"), "{body}");
                    assert!(
                        body.contains("(1 , () , (encode (__part0) ? ,) ,)"),
                        "{body}"
                    );
                }
            }
            assert!(rendered.fallible);
        }
    }

    #[derive(Clone)]
    struct TestInvokePart(&'static str);

    impl InvokePart for TestInvokePart {
        fn render(&self, value: &syn::Ident, _index: usize, _emit: &Emit) -> RenderedInvokePart {
            let prepare = syn::Ident::new(
                &format!("prepare_{}", self.0),
                proc_macro2::Span::call_site(),
            );
            let cleanup = syn::Ident::new(
                &format!("cleanup_{}", self.0),
                proc_macro2::Span::call_site(),
            );
            RenderedInvokePart {
                prepare: quote!(let #prepare = deliver(#value);),
                arguments: vec![quote!(#prepare)],
                cleanup: quote!(#cleanup();),
            }
        }
    }

    #[derive(Clone)]
    struct TestInvoke;

    impl InvokeBridge for TestInvoke {
        fn intermediate(&self) -> syn::Type {
            syn::parse_quote!(Callable)
        }

        fn capture(&self, value: TokenStream, closure: TokenStream) -> TokenStream {
            quote!(capture(#value, #closure))
        }

        fn invoke(&self, arguments: &[TokenStream]) -> TokenStream {
            quote!(call(#(#arguments),*);)
        }

        fn fallible(&self) -> bool {
            false
        }
    }

    #[test]
    fn invoke_spells_only_at_render_and_owns_delivery_order() {
        let spells = Rc::new(Cell::new(0));
        let plan = Invoke {
            source: TypeRef::scalar(ScalarKind::I64),
            arguments: vec![
                TypeRef::scalar(ScalarKind::I32),
                TypeRef::scalar(ScalarKind::U64),
            ],
            source_policy: TestSource {
                spells: spells.clone(),
            },
            bridge: TestInvoke,
            parts: vec![TestInvokePart("a"), TestInvokePart("b")],
        };

        assert_eq!(spells.get(), 0, "planning must not spell callback types");
        let rendered = plan.render(&Emit::for_test());
        assert_eq!(spells.get(), 3);
        let body = rendered.body.to_token_stream().to_string();
        let prepare_a = body.find("prepare_a").unwrap();
        let prepare_b = body.find("prepare_b").unwrap();
        let call = body.find("call (prepare_a , prepare_b)").unwrap();
        let cleanup_a = body.find("cleanup_a").unwrap();
        let cleanup_b = body.find("cleanup_b").unwrap();
        assert!(prepare_a < prepare_b && prepare_b < call);
        assert!(call < cleanup_a && cleanup_a < cleanup_b);
        assert_eq!(
            rendered.intermediate.to_token_stream().to_string(),
            "Callable"
        );
        assert!(!rendered.fallible);
    }
}
