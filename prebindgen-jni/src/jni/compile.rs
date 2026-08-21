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

/// How Kotlin reaches one wire value, relative to the object the site names.
///
/// Three forms rather than one string, because two of them put the base in the
/// **middle**: a sum's tag reads `when (<base>.f) { … }` and a sum's payload
/// slot reads `(<base>.f as? I.V)?.v0`. A suffix cannot say that, and a raw
/// prefix/suffix pair cannot be composed — an optional layer has to know
/// whether it is adding a `?.` navigation or a `null ->` arm, and only a named
/// form tells it which.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Access {
    /// `<base><tail>` — the ordinary read, including a presence comparison.
    Read(String),
    /// `when (<base><tail>) { … }` — which alternative of a sum is live, as the
    /// `jint` tag the arms below are numbered by.
    Select {
        /// What reaches the sum value from the base.
        tail: String,
        /// One arm per alternative, in declaration order, without the `null`
        /// one — an optional ancestor adds that by setting `nullable`.
        arms: Vec<String>,
        /// Whether the value reached can be null, which is its own arm.
        nullable: bool,
    },
    /// `(<base><tail> as? <class>)?.<read>[ ?: <zero>]` — one payload slot of
    /// one alternative. Every alternative's slots cross on every call; the
    /// cast yields null for the ones that are not live, and `zero` is what a
    /// non-nullable wire carries instead.
    Slot {
        /// What reaches the sum value from the base.
        tail: String,
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

impl Access {
    /// The Kotlin expression, rooted at the object this site destructures.
    ///
    /// Only the equivalence check calls it until the emitters take the row —
    /// the same `#[cfg(test)]` the wire's other readers carry.
    #[cfg(test)]
    pub(crate) fn render(&self, base: &str) -> String {
        match self {
            Access::Read(tail) => format!("{base}{tail}"),
            Access::Select {
                tail,
                arms,
                nullable,
            } => {
                let arms = nullable
                    .then(|| "null -> 0".to_string())
                    .into_iter()
                    .chain(arms.iter().cloned())
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("when ({base}{tail}) {{ {arms} }}")
            }
            Access::Slot {
                tail,
                class,
                read,
                zero,
            } => {
                let zero = zero
                    .as_ref()
                    .map(|z| format!(" ?: {z}"))
                    .unwrap_or_default();
                format!("({base}{tail} as? {class})?.{read}{zero}")
            }
        }
    }

    /// What reaches this value from the base — the part a container prepends
    /// its own field to, whichever form the access takes.
    fn tail_mut(&mut self) -> &mut String {
        match self {
            Access::Read(tail) | Access::Select { tail, .. } | Access::Slot { tail, .. } => tail,
        }
    }

    /// This access read from one field in, rather than from the object itself.
    fn under(mut self, field: &str) -> Self {
        let tail = self.tail_mut();
        *tail = format!(".{field}{tail}");
        self
    }
}

/// One wire value of a crossing that occupies several.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// — which is read on the Rust side and converts nothing.
    pub(crate) conv: Option<syn::Ident>,
    /// For a nested owned handle: where Kotlin finds the handle **object**, as
    /// against the `Long` this wire carries.
    ///
    /// A nested handle crosses under the same lock-and-consume scaffold as a
    /// top-level one, and that scaffold needs the object, not its pointer. The
    /// wire itself is filled from a local the scaffold binds.
    pub(crate) handle_target: Option<String>,
    /// Whether that handle access can be null — the field is optional, or an
    /// optional ancestor gates it.
    pub(crate) handle_nullable: bool,
    /// Whether the conversion carries Rust-side stages beyond its wire-facing
    /// function — a `convert!` with a semantic step, say `jlong -> u64 ->
    /// Duration`.
    ///
    /// Read where a caller may only call the wire-facing function and would
    /// otherwise bind the representation where the value is wanted: the `Vec`
    /// build helper declines such an element rather than emit a call that does
    /// not compile.
    pub(crate) staged: bool,
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
}

