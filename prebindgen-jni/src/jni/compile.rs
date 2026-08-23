//! What one crossing costs on the JNI wire.
//!
//! [`super::recipes`] states which parts a value gets across in; this says what
//! each of those parts looks like across the Java Native Interface. The
//! registry drives the walk over the table and hands every hook the fragment
//! its inner crossing already produced, so nothing here decides which arity
//! layer it is looking at and nothing here recurses.

use prebindgen_registry::{
    flat::{Alternative, Function, TypeKind, TypeRef},
    recipe::{
        At, Bound, Carrier, Compile, Cx, Direction, Frag, Mode, Part, Parts, Validity, Yield,
    },
    Conversions,
};

use super::{
    trait_impl::{Produced, WrapperShape},
    *,
};

/// The JNI adapter's answer for one site.
///
/// Both variants are boxed. A `PlanLeaf` embeds whole sub-plans and runs to
/// ~856 bytes, a `ValueOutputPlan` to ~608, and every site pays the larger of
/// the two — the same reason `FnOutputPlan::Value` boxes its own payload.
pub(crate) enum JPlan {
    /// One parameter, in the seven-way wire layout the emitters branch on.
    Param(Box<crate::jni::fn_plan::PlanLeaf>),
    /// A return that crosses as one value: what it converts through, and what
    /// the Kotlin surface declares.
    Return(Box<crate::jni::fn_plan::ValueOutputPlan>),
    /// A return the binding takes apart: the values it hands out, in the order
    /// the builder receives them.
    ///
    /// Read only by the equivalence fixture until the encode side takes it —
    /// `reach_leaf_flat` and `encode_plan_leaves` still read the plan, because
    /// where the Rust side reaches a value is the plan's and not the wire's.
    #[cfg_attr(not(test), allow(dead_code))]
    Decomposed(Vec<OutWire>),
}

impl JPlan {
    /// This plan as a parameter's, or `None` if it is a return's.
    pub(crate) fn param(self) -> Option<crate::jni::fn_plan::PlanLeaf> {
        match self {
            JPlan::Param(leaf) => Some(*leaf),
            _ => None,
        }
    }

    /// This plan as a return's, or `None` if it is a parameter's.
    pub(crate) fn returned(self) -> Option<crate::jni::fn_plan::ValueOutputPlan> {
        match self {
            JPlan::Return(plan) => Some(*plan),
            _ => None,
        }
    }

    /// The values a decomposed return hands out, or `None` if it is not one.
    #[cfg(test)]
    pub(crate) fn decomposed(self) -> Option<Vec<OutWire>> {
        match self {
            JPlan::Decomposed(wires) => Some(wires),
            _ => None,
        }
    }
}

/// The JNI adapter's answer for one crossing.
///
/// What a `ConverterImpl` was, minus the bookkeeping the table now does.
#[derive(Clone)]
pub(crate) struct JFrag {
    pub(crate) conv: ConverterImpl<KotlinMeta>,
    pub(crate) rust: crate::jni::chain::JFunction,
    pub(crate) yields: Yield,
    /// Shape of the single adapter-side intermediate over flattened ABI leaves.
    pub(crate) layout: Option<JLayout>,
    /// Payload composition retained until `choice` supplies the variant.
    choice_arm: Option<JChoiceArmPlan>,
    /// Composed element retained by a containing sequence for callback folding.
    pub(crate) nested_chain: Option<ComposedChain>,
    /// The wire values this crossing occupies, when it occupies more than the
    /// one `conv.destination` names.
    ///
    /// A JniGen `data_class` parameter arrives as **several** JNI parameters —
    /// `Option<Holder>` is `(hPresent, hTag, hSummary)` — so a single
    /// destination cannot say what it costs. `None` is the ordinary case: one
    /// wire, and `conv.destination` is it.
    ///
    /// Composed by [`Compile::fields`] from the parts' own wires, which is what
    /// makes a nested `data_class` field contribute its own several rather than
    /// one.
    pub(crate) wires: Option<Vec<Wire>>,
    /// The values this crossing hands **out**, when it hands out more than one.
    ///
    /// The deconstructing twin of [`Self::wires`], and a different shape rather
    /// than the same one read backwards: an outgoing value is not filled from a
    /// Kotlin expression, it is produced by the Rust side and named for the
    /// builder parameter it lands in. Never set at the same time as
    /// [`Self::wires`] — a fragment answers one crossing, and a crossing does
    /// one direction.
    pub(crate) out_wires: Option<Vec<OutWire>>,
    /// This fragment states a wire list and nothing else — no conversion of its
    /// own, so nothing of it reaches the generated file.
    ///
    /// Only the `parts` recipe is this. A crossing may legitimately carry **both**
    /// a wire list and a real conversion: an `Option<data_class>` composes a
    /// presence flag ahead of the inner's wires and still has the optional's
    /// own conversion to emit, so "has wires" is the wrong test for what to
    /// leave out of the file.
    pub(crate) composed_only: bool,
}

/// Structural JNI intermediate layout. Leaves are ABI values; Products nest
/// them into the one tuple type consumed or produced by the registry chain.
#[derive(Clone)]
pub(crate) enum JLayout {
    Leaf,
    Product(Vec<JLayout>),
    Optional(Box<JLayout>),
    Choice(Vec<JLayout>),
}

impl JLayout {
    pub(crate) fn leaf_count(&self) -> usize {
        match self {
            Self::Leaf => 1,
            Self::Product(parts) => parts.iter().map(Self::leaf_count).sum(),
            Self::Optional(inner) => 1 + inner.leaf_count(),
            Self::Choice(arms) => 1 + arms.iter().map(Self::leaf_count).sum::<usize>(),
        }
    }

