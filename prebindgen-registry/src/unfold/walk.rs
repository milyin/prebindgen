//! The walk over a decomposed value.
//!
//! A decomposition — an [`UnfoldPlan`](crate::unfold::UnfoldPlan) — says which
//! **leaves** a value is delivered as and which **hoists** (intermediate value
//! forms) they are reached through. Performing that walk is the registry's:
//! binding each hoist once, reaching each leaf through its own path, and
//! deciding at every step whether what it holds is owned or borrowed.
//!
//! What an adapter supplies is the target-language half — how a leaf is
//! encoded, and how the delivered values are handed over — plus the two facts
//! this module cannot know: where a source item is qualified from
//! (`qualify`), and what each leaf is ([`DecomposedLeaf`]).

use prebindgen_flat::flat::TypeRef;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::unfold::{steps_are_movable, PathStep};

/// What the walk needs to know about one delivered leaf.
///
/// The adapter's own leaf carries far more — its wire type, its JNI or C
/// encoding, its projection — and none of that reaches here.
pub trait DecomposedLeaf {
    /// Steps from the decomposed value to this leaf.
    fn reach(&self) -> &[PathStep];

    /// The leaf's own name, used in diagnostics and in local names.
    fn name(&self) -> &str;

    /// Whether this leaf *is* the decomposed value rather than a part of it.
    fn identity(&self) -> bool;

    /// The reading the leaf delivers.
    fn source(&self) -> &TypeRef;

    /// Whether the last step reaching it is a field read rather than a call.
    ///
    /// Derived from [`Self::reach`], so no implementation can disagree with
    /// its own path — and the answer decides whether the reached place is
    /// cloned, which is an ownership decision rather than a spelling.
    fn is_field_read(&self) -> bool {
        matches!(self.reach().last(), Some(PathStep::Field { .. }))
    }
}