fn refuse(at: At<'_>, why: &str) -> String {
    format!("JniGen: {} ({why})", at.crossing.key())
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
    /// JniGen keeps its own per-site emission for now: the exported signature,
    /// the call and the cleanup are built from the resolved registry.
    type Plan = ();
    type Error = String;

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
            if let Some(pair) = self.decoupled_optional(at, inner) {
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
                access: Access::Read(" != null".to_string()),
                conv: None,
                handle_target: None,
                handle_nullable: false,
                staged: false,
                field: None,
                whole_gate: !sum,
            }];
            // Everything under the gate is reached through it, and a
            // non-nullable slot still has to hold something when the value is
            // absent — the flag is what tells Rust to ignore it.
            wires.extend(inner_wires.iter().map(|w| Wire {
                access: gated(&w.access, w),
                // Anything under the gate can be absent, however the field
                // itself was spelled.
                handle_nullable: w.handle_target.is_some() || w.handle_nullable,
                ..w.clone()
            }));
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
        _func: &Function,
        _parts: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(refuse(at, "JniGen states no value-form rows yet"))
    }

    fn fields(&mut self, cx: &mut Cx<'_>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self> {
        if at.crossing.assembly() != Assembly::Construct {
            return Err(refuse(
                at,
                "JniGen states no deconstructing product rows yet",
            ));
        }
        // A product whose own type is a sum is one **alternative's** payload:
        // the registry composes every arm through this hook and hands the lot
        // to `choice`. Its parts are read off a cast rather than off the value,
        // so they take the slot form — and which alternative to cast to is
        // `choice`'s to fill in, being the only hook told.
        if self.is_sum(cx, at) {
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
                        conv: w.conv.clone(),
                        handle_target: w
                            .handle_target
                            .as_ref()
                            .map(|t| format!(".{}{t}", field_kt(part))),
                        handle_nullable: w.handle_nullable,
                        staged: w.staged,
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
                    access: Access::Read(format!(".{}", field_kt(part))),
                    conv: None,
                    handle_target: Some(format!(".{}", field_kt(part))),
                    handle_nullable: part.ty.optional_inner().is_some(),
                    staged: false,
                    field: Some(part.name.clone()),
                    whole_gate: false,
                }),
                None => wires.push(Wire {
                    ty: frag.conv.destination.clone(),
                    kt_ty: crate::jni::emit::wire_kotlin_type(&frag.conv),
                    path: part.name.clone(),
                    access: Access::Read(format!(".{}", field_kt(part))),
                    conv: Some(frag.conv.function.sig.ident.clone()),
                    handle_target: None,
                    handle_nullable: false,
                    staged: !frag.conv.pre_stages.is_empty(),
                    field: Some(part.name.clone()),
                    whole_gate: false,
                }),
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
            return Err(refuse(
                at,
                "JniGen states no deconstructing choice rows yet",
            ));
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

    fn plan(&mut self, _cx: &mut Cx<'_>, _bound: &Bound, _root: &JFrag) -> Result<(), String> {
        Ok(())
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

/// Facts a wire states about itself, which the emitters read once they take
/// the `parts` row. Only the equivalence check calls them until then.
#[cfg(test)]
impl Wire {
    /// The struct field this value fills, which the Rust-side rebuild binds by
    /// name.
    ///
    /// The last segment of the path, because a path **is** the chain of fields
    /// that reached the value.
    ///
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
                tail: String::new(),
                // Named by `choice`, the only hook told which alternative this
                // payload belongs to.
                class: String::new(),
                read,
                zero: prim.map(|p| p.kotlin_zero().to_string()),
            },
            conv: Some(frag.conv.function.sig.ident.clone()),
            handle_target: None,
            handle_nullable: false,
            staged: !frag.conv.pre_stages.is_empty(),
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
                tail: String::new(),
                arms: arms
                    .iter()
                    .zip(&classes)
                    .map(|((alt, _), class)| {
                        format!("is {class} -> {}", crate::jni::struct_plan::sum_tag(alt))
                    })
                    .collect(),
                nullable: false,
            },
            conv: None,
            handle_target: None,
            handle_nullable: false,
            staged: false,
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
                        tail: String::new(),
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
    fn decoupled_optional(&self, at: At<'_>, inner: &JFrag) -> Option<Vec<Wire>> {
        let inner_reading = at.crossing.value().optional_inner()?;
        if inner_reading.borrow_target().is_some() {
            return None;
        }
        let c = &inner.conv;
        let prim = crate::jni::JniPrim::from_wire(&c.destination)?;
        if c.niches.clone().carve().is_some()
            || c.metadata.projection.is_some()
            || !c.pre_stages.is_empty()
        {
            return None;
        }
        // A Kotlin enum's slot is its `value`, reached through the same gate.
        let value_access = if self.decls.is_kotlin_enum_reading(inner_reading) {
            format!("?.value ?: {}", prim.kotlin_zero())
        } else {
            format!(" ?: {}", prim.kotlin_zero())
        };
        Some(vec![
            Wire {
                ty: syn::parse_quote!(jni::sys::jboolean),
                kt_ty: "Boolean".to_string(),
                path: "present".to_string(),
                access: Access::Read(" != null".to_string()),
                conv: None,
                handle_target: None,
                handle_nullable: false,
                staged: false,
                field: None,
                whole_gate: false,
            },
            Wire {
                ty: c.destination.clone(),
                kt_ty: crate::jni::emit::wire_kotlin_type(c),
                path: "value".to_string(),
                access: Access::Read(value_access),
                conv: Some(c.function.sig.ident.clone()),
                handle_target: None,
                handle_nullable: false,
                staged: false,
                field: None,
                whole_gate: false,
            },
        ])
    }
}