    pub(crate) fn expression(&self, leaves: &[syn::Ident]) -> TokenStream {
        fn build(layout: &JLayout, leaves: &[syn::Ident], next: &mut usize) -> TokenStream {
            match layout {
                JLayout::Leaf => {
                    let leaf = &leaves[*next];
                    *next += 1;
                    quote!(#leaf)
                }
                JLayout::Product(parts) => {
                    let values = parts.iter().map(|part| build(part, leaves, next));
                    quote!((#(#values,)*))
                }
                JLayout::Optional(inner) => {
                    let present = build(&JLayout::Leaf, leaves, next);
                    let value = build(inner, leaves, next);
                    quote!((#present, #value))
                }
                JLayout::Choice(arms) => {
                    let tag = build(&JLayout::Leaf, leaves, next);
                    let arms = arms.iter().map(|arm| build(arm, leaves, next));
                    quote!((#tag, #(#arms,)*))
                }
            }
        }
        assert_eq!(self.leaf_count(), leaves.len());
        build(self, leaves, &mut 0)
    }

    pub(crate) fn is_composed(&self) -> bool {
        matches!(self, Self::Product(_) | Self::Optional(_) | Self::Choice(_))
    }

    pub(crate) fn pattern(&self, leaves: &[syn::Ident]) -> TokenStream {
        self.expression(leaves)
    }
}

#[derive(Clone)]
struct JChoiceArmPlan {
    dependencies: Vec<crate::jni::chain::JFunction>,
    bridge: prebindgen_registry::chain::TupleProduct,
    parts: Vec<prebindgen_registry::chain::ChoicePart<crate::jni::chain::JChild>>,
    layout: JLayout,
}

/// One step of the walk from the object a site names to a wire's value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Nav {
    /// The Kotlin property read.
    pub(crate) field: String,
    /// Whether the value it is read **off** may be null, which makes the read a
    /// safe call.
    ///
    /// Set by every gate above the step, not only the nearest: once a chain
    /// passes through one `?.` every read after it is on a nullable value, so a
    /// gate marks the whole chain below it rather than its first step.
    pub(crate) gated: bool,
}

/// How Kotlin reaches one wire value, relative to the object the site names.
///
/// The walk is kept as steps rather than as one string, and that is what makes
/// a gate composable: an optional is applied **after** the fields under it have
/// composed, so it has to reach back and make every step below it a safe call.
/// A string cannot say which of its dots are property reads — the `0.0` a
/// non-nullable slot falls back to has one too.
///
/// Three forms, because two of them put the base in the **middle**: a sum's tag
/// reads `when (<base>.f) { … }` and a sum's payload slot reads
/// `(<base>.f as? I.V)?.v0`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Access {
    /// `<base><walk><tail>` — the ordinary read. `tail` is whatever follows the
    /// property chain: a presence comparison, an elvis default, a Kotlin enum's
    /// `?.value`, or nothing.
    Read {
        /// The property chain from the base.
        walk: Vec<Nav>,
        /// What follows it.
        tail: String,
    },
    /// `when (<base><walk>) { … }` — which alternative of a sum is live, as the
    /// `jint` tag the arms below are numbered by.
    Select {
        /// The property chain reaching the sum value.
        walk: Vec<Nav>,
        /// One arm per alternative, in declaration order, without the `null`
        /// one — a gate above adds that by setting `nullable`.
        arms: Vec<String>,
        /// Whether the value reached can be null, which is its own arm.
        nullable: bool,
    },
    /// `(<base><walk> as? <class>)?.<read>[ ?: <zero>]` — one payload slot of
    /// one alternative. Every alternative's slots cross on every call; the cast
    /// yields null for the ones that are not live, and `zero` is what a
    /// non-nullable wire carries instead.
    Slot {
        /// The property chain reaching the sum value.
        walk: Vec<Nav>,
        /// The Kotlin class of the alternative this slot belongs to. Empty
        /// until [`Compile::choice`] names it — the arm's own composition
        /// cannot, because a product hook is not told which alternative it is.
        class: String,
        /// What reads the payload off the cast alternative — `v0`, or
        /// `v1?.value` for a Kotlin enum payload.
        read: String,
        /// What an inert slot carries, or `None` for a wire that rides a JVM
        /// `null`.
        zero: Option<String>,
    },
}

/// The Kotlin property chain, rooted at the object a site destructures.
pub(crate) fn reached(base: &str, walk: &[Nav]) -> String {
    let mut out = base.to_string();
    for nav in walk {
        out.push_str(if nav.gated { "?." } else { "." });
        out.push_str(&nav.field);
    }
    out
}

impl Access {
    /// An ordinary read of the base itself, with nothing walked.
    fn read(tail: impl Into<String>) -> Self {
        Access::Read {
            walk: Vec::new(),
            tail: tail.into(),
        }
    }

    /// The Kotlin expression, rooted at the object this site destructures.
    pub(crate) fn render(&self, base: &str) -> String {
        match self {
            Access::Read { walk, tail } => format!("{}{tail}", reached(base, walk)),
            Access::Select {
                walk,
                arms,
                nullable,
            } => {
                let arms = nullable
                    .then(|| "null -> 0".to_string())
                    .into_iter()
                    .chain(arms.iter().cloned())
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("when ({}) {{ {arms} }}", reached(base, walk))
            }
            Access::Slot {
                walk,
                class,
                read,
                zero,
            } => {
                let zero = zero
                    .as_ref()
                    .map(|z| format!(" ?: {z}"))
                    .unwrap_or_default();
                format!("({} as? {class})?.{read}{zero}", reached(base, walk))
            }
        }
    }

    /// The property chain, whichever form the access takes.
    fn walk_mut(&mut self) -> &mut Vec<Nav> {
        match self {
            Access::Read { walk, .. } | Access::Select { walk, .. } | Access::Slot { walk, .. } => {
                walk
            }
        }
    }

    /// This access read from one field in, rather than from the object itself.
    fn under(mut self, field: &str) -> Self {
        self.walk_mut().insert(
            0,
            Nav {
                field: field.to_string(),
                gated: false,
            },
        );
        self
    }
}

/// One value a crossing hands out, when it hands out several.
///
/// What a JVM builder call receives: a name, the Rust value behind it, and —
/// for a sum — which alternative produces it. There is no access expression
/// and no Kotlin type, because the foreign side does not reach for this value;
/// the Rust side pushes it.
#[derive(Clone)]
pub(crate) struct OutWire {
    /// The builder parameter this value fills, used literally.
    pub(crate) name: String,
    /// The reading whose output conversion encodes it.
    ///
    /// For the tag it is **the sum**, not the `jint` the tag crosses as: what
    /// the tag carries is which alternative is live, and naming the sum is how
    /// an emitter finds the enum to match over.
    pub(crate) out_ty: TypeRef,
    /// Which alternative produces this value, or `None` for one every call
    /// produces — including the tag, which selects between the groups rather
    /// than joining one.
    ///
    /// Composed and checked but not yet read outside the equivalence fixture:
    /// grouping is what turns the list into a `match`, and the emitter that
    /// writes that `match` still reads it off the leaf synthesis.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) group: Option<i32>,
    /// How the Rust side reaches it.
    pub(crate) from: OutFrom,
    /// The steps from the crossed value down to this one.
    ///
    /// **Steps** rather than a chain of idents, because the two kinds mix: a
    /// value form calls an accessor and then reads fields off what it returned,
    /// while a nested `data_class` reads fields all the way down. A step also
    /// says whether it may find nothing, which is what puts every value below
    /// it in doubt.
    ///
    /// Spelled in the plans' own vocabulary rather than a second one. A reach
    /// **is** a path, and inventing a parallel spelling would leave two things
    /// to keep in step for no gain — the mistake the constructing side avoided
    /// by making `Access` and `handle_target` one type.
    ///
    /// Independent of [`Self::from`], because every kind of value has one: a
    /// selector spliced into a value form reaches the sum it selects over, and
    /// a payload reaches the sum whose arm binds it. Empty means the value the
    /// site names, which is the common case.
    pub(crate) reach: Vec<prebindgen_registry::unfold::PathStep>,
    /// Whether this value is the whole crossed object rather than a part of it
    /// — the move-or-clone handle a decomposition delivers beside its fields.
    ///
    /// Never true in a composition: a recipe states what a value is **made of**,
    /// and the value itself is not one of its own parts. A declaration is what
    /// asks for one, through `.field_self()`.
    pub(crate) identity: bool,
    /// Whether the value may be absent, so the wire boxes rather than carrying
    /// a raw primitive.
    ///
    /// Always false in a composition: a decomposition of its own is reached
    /// unconditionally. It is a **splice** that makes one nullable — a value
    /// form reached through an `Option` puts every value under it in doubt —
    /// and the site that splices is what sets it.
    pub(crate) nullable: bool,
}

impl OutWire {
    /// One leaf of an expansion plan, in the recipe's vocabulary.
    ///
    /// The shim that lets the sum emitters speak recipes before every plan is one:
    /// what they read of a leaf is exactly what a wire states, so the switch is
    /// per call site rather than all at once.
    pub(crate) fn from_leaf(leaf: &prebindgen_registry::unfold::UnfoldLeaf) -> Self {
        use prebindgen_registry::unfold::LeafSource;
        Self {
            name: leaf.name.clone(),
            out_ty: leaf.out_ty.clone(),
            group: leaf.group,
            from: match &leaf.source {
                LeafSource::SumTag => OutFrom::Tag,
                LeafSource::VariantField { variant, member } => OutFrom::Payload {
                    variant: Some(variant.clone()),
                    member: member.clone(),
                },
                // Every other leaf is read off the place its reach names.
                _ => OutFrom::Place,
            },
            reach: leaf.path.clone(),
            identity: leaf.identity,
            nullable: leaf.nullable,
        }
    }

    /// A whole plan's leaves in the recipe's vocabulary.
    pub(crate) fn from_leaves(leaves: &[prebindgen_registry::unfold::UnfoldLeaf]) -> Vec<Self> {
        leaves.iter().map(Self::from_leaf).collect()
    }

    /// The steps from the crossed value down to this one, or empty for a value
    /// no reach describes — the selector, or a payload its arm's pattern binds.
    pub(crate) fn reach(&self) -> &[prebindgen_registry::unfold::PathStep] {
        &self.reach
    }

    /// Whether this value is **read off** a place rather than produced by a
    /// call — so it is cloned out of the value that holds it.
    ///
    /// The last step being a field read is the whole question, and it is what
    /// `LeafSource::Reach` meant: a value form's field leaf reads a field off
    /// what the accessor returned, and an accessor leaf ends at the call.
    pub(crate) fn is_field_read(&self) -> bool {
        matches!(
            self.reach.last(),
            Some(prebindgen_registry::unfold::PathStep::Field { .. })
        )
    }

    /// Whether this value is the synthesized selector.
    pub(crate) fn is_tag(&self) -> bool {
        matches!(self.from, OutFrom::Tag)
    }
}

/// Where an outgoing value comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OutFrom {
    /// The synthesized selector, which is not read off the value at all: the
    /// emitter assigns the alternative's number in each arm of its `match`.
    Tag,
    /// Read off the place [`OutWire::reach`] names — a field access, or the
    /// result of the accessor the reach ends in.
    Place,
    /// A payload of one alternative, bound by that arm's pattern.
    Payload {
        /// The alternative's ident as the source enum declares it. Empty until
        /// [`Compile::choice`] names it — an arm's own composition cannot,
        /// because a product hook is not told which alternative it is.
        variant: Option<syn::Ident>,
        /// How the payload is addressed in the arm's pattern.
        member: syn::Member,
    },
}

/// One wire value of a crossing that occupies several.
///
/// `Clone` alone: a wire carries the whole conversion its value crosses
/// through, and a `syn::ItemFn` is neither comparable nor cheap to print.
#[derive(Clone)]
pub(crate) struct Wire {
    /// The JNI type this value crosses as.
    pub(crate) ty: syn::Type,
    /// The Kotlin type of the same value, as an `external fun` writes it.
    pub(crate) kt_ty: String,
    /// The path through the value that reached it — `tag`, `summary.count`.
    ///
    /// **Relative**, because a fragment answers for a crossing and a name
    /// belongs to a site: the same `Holder` is `hTag` in one signature and
    /// `otherTag` in the next, so the parameter it hangs off is the caller's to
    /// prepend.
    pub(crate) path: String,
    /// How Kotlin reaches this value from the object it is destructuring,
    /// relative to that object — `.flat.id`, `.maybe?.id ?: 0L`, ` != null`.
    ///
    /// Relative for the same reason `path` is: the site supplies the base, so
    /// the same wire reads `h.flat.id` in one call and `this.flat.id` in the
    /// next.
    pub(crate) access: Access,
    /// The conversion this value crosses through, or `None` for a presence flag
    /// or a tag — both of which are read on the Rust side and convert nothing.
    ///
    /// The whole conversion rather than its name: the Rust side that rebuilds
    /// the value calls it through its wire-facing function **and** whatever
    /// Rust-side stages follow, and a name says nothing about those.
    pub(crate) entry: Option<ConverterImpl<KotlinMeta>>,
    /// For a nested owned handle: where Kotlin finds the handle **object**, as
    /// against the `Long` this wire carries.
    ///
    /// A nested handle crosses under the same lock-and-consume scaffold as a
    /// top-level one, and that scaffold needs the object, not its pointer. The
    /// wire itself is filled from a local the scaffold binds.
    ///
    /// A walk rather than a string, and for the reason [`Access`] is one: a gate
    /// above has to make every read in it a safe call, and the scaffold locks
    /// what this reaches — so a chain that ignored the gate would lock and
    /// consume the wrong expression.
    pub(crate) handle_target: Option<Vec<Nav>>,
    /// Whether that handle access can be null — the field is optional, or an
    /// optional ancestor gates it.
    pub(crate) handle_nullable: bool,
    /// The Kotlin literal this wire carries while a gate above it is closed,
    /// or `None` for one that rides a JVM `null` or already substitutes its
    /// own.
    ///
    /// Stated where the wire is built rather than derived where the gate is
    /// applied, because only the former knows what the value is: an unsigned
    /// projection substitutes its niche sentinel, and a decoupled pair has
    /// already put its zero in the access.
    pub(crate) absent: Option<String>,
    /// The struct field this value fills or gates, once a product says which.
    ///
    /// `None` until then, and permanently `None` for a gate over a whole value:
    /// such a gate says whether the fields beside it mean anything and fills
    /// none itself. A gate over a decoupled scalar is the other case — it gates
    /// exactly one field, and answers with it.
    pub(crate) field: Option<String>,
    /// Whether this wire gates a whole value rather than one field.
    pub(crate) whole_gate: bool,
}

impl Carrier for JFrag {
    fn yields(&self) -> Yield {
        self.yields.clone()
    }
}

impl JFrag {
    /// A callback conversion built by the legacy fallback.
    ///
    /// Registry recipes handle composable callback products. Irregular layouts
    /// still enter through this compatibility path and are indexed as fragments
    /// so downstream emitters use the same lookup in either case.
    pub(crate) fn by_hand(ty: TypeKey, conv: ConverterImpl<KotlinMeta>) -> Self {
        let rust = crate::jni::chain::JFunction::complete(conv.function.clone());
        Self {
            conv,
            rust,
            layout: Some(JLayout::Leaf),
            choice_arm: None,
            nested_chain: None,
            wires: None,
            out_wires: None,
            composed_only: false,
            yields: Yield {
                ty,
                mode: Mode::Owned,
                validity: Validity::SelfSufficient,
            },
        }
    }

    fn new(at: At<'_>, conv: ConverterImpl<KotlinMeta>) -> Self {
        let validity = validity_of(&conv, at.crossing.direction());
        let rust = crate::jni::chain::JFunction::complete(conv.function.clone());
        Self {
            conv,
            rust,
            choice_arm: None,
            layout: Some(JLayout::Leaf),
            nested_chain: None,
            wires: None,
            out_wires: None,
            composed_only: false,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity,
            },
        }
    }

    pub(crate) fn composed_chain(&self) -> Option<ComposedChain> {
        if !self.composed_only {
            if let Some(layout) = self.layout.clone().filter(JLayout::is_composed) {
                return Some(ComposedChain {
                    ident: self.conv.converter_ident().clone(),
                    layout,
                    rust: self.rust.clone(),
                });
            }
        }
        self.nested_chain.clone()
    }
}

/// How long what this conversion produces stays usable.
///
/// A property of the **conversion**, not of how the crossing was spelled. The
/// two disagree and the spelling is the wrong one to read: a `&T` output over a
/// declared opaque handle clones its referent into a fresh `Box`-handle, and a
/// `&str` output copies into a JVM string, so both are self-sufficient although
/// the crossing is a borrow.
fn validity_of(conv: &ConverterImpl<KotlinMeta>, direction: Direction) -> Validity {
    match direction {
        // Rust to the JVM. Every JNI wire value is a `jlong` the Rust side
        // handed over or a JVM object the JVM now owns; nothing on this wire
        // points into the Rust value it came from.
        Direction::Deconstruct => Validity::SelfSufficient,
        // The JVM to Rust: what the converter's own function hands back. A
        // decode that yields a borrow is valid only for the call, which is
        // exactly right at a parameter and refused at a return.
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
        _ => prebindgen_registry::types_util::result_parts(ty)
            .is_some_and(|(ok, _)| matches!(ok, syn::Type::Reference(_))),
    }
}

/// The adapter, for the length of one crossing's compilation.
pub(crate) struct JCompile<'a, R> {
    pub(crate) decls: &'a Declarations,
    pub(crate) registry: &'a R,
    /// The return as the signature declares it, when that differs from the
    /// crossing being compiled.
    ///
    /// A `Return`-delivery convert crosses the value its decomposition
    /// produced, while the Kotlin surface is classified over what the function
    /// says it returns. `None` when the two are the same.
    pub(crate) declared_return: Option<TypeRef>,
    /// Which site is being planned, when one is.
    ///
    /// `None` while compiling a recipe: a recipe answers for a crossing wherever it
    /// appears, so nothing about a site may reach it. Set only for the one hook
    /// the registry calls per site.
    pub(crate) site: Option<PlanSite>,
}

/// What [`Compile::plan`] needs about a site that the crossing does not say.
pub(crate) enum PlanSite {
    /// One parameter of an exported function.
    Param(ParamSite),
    /// What the function returns, as one value.
    ///
    /// A **decomposed** return is not this: it has no single crossing, which is
    /// what decomposing it means, so it never reaches `Compiler::site` at all.
    Return,
}

/// What planning a parameter needs beyond its crossing.
pub(crate) struct ParamSite {
    /// The parameter ident the wrapper binds this value to.
    pub(crate) ident: syn::Ident,
    /// Whether this leaf is one of a constructor expansion's arguments rather
    /// than a parameter the signature names.
    ///
    /// The one fact a plan needs that the crossing cannot say: a `Vec`
    /// parameter builds through a collection helper where the site is a real
    /// parameter, and crosses as one value where it is an expansion's leaf.
    pub(crate) expanded: bool,
}

/// What this adapter reports when it cannot answer.
///
/// Two kinds, because two readers act on them differently. A **refusal** names
/// a crossing this adapter has no representation for, and the driver turns it
/// into an adapter invariant. A **plan failure** is `fn_plan`'s own typed
/// error, whose readers choose their diagnostic from which failure it is — so
/// it travels whole rather than as the string it would print to.
#[derive(Debug)]
pub(crate) enum JErr {
    /// No JNI representation for this crossing.
    Refused(String),
    /// One site could not be planned.
    ///
    /// Boxed for the reason `PlanError` boxes its own readings: a `Result` is
    /// sized by its largest variant, and the success side of every hook on this
    /// path is the one that always happens.
    Plan(Box<crate::jni::fn_plan::PlanError>),
}

impl std::fmt::Display for JErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JErr::Refused(why) => f.write_str(why),
            JErr::Plan(e) => write!(f, "{e:?}"),
        }
    }
}

