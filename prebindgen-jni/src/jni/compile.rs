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
}

impl Carrier for JFrag {
    fn yields(&self) -> Yield {
        self.yields.clone()
    }
}

impl JFrag {
    fn new(at: At<'_>, conv: ConverterImpl<KotlinMeta>) -> Self {
        let mode = at.crossing.mode();
        Self {
            conv,
            yields: Yield {
                ty: at.crossing.value().stripped_key(),
                mode,
                // A borrow is only usable while what it was reached through is
                // alive; anything else the JVM may keep.
                validity: match mode {
                    Mode::Owned => Validity::SelfSufficient,
                    Mode::Shared | Mode::Exclusive => Validity::Borrowed,
                },
            },
        }
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

impl<R: Conversions<KotlinMeta>> JCompile<'_, R> {
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
                self.registry,
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
                self.registry,
                emit,
            )?;
            c.subs = vec![inner.key()];
            Some(c)
        }
    }
}

impl<R: Conversions<KotlinMeta>> Compile for JCompile<'_, R> {
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
                .or_else(|| self.decls.input_optional(ty, self.registry, emit)),
            Assembly::Deconstruct => self
                .decls
                .output_terminal(ty, self.registry, emit)
                .or_else(|| self.decls.output_optional(ty, self.registry, emit)),
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
                .or_else(|| self.decls.input_run(ty, self.registry, emit))
                .or_else(|| self.borrow(ty, emit, true))
                .or_else(|| self.decls.input_transparent_bridge(ty, self.registry, emit)),
            Assembly::Deconstruct => self
                .decls
                .output_terminal(ty, self.registry, emit)
                .or_else(|| self.decls.output_run(ty, self.registry, emit))
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

    fn fields(&mut self, _cx: &mut Cx<'_>, at: At<'_>, _parts: Parts<'_, Self>) -> Frag<Self> {
        Err(refuse(at, "JniGen states no product rows yet"))
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
