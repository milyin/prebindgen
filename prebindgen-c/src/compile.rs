//! What one crossing costs on the C wire.
//!
//! [`crate::recipes`] states which parts a value gets across in; this says what
//! each of those parts looks like in C. The registry drives the walk over the
//! table and hands every hook the fragments its parts already produced, so
//! nothing here looks a converter up and nothing here recurses.
//!
//! A [`CFrag`] is what a `ConverterImpl` was, minus the bookkeeping the table
//! now does: no `subs` walk to keep the registry's reachability right by hand,
//! no `pre_stages` chain, and no guessing which of nine categories a type falls
//! into — the recipe already said.

use prebindgen_registry::{
    flat::{Alternative, Function},
    generation::{
        AbiLayout, ChoiceArity, Cleanup, ConverterPlan, Failure, FixedArity, FragmentPlan,
        FragmentUse, NichePlan, Representation, ShapePlan, SiteId, SitePlan,
    },
    recipe::{At, Carrier, Compile, Cx, Frag, Mode, Parts, Role, Validity, Yield},
    FragmentId,
};

use super::*;
use crate::chain::{
    CCall, CFunction, ChoicePlan, OptionalPlan, OptionalRepr, ProductField, ProductPlan,
    SequencePlan,
};

/// The shape selected by the recipe compiler, retaining semantic child edges.
#[derive(Clone)]
pub(crate) enum CShape {
    Atomic,
    Product(Vec<FragmentUse>),
    Optional(FragmentUse),
    Sequence(FragmentUse),
    Choice(Vec<Vec<FragmentUse>>),
    Invoke(Vec<FragmentUse>),
}

/// One already-composed C wire-value operation reused by ordinary and callback sites.
#[derive(Clone)]
pub(crate) enum CValue {
    Direct {
        wire: syn::Type,
        converter: CCall,
        niches: Niches,
    },
    Optional {
        inner: Box<CValue>,
        absent: Option<NicheSlot>,
    },
    OwnedSequence {
        element_wire: syn::Type,
        converter: CCall,
    },
    BorrowedSequence {
        element_wire: syn::Type,
        converter: CCall,
    },
    BorrowedInput {
        element: TypeRef,
        wire: syn::Type,
        reinterpret: bool,
    },
}

impl CValue {
    pub(crate) fn direct(&self) -> Option<(&syn::Type, &CCall)> {
        match self {
            Self::Direct {
                wire, converter, ..
            } => Some((wire, converter)),
            _ => None,
        }
    }

    /// Whether every terminal leaf occupies a real C wire. Structural values
    /// are valid only when the registry-composed children they terminate in are.
    pub(crate) fn has_abi(&self) -> bool {
        match self {
            Self::Direct { wire, .. } => !marker_destination(wire),
            Self::Optional { inner, .. } => inner.has_abi(),
            Self::OwnedSequence { element_wire, .. }
            | Self::BorrowedSequence { element_wire, .. } => !marker_destination(element_wire),
            Self::BorrowedInput { .. } => true,
        }
    }

    pub(crate) fn slots(&self) -> usize {
        match self {
            Self::Direct { .. } => 1,
            Self::Optional {
                inner,
                absent: Some(_),
            } => inner.slots(),
            Self::Optional {
                inner,
                absent: None,
            } => 1 + inner.slots(),
            Self::OwnedSequence { .. }
            | Self::BorrowedSequence { .. }
            | Self::BorrowedInput { .. } => 2,
        }
    }

    pub(crate) fn failure(&self) -> Failure {
        let fallible = match self {
            Self::Direct { converter, .. }
            | Self::OwnedSequence { converter, .. }
            | Self::BorrowedSequence { converter, .. } => converter.fallible(),
            Self::Optional { inner, .. } => inner.failure() == Failure::Fallible,
            Self::BorrowedInput { .. } => false,
        };
        if fallible {
            Failure::Fallible
        } else {
            Failure::Infallible
        }
    }

