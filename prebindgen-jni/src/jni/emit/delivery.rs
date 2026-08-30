//! Output-expansion delivery: unfold plans and leaf encoding.
//!
//! # What this module walks
//!
//! Nothing. The decomposition walk is the registry's: binding the hoists,
//! deciding where a leaf's reach starts, gating each optional step, saying
//! what is moved and what is cloned, recognising a sum's segment and gating
//! it, and the `match` a conditional value form's leaves share.
//!
//! What is here is the target-language half — the [`DeliveryBridge`]
//! implementation for [`FrozenDelivery`]: encoding a leaf, placing it as a
//! call argument, what a slot holds when it is not filled, and what absence
//! looks like — together with `jvalue` layout, local-frame sizing, cached
//! method lookup and exception routing, which are JNI policy and stay.
//!
//! [`DeliveryBridge`]: prebindgen_registry::unfold::DeliveryBridge

use prebindgen_registry::{
    unfold::{bind_hoists, DeliveryBridge, PathStep},
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
        let mut delivered = Delivered::planned(plan, (*output.wires).clone(), output.chain.clone());
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
    /// Which call the encoded slots ride: a typed builder `run` taking
    /// `jvalue`s, or a `fromParts` factory spelled with a signature string and
    /// typed `JValue`s. The walk is the same either way — what differs is how a
    /// primitive slot is held and handed over.
    factory: bool,
    /// Per-leaf JVM descriptors, by leaf name. Filled for the factory
    /// convention only, from the same derivation the builder interface uses.
    descriptors: std::collections::BTreeMap<String, String>,
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
        Self::build(
            ext,
            registry,
            plan.hoists.clone(),
            plan.by_ref,
            plan.fixed_builder,
            plan.is_optional_base(),
            wires,
            chain,
        )
    }

    /// The construction both conventions share: origin qualification for every
    /// call step, and the sum model behind every selector. What a delivery
    /// needs to render without asking the registry anything.
    #[allow(clippy::too_many_arguments)]
    fn build(
        ext: &Declarations,
        registry: &impl Conversions,
        hoists: Vec<prebindgen_registry::unfold::Hoist>,
        by_ref: bool,
        fixed_product: bool,
        optional: bool,
        wires: Vec<crate::jni::compile::OutWire>,
        chain: Option<crate::jni::compile::ComposedChain>,
    ) -> Self {
        let mut modules = std::collections::BTreeMap::new();
        for step in hoists
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
            hoists,
            by_ref,
            fixed_product,
            optional,
            chain,
            modules,
            sums,
            factory: false,
            descriptors: std::collections::BTreeMap::new(),
        }
    }

    /// The delivery a **whole-object struct encode** renders through: the
    /// struct's own frozen decomposition, handed to its Kotlin `fromParts`.
    ///
    /// No hoists and no chain — a data class's leaves are read straight off the
    /// value, which arrives as `&Source`. What makes it a delivery at all is
    /// that the leaves and the walk are the same ones a fixed-builder site
    /// uses; only the call at the end differs.
    pub(crate) fn for_value_struct(
        ext: &Declarations,
        registry: &impl Conversions,
        wires: Vec<crate::jni::compile::OutWire>,
    ) -> Option<Self> {
        let descriptors = crate::jni::iface::leaf_descriptors(ext, &wires)?;
        let mut frozen = Self::build(ext, registry, Vec::new(), true, true, false, wires, None);
        frozen.factory = true;
        frozen.descriptors = frozen
            .wires
            .iter()
            .map(|wire| wire.name.clone())
            .zip(descriptors)
            .collect();
        Some(frozen)
    }

    /// Whether the encoded slots ride a `fromParts` factory rather than a
    /// builder's typed `run`.
    pub(crate) fn is_factory(&self) -> bool {
        self.factory
    }

    /// How a converted **primitive** sits in its slot: the value itself under
    /// the factory convention, wrapped in the `jvalue` member the descriptor
    /// names under the builder's.
    pub(crate) fn hold_prim(&self, letter: &syn::Ident, value: TokenStream) -> TokenStream {
        match self.factory {
            true => value,
            false => quote!(jni::sys::jvalue { #letter: #value }),
        }
    }

    /// A **selector**'s value in the form its slot holds — a tag's alternative
    /// number, a presence flag's `1`.
    ///
    /// Asked here rather than spelled by the segment emitters, which build
    /// these two slots themselves instead of through [`DeliveryBridge::encode`]
    /// (a selector is assigned, not converted) and so would otherwise state the
    /// slot convention a second time.
    pub(crate) fn selector_value(
        &self,
        leaf: &crate::jni::compile::OutWire,
        value: TokenStream,
    ) -> TokenStream {
        if self.factory {
            return value;
        }
        let letter = match leaf.is_tag() {
            true => format_ident!("i"),
            false => format_ident!("z"),
        };
        self.hold_prim(&letter, value)
    }

    /// The JVM descriptor a non-primitive leaf's slot occupies, frozen with the
    /// leaves so rendering asks nothing. Empty for the builder convention,
    /// which resolves its method id from the interface descriptor instead.
    pub(crate) fn object_descriptor(&self, leaf: &crate::jni::compile::OutWire) -> String {
        match self.factory {
            false => String::new(),
            true => self
                .descriptors
                .get(&leaf.name)
                .unwrap_or_else(|| {
                    panic!(
                        "frozen JNI factory has no descriptor for leaf `{}`",
                        leaf.name
                    )
                })
                .clone(),
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

    /// The JVM signature of the `fromParts` factory these leaves feed —
    /// each slot's descriptor in order, returning the class itself.
    pub(crate) fn factory_signature(&self, java_class_name: &str) -> String {
        let mut sig = String::from("(");
        for wire in &self.wires {
            sig.push_str(&crate::jni::emit::sum_out::leaf_slot(self, wire).descriptor);
        }
        sig.push_str(&format!(")L{java_class_name};"));
        sig
    }
}

impl FrozenDelivery {
    /// Whether this leaf crosses the typed `run` as a raw primitive rather
    /// than as an object. JNI's own question — which jvalue member a slot
    /// carries — so it stays off the bridge.
    pub(crate) fn leaf_is_prim(&self, leaf: &crate::jni::compile::OutWire) -> bool {
        frozen_leaf_is_prim(leaf)
    }

    /// The Rust type this leaf's converter produces, which is what its jvalue
    /// member and its slot default are read from.
    pub(crate) fn leaf_wire(&self, leaf: &crate::jni::compile::OutWire) -> syn::Type {
        frozen_leaf_wire(leaf)
    }
}

impl prebindgen_registry::unfold::DeliveryBridge for FrozenDelivery {
    type Leaf = crate::jni::compile::OutWire;

    fn qualify(&self, ident: &syn::Ident) -> syn::Path {
        self.modules
            .get(&ident.to_string())
            .unwrap_or_else(|| panic!("frozen JNI delivery has no origin for `{ident}`"))
            .clone()
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

    /// A leaf's own encode: run its output converter on the reached Rust
    /// value, then present the result the way its slot carries it. A
    /// primitive-wire leaf is a typed `jvalue` and crosses with no JNI call at
    /// all; every other leaf becomes a `JObject`, boxing where the typed `run`
    /// descriptor declares an object.
    ///
    /// Both forms encode INSIDE `reach`, so an absent value on the way to the
    /// leaf yields this adapter's absence — a JVM null — rather than a value
    /// the walk invented.
    fn encode(
        &self,
        leaf: &crate::jni::compile::OutWire,
        index: usize,
        slot: &syn::Ident,
        reach: &prebindgen_registry::unfold::Reach<'_>,
        fail: &dyn Fn(TokenStream) -> TokenStream,
        emit: &prebindgen_registry::RustWriter,
    ) -> TokenStream {
        let pipeline = match &leaf.abi {
            Some(crate::jni::compile::OutAbi::Value(value)) => &value.pipeline,
            Some(crate::jni::compile::OutAbi::Tag) | Some(crate::jni::compile::OutAbi::Present) => {
                unreachable!("a selector is encoded with its own segment")
            }
            None => panic!(
                "jnigen delivery: leaf `{}` reached Rust rendering without a frozen output ABI",
                leaf.name
            ),
        };
        let wire = pipeline.wire().clone();
        let conv_fail = fail(quote!(__e.to_string()));
        let convert = |input: TokenStream| -> TokenStream {
            let call = pipeline.invoke(input, emit);
            quote! {
                match #call {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        #conv_fail
                    }
                }
            }
        };
        let encoded = format_ident!("__enc{}", index);
        if self.leaf_is_prim(leaf) {
            let letter = jni_field_access(&wire)
                .expect("leaf_is_prim guarantees a primitive wire")
                .1;
            let held = self.hold_prim(&letter, quote!(#encoded));
            let expr = reach(&|reached| {
                let converted = convert(quote!(#reached));
                quote! {{
                    let #encoded = #converted;
                    #held
                }}
            });
            let ty = crate::jni::emit::sum_out::leaf_slot(self, leaf).ty;
            return quote! { let #slot: #ty = #expr; };
        }
        let cast = cast_wire_to_jobject(&encoded, &wire, fail);
        let expr = reach(&|reached| {
            let converted = convert(quote!(#reached));
            quote! {{
                let #encoded = #converted;
                #cast
            }}
        });
        quote! { let #slot: jni::objects::JObject = #expr; }
    }

    /// A slot is a `jvalue` for a primitive-wire leaf and a `JObject`
    /// otherwise, and an unfilled one carries that shape's own empty value: a
    /// zero of the right jvalue member, or a JVM null.
    fn slot(&self, leaf: &crate::jni::compile::OutWire) -> prebindgen_registry::unfold::Slot {
        let slot = crate::jni::emit::sum_out::leaf_slot(self, leaf);
        prebindgen_registry::unfold::Slot {
            ty: slot.ty,
            default: slot.default,
        }
    }

    /// A leaf whose value was not there delivers a JVM null. Every slot that
    /// can be absent is an object slot for exactly that reason: a primitive
    /// one has no null to carry.
    fn absent(&self) -> TokenStream {
        quote!(jni::objects::JObject::null())
    }

    /// How a filled slot rides the typed `run` call: a primitive-wire leaf IS
    /// its jvalue, and every other leaf's `JObject` passes its raw pointer in
    /// the `l` slot. Matches the descriptor [`crate::jni::iface`] derives for
    /// the same leaf.
    fn argument(&self, leaf: &crate::jni::compile::OutWire, slot: &syn::Ident) -> TokenStream {
        match (self.factory, self.leaf_is_prim(leaf)) {
            // A `fromParts` call is spelled with a signature string, so its
            // arguments are typed `JValue`s: the wire value itself for a
            // primitive, a borrow of the object otherwise.
            (true, true) => quote!(jni::objects::JValue::from(#slot)),
            (true, false) => quote!(jni::objects::JValue::Object(&#slot)),
            (false, true) => quote!(#slot),
            (false, false) => quote!(jni::sys::jvalue { l: #slot.as_raw() }),
        }
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
    context: &FrozenDelivery,
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

    // The argument expression per leaf, in leaf order — how a filled slot
    // rides the call, which is the bridge's answer rather than this loop's.
    let mut arg_exprs: Vec<TokenStream> = Vec::with_capacity(n);
    for (idx, leaf) in wires.iter().enumerate() {
        arg_exprs.push(context.argument(leaf, &obj_idents[idx]));
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

    // Where each leaf's reach starts is the walk's answer, not this loop's:
    // `Hoisted::place` decides which value form a leaf sits under, what is
    // left of its path, and whether that form gave its value away.
    let place = |leaf: &crate::jni::compile::OutWire| hoisted.place(leaf, value, by_ref);

    // Which leaves form a segment is the plan's own answer, read by the
    // walk: a selector plus the group leaves after it.
    let sum_segments = prebindgen_registry::unfold::segments(&wires);

    // Leaves under a conditional value form are collected per hoist and emitted
    // below as ONE `match` on its `Option` local — the same treatment a sum's
    // groups get, and for the same reason: their slots exist unconditionally
    // but only one arm computes them. Built BEFORE the sum pass, because a
    // conditional form may carry a sum field and that segment has to land in
    // the arm too: emitted ahead of it, its `match` would reach a binding the
    // arm has not introduced yet.
    // Which arm a leaf belongs in is its place's answer, asked here the same
    // way every other consumer asks it — a bucket exists exactly where some
    // leaf reports one.
    let mut cond_stmts: std::collections::BTreeMap<usize, TokenStream> = wires
        .iter()
        .filter_map(|leaf| place(leaf).conditional)
        .map(|i| (i, TokenStream::new()))
        .collect();

    for seg in &sum_segments {
        let at = place(&wires[seg.start]);
        // The segment's own encode — which slot carries which alternative's
        // value, and what the selector is set to — is this adapter's. Reaching
        // the sum it is encoded from, and gating the whole segment when that
        // reach passes through an optional step, is the walk's.
        let group_args = std::cell::RefCell::new(Vec::new());
        let group_stmts = prebindgen_registry::unfold::segment(
            context,
            &qualify,
            &at,
            seg.start,
            &wires[seg.clone()],
            &obj_idents[seg.clone()],
            &|matched| {
                // A segment is selected by a tag or by a presence flag, and
                // the two fill their group differently: one alternative's
                // payload, or the child's own leaves off the value the gate
                // unwrapped.
                let (stmts, args) = crate::jni::emit::encode_segment_group(
                    context,
                    &wires[seg.clone()],
                    &obj_idents[seg.clone()],
                    matched,
                    seg.start,
                    &qualify,
                    fail,
                    emit,
                );
                *group_args.borrow_mut() = args;
                stmts
            },
        );
        // The whole segment — its slot declarations and its `match` — is
        // routed like any other leaf under the same form.
        match at.conditional {
            Some(i) => cond_stmts
                .get_mut(&i)
                .expect("a conditional leaf's hoist has a bucket")
                .extend(group_stmts),
            None => stmts.extend(group_stmts),
        }
        let group_args = group_args.into_inner();
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
        let at = place(leaf);
        // Route this leaf's statements: into its conditional arm, or straight
        // out. The arm is the place's own answer. Shadows `stmts` for the rest
        // of the body, so every `extend` below lands in the right place
        // without knowing which case it is in.
        let stmts: &mut TokenStream = match at.conditional {
            Some(i) => cond_stmts.get_mut(&i).expect("collected above"),
            None => &mut stmts,
        };
        let owned_place = at.owned(leaf);
        let (value, by_ref, path, consuming) = (&at.base, at.base_is_ref, &at.path, at.consuming);
        // Every reach below is the registry's, with this adapter's absence in
        // its `None` arms. The terminal treatment — move, clone or borrow —
        // comes with it, which is what let the three-way dispatch this loop
        // used to make disappear.
        let absent = || DeliveryBridge::absent(context);
        let gated = |path: &[PathStep],
                     base: TokenStream,
                     base_is_ref: bool,
                     unwrap_last: bool,
                     body: &dyn Fn(TokenStream) -> TokenStream| {
            prebindgen_registry::unfold::reach_leaf(
                &qualify,
                prebindgen_registry::unfold::LeafAt {
                    leaf,
                    path,
                    base,
                    base_is_ref,
                    consuming,
                    unwrap_last,
                },
                Some(&absent),
                body,
            )
        };
        let (frozen_pipeline, frozen_projection) = match &leaf.abi {
            Some(crate::jni::compile::OutAbi::Value(value)) => {
                (&value.pipeline, value.projection.as_ref())
            }
            Some(crate::jni::compile::OutAbi::Tag) | Some(crate::jni::compile::OutAbi::Present) => {
                unreachable!("selector segments are encoded above")
            }
            None => panic!(
                "jnigen delivery: leaf `{}` reached Rust rendering without a frozen output ABI",
                leaf.name
            ),
        };
        let projection = frozen_projection;
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
            // `owned_place` — bound above from `LeafPlace::owned` — is the
            // place this handle lives when it is OURS to give away: the owned
            // root, or a field of a CONSUMING value form, which handed its
            // value over so its handle fields move out like every other field
            // rather than being cloned through the borrowed converter (which
            // would also demand a `Clone` the type need not have). Which it is
            // was decided in the plan and is read there, not restated here.
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
                        let expr = gated(path, value.clone(), by_ref, true, &|reached| {
                            let __encoded = conv(quote!(#reached));
                            quote! {{
                                let #handle_ident: jni::sys::jlong = #__encoded;
                                jni::sys::jvalue { j: #handle_ident }
                            }}
                        });
                        stmts.extend(quote! {
                            let #obj_ident: jni::sys::jvalue = #expr;
                        });
                    } else {
                        // Nullable handle (Option nesting step): boxed
                        // `java.lang.Long` when present (cached valueOf),
                        // JVM null when absent — matching the `Long?` param.
                        let box_fail = fail(quote!(__e.to_string()));
                        let expr = gated(path, value.clone(), by_ref, true, &|reached| {
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
                        });
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
                        let expr = gated(path, value.clone(), by_ref, true, &|reached| {
                            encode(quote!(*#reached))
                        });
                        stmts.extend(quote! { let #obj_ident: jni::sys::jvalue = #expr; });
                    } else {
                        let box_fail = fail(quote!(__e.to_string()));
                        let expr = gated(path, value.clone(), by_ref, true, &|reached| {
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
                        });
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
        // A non-identity leaf's converter takes the final step's full type,
        // `Option` included, so the last step is not unwrapped here.
        let reach = |body: &dyn Fn(TokenStream) -> TokenStream| -> TokenStream {
            gated(path, value.clone(), by_ref, false, body)
        };

        // The encode itself is the bridge's: this loop hands it the leaf's
        // reach and it says what the slot holds. Reaching is still decided
        // here — which is what the steps after this one move.
        stmts.extend(context.encode(leaf, idx, obj_ident, &reach, fail, emit));
    }

    // Each conditional value form's leaves share one `match` on its `Option`
    // local — the walk's shape, filled with this adapter's slots.
    for (i, body) in cond_stmts {
        stmts.extend(prebindgen_registry::unfold::conditional_arm(
            context, &hoisted, i, &wires, obj_idents, body,
        ));
    }
    (stmts, arg_exprs, None)
}

fn frozen_leaf_wire(leaf: &crate::jni::compile::OutWire) -> syn::Type {
    match &leaf.abi {
        Some(crate::jni::compile::OutAbi::Tag) => syn::parse_quote!(jni::sys::jint),
        Some(crate::jni::compile::OutAbi::Present) => syn::parse_quote!(jni::sys::jboolean),
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
    // A presence flag is a `jboolean` the gate assigns — there is nothing to
    // box and no value behind it that could be absent.
    if matches!(leaf.abi, Some(crate::jni::compile::OutAbi::Present)) {
        return true;
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
            groups: Vec::new(),
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
            assert!(
                prebindgen_registry::unfold::steps_are_movable(&path),
                "fixture must be movable"
            );
            let l = leaf(
                syn::parse_quote!(Owned),
                path.clone(),
                true,
                LeafSource::Reach,
            );
            let got = prebindgen_registry::unfold::reach_leaf(
                &qualify,
                prebindgen_registry::unfold::LeafAt {
                    leaf: &crate::jni::compile::OutWire::from_leaf(&l),
                    path: &path,
                    base: quote!(__src),
                    base_is_ref: false,
                    consuming: false,
                    unwrap_last: false,
                },
                None,
                &|reached| reached,
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
        let got = prebindgen_registry::unfold::reach_leaf(
            &qualify,
            prebindgen_registry::unfold::LeafAt {
                leaf: &crate::jni::compile::OutWire::from_leaf(&l),
                path: &path,
                base: quote!(__src),
                base_is_ref: false,
                consuming: false,
                unwrap_last: false,
            },
            None,
            &|reached| reached,
        )
        .to_string();
        assert!(
            got.contains('&'),
            "a borrowed out_ty keeps its borrow — got `{got}`"
        );
    }

    /// An owned `Option` payload is borrowed for the accessor that follows it.
    ///
    /// The `Some(..)` arm of a gated step binds the step's own value, and an
    /// accessor returning `Option<T>` binds a bare `T` there. Composing the
    /// next accessor straight onto it hands `T` to a receiver typed `&T` —
    /// ill-typed Rust in the consumer's crate, and invisible here until a
    /// reach has BOTH an owned optional step and something after it. The gated
    /// reach hardcoded "already a reference" for years because no declared
    /// leaf had that shape (#609 review). The hoist side of the same rule is
    /// pinned by `value_form::an_owned_optional_payload_is_borrowed_for_the_steps_after_it`.
    #[test]
    fn a_gated_owned_payload_is_borrowed_for_the_step_after_it() {
        let path = vec![
            PathStep::call(syn::parse_quote!(get_opt), true, true),
            PathStep::call(syn::parse_quote!(next), false, false),
        ];
        let l = leaf(
            syn::parse_quote!(Owned),
            path.clone(),
            false,
            LeafSource::Reach,
        );
        let got = prebindgen_registry::unfold::reach_leaf(
            &qualify,
            prebindgen_registry::unfold::LeafAt {
                leaf: &crate::jni::compile::OutWire::from_leaf(&l),
                path: &path,
                base: quote!(__src),
                base_is_ref: false,
                consuming: false,
                unwrap_last: true,
            },
            Some(&|| quote!(__absent)),
            &|reached| reached,
        )
        .to_string();
        assert!(
            got.contains("next (& __n0)"),
            "the owned payload is borrowed for the accessor that follows it — got `{got}`"
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
        let got = prebindgen_registry::unfold::reach_leaf(
            &qualify,
            prebindgen_registry::unfold::LeafAt {
                leaf: &crate::jni::compile::OutWire::from_leaf(&l),
                path: &path,
                base: quote!(__src),
                base_is_ref: false,
                consuming: false,
                unwrap_last: false,
            },
            None,
            &|reached| reached,
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
        let _ = prebindgen_registry::unfold::reach_leaf(
            &qualify,
            prebindgen_registry::unfold::LeafAt {
                leaf: &crate::jni::compile::OutWire::from_leaf(&l),
                path: &rest,
                base: quote!(__vf0),
                base_is_ref: false,
                consuming: false,
                unwrap_last: false,
            },
            None,
            &|reached| reached,
        );
    }
}