/// Compose one [`PathStep`] onto the reference expression reached so far.
/// A `Call` applies its accessor (origin-qualified); a `Field` reads the field
/// and re-borrows, so the result is a reference either way and steps chain
/// uniformly.
pub fn compose_step(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    step: &PathStep,
    e: TokenStream,
) -> TokenStream {
    match step {
        PathStep::Call { ident, .. } => {
            let m = qualify(ident);
            quote!(#m::#ident(#e))
        }
        PathStep::Field { ident, .. } => quote!(&(#e).#ident),
    }
}
/// Fold a run of steps onto `e`, borrowing wherever ownership demands it.
///
/// [`compose_step`] hands a `Call` its receiver as written, and an accessor
/// takes that receiver **by reference** — so an owned value in hand has to be
/// borrowed before the next call composes onto it. A value is in hand whenever
/// the previous step returned one (`f(..) -> T` rather than `-> &T`), which is
/// what [`PathStep::yields_owned`] records; `owned` says whether `e` itself
/// started that way.
///
/// A `Field` step needs no borrow either way — it composes as `&(e).f`, which
/// reads through a value and a reference alike.
///
/// This is the ONE place the rule lives, so every fold — a leaf's reach, a
/// conditional hoist's prefix, a sum's matched value — answers it the same way.
/// Splitting it produced exactly the bug it exists to prevent: ownership was
/// handled at the optional binding and at the value form, and lost at every
/// ordinary call in between.
pub fn fold_steps(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    steps: &[PathStep],
    mut e: TokenStream,
    mut owned: bool,
) -> TokenStream {
    for step in steps {
        if owned && matches!(step, PathStep::Call { .. }) {
            e = quote!(&#e);
        }
        e = compose_step(qualify, step, e);
        owned = step.yields_owned();
    }
    e
}
/// Compose a value form's OWN call — the one step in the whole system whose
/// receiver may be by value.
///
/// Four cases, from what the fold ended up holding crossed with what the
/// accessor takes. A CONSUMING form takes its receiver by value: hand it the
/// value when that is ours, clone when it is not — the same cost the borrowing
/// form of the accessor would have paid, which keeps one declaration usable by
/// both owned and `&T` roots. A borrowing form takes a reference, so an owned
/// value is borrowed for it.
///
/// The decision is made from the fold's RESULT, never from where the fold
/// began: that is what lets a consuming form sit behind ordinary accessors,
/// where the chain in front borrows and the form itself still moves.
///
/// One function because both hoist paths — the conditional binding and the
/// ordinary one — need exactly this rule, and stating it twice is what turned
/// each new shape into a new defect.
fn compose_value_form_call(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    call: &PathStep,
    e: TokenStream,
    e_owned: bool,
    consuming: bool,
) -> TokenStream {
    match (consuming, e_owned) {
        (true, owned) => {
            let (m, f) = (qualify(call.ident()), call.ident());
            // Parenthesized: the clone applies to whatever the fold holds, and
            // `&x.clone()` would parse as `&(x.clone())`.
            let arg = if owned { e } else { quote!((#e).clone()) };
            quote!(#m::#f(#arg))
        }
        (false, true) => compose_step(qualify, call, quote!(&#e)),
        (false, false) => compose_step(qualify, call, e),
    }
}
/// Start a reach from `base`, projecting the **leading run of plain field
/// steps** directly (`&base.a.b`) instead of through a borrow of the base
/// (`&(&base).a.b`). Returns the expression and how many steps it consumed.
///
/// The two forms name the same value, but the second borrows the base **as a
/// whole**, which the borrow checker rejects once a sibling leaf has moved a
/// different field out of it. Projecting directly makes each leaf's borrow
/// disjoint, so a consuming value form's field moves are order-independent
/// rather than compiling only while the borrowing leaves happen to be declared
/// first.
pub fn project_leading_fields(
    base: &TokenStream,
    base_is_ref: bool,
    path: &[PathStep],
) -> (TokenStream, usize) {
    if base_is_ref {
        return (base.clone(), 0);
    }
    let n = path.iter().take_while(|s| s.is_plain_field()).count();
    if n == 0 {
        return (quote!(&#base), 0);
    }
    let segs: Vec<&syn::Ident> = path[..n].iter().map(PathStep::ident).collect();
    (quote!(&#base #(.#segs)*), n)
}

/// Fold a leaf's whole `path` over `base` with no optional-step handling, then
/// apply the terminal treatment the leaf calls for: a leaf reached by a field
/// read is **cloned** out of the place it reached, because its converter takes
/// the field type as written (owned); every other leaf keeps the borrow its
/// converter expects.
///
/// One derivation, used by every delivery an adapter renders and by the
/// single-leaf shortcut a decomposed return takes. They drifted once while
/// they were two — the shortcut was missing the field clone and handed `&F` to
/// an `F` converter — which is why the walk is the registry's.
pub fn reach_leaf_flat<L: DecomposedLeaf>(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    leaf: &L,
    path: &[PathStep],
    base: TokenStream,
    base_is_ref: bool,
    consuming: bool,
) -> TokenStream {
    // An optional step BEFORE the last one needs a `match` whose `None` arm has
    // somewhere to go. This derivation has none — it yields a plain Rust value,
    // not a representation that can carry absence — so the shape is refused
    // here rather than composed into code that cannot type-check in the
    // consumer's crate.
    //
    // Asked of the leaf's OWN path, not of `path`. The caller may hand a
    // suffix: `wrapper.rs` rebases onto a hoisted local, and `Hoisted::innermost`
    // strips the prefix that bound it — including any optional step inside it.
    // Checking the parameter would therefore pass exactly when the hoist is the
    // conditional one, which is the case that cannot compose (an `Option<T>`
    // local with a field read hung off it). The full path is what the shape
    // question is about.
    let own_path = leaf.reach();
    assert!(
        !own_path.iter().rev().skip(1).any(PathStep::is_optional),
        "unfold: leaf `{}` reaches through an optional step but is \
         delivered as a single return value, which has no `None` arm — this \
         shape needs callback delivery",
        leaf.name(),
    );
    // Whether what this leaf reaches is OURS, and so is moved rather than
    // borrowed or cloned. The two leaf kinds say it differently:
    //
    // * an IDENTITY leaf carries the answer in its `out_ty` — the plan resolved
    //   it to the owned type exactly when the value is the plan's to give away
    //   (`place_is_owned`: an owned root, or a field of a CONSUMING value form),
    //   and that is also what selected the owning converter, which boxes the
    //   move rather than cloning a borrow;
    // * a FIELD leaf's `out_ty` is the field type as written, owned either way,
    //   so ownership is the enclosing form's: only a consuming one gives its
    //   fields away.
    //
    // How to project that place is `steps_are_movable`'s question, and it is
    // asked there rather than restated here. This used to spell it
    // `all(is_plain_field)`, defending the restatement on the grounds that a
    // trailing `Option` cannot reach return delivery anyway — true, and enforced
    // in `single_return` (`core/unfold.rs`), which is precisely why a local
    // restatement could disagree with the rule for as long as the invariant held
    // somewhere else. `plan.rs` says two readings would drift and the
    // disagreement would be a borrow handed to an owning converter; this is the
    // second reading, removed.
    let reached_is_ours = if leaf.identity() {
        !matches!(
            leaf.source().kind(),
            prebindgen_flat::flat::TypeKind::Ref { .. }
        )
    } else {
        consuming
    };
    if reached_is_ours && steps_are_movable(path) {
        let segs: Vec<&syn::Ident> = path.iter().map(PathStep::ident).collect();
        return quote!(#base #(.#segs)*);
    }
    let (e, lead) = project_leading_fields(&base, base_is_ref, path);
    let e = fold_steps(qualify, &path[lead..], e, false);
    if leaf.is_field_read() {
        quote!((#e).clone())
    } else {
        e
    }
}
/// Every value form on a plan, evaluated **once** and bound to a local
/// (`__vf0`, `__vf1`, …), so a struct is built once per delivery rather than
/// once per field. The bound prefixes come back with the statements, since
/// reaching a leaf means starting from the innermost local it sits under.
///
/// Shared by both delivery paths — the multi-leaf encoder below and the
/// single-leaf `Delivery::Return` shortcut in `emit/wrapper.rs`. The shortcut
/// used to compose its reach straight off the raw value, which for a consuming
/// value form emitted `f(&v)` against a by-value receiver: ill-typed Rust in
/// the consumer's crate. One binder, so the two cannot disagree about what a
/// hoist is or who owns it.
pub struct Hoisted {
    /// The `let __vfN = …;` bindings, outermost-first.
    pub stmts: TokenStream,
    /// Each hoist's path prefix and the local it was bound to.
    bound: Vec<(Vec<PathStep>, syn::Ident)>,
    /// Whether each bound hoist consumed the value it decomposed.
    consuming: Vec<bool>,
    /// Whether each bound local is `Option<TStruct>` rather than `TStruct` —
    /// the hoist sits under an optional step, so the value form ran only where
    /// the value was present. Its leaves cannot be emitted as independent
    /// statements: they share ONE `match` on the local (see
    /// [`encode_plan_leaves`]), taken by value, so a consuming form's fields
    /// still move out inside the arm.
    optional: Vec<bool>,
}
impl Hoisted {
    /// Index of the innermost bound hoist `path` sits under, with that prefix
    /// already consumed. `None` for a leaf under no value form at all — a
    /// sibling `.field()` / `.field_self()`, which still reaches from the value
    /// itself.
    fn innermost(&self, path: &[PathStep]) -> Option<(usize, Vec<PathStep>)> {
        self.bound
            .iter()
            .enumerate()
            .filter(|(_, (p, _))| p.len() < path.len() && path.starts_with(p))
            .max_by_key(|(_, (p, _))| p.len())
            .map(|(i, (p, _))| (i, path[p.len()..].to_vec()))
    }

    /// The innermost bound local `path` sits under, with that prefix already
    /// consumed, and whether that hoist gave its value away.
    pub fn rebase(&self, path: &[PathStep]) -> Option<(syn::Ident, Vec<PathStep>, bool)> {
        self.innermost(path)
            .map(|(i, rest)| (self.bound[i].1.clone(), rest, self.consuming[i]))
    }

    /// The innermost **conditional** hoist `path` sits under: its index, the
    /// local holding the `Option`, the name its `Some` arm binds, and the steps
    /// left to reach the leaf from there. `None` when the leaf's innermost
    /// hoist is unconditional (or there is none) — then [`Self::rebase`]
    /// applies and the leaf is an ordinary standalone statement.
    pub fn conditional(
        &self,
        path: &[PathStep],
    ) -> Option<(usize, syn::Ident, syn::Ident, Vec<PathStep>)> {
        let (i, rest) = self.innermost(path)?;
        self.optional[i].then(|| (i, self.bound[i].1.clone(), format_ident!("__u{}", i), rest))
    }

    /// The local a hoist was bound to.
    pub fn local(&self, i: usize) -> syn::Ident {
        self.bound[i].1.clone()
    }

    /// Whether a hoist consumed the value it decomposed.
    pub fn consumed(&self, i: usize) -> bool {
        self.consuming[i]
    }
}
/// Fold `path` over `base` the way an adapter's own gated reach does, but
/// yielding an `Option<…>` rather than the adapter's absent value: the optional
/// steps become a `map`/`and_then` chain, so an absent value short-circuits to
/// `None` instead of to whatever that adapter uses for absence. `body` renders
/// the innermost reached expression as a BARE value — the chain's last link
/// wraps it.
///
/// The gated reach this mirrors is not in this crate. JniGen keeps its own, as
/// `emit::delivery::reach_leaf`, because the absent value is its
/// representation's — which is one of the walks this module was meant to end.
///
/// This is how a CONDITIONAL value form is bound — the accessor runs only where
/// the value it decomposes is actually present.
fn reach_optional(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    path: &[PathStep],
    base: TokenStream,
    base_is_ref: bool,
    depth: usize,
    body: &dyn Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    let (e, lead) = project_leading_fields(&base, base_is_ref, path);
    match (lead..path.len()).find(|&i| path[i].is_optional()) {
        None => body(fold_steps(qualify, &path[lead..], e, false)),
        Some(k) => {
            // Through the optional step INCLUSIVE: the same fold, so the
            // borrow in front of it is the ordinary rule rather than a second
            // statement of it.
            let opt_e = fold_steps(qualify, &path[lead..=k], e, false);
            let bind = format_ident!("__hb{}", depth);
            // What the arm binds is the step's own value: an OWNED payload is a
            // bare `T`, so composing the next step onto it directly would hand
            // `T` to an accessor typed for `&T`. Say it is not a reference and
            // let `project_leading_fields` borrow it; a borrowed payload is
            // already one and passes through.
            //
            // With NO steps left the binding goes to `body` untouched — that is
            // what lets a consuming value form MOVE an owned payload rather than
            // borrow it straight back, so the terminal case stays "already a
            // reference" whatever the payload is.
            let rest = &path[k + 1..];
            let inner = reach_optional(
                qualify,
                rest,
                quote!(#bind),
                rest.is_empty() || !path[k].yields_owned(),
                depth + 1,
                body,
            );
            // `map` when this is the LAST optional step (the body yields a bare
            // value) and `and_then` when another follows (the recursion yields
            // an `Option` that must not nest). The equivalent `match` reads the
            // same but generated code runs through the consumer's lints, where
            // `clippy::manual_map` is a denial.
            let combinator = if rest.iter().any(PathStep::is_optional) {
                format_ident!("and_then")
            } else {
                format_ident!("map")
            };
            quote! {
                #opt_e.#combinator(|#bind| #inner)
            }
        }
    }
}
pub fn bind_hoists(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    hoists: &[crate::unfold::Hoist],
    value: &TokenStream,
    by_ref: bool,
) -> Hoisted {
    let mut out = Hoisted {
        stmts: TokenStream::new(),
        bound: Vec::new(),
        consuming: Vec::new(),
        optional: Vec::new(),
    };
    // Value forms COMPOSE, so each hoist is built from the longest hoist that
    // is already a proper prefix of it (they arrive outermost-first), and from
    // `value` otherwise.
    for (i, h) in hoists.iter().enumerate() {
        let local = format_ident!("__vf{}", i);
        // A hoist under an optional step binds `Option<TStruct>`: the value
        // form runs in the `Some` arm only. Core refuses to nest these, so the
        // enclosing value is always the plan's own — no rebase to consider.
        if h.prefix.iter().any(PathStep::is_optional) {
            let (last, lead) = h
                .prefix
                .split_last()
                .expect("a hoist prefix ends in its value-form call");
            let consuming = h.consuming;
            // The value form is handed the payload only when the optional step
            // is the LAST thing before it; any step in between composes as a
            // borrow, so what arrives is a reference either way.
            let owned = lead.last().is_some_and(PathStep::yields_owned);
            let expr = reach_optional(qualify, lead, value.clone(), by_ref, 0, &|reached| {
                compose_value_form_call(qualify, last, reached, owned, consuming)
            });
            out.stmts.extend(quote! { let #local = #expr; });
            out.bound.push((h.prefix.clone(), local));
            out.consuming.push(h.consuming);
            out.optional.push(true);
            continue;
        }
        // Where the fold starts, and whether what it starts from is OWNED. The
        // value form's own boundary is decided below, from what the fold ends
        // up holding — never from where it began.
        let (from, start, start_owned) = match out.rebase(&h.prefix) {
            // A NESTED consuming form is handed the parent's field by MOVE: a
            // hoisted value form is an owned struct and its fields are
            // disjoint, so moving one out leaves every sibling leaf readable.
            // `compose_step` borrows (`&(e).f`), so a plain field run to that
            // field is projected here instead of going through it.
            Some((outer, rest, _))
                if h.consuming && rest[..rest.len() - 1].iter().all(PathStep::is_plain_field) =>
            {
                let lead = &rest[..rest.len() - 1];
                let segs: Vec<&syn::Ident> = lead.iter().map(PathStep::ident).collect();
                (h.prefix.len() - 1, quote!(#outer #(.#segs)*), true)
            }
            // Any other rebased hoist: project its own leading field run
            // DIRECTLY off the parent local rather than reaching it through a
            // borrow of the parent. A sibling hoist may already have moved a
            // different field out — that is what a consuming value form does —
            // and `&(&__vf0).wrapper` borrows the partially moved parent as a
            // whole where `&__vf0.wrapper` is a disjoint borrow that survives.
            // Same invariant `project_leading_fields` states for leaf reaches,
            // and the same reason.
            Some((outer, rest, _)) => {
                let (e, lead) = project_leading_fields(&quote!(#outer), false, &rest);
                (h.prefix.len() - rest.len() + lead, e, false)
            }
            None if by_ref => (0, value.clone(), false),
            None => (0, value.clone(), true),
        };
        // Everything before the value form is an ordinary accessor chain.
        let last = h.prefix.len() - 1;
        let head = &h.prefix[from..last];
        let e = fold_steps(qualify, head, start, start_owned);
        let e_owned = head.last().map_or(start_owned, PathStep::yields_owned);
        // The value form itself. A CONSUMING one takes its receiver BY VALUE —
        // that is the move the whole declaration exists for — so it is handed
        // what the fold holds when that is ours, and a clone when it is not:
        // the same cost the borrowing form of the accessor would have paid,
        // which keeps one declaration usable by both owned and `&T` returns.
        // A borrowing one takes a reference, so an owned value is borrowed.
        //
        // Deciding this from the fold's RESULT rather than from its start is
        // what lets a consuming form sit behind ordinary accessors: the chain
        // in front borrows, the form itself still moves.
        let expr = compose_value_form_call(qualify, &h.prefix[last], e, e_owned, h.consuming);
        out.stmts.extend(quote! { let #local = #expr; });
        out.bound.push((h.prefix.clone(), local));
        out.consuming.push(h.consuming);
        out.optional.push(false);
    }
    out
}
