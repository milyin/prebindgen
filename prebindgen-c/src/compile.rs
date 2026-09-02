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
        AbiLayout, ArtifactId, Cleanup, ConverterPlan, Failure, FragmentPlan, NichePlan,
        OperationId, Representation, ShapePlan, SiteId, SitePlan,
    },
    recipe::{
        At, Carrier, Compile, Crossing, Ctx, Frag, Mode, Parts, Refusal, Role, Site, Validity,
        Yield,
    },
    FragmentId,
};

use super::*;
use crate::chain::{
    CCall, CFunction, ChoicePlan, MarkerOperation, MarkerPlan, OptionalPlan, OptionalRepr,
    ProductField, ProductPlan, SequencePlan, SliceInputPlan,
};

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
    /// The converters this value's decode or encode calls.
    pub(crate) fn calls(&self, out: &mut Vec<prebindgen_registry::write::ArtifactKey>) {
        match self {
            Self::Direct { converter, .. } => out.push(converter.artifact_key()),
            // A sequence hands C a malloc'd block, which the array builder
            // among the memory helpers allocates.
            Self::OwnedSequence { converter, .. } | Self::BorrowedSequence { converter, .. } => {
                out.push(converter.artifact_key());
                out.push(crate::assembly::array_builder_key());
            }
            Self::Optional { inner, .. } => inner.calls(out),
            // A borrowed input is reinterpreted, not converted.
            Self::BorrowedInput { .. } => {}
        }
    }

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
        emit: &prebindgen_registry::RustWriter,
    ) -> TokenStream {
        match self {
            Self::Direct { converter, .. } => {
                let conv = converter.ident(emit);
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
                let inner_encode = inner.encode(quote!(__x), targets, route, emit);
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
                let inner_encode = inner.encode(quote!(__x), &targets[1..], route, emit);
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
                let conv = converter.ident(emit);
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
                let conv = converter.ident(emit);
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
                    let map = map_arg(&conv, converter.unsafe_());
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
    type Cleanup = ();
    type ConverterArtifact = CFunction;
    type FailureRoute = CFailureRoute;
    type AbiLayout = CValue;
    type Artifact = crate::assembly::CFinalArtifact;
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
    /// The shape the registry composed for this fragment, taken through
    /// [`Carrier::composed`]. `None` until the compiler hands it over, which it
    /// does before any other code sees the fragment.
    pub(crate) shape: Option<ShapePlan>,
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
    pub(crate) wire: syn::Type,
    pub(crate) child: CCall,
    pub(crate) mode: Mode,
    pub(crate) hold_uninit: bool,
}

fn c_artifact(kind: &'static str, name: impl Into<String>) -> ArtifactId {
    ArtifactId::new(kind, name).expect("C operation identity is non-empty")
}

fn model_operation(at: At<'_>, kind: &'static str, name: impl Into<String>) -> OperationId {
    model_operation_for(at.crossing.value(), at, kind, name)
}

fn model_operation_for(
    source: &TypeRef,
    at: At<'_>,
    kind: &'static str,
    name: impl Into<String>,
) -> OperationId {
    OperationId::model_artifact(source, c_artifact(kind, name), at.crossing.direction())
}

pub(crate) fn callback_operation(source: &TypeRef) -> OperationId {
    OperationId::model_artifact(
        source,
        c_artifact("c-invoke", "callback-capture"),
        Direction::Construct,
    )
}

fn marker_name(operation: &MarkerOperation) -> &'static str {
    match operation {
        MarkerOperation::ChoiceArm => "choice-arm",
        MarkerOperation::Optional => "optional",
        MarkerOperation::Sequence => "sequence",
        MarkerOperation::Result => "result",
    }
}

fn input_terminal_name(operation: &crate::chain::InputTerminalOperation) -> &'static str {
    match operation {
        crate::chain::InputTerminalOperation::OwnedHandle { .. } => "owned-handle",
        crate::chain::InputTerminalOperation::ValueOpaque { .. } => "value-opaque",
        crate::chain::InputTerminalOperation::Enum { .. } => "enum",
        crate::chain::InputTerminalOperation::String => "string",
        crate::chain::InputTerminalOperation::StringField => "string-field",
        crate::chain::InputTerminalOperation::StrMarker => "str-marker",
        crate::chain::InputTerminalOperation::Bool => "bool",
        crate::chain::InputTerminalOperation::Scalar => "scalar",
    }
}