    pub(crate) fn fields(&self) -> Vec<WireField> {
        match self {
            Self::Direct { wire, .. } => vec![WireField {
                suffix: "",
                wire: wire.clone(),
            }],
            Self::Optional {
                inner,
                absent: Some(_),
            } => inner.fields(),
            Self::Optional {
                inner,
                absent: None,
            } => {
                let mut fields = vec![WireField {
                    suffix: "_present",
                    wire: syn::parse_quote!(bool),
                }];
                fields.extend(inner.fields());
                fields
            }
            Self::OwnedSequence { element_wire, .. }
            | Self::BorrowedSequence { element_wire, .. } => vec![
                WireField {
                    suffix: "",
                    wire: syn::parse_quote!(*mut #element_wire),
                },
                WireField {
                    suffix: "_len",
                    wire: syn::parse_quote!(usize),
                },
            ],
            Self::BorrowedInput { wire, .. } => vec![
                WireField {
                    suffix: "",
                    wire: wire.clone(),
                },
                WireField {
                    suffix: "_len",
                    wire: syn::parse_quote!(usize),
                },
            ],
        }
    }

    pub(crate) fn encode(
        &self,
        val: TokenStream,
        targets: &[TokenStream],
        route: &ErrRoute<'_>,
    ) -> TokenStream {
        match self {
            Self::Direct { converter, .. } => {
                let conv = converter.ident();
                let converted = if converter.fallible() {
                    route_result(quote!(#conv(#val)), route)
                } else {
                    quote!(#conv(#val))
                };
                let target = &targets[0];
                quote!(#target = #converted;)
            }
            Self::Optional {
                inner,
                absent: Some(slot),
            } => {
                let inner_encode = inner.encode(quote!(__x), targets, route);
                let absent = &slot.value;
                let target = &targets[0];
                quote!(
                    match #val {
                        ::core::option::Option::Some(__x) => { #inner_encode }
                        ::core::option::Option::None => { #target = #absent; }
                    }
                )
            }
            Self::Optional {
                inner,
                absent: None,
            } => {
                let present = &targets[0];
                let inner_encode = inner.encode(quote!(__x), &targets[1..], route);
                quote!(
                    match #val {
                        ::core::option::Option::Some(__x) => {
                            #present = true;
                            #inner_encode
                        }
                        ::core::option::Option::None => { #present = false; }
                    }
                )
            }
            Self::OwnedSequence {
                element_wire,
                converter,
            } => {
                let conv = converter.ident();
                let converted = if converter.fallible() {
                    route_result(quote!(#conv(#val)), route)
                } else {
                    quote!(#conv(#val))
                };
                let pointer = &targets[0];
                let length = &targets[1];
                quote!(
                    let __arr: ::std::vec::Vec<#element_wire> = #converted;
                    let (__p, __n) = __cbg_alloc_array(__arr);
                    #pointer = __p;
                    #length = __n;
                )
            }
            Self::BorrowedSequence {
                element_wire,
                converter,
            } => {
                let conv = converter.ident();
                let pointer = &targets[0];
                let length = &targets[1];
                if converter.fallible() {
                    let converted = route_result(quote!(#conv(__value)), route);
                    quote!(
                        let mut __arr: ::std::vec::Vec<#element_wire> = ::std::vec::Vec::new();
                        for __value in #val.iter().copied() {
                            __arr.push(#converted);
                        }
                        let (__p, __n) = __cbg_alloc_array(__arr);
                        #pointer = __p;
                        #length = __n;
                    )
                } else {
                    let map = map_arg(conv, converter.unsafe_());
                    quote!(
                        let __arr: ::std::vec::Vec<#element_wire> =
                            #val.iter().copied().map(#map).collect();
                        let (__p, __n) = __cbg_alloc_array(__arr);
                        #pointer = __p;
                        #length = __n;
                    )
                }
            }
            Self::BorrowedInput { .. } => {
                unreachable!("borrowed input plan cannot encode an output")
            }
        }
    }

    pub(crate) fn effective_niches(&self) -> Niches {
        match self {
            Self::Direct { wire, niches, .. }
                if niches.is_empty() && matches!(wire, syn::Type::Ptr(_)) =>
            {
                let null = null_for(wire);
                Niches::one(syn::parse_quote!(#null), syn::parse_quote!(v.is_null()))
            }
            Self::Direct { niches, .. } => niches.clone(),
            Self::Optional { inner, absent } => match absent {
                Some(_) => inner
                    .effective_niches()
                    .carve()
                    .map_or_else(Niches::empty, |(_, rest)| rest),
                None => Niches::empty(),
            },
            Self::OwnedSequence { .. }
            | Self::BorrowedSequence { .. }
            | Self::BorrowedInput { .. } => Niches::empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CFailureRoute {
    Panic,
    Error(SiteId),
}

/// C-specific payloads held behind the registry's generic frozen vocabulary.
pub(crate) enum CRepresentation {}

impl Representation for CRepresentation {
    type Intermediate = TypeKey;
    type Step = CCall;
    type TerminalCodec = CCall;
    type ProductBridge = CCall;
    type OptionalBridge = CCall;
    type SequenceBridge = CCall;
    type ChoiceBridge = CCall;
    type CallbackBridge = CCall;
    type Niche = String;
    type Cleanup = ();
    type FailureRoute = CFailureRoute;
    type AbiLayout = CValue;
    type Artifact = crate::chain::CArtifact;
}

/// The C adapter's answer for one crossing.
#[derive(Clone)]
pub(crate) struct CFrag {
    /// Stable identity and opaque source spelling of this recipe answer.
    pub(crate) id: FragmentId,
    pub(crate) source: TypeRef,
    /// The C wire type this crossing carries.
    pub(crate) destination: syn::Type,
    /// The generated converter's callable contract and late-rendered plan.
    pub(crate) function: CFunction,
    /// Bit patterns the wire can represent and this conversion never produces.
    pub(crate) niches: Niches,
    /// Inner types this fragment composed from, which is what marks them
    /// reachable in the registry that still emits them.
    pub(crate) subs: Vec<TypeKey>,
    /// One arm's payload plan on its way from
    /// [`Compile::fields`] to [`Compile::choice`].
    ///
    /// `None` for every other fragment, which is what a struct's is.
    pub(crate) arm: Option<Arm>,
    /// What the fragment produces, which is all the registry reads of it.
    pub(crate) yields: Yield,
    /// Registry-checkable shape edges retained beside the C operation payload.
    pub(crate) shape: CShape,
    /// Frozen site wire layout and encoder, shared by ordinary and callback sites.
    pub(crate) value: CValue,
}

/// One alternative's payload plan on its way to [`Compile::choice`].
#[derive(Clone)]
pub(crate) struct Arm {
    /// Ordered payload wires and resolved child calls.
    pub(crate) parts: Vec<ArmPart>,
    /// Inner types the payloads composed from.
    pub(crate) subs: Vec<TypeKey>,
}

#[derive(Clone)]
pub(crate) struct ArmPart {
    pub(crate) use_: FragmentUse,
    pub(crate) wire: syn::Type,
    pub(crate) child: CCall,
    pub(crate) mode: Mode,
    pub(crate) hold_uninit: bool,
}
fn fragment_use(fragment: &CFrag) -> FragmentUse {
    FragmentUse::new(fragment.id.clone(), fragment.yields.clone())
}

impl Carrier for CFrag {
    fn yields(&self) -> Yield {
        self.yields.clone()
    }
}

impl CFrag {
    pub(crate) fn freeze(&self) -> FragmentPlan<CRepresentation> {
        let call = self.function.call().clone();
        let shape = match &self.shape {
            CShape::Atomic => ShapePlan::Atomic(call),
            CShape::Product(parts) => ShapePlan::Product {
                bridge: FixedArity::new(parts.len(), call),
                parts: parts.clone(),
            },
            CShape::Optional(value) => ShapePlan::Optional {
                bridge: call,
                value: value.clone(),
            },
            CShape::Sequence(element) => ShapePlan::Sequence {
                bridge: call,
                element: element.clone(),
            },
            CShape::Choice(arms) => ShapePlan::Choice {
                bridge: ChoiceArity::new(arms.iter().map(Vec::len).collect(), call),
                arms: arms.clone(),
            },
            CShape::Invoke(arguments) => ShapePlan::Invoke {
                bridge: FixedArity::new(arguments.len(), call),
                arguments: arguments.clone(),
            },
        };
        let niche_key = |slot: &NicheSlot| {
            format!(
                "{}=>{}",
                slot.value.to_token_stream(),
                slot.matches.to_token_stream()
            )
        };
        let exposed: Vec<String> = self
            .value
            .effective_niches()
            .slots
            .iter()
            .map(niche_key)
            .collect();
        let consumed: Vec<String> = match &self.value {
            CValue::Optional {
                absent: Some(slot), ..
            } => vec![niche_key(slot)],
            _ => Vec::new(),
        };
        let discriminants = usize::from(!consumed.is_empty());
        let converter = ConverterPlan::new(
            shape,
            NichePlan::new(discriminants, consumed, exposed),
            self.value.failure(),
            Cleanup::None,
        );
        FragmentPlan::new(
            self.id.clone(),
            self.source.clone(),
            TypeKey::from_type(&self.destination),
            converter,
            self.yields.clone(),
        )
    }

    /// One of the adapter's existing converter builders, as a fragment.
    fn from_converter(at: At<'_>, conv: ConverterImpl) -> Self {
        let validity = validity_of(&conv, at.crossing.direction());
        let destination = conv.destination;
        let niches = conv.niches;
        let function = CFunction::complete(conv.function);
        let value = CValue::Direct {
            wire: destination.clone(),
            converter: function.call().clone(),
            niches: niches.clone(),
        };
        Self {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination,
            function,
            niches,
            subs: conv.subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity,
            },
            shape: CShape::Atomic,
            value,
        }
    }

    /// A canonical scalar conversion retained as an operation until final
    /// Rust emission rather than routed through `ConverterImpl::function`.
    fn from_custom(at: At<'_>, plan: crate::chain::CustomPlan, niches: Niches) -> Self {
        let destination = plan.wire.clone();
        let subs = vec![TypeKey::from_type(&destination)];
        let function = CFunction::custom(plan);
        let value = CValue::Direct {
            wire: destination.clone(),
            converter: function.call().clone(),
            niches: niches.clone(),
        };
        Self {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination,
            function,
            niches,
            subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            shape: CShape::Atomic,
            value,
        }
    }

    /// A whole-value output retained as an operation until final Rust emission.
    fn from_output_terminal(at: At<'_>, plan: crate::chain::OutputTerminalPlan) -> Self {
        let destination = plan.wire.clone();
        let function = CFunction::output_terminal(plan);
        let niches = Niches::empty();
        let value = CValue::Direct {
            wire: destination.clone(),
            converter: function.call().clone(),
            niches: niches.clone(),
        };
        Self {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination,
            function,
            niches,
            subs: vec![],
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            shape: CShape::Atomic,
            value,
        }
    }

    /// A whole-value input retained as an operation until final Rust emission.
    fn from_input_terminal(at: At<'_>, plan: crate::chain::InputTerminalPlan) -> Self {
        let destination = plan.wire.clone();
        let function = CFunction::input_terminal(plan);
        let niches = Niches::empty();
        let value = CValue::Direct {
            wire: destination.clone(),
            converter: function.call().clone(),
            niches: niches.clone(),
        };
        Self {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination,
            function,
            niches,
            subs: vec![],
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            shape: CShape::Atomic,
            value,
        }
    }
}

/// How long what this conversion produces stays usable.
///
/// A property of the **conversion**, not of how the crossing was spelled.
/// Reading the spelling is wrong in both directions: a conversion may clone or
/// allocate out of a borrow, and — the case C actually has — a conversion may
/// hand C a pointer *into* a Rust value from a crossing that looks owned.
fn validity_of(conv: &ConverterImpl, direction: Direction) -> Validity {
    match direction {
        // Rust to C. A `*const T` is the zero-copy borrow: `out_borrow_or_result`
        // casts the Rust value's own address, and `repr_c_struct`'s reinterpret
        // does the same, so the pointer dies with the value it points into. A
        // `*mut` is a handle C now owns (`Box::into_raw`) or a block C must free
        // (`__cbg_alloc_cstr`), and every by-value wire is a copy.
        Direction::Deconstruct => match &conv.destination {
            syn::Type::Ptr(p) if p.mutability.is_none() => Validity::Borrowed,
            _ => Validity::SelfSufficient,
        },
        // C to Rust: what the converter's own function hands back. A decode
        // yielding `&'a T` borrows the caller's memory, which is right at a
        // parameter and refused at a return.
        Direction::Construct => match &conv.function.sig.output {
            syn::ReturnType::Type(_, ty) if produces_borrow(ty) => Validity::Borrowed,
            _ => Validity::SelfSufficient,
        },
    }
}

/// Whether a converter's return type hands back a borrow — `&T`, or a
/// `Result<&T, E>` whose success arm is one.
fn produces_borrow(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Reference(_) => true,
        _ => result_parts(ty).is_some_and(|(ok, _)| matches!(ok, syn::Type::Reference(_))),
    }
}

/// The adapter, for the length of one crossing's compilation.
///
/// Holds the binding's declarations and the registry view the emission helpers
/// still read the model through.
pub(crate) struct CCompile<'a, R> {
    pub(crate) gen: &'a CbindgenBuilder,
    pub(crate) registry: &'a R,
}

/// Whether the mirror's declared field wire and the conversion's are the same
/// C type.
///
/// Compared **modulo pointer constness**, because the mirror declares one
/// field and the two directions read it differently: a `String` field is
/// `*mut c_char` in the struct C owns and frees, and the decode takes the same
/// memory as `*const`. That is one wire and two readings of it, not two wires,
/// and C says so with a cast rather than a second field.
fn same_wire(declared: &syn::Type, produced: &syn::Type) -> bool {
    fn strip(t: &syn::Type) -> syn::Type {
        match t {
            syn::Type::Ptr(p) => {
                let inner = strip(&p.elem);
                syn::parse_quote!(*const #inner)
            }
            other => other.clone(),
        }
    }
    TypeKey::from_type(&strip(declared)) == TypeKey::from_type(&strip(produced))
}

/// Whether the mirror holds this field as `MaybeUninit<T>` where the
/// conversion produces a bare `T`.
///
/// One field, two readings again: the decode needs somewhere a C caller's
/// arbitrary bytes can legally sit until they are checked, and the encode
/// writes a value that is already valid.
fn held_uninit(declared: &syn::Type, produced: &syn::Type) -> bool {
    let syn::Type::Path(p) = declared else {
        return false;
    };
    let Some(last) = p.path.segments.last() else {
        return false;
    };
    if last.ident != "MaybeUninit" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return false;
    };
    matches!(args.args.first(), Some(syn::GenericArgument::Type(inner))
        if TypeKey::from_type(inner) == TypeKey::from_type(produced))
}

/// A refusal naming the crossing that could not be answered.
fn refuse(at: At<'_>, why: &str) -> String {
    format!("Cbindgen: {} ({why})", at.crossing.key())
}

impl<R: Conversions> CCompile<'_, R> {
    fn wrap(&self, at: At<'_>, why: &str, conv: Option<ConverterImpl>) -> Frag<Self> {
        conv.map(|c| CFrag::from_converter(at, c))
            .ok_or_else(|| refuse(at, why))
    }
}

impl<R: Conversions> Compile for CCompile<'_, R> {
    type Fragment = CFrag;
    /// C signatures, calls, and callback arguments render from this immutable
    /// site plan.
    type Plan = SitePlan<CRepresentation>;
    type Error = String;

    fn atomic(&mut self, _cx: &mut Cx<'_>, at: At<'_>) -> Frag<Self> {
        let ty = at.crossing.spelled();
        // The field recipe, where a value crosses differently **inside a
        // `data_struct`'s mirror** than it does on its own. Two types have one:
        // `bool`, whose field shares a mirror with the decode that normalises
        // it, and `String`, whose field decodes a null pointer leniently so one
        // field cannot make a whole struct's decode fallible.
        // A `Box`-over-handle rides in a union arm as a bare pointer the C side
        // owns, and that is the only place C crosses one — a handle parameter
        // is spelled `Blob` and reclaimed from its own pointer.
        //
        // Keyed by the **spelling** rather than by a recipe of its own: a recipe is
        // filed under `Crossing::key`, which strips `Box`, so `Box<Blob>` and
        // `Blob` share one recipe and could not be told apart there. A fragment is
        // keyed by the spelling, which is exactly the distinction needed.
        if at.recipe.name() == &crate::recipes::payload() {
            let conv = match at.crossing.direction() {
                Direction::Construct => self.gen.in_boxed_payload(ty),
                Direction::Deconstruct => self.gen.out_boxed_payload(ty),
            };
            return self.wrap(at, "no payload reading for this handle", conv);
        }
        if at.recipe.name() == &crate::recipes::in_field() {
            if at.crossing.direction() == Direction::Construct && r_is_bool(ty) {
                let plan = self
                    .gen
                    .in_terminal(ty, self.registry)
                    .expect("bool has an ordinary input-terminal plan");
                return Ok(CFrag::from_input_terminal(at, plan));
            }
            let conv = match at.crossing.direction() {
                Direction::Construct => self.gen.in_string_field(ty),
                // Only `bool` reads differently on the way out; a `String`
                // field is allocated exactly as a `String` return is.
                Direction::Deconstruct => self.gen.out_bool_field(ty),
            };
            return self.wrap(at, "no field reading for this type", conv);
        }
        if let Some((plan, niches)) =
            self.gen
                .custom_plan(ty, self.registry, at.crossing.direction())
        {
            return Ok(CFrag::from_custom(at, plan, niches));
        }
        if at.crossing.direction() == Direction::Deconstruct {
            if let Some(plan) = self.gen.out_terminal(ty, self.registry) {
                return Ok(CFrag::from_output_terminal(at, plan));
            }
        }
        if at.crossing.direction() == Direction::Construct {
            if let Some(plan) = self.gen.in_terminal(ty, self.registry) {
                return Ok(CFrag::from_input_terminal(at, plan));
            }
        }
        let conv = match at.crossing.direction() {
            Direction::Construct => self.gen.in_borrow(ty),
            Direction::Deconstruct => self.gen.out_borrow_or_result(ty),
        };
        self.wrap(at, "no C representation for this type", conv)
    }

    fn optional(&mut self, _cx: &mut Cx<'_>, at: At<'_>, inner: &CFrag) -> Frag<Self> {
        let Some(elem) = at.crossing.value().optional_inner() else {
            return Err(refuse(
                at,
                "an optional recipe over a type that is not optional",
            ));
        };
        if at.crossing.direction() == Direction::Deconstruct {
            let marker = self.gen.out_arity_marker("option", elem);
            let mut fragment = CFrag::from_converter(at, marker);
            let absent = inner.value.effective_niches().carve().map(|(slot, _)| slot);
            fragment.shape = CShape::Optional(fragment_use(inner));
            fragment.value = CValue::Optional {
                inner: Box::new(inner.value.clone()),
                absent,
            };
            fragment.niches = fragment.value.effective_niches();
            return Ok(fragment);
        }

        let inner_wire = inner.destination.clone();
        let (wire, repr, niches) = if let Some((slot, rest)) = inner.niches.clone().carve() {
            (
                inner_wire,
                OptionalRepr::Niche {
                    absent: slot.matches,
                },
                rest,
            )
        } else {
            let read_direct = matches!(inner_wire, syn::Type::Ptr(_));
            let wire = if read_direct {
                inner_wire
            } else {
                syn::parse_quote!(*const #inner_wire)
            };
            (
                wire,
                OptionalRepr::Nullable { read_direct },
                Niches::empty(),
            )
        };
        let function = CFunction::optional(OptionalPlan {
            ident: format_ident!("__cbg_in_option_{}", sanitize(&elem.key())),
            source: at.crossing.spelled().clone(),
            source_module: self.gen.source_module.clone(),
            wire: wire.clone(),
            converter: inner.function.call().clone(),
            repr,
            borrowed: elem.borrow_target().is_some(),
        });
        let value = CValue::Direct {
            wire: wire.clone(),
            converter: function.call().clone(),
            niches: niches.clone(),
        };
        Ok(CFrag {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination: wire,
            function,
            niches,
            subs: vec![elem.key()],
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            shape: CShape::Optional(fragment_use(inner)),
            value,
        })
    }

    fn sequence(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _elements: Mode,
        inner: &CFrag,
    ) -> Frag<Self> {
        let ty = at.crossing.spelled();
        if at.crossing.direction() == Direction::Deconstruct {
            if let TypeKind::Vec(element) = at.crossing.value().kind() {
                if !marker_destination(&inner.destination) {
                    let function = CFunction::sequence(SequencePlan {
                        ident: format_ident!("__cbg_out_chain_vec_{}", sanitize(&element.key())),
                        source: ty.clone(),
                        element: (**element).clone(),
                        source_module: self.gen.source_module.clone(),
                        child_wire: inner.destination.clone(),
                        child: inner.function.call().clone(),
                    });
                    let value = CValue::OwnedSequence {
                        element_wire: inner.destination.clone(),
                        converter: function.call().clone(),
                    };
                    return Ok(CFrag {
                        id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
                        source: at.crossing.spelled().clone(),
                        destination: syn::parse_quote!(()),
                        function,
                        niches: Niches::empty(),
                        subs: vec![element.key()],
                        arm: None,
                        yields: Yield {
                            ty: at.crossing.value().stripped_key(),
                            mode: at.crossing.mode(),
                            validity: Validity::SelfSufficient,
                        },
                        shape: CShape::Sequence(fragment_use(inner)),
                        value,
                    });
                }
            }
        }
        let conv = match at.crossing.direction() {
            // A `&[E]` is the only run C builds a Rust value out of, and it does
            // it zero-copy from the caller's own block.
            Direction::Construct => self.gen.in_slice(ty),
            // A deconstructed run has no single wire of its own. The frozen
            // CValue below carries its pointer-plus-length ABI and encoder.
            Direction::Deconstruct => Some(self.gen.out_arity_marker("vec", &inner.source)),
        };
        let mut fragment = self.wrap(at, "no C representation for this run", conv)?;
        fragment.shape = CShape::Sequence(fragment_use(inner));
        fragment.value = match at.crossing.direction() {
            Direction::Construct => CValue::BorrowedInput {
                element: inner.source.clone(),
                wire: fragment.destination.clone(),
                reinterpret: scalar_ty(&inner.source).is_none(),
            },
            Direction::Deconstruct => CValue::BorrowedSequence {
                element_wire: inner.destination.clone(),
                converter: inner.function.call().clone(),
            },
        };
        Ok(fragment)
    }

    fn construct(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _func: &Function,
        _args: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(refuse(at, "Cbindgen declares no constructor recipes"))
    }

    fn value_form(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _func: &Function,
        _parts: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(refuse(at, "Cbindgen declares no value-form recipes"))
    }

    fn fields(&mut self, _cx: &mut Cx<'_>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let key = ty.key();
        // A tagged union's arm is a product whose parts do not assemble into a
        // value of their own: they are bound by a `match` and rebuilt on the
        // other side. So this hands `choice` the converted payloads and builds
        // no function — which is what "compose a product's fragment from its
        // parts' fragments" means when the product is one arm of a sum.
        if self.gen.tagged_unions.contains_key(&key) {
            // Which alternative these parts belong to, for a refusal that names
            // the arm the way the declaration writes it.
            let arm_name = self
                .gen
                .union_arm_name(&key, self.registry, parts)
                .unwrap_or_default();
            // Acceptance is decided here, before a part is asked to convert:
            // `payload_field_wire` is where a union says what one of its fields
            // can carry, and its refusals name the shape rather than the
            // missing converter — a `Vec` needs two C wires and a union field
            // has one, which is a fact about the union and not about the
            // sequence. Asking the registry for a `Vec<u8>` conversion first
            // would report an unresolved crossing instead.
            //
            // A refusal here is a **declaration** error, not a missing
            // conversion, so it aborts with the reason rather than leaving the
            // crossing unresolved — the same contract the walk this replaces
            // had, and the same one `prereq_data_structs` uses for a field it
            // cannot mirror.
            //
            // Only the shapes that can **never** cross. "This payload has no
            // converter yet" was the other half of the old check and is not a
            // question any more: the part in hand *is* the conversion, so a
            // payload that reached here has one by construction.
            for (part, _) in parts {
                if let Err(why) = self.gen.payload_shape_refusal(&part.ty) {
                    panic!(
                        "Cbindgen::tagged_union: payload `{}::{}{}` of type `{}` cannot cross: {}",
                        type_short(&key),
                        arm_name,
                        match &part.name.parse::<usize>() {
                            Ok(_) => String::new(),
                            Err(_) => format!(".{}", part.name),
                        },
                        part.ty,
                        why
                    );
                }
            }
            let arm_parts = parts
                .iter()
                .map(|(part, frag)| {
                    // Input planning can precede the matching output fragment.
                    // The output pass later validates the one-wire contract
                    // and refuses fallible encoders before planning Choice.
                    let wire = self
                        .gen
                        .payload_field_wire(&part.ty)
                        .unwrap_or_else(|_| frag.destination.clone());
                    ArmPart {
                        use_: fragment_use(frag),
                        hold_uninit: at.crossing.direction() == Direction::Deconstruct
                            && held_uninit(&wire, &frag.destination),
                        wire,
                        child: frag.function.call().clone(),
                        mode: part.mode,
                    }
                })
                .collect();
            // A payload built inline from the handle's own C name is not a
            // crossing the registry has to resolve — the old walk did not make
            // one either. Marking it reachable would demand a whole-value
            // conversion for `Box<Blob>`, which C has none of: a handle
            // parameter is spelled `Blob`.
            let subs: Vec<TypeKey> = parts
                .iter()
                .filter(|(p, _)| {
                    self.gen.declared_opaque_payload_inner(&p.ty).is_none()
                        && r_boxed_inner(&p.ty).is_none()
                })
                .map(|(p, _)| p.ty.key())
                .collect();
            let function = CFunction::complete(syn::parse_quote!(
                #[allow(dead_code)]
                fn __cbg_arm() {}
            ));
            let destination: syn::Type = syn::parse_quote!(());
            let value = CValue::Direct {
                wire: destination.clone(),
                converter: function.call().clone(),
                niches: Niches::empty(),
            };
            return Ok(CFrag {
                id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
                source: at.crossing.spelled().clone(),
                destination,
                function,
                niches: Niches::empty(),
                subs: subs.clone(),
                arm: Some(Arm {
                    subs,
                    parts: arm_parts,
                }),
                yields: Yield {
                    ty: at.crossing.value().stripped_key(),
                    mode: at.crossing.mode(),
                    validity: Validity::SelfSufficient,
                },
                shape: CShape::Product(parts.iter().map(|(_, frag)| fragment_use(frag)).collect()),
                value,
            });
        }
        let c_struct = self.gen.c_type_ident(&key);
        // Each part converts itself. The three statements of a field's wire —
        // this conversion, its twin in the other direction, and the mirror's
        // own field list — collapse to one: the part's fragment says it, and
        // `data_field_wire` is checked against it rather than re-deriving it.
        for (part, frag) in parts {
            let declared = self.gen.data_field_wire(&part.ty);
            if declared.as_ref().is_some_and(|w| {
                !same_wire(w, &frag.destination) && !held_uninit(w, &frag.destination)
            }) {
                return Err(refuse(
                    at,
                    &format!(
                        "field `{}` crosses as `{}` and its mirror declares `{}`",
                        part.name,
                        frag.destination.to_token_stream(),
                        declared.to_token_stream(),
                    ),
                ));
            }
        }
        let fields = parts
            .iter()
            .map(|(part, frag)| {
                // The mirror holds a tagged-union field as `MaybeUninit`, so a
                // C caller can hand over any discriminant and the decode can
                // check it before assuming it initialised. That is the
                // struct's holding form rather than the union's wire, so the
                // wrap belongs here and the union's own conversion stays the
                // one both directions use.
                let hold_uninit = at.crossing.direction() == Direction::Deconstruct
                    && self
                        .gen
                        .data_field_wire(&part.ty)
                        .is_some_and(|wire| held_uninit(&wire, &frag.destination));
                ProductField {
                    name: format_ident!("{}", part.name),
                    converter: frag.function.call().clone(),
                    mode: part.mode,
                    hold_uninit,
                }
            })
            .collect();
        let subs: Vec<TypeKey> = parts.iter().map(|(p, _)| p.ty.key()).collect();
        let direction = at.crossing.direction();
        let ident = match direction {
            Direction::Construct => CbindgenBuilder::in_name_of(&key),
            Direction::Deconstruct => CbindgenBuilder::out_name_of(&key),
        };
        let wire: syn::Type = syn::parse_quote!(#c_struct);
        let function = CFunction::product(ProductPlan {
            ident,
            source: at.crossing.spelled().clone(),
            source_module: self.gen.source_module.clone(),
            wire: wire.clone(),
            direction,
            fields,
        });
        let value = CValue::Direct {
            wire: wire.clone(),
            converter: function.call().clone(),
            niches: Niches::empty(),
        };
        Ok(CFrag {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination: wire,
            function,
            niches: Niches::empty(),
            subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            shape: CShape::Product(parts.iter().map(|(_, frag)| fragment_use(frag)).collect()),
            value,
        })
    }

    fn choice(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        arms: &[(&Alternative, &CFrag)],
    ) -> Frag<Self> {
        let key = at.crossing.spelled().key();
        let cname = self.gen.c_type_ident(&key);
        let direction = at.crossing.direction();
        let mut subs = Vec::new();
        let mut planned_arms = Vec::with_capacity(arms.len());
        for (alternative, fragment) in arms {
            let Some(arm) = fragment.arm.as_ref() else {
                return Err(refuse(at, "an arm that composed no payload"));
            };
            if direction == Direction::Deconstruct
                && arm.parts.iter().any(|part| part.child.fallible())
            {
                return Err(refuse(
                    at,
                    "a payload whose encode can fail, which a union has no way to report",
                ));
            }
            subs.extend(arm.subs.iter().cloned());
            planned_arms.push(prebindgen_registry::chain::ChoiceArm {
                alternative: (*alternative).clone(),
                tag: {
                    let tag =
                        syn::LitInt::new(&alternative.index.to_string(), alternative.name.span());
                    syn::parse_quote!(#tag)
                },
                bridge: prebindgen_registry::chain::TupleProduct {
                    parts: arm.parts.iter().map(|part| part.wire.clone()).collect(),
                },
                parts: arm
                    .parts
                    .iter()
                    .map(|part| prebindgen_registry::chain::ChoicePart {
                        child: part.child.clone(),
                        mode: part.mode,
                        hold_uninit: part.hold_uninit,
                    })
                    .collect(),
            });
        }

        let destination: syn::Type = syn::parse_quote!(::core::mem::MaybeUninit<#cname>);
        let ident = match direction {
            Direction::Construct => CbindgenBuilder::in_name_of(&key),
            Direction::Deconstruct => CbindgenBuilder::out_name_of(&key),
        };
        let shape_arms = arms
            .iter()
            .map(|(_, fragment)| {
                fragment
                    .arm
                    .as_ref()
                    .expect("validated Choice arm")
                    .parts
                    .iter()
                    .map(|part| part.use_.clone())
                    .collect()
            })
            .collect();
        let function = CFunction::choice(ChoicePlan {
            ident,
            source: at.crossing.spelled().clone(),
            source_module: self.gen.source_module.clone(),
            wire: cname,
            direction,
            arms: planned_arms,
        });
        let value = CValue::Direct {
            wire: destination.clone(),
            converter: function.call().clone(),
            niches: Niches::empty(),
        };
        Ok(CFrag {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination,
            function,
            niches: Niches::empty(),
            subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            shape: CShape::Choice(shape_arms),
            value,
        })
    }

    fn callback(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        fragments: &[&CFrag],
        _result: Option<&CFrag>,
    ) -> Frag<Self> {
        let Some(args) = at.crossing.value().callback_args() else {
            return Err(refuse(at, "a callback recipe over a type that is not one"));
        };
        let key: CallbackKey = args.iter().map(|arg| arg.key()).collect();
        if !self.gen.callbacks.contains_key(&key) {
            return Err(refuse(at, "undeclared callback signature"));
        }
        let c_struct = self.gen.callback_c_ident(&key);
        let destination: syn::Type = syn::parse_quote!(#c_struct);
        let function = CFunction::deferred_invoke(format_ident!(
            "__cbg_in_{}",
            self.gen.callback_c_name(&key)
        ));
        let value = CValue::Direct {
            wire: destination.clone(),
            converter: function.call().clone(),
            niches: Niches::empty(),
        };
        Ok(CFrag {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination,
            function,
            niches: Niches::empty(),
            subs: Vec::new(),
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            shape: CShape::Invoke(fragments.iter().map(|frag| fragment_use(frag)).collect()),
            value,
        })
    }

    /// C hands out borrows deliberately, so a returned one is not an error.
    ///
    /// A zero-copy accessor — `fn(&Sample) -> &ZBytes` — crosses as
    /// `*const zbytes_t`, and C's own contract is that a `const` pointer is
    /// non-owning: the caller neither frees it nor outlives the value it points
    /// into. That is the target's ownership model rather than a weaker check,
    /// and it is why the default strict reading belongs to the JVM and not
    /// here.
    fn tolerates(&self, _role: &Role) -> Validity {
        Validity::Borrowed
    }

    fn plan(
        &mut self,
        _cx: &mut Cx<'_>,
        bound: &Bound,
        root: &CFrag,
    ) -> Result<SitePlan<CRepresentation>, String> {
        let failure_route = (root.value.failure() == Failure::Fallible).then(|| {
            let function = self
                .registry
                .flat()
                .function(&bound.site.owner)
                .expect("site owner is a declared function");
            if function.ret.fallible_parts().is_some() {
                CFailureRoute::Error(SiteId::new(prebindgen_registry::recipe::Site {
                    owner: bound.site.owner.clone(),
                    role: Role::Error,
                }))
            } else {
                CFailureRoute::Panic
            }
        });
        Ok(SitePlan::new(
            SiteId::new(bound.site.clone()),
            bound.clone(),
            root.id.clone(),
            root.yields.clone(),
            AbiLayout::new(root.value.slots(), root.value.clone()),
            failure_route,
            Cleanup::None,
        ))
    }
}
