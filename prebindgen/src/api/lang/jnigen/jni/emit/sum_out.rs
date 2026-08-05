//! Sum outputs: the leaf synthesis for a sum that IS a function's own return
//! (or a callback argument), and the `match` that encodes it.
//!
//! A sum is the one decomposition that is not a deterministic product: only
//! ONE alternative's leaves are live per value. Core models that with a
//! synthesized [`LeafSource::SumTag`] selector plus per-leaf
//! [`UnfoldLeaf::group`] membership
//! ([`apply_sum_returns`](crate::api::core::unfold::apply_sum_returns)); this
//! module is the JNI adapter's two ends of it — [`synth_sum_leaves`] builds the
//! leaf list before `resolve`, [`encode_sum_leaves`] emits the single `match`
//! that fills every slot at emit time.
//!
//! The wire layout is the same one a sum-typed **struct field** gets
//! ([`PlanFieldKind::Sum`](super::super::PlanFieldKind::Sum)): a tag slot
//! followed by one leaf group per variant, laid side by side, inert groups
//! wire-defaulted. Only the delivery differs — a field's slots ride the
//! parent's `fromParts`, a return's ride the hoisted builder singleton.

use super::*;
use crate::api::core::registry::Conversions;

/// Leaf name of the synthesized selector. Distinct from every group slot by
/// construction: a group slot always contains the `_` that separates its
/// variant fragment from its property.
pub(crate) const SUM_TAG_LEAF: &str = "tag";

/// Synthesize the leaves of one `sealed_class`-declared sum: the
/// [`LeafSource::SumTag`] selector followed by one
/// [`LeafSource::VariantField`] leaf per payload field, in tag order, each
/// carrying its variant's tag as its [`group`](UnfoldLeaf::group).
///
/// Runs BEFORE `resolve` (like
/// [`synth_value_struct_leaves`](super::synth_value_struct_leaves)), so it may
/// only read the declaration (`sum_cfg`) and the enum's syntax — never the
/// converter tables. Every payload's own `out_ty` is registered as a required
/// output by the core wiring, so a payload that cannot cross fails naming
/// itself.
///
/// Slot names come from the same [`sum_slot_fragment`] / [`sum_field_prop_name`]
/// pair the sealed-interface emitter and the struct-field bridge use, so a
/// `variant!(V).name(...)` rename carries through to the builder's parameter
/// names too.
pub(crate) fn synth_sum_leaves(
    ext: &Declarations,
    sum_cfg: &SumConfig,
    sum: &crate::api::core::flat::Variant,
) -> Vec<crate::api::core::unfold::UnfoldLeaf> {
    use crate::api::core::unfold::{LeafSource, UnfoldLeaf};

    // The selector rides ahead of the groups it chooses between, and carries
    // **which sum** it selects over as its `out_ty` — that is how the emitter
    // finds the enum to `match` when the sum is a field rather than the whole
    // returned value. Nothing looks up a converter for it (`has_converter()` is
    // false for a `SumTag`): there is no value to convert, the emitter assigns
    // the tag literal per arm. Its wire is a `jint` by definition.
    let mut leaves = vec![UnfoldLeaf {
        name: SUM_TAG_LEAF.to_string(),
        path: Vec::new(),
        // The tag names WHICH sum it selects over, and no source wrote that as
        // a standalone type — so the DECLARATION answers, rather than this
        // emitter minting a reading from the name. Nothing resolves a converter
        // for it (`has_converter()` is false), but the emitter reads it back to
        // find the enum to `match`.
        out_ty: sum.type_ref().clone(),
        identity: false,
        nullable: false,
        source: LeafSource::SumTag,
        group: None,
    }];
    for alt in &sum.alternatives {
        let kotlin_name = ext.sum_variant_class_name(sum_cfg, &alt.name);
        for field in &alt.fields {
            let prop = sum_field_prop_name(&field.member());
            leaves.push(UnfoldLeaf {
                name: sum_slot_fragment(&kotlin_name, &prop),
                path: Vec::new(),
                out_ty: field.ty.clone(),
                identity: false,
                nullable: false,
                source: LeafSource::VariantField {
                    variant: alt.name.clone(),
                    member: field.member(),
                },
                group: Some(sum_tag(alt)),
            });
        }
    }
    leaves
}

