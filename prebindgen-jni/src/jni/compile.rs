//! What one crossing costs on the JNI wire.
//!
//! [`super::rows`] states which parts a value gets across in; this says what
//! each of those parts looks like across the Java Native Interface. The
//! registry drives the walk over the table and hands every hook the fragment
//! its inner crossing already produced, so nothing here decides which arity
//! layer it is looking at and nothing here recurses.

use prebindgen_registry::{
    flat::{Alternative, Function, TypeKind, TypeRef},
    recipe::{Assembly, At, Bound, Carrier, Compile, Cx, Frag, Mode, Part, Parts, Validity, Yield},
    Conversions,
};

use super::{
    trait_impl::{Produced, WrapperShape},
    *,
};

/// The JNI adapter's answer for one crossing.
///
/// What a `ConverterImpl` was, minus the bookkeeping the table now does.
#[derive(Clone)]
pub(crate) struct JFrag {
    pub(crate) conv: ConverterImpl<KotlinMeta>,
    pub(crate) yields: Yield,
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
    /// one job.
    pub(crate) out_wires: Option<Vec<OutWire>>,
    /// This fragment states a wire list and nothing else — no conversion of its
    /// own, so nothing of it reaches the generated file.
    ///
    /// Only the `parts` row is this. A crossing may legitimately carry **both**
    /// a wire list and a real conversion: an `Option<data_class>` composes a
    /// presence flag ahead of the inner's wires and still has the optional's
    /// own conversion to emit, so "has wires" is the wrong test for what to
    /// leave out of the file.
    pub(crate) composed_only: bool,
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
    /// One leaf of an expansion plan, in the row's vocabulary.
    ///
    /// The shim that lets the sum emitters speak rows before every plan is one:
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
                // An accessor chain and a field chain are one thing to a wire:
                // where the Rust side reaches the value. Which of the two it
                // was is the plan's, and nothing the sum emitters read.
                _ => OutFrom::Field { path: Vec::new() },
            },
            nullable: leaf.nullable,
        }
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
    /// A field of the value, reached by field access and cloned — the chain of
    /// idents from the value itself.
    ///
    /// Nested where a `data_class` field is itself one: its fields cross as
    /// decoupled values under the parent's chain, and the foreign side
    /// reassembles the whole graph in one call.
    Field {
        /// The field idents from the value down to this one.
        path: Vec<syn::Ident>,
    },
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
    /// A conversion this adapter built without the compiler.
    ///
    /// A callback crossing is the only one: `JniGen::compile_crossing` answers
    /// it with `dispatch_fn_input` rather than through a row, so there is no
    /// `At` to take a crossing's own mode from. It is an owned, self-sufficient
    /// value — a callback is delivered as a JVM object the wrapper holds — and
    /// nothing composes a callback as an inner, so no row ever reads this
    /// `Yield`. Goes with the derived callback row.
    pub(crate) fn by_hand(ty: TypeKey, conv: ConverterImpl<KotlinMeta>) -> Self {
        Self {
            conv,
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
        let validity = validity_of(&conv, at.crossing.assembly());
        Self {
            conv,
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
}

/// How long what this conversion produces stays usable.
///
/// A property of the **conversion**, not of how the crossing was spelled. The
/// two disagree and the spelling is the wrong one to read: a `&T` output over a
/// declared opaque handle clones its referent into a fresh `Box`-handle, and a
/// `&str` output copies into a JVM string, so both are self-sufficient although
/// the crossing is a borrow.
fn validity_of(conv: &ConverterImpl<KotlinMeta>, assembly: Assembly) -> Validity {
    match assembly {
        // Rust to the JVM. Every JNI wire value is a `jlong` the Rust side
        // handed over or a JVM object the JVM now owns; nothing on this wire
        // points into the Rust value it came from.
        Assembly::Deconstruct => Validity::SelfSufficient,
        // The JVM to Rust: what the converter's own function hands back. A
        // decode that yields a borrow is valid only for the call, which is
        // exactly right at a parameter and refused at a return.
        Assembly::Construct => match &conv.function.sig.output {
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
    /// Which site is being planned, when one is.
    ///
    /// `None` while compiling a row: a row answers for a crossing wherever it
    /// appears, so nothing about a site may reach it. Set only for the one hook
    /// the registry calls per site.
    pub(crate) site: Option<PlanSite>,
}

/// What [`Compile::plan`] needs about a site that the crossing does not say.
pub(crate) struct PlanSite {
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
}

impl<R: Conversions> Compile for JCompile<'_, R> {
    type Fragment = JFrag;
    /// One parameter of one exported function, classified.
    type Plan = crate::jni::fn_plan::PlanLeaf;
    type Error = JErr;

    fn atomic(&mut self, cx: &mut Cx<'_>, at: At<'_>) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let emit = cx.emit();
        let conv = match at.crossing.assembly() {
            Assembly::Construct => self
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
                    self.decls.dispatch_fn_input(args, self.registry, emit)
                }),
            Assembly::Deconstruct => self
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
        // A declared terminal outranks the arity the registry derived, exactly
        // as it did when one chain answered both: `input_terminal` claims a
        // `Cow<'_, [u8]>` blob, and a `convert!` may name an optional.
        let conv = match at.crossing.assembly() {
            Assembly::Construct => self
                .decls
                .input_terminal(ty, self.registry, emit)
                .or_else(|| self.decls.input_optional(ty, emit)),
            Assembly::Deconstruct => self
                .decls
                .output_terminal(ty, self.registry, emit)
                .or_else(|| self.decls.output_optional(ty, emit)),
        };
        let mut frag = self.wrap(at, "no JNI representation for this optional", conv)?;
        // An optional over something that crosses as several values cannot ride
        // a niche in any one of them — which of `(tag, summary)` would carry
        // the absence? So the presence is its own wire, ahead of the rest: the
        // `hMaybePresent` in `(hMaybePresent, hMaybeId)`.
        // A nullable primitive or enum with no niche keeps the
        // allocation-free `(present, value)` pair rather than boxing: the gate
        // is read on the Rust side and the slot carries the raw value. The
        // value crosses through the INNER's conversion, not the optional's —
        // there is no boxed `Option` on this wire to decode.
        if at.crossing.assembly() == Assembly::Construct && inner.wires.is_none() {
            if let Some(pair) = self.decoupled_optional(at, inner, &frag.conv) {
                frag.wires = Some(pair);
                return Ok(frag);
            }
        }
        if let (Assembly::Construct, Some(inner_wires)) = (at.crossing.assembly(), &inner.wires) {
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
            frag.wires = Some(wires);
        }
        Ok(frag)
    }

    fn sequence(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        _elements: Mode,
        _inner: &JFrag,
    ) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let emit = cx.emit();
        let conv = match at.crossing.assembly() {
            Assembly::Construct => self
                .decls
                .input_terminal(ty, self.registry, emit)
                .or_else(|| self.decls.input_run(ty, emit))
                .or_else(|| self.borrow(ty, emit, true))
                .or_else(|| self.decls.input_transparent_bridge(ty, self.registry, emit)),
            Assembly::Deconstruct => self
                .decls
                .output_terminal(ty, self.registry, emit)
                .or_else(|| self.decls.output_run(ty, emit))
                .or_else(|| self.borrow(ty, emit, false))
                .or_else(|| {
                    self.decls
                        .output_transparent_bridge(ty, self.registry, emit)
                }),
        };
        self.wrap(at, "no JNI representation for this run", conv)
    }

    fn construct(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _func: &Function,
        _args: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(refuse(at, "JniGen declares no constructor rows"))
    }

    fn value_form(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        func: &Function,
        parts: Parts<'_, Self>,
    ) -> Frag<Self> {
        if at.crossing.assembly() != Assembly::Construct {
            return Ok(self.out_value_form(at, func, parts));
        }
        Err(refuse(
            at,
            "JniGen states no constructing value-form rows yet",
        ))
    }

    fn fields(&mut self, cx: &mut Cx<'_>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self> {
        // A product whose own type is a sum is one **alternative's** payload:
        // the registry composes every arm through this hook and hands the lot
        // to `choice`. Which alternative that is stays `choice`'s to fill in,
        // being the only hook told — so both directions leave a hole here.
        let sum = self.is_sum(cx, at);
        if at.crossing.assembly() != Assembly::Construct {
            return match sum {
                true => Ok(self.out_arm(at, parts)),
                false => Ok(self.out_product(at, parts)),
            };
        }
        if sum {
            // Its parts are read off a cast rather than off the value, so they
            // take the slot form.
            return Ok(self.arm(at, parts));
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
        // Only the wire list is composed here. The conversion that reads these
        // several values and rebuilds the struct is what the emitter switch
        // brings; until then this row is compiled but never taken, so it
        // carries a marker rather than a conversion it would have to invent.
        // `prebindgen-c` does the same for a union arm, and for the same
        // reason: a product's parts do not always assemble into a function of
        // their own.
        let mut frag = JFrag::new(
            at,
            self.parts_marker(parts.iter().map(|(p, _)| p.ty.key()).collect()),
        );
        frag.wires = Some(wires);
        frag.composed_only = true;
        Ok(frag)
    }

    fn choice(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        arms: &[(&Alternative, &JFrag)],
    ) -> Frag<Self> {
        if at.crossing.assembly() != Assembly::Construct {
            return self.selected_out(at, arms);
        }
        // Which alternative is live crosses as its own `jint`, and every
        // alternative's slots cross on every call — the inert ones carrying the
        // literal their wire takes when the cast finds nothing. The N-way form
        // of the presence flag an optional already uses.
        match self.selected(at, arms) {
            Some(wires) => {
                let mut frag = JFrag::new(at, self.parts_marker(parts_subs(arms)));
                frag.wires = Some(wires);
                frag.composed_only = true;
                Ok(frag)
            }
            // A payload this adapter has no slot for — a nested object, a
            // handle — leaves the whole sum object-shaped, which is the row it
            // already had. Stated as a fragment with no wires rather than as a
            // refusal: the site that asked composes it as one value, exactly as
            // it did before a `parts` row existed.
            None => {
                let conv = self
                    .decls
                    .in_frag(at.crossing.spelled())
                    .ok_or_else(|| refuse(at, "no JNI representation for this sum"))?;
                Ok(JFrag::new(at, (*conv).clone()))
            }
        }
    }

    fn callback(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        _args: &[&JFrag],
        _result: Option<&JFrag>,
    ) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let TypeKind::Callback { args } = ty.unwrapped().kind() else {
            return Err(refuse(at, "a callback row over a type that is not one"));
        };
        let conv = self.decls.dispatch_fn_input(args, self.registry, cx.emit());
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
    fn plan(&mut self, _cx: &mut Cx<'_>, bound: &Bound, root: &JFrag) -> Result<PlanLeaf, JErr> {
        use crate::jni::fn_plan::{plan_error, InputKind, PlanLeaf};
        let site = self
            .site
            .as_ref()
            .ok_or_else(|| JErr::Refused("JniGen: a site compiled with no site context".into()))?;
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

        Ok(PlanLeaf {
            reading: reading.clone(),
            kt_name,
            kt_public,
            kt_meta,
            optional,
            as_enum_value,
            kind,
        })
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
    pub(crate) fn frag(&self, ty: &TypeRef, assembly: Assembly) -> Option<Conv> {
        Some(Conv(self.compiled.borrow().fragment(&ty.key(), assembly)?))
    }

    pub(crate) fn in_frag(&self, ty: &TypeRef) -> Option<Conv> {
        self.frag(ty, Assembly::Construct)
    }

    pub(crate) fn out_frag(&self, ty: &TypeRef) -> Option<Conv> {
        self.frag(ty, Assembly::Deconstruct)
    }

    /// Every wire the Kotlin → Rust crossing of `ty` occupies, or `None` when
    /// it occupies the single one its conversion names.
    ///
    /// A declared class states its composition under the `parts` row; an
    /// optional over one has no row of its own and composes on the row the
    /// registry derived, which is that crossing's default.
    pub(crate) fn wires_of(&self, ty: &TypeRef) -> Option<Vec<Wire>> {
        let key = ty.key();
        let compiled = self.compiled.borrow();
        compiled
            .row_fragment(&key, Assembly::Construct, &crate::jni::rows::parts())
            .or_else(|| compiled.fragment(&key, Assembly::Construct))?
            .wires
            .clone()
    }
}

/// A fragment's conversion, read without copying it.
///
/// The store is read while compilation is still writing to it, so a caller
/// cannot hold a borrow into it; sharing the fragment's `Rc` and reaching the
/// conversion through it costs a refcount instead of a whole `syn::ItemFn` per
/// lookup.
pub(crate) struct Conv(std::rc::Rc<JFrag>);

impl std::ops::Deref for Conv {
    type Target = ConverterImpl<KotlinMeta>;

    fn deref(&self) -> &Self::Target {
        &self.0.conv
    }
}

/// Facts a wire states about itself, which the emitters read off the row.
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
    /// The conversion a composed-only row carries: none.
    ///
    /// The `parts` and arm rows state what a value is made of and nothing
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

    /// The values a `data_class` hands out: its fields, and a field that is
    /// itself one contributes its own.
    ///
    /// A fragment with **no** out-wires is this adapter declining, and it
    /// declines for the whole value rather than per field. A handle, an
    /// `enum_class`, a sum or a `data_class` behind an `Option` or a `Vec` is
    /// delivered with a transform the decoupled form does not carry, and one
    /// such field sends the whole object down the whole-value `fromParts` path
    /// — so a row that decomposed the rest of it would describe a shape nothing
    /// emits.
    fn out_product(&self, at: At<'_>, parts: Parts<'_, Self>) -> JFrag {
        let declined = JFrag::new(at, self.parts_marker(Vec::new()));
        let mut wires = Vec::new();
        for (part, frag) in parts {
            let Some(field) = part_field(part) else {
                return declined;
            };
            let ident = match &field.name {
                Some(name) => name.clone(),
                None => return declined,
            };
            // The same name the leaf synthesis gives it: the Kotlin property,
            // and nested names joined by the reserved `__` separator.
            let name =
                crate::jni::mangle_kotlin_ident(&crate::jni::kt_snake_to_camel(&ident.to_string()));
            // The layer questions off the field's own reading — `Optional` to
            // look through, `Vec` to decline — never a last path segment.
            let probe = part.ty.optional_inner().unwrap_or(&part.ty);
            if matches!(
                self.decls.type_kind(self.registry, &probe.key()),
                crate::jni::classify::TypeKind::Handle
                    | crate::jni::classify::TypeKind::Enum
                    | crate::jni::classify::TypeKind::Sum
            ) {
                return declined;
            }
            match &frag.out_wires {
                // A nested `data_class`, which contributes its own values under
                // this field's name and chain. Only unwrapped: behind an
                // `Option` or a `Vec` there is no chain to reach through.
                Some(inner) => {
                    if part.ty.optional_inner().is_some()
                        || matches!(part.ty.kind(), TypeKind::Vec(_))
                    {
                        return declined;
                    }
                    for w in inner {
                        let OutFrom::Field { path } = &w.from else {
                            return declined;
                        };
                        wires.push(OutWire {
                            name: format!("{name}__{}", w.name),
                            out_ty: w.out_ty.clone(),
                            group: None,
                            from: OutFrom::Field {
                                path: std::iter::once(ident.clone())
                                    .chain(path.iter().cloned())
                                    .collect(),
                            },
                            nullable: false,
                        });
                    }
                }
                None => wires.push(OutWire {
                    name,
                    out_ty: part.ty.clone(),
                    group: None,
                    from: OutFrom::Field { path: vec![ident] },
                    nullable: false,
                }),
            }
        }
        let mut frag = JFrag::new(
            at,
            self.parts_marker(parts.iter().map(|(p, _)| p.ty.key()).collect()),
        );
        frag.out_wires = Some(wires);
        frag.composed_only = true;
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
                from: OutFrom::Field { path: vec![ident] },
                nullable: false,
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
            return Err(refuse(at, "a choice row over a type that is not named"));
        };
        let ident = id
            .ident()
            .ok_or_else(|| refuse(at, "a choice row over a type that is not one identifier"))?;
        // The composition is the declaration's and the model's, so the same
        // answer serves the leaf synthesis that runs before `resolve`. The arm
        // fragments the driver built are not read at all — a sum hands its
        // payloads out through their own conversions, and which those are is
        // the emitter's question rather than this row's.
        let wires = self
            .decls
            .sum_out_wires(self.registry, &ident, at.crossing.value())
            .ok_or_else(|| refuse(at, "a choice row over an undeclared sum"))?;
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
    /// The values a `sealed_class` hands out: the selector, then one group of
    /// slots per alternative, laid beside the others.
    ///
    /// Model and declaration only — no conversion is read. That is what lets
    /// one answer serve on both sides of `resolve`: the leaf synthesis feeding
    /// `Decompositions` runs before it, the row composes after it, and a fact
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
                });
            }
        }
        Some(wires)
    }
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