fn output_terminal_name(operation: &crate::chain::OutputTerminalOperation) -> &'static str {
    match operation {
        crate::chain::OutputTerminalOperation::Unit => "unit",
        crate::chain::OutputTerminalOperation::String => "string",
        crate::chain::OutputTerminalOperation::BoolField => "bool-field",
        crate::chain::OutputTerminalOperation::Scalar => "scalar",
        crate::chain::OutputTerminalOperation::OwnedHandle { .. } => "owned-handle",
        crate::chain::OutputTerminalOperation::OpaqueError { .. } => "opaque-error",
        crate::chain::OutputTerminalOperation::ValueOpaque => "value-opaque",
        crate::chain::OutputTerminalOperation::Enum { .. } => "enum",
    }
}

impl Carrier for CFrag {
    fn yields(&self) -> Yield {
        self.yields.clone()
    }

    fn composed(&mut self, shape: ShapePlan) {
        self.shape = Some(shape);
    }

    /// Narrower than the parts the registry compiled, on purpose: a choice arm
    /// and a construct both drop an opaque union payload and a boxed inner,
    /// which cross inside this conversion and never as crossings of their own.
    fn delegates_to(&self, _parts: &[TypeKey]) -> Vec<TypeKey> {
        self.subs.clone()
    }
}

impl CFrag {
    pub(crate) fn freeze(&self) -> FragmentPlan<CRepresentation> {
        let shape = self
            .shape
            .clone()
            .expect("the registry composes a shape for every fragment");
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
        let plan = FragmentPlan::new(
            self.id.clone(),
            self.source.clone(),
            TypeKey::from_type(&self.destination),
            converter,
            self.yields.clone(),
        );
        if self.function.is_deferred_invoke() {
            // Invoked where it lands rather than through a function of its own:
            // no artifact, and still a conversion the plan can name.
            plan.with_composed_conversion(self.function.clone())
        } else {
            plan.with_artifact(self.function.clone())
        }
    }