/// The model field one part reads, which a sum payload names its slot after.
fn part_member(part: &Part<'_>) -> Option<syn::Member> {
    match part.from {
        prebindgen_registry::recipe::PartSource::Field { field, .. } => Some(field.member()),
        _ => None,
    }
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

/// What a non-nullable slot holds while its gate is closed, as an elvis tail.
///
/// Empty for the two cases that need none. A **presence flag** is a `!= null`
/// comparison: it is already non-null, and a Kotlin elvis on a non-null operand
/// does not compile. A wire that **already carries one** got it from a gate
/// further in — the expression yields a value from there on, so an outer gate
/// has nothing left to substitute for.
fn absent_default(w: &Wire, tail: &str) -> String {
    if w.conv.is_none() && w.handle_target.is_none() {
        return String::new();
    }
    if tail.contains(" ?: ") {
        return String::new();
    }
    crate::jni::emit::kt_leaf_default(
        crate::jni::wire_access::jni_field_access(&w.ty)
            .map(|(sig, _, _)| sig)
            .unwrap_or(""),
        w.kt_ty.ends_with('?'),
    )
    .map(|d| format!(" ?: {d}"))
    .unwrap_or_default()
}

/// One wire's access, read through a gate that may find nothing.
///
/// Only an ordinary read navigates: it gains the `?` that stops at a null and
/// the literal a non-nullable slot then carries. A sum's `when` gains the
/// `null` arm instead, and a sum's `as?` slot needs neither — a cast over a
/// null subject is already null.
fn gated(access: &Access, w: &Wire) -> Access {
    match access {
        Access::Read(tail) => Access::Read(format!("?{tail}{}", absent_default(w, tail))),
        Access::Select { tail, arms, .. } => Access::Select {
            tail: tail.clone(),
            arms: arms.clone(),
            nullable: true,
        },
        slot @ Access::Slot { .. } => slot.clone(),
    }
}

/// Whether this fragment states a sum taken apart into a tag and its arms'
/// slots, rather than a product taken apart into its fields.
fn is_choice(frag: &JFrag) -> bool {
    frag.wires
        .as_ref()
        .and_then(|w| w.first())
        .is_some_and(|w| matches!(w.access, Access::Select { .. }))
}
