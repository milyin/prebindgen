//! What one crossing costs on the JNI wire.
//!
//! [`super::rows`] states which parts a value gets across in; this says what
//! each of those parts looks like across the Java Native Interface. The
//! registry drives the walk over the table and hands every hook the fragment
//! its inner crossing already produced, so nothing here decides which arity
//! layer it is looking at and nothing here recurses.

use prebindgen_registry::{
    flat::{Alternative, Function, TypeKind, TypeRef},
    recipe::{Assembly, At, Bound, Carrier, Compile, Cx, Frag, Mode, Parts, Validity, Yield},
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
}

/// One wire value of a crossing that occupies several.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Wire {
    /// The JNI type this value crosses as.
    pub(crate) ty: syn::Type,
    /// The path through the value that reached it — `tag`, `summary.count` —
    /// which is what a JNI parameter name and a Kotlin accessor are both
    /// mangled from.
    pub(crate) path: String,
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

    fn optional(&mut self, cx: &mut Cx<'_>, at: At<'_>, _inner: &JFrag) -> Frag<Self> {
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
        self.wrap(at, "no JNI representation for this optional", conv)
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

    fn identity(&mut self, _cx: &mut Cx<'_>, at: At<'_>, _inner: &JFrag) -> Frag<Self> {
        Err(refuse(at, "JniGen declares no identity rows"))
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

    fn fields(&mut self, _cx: &mut Cx<'_>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self> {
        if at.crossing.assembly() != Assembly::Construct {
            return Err(refuse(
                at,
                "JniGen states no deconstructing product rows yet",
            ));
        }
        // A `data_class` crosses as its fields, and a field that is itself one
        // contributes its own several — which is the recursion, stated once
        // here rather than walked by hand.
        let mut wires: Vec<Wire> = Vec::new();
        for (part, frag) in parts {
            match &frag.wires {
                Some(inner) => wires.extend(inner.iter().map(|w| Wire {
                    ty: w.ty.clone(),
                    path: format!("{}.{}", part.name, w.path),
                })),
                None => wires.push(Wire {
                    ty: frag.conv.destination.clone(),
                    path: part.name.clone(),
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
            ConverterImpl {
                destination: syn::parse_quote!(()),
                function: syn::parse_quote!(
                    #[allow(dead_code)]
                    fn __jni_parts() {}
                ),
                pre_stages: Vec::new(),
                niches: Niches::empty(),
                metadata: KotlinMeta::default(),
                subs: parts.iter().map(|(p, _)| p.ty.key()).collect(),
            },
        );
        frag.wires = Some(wires);
        Ok(frag)
    }

    fn choice(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _arms: &[(&Alternative, &JFrag)],
    ) -> Frag<Self> {
        Err(refuse(at, "JniGen states no choice rows yet"))
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