    /// A multi-wire crossing whose exact ABI is carried by its frozen value.
    fn from_marker(at: At<'_>, plan: MarkerPlan) -> Self {
        let destination: syn::Type = syn::parse_quote!(());
        let subs = plan.subs.iter().map(TypeRef::key).collect();
        let operation = model_operation(at, "c-marker", marker_name(&plan.operation));
        let function = CFunction::marker(operation, plan);
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
            shape: None,
            function,
            niches,
            subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            value,
        }
    }

    /// A canonical scalar conversion retained as an operation until final
    /// Rust emission rather than routed through a complete-function carrier.
    fn from_custom(at: At<'_>, plan: crate::chain::CustomPlan, niches: Niches) -> Self {
        let destination = plan.wire.clone();
        let subs = vec![TypeKey::from_type(&destination)];
        let operation = model_operation_for(&plan.source, at, "c-terminal", "custom");
        let function = CFunction::custom(operation, plan);
        let value = CValue::Direct {
            wire: destination.clone(),
            converter: function.call().clone(),
            niches: niches.clone(),
        };
        Self {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination,
            shape: None,
            function,
            niches,
            subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            value,
        }
    }

    /// A whole-value output retained as an operation until final Rust emission.
    fn from_output_terminal(at: At<'_>, plan: crate::chain::OutputTerminalPlan) -> Self {
        let destination = plan.wire.clone();
        let operation = model_operation_for(
            &plan.source,
            at,
            "c-terminal-output",
            output_terminal_name(&plan.operation),
        );
        let function = CFunction::output_terminal(operation, plan);
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
            shape: None,
            function,
            niches,
            subs: vec![],
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            value,
        }
    }

    /// A whole-value input retained as an operation until final Rust emission.
    fn from_input_terminal(at: At<'_>, plan: crate::chain::InputTerminalPlan) -> Self {
        let destination = plan.wire.clone();
        let operation = model_operation_for(
            &plan.source,
            at,
            "c-terminal-input",
            input_terminal_name(&plan.operation),
        );
        let function = CFunction::input_terminal(operation, plan);
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
            shape: None,
            function,
            niches,
            subs: vec![],
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            value,
        }
    }

    /// A tagged-union pointer payload retained until final Rust emission.
    fn from_payload(at: At<'_>, plan: crate::chain::PayloadPlan) -> Self {
        let destination = plan.wire.clone();
        let payload_kind = format!("payload-optional-{}-boxed-{}", plan.optional, plan.boxed);
        let operation = model_operation_for(&plan.source, at, "c-terminal", payload_kind);
        let function = CFunction::payload(operation, plan);
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
            shape: None,
            function,
            niches,
            subs: vec![],
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            value,
        }
    }

    /// A source borrow retained until final Rust emission.
    fn from_borrow(at: At<'_>, plan: crate::chain::BorrowPlan) -> Self {
        let destination = plan.wire.clone();
        let subs = vec![plan.source_inner.key()];
        let borrow_kind = match plan.operation {
            crate::chain::BorrowOperation::StrInput => "str-input",
            crate::chain::BorrowOperation::SharedInput => "shared-input",
            crate::chain::BorrowOperation::MutableInput => "mutable-input",
            crate::chain::BorrowOperation::MutableUninitInput => "mutable-uninit-input",
            crate::chain::BorrowOperation::SharedOutput => "shared-output",
        };
        let operation = model_operation_for(&plan.source_inner, at, "c-borrow", borrow_kind);
        let function = CFunction::borrow(operation, plan);
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
            shape: None,
            function,
            niches,
            subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::Borrowed,
            },
            value,
        }
    }

    /// A shared-slice marker retained as semantic data until final emission.
    fn from_slice_input(at: At<'_>, plan: SliceInputPlan) -> Self {
        let destination = plan.wire.clone();
        let element = plan.element.clone();
        let reinterpret = plan.reinterpret;
        let subs = vec![element.key()];
        let operation = model_operation(
            at,
            "c-slice-input",
            if plan.reinterpret {
                "reinterpret"
            } else {
                "direct"
            },
        );
        let function = CFunction::slice_input(operation, plan);
        let niches = Niches::empty();
        let value = CValue::BorrowedInput {
            element,
            wire: destination.clone(),
            reinterpret,
        };
        Self {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination,
            shape: None,
            function,
            niches,
            subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                // Preserve the legacy marker's classification. The actual
                // borrowed wire is explicit in CValue::BorrowedInput.
                validity: Validity::SelfSufficient,
            },
            value,
        }
    }
}

/// The adapter, for the length of one crossing's compilation.
///
/// Holds the binding's declarations. The registry view the emission helpers
/// read the model through arrives per call, on [`Cx`], because the registry
/// drives the walk and cannot lend the view to something it is handed.
pub(crate) struct CCompile<'a> {
    pub(crate) gen: &'a CbindgenBuilder,
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

/// A gap: C has nothing for this crossing. The scan over-approximates, so most
/// bindings leave some unanswered, and whether one matters is decided by
/// whether a declared function reaches it.
fn refuse(at: At<'_>, why: &str) -> Refusal<String> {
    Refusal::Gap(format!("Cbindgen: {} ({why})", at.crossing.key()))
}