fn refuse(at: At<'_>, why: &str) -> JErr {
    JErr::Refused(format!("JniGen: {} ({why})", at.crossing.key()))
}

impl<R: Conversions> JCompile<'_, R> {
    fn wrap(&self, at: At<'_>, why: &str, conv: Option<ConverterImpl<KotlinMeta>>) -> Frag<Self> {
        conv.map(|c| JFrag::new(at, c))
            .ok_or_else(|| refuse(at, why))
    }

    /// The borrow arms, which are neither a terminal nor an arity layer.
    fn borrow(
        &self,
        ty: &TypeRef,
        emit: &prebindgen_registry::Emit,
        into_rust: bool,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let TypeKind::Ref {
            mutable,
            inner: borrowed,
            ..
        } = ty.unwrapped().kind()
        else {
            return None;
        };
        let mutable = *mutable;
        // The target through the accessor: an out-parameter's `MaybeUninit` is
        // the slot a `T` goes in, and it is the `T` that converts.
        let inner = ty.borrow_target().expect("a borrow");
        let produced = Produced::Reading(ty);
        if into_rust {
            // An exclusive borrow crosses only when the borrowed value lives on
            // the Rust side — see `input_borrow`'s own note. `mutable` is read
            // off the `Ref` kind rather than through `is_exclusive_borrow`,
            // which answers `false` for a `&mut MaybeUninit<T>` out-parameter.
            let writes_reach_the_caller = !mutable
                || self
                    .decls
                    .types
                    .get(&borrowed.key())
                    .is_some_and(|cfg| cfg.is_opaque());
            if !writes_reach_the_caller {
                return None;
            }
            if ty.erased_wrappers().is_empty()
                && self
                    .decls
                    .types
                    .get(&inner.key())
                    .is_some_and(|cfg| cfg.is_opaque())
            {
                let mut c = self.decls.opaque_handle_input(inner, emit);
                c.metadata.projection = c.metadata.projection.map(|projection| Projection {
                    owned: false,
                    ..projection
                });
                c.subs = vec![inner.key()];
                return Some(c);
            }
            let mutable = ty.is_exclusive_borrow();
            let mut c = self.decls.input_wrapper_shape(
                WrapperShape::Borrow { mutable },
                &produced,
                inner,
                emit,
            )?;
            c.subs = vec![inner.key()];
            Some(c)
        } else {
            let mutable = ty.is_exclusive_borrow();
            let mut c = self.decls.output_wrapper_shape(
                WrapperShape::Borrow { mutable },
                &produced,
                inner,
                emit,
            )?;
            c.subs = vec![inner.key()];
            Some(c)
        }
    }

    /// Plan an owned opaque input without spelling its Rust type. Product
    /// fields use this ordinary atomic fragment; final emission performs the
    /// only `TypeRef` -> Rust syntax conversion.
    fn planned_owned_handle(&self, at: At<'_>) -> Option<JFrag> {
        if at.crossing.direction() != Direction::Construct || at.crossing.mode() != Mode::Owned {
            return None;
        }
        let source = at.crossing.spelled();
        let cfg = self.decls.types.get(&source.key())?;
        if !cfg.is_opaque() || !source.erased_wrappers().is_empty() {
            return None;
        }
        let TypeKind::Named { id, .. } = source.unwrapped().kind() else {
            return None;
        };
        let source_ident = id.ident()?;
        let module = self.decls.fn_module(self.registry, &source_ident);
        let wire: syn::Type = syn::parse_quote!(jni::sys::jlong);
        let base = crate::jni::chain::planned_name(Direction::Construct, source, &wire);
        let ident = format_ident!("{base}_owned");
        let marker = crate::jni::chain::planned_marker(&ident);
        let rust =
            crate::jni::chain::JFunction::owned_handle(crate::jni::chain::JOwnedHandlePlan {
                ident,
                reachable: std::rc::Rc::new(std::cell::Cell::new(false)),
                source: source.clone(),
                module,
            });
        Some(JFrag {
            conv: ConverterImpl {
                destination: wire,
                function: marker,
                pre_stages: Vec::new(),
                niches: Niches::one(syn::parse_quote!(0i64), syn::parse_quote!(*v == 0)),
                metadata: self.decls.opaque_leaf_meta(source.key()),
                subs: Vec::new(),
            },
            choice_arm: None,
            rust,
            layout: Some(JLayout::Leaf),
            nested_chain: None,
            wires: None,
            out_wires: None,
            composed_only: false,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: Mode::Owned,
                validity: Validity::SelfSufficient,
            },
        })
    }
    fn planned_child(
        direction: Direction,
        part: &Part<'_>,
        frag: &JFrag,
    ) -> crate::jni::chain::JChild {
        let stages = match direction {
            Direction::Construct => frag
                .conv
                .input_stage_order()
                .map(|(_, stage)| stage.function.sig.ident.clone())
                .collect(),
            Direction::Deconstruct => frag
                .conv
                .output_stage_order()
                .map(|(_, stage)| stage.function.sig.ident.clone())
                .collect(),
        };
        match direction {
            Direction::Construct => crate::jni::chain::JChild::input(
                frag.conv.converter_ident().clone(),
                stages,
                if matches!(frag.conv.destination, syn::Type::Ptr(_))
                    || frag.layout.as_ref().is_some_and(JLayout::is_composed)
                {
                    crate::jni::chain::JValueUse::Direct
                } else {
                    crate::jni::chain::JValueUse::SharedRef
                },
            ),
            Direction::Deconstruct => crate::jni::chain::JChild::output(
                frag.conv.converter_ident().clone(),
                stages,
                match part.mode {
                    Mode::Owned => crate::jni::chain::JValueUse::Direct,
                    Mode::Shared | Mode::Exclusive => crate::jni::chain::JValueUse::Cloned,
                },
            ),
        }
    }

    /// Describe a Product as one tuple intermediate. The registry owns the
    /// field walk, tuple construction/destruction, child calls and `?`
    /// propagation; JNI contributes only child converter contracts and ABI
    /// layout metadata.
    fn planned_product(&self, at: At<'_>, parts: Parts<'_, Self>) -> Option<JFrag> {
        if parts.iter().any(|(_, frag)| frag.composed_only) {
            return None;
        }
        let layouts = parts
            .iter()
            .map(|(_, frag)| frag.layout.clone())
            .collect::<Option<Vec<_>>>()?;
        let source = at.crossing.value();
        let wrappers = source.erased_wrappers();
        let direction = at.crossing.direction();
        if wrappers.iter().any(|wrapper| {
            let Some(ops) = super::trait_impl::wrapper_ops(wrapper) else {
                return true;
            };
            match direction {
                Direction::Construct => ops.build.is_none(),
                Direction::Deconstruct => at.crossing.mode() != Mode::Owned || ops.read.is_none(),
            }
        }) {
            return None;
        }
        let TypeKind::Named { id, .. } = source.unwrapped().kind() else {
            return None;
        };
        let source_ident = id.ident()?;
        let source_module = self.decls.fn_module(self.registry, &source_ident);
        let intermediate_parts: Vec<syn::Type> = parts
            .iter()
            .map(|(_, frag)| frag.conv.destination.clone())
            .collect();
        let intermediate: syn::Type = syn::parse_quote!((#(#intermediate_parts,)*));
        let ident =
            crate::jni::chain::planned_name(direction, at.crossing.spelled(), &intermediate);
        let dependencies = parts
            .iter()
            .map(|(_, fragment)| fragment.rust.clone())
            .collect();
        let children = parts
            .iter()
            .map(|(part, frag)| {
                let child = Self::planned_child(direction, part, frag);
                prebindgen_registry::chain::ProductPart {
                    name: format_ident!("{}", part.name),
                    child,
                    mode: part.mode,
                    hold_uninit: false,
                }
            })
            .collect();
        let marker = crate::jni::chain::planned_marker(&ident);
        let rust = crate::jni::chain::JFunction::product(crate::jni::chain::JProductPlan {
            reachable: std::rc::Rc::new(std::cell::Cell::new(false)),
            dependencies,
            ident,
            mode: at.crossing.mode(),
            chain: prebindgen_registry::chain::Product {
                source: source.clone(),
                direction,
                source_policy: crate::jni::chain::JSource {
                    wrappers,
                    module: Some(source_module),
                },
                bridge: prebindgen_registry::chain::TupleProduct {
                    parts: intermediate_parts,
                },
                parts: children,
            },
        });
        Some(JFrag {
            conv: ConverterImpl {
                destination: intermediate,
                function: marker,
                pre_stages: Vec::new(),
                niches: Niches::empty(),
                metadata: KotlinMeta::default(),
                subs: parts.iter().map(|(part, _)| part.ty.key()).collect(),
            },
            rust,
            layout: Some(JLayout::Product(layouts)),
            choice_arm: None,
            nested_chain: None,
            wires: None,
            out_wires: None,
            composed_only: false,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
        })
    }

    /// Retain one sum arm's child chains until `choice` supplies its variant
    /// and tag. The current sealed-class ABI gives each payload field one slot,
    /// so composed child layouts remain a later stage.
    fn planned_choice_arm(&self, at: At<'_>, parts: Parts<'_, Self>) -> Option<JChoiceArmPlan> {
        if parts.iter().any(|(_, frag)| frag.composed_only) {
            return None;
        }
        let layouts = parts
            .iter()
            .map(|(_, frag)| frag.layout.clone())
            .collect::<Option<Vec<_>>>()?;
        if layouts
            .iter()
            .any(|layout| !matches!(layout, JLayout::Leaf))
        {
            return None;
        }
        let direction = at.crossing.direction();
        let intermediate_parts: Vec<syn::Type> = parts
            .iter()
            .map(|(_, frag)| frag.conv.destination.clone())
            .collect();
        let dependencies = parts
            .iter()
            .map(|(_, fragment)| fragment.rust.clone())
            .collect();
        let children = parts
            .iter()
            .map(|(part, frag)| {
                let child = Self::planned_child(direction, part, frag);
                Some(prebindgen_registry::chain::ChoicePart {
                    child,
                    mode: part.mode,
                    hold_uninit: false,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(JChoiceArmPlan {
            dependencies,
            bridge: prebindgen_registry::chain::TupleProduct {
                parts: intermediate_parts,
            },
            parts: children,
            layout: JLayout::Product(layouts),
        })
    }

    /// Finish the arm plans the registry handed to `choice` as one tuple
    /// intermediate. The plan keeps only Flat identities and adapter
    /// representation data; the source enum is spelled by the final renderer.
    fn planned_choice(
        &self,
        at: At<'_>,
        arms: &[(&Alternative, &JFrag)],
        wires: Option<Vec<Wire>>,
        out_wires: Option<Vec<OutWire>>,
    ) -> Option<JFrag> {
        let planned = arms
            .iter()
            .map(|(alternative, fragment)| Some((*alternative, fragment.choice_arm.clone()?)))
            .collect::<Option<Vec<_>>>()?;
        let source = at.crossing.value();
        let TypeKind::Named { id, .. } = source.unwrapped().kind() else {
            return None;
        };
        let source_ident = id.ident()?;
        let source_module = self.decls.fn_module(self.registry, &source_ident);
        let direction = at.crossing.direction();
        let arm_types: Vec<syn::Type> = planned
            .iter()
            .map(|(_, arm)| {
                let parts = &arm.bridge.parts;
                syn::parse_quote!((#(#parts,)*))
            })
            .collect();
        let destination: syn::Type = {
            let arms = &arm_types;
            syn::parse_quote!((jni::sys::jint, #(#arms,)*))
        };
        let tags: Vec<syn::Expr> = planned
            .iter()
            .map(|(alternative, _)| {
                let tag = crate::jni::struct_plan::sum_tag(alternative);
                syn::parse_quote!(#tag)
            })
            .collect();
        let inactive = arm_types.iter().map(Self::inactive_intermediate).collect();
        let source_name = source_ident.to_string();
        let invalid = quote!(<__JniErr as ::core::convert::From<String>>::from(format!(
            "{}: invalid tag {}",
            #source_name,
            __invalid_tag,
        )));
        let choice_arms = planned
            .iter()
            .map(|(alternative, arm)| {
                let tag = crate::jni::struct_plan::sum_tag(alternative);
                prebindgen_registry::chain::ChoiceArm {
                    alternative: (*alternative).clone(),
                    tag: syn::parse_quote!(#tag),
                    bridge: arm.bridge.clone(),
                    parts: arm.parts.clone(),
                }
            })
            .collect();
        let dependencies = planned
            .iter()
            .flat_map(|(_, arm)| arm.dependencies.iter().cloned())
            .collect();
        let layouts = planned.iter().map(|(_, arm)| arm.layout.clone()).collect();
        let ident = crate::jni::chain::planned_name(direction, at.crossing.spelled(), &destination);
        let marker = crate::jni::chain::planned_marker(&ident);
        let rust = crate::jni::chain::JFunction::choice(crate::jni::chain::JChoicePlan {
            ident,
            reachable: std::rc::Rc::new(std::cell::Cell::new(false)),
            dependencies,
            mode: at.crossing.mode(),
            chain: prebindgen_registry::chain::Choice {
                source: source.clone(),
                direction,
                source_policy: crate::jni::chain::JSource {
                    wrappers: source.erased_wrappers(),
                    module: Some(source_module),
                },
                bridge: prebindgen_registry::chain::TupleChoice {
                    tag: syn::parse_quote!(jni::sys::jint),
                    arms: arm_types,
                    tags,
                    inactive,
                    invalid,
                },
                arms: choice_arms,
            },
        });
        Some(JFrag {
            conv: ConverterImpl {
                destination,
                function: marker,
                pre_stages: Vec::new(),
                niches: Niches::empty(),
                metadata: KotlinMeta::default(),
                subs: parts_subs(arms),
            },
            rust,
            layout: Some(JLayout::Choice(layouts)),
            choice_arm: None,
            nested_chain: None,
            wires,
            out_wires,
            composed_only: false,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
        })
    }

    /// One target-safe inactive value for an intermediate. Product
    /// intermediates recurse positionally; terminal JNI wires reuse the same
    /// zero/null policy as wrapper error sentinels.
    fn inactive_intermediate(ty: &syn::Type) -> TokenStream {
        match ty {
            syn::Type::Tuple(tuple) => {
                let values = tuple.elems.iter().map(Self::inactive_intermediate);
                quote!((#(#values,)*))
            }
            wire => crate::jni::emit::sentinel_for_wire(wire),
        }
    }

    /// Plan an Optional over one already-composed child intermediate without
    /// spelling its source type or generating its Rust body.
    ///
    /// The deep input recipe for `Option<&opaque>` remains terminal because it
    /// deliberately yields `OwnedObject<T>` and clones the Java-owned handle.
    /// Choice layouts remain with the legacy parts compiler.
    fn planned_optional(&self, at: At<'_>, inner: &JFrag) -> Option<JFrag> {
        let source = at.crossing.spelled();
        let element = source.optional_inner()?;
        let direction = at.crossing.direction();
        let composed_child =
            !inner.composed_only && inner.layout.as_ref().is_some_and(JLayout::is_composed);
        match direction {
            Direction::Construct
                if inner.out_wires.is_some() || (inner.wires.is_some() && !composed_child) =>
            {
                return None;
            }
            Direction::Deconstruct
                if inner.wires.is_some() || (inner.out_wires.is_some() && !composed_child) =>
            {
                return None;
            }
            _ => {}
        }
        if direction == Direction::Construct
            && (element.borrow_target().is_some() || inner.conv.metadata.is_direct_handle())
        {
            return None;
        }

        let wrappers = source.erased_wrappers();
        if wrappers.iter().any(|wrapper| {
            let Some(ops) = super::trait_impl::wrapper_ops(wrapper) else {
                return true;
            };
            match direction {
                Direction::Construct => ops.build.is_none(),
                Direction::Deconstruct => ops.read.is_none(),
            }
        }) {
            return None;
        }

        let inner_wire = inner.conv.destination.clone();
        let stages = match direction {
            Direction::Construct => inner
                .conv
                .input_stage_order()
                .map(|(_, stage)| stage.function.sig.ident.clone())
                .collect(),
            Direction::Deconstruct => inner
                .conv
                .output_stage_order()
                .map(|(_, stage)| stage.function.sig.ident.clone())
                .collect(),
        };

        let (bridge, child, destination, niches, nullable_kind, layout, input_by_ref) =
            match direction {
                Direction::Construct => {
                    if inner.wires.is_some() {
                        let inner_layout = inner.layout.clone()?;
                        let destination: syn::Type =
                            syn::parse_quote!((jni::sys::jboolean, #inner_wire));
                        (
                            crate::jni::chain::JOptionalBridge::InputGated { child: inner_wire },
                            crate::jni::chain::JChild::input(
                                inner.conv.converter_ident().clone(),
                                stages,
                                crate::jni::chain::JValueUse::Direct,
                            ),
                            destination,
                            Niches::empty(),
                            NullableKind::Boxed,
                            JLayout::Optional(Box::new(inner_layout)),
                            false,
                        )
                    } else if let Some((slot, rest)) = inner.conv.niches.clone().carve() {
                        (
                            crate::jni::chain::JOptionalBridge::InputNiche {
                                wire: inner_wire.clone(),
                                absent: slot.matches,
                            },
                            crate::jni::chain::JChild::input(
                                inner.conv.converter_ident().clone(),
                                stages,
                                crate::jni::chain::JValueUse::Direct,
                            ),
                            inner_wire,
                            rest,
                            NullableKind::Niche,
                            JLayout::Leaf,
                            true,
                        )
                    } else if is_jni_primitive(&inner_wire) {
                        (
                            crate::jni::chain::JOptionalBridge::InputBoxed {
                                inner_wire: inner_wire.clone(),
                                method: jni_unbox_method(&inner_wire),
                                signature: jni_unbox_sig(&inner_wire),
                                getter: format_ident!("{}", jni_unbox_getter(&inner_wire)),
                            },
                            crate::jni::chain::JChild::input(
                                inner.conv.converter_ident().clone(),
                                stages,
                                crate::jni::chain::JValueUse::SharedRef,
                            ),
                            syn::parse_quote!(jni::objects::JObject),
                            Niches::empty(),
                            NullableKind::Boxed,
                            JLayout::Leaf,
                            true,
                        )
                    } else {
                        return None;
                    }
                }
                Direction::Deconstruct => {
                    if inner.out_wires.is_some() {
                        let inner_layout = inner.layout.clone()?;
                        let absent = Self::inactive_intermediate(&inner_wire);
                        let destination: syn::Type =
                            syn::parse_quote!((jni::sys::jboolean, #inner_wire));
                        (
                            crate::jni::chain::JOptionalBridge::OutputGated {
                                child: inner_wire,
                                absent,
                            },
                            crate::jni::chain::JChild::output(
                                inner.conv.converter_ident().clone(),
                                stages,
                                crate::jni::chain::JValueUse::Direct,
                            ),
                            destination,
                            Niches::empty(),
                            NullableKind::Boxed,
                            JLayout::Optional(Box::new(inner_layout)),
                            true,
                        )
                    } else if let Some((slot, rest)) = inner.conv.niches.clone().carve() {
                        (
                            crate::jni::chain::JOptionalBridge::OutputNiche {
                                wire: inner_wire.clone(),
                                absent: slot.value,
                            },
                            crate::jni::chain::JChild::output(
                                inner.conv.converter_ident().clone(),
                                stages,
                                crate::jni::chain::JValueUse::Direct,
                            ),
                            inner_wire,
                            rest,
                            NullableKind::Niche,
                            JLayout::Leaf,
                            true,
                        )
                    } else {
                        let helper = box_helper_for_wire(&inner_wire)?;
                        (
                            crate::jni::chain::JOptionalBridge::OutputBoxed { inner_wire, helper },
                            crate::jni::chain::JChild::output(
                                inner.conv.converter_ident().clone(),
                                stages,
                                crate::jni::chain::JValueUse::Direct,
                            ),
                            syn::parse_quote!(jni::objects::JObject),
                            Niches::empty(),
                            NullableKind::Boxed,
                            JLayout::Leaf,
                            true,
                        )
                    }
                }
            };

        let inherited = inner
            .conv
            .metadata
            .kotlin_name
            .clone()
            .or_else(|| self.decls.override_kotlin_name(&element.key(), None));
        let kotlin_name = self.decls.override_kotlin_name(&source.key(), inherited);
        let projection = inner
            .conv
            .metadata
            .projection
            .clone()
            .map(|projection| Projection {
                strategy: FoldStrategy::Optional(nullable_kind, Box::new(projection.strategy)),
                ..projection
            });
        let kotlin_name = if direction == Direction::Deconstruct && projection.is_none() {
            kotlin_name.map(|name| {
                if name.is_nullable() {
                    name
                } else {
                    name.nullable()
                }
            })
        } else {
            kotlin_name
        };
        let ident = crate::jni::chain::planned_name(direction, source, &destination);
        let out_wires = (direction == Direction::Deconstruct
            && matches!(&layout, JLayout::Optional(_)))
        .then(|| inner.out_wires.clone())
        .flatten();
        let marker = crate::jni::chain::planned_marker(&ident);
        let rust = crate::jni::chain::JFunction::optional(crate::jni::chain::JOptionalPlan {
            ident,
            reachable: std::rc::Rc::new(std::cell::Cell::new(false)),
            dependencies: vec![inner.rust.clone()],
            chain: prebindgen_registry::chain::Optional {
                source: source.clone(),
                direction,
                source_policy: crate::jni::chain::JSource {
                    wrappers,
                    module: None,
                },
                bridge,
                child,
            },
            input_by_ref,
        });
        Some(JFrag {
            conv: ConverterImpl {
                destination,
                function: marker,
                pre_stages: Vec::new(),
                niches,
                metadata: KotlinMeta {
                    projection,
                    ..self.decls.framework_meta(kotlin_name)
                },
                subs: vec![element.key()],
            },
            rust,
            layout: Some(layout),
            choice_arm: None,
            nested_chain: None,
            wires: None,
            out_wires,
            composed_only: false,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode: at.crossing.mode(),
                validity: Validity::SelfSufficient,
            },
        })
    }
}

impl<R: Conversions> Compile for JCompile<'_, R> {
    type Fragment = JFrag;
    /// One site of one exported function, classified.
    type Plan = JPlan;
    type Error = JErr;

    fn atomic(&mut self, cx: &mut Cx<'_>, at: At<'_>) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let emit = cx.emit();
        if let Some(frag) = self.planned_owned_handle(at) {
            return Ok(frag);
        }
        let conv = match at.crossing.direction() {
            Direction::Construct => self
                .decls
                .input_terminal(ty, self.registry, emit)
                .or_else(|| self.borrow(ty, emit, true))
                .or_else(|| self.decls.input_transparent_bridge(ty, self.registry, emit))
                .or_else(|| {
                    // `impl Fn(args)` that nothing else claimed. Callback args
                    // cross in the opposite direction, which is why their
                    // required-ness rides `immediate_edges` rather than `subs`.
                    let TypeKind::Callback { args } = ty.unwrapped().kind() else {
                        return None;
                    };
                    self.decls
                        .dispatch_fn_input(args, self.registry, None, emit)
                }),
            Direction::Deconstruct => self
                .decls
                .output_terminal(ty, self.registry, emit)
                .or_else(|| self.decls.result_shape(ty, self.registry, emit))
                .or_else(|| self.borrow(ty, emit, false))
                .or_else(|| {
                    self.decls
                        .output_transparent_bridge(ty, self.registry, emit)
                }),
        };
        self.wrap(at, "no JNI representation for this type", conv)
    }

    fn optional(&mut self, cx: &mut Cx<'_>, at: At<'_>, inner: &JFrag) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let emit = cx.emit();
        // Declared whole-Optional converters are selected as terminal recipes
        // before structural compilation. If this hook runs, the recipe table
        // has already chosen Optional composition or its legacy structural
        // fallback, so repeating the terminal lookup here was dead code.
        let mut frag = if let Some(planned) = self.planned_optional(at, inner) {
            planned
        } else {
            let legacy = match at.crossing.direction() {
                Direction::Construct => self.decls.input_optional(ty, emit),
                Direction::Deconstruct => self.decls.output_optional(ty, emit),
            };
            self.wrap(at, "no JNI representation for this optional", legacy)?
        };
        // An optional over something that crosses as several values cannot ride
        // a niche in any one of them — which of `(tag, summary)` would carry
        // the absence? So the presence is its own wire, ahead of the rest: the
        // `hMaybePresent` in `(hMaybePresent, hMaybeId)`.
        // A nullable primitive or enum with no niche keeps the
        // allocation-free `(present, value)` pair rather than boxing: the gate
        // is read on the Rust side and the slot carries the raw value. The
        // value crosses through the INNER's conversion, not the optional's —
        // there is no boxed `Option` on this wire to decode.
        if at.crossing.direction() == Direction::Construct && inner.wires.is_none() {
            if let Some(pair) = self.decoupled_optional(at, inner, &frag.conv) {
                frag.layout = None;
                frag.wires = Some(pair);
                return Ok(frag);
            }
        }
        if let (Direction::Construct, Some(inner_wires)) = (at.crossing.direction(), &inner.wires) {
            // A sum reached through the gate needs no navigation added: every
            // one of its wires already reads the value through a `when` or an
            // `as?`, both of which take a null subject. What the gate does add
            // is the `null` arm — and the flag it prepends gates one field
            // rather than a whole value, because the value under it IS the
            // field.
            let sum = is_choice(inner);
            let mut wires = vec![Wire {
                ty: syn::parse_quote!(jni::sys::jboolean),
                kt_ty: "Boolean".to_string(),
                path: "present".to_string(),
                // The gate reads the object itself, not through it.
                access: Access::read(" != null"),
                entry: None,
                handle_target: None,
                handle_nullable: false,
                absent: None,
                field: None,
                whole_gate: !sum,
            }];
            // Everything under the gate is reached through it, and a
            // non-nullable slot still has to hold something when the value is
            // absent — the flag is what tells Rust to ignore it.
            wires.extend(inner_wires.iter().map(gated));
            if !matches!(frag.layout, Some(JLayout::Optional(_))) {
                frag.layout = None;
            }
            frag.wires = Some(wires);
        }
        Ok(frag)
    }

    fn sequence(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        _elements: Mode,
        inner: &JFrag,
    ) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let emit = cx.emit();
        let conv = match at.crossing.direction() {
            Direction::Construct => self
                .decls
                .input_terminal(ty, self.registry, emit)
                .or_else(|| self.decls.input_run(ty, emit))
                .or_else(|| self.borrow(ty, emit, true))
                .or_else(|| self.decls.input_transparent_bridge(ty, self.registry, emit)),
            Direction::Deconstruct => self
                .decls
                .output_terminal(ty, self.registry, emit)
                .or_else(|| self.decls.output_run(ty, emit))
                .or_else(|| self.borrow(ty, emit, false))
                .or_else(|| {
                    self.decls
                        .output_transparent_bridge(ty, self.registry, emit)
                }),
        };
        let mut frag = self.wrap(at, "no JNI representation for this run", conv)?;
        frag.nested_chain = inner.composed_chain();
        Ok(frag)
    }

    fn construct(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _func: &Function,
        _args: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(refuse(at, "JniGen declares no constructor recipes"))
    }

    fn value_form(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        func: &Function,
        parts: Parts<'_, Self>,
    ) -> Frag<Self> {
        if at.crossing.direction() != Direction::Construct {
            return Ok(self.out_value_form(at, func, parts));
        }
        Err(refuse(
            at,
            "JniGen states no constructing value-form recipes yet",
        ))
    }

    fn fields(&mut self, cx: &mut Cx<'_>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self> {
        // A product whose own type is a sum is one **alternative's** payload:
        // the registry composes every arm through this hook and hands the lot
        // to `choice`. Which alternative that is stays `choice`'s to fill in,
        // being the only hook told — so both directions leave a hole here.
        let sum = self.is_sum(cx, at);
        if sum {
            let direction = at.crossing.direction();
            let mut frag = match direction {
                Direction::Construct => self.arm(at, parts),
                Direction::Deconstruct => self.out_arm(at, parts),
            };
            let represented = match direction {
                Direction::Construct => frag.wires.is_some(),
                Direction::Deconstruct => frag.out_wires.is_some(),
            };
            if represented {
                frag.choice_arm = self.planned_choice_arm(at, parts);
            }
            return Ok(frag);
        }
        if at.crossing.direction() == Direction::Deconstruct {
            return Ok(self.out_product(at, parts));
        }
        // A `data_class` crosses as its fields, and a field that is itself one
        // contributes its own several — which is the recursion, stated once
        // here rather than walked by hand.
        let mut wires: Vec<Wire> = Vec::new();
        for (part, frag) in parts {
            match &frag.wires {
                Some(inner) => wires.extend(inner.iter().map(|w| {
                    Wire {
                        ty: w.ty.clone(),
                        kt_ty: w.kt_ty.clone(),
                        path: format!("{}.{}", part.name, w.path),
                        // The field this part reads, then however the part's own
                        // wire reaches on from there.
                        access: w.access.clone().under(&field_kt(part)),
                        entry: w.entry.clone(),
                        handle_target: w.handle_target.as_ref().map(|t| {
                            std::iter::once(Nav {
                                field: field_kt(part),
                                gated: false,
                            })
                            .chain(t.iter().cloned())
                            .collect()
                        }),
                        handle_nullable: w.handle_nullable,
                        absent: w.absent.clone(),
                        // A part's wires name their own fields once they are
                        // inside a product; what a decoupled optional left
                        // open, the part it belongs to closes.
                        field: w
                            .field
                            .clone()
                            .or_else(|| (!w.whole_gate).then(|| part.name.clone())),
                        whole_gate: w.whole_gate,
                    }
                })),
                // A part whose conversion projects a handle crosses as that
                // handle's `Long`, and Kotlin reaches the object it has to lock
                // through the same access.
                None if is_handle(frag) => wires.push(Wire {
                    ty: syn::parse_quote!(jni::sys::jlong),
                    kt_ty: "Long".to_string(),
                    path: part.name.clone(),
                    access: Access::read("").under(&field_kt(part)),
                    entry: None,
                    handle_target: Some(vec![Nav {
                        field: field_kt(part),
                        gated: false,
                    }]),
                    handle_nullable: part.ty.optional_inner().is_some(),
                    absent: None,
                    field: Some(part.name.clone()),
                    whole_gate: false,
                }),
                None => wires.push(self.field_wire(part, frag)),
            }
        }
        let mut frag = self.planned_product(at, parts).unwrap_or_else(|| {
            let mut marker = JFrag::new(
                at,
                self.parts_marker(parts.iter().map(|(part, _)| part.ty.key()).collect()),
            );
            marker.composed_only = true;
            marker
        });
        frag.wires = Some(wires);
        Ok(frag)
    }

    fn choice(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        arms: &[(&Alternative, &JFrag)],
    ) -> Frag<Self> {
        match at.crossing.direction() {
            Direction::Construct => match self.selected(at, arms) {
                Some(wires) => {
                    if let Some(planned) = self.planned_choice(at, arms, Some(wires.clone()), None)
                    {
                        return Ok(planned);
                    }
                    let mut legacy = JFrag::new(at, self.parts_marker(parts_subs(arms)));
                    legacy.wires = Some(wires);
                    legacy.composed_only = true;
                    Ok(legacy)
                }
                // A payload this adapter has no slot for — a nested object or
                // handle — keeps the explicit whole-value sealed-class decoder.
                None => {
                    let conv = self
                        .decls
                        .in_frag(at.crossing.spelled())
                        .ok_or_else(|| refuse(at, "no JNI representation for this sum"))?;
                    Ok(JFrag::new(at, (*conv).clone()))
                }
            },
            Direction::Deconstruct => {
                let legacy = self.selected_out(at, arms)?;
                let out_wires = legacy.out_wires.clone();
                Ok(self
                    .planned_choice(at, arms, None, out_wires)
                    .unwrap_or(legacy))
            }
        }
    }

    fn callback(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        arg_fragments: &[&JFrag],
        _result: Option<&JFrag>,
    ) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let TypeKind::Callback { args } = ty.unwrapped().kind() else {
            return Err(refuse(at, "a callback recipe over a type that is not one"));
        };
        let conv =
            self.decls
                .dispatch_fn_input(args, self.registry, Some(arg_fragments), cx.emit());
        self.wrap(at, "undeclared callback signature", conv)
    }

    /// One site: which of the seven wire layouts this parameter takes, and the
    /// names the three coordinated emitters give it.
    ///
    /// The only hook the registry calls once per **site**; every hook above it
    /// answers once per crossing, however many sites reuse the answer. What
    /// makes this one per-site is not the type — that is the fragment's — but
    /// what wraps it: a `Vec` parameter builds through a collection helper only
    /// where the site is a real parameter and not a constructor expansion's
    /// leaf, and the diagnostic for an unresolved leaf names the parameter that
    /// expanded.
    fn plan(&mut self, _cx: &mut Cx<'_>, bound: &Bound, root: &JFrag) -> Result<JPlan, JErr> {
        use crate::jni::fn_plan::{plan_error, InputKind, PlanLeaf};
        let site = self
            .site
            .as_ref()
            .ok_or_else(|| JErr::Refused("JniGen: a site compiled with no site context".into()))?;
        if matches!(root.layout, Some(JLayout::Leaf)) {
            root.rust.mark_reachable();
        }
        let site = match site {
            PlanSite::Return => {
                // A fragment that occupies several wires IS the decomposed
                // return: the site asked for the `parts` recipe and got what that
                // recipe states. Nothing else distinguishes the two cases, and
                // nothing needs to.
                return Ok(match &root.out_wires {
                    Some(wires) => JPlan::Decomposed(wires.clone()),
                    None => JPlan::Return(Box::new(self.return_plan(bound, root))),
                });
            }
            PlanSite::Param(site) => site,
        };
        let (ident, expanded) = (&site.ident, site.expanded);
        let reading = bound.crossing.spelled();
        let registry = self.registry;
        let ext = self.decls;

        // Every question below is the model's — the local spelling this
        // function opened with has no users left.
        let optional = reading.optional_inner().is_some();
        // The enum probe off the reading: the layers it peels are the model's
        // own (`&`, `Option`), so there is nothing to re-spell and nothing to
        // look up.
        let as_enum_value = ext.is_kotlin_enum_reading(reading);
        let kt_name = crate::jni::kt_param_name(&ident.to_string());

        // The site's own conversion, which the registry built before calling
        // this. What used to be a lookup by type is the fragment it is handed.
        let entry = &root.conv;

        let flat_plan = crate::jni::emit::build_flat_input_plan(ext, registry, ident, reading)
            .map_err(|e| plan_error(crate::jni::fn_plan::PlanError::UnflattenableDataClass(e)))?;
        let kind = if let Some(v) = (!expanded)
            .then(|| crate::jni::emit::vec_build_elem(ext, registry, reading))
            .flatten()
        {
            InputKind::VecBuild {
                elem: v.elem,
                by_ref: v.by_ref,
                elem_wrappers: v.elem_wrappers,
            }
        } else if let Some(sp) =
            crate::jni::emit::build_option_scalar_input_plan(ext, ident, reading)
        {
            InputKind::OptionScalar(sp)
        } else if let Some(plan) = flat_plan {
            InputKind::FlattenStruct(plan)
        } else {
            match entry.metadata.projection.as_ref().map(|p| p.kind.clone()) {
                Some(ProjectionKind::Handle) => InputKind::Handle {
                    direct: entry.metadata.is_direct_handle(),
                },
                Some(ProjectionKind::Unsigned64) => InputKind::Unsigned64 {
                    niche: entry.metadata.projection.as_ref().and_then(|p| {
                        reading
                            .optional_inner()
                            .is_some()
                            .then(|| p.niche_sentinels.first().cloned())
                            .flatten()
                    }),
                },
                None => InputKind::Plain,
            }
        };

        // Typed surface: handle/value projections show their Kotlin class (from
        // the projection's leaf key); everything else the conversion's resolved
        // name.
        let kt_meta = entry.metadata.kotlin_name.clone();
        let kt_public = match entry.metadata.projection.as_ref() {
            Some(p) => crate::jni::projection_leaf_kt(ext, p),
            None => kt_meta.clone(),
        };

        Ok(JPlan::Param(Box::new(PlanLeaf {
            reading: reading.clone(),
            kt_name,
            kt_public,
            kt_meta,
            optional,
            as_enum_value,
            kind,
        })))
    }
}

/// What an emitter asks instead of the converter table.
///
/// Each hands back the fragment's `ConverterImpl`, which is what a table entry
/// was, so a call site reads the same fields it always did — from this
/// adapter's own answer rather than from the shared index. Cloned rather than
/// borrowed because [`Declarations::compiled`] is read while it is still being
/// filled; a conversion is built once per crossing, so the clone is not on any
/// hot path.
///
/// `None` means nothing has compiled that crossing. During compilation that is
/// the deferral the resolver already understands — it retries — and after it,
/// the only crossing without a fragment is a callback, which
/// `JniGen::compile_crossing` answers without the compiler.
impl crate::jni::Declarations {
    /// The conversion for `ty` in the given direction, from the fragments
    /// compiled so far.
    pub(crate) fn frag(&self, ty: &TypeRef, direction: Direction) -> Option<Conv> {
        Some(Conv(self.compiled.borrow().fragment(&ty.key(), direction)?))
    }

    pub(crate) fn in_frag(&self, ty: &TypeRef) -> Option<Conv> {
        self.frag(ty, Direction::Construct)
    }

    pub(crate) fn out_frag(&self, ty: &TypeRef) -> Option<Conv> {
        self.frag(ty, Direction::Deconstruct)
    }

    /// Every wire the Kotlin → Rust crossing of `ty` occupies, or `None` when
    /// it occupies the single one its conversion names.
    ///
    /// A declared class states its composition under the `parts` recipe; an
    /// optional over one has no recipe of its own and composes on the recipe the
    /// registry derived, which is that crossing's default.
    pub(crate) fn wires_of(&self, ty: &TypeRef) -> Option<Vec<Wire>> {
        let key = ty.key();
        let crossing = prebindgen_registry::recipe::Crossing::new(ty.clone(), Direction::Construct);
        let parts = self
            .recipe_table()
            .key_of(&crossing.key(), &crate::jni::recipes::parts())
            .cloned();
        let compiled = self.compiled.borrow();
        parts
            .and_then(|parts| compiled.recipe_fragment(&key, &parts))
            .or_else(|| compiled.fragment(&key, Direction::Construct))?
            .wires
            .clone()
    }

    /// The exact registry-composed converter for a crossing.
    ///
    /// Products live in their explicit `parts` row. Optional composition is
    /// the crossing's default row, so it is considered only when no Product row
    /// exists. Leaf terminal converters cannot escape because a fragment must
    /// carry a composed layout before it can return a chain.
    pub(crate) fn composed_chain(
        &self,
        ty: &TypeRef,
        direction: Direction,
    ) -> Option<ComposedChain> {
        let crossing = prebindgen_registry::recipe::Crossing::new(ty.clone(), direction);
        let row = self
            .recipe_table()
            .key_of(&crossing.key(), &crate::jni::recipes::parts())
            .cloned();
        let compiled = self.compiled.borrow();
        let frag = match row {
            Some(row) => compiled.recipe_fragment(&ty.key(), &row)?,
            None => compiled.fragment(&ty.key(), direction)?,
        };
        frag.composed_chain()
    }
}

/// Callable registry-composed shape and its ABI-leaf layout.
#[derive(Clone)]
pub(crate) struct ComposedChain {
    pub(crate) ident: syn::Ident,
    rust: crate::jni::chain::JFunction,
    pub(crate) layout: JLayout,
}

impl ComposedChain {
    pub(crate) fn activate(&self) {
        self.rust.mark_reachable();
    }
}

/// A fragment's conversion, read without copying it.
///
/// The store is read while compilation is still writing to it, so a caller
/// cannot hold a borrow into it; sharing the fragment's `Rc` and reaching the
/// conversion through it costs a refcount instead of a whole `syn::ItemFn` per
/// lookup.
pub(crate) struct Conv(std::rc::Rc<JFrag>);

impl Conv {
    pub(crate) fn activate(&self) {
        self.0.rust.mark_reachable();
    }
}

impl std::ops::Deref for Conv {
    type Target = ConverterImpl<KotlinMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0.conv
    }
}

/// Facts a wire states about itself, which the emitters read off the recipe.
impl Wire {
    /// The wire-facing function this value crosses through.
    pub(crate) fn conv(&self) -> Option<&syn::Ident> {
        self.entry.as_ref().map(|e| &e.function.sig.ident)
    }

    /// Whether the conversion carries Rust-side stages beyond its wire-facing
    /// function — a `convert!` with a semantic step, say `jlong -> u64 ->
    /// Duration`.
    ///
    /// Read where a caller may only call the wire-facing function and would
    /// otherwise bind the representation where the value is wanted: the `Vec`
    /// build helper declines such an element rather than emit a call that does
    /// not compile.
    pub(crate) fn staged(&self) -> bool {
        self.entry
            .as_ref()
            .is_some_and(|e| !e.pre_stages.is_empty())
    }

    /// Whether this value is a gate rather than something that crosses.
    ///
    /// A presence flag is the one wire with neither a conversion nor a handle
    /// behind it that is still an ordinary read. A tag has neither either, and
    /// is told apart by being read through a `when`.
    pub(crate) fn is_present_flag(&self) -> bool {
        self.entry.is_none()
            && self.handle_target.is_none()
            && matches!(self.access, Access::Read { .. })
    }

    /// The struct field this value fills or gates.
    pub(crate) fn field(&self) -> Option<&str> {
        self.field.as_deref()
    }
}

impl<R: Conversions> JCompile<'_, R> {
    /// The conversion a composed-only recipe carries: none.
    ///
    /// The `parts` and arm recipes state what a value is made of and nothing
    /// about how it is rebuilt, so there is no function to name. `subs` is
    /// still real — it is what the registry walks for reachability.
    /// `prebindgen-c` does the same for a union arm, and for the same reason.
    fn parts_marker(&self, subs: Vec<TypeKey>) -> ConverterImpl<KotlinMeta> {
        ConverterImpl {
            destination: syn::parse_quote!(()),
            function: syn::parse_quote!(
                #[allow(dead_code)]
                fn __jni_parts() {}
            ),
            pre_stages: Vec::new(),
            niches: Niches::empty(),
            metadata: KotlinMeta::default(),
            subs,
        }
    }

    /// Whether the crossing being composed is a data-carrying enum, which is
    /// what makes a product hook an **arm** rather than a struct.
    fn is_sum(&self, cx: &Cx<'_>, at: At<'_>) -> bool {
        let TypeKind::Named { id, .. } = at.crossing.value().unwrapped().kind() else {
            return false;
        };
        id.ident().is_some_and(|ident| {
            matches!(
                cx.model().declared_type(&ident),
                Some(prebindgen_registry::flat::Type::Variant(_))
            )
        })
    }

    /// One alternative's payload, as the slots it crosses in.
    ///
    /// A fragment with **no** wires is this adapter declining: a payload it has
    /// no slot for — one that is itself several values, an opaque handle, or a
    /// wire that is neither a JNI primitive nor a string — keeps the whole sum
    /// object-shaped. See [`Compile::choice`]'s `None` arm for what that costs.
    fn arm(&self, at: At<'_>, parts: Parts<'_, Self>) -> JFrag {
        let mut wires = Vec::new();
        for (part, frag) in parts {
            let Some(wire) = self.slot(part, frag) else {
                return JFrag::new(at, self.parts_marker(Vec::new()));
            };
            wires.push(wire);
        }
        let mut frag = JFrag::new(
            at,
            self.parts_marker(parts.iter().map(|(p, _)| p.ty.key()).collect()),
        );
        frag.wires = Some(wires);
        frag.composed_only = true;
        frag
    }

    /// One field that crosses as a single value of its own conversion.
    ///
    /// Three reads the value itself asks for, each because the Kotlin property
    /// holds something other than the wire: an `enum_class` property is the
    /// enum object and the wire is its discriminant, an unsigned projection's
    /// property is a `ULong` and the wire is the `Long` under it, and an
    /// optional whose wire is a JVM object rides a `null` rather than a
    /// literal.
    fn field_wire(&self, part: &Part<'_>, frag: &JFrag) -> Wire {
        let optional = part.ty.optional_inner().is_some();
        let mut walk = vec![Nav {
            field: field_kt(part),
            gated: false,
        }];
        let mut kt_ty = crate::jni::emit::wire_kotlin_type(&frag.conv);
        let projection = frag.conv.metadata.projection.as_ref();
        let mut absent = None;
        let mut tail = String::new();
        if projection.map(|p| &p.kind) == Some(&ProjectionKind::Unsigned64) {
            walk.push(Nav {
                field: "toLong()".to_string(),
                gated: optional,
            });
            // An unsigned projection carves its absent value out of the
            // representation's own range, so what stands in for absence is that
            // sentinel rather than the wire's zero. The field's own optional
            // reaches it here; an ancestor's gate reaches it through `absent`.
            let sentinel = projection
                .and_then(|p| p.niche_sentinels.first().cloned())
                .unwrap_or_else(|| "0L".to_string());
            match optional {
                true => tail = format!(" ?: {sentinel}"),
                false => absent = Some(sentinel),
            }
        } else if self.decls.is_kotlin_enum_reading(&part.ty) {
            walk.push(Nav {
                field: "value".to_string(),
                gated: optional,
            });
            if optional {
                tail = " ?: 0".to_string();
            }
        }
        if optional && crate::jni::emit::is_jobject_shaped_wire(&frag.conv.destination) {
            if !kt_ty.ends_with('?') {
                kt_ty.push('?');
            }
        } else if absent.is_none() {
            absent = crate::jni::wire_access::jni_field_access(&frag.conv.destination)
                .and_then(|(sig, _, _)| crate::jni::emit::kt_leaf_default(sig, false));
        }
        Wire {
            ty: frag.conv.destination.clone(),
            kt_ty,
            path: part.name.clone(),
            access: Access::Read { walk, tail },
            entry: Some(frag.conv.clone()),
            handle_target: None,
            handle_nullable: false,
            absent,
            field: Some(part.name.clone()),
            whole_gate: false,
        }
    }

    /// One alternative's payloads, as the values it hands out.
    ///
    /// Every payload contributes, unconditionally: a sum's alternatives are
    /// laid side by side on the wire and the tag says which group is live, so
    /// there is nothing for an arm to decline. That is the asymmetry with the
    /// constructing side, where a payload with no slot form leaves the whole
    /// sum object-shaped — going the other way the Rust side produces the
    /// value and its own output conversion is what encodes it.
    fn out_arm(&self, at: At<'_>, parts: Parts<'_, Self>) -> JFrag {
        let wires = parts
            .iter()
            .filter_map(|(part, _)| {
                Some(OutWire {
                    // Named by `choice`, which knows the alternative the name
                    // is built from.
                    name: crate::jni::struct_plan::sum_field_prop_name(&part_member(part)?),
                    out_ty: part.ty.clone(),
                    group: None,
                    from: OutFrom::Payload {
                        variant: None,
                        member: part_member(part)?,
                    },
                    nullable: false,
                    identity: false,
                    reach: Vec::new(),
                })
            })
            .collect();
        let mut frag = JFrag::new(
            at,
            self.parts_marker(parts.iter().map(|(p, _)| p.ty.key()).collect()),
        );
        frag.out_wires = Some(wires);
        frag.composed_only = true;
        frag
    }

    /// A return that crosses as one value.
    ///
    /// The site's own fragment is the conversion — the registry built it before
    /// calling this, which is the whole point of the hook. What the plan adds
    /// is the Kotlin surface, which is classified over the **declared** return
    /// rather than over the crossing: an error peel rides the conversion's
    /// `value_rust_type`, so the full `Result<T, E>` is what the surface reads.
    fn return_plan(&self, bound: &Bound, root: &JFrag) -> crate::jni::fn_plan::ValueOutputPlan {
        let declared = self
            .declared_return
            .as_ref()
            .unwrap_or_else(|| bound.crossing.spelled());
        let (surface, enums) = crate::jni::fn_plan::ReturnSurface::classify(self.decls, declared);
        crate::jni::fn_plan::ValueOutputPlan {
            is_convert: self.declared_return.is_some(),
            target_ty: bound.crossing.spelled().clone(),
            wire_ty: root.conv.destination.clone(),
            surface,
            is_enum: enums.is_enum,
            is_option_enum: enums.is_option_enum,
        }
    }

    /// Describe a `data_class` as one tuple intermediate while retaining the
    /// independently flattened ABI leaves used by Kotlin signatures.
    fn out_product(&self, at: At<'_>, parts: Parts<'_, Self>) -> JFrag {
        let Some(wires) = self
            .decls
            .struct_out_wires(self.registry, at.crossing.value())
        else {
            return JFrag::new(at, self.parts_marker(Vec::new()));
        };
        let mut frag = self.planned_product(at, parts).unwrap_or_else(|| {
            let mut marker = JFrag::new(
                at,
                self.parts_marker(parts.iter().map(|(part, _)| part.ty.key()).collect()),
            );
            marker.composed_only = true;
            marker
        });
        frag.out_wires = Some(wires);
        frag
    }

    /// The values a **value form** hands out: call the accessor once, then read
    /// the fields of what it returned.
    ///
    /// Every field is one value here, where a by-value `data_class` inlines a
    /// nested one — the difference is the declaration, not the type. A value
    /// form states its own field list, and what it does not state, it does not
    /// decompose.
    ///
    /// The names are the declaration's: a `.name(..)` rename carries through,
    /// which is why the record list is asked for again rather than the Kotlin
    /// property being derived a second time here.
    fn out_value_form(&self, at: At<'_>, func: &Function, parts: Parts<'_, Self>) -> JFrag {
        let declined = JFrag::new(at, self.parts_marker(Vec::new()));
        let Some(names) = self
            .decls
            .value_form_names(self.registry, at.crossing.value())
        else {
            return declined;
        };
        let mut wires = Vec::new();
        for (part, _) in parts {
            let Some(field) = part_field(part) else {
                return declined;
            };
            let Some(ident) = field.name.clone() else {
                return declined;
            };
            let Some(name) = names.get(&ident.to_string()).cloned() else {
                return declined;
            };
            wires.push(OutWire {
                name,
                out_ty: part.ty.clone(),
                group: None,
                // The chain starts at the accessor's result, which the emitter
                // binds once. The call itself is the site's to make, so what a
                // wire states is the field read off it.
                from: OutFrom::Place,
                reach: vec![field_step(&ident)],
                nullable: false,
                identity: false,
            });
        }
        let mut frag = JFrag::new(
            at,
            self.parts_marker(
                std::iter::once(TypeKey::from_ident(&func.name))
                    .chain(parts.iter().map(|(p, _)| p.ty.key()))
                    .collect(),
            ),
        );
        frag.out_wires = Some(wires);
        frag.composed_only = true;
        frag
    }

    /// The tag, then every alternative's payloads.
    ///
    /// The deconstructing shape of a `sealed_class`: one `jint` naming which
    /// alternative is live, and one group of values per alternative laid
    /// beside the others. Exactly one group is live per value and the rest
    /// carry their wire defaults, which is what makes the whole thing one
    /// `match` on the Rust side rather than N conditionals.
    fn selected_out(&self, at: At<'_>, arms: &[(&Alternative, &JFrag)]) -> Frag<Self> {
        let TypeKind::Named { id, .. } = at.crossing.value().unwrapped().kind() else {
            return Err(refuse(at, "a choice recipe over a type that is not named"));
        };
        let ident = id
            .ident()
            .ok_or_else(|| refuse(at, "a choice recipe over a type that is not one identifier"))?;
        // The composition is the declaration's and the model's, so the same
        // answer serves the leaf synthesis that runs before `resolve`. The arm
        // fragments the driver built are not read at all — a sum hands its
        // payloads out through their own conversions, and which those are is
        // the emitter's question rather than this recipe's.
        let wires = self
            .decls
            .sum_out_wires(self.registry, &ident, at.crossing.value())
            .ok_or_else(|| refuse(at, "a choice recipe over an undeclared sum"))?;
        let mut frag = JFrag::new(at, self.parts_marker(parts_subs(arms)));
        frag.out_wires = Some(wires);
        frag.composed_only = true;
        Ok(frag)
    }

    /// One payload of one alternative, or `None` if it has no slot form.
    fn slot(&self, part: &Part<'_>, frag: &JFrag) -> Option<Wire> {
        if frag.wires.is_some() || frag.conv.metadata.projection.is_some() {
            return None;
        }
        let prim = crate::jni::JniPrim::from_wire(&frag.conv.destination);
        let string_like = matches!(&frag.conv.destination, syn::Type::Path(tp)
            if tp.path.segments.last().is_some_and(|s| s.ident == "JString"));
        if prim.is_none() && !string_like {
            return None;
        }
        let prop = crate::jni::struct_plan::sum_field_prop_name(&part_member(part)?);
        // An `enum_class` payload is a Kotlin enum object whose wire is the
        // `jint` discriminant, so the slot reads `.value` — without it the wire
        // would carry a `Priority?` where it wants an `Int`.
        let read = if self.decls.is_kotlin_enum_reading(&part.ty) {
            format!("{prop}?.value")
        } else {
            prop.clone()
        };
        let mut kt_ty = crate::jni::emit::wire_kotlin_type(&frag.conv);
        if prim.is_none() && !kt_ty.ends_with('?') {
            kt_ty.push('?');
        }
        Some(Wire {
            ty: frag.conv.destination.clone(),
            kt_ty,
            path: prop,
            access: Access::Slot {
                walk: Vec::new(),
                // Named by `choice`, the only hook told which alternative this
                // payload belongs to.
                class: String::new(),
                read,
                zero: prim.map(|p| p.kotlin_zero().to_string()),
            },
            entry: Some(frag.conv.clone()),
            handle_target: None,
            handle_nullable: false,
            absent: None,
            field: None,
            whole_gate: false,
        })
    }

    /// The tag, then every alternative's slots — or `None` if any alternative
    /// declined, since a sum crosses whole or not at all.
    fn selected(&self, at: At<'_>, arms: &[(&Alternative, &JFrag)]) -> Option<Vec<Wire>> {
        let TypeKind::Named { id, .. } = at.crossing.value().unwrapped().kind() else {
            return None;
        };
        let cfg = self.decls.types.get(&TypeKey::from_ident(&id.ident()?))?;
        let sum_cfg = cfg.sum()?;
        let iface = cfg.name_spec.as_ref().map(|s| self.decls.fqn_of(s))?;

        let classes: Vec<String> = arms
            .iter()
            .map(|(alt, _)| {
                format!(
                    "{iface}.{}",
                    self.decls.sum_variant_class_name(sum_cfg, &alt.name)
                )
            })
            .collect();
        let mut wires = vec![Wire {
            ty: syn::parse_quote!(jni::sys::jint),
            kt_ty: "Int".to_string(),
            path: "_tag".to_string(),
            access: Access::Select {
                walk: Vec::new(),
                arms: arms
                    .iter()
                    .zip(&classes)
                    .map(|((alt, _), class)| {
                        format!("is {class} -> {}", crate::jni::struct_plan::sum_tag(alt))
                    })
                    .collect(),
                nullable: false,
            },
            entry: None,
            handle_target: None,
            handle_nullable: false,
            absent: None,
            field: None,
            whole_gate: false,
        }];
        for ((_, frag), class) in arms.iter().zip(&classes) {
            let short = class.rsplit('.').next().unwrap_or(class);
            for w in frag.wires.as_ref()? {
                let Access::Slot { read, zero, .. } = &w.access else {
                    return None;
                };
                wires.push(Wire {
                    path: crate::jni::struct_plan::sum_slot_fragment(short, &w.path),
                    access: Access::Slot {
                        walk: Vec::new(),
                        class: class.clone(),
                        read: read.clone(),
                        zero: zero.clone(),
                    },
                    ..w.clone()
                });
            }
        }
        Some(wires)
    }

    /// The `(present, value)` pair a nullable primitive or enum crosses as, or
    /// `None` if this optional boxes instead.
    ///
    /// The conditions are the ones the walk applies: the inner is not a borrow,
    /// its wire is a JNI primitive, and it has nothing that would already carry
    /// the absence — no carved niche, no projection, no Rust-side stages.
    fn decoupled_optional(
        &self,
        at: At<'_>,
        inner: &JFrag,
        outer: &ConverterImpl<KotlinMeta>,
    ) -> Option<Vec<Wire>> {
        let inner_reading = at.crossing.value().optional_inner()?;
        if inner_reading.borrow_target().is_some() {
            return None;
        }
        let c = &inner.conv;
        let prim = crate::jni::JniPrim::from_wire(&c.destination)?;
        if c.niches.clone().carve().is_some() || !c.pre_stages.is_empty() {
            return None;
        }
        // An unsigned representation is the one projection that still takes the
        // pair. Its Kotlin property is a `ULong?`, which boxes, and its wire is
        // the `Long` under it — so the gate is worth the same here as it is over
        // a signed primitive. Only where the optional has no primitive wire of
        // its own: a bounded representation whose range leaves a niche already
        // crosses as one value and keeps doing so.
        let unsigned = match c.metadata.projection.as_ref().map(|p| &p.kind) {
            None => false,
            Some(ProjectionKind::Unsigned64)
                if crate::jni::JniPrim::from_wire(&outer.destination).is_none() =>
            {
                true
            }
            Some(_) => return None,
        };
        // A Kotlin enum's slot is its `value`, reached through the same gate —
        // what this pair was decoupled from is optional, so that read is a safe
        // call however the wire above it is composed.
        let step = if unsigned {
            // The `ULong` property's own representation, which is the wire.
            Some("toLong()")
        } else if self.decls.is_kotlin_enum_reading(inner_reading) {
            Some("value")
        } else {
            None
        };
        let value_access = Access::Read {
            walk: step
                .map(|field| {
                    vec![Nav {
                        field: field.to_string(),
                        gated: true,
                    }]
                })
                .unwrap_or_default(),
            tail: format!(" ?: {}", prim.kotlin_zero()),
        };
        Some(vec![
            Wire {
                ty: syn::parse_quote!(jni::sys::jboolean),
                kt_ty: "Boolean".to_string(),
                path: "present".to_string(),
                access: Access::read(" != null"),
                entry: None,
                handle_target: None,
                handle_nullable: false,
                absent: None,
                field: None,
                whole_gate: false,
            },
            Wire {
                ty: c.destination.clone(),
                kt_ty: crate::jni::emit::wire_kotlin_type(c),
                path: "value".to_string(),
                access: value_access,
                entry: Some(c.clone()),
                handle_target: None,
                handle_nullable: false,
                absent: None,
                field: None,
                whole_gate: false,
            },
        ])
    }
}

impl Declarations {
    /// The values a `data_class` hands out: its fields, and a field that is
    /// itself one contributes its own under the parent's name and chain.
    ///
    /// Model and declaration only, like [`Self::sum_out_wires`] — so the same
    /// answer serves the leaf synthesis that runs before `resolve` and the recipe
    /// that composes after it.
    ///
    /// `None` is this adapter declining, and it declines for the **whole**
    /// value rather than per field. A handle, an `enum_class`, a sum, or a
    /// `data_class` behind an `Option` or a `Vec` is delivered with a transform
    /// the decoupled form does not carry, and one such field sends the whole
    /// object down the whole-value `fromParts` path — so a recipe that decomposed
    /// the rest of it would describe a shape nothing emits.
    pub(crate) fn struct_out_wires(
        &self,
        registry: &impl Conversions,
        ty: &TypeRef,
    ) -> Option<Vec<OutWire>> {
        let TypeKind::Named { id, .. } = ty.unwrapped().kind() else {
            return None;
        };
        self.struct_out_wires_at(registry, &id.ident()?, &[], "", 0)
    }

    /// One level of [`Self::struct_out_wires`]'s walk. `path` and `name_prefix`
    /// accumulate through inlined nested classes, whose names join with the
    /// reserved `__` separator.
    /// [`Self::struct_out_wires`] by the struct's own name, for a caller that
    /// has the element rather than a reading of it.
    pub(crate) fn struct_out_wires_of(
        &self,
        registry: &impl Conversions,
        ident: &syn::Ident,
    ) -> Option<Vec<OutWire>> {
        self.struct_out_wires_at(registry, ident, &[], "", 0)
    }

    fn struct_out_wires_at(
        &self,
        registry: &impl Conversions,
        ident: &syn::Ident,
        path: &[syn::Ident],
        name_prefix: &str,
        depth: usize,
    ) -> Option<Vec<OutWire>> {
        if depth > 16 {
            return None;
        }
        let prebindgen_registry::flat::Type::Struct(st) = registry.flat().declared_type(ident)?
        else {
            return None;
        };
        let mut wires = Vec::new();
        for field in &st.fields {
            // Named by construction — a tuple struct is an `Extern`, not a
            // `Struct` — so a positional field means the model and this walk
            // disagree, and declining is the safe answer.
            let fname = field.name.as_ref()?;
            let name =
                crate::jni::mangle_kotlin_ident(&crate::jni::kt_snake_to_camel(&fname.to_string()));
            let name = match name_prefix.is_empty() {
                true => name,
                false => format!("{name_prefix}__{name}"),
            };
            let mut field_path = path.to_vec();
            field_path.push(fname.clone());

            // The layer questions off the field's own reading — `Optional` to
            // look through, `Vec` to decline — never a last path segment.
            let probe = field.ty.optional_inner().unwrap_or(&field.ty);
            match self.type_kind(registry, &probe.key()) {
                crate::jni::classify::TypeKind::Handle
                | crate::jni::classify::TypeKind::Enum
                | crate::jni::classify::TypeKind::Sum => return None,
                // A nested `data_class` inlines when it is reached directly.
                // Behind an `Option` or a `Vec` there is no chain to reach
                // through, so the whole value stays object-shaped.
                crate::jni::classify::TypeKind::DataStruct {
                    st: _,
                    cfg: Some(_),
                } => {
                    if field.ty.optional_inner().is_some()
                        || matches!(field.ty.kind(), TypeKind::Vec(_))
                    {
                        return None;
                    }
                    let child = match probe.unwrapped().kind() {
                        TypeKind::Named { id, .. } => id.ident()?,
                        _ => return None,
                    };
                    wires.extend(self.struct_out_wires_at(
                        registry,
                        &child,
                        &field_path,
                        &name,
                        depth + 1,
                    )?);
                    continue;
                }
                _ => {}
            }

            // A simple value: the field's own output conversion encodes it, and
            // the foreign `fromParts` forwards it verbatim. Nullability rides
            // that conversion — `Option<Box<String>>` is a `String?` — so the
            // value itself is not path-nullable.
            wires.push(OutWire {
                name,
                out_ty: field.ty.clone(),
                group: None,
                from: OutFrom::Place,
                reach: field_path.iter().map(field_step).collect(),
                nullable: false,
                identity: false,
            });
        }
        Some(wires)
    }

    /// The values a `sealed_class` hands out: the selector, then one group of
    /// slots per alternative, laid beside the others.
    ///
    /// Model and declaration only — no conversion is read. That is what lets
    /// one answer serve on both sides of `resolve`: the leaf synthesis feeding
    /// `Decompositions` runs before it, the recipe composes after it, and a fact
    /// derived twice is a fact that can differ.
    ///
    /// `None` for a type the model does not hold as a data-carrying enum, or
    /// one no `sealed_class!` declares — neither has a decomposition to state.
    pub(crate) fn sum_out_wires(
        &self,
        registry: &impl Conversions,
        ident: &syn::Ident,
        sum_ty: &TypeRef,
    ) -> Option<Vec<OutWire>> {
        let prebindgen_registry::flat::Type::Variant(sum) = registry.flat().declared_type(ident)?
        else {
            return None;
        };
        let cfg = self.types.get(&TypeKey::from_ident(ident))?.sum()?;

        // The selector rides ahead of the groups it chooses between, and
        // carries **which sum** it selects over — nothing converts it, so
        // naming the sum is the only use its type has, and it is the one an
        // emitter needs: which enum to match over.
        let mut wires = vec![OutWire {
            name: crate::jni::emit::SUM_TAG_LEAF.to_string(),
            out_ty: sum_ty.clone(),
            group: None,
            from: OutFrom::Tag,
            nullable: false,
            identity: false,
            reach: Vec::new(),
        }];
        for alt in &sum.alternatives {
            let kotlin = self.sum_variant_class_name(cfg, &alt.name);
            for field in &alt.fields {
                let member = field.member();
                wires.push(OutWire {
                    name: crate::jni::struct_plan::sum_slot_fragment(
                        &kotlin,
                        &crate::jni::struct_plan::sum_field_prop_name(&member),
                    ),
                    out_ty: field.ty.clone(),
                    group: Some(crate::jni::struct_plan::sum_tag(alt)),
                    from: OutFrom::Payload {
                        variant: Some(alt.name.clone()),
                        member,
                    },
                    nullable: false,
                    identity: false,
                    reach: Vec::new(),
                });
            }
        }
        Some(wires)
    }
}

/// One step of a reach that reads a struct field.
///
/// Never optional: a composition looks through an `Option` rather than stopping
/// at one — a terminal optional rides its own conversion, and an intermediate
/// one is a shape the compositions decline.
fn field_step(ident: &syn::Ident) -> prebindgen_registry::unfold::PathStep {
    prebindgen_registry::unfold::PathStep::field(ident.clone(), false)
}

/// The model field one part reads, or `None` for a part that is not a field.
fn part_field<'a>(part: &Part<'a>) -> Option<&'a prebindgen_registry::flat::Field> {
    match part.from {
        prebindgen_registry::recipe::PartSource::Field { field, .. } => Some(field),
        _ => None,
    }
}

/// The model field one part reads, which a sum payload names its slot after.
fn part_member(part: &Part<'_>) -> Option<syn::Member> {
    part_field(part).map(|f| f.member())
}

/// Every crossing a sum's arms delegate to, which is what reachability walks.
fn parts_subs(arms: &[(&Alternative, &JFrag)]) -> Vec<TypeKey> {
    arms.iter()
        .flat_map(|(_, f)| f.conv.subs.iter().cloned())
        .collect()
}

/// The Kotlin property name for one part of a product, sanitized — `object` is
/// a keyword, and only the emitter's own mangler knows that.
fn field_kt(part: &Part<'_>) -> String {
    crate::jni::render::kotlin_property_name(&syn::Ident::new(
        &part.name,
        proc_macro2::Span::call_site(),
    ))
}

/// Whether this conversion delivers a Kotlin typed handle rather than a value.
fn is_handle(frag: &JFrag) -> bool {
    frag.conv
        .metadata
        .projection
        .as_ref()
        .is_some_and(|p| p.kind == crate::jni::ProjectionKind::Handle)
}

/// One wire, read through a gate above it that may find nothing.
///
/// Every step below the gate becomes a safe call — not only the first: after
/// one `?.` the chain is on a nullable value and stays there. What each form
/// then needs is its own. An ordinary read carries the literal it stated for
/// exactly this case; a sum's `when` gains the `null` arm; a sum's `as?` slot
/// needs neither, a cast over a null subject being already null.
///
/// A wire whose value is a JVM object rides that `null` instead of a literal,
/// and its Kotlin type says so. Read **after** the literal rather than before,
/// because a `String` slot does both: it falls back to `""` when its own field
/// is present under a closed ancestor, and it is typed `String?`.
fn gated(w: &Wire) -> Wire {
    let mut gated = w.clone();
    for nav in gated
        .access
        .walk_mut()
        .iter_mut()
        .chain(gated.handle_target.iter_mut().flatten())
    {
        nav.gated = true;
    }
    match &mut gated.access {
        Access::Read { tail, .. } => {
            if let Some(absent) = gated.absent.take() {
                if !tail.contains(" ?: ") {
                    *tail = format!("{tail} ?: {absent}");
                }
            }
        }
        Access::Select { nullable, .. } => *nullable = true,
        Access::Slot { .. } => {}
    }
    if gated.entry.is_some()
        && matches!(gated.access, Access::Read { .. })
        && crate::jni::emit::is_jobject_shaped_wire(&gated.ty)
        && !gated.kt_ty.ends_with('?')
    {
        gated.kt_ty.push('?');
    }
    // Anything under the gate can be absent, however the field itself was
    // spelled.
    gated.handle_nullable = gated.handle_target.is_some() || gated.handle_nullable;
    gated
}

/// Whether this fragment states a sum taken apart into a tag and its arms'
/// slots, rather than a product taken apart into its fields.
fn is_choice(frag: &JFrag) -> bool {
    frag.wires
        .as_ref()
        .and_then(|w| w.first())
        .is_some_and(|w| matches!(w.access, Access::Select { .. }))
}
