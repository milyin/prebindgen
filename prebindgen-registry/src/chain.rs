//! Shared converter-chain composition for recipe shapes.
//!
//! Adapters choose an intermediate representation and implement its bridge
//! operations. The registry owns the source-value walk and child-call/error
//! propagation. Source [`TypeRef`]s remain opaque until [`Chain::render`] is
//! called by the final Rust writer with [`Emit`].

use proc_macro2::TokenStream;

use crate::{
    flat::{AlternativeForm, TypeRef},
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

/// One source field inside one [`Choice`] arm.
#[derive(Clone)]
pub struct ChoicePart<C> {
    /// Named or positional Rust member, taken from the Flat model.
    pub member: syn::Member,
    /// Child converter selected by the recipe driver.
    pub child: C,
    /// How the part is reached through its containing source value.
    pub mode: Mode,
}

/// One already-composed arm of a [`Choice`] recipe.
#[derive(Clone)]
pub struct ChoiceArm<B, C> {
    /// Rust variant name, taken from the Flat model.
    pub variant: syn::Ident,
    /// Unit, tuple or struct delimiter form, taken from Flat.
    pub form: AlternativeForm,
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

    /// Read one arm's Product intermediate from an inbound value.
    fn arm(&self, value: TokenStream, index: usize) -> TokenStream;

    /// Construct the outbound Choice intermediate with `active` selected.
    ///
    /// Inactive arm storage is representation policy. Implementations must not
    /// manufacture source or child-intermediate values merely to fill it.
    fn build(&self, active: usize, value: TokenStream) -> TokenStream;

    /// Error returned when an inbound selector names no arm.
    fn invalid_tag(&self) -> TokenStream;
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

    fn arm(&self, value: TokenStream, index: usize) -> TokenStream {
        let index = syn::Index::from(index + 1);
        quote::quote!((#value).#index)
    }

    fn build(&self, active: usize, value: TokenStream) -> TokenStream {
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

    fn invalid_tag(&self) -> TokenStream {
        self.invalid.clone()
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
    pub arms: Vec<ChoiceArm<P, C>>,
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
                    let variant = &arm.variant;
                    let arm_value = self.bridge.arm(quote::quote!(v), arm_index);
                    let fields: Vec<_> = arm
                        .parts
                        .iter()
                        .enumerate()
                        .map(|(part_index, part)| {
                            let name = match &part.member {
                                syn::Member::Named(name) => name.clone(),
                                syn::Member::Unnamed(index) => {
                                    syn::Ident::new(&format!("v{}", index.index), index.span)
                                }
                            };
                            let value = arm.bridge.part(quote::quote!(__arm), part_index, &name);
                            (part.member.clone(), child_value(&part.child, value))
                        })
                        .collect();
                    let canonical = match arm.form {
                        AlternativeForm::Unit => {
                            quote::quote!(#canonical_source::#variant)
                        }
                        AlternativeForm::Tuple => {
                            let values = fields.iter().map(|(_, value)| value);
                            quote::quote!(#canonical_source::#variant(#(#values),*))
                        }
                        AlternativeForm::Struct => {
                            let names = fields.iter().map(|(member, _)| match member {
                                syn::Member::Named(name) => name,
                                syn::Member::Unnamed(_) => unreachable!(),
                            });
                            let values = fields.iter().map(|(_, value)| value);
                            quote::quote!(#canonical_source::#variant { #(#names: #values),* })
                        }
                    };
                    let built = self.source_policy.build(canonical);
                    quote::quote!(#tag_pattern => {
                        let __arm = #arm_value;
                        #built
                    })
                });
                let invalid = self.bridge.invalid_tag();
                syn::parse2(quote::quote!({
                    match #tag {
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
                    let variant = &arm.variant;
                    let bindings: Vec<_> = arm
                        .parts
                        .iter()
                        .enumerate()
                        .map(|(index, _)| {
                            syn::Ident::new(&format!("__part{index}"), variant.span())
                        })
                        .collect();
                    let pattern = match arm.form {
                        AlternativeForm::Unit => {
                            quote::quote!(#canonical_source::#variant)
                        }
                        AlternativeForm::Tuple => {
                            quote::quote!(#canonical_source::#variant(#(#bindings),*))
                        }
                        AlternativeForm::Struct => {
                            let names = arm.parts.iter().map(|part| match &part.member {
                                syn::Member::Named(name) => name,
                                syn::Member::Unnamed(_) => unreachable!(),
                            });
                            quote::quote!(#canonical_source::#variant { #(#names: #bindings),* })
                        }
                    };
                    let parts: Vec<_> = arm
                        .parts
                        .iter()
                        .zip(&bindings)
                        .map(|(part, binding)| {
                            let value = match part.mode {
                                Mode::Owned => quote::quote!(#binding),
                                Mode::Shared => quote::quote!(&*#binding),
                                Mode::Exclusive => quote::quote!(&mut *#binding),
                            };
                            let value = child_value(&part.child, value);
                            let name = match &part.member {
                                syn::Member::Named(name) => name.clone(),
                                syn::Member::Unnamed(index) => {
                                    syn::Ident::new(&format!("v{}", index.index), index.span)
                                }
                            };
                            (name, value)
                        })
                        .collect();
                    let arm_value = arm.bridge.build(&parts);
                    let built = self.bridge.build(arm_index, arm_value);
                    quote::quote!(#pattern => #built)
                });
                syn::parse2(quote::quote!({ match #value { #(#arms,)* } }))
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

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use quote::{quote, ToTokens};

    use super::*;
    use crate::flat::ScalarKind;

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
                    variant: syn::parse_quote!(A),
                    form: AlternativeForm::Unit,
                    tag: syn::parse_quote!(0),
                    bridge: TupleProduct { parts: Vec::new() },
                    parts: Vec::new(),
                },
                ChoiceArm {
                    variant: syn::parse_quote!(B),
                    form: AlternativeForm::Tuple,
                    tag: syn::parse_quote!(1),
                    bridge: TupleProduct {
                        parts: vec![syn::parse_quote!(i64)],
                    },
                    parts: vec![ChoicePart {
                        member: syn::Member::Unnamed(syn::Index::from(0)),
                        child,
                        mode: Mode::Owned,
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
                    assert!(body.contains("match (v) . 0"), "{body}");
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
}
