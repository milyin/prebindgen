//! Output-expansion delivery: unfold plans and leaf encoding.

use prebindgen_registry::{
    unfold::{bind_hoists, fold_steps, project_leading_fields, steps_are_movable, PathStep},
    Conversions,
};

use super::*;

/// Emit the output-expansion delivery body (output phase) for a function
/// marked `.expand_output()`. The return value (`__out`) is decomposed by the
/// plan's accessor leaves, each encoded into a JVM `Object`, and all delivered
/// to the foreign builder lambda (`__builder`) in a single `invoke` call whose
/// `JObject` result becomes the wrapper's return.
///
/// **Borrow ordering** (the user's zero-copy rationale): the reference
/// (non-identity) accessor leaves are encoded **first** — each leaf's converter
/// performs the single JVM copy (`&str -> jstring`), ending its borrow into
/// `__out` — then the identity/handle leaf is emitted **last**: an owned `T`
/// return is **moved** into the handle (`Box::into_raw(Box::new(__out))`, no
/// clone) once the borrows are gone; a `&T` return is **cloned** via the
/// borrowed-opaque output converter. The builder args are assembled in declared
/// leaf order regardless of encode order.
///
/// Shape handling: [`UnfoldShape::Base`] decomposes the returned value
/// directly; [`UnfoldShape::Optional`] uses the registry's Optional-over-Product
/// chain when available, yielding one presence wire plus the child's wires.
/// Absence skips the builder and delivers null. Leaf wires may be object
/// (JString/JByteArray/JObject — cast via `.into()`) or primitive (boxed to
/// `java.lang.*` via the cached `box_helper_for_wire` runtime helpers).
///
/// [`UnfoldShape::Base`]: prebindgen_registry::unfold::UnfoldShape::Base
/// [`UnfoldShape::Optional`]: prebindgen_registry::unfold::UnfoldShape::Optional
pub(crate) fn emit_unfold_delivery(
    plan: &prebindgen_registry::unfold::UnfoldPlan,
    output: &crate::jni::fn_plan::UnfoldOutputPlan,
    call_expr: &TokenStream,
    on_err: &TokenStream,
    emit: &prebindgen_registry::RustWriter,
) -> TokenStream {
    use prebindgen_registry::unfold::UnfoldShape;
    let context = &output.delivery;

    let n = plan.leaves.len();

    // Builder-arg locals, one per leaf in declared order. The builder is a
    // generated typed `<Source>Builder<out R>` fun interface — its `run`
    // method ID is resolved once per process on the interface class
    // ([`CachedIfaceMethod`]); primitives cross as raw typed jvalues.
    let obj_idents: Vec<syn::Ident> = (0..n).map(|i| format_ident!("__obj{}", i)).collect();

    // Return-site error path: route the message to the error sink, then return
    // the wrapper's sentinel. Threads through the shared leaf encoder.
    let fail = |msg: TokenStream| -> TokenStream {
        quote! {
            signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &#msg);
            return #on_err;
        }
    };

    // Encode a value's leaves (`__out`, a `Some`-bound `__inner`, or a Vec
    // `__elem`) into `__obj0…__objN` (shared with the callback trampoline),
    // yielding the per-leaf typed jvalue arg expressions.
    let encode_leaves = |value: &TokenStream, optional: bool| {
        let mut delivered = Delivered::planned(plan, output.wires.clone(), output.chain.clone());
        delivered.optional = optional;
        encode_plan_leaves(context, delivered, &obj_idents, value, &fail, emit)
    };

    // Cached-interface call statics for the builder / folder `run`.
    let iface_statics = |spec: &IfaceSpec| -> TokenStream {
        let fqn_lit = syn::LitStr::new(&spec.raw_slash_fqn(), Span::call_site());
        let descr_lit = syn::LitStr::new(&spec.descr, Span::call_site());
        quote! {
            #[allow(non_upper_case_globals)]
            static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod =
                ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            const __CB_FQN: &str = #fqn_lit;
            const __CB_DESCR: &str = #descr_lit;
        }
    };

    // Common builder-invoke (typed `run`, `Object` return = the erased `R`).
    // Used by `Decompose`/`Optional`; its success arm yields the result
    // `JObject`, error arms route to the sink + return the wrapper's sentinel.
    let builder_invoke = |arg_exprs: &[TokenStream]| -> TokenStream {
        quote! {
            match __CB_MID.call_object(
                &mut env, __CB_FQN, "run", __CB_DESCR, &__builder, &[#(#arg_exprs),*],
            ) {
                ::core::result::Result::Ok(__o) => __o,
                ::core::result::Result::Err(__e) => {
                    // Clears any pending JVM exception so the sink call is safe.
                    let _ = env.exception_describe();
                    let __e2 = <__JniErr as ::core::convert::From<String>>::from(__e.to_string());
                    signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e2.to_string());
                    #on_err
                }
            }
        }
    };

    // Decompose a value into leaves then invoke the builder once (`Decompose`/
    // `Optional`).
    let emit_decompose = |value: &TokenStream| -> TokenStream {
        let (leaves, arg_exprs, _) = encode_leaves(value, false);
        let invoke = builder_invoke(&arg_exprs);
        quote! { #leaves #invoke }
    };

    // Iterable (fold) delivery — possibly wrapped in ONE `Optional` layer.
    // `Vec<T>` folds the elements through the typed `<Element>Folder<A>.run(acc,
    // …)`, threading `__acc` and returning the final accumulator; per element the
    // fold args are either the element WHOLE (M4) or its decomposed leaves (M5),
    // with `acc` the erased `A` (`Object`). `Option<Vec<T>>` additionally yields a
    // null result for `None` (the fold is skipped).
    let opt_iterable = match &plan.shape {
        UnfoldShape::Iterable(_) => Some(false),
        UnfoldShape::Optional((), inner) if matches!(**inner, UnfoldShape::Iterable(_)) => {
            Some(true)
        }
        _ => None,
    };
    if let Some(optional) = opt_iterable {
        let statics = iface_statics(
            output
                .iface
                .as_deref()
                .expect("folder interface spec derivable for a resolved plan"),
        );
        let fold_invoke = |arg_exprs: &[TokenStream]| -> TokenStream {
            quote! {
                __acc = match __CB_MID.call_object(
                    &mut env, __CB_FQN, "run", __CB_DESCR, &__fold,
                    &[jni::sys::jvalue { l: __acc.as_raw() }, #(#arg_exprs),*],
                ) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        let _ = env.exception_describe();
                        let __e2 = <__JniErr as ::core::convert::From<String>>::from(__e.to_string());
                        signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e2.to_string());
                        return #on_err;
                    }
                };
            }
        };

        let loop_body = if plan.element.is_some() {
            // Whole-element (M4): encode the element via its own converter —
            // a raw typed jvalue for a primitive-wire element, a JObject
            // otherwise (mirrors `leaf_is_prim`; the folder interface
            // declares the matching typed param).
            let pipeline = output
                .element_pipeline
                .as_ref()
                .expect("whole-element fold carries its frozen output pipeline");
            let elem_call = pipeline.invoke(quote!(__elem), emit);
            let elem_conv = quote! {
                match #elem_call {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
                        return #on_err;
                    }
                }
            };
            let elem_wire = pipeline.wire().clone();
            // Primitive-wire elements (including an opaque **handle**, whose wire
            // is `jlong`) cross as a raw typed jvalue; object wires (String,
            // arrays) cross as a `JObject`. Keyed purely on the wire shape —
            // a handle's `Some(Handle)` projection still rides its `jlong`, and
            // the folder interface declares the matching `Long` (raw) param.
            let elem_is_prim = matches!(jni_field_access(&elem_wire), Some((_, _, false)));
            let enc = format_ident!("__enc");
            let (bind_obj, arg_expr) = if elem_is_prim {
                let letter = jni_field_access(&elem_wire).unwrap().1;
                (
                    TokenStream::new(),
                    quote!(jni::sys::jvalue { #letter: __enc }),
                )
            } else {
                let cast = cast_wire_to_jobject(&enc, &elem_wire, &fail);
                (
                    quote! { let __obj: jni::objects::JObject = #cast; },
                    quote!(jni::sys::jvalue { l: __obj.as_raw() }),
                )
            };
            let invoke = fold_invoke(&[arg_expr]);
            quote! {
                let __enc = #elem_conv;
                #bind_obj
                #invoke
            }
        } else {
            // Decomposed (M5): encode each element's leaves, fold over them.
            let (leaves, arg_exprs, _) = encode_leaves(&quote!(__elem), false);
            let invoke = fold_invoke(&arg_exprs);
            quote! {
                #leaves
                #invoke
            }
        };
        // Fold the elements of `__vec` into `__acc` (`into_iter()` yields the
        // element type exactly as written — owned `T`, or `&T` for a borrow).
        let fold = quote! {
            let mut __acc = __acc;
            for __elem in __vec.into_iter() {
                #loop_body
            }
            __acc
        };
        // `Option<Vec<T>>`: `None` ⇒ null result; `Some(vec)` ⇒ fold. A bare
        // `Vec<T>` folds the returned value directly.
        return if optional {
            quote! {
                #statics
                let __out = #call_expr;
                match __out {
                    ::core::option::Option::Some(__vec) => { #fold }
                    ::core::option::Option::None => #on_err,
                }
            }
        } else {
            quote! {
                #statics
                let __vec = #call_expr;
                #fold
            }
        };
    }

    match &plan.shape {
        UnfoldShape::Base => {
            let statics = iface_statics(
                output
                    .iface
                    .as_deref()
                    .expect("builder interface spec derivable for a registered declaration"),
            );
            let body = emit_decompose(&quote!(__out));
            quote! {
                #statics
                let __out = #call_expr;
                #body
            }
        }
        UnfoldShape::Optional((), inner) => {
            match **inner {
                UnfoldShape::Base => {}
                _ => panic!(
                    "emit_unfold_delivery: Optional inner must be Base (scalar) or \
                     Iterable (`Option<Vec<T>>`, handled above)"
                ),
            }
            let statics = iface_statics(
                output
                    .iface
                    .as_deref()
                    .expect("builder interface spec derivable for a registered declaration"),
            );
            let (leaves, arg_exprs, present) = encode_leaves(&quote!(__out), true);
            if let Some(present) = present {
                let invoke = builder_invoke(&arg_exprs);
                quote! {
                    #statics
                    let __out = #call_expr;
                    #leaves
                    if #present != 0 {
                        #invoke
                    } else {
                        #on_err
                    }
                }
            } else {
                // A non-composed declaration keeps the established delivery
                // path until its shape has a registry recipe.
                let body = emit_decompose(&quote!(__inner));
                quote! {
                    #statics
                    let __out = #call_expr;
                    match __out {
                        ::core::option::Option::Some(__inner) => { #body }
                        ::core::option::Option::None => #on_err,
                    }
                }
            }
        }
        UnfoldShape::Iterable(_) => {
            unreachable!("Iterable delivery is handled by the `opt_iterable` branch above")
        }
    }
}

/// Cast an encoded wire local to a `JObject` for the erased `invoke`: object
/// wires pass through / `.into()`; primitive wires box to `java.lang.*`.
/// `fail(msg)` — `msg` an expression yielding `String` — produces the
/// diverging on-error statements (sink + sentinel at a return site, `Err` in
/// the trampoline). Returns an expression yielding `JObject`.
pub(crate) fn cast_wire_to_jobject(
    enc: &syn::Ident,
    wire: &syn::Type,
    fail: &dyn Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    if is_jobject_wire(wire) {
        quote!(#enc)
    } else if matches!(jni_field_access(wire), Some((_, _, true))) {
        quote!(#enc.into())
    } else if let Some(helper) = box_helper_for_wire(wire) {
        let on_fail = fail(quote!(__e));
        quote! {
            match ::prebindgen_jni_runtime::#helper(&mut env, #enc) {
                ::core::result::Result::Ok(__o) => __o,
                ::core::result::Result::Err(__e) => {
                    #on_fail
                }
            }
        }
    } else {
        panic!(
            "jnigen unfold: leaf has unsupported wire `{}`",
            wire.to_token_stream()
        )
    }
}

/// One delivery site, as the encoder sees it: what it hands out, and the value
/// forms it evaluates once on the way.
///
/// Not an `UnfoldPlan`. A plan carries the delivery's shape as well — whether
/// it folds, whether it is optional, what the builder's generic is — and none
/// of that reaches this encoder, which writes only the statements that fill the
/// slots. Naming exactly what it reads is what lets the caller assemble one
/// from a recipe.
pub(crate) struct Delivered<'a> {
    /// The values handed out, in the order the builder receives them.
    pub(crate) wires: Vec<crate::jni::compile::OutWire>,
    /// The value forms to evaluate once and bind to a local, outermost first.
    ///
    /// Per **site** rather than per value: two values reached through one
    /// accessor share its call, and that is the whole point of a hoist.
    pub(crate) hoists: &'a [prebindgen_registry::unfold::Hoist],
    /// Whether the delivered value is reached through a borrow.
    pub(crate) by_ref: bool,
    /// True when these leaves are the model-derived data-class Product itself.
    pub(crate) fixed_product: bool,
    /// Whether the fixed Product is wrapped in one outer Optional shape.
    pub(crate) optional: bool,

    /// Exact composed child handed down by an enclosing registry recipe.
    pub(crate) chain: Option<crate::jni::compile::ComposedChain>,
}

/// Facts the leaf renderer may consult after a delivery operation has been
/// selected. The live implementation serves ordinary wrapper rendering; an
/// Invoke plan freezes the same answers so callback planning does not retain a
/// registry or obtain [`prebindgen_registry::RustWriter`].
pub(crate) trait DeliveryContext {
    fn qualify(&self, ident: &syn::Ident) -> syn::Path;
    fn leaf_is_prim(&self, leaf: &crate::jni::compile::OutWire) -> bool;
    fn leaf_wire(&self, leaf: &crate::jni::compile::OutWire) -> syn::Type;
    fn sum(
        &self,
        leaf: &crate::jni::compile::OutWire,
    ) -> (syn::Path, &prebindgen_registry::flat::Variant);
}

#[derive(Clone)]
struct FrozenSum {
    source: syn::Path,
    model: prebindgen_registry::flat::Variant,
}

/// Callback delivery facts frozen while the registry is available. Rust
/// source types remain as opaque readings inside the wires/pipelines; the only
/// syntax retained here is origin qualification and Flat alternative shape,
/// both consumed with the writer by the final Invoke renderer.
#[derive(Clone)]
pub(crate) struct FrozenDelivery {
    wires: Vec<crate::jni::compile::OutWire>,
    hoists: Vec<prebindgen_registry::unfold::Hoist>,
    by_ref: bool,
    fixed_product: bool,
    optional: bool,
    chain: Option<crate::jni::compile::ComposedChain>,
    modules: std::collections::BTreeMap<String, syn::Path>,
    sums: std::collections::BTreeMap<String, FrozenSum>,
}

impl FrozenDelivery {
    /// Every converter encoding these leaves calls: each leaf's own pipeline,
    /// and the composed converter when the delivery has one.
    pub(crate) fn calls(&self, out: &mut Vec<prebindgen_registry::write::ArtifactKey>) {
        for wire in &self.wires {
            wire.calls(out);
        }
        if let Some(chain) = &self.chain {
            out.push(prebindgen_registry::write::ArtifactKey::Operation(
                chain.operation.clone(),
            ));
        }
    }

    pub(crate) fn new(
        ext: &Declarations,
        registry: &impl Conversions,
        plan: &prebindgen_registry::unfold::UnfoldPlan,
        wires: Vec<crate::jni::compile::OutWire>,
        chain: Option<crate::jni::compile::ComposedChain>,
    ) -> Self {
        let mut modules = std::collections::BTreeMap::new();
        for step in plan
            .hoists
            .iter()
            .flat_map(|hoist| &hoist.prefix)
            .chain(wires.iter().flat_map(|wire| wire.reach()))
        {
            if let prebindgen_registry::unfold::PathStep::Call { ident, .. } = step {
                modules
                    .entry(ident.to_string())
                    .or_insert_with(|| ext.fn_module(registry, ident));
            }
        }
        let mut sums = std::collections::BTreeMap::new();
        for wire in wires.iter().filter(|wire| wire.is_tag()) {
            let prebindgen_registry::flat::TypeKind::Named { id, .. } =
                wire.out_ty.unwrapped().kind()
            else {
                panic!("jnigen sum unfold: selector type is not named")
            };
            let ident = id.ident().unwrap_or_else(|| {
                panic!(
                    "jnigen sum unfold: selector type `{}` is not an identifier",
                    id.name
                )
            });
            let Some(prebindgen_registry::flat::Type::Variant(sum)) =
                registry.flat().declared_type(&ident)
            else {
                panic!("jnigen sum unfold: no indexed sum `{ident}`")
            };
            let module = ext.fn_module(registry, &ident);
            sums.entry(id.name.clone()).or_insert_with(|| FrozenSum {
                source: syn::parse_quote!(#module::#ident),
                model: sum.clone(),
            });
        }
        Self {
            wires,
            hoists: plan.hoists.clone(),
            by_ref: plan.by_ref,
            fixed_product: plan.fixed_builder,
            optional: plan.is_optional_base(),
            chain,
            modules,
            sums,
        }
    }

    pub(crate) fn delivered(&self) -> Delivered<'_> {
        Delivered {
            wires: self.wires.clone(),
            hoists: &self.hoists,
            by_ref: self.by_ref,
            fixed_product: self.fixed_product,
            optional: self.optional,
            chain: self.chain.clone(),
        }
    }

    pub(crate) fn wire_count(&self) -> usize {
        self.wires.len()
    }
}

impl DeliveryContext for FrozenDelivery {
    fn qualify(&self, ident: &syn::Ident) -> syn::Path {
        self.modules
            .get(&ident.to_string())
            .unwrap_or_else(|| panic!("frozen JNI delivery has no origin for `{ident}`"))
            .clone()
    }

    fn leaf_is_prim(&self, leaf: &crate::jni::compile::OutWire) -> bool {
        frozen_leaf_is_prim(leaf)
    }

    fn leaf_wire(&self, leaf: &crate::jni::compile::OutWire) -> syn::Type {
        frozen_leaf_wire(leaf)
    }

    fn sum(
        &self,
        leaf: &crate::jni::compile::OutWire,
    ) -> (syn::Path, &prebindgen_registry::flat::Variant) {
        let prebindgen_registry::flat::TypeKind::Named { id, .. } = leaf.out_ty.unwrapped().kind()
        else {
            panic!("jnigen sum unfold: selector type is not named")
        };
        let sum = self
            .sums
            .get(&id.name)
            .unwrap_or_else(|| panic!("frozen JNI delivery has no sum `{}`", id.name));
        (sum.source.clone(), &sum.model)
    }
}

impl<'a> Delivered<'a> {
    /// Delivery from a registry-compiled return, callback, or error site.
    pub(crate) fn planned(
        plan: &'a prebindgen_registry::unfold::UnfoldPlan,
        wires: Vec<crate::jni::compile::OutWire>,
        chain: Option<crate::jni::compile::ComposedChain>,
    ) -> Self {
        Self {
            wires,
            hoists: &plan.hoists,
            by_ref: plan.by_ref,
            chain,
            fixed_product: plan.fixed_builder,
            optional: plan.is_optional_base(),
        }
    }
}

/// Bind `e` so it can be destructured as an `Option` **whatever Rust
/// representation the source used for it**.
///
/// `kind` says a position is optional; it deliberately does not say whether
/// Rust spells that `Option<T>`, `Box<Option<T>>`, or something else — the flat
/// model states the destination-language invariant, and the side interpreting
/// it is the side that must accept any representation. Matching the reached
/// place directly assumed one, which is `classify off kind, spell with spell()`
/// broken in the direction nothing was watching: the classification was right
/// and the *spelling* came from it too. `Box<Option<T>>` then produced
/// `match &place { Some(..) => .. }` and `E0308` (#268).
///
/// A type-ascribed `let` is a coercion site, and deref coercion is transitive
/// **and** a no-op when the types already match — so this one shape serves
/// every representation, and the plain `Option<T>` case is unchanged in
/// behaviour. The payload stays `_`: what it is, is the source's business.
///
/// `e` is expected to be a **reference** already — [`compose_step`] composes a
/// field read as `&(e).f` — so nothing is borrowed here. Borrowing only: an
/// owned position cannot be made representation-agnostic this way, because
/// deref coercion applies to references and moving out of a wrapper is
/// something only some of them permit (`Box` does, `Rc` cannot). A site that
/// must MOVE the payload keeps its direct match; see `owned_place` below.
pub(crate) fn bind_as_option(e: &TokenStream, bind: &syn::Ident) -> TokenStream {
    quote! { let #bind: &::core::option::Option<_> = #e; }
}

/// Reach a leaf's input by folding its `path` over `base`, then hand the
/// reached expression to `body` (which renders the encode and yields
/// `JObject`). Every optional nesting step becomes a `match`: its `None` arm
/// short-circuits the whole leaf to `JObject::null()` (the value is absent ⇒
/// the leaf is null) — any number of optional steps on the path nest.
/// With `unwrap_last == false` the final path element composes directly — a
/// non-identity leaf's converter takes the final step's **full** type
/// (`Option` included), so only the steps *before* it are nesting. An
/// identity leaf (`unwrap_last == true`) delivers the reached value itself, so
/// a final `Option` step unwraps too.
fn reach_leaf(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    path: &[PathStep],
    base: TokenStream,
    base_is_ref: bool,
    unwrap_last: bool,
    depth: usize,
    body: &dyn Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    let limit = if unwrap_last {
        path.len()
    } else {
        path.len().saturating_sub(1)
    };
    let (e, lead) = project_leading_fields(&base, base_is_ref, path);
    match (lead..limit).find(|&i| path[i].is_optional()) {
        // No (more) optional nesting steps: compose the rest plainly.
        None => body(fold_steps(qualify, &path[lead..], e, false)),
        Some(k) => {
            // Through the optional step INCLUSIVE: the same fold, so the
            // borrow in front of it is the ordinary rule rather than a second
            // statement of it.
            let opt_e = fold_steps(qualify, &path[lead..=k], e, false);
            let nested = format_ident!("__n{}", depth);
            let inner = reach_leaf(
                qualify,
                &path[k + 1..],
                quote!(#nested),
                true,
                unwrap_last,
                depth + 1,
                body,
            );
            // A FIELD read composes to a borrow (`&(e).f`), so it goes through
            // a coercion site and the destructuring stops caring which
            // representation the source spelled the optional as.
            //
            // A CALL composes to the accessor's own returned value, which is
            // owned and whose payload downstream may move. Borrowing it to
            // coerce would change that ownership, so it keeps its direct match
            // — and an owned position could not be made representation-agnostic
            // this way regardless (see [`bind_as_option`]).
            if path[k].is_field() {
                let opt_bind = format_ident!("__o{}", depth);
                let coerce = bind_as_option(&opt_e, &opt_bind);
                quote! {
                    {
                        #coerce
                        match #opt_bind {
                            ::core::option::Option::Some(#nested) => { #inner }
                            ::core::option::Option::None => jni::objects::JObject::null(),
                        }
                    }
                }
            } else {
                quote! {
                    match #opt_e {
                        ::core::option::Option::Some(#nested) => { #inner }
                        ::core::option::Option::None => jni::objects::JObject::null(),
                    }
                }
            }
        }
    }
}

/// Encode a plan's leaves off `value` (`__out`, a `Some`-bound `__inner`, a Vec
/// `__elem`, an owned callback arg, or a domain error `__de`) into the
/// `obj_idents` locals, in declared-leaf order. Reference (non-identity)
/// leaves are encoded first — ending their borrow into the value — and the
/// identity leaf last (move owned / clone `&T`). Each leaf's value is reached
/// by folding its accessor `path` over `value`; every `Option`-returning
/// nesting step on the path wraps the rest in a `match Some/None` (`None` ⇒ a
/// null leaf) — see [`reach_leaf`]. Error arms are produced by `fail` (see
/// [`cast_wire_to_jobject`]). Shared by the return-delivery site
/// ([`emit_unfold_delivery`]), the callback trampoline, and the domain-error
/// arm of fallible externs (whose `fail` routes an encoding failure to the
/// binding-error channel).
pub(crate) fn encode_plan_leaves(
    context: &impl DeliveryContext,
    site: Delivered<'_>,
    obj_idents: &[syn::Ident],
    value: &TokenStream,
    fail: &dyn Fn(TokenStream) -> TokenStream,
    emit: &prebindgen_registry::RustWriter,
) -> (TokenStream, Vec<TokenStream>, Option<syn::Ident>) {
    let Delivered {
        wires,
        hoists,
        by_ref,
        chain,
        fixed_product,
        optional,
    } = site;
    // Per-fn origin qualification: each accessor call is prefixed with the
    // module of the crate that defines it (multi-source bindings).
    let qualify = |id: &syn::Ident| -> syn::Path { context.qualify(id) };
    let n = wires.len();

    // Typed `jvalue` argument expression per leaf, in leaf order: a non-null
    // primitive-wire leaf passes its raw primitive (`__objN` IS the jvalue);
    // every other leaf is a `JObject` local whose raw pointer rides the `l`
    // slot. Matches the descriptor [`crate::jni::iface`]
    // derives for the same leaf (primitive chunk vs object chunk).
    let mut arg_exprs: Vec<TokenStream> = Vec::with_capacity(n);
    for (idx, leaf) in wires.iter().enumerate() {
        let obj_ident = &obj_idents[idx];
        if context.leaf_is_prim(leaf) {
            arg_exprs.push(quote!(#obj_ident));
        } else {
            arg_exprs.push(quote!(jni::sys::jvalue { l: #obj_ident.as_raw() }));
        }
    }

    // A fixed decomposition has one Product, Optional or Choice intermediate.
    // Invoke its converter once, then adapt the intermediate leaves to the JNI
    // call ABI. Callback and ordinary output delivery therefore share the
    // same Rust-value walk; only the final delivery remains JNI-specific.
    if fixed_product && hoists.is_empty() {
        let chain = chain;
        if let Some(chain) = chain.filter(|chain| match &chain.layout {
            crate::jni::compile::JLayout::Optional(inner) => optional && inner.leaf_count() == n,
            layout => !optional && layout.leaf_count() == n,
        }) {
            chain.activate();
            let encoded: Vec<syn::Ident> = (0..n)
                .map(|index| format_ident!("__chain_wire{index}"))
                .collect();
            let present = optional.then(|| format_ident!("__chain_present"));
            let pattern_values: Vec<syn::Ident> = present
                .iter()
                .cloned()
                .chain(encoded.iter().cloned())
                .collect();
            let pattern = chain.layout.pattern(&pattern_values);
            let converter = emit.operation_ident("jni", &chain.operation);
            let on_chain_error = fail(quote!(__chain_error.to_string()));
            let mut stmts = quote! {
                let #pattern = match #converter(&mut env, #value) {
                    ::core::result::Result::Ok(__intermediate) => __intermediate,
                    ::core::result::Result::Err(__chain_error) => {
                        #on_chain_error
                    }
                };
            };
            for (index, leaf) in wires.iter().enumerate() {
                let encoded = &encoded[index];
                let object = &obj_idents[index];
                if leaf.is_tag() {
                    stmts.extend(quote! {
                        let #object = jni::sys::jvalue { i: #encoded };
                    });
                    continue;
                }
                let wire = context.leaf_wire(leaf);
                if context.leaf_is_prim(leaf) {
                    let (_, member, _) = jni_field_access(&wire)
                        .expect("a primitive Product leaf has a JNI jvalue member");
                    stmts.extend(quote! {
                        let #object = jni::sys::jvalue { #member: #encoded };
                    });
                } else {
                    let cast = cast_wire_to_jobject(encoded, &wire, fail);
                    stmts.extend(quote! {
                        let #object: jni::objects::JObject = #cast;
                    });
                }
            }
            return (stmts, arg_exprs, present);
        }
    }
    let hoisted = bind_hoists(&qualify, hoists, value, by_ref);
    let mut stmts = hoisted.stmts.clone();

    // Reach a leaf off the innermost value form it sits under, with that
    // prefix's steps already consumed, and say whether that form CONSUMED its
    // value — so the leaf owns its field and may move it out rather than clone
    // it. A leaf under no value form at all (a sibling `.field()` /
    // `.field_self()`) still reaches from the value itself.
    // A leaf under a CONDITIONAL value form reaches off the name that form's
    // `Some` arm binds — a borrow of the struct, so nothing moves out of it —
    // and its statements are collected into that arm rather than emitted here.
    let rebase = |leaf: &crate::jni::compile::OutWire| -> (TokenStream, bool, Vec<PathStep>, bool) {
        let path = leaf.reach();
        if let Some((i, _, bind, rest)) = hoisted.conditional(path) {
            return (quote!(#bind), false, rest, hoisted.consumed(i));
        }
        match hoisted.rebase(path) {
            Some((local, rest, consuming)) => (quote!(#local), false, rest, consuming),
            None => (value.clone(), by_ref, path.to_vec(), false),
        }
    };

    // A decomposed **sum** is the one shape whose leaves are not independent:
    // only one group is live per value, so its whole segment — the selector
    // leaf plus the group leaves that follow it — is emitted as ONE `match`
    // instead of per-leaf expressions. A plan may carry several: a sum that IS
    // the returned value is the degenerate case of one segment covering
    // everything, while a value form contributes one per sum-typed field.
    let sum_segments: Vec<std::ops::Range<usize>> = (0..n)
        .filter(|&i| wires[i].is_tag())
        .map(|start| {
            let end = (start + 1..n)
                .take_while(|&i| wires[i].group.is_some())
                .last()
                .map_or(start + 1, |i| i + 1);
            start..end
        })
        .collect();

    // Leaves under a conditional value form are collected per hoist and emitted
    // below as ONE `match` on its `Option` local — the same treatment a sum's
    // groups get, and for the same reason: their slots exist unconditionally
    // but only one arm computes them. Built BEFORE the sum pass, because a
    // conditional form may carry a sum field and that segment has to land in
    // the arm too: emitted ahead of it, its `match` would reach a binding the
    // arm has not introduced yet.
    let mut cond_stmts: std::collections::BTreeMap<usize, TokenStream> = hoists
        .iter()
        .enumerate()
        .filter_map(|(i, _)| {
            wires
                .iter()
                .any(|w| hoisted.conditional(w.reach()).is_some_and(|(j, ..)| j == i))
                .then_some((i, TokenStream::new()))
        })
        .collect();

    for seg in &sum_segments {
        let leaf = &wires[seg.start];
        let (base, base_is_ref, path, _) = rebase(leaf);
        // The value to `match` on. The selector's own path reaches the sum
        // (empty when the sum IS the value), and a step on it MAY be optional:
        // the refusal that used to guarantee otherwise is gone (#220), which is
        // what the gate below exists for.
        //
        // A plain field chain is borrowed DIRECTLY (`&base.a.b`) rather than
        // through the base (`&(&base).a.b`). The two are the same value, but
        // the second borrows the base as a whole, which the borrow checker
        // rejects once a sibling leaf has moved another field out of it — and
        // borrowing this field while sibling fields move is exactly what a
        // consuming value form does.
        let (projected, lead) = project_leading_fields(&base, base_is_ref, &path);
        // An `Option<sum>` field gates the WHOLE segment, not each slot (#220).
        // A sum's leaves are not independent — only one group is live per value
        // — so absence cannot be the per-leaf `null` `reach_leaf` gives an
        // ordinary optional field. It is one tuple bind whose `None` arm carries
        // every slot's default, which is the shape a conditional value form's
        // hoist already emits below; this applies it to an optional step inside
        // the segment's own path.
        let opt_at = (lead..path.len()).find(|&i| path[i].is_optional());
        let (matched, gate) = match opt_at {
            None => (fold_steps(&qualify, &path[lead..], projected, false), None),
            Some(k) => {
                // Through the optional step INCLUSIVE, then the rest off the
                // binding — the same split `reach_leaf` makes, so the borrow in
                // front of it stays the ordinary rule rather than a second
                // statement of it.
                let opt_e = fold_steps(&qualify, &path[lead..=k], projected, false);
                let bind = format_ident!("__sg{}", seg.start);
                // ONE optional step is what the gate below handles. A second
                // one in the tail would compose `match &Option<..>` against bare
                // variant patterns — the E0308 in the consumer's crate that the
                // deleted `builder.rs` assert used to pre-empt by name, so the
                // named diagnostic keeps a home here.
                //
                // `assert!`, not `debug_assert!`: a build script inherits the
                // consumer's profile, so a debug-only check is absent from
                // exactly the release build where a mis-emission costs the most
                // to diagnose. Same rule, same phrasing, as the single-return
                // optional-step assert in `reach_leaf` above.
                //
                // The condition is what actually breaks, not the stronger fact
                // that happens to hold: every sum leaf's path stops AT the sum,
                // so the tail is empty today, but a NON-optional tail composes
                // correctly through `fold_steps` — refusing it would refuse a
                // shape that works.
                assert!(
                    !path[k + 1..].iter().any(PathStep::is_optional),
                    "jnigen unfold: leaf `{}` reaches its sum through TWO optional \
                     steps — the segment gate has one `None` arm, so the second \
                     would be matched as if it were the sum itself",
                    leaf.name,
                );
                // What the `match` binds, asked of the step rather than assumed:
                // the FIELD branch scrutinizes `&Option<_>` (that is what
                // `bind_as_option` is for), so ergonomics binds `&Sum` — a
                // borrow. Only an owned-yielding CALL binds an owned value. The
                // literal `true` disagreed with `reach_leaf`, which passes
                // `false` for its analogous recursion.
                let inner = fold_steps(
                    &qualify,
                    &path[k + 1..],
                    quote!(#bind),
                    path[k].yields_owned(),
                );
                (inner, Some((k, opt_e, bind)))
            }
        };
        let (group_stmts, group_args) = encode_sum_group(
            context,
            &wires[seg.clone()],
            &obj_idents[seg.clone()],
            matched,
            fail,
            emit,
        );
        let group_stmts = match gate {
            None => group_stmts,
            Some((k, opt_e, bind)) => {
                let ids: Vec<&syn::Ident> = obj_idents[seg.clone()].iter().collect();
                let slots: Vec<Slot> = wires[seg.clone()]
                    .iter()
                    .map(|l| leaf_slot(context, l))
                    .collect();
                let tys = slots.iter().map(|s| &s.ty);
                let defaults = slots.iter().map(|s| &s.default);
                // A FIELD step composes to a borrow, so it goes through a
                // coercion site and the destructure stops caring which
                // representation the source spelled the optional as (#268). A
                // CALL yields its own owned value, whose payload downstream may
                // move, so it keeps the direct match — the same division
                // `reach_leaf` makes.
                let (prelude, scrutinee) = if path[k].is_field() {
                    let opt_bind = format_ident!("__so{}", seg.start);
                    (bind_as_option(&opt_e, &opt_bind), quote!(#opt_bind))
                } else {
                    (TokenStream::new(), opt_e)
                };
                quote! {
                    let (#(#ids,)*): (#(#tys,)*) = {
                        #prelude
                        match #scrutinee {
                            ::core::option::Option::Some(#bind) => {
                                #group_stmts
                                (#(#ids,)*)
                            }
                            ::core::option::Option::None => (#(#defaults,)*),
                        }
                    };
                }
            }
        };
        // The whole segment — its slot declarations and its `match` — is
        // routed like any other leaf under the same form.
        match hoisted.conditional(leaf.reach()) {
            Some((i, ..)) => cond_stmts
                .get_mut(&i)
                .expect("a conditional leaf's hoist has a bucket")
                .extend(group_stmts),
            None => stmts.extend(group_stmts),
        }
        for (k, e) in group_args.into_iter().enumerate() {
            arg_exprs[seg.start + k] = e;
        }
    }

    let in_sum = |i: usize| sum_segments.iter().any(|s| s.contains(&i));
    let mut order: Vec<usize> = (0..n)
        .filter(|&i| !wires[i].identity && !in_sum(i))
        .collect();
    order.extend((0..n).filter(|&i| wires[i].identity && !in_sum(i)));

    for idx in order {
        let leaf = &wires[idx];
        let obj_ident = &obj_idents[idx];
        // Route this leaf's statements: into its conditional arm, or straight
        // out. Shadows `stmts` for the rest of the body, so every `extend`
        // below lands in the right place without knowing which case it is in.
        let stmts: &mut TokenStream = match hoisted.conditional(leaf.reach()) {
            Some((i, ..)) => cond_stmts.get_mut(&i).expect("collected above"),
            None => &mut stmts,
        };
        let (value, by_ref, path, consuming) = rebase(leaf);
        let value = &value;
        let (frozen_pipeline, frozen_projection) = match &leaf.abi {
            Some(crate::jni::compile::OutAbi::Value(value)) => {
                (&value.pipeline, value.projection.as_ref())
            }
            Some(crate::jni::compile::OutAbi::Tag) => {
                unreachable!("sum selector segments are encoded above")
            }
            None => panic!(
                "jnigen delivery: leaf `{}` reached Rust rendering without a frozen output ABI",
                leaf.name
            ),
        };
        let projection = frozen_projection;
        let wire = frozen_pipeline.wire().clone();
        let conv_fail = fail(quote!(__e.to_string()));
        let conv = |input: TokenStream| -> TokenStream {
            let call = frozen_pipeline.invoke(input, emit);
            quote! {
                match #call {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        #conv_fail
                    }
                }
            }
        };

        // Bind `obj_ident` to a JObject-yielding `expr`.
        let bind_obj = |obj_ident: &syn::Ident, expr: TokenStream| -> TokenStream {
            quote! {
                let #obj_ident: jni::objects::JObject = #expr;
            }
        };

        if leaf.identity {
            // Identity leaf: deliver the value itself. Its projection decides
            // how: a `ptr_class` Handle is cloned (`&T`, reached by the path)
            // or — at the root of an owned value — moved into a fresh Box,
            // and crosses as the RAW `jlong` (the receiver constructs the
            // typed class in bytecode — a native `new_object` would cost a
            // descriptor parse + FindClass + GetMethodID + NewObjectA per
            // delivery). A nullable handle (an `Option` nesting step on the
            // path) boxes to `java.lang.Long` / null. The whole path is `Option`-unwrapped
            // (`unwrap_last`): an optional nesting step makes the leaf null
            // when the value is absent.
            let proj = projection.unwrap_or_else(|| {
                panic!(
                    "jnigen unfold: identity leaf `{}` has no projection — \
                     `.accessor_record_id()` requires a ptr_class type",
                    leaf.out_ty.key()
                )
            });
            // The place this handle lives, when it is OURS to give away — the
            // owned root, or a field of a CONSUMING value form, which handed its
            // value over so its handle fields move out like every other field
            // rather than being cloned through the borrowed converter (which
            // would also demand a `Clone` the type need not have).
            //
            // Which it is was decided in the plan, not here: an owned `out_ty`
            // IS the statement that this leaf owns what it reaches, and it is
            // what selected the owning converter. `steps_are_movable` then says
            // how to project it — a plain-field run directly, a trailing
            // `Option` through the nullable branch's `match`, which moves the
            // whole `Option` in rather than borrowing it.
            let owned_place: Option<TokenStream> = if !matches!(
                leaf.out_ty.kind(),
                prebindgen_registry::flat::TypeKind::Ref { .. }
            ) && steps_are_movable(&path)
            {
                let segs: Vec<&syn::Ident> = path.iter().map(PathStep::ident).collect();
                Some(quote!(#value #(.#segs)*))
            } else {
                None
            };
            match proj.kind {
                ProjectionKind::Handle => {
                    let handle_ident = format_ident!("__h{}", idx);
                    if let (Some(place), false) = (&owned_place, leaf.nullable) {
                        // Ours, and always present: move into a Box, raw jlong.
                        stmts.extend(quote! {
                            let #obj_ident: jni::sys::jvalue = jni::sys::jvalue {
                                j: std::boxed::Box::into_raw(std::boxed::Box::new(#place))
                                    as jni::sys::jlong,
                            };
                        });
                    } else if let (Some(place), false) =
                        (&owned_place, path.last().is_some_and(PathStep::is_optional))
                    {
                        // Ours, always present, but the SLOT is nullable — the
                        // leaf hangs off a conditional value form, so the absent
                        // case is the enclosing `match`'s other arm, not an
                        // `Option` here. Move into the Box and box the jlong, so
                        // both arms fill the slot with the same shape.
                        let box_fail = fail(quote!(__e.to_string()));
                        stmts.extend(bind_obj(
                            obj_ident,
                            quote! {{
                                let #handle_ident: jni::sys::jlong =
                                    std::boxed::Box::into_raw(std::boxed::Box::new(#place))
                                        as jni::sys::jlong;
                                match ::prebindgen_jni_runtime::box_jlong(&mut env, #handle_ident) {
                                    ::core::result::Result::Ok(__o) => __o,
                                    ::core::result::Result::Err(__e) => {
                                        #box_fail
                                    }
                                }
                            }},
                        ));
                    } else if let Some(place) = &owned_place {
                        // Ours, behind an `Option`: match the option BY VALUE so
                        // the present handle is moved into its Box, boxed
                        // `java.lang.Long` when present / JVM null when absent.
                        // Matching `&place` here is what used to clone it back
                        // through the borrowed converter.
                        let box_fail = fail(quote!(__e.to_string()));
                        stmts.extend(bind_obj(
                            obj_ident,
                            quote! {{
                                match #place {
                                    ::core::option::Option::Some(__n) => {
                                        let #handle_ident: jni::sys::jlong =
                                            std::boxed::Box::into_raw(std::boxed::Box::new(__n))
                                                as jni::sys::jlong;
                                        match ::prebindgen_jni_runtime::box_jlong(&mut env, #handle_ident) {
                                            ::core::result::Result::Ok(__o) => __o,
                                            ::core::result::Result::Err(__e) => {
                                                #box_fail
                                            }
                                        }
                                    }
                                    ::core::option::Option::None => jni::objects::JObject::null(),
                                }
                            }},
                        ));
                    } else if !leaf.nullable {
                        // Reached non-null handle: clone via the converter,
                        // raw jlong (no Option steps on the path).
                        let expr = reach_leaf(
                            &qualify,
                            &path,
                            value.clone(),
                            by_ref,
                            true,
                            0,
                            &|reached| {
                                let __encoded = conv(quote!(#reached));
                                quote! {{
                                    let #handle_ident: jni::sys::jlong = #__encoded;
                                    jni::sys::jvalue { j: #handle_ident }
                                }}
                            },
                        );
                        stmts.extend(quote! {
                            let #obj_ident: jni::sys::jvalue = #expr;
                        });
                    } else {
                        // Nullable handle (Option nesting step): boxed
                        // `java.lang.Long` when present (cached valueOf),
                        // JVM null when absent — matching the `Long?` param.
                        let box_fail = fail(quote!(__e.to_string()));
                        let expr = reach_leaf(
                            &qualify,
                            &path,
                            value.clone(),
                            by_ref,
                            true,
                            0,
                            &|reached| {
                                let __encoded = conv(quote!(#reached));
                                quote! {{
                                    let #handle_ident: jni::sys::jlong = #__encoded;
                                    match ::prebindgen_jni_runtime::box_jlong(&mut env, #handle_ident) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            #box_fail
                                        }
                                    }
                                }}
                            },
                        );
                        stmts.extend(bind_obj(obj_ident, expr));
                    }
                }
                ProjectionKind::Unsigned64 => {
                    let enc_ident = format_ident!("__enc{}", idx);
                    let encode = |reached: TokenStream| {
                        let encoded = conv(reached);
                        quote! {{
                            let #enc_ident: jni::sys::jlong = #encoded;
                            jni::sys::jvalue { j: #enc_ident }
                        }}
                    };
                    if path.is_empty() && !by_ref {
                        let expr = encode(value.clone());
                        stmts.extend(quote! { let #obj_ident: jni::sys::jvalue = #expr; });
                    } else if !leaf.nullable {
                        let expr = reach_leaf(
                            &qualify,
                            &path,
                            value.clone(),
                            by_ref,
                            true,
                            0,
                            &|reached| encode(quote!(*#reached)),
                        );
                        stmts.extend(quote! { let #obj_ident: jni::sys::jvalue = #expr; });
                    } else {
                        let box_fail = fail(quote!(__e.to_string()));
                        let expr = reach_leaf(
                            &qualify,
                            &path,
                            value.clone(),
                            by_ref,
                            true,
                            0,
                            &|reached| {
                                let __encoded = conv(quote!(*#reached));
                                quote! {{
                                    let #enc_ident: jni::sys::jlong = #__encoded;
                                    match ::prebindgen_jni_runtime::box_jlong(&mut env, #enc_ident) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            #box_fail
                                        }
                                    }
                                }}
                            },
                        );
                        stmts.extend(bind_obj(obj_ident, expr));
                    }
                }
            }
            continue;
        }

        // Leaf reach. An `Accessor` leaf walks its accessor-fn path — unwrapping
        // every `Option` nesting step (`None` ⇒ a null leaf). A `Field` leaf
        // (synthesized `data_class`) reaches a struct field and clones it
        // (`value.a.b.clone()`); the converter (`Option<Box<String>>` → nullable
        // String, …) carries any nullability, so there is no path `Option` to
        // unwrap. `reach(body)` dispatches on the source and feeds the reached
        // Rust expression to `body`.
        //
        // Which of the two it is, asked of the wire: a field read ends its
        // reach in a field step, an accessor's ends at the call. That is what
        // `LeafSource` said, and saying it off the reach keeps one answer.
        let reach = |body: &dyn Fn(TokenStream) -> TokenStream| -> TokenStream {
            match leaf.is_field_read() {
                false => reach_leaf(&qualify, &path, value.clone(), by_ref, false, 0, body),
                // Under a CONSUMING value form the leaf owns its field, so it
                // is **moved** out; the whole point of consuming is that this
                // clone disappears. Each field is read by exactly one leaf, so
                // the partial moves are disjoint — but nothing may then borrow
                // the local as a whole, which is why the reach below projects
                // the field directly rather than through `&(&local)`.
                true if path.iter().all(PathStep::is_plain_field) => {
                    let segs: Vec<&syn::Ident> = path.iter().map(PathStep::ident).collect();
                    match consuming {
                        true => body(quote!(#value #(.#segs)*)),
                        false => body(quote!(#value #(.#segs)*.clone())),
                    }
                }
                // A `.fields()` leaf reaches its field through the value-form
                // accessor, so the path can be mixed: compose it like an
                // accessor leaf — the final step composes directly, since the
                // converter takes the field type as written (`Option` and all)
                // — and clone the reached borrow, which is what a field leaf
                // delivers.
                true => reach_leaf(
                    &qualify,
                    &path,
                    value.clone(),
                    by_ref,
                    false,
                    0,
                    &|reached| body(quote!((#reached).clone())),
                ),
            }
        };

        // A non-null primitive-wire leaf delivers its raw primitive as a typed
        // `jvalue` — no boxing, no JNI call at all (the typed `run` descriptor
        // declares the primitive). Everything else (object wires, and nullable
        // leaves whose `None` arm must yield a JVM null) encodes the reached
        // value with the leaf's output converter and casts to JObject.
        let enc_ident = format_ident!("__enc{}", idx);
        if context.leaf_is_prim(leaf) {
            let letter = jni_field_access(&wire)
                .expect("leaf_is_prim guarantees a primitive wire")
                .1;
            let expr = reach(&|reached| {
                let __encoded = conv(quote!(#reached));
                quote! {{
                    let #enc_ident = #__encoded;
                    jni::sys::jvalue { #letter: #enc_ident }
                }}
            });
            stmts.extend(quote! {
                let #obj_ident: jni::sys::jvalue = #expr;
            });
            continue;
        }
        let cast = cast_wire_to_jobject(&enc_ident, &wire, fail);
        let expr = reach(&|reached| {
            let __encoded = conv(quote!(#reached));
            quote! {{
                let #enc_ident = #__encoded;
                #cast
            }}
        });
        stmts.extend(bind_obj(obj_ident, expr));
    }

    // One `match` per conditional value form: the `Some` arm runs the leaves
    // that hang off it and yields their locals as a tuple; the `None` arm
    // yields the same wire defaults an inert sum group carries. Binding the
    // tuple outside the match is what keeps the leaves' locals in scope for the
    // argument expressions, which are indifferent to how the slot was filled.
    for (i, body) in cond_stmts {
        let local = hoisted.local(i);
        let bind = format_ident!("__u{}", i);
        let idxs: Vec<usize> = (0..n)
            .filter(|&k| {
                hoisted
                    .conditional(wires[k].reach())
                    .is_some_and(|(j, ..)| j == i)
            })
            .collect();
        let ids: Vec<&syn::Ident> = idxs.iter().map(|&k| &obj_idents[k]).collect();
        let tys = idxs.iter().map(|&k| leaf_slot(context, &wires[k]).ty);
        let defaults = idxs.iter().map(|&k| leaf_slot(context, &wires[k]).default);
        // Matched BY VALUE: the local is this arm's alone (every leaf under the
        // hoist is in it), so a consuming value form's fields move out here
        // exactly as they do at an unconditional one.
        stmts.extend(quote! {
            let (#(#ids,)*): (#(#tys,)*) = match #local {
                ::core::option::Option::Some(#bind) => { #body (#(#ids,)*) }
                ::core::option::Option::None => (#(#defaults,)*),
            };
        });
    }
    (stmts, arg_exprs, None)
}

fn frozen_leaf_wire(leaf: &crate::jni::compile::OutWire) -> syn::Type {
    match &leaf.abi {
        Some(crate::jni::compile::OutAbi::Tag) => syn::parse_quote!(jni::sys::jint),
        Some(crate::jni::compile::OutAbi::Value(value)) => value.pipeline.wire().clone(),
        None => panic!("frozen JNI delivery leaf `{}` has no output ABI", leaf.name),
    }
}

/// True when a plan leaf crosses the typed `run` as a **raw primitive**
/// `jvalue`: non-nullable, no projection (not a handle), and a
/// primitive JNI wire. Must agree with the descriptor chunk
/// [`crate::jni::iface`] derives for the same leaf — a
/// nullable primitive boxes (object chunk), object wires pass as objects.
///
/// The synthesized sum selector is a `jint` by definition — it is assigned,
/// never converted — unless it is NULLABLE: the sum then sits under a
/// conditional value form, and the absent case needs a representation the
/// tag's own variants do not provide. A raw `jint` has none (zero is a real
/// variant), so the selector boxes like any other nullable leaf.
fn frozen_leaf_is_prim(leaf: &crate::jni::compile::OutWire) -> bool {
    if leaf.is_tag() {
        return !leaf.nullable;
    }
    if leaf.nullable {
        return false;
    }
    let Some(crate::jni::compile::OutAbi::Value(value)) = &leaf.abi else {
        panic!("frozen JNI delivery leaf `{}` has no output ABI", leaf.name)
    };
    let proj_ok = match &value.projection {
        None => true,
        Some(projection) => matches!(
            projection.kind,
            ProjectionKind::Handle | ProjectionKind::Unsigned64
        ),
    };
    proj_ok && matches!(jni_field_access(value.pipeline.wire()), Some((_, _, false)))
}

/// The wire half of [`leaf_is_prim`]: does a leaf of this type occupy a **raw
/// primitive** slot? Split out so the interface derivation can ask the question
/// about a leaf whose own `nullable` flag it is in the middle of computing (an
/// inert sum group slot).
pub(crate) fn leaf_ty_is_prim(
    ext: &Declarations,
    out_ty: &prebindgen_registry::flat::TypeRef,
) -> bool {
    let Some(entry) = ext.out_frag(out_ty) else {
        return false;
    };
    // No projection (plain primitive/enum wire) — or an opaque HANDLE, whose
    // converter's wire is the raw `jlong` the typed `run` declares as `Long`
    // (`J`): the receiver constructs the typed class in bytecode. A nullable
    // handle boxes to `java.lang.Long` instead (object chunk). Fixed-size
    // arrays and other object wires stay objects (for example, `[B`).
    let proj_ok = match &entry.metadata.projection {
        None => true,
        Some(p) => matches!(p.kind, ProjectionKind::Handle | ProjectionKind::Unsigned64),
    };
    proj_ok && matches!(jni_field_access(&entry.destination), Some((_, _, false)))
}

#[cfg(test)]
mod tests {
    use prebindgen_registry::unfold::{LeafSource, UnfoldLeaf};

    use super::*;
    /// A `TypeRef` through the model. `Flat::classify` is sealed to `api::core`
    /// (#280), so a test under `api::lang` asks the sanctioned probe helper
    /// rather than reaching around the seal — which is the seal working.
    use crate::test_util::reading as tref;

    /// A leaf as the resolver builds one. `source` is not decoration: it
    /// decides the terminal treatment (a `Field` leaf is CLONED out of the
    /// place it reached), and production pairs it with the path shape —
    /// `Accessor` for identity leaves and accessor chains (`unfold.rs`'s
    /// `DeconRecord::Identity` arm), `Field` only for the synthesized
    /// by-value `data_class` decomposition, whose paths are field idents and
    /// never calls. A fixture that mixed them would exercise a leaf the
    /// resolver cannot produce.
    fn leaf(
        out_ty: syn::Type,
        path: Vec<PathStep>,
        identity: bool,
        source: LeafSource,
    ) -> UnfoldLeaf {
        UnfoldLeaf {
            name: "probe".to_string(),
            path,
            out_ty: tref(out_ty),
            identity,
            nullable: false,
            source,
            group: None,
        }
    }

    fn qualify(id: &syn::Ident) -> syn::Path {
        syn::parse_quote!(myflat::#id)
    }

    /// `reach_leaf_flat` projects the place `steps_are_movable` says is movable
    /// — the two are one rule, not two readings of one.
    ///
    /// The trailing-optional path is the case that discriminates: it IS movable
    /// (a `None` arm still hands the whole `Option` over by value), and the
    /// `all(is_plain_field)` restatement this replaced called it not-movable.
    /// Where the plan had already granted an owned `out_ty` on the strength of
    /// `steps_are_movable`, that disagreement is a borrow reaching an owning
    /// converter — `plan.rs`'s stated hazard, and PR#221's P1.
    #[test]
    fn a_movable_place_is_projected_as_a_move() {
        for path in [
            vec![PathStep::field(syn::parse_quote!(a), false)],
            vec![
                PathStep::field(syn::parse_quote!(a), false),
                PathStep::field(syn::parse_quote!(b), false),
            ],
            // Movable by `steps_are_movable`; NOT by `all(is_plain_field)`.
            vec![
                PathStep::field(syn::parse_quote!(a), false),
                PathStep::field(syn::parse_quote!(b), true),
            ],
        ] {
            assert!(steps_are_movable(&path), "fixture must be movable");
            let l = leaf(
                syn::parse_quote!(Owned),
                path.clone(),
                true,
                LeafSource::Reach,
            );
            let got = prebindgen_registry::unfold::reach_leaf_flat(
                &qualify,
                &crate::jni::compile::OutWire::from_leaf(&l),
                &path,
                quote!(__src),
                false,
                false,
            )
            .to_string();
            assert!(
                !got.contains('&') && !got.contains("clone"),
                "a movable place is moved, not borrowed or cloned — got `{got}`"
            );
        }
    }

    /// A borrow stays a borrow: an identity leaf whose `out_ty` is a reference
    /// did not own what it reached, whatever the path shape says.
    #[test]
    fn a_borrowed_out_ty_is_never_moved() {
        let path = vec![PathStep::field(syn::parse_quote!(a), false)];
        let l = leaf(
            syn::parse_quote!(&Owned),
            path.clone(),
            true,
            LeafSource::Reach,
        );
        let got = prebindgen_registry::unfold::reach_leaf_flat(
            &qualify,
            &crate::jni::compile::OutWire::from_leaf(&l),
            &path,
            quote!(__src),
            false,
            false,
        )
        .to_string();
        assert!(
            got.contains('&'),
            "a borrowed out_ty keeps its borrow — got `{got}`"
        );
    }

    /// A `Field` leaf is cloned out of the place it reached, whatever the path
    /// shape says — its converter takes the field type as written.
    ///
    /// The counterpart of the move above, and what keeps `source` load-bearing
    /// in these fixtures rather than a value they all happen to share.
    #[test]
    fn a_field_leaf_is_cloned_out_of_its_place() {
        let path = vec![
            PathStep::field(syn::parse_quote!(a), false),
            PathStep::field(syn::parse_quote!(b), false),
        ];
        let l = leaf(
            syn::parse_quote!(Owned),
            path.clone(),
            false,
            LeafSource::Reach,
        );
        let got = prebindgen_registry::unfold::reach_leaf_flat(
            &qualify,
            &crate::jni::compile::OutWire::from_leaf(&l),
            &path,
            quote!(__src),
            false,
            false,
        )
        .to_string();
        assert!(
            got.contains("clone"),
            "a non-consuming field leaf clones rather than moves — got `{got}`"
        );
    }

    /// The optional-step guard asks the LEAF's path, not the caller's slice.
    ///
    /// `wrapper.rs` rebases onto a hoisted local and hands over the remaining
    /// suffix, so checking the parameter would pass exactly when the hoist is
    /// the conditional one — an `Option<T>` local with a field read hung off it,
    /// which cannot compose. Passing the suffix here mimics that rebase.
    #[test]
    #[should_panic(expected = "which has no `None` arm")]
    fn an_optional_step_in_a_stripped_prefix_is_still_refused() {
        let full = vec![
            PathStep::call(syn::parse_quote!(get_it), true, false),
            PathStep::field(syn::parse_quote!(a), false),
        ];
        let l = leaf(syn::parse_quote!(Owned), full, false, LeafSource::Reach);
        // The suffix a rebase would hand over — the optional call is gone from
        // it, and used to take the guard with it.
        let rest = vec![PathStep::field(syn::parse_quote!(a), false)];
        let _ = prebindgen_registry::unfold::reach_leaf_flat(
            &qualify,
            &crate::jni::compile::OutWire::from_leaf(&l),
            &rest,
            quote!(__vf0),
            false,
            false,
        );
    }
}