/// The wire slot one leaf occupies when its value is only computed in SOME arm
/// of a `match`: a primitive-wire leaf occupies a typed `jvalue` (its raw
/// primitive rides the typed `run`), everything else a `JObject`. `default` is
/// what an arm that does not compute the leaf assigns instead.
///
/// Shared by the two conditional shapes — a decomposed sum's inert groups
/// ([`encode_sum_group`]) and the absent arm of a conditional value form
/// ([`encode_plan_leaves`](super::encode_plan_leaves)) — so "what does an
/// unfilled slot carry" has one answer. Derived from the same
/// [`leaf_is_prim`] the argument expressions and the interface descriptor use.
pub(crate) struct Slot {
    pub(crate) prim: bool,
    pub(crate) ty: TokenStream,
    pub(crate) default: TokenStream,
}

pub(crate) fn leaf_slot(
    registry: &impl Conversions<KotlinMeta>,
    leaf: &crate::api::core::unfold::UnfoldLeaf,
) -> Slot {
    use crate::api::core::unfold::LeafSource;
    if !leaf_is_prim(registry, leaf) {
        return Slot {
            prim: false,
            ty: quote!(jni::objects::JObject),
            default: quote!(jni::objects::JObject::null()),
        };
    }
    // The tag is synthesized, so it has no converter to read a wire from — it
    // is a `jint` by definition.
    let (sig, letter) = if leaf.source == LeafSource::SumTag {
        ("I", format_ident!("i"))
    } else {
        let wire = registry
            .output_entry(&leaf.out_ty)
            .expect("leaf_is_prim implies a resolved output entry")
            .destination
            .clone();
        let (sig, letter, _) =
            jni_field_access(&wire).expect("leaf_is_prim guarantees a primitive wire");
        (sig, letter)
    };
    let zero = primitive_default_for_descriptor(sig);
    Slot {
        prim: true,
        ty: quote!(jni::sys::jvalue),
        default: quote!(jni::sys::jvalue { #letter: #zero }),
    }
}

/// True when `plan` decomposes a sum — it carries the synthesized selector.
/// The one place that question is asked, so every consumer agrees on it.
pub(crate) fn is_sum_leaves(leaves: &[crate::api::core::unfold::UnfoldLeaf]) -> bool {
    use crate::api::core::unfold::LeafSource;
    leaves.iter().any(|l| l.source == LeafSource::SumTag)
}

/// Emit the Rust-side encode of a decomposed sum: ONE `match` over the value
/// binding the tag and EVERY group's slots — the live group from its variant
/// pattern's payload bindings, every other group from the same wire defaults an
/// absent `Option<nested>` uses.
///
/// `leaves` is ONE sum's segment — its [`LeafSource::SumTag`] selector followed
/// by that selector's group leaves — with `obj_idents` the matching slice of
/// slot locals. `matched` is the expression to `match` on (a reference to the
/// value), which is the whole returned value when the sum IS the return, and
/// the reached field when a value form carries it.
///
/// The signature mirrors [`encode_plan_leaves`](super::encode_plan_leaves), and
/// the two are interchangeable at the call site: both bind `obj_idents` and
/// return the per-leaf `jvalue` argument expressions in leaf order. What differs
/// is that a leaf here is not an independent expression — its slot exists in
/// every arm and only one arm computes it.
pub(crate) fn encode_sum_group(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    leaves: &[crate::api::core::unfold::UnfoldLeaf],
    obj_idents: &[syn::Ident],
    matched: TokenStream,
    fail: &dyn Fn(TokenStream) -> TokenStream,
    emit: &crate::api::core::emit::Emit,
) -> (TokenStream, Vec<TokenStream>) {
    use crate::api::core::unfold::LeafSource;

    // Which sum this is comes from the selector leaf, not from the plan's
    // source: the plan's source is the *containing* value when the sum is a
    // field of a value form.
    let tag_leaf = leaves
        .iter()
        .find(|l| l.source == LeafSource::SumTag)
        .expect("a sum segment carries its selector leaf");
    // The name off the reading — `TypeId` IS the name, so nothing takes a path
    // apart to re-derive one.
    let crate::api::core::flat::TypeKind::Named { id, .. } = tag_leaf.out_ty.unwrapped().kind()
    else {
        panic!(
            "jnigen sum unfold: selector type `{}` is not a named type",
            tag_leaf.out_ty.key()
        )
    };
    // Raw-aware: a sum may legitimately be named `r#type`, and `Ident::new`
    // rejects that spelling.
    let ident = id.ident().unwrap_or_else(|| {
        panic!(
            "jnigen sum unfold: selector type `{}` is not a single identifier",
            id.name
        )
    });
    let module = ext.fn_module(registry, &ident);
    let source: syn::Path = syn::parse_quote!(#module::#ident);

    let slots: Vec<Slot> = leaves.iter().map(|l| leaf_slot(registry, l)).collect();

    let arg_exprs: Vec<TokenStream> = leaves
        .iter()
        .enumerate()
        .map(|(idx, _)| {
            let id = &obj_idents[idx];
            if slots[idx].prim {
                quote!(#id)
            } else {
                quote!(jni::sys::jvalue { l: #id.as_raw() })
            }
        })
        .collect();

    // Declare every slot up front; each arm assigns all of them.
    let decls: TokenStream = leaves
        .iter()
        .enumerate()
        .map(|(idx, _)| {
            let id = &obj_idents[idx];
            let ty = &slots[idx].ty;
            quote! { let #id: #ty; }
        })
        .collect();

    // One arm per variant, in tag order. `groups[tag]` are the leaf indices of
    // that variant's payload, in declaration order — which is also the order
    // its pattern binds them in.
    let tag_idx = leaves
        .iter()
        .position(|l| l.source == LeafSource::SumTag)
        .expect("a sum plan carries its selector leaf");
    let tag_id = &obj_idents[tag_idx];
    // A unit variant contributes no leaf, so the arm list is driven by the
    // enum's own alternatives, not by the grouped leaves. `enum_item` hands
    // back only the `syn::ItemEnum`, deliberately — a consumer that acts on the
    // Variant/Enum distinction asks `declared_type` (#289).
    let Some(crate::api::core::flat::Type::Variant(sum)) = registry.flat().declared_type(&ident)
    else {
        panic!("jnigen sum unfold: no indexed sum `{ident}` for the decomposed sum")
    };

    let arms: Vec<TokenStream> = sum
        .alternatives
        .iter()
        .map(|alt| {
            let tag = sum_tag(alt);
            let group: Vec<usize> = leaves
                .iter()
                .enumerate()
                .filter(|(_, l)| l.group == Some(tag))
                .map(|(i, _)| i)
                .collect();
            let binds: Vec<syn::Ident> = group
                .iter()
                .enumerate()
                .map(|(k, _)| format_ident!("__sv{}", k))
                .collect();
            let vident = &alt.name;
            // The alternative's OWN delimiters, from the one place that chooses
            // them — for match patterns and constructors alike. Branching on
            // `fields.first()` could not answer this: an empty alternative has
            // no first field, so `enum E { B() }` and `enum E { B {} }` both
            // matched the `None` arm and emitted the bare `E::B`, which is
            // E0533 in pattern position. Same shape as the empty struct that
            // emitted `Unit {}` in #302.
            let parts: Vec<TokenStream> = alt
                .fields
                .iter()
                .zip(&binds)
                .map(|(f, b)| f.bind(b))
                .collect();
            let pattern = emit.shape(alt, quote!(#source::#vident), &parts);
            // The live group: convert each payload through its own output
            // converter, exactly as a struct field of the same type would be.
            let live: TokenStream = group
                .iter()
                .zip(&binds)
                .map(|(&idx, bind)| {
                    encode_group_leaf(
                        registry,
                        &leaves[idx],
                        &obj_idents[idx],
                        slots[idx].prim,
                        bind,
                        fail,
                    )
                })
                .collect();
            // Every slot outside this arm's own group is inert.
            let inert: TokenStream = (0..leaves.len())
                .filter(|i| *i != tag_idx && !group.contains(i))
                .map(|i| {
                    let id = &obj_idents[i];
                    let d = &slots[i].default;
                    quote! { #id = #d; }
                })
                .collect();
            let tag_lit = proc_macro2::Literal::i32_unsuffixed(tag);
            // A nullable selector rides an OBJECT slot (its absent case is JVM
            // null, which a raw `jint` has no room for), so the live tag boxes
            // like any other nullable primitive leaf.
            let set_tag = if slots[tag_idx].prim {
                quote! { #tag_id = jni::sys::jvalue { i: #tag_lit }; }
            } else {
                let box_fail = fail(quote!(__e));
                quote! {
                    #tag_id = match ::prebindgen_jni_runtime::box_jint(&mut env, #tag_lit) {
                        ::core::result::Result::Ok(__o) => __o,
                        ::core::result::Result::Err(__e) => {
                            #box_fail
                        }
                    };
                }
            };
            quote! {
                #pattern => {
                    #live
                    #set_tag
                    #inert
                }
            }
        })
        .collect();

    let stmts = quote! {
        #decls
        match #matched { #(#arms)* }
    };
    (stmts, arg_exprs)
}

/// Encode ONE payload binding of a live variant arm into its slot. `bind` is
/// the pattern variable holding `&Payload`; the value is cloned out of it, so a
/// payload and a struct field of the same type reach their converter the same
/// way.
fn encode_group_leaf(
    registry: &impl Conversions<KotlinMeta>,
    leaf: &crate::api::core::unfold::UnfoldLeaf,
    obj_ident: &syn::Ident,
    prim: bool,
    bind: &syn::Ident,
    fail: &dyn Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    let out_entry = registry.output_entry(&leaf.out_ty).unwrap_or_else(|| {
        panic!(
            "jnigen sum unfold: payload leaf `{}` (`{}`) has no registered output converter",
            leaf.name,
            leaf.out_ty.key()
        )
    });
    let wire = out_entry.destination.clone();
    let conv_fail = fail(quote!(__e.to_string()));
    let enc = format_ident!("__enc_{}", obj_ident);
    // The payload's COMPLETE chain: a `convert!`-declared type reaches the
    // wire through its rust-side stages first (`Duration → u64 → jlong`).
    // Stopping at the wire-facing converter would hand it the semantic value
    // where it expects the representation.
    let mut encode = TokenStream::new();
    let mut previous = quote!(#bind.clone());
    for (order, (_, stage)) in out_entry.output_stage_order().enumerate() {
        let stage_fn = &stage.function.sig.ident;
        let next = format_ident!("__enc_{}_s{}", obj_ident, order);
        encode.extend(quote! {
            let #next = match #stage_fn(&mut env, #previous) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    #conv_fail
                }
            };
        });
        previous = quote!(#next);
    }
    let conv = out_entry.converter_ident();
    encode.extend(quote! {
        let #enc = match #conv(&mut env, #previous) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                #conv_fail
            }
        };
    });
    if prim {
        let letter = jni_field_access(&wire)
            .expect("leaf_is_prim guarantees a primitive wire")
            .1;
        quote! {
            #encode
            #obj_ident = jni::sys::jvalue { #letter: #enc };
        }
    } else {
        let cast = cast_wire_to_jobject(&enc, &wire, fail);
        quote! {
            #encode
            #obj_ident = #cast;
        }
    }
}