/// A wrong declaration, reported whether or not anything reaches this crossing:
/// a recipe over a type that cannot take it, or a mirror that contradicts how a
/// field crosses.
fn wrong(at: At<'_>, why: &str) -> Refusal<String> {
    Refusal::Error(format!("Cbindgen: {} ({why})", at.crossing.key()))
}

impl Compile for CCompile<'_> {
    /// A `()` return is not a value C hands back, so there is no site there.
    fn plans_site(&self, site: &Site, crossing: &Crossing) -> bool {
        // Nothing crosses at a `()` **return**: C has no value to hand back
        // there, and that includes the ok arm of a `Result<(), E>`, which is a
        // return site of its own. Only the return — a unit anywhere else is a
        // position the wrapper still renders, and declining it would leave the
        // renderer asking for a site nobody planned.
        !matches!(site.role, Role::Return)
            || !matches!(
                crossing.spelled().kind(),
                prebindgen_registry::flat::TypeKind::Unit
            )
    }

    type Fragment = CFrag;
    /// C signatures, calls, and callback arguments render from this immutable
    /// site plan.
    type Plan = SitePlan<CRepresentation>;
    type Error = String;

    fn atomic(&mut self, cx: &mut Ctx<'_, Self>, at: At<'_>) -> Frag<Self> {
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
            let plan = self
                .gen
                .payload_plan(ty, at.crossing.direction())
                .ok_or_else(|| refuse(at, "no payload reading for this handle"))?;
            return Ok(CFrag::from_payload(at, plan));
        }
        if at.recipe.name() == &crate::recipes::in_field() {
            if at.crossing.direction() == Direction::Construct && r_is_bool(ty) {
                let plan = self
                    .gen
                    .in_terminal(ty, cx.conversions())
                    .expect("bool has an ordinary input-terminal plan");
                return Ok(CFrag::from_input_terminal(at, plan));
            }
            return match at.crossing.direction() {
                Direction::Construct => self
                    .gen
                    .in_string_field_plan(ty)
                    .map(|plan| CFrag::from_input_terminal(at, plan))
                    .ok_or_else(|| refuse(at, "no field reading for this type")),
                // Only `bool` reads differently on the way out; a `String`
                // field is allocated exactly as a `String` return is.
                Direction::Deconstruct => self
                    .gen
                    .out_bool_field_plan(ty)
                    .map(|plan| CFrag::from_output_terminal(at, plan))
                    .ok_or_else(|| refuse(at, "no field reading for this type")),
            };
        }
        if let Some((plan, niches)) =
            self.gen
                .custom_plan(ty, cx.conversions(), at.crossing.direction())
        {
            return Ok(CFrag::from_custom(at, plan, niches));
        }
        if at.crossing.direction() == Direction::Deconstruct {
            if let Some(plan) = self.gen.out_terminal(ty, cx.conversions()) {
                return Ok(CFrag::from_output_terminal(at, plan));
            }
        }
        if at.crossing.direction() == Direction::Construct {
            if let Some(plan) = self.gen.in_terminal(ty, cx.conversions()) {
                return Ok(CFrag::from_input_terminal(at, plan));
            }
        }
        if let Some(plan) = self.gen.borrow_plan(ty, at.crossing.direction()) {
            return Ok(CFrag::from_borrow(at, plan));
        }
        if at.crossing.direction() == Direction::Deconstruct {
            if let Some(plan) = self.gen.out_marker_plan(MarkerOperation::Result, ty) {
                return Ok(CFrag::from_marker(at, plan));
            }
        }
        Err(refuse(at, "no C representation for this type"))
    }

    fn optional(&mut self, _cx: &mut Ctx<'_, Self>, at: At<'_>, inner: &CFrag) -> Frag<Self> {
        let Some(elem) = at.crossing.value().optional_inner() else {
            return Err(wrong(
                at,
                "an optional recipe over a type that is not optional",
            ));
        };
        if at.crossing.direction() == Direction::Deconstruct {
            let marker = self
                .gen
                .out_marker_plan(MarkerOperation::Optional, elem)
                .expect("Optional output markers accept every resolved inner");
            let mut fragment = CFrag::from_marker(at, marker);
            let absent = inner.value.effective_niches().carve().map(|(slot, _)| slot);
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
        let representation = match &repr {
            OptionalRepr::Niche { .. } => "niche",
            OptionalRepr::Nullable { read_direct: true } => "nullable-direct",
            OptionalRepr::Nullable { read_direct: false } => "nullable-indirect",
        };
        let operation = OperationId::optional_converter(
            at.crossing.value(),
            at.crossing.mode(),
            c_artifact("c-optional-intermediate", representation),
            at.crossing.direction(),
        );
        let function = CFunction::optional(
            operation,
            OptionalPlan {
                source: at.crossing.spelled().clone(),
                wire: wire.clone(),
                converter: inner.function.call().clone(),
                repr,
                borrowed: elem.borrow_target().is_some(),
            },
        );
        let value = CValue::Direct {
            wire: wire.clone(),
            converter: function.call().clone(),
            niches: niches.clone(),
        };
        Ok(CFrag {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination: wire,
            shape: None,
            function,
            niches,
            subs: vec![elem.key()],
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            value,
        })
    }

    fn sequence(
        &mut self,
        _cx: &mut Ctx<'_, Self>,
        at: At<'_>,
        _elements: Mode,
        inner: &CFrag,
    ) -> Frag<Self> {
        let ty = at.crossing.spelled();
        if at.crossing.direction() == Direction::Deconstruct {
            if let TypeKind::Vec(element) = at.crossing.value().kind() {
                if !marker_destination(&inner.destination) {
                    let operation = at.sequence_converter_for(at.crossing.value());
                    let function = CFunction::sequence(
                        operation,
                        SequencePlan {
                            source: ty.clone(),
                            element: (**element).clone(),
                            child_wire: inner.destination.clone(),
                            child: inner.function.call().clone(),
                        },
                    );
                    let value = CValue::OwnedSequence {
                        element_wire: inner.destination.clone(),
                        converter: function.call().clone(),
                    };
                    return Ok(CFrag {
                        id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
                        source: at.crossing.spelled().clone(),
                        destination: syn::parse_quote!(()),
                        shape: None,
                        function,
                        niches: Niches::empty(),
                        subs: vec![element.key()],
                        arm: None,
                        yields: Yield {
                            ty: at.crossing.value().stripped_key(),
                            mode: at.crossing.mode(),
                            validity: Validity::SelfSufficient,
                        },
                        value,
                    });
                }
            }
        }
        let mut fragment = match at.crossing.direction() {
            // A `&[E]` is the only run C builds a Rust value out of, and it does
            // it zero-copy from the caller's own block.
            Direction::Construct => {
                let plan = self
                    .gen
                    .in_slice_plan(ty)
                    .ok_or_else(|| refuse(at, "no C representation for this run"))?;
                CFrag::from_slice_input(at, plan)
            }
            // A deconstructed run has no single wire of its own. The frozen
            // CValue below carries its pointer-plus-length ABI and encoder.
            Direction::Deconstruct => CFrag::from_marker(
                at,
                self.gen
                    .out_marker_plan(MarkerOperation::Sequence, &inner.source)
                    .expect("Sequence output markers accept every resolved element"),
            ),
        };
        if at.crossing.direction() == Direction::Deconstruct {
            fragment.value = CValue::BorrowedSequence {
                element_wire: inner.destination.clone(),
                converter: inner.function.call().clone(),
            };
        }
        Ok(fragment)
    }

    fn construct(
        &mut self,
        _cx: &mut Ctx<'_, Self>,
        at: At<'_>,
        _func: &Function,
        _args: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(wrong(at, "Cbindgen declares no constructor recipes"))
    }

    fn value_form(
        &mut self,
        _cx: &mut Ctx<'_, Self>,
        at: At<'_>,
        _func: &Function,
        _parts: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(wrong(at, "Cbindgen declares no value-form recipes"))
    }

    fn fields(&mut self, cx: &mut Ctx<'_, Self>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self> {
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
                .union_arm_name(&key, cx.conversions(), parts)
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
            let operation = OperationId::shared(
                c_artifact("c-marker", "choice-arm"),
                at.crossing.direction(),
            );
            let function = CFunction::marker(
                operation,
                MarkerPlan {
                    operation: MarkerOperation::ChoiceArm,
                    // Choice retains the exact part FragmentUse edges; this
                    // transient bridge has no independent dependency.
                    subs: Vec::new(),
                },
            );
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
                shape: None,
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
                return Err(wrong(
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
        let wire: syn::Type = syn::parse_quote!(#c_struct);
        let operation = OperationId::product_converter(
            at.crossing.value(),
            at.crossing.mode(),
            c_artifact("c-product-intermediate", "repr-c-struct"),
            at.crossing.direction(),
        );
        let function = CFunction::product(
            operation,
            ProductPlan {
                source: at.crossing.spelled().clone(),
                wire: wire.clone(),
                direction,
                fields,
            },
        );
        let value = CValue::Direct {
            wire: wire.clone(),
            converter: function.call().clone(),
            niches: Niches::empty(),
        };
        Ok(CFrag {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination: wire,
            shape: None,
            function,
            niches: Niches::empty(),
            subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            value,
        })
    }

    fn choice(
        &mut self,
        _cx: &mut Ctx<'_, Self>,
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
                return Err(wrong(at, "an arm that composed no payload"));
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
        let operation = OperationId::choice_converter(
            at.crossing.value(),
            at.crossing.mode(),
            c_artifact("c-choice-intermediate", "repr-c-tagged-union"),
            at.crossing.direction(),
        );
        let function = CFunction::choice(
            operation,
            ChoicePlan {
                source: at.crossing.spelled().clone(),
                wire: cname,
                direction,
                arms: planned_arms,
            },
        );
        let value = CValue::Direct {
            wire: destination.clone(),
            converter: function.call().clone(),
            niches: Niches::empty(),
        };
        Ok(CFrag {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination,
            shape: None,
            function,
            niches: Niches::empty(),
            subs,
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
            value,
        })
    }

    fn callback(
        &mut self,
        _cx: &mut Ctx<'_, Self>,
        at: At<'_>,
        _fragments: &[&CFrag],
        _result: Option<&CFrag>,
    ) -> Frag<Self> {
        let Some(args) = at.crossing.value().callback_args() else {
            return Err(wrong(at, "a callback recipe over a type that is not one"));
        };
        let key: CallbackKey = args.iter().map(|arg| arg.key()).collect();
        if !self.gen.callbacks.contains_key(&key) {
            return Err(refuse(at, "undeclared callback signature"));
        }
        let c_struct = self.gen.callback_c_ident(&key);
        let destination: syn::Type = syn::parse_quote!(#c_struct);
        let operation = callback_operation(at.crossing.value());
        let function = CFunction::deferred_invoke(operation);
        let value = CValue::Direct {
            wire: destination.clone(),
            converter: function.call().clone(),
            niches: Niches::empty(),
        };
        Ok(CFrag {
            id: FragmentId::new(at.crossing.spelled().key(), at.recipe.clone()),
            source: at.crossing.spelled().clone(),
            destination,
            shape: None,
            function,
            niches: Niches::empty(),
            subs: Vec::new(),
            arm: None,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
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
        cx: &mut Ctx<'_, Self>,
        bound: &Bound,
        root: &CFrag,
    ) -> Result<SitePlan<CRepresentation>, String> {
        let failure_route = (root.value.failure() == Failure::Fallible).then(|| {
            let function = cx
                .conversions()
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
