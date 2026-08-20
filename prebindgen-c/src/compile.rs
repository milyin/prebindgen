//! What one crossing costs on the C wire.
//!
//! [`crate::rows`] states which parts a value gets across in; this says what
//! each of those parts looks like in C. The registry drives the walk over the
//! table and hands every hook the fragments its parts already produced, so
//! nothing here looks a converter up and nothing here recurses.
//!
//! A [`CFrag`] is what a `ConverterImpl` was, minus the bookkeeping the table
//! now does: no `subs` walk to keep the registry's reachability right by hand,
//! no `pre_stages` chain, and no guessing which of nine categories a type falls
//! into — the row already said.

use prebindgen_registry::{
    flat::{Alternative, Function},
    recipe::{At, Carrier, Compile, Cx, Frag, Mode, Parts, Validity, Yield},
};

use super::*;

/// The C adapter's answer for one crossing.
#[derive(Clone)]
pub(crate) struct CFrag {
    /// The C wire type this crossing carries.
    pub(crate) destination: syn::Type,
    /// The generated converter, complete.
    pub(crate) function: syn::ItemFn,
    /// Bit patterns the wire can represent and this conversion never produces.
    pub(crate) niches: Niches,
    /// Inner types this fragment composed from, which is what marks them
    /// reachable in the registry that still emits them.
    pub(crate) subs: Vec<TypeKey>,
    /// What the fragment produces, which is all the registry reads of it.
    pub(crate) yields: Yield,
}

impl Carrier for CFrag {
    fn yields(&self) -> Yield {
        self.yields.clone()
    }
}

impl CFrag {
    /// One of the adapter's existing converter builders, as a fragment.
    fn from_converter(at: At<'_>, conv: ConverterImpl<()>) -> Self {
        let crossing = at.crossing;
        let mode = crossing.mode();
        Self {
            destination: conv.destination,
            function: conv.function,
            niches: conv.niches,
            subs: conv.subs,
            yields: Yield {
                ty: crossing.value().stripped_key(),
                mode,
                // A borrow is only usable while what it was reached through is
                // alive; anything else the C side may keep.
                validity: match mode {
                    Mode::Owned => Validity::SelfSufficient,
                    Mode::Shared | Mode::Exclusive => Validity::Borrowed,
                },
            },
        }
    }

    /// What the registry that still emits converters expects back.
    pub(crate) fn into_converter(self) -> ConverterImpl<()> {
        ConverterImpl {
            destination: self.destination,
            function: self.function,
            niches: self.niches,
            subs: self.subs,
            pre_stages: vec![],
            metadata: (),
        }
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

/// A refusal naming the crossing that could not be answered.
fn refuse(at: At<'_>, why: &str) -> String {
    format!("Cbindgen: {} ({why})", at.crossing.key())
}

impl<R: Conversions<()>> CCompile<'_, R> {
    fn wrap(&self, at: At<'_>, why: &str, conv: Option<ConverterImpl<()>>) -> Frag<Self> {
        conv.map(|c| CFrag::from_converter(at, c))
            .ok_or_else(|| refuse(at, why))
    }
}

impl<R: Conversions<()>> Compile for CCompile<'_, R> {
    type Fragment = CFrag;
    /// C keeps its own per-site emission for now: the exported signature, the
    /// call and the cleanup are built in `emit.rs` from the resolved registry.
    type Plan = ();
    type Error = String;

    fn atomic(&mut self, cx: &mut Cx<'_>, at: At<'_>) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let conv = match at.crossing.assembly() {
            Assembly::Construct => self
                .gen
                .in_custom(ty, self.registry, cx.emit())
                .or_else(|| self.gen.in_opaque_handle(ty))
                .or_else(|| self.gen.in_data_struct(ty, self.registry))
                .or_else(|| self.gen.in_value_opaque(ty, self.registry))
                .or_else(|| self.gen.in_enum(ty, self.registry))
                .or_else(|| self.gen.in_tagged_union(ty, self.registry, cx.emit()))
                .or_else(|| self.gen.in_string(ty))
                .or_else(|| self.gen.in_str(ty))
                .or_else(|| self.gen.in_bool(ty))
                .or_else(|| self.gen.in_scalar(ty))
                .or_else(|| self.gen.in_borrow(ty))
                .or_else(|| {
                    // A callback whose signature was declared: its own
                    // `#[repr(C)]` closure struct.
                    let args = ty.callback_args()?;
                    self.gen.dispatch_fn_input(args, self.registry)
                }),
            Assembly::Deconstruct => self
                .gen
                .out_custom(ty, self.registry, cx.emit())
                .or_else(|| self.gen.out_terminal(ty, self.registry, cx.emit()))
                .or_else(|| self.gen.out_borrow_or_result(ty)),
        };
        self.wrap(at, "no C representation for this type", conv)
    }

    fn optional(&mut self, _cx: &mut Cx<'_>, at: At<'_>, inner: &CFrag) -> Frag<Self> {
        let Some(elem) = at.crossing.value().optional_inner() else {
            return Err(refuse(
                at,
                "an optional row over a type that is not optional",
            ));
        };
        let conv = match at.crossing.assembly() {
            Assembly::Construct => self.gen.in_option(elem, inner),
            Assembly::Deconstruct => Some(self.gen.out_arity_marker("option", elem)),
        };
        self.wrap(at, "no C representation for this optional", conv)
    }

    fn sequence(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _elements: Mode,
        _inner: &CFrag,
    ) -> Frag<Self> {
        let ty = at.crossing.spelled();
        let conv = match at.crossing.assembly() {
            // A `&[E]` is the only run C builds a Rust value out of, and it does
            // it zero-copy from the caller's own block.
            Assembly::Construct => self.gen.in_slice(ty),
            // A `&[E]` callback argument is delivered by reference and has its
            // own marker; every other run is lowered structurally from the
            // element's.
            Assembly::Deconstruct => self
                .gen
                .out_slice_marker(ty)
                .or_else(|| Some(self.gen.out_arity_marker("vec", ty.sequence_elem()?))),
        };
        self.wrap(at, "no C representation for this run", conv)
    }

    fn identity(&mut self, _cx: &mut Cx<'_>, at: At<'_>, _inner: &CFrag) -> Frag<Self> {
        Err(refuse(at, "Cbindgen declares no identity rows"))
    }

    fn construct(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _func: &Function,
        _args: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(refuse(at, "Cbindgen declares no constructor rows"))
    }

    fn value_form(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _func: &Function,
        _parts: Parts<'_, Self>,
    ) -> Frag<Self> {
        Err(refuse(at, "Cbindgen declares no value-form rows"))
    }

    fn fields(&mut self, _cx: &mut Cx<'_>, at: At<'_>, _parts: Parts<'_, Self>) -> Frag<Self> {
        Err(refuse(at, "Cbindgen states no product rows yet"))
    }

    fn choice(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _arms: &[(&Alternative, &CFrag)],
    ) -> Frag<Self> {
        Err(refuse(at, "Cbindgen states no choice rows yet"))
    }

    fn callback(
        &mut self,
        _cx: &mut Cx<'_>,
        at: At<'_>,
        _args: &[&CFrag],
        _result: Option<&CFrag>,
    ) -> Frag<Self> {
        let Some(args) = at.crossing.value().callback_args() else {
            return Err(refuse(at, "a callback row over a type that is not one"));
        };
        let conv = self.gen.dispatch_fn_input(args, self.registry);
        self.wrap(at, "undeclared callback signature", conv)
    }

    fn plan(&mut self, _cx: &mut Cx<'_>, _bound: &Bound, _root: &CFrag) -> Result<(), String> {
        Ok(())
    }
}
