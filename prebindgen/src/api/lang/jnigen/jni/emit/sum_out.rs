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
    ext: &JniGen,
    sum_cfg: &SumConfig,
    item_enum: &syn::ItemEnum,
) -> Vec<crate::api::core::unfold::UnfoldLeaf> {
    use crate::api::core::{
        types_util::SumSpec,
        unfold::{LeafSource, UnfoldLeaf},
    };

    let spec = SumSpec::from_item_enum(item_enum);
    // The selector rides ahead of the groups it chooses between, and carries
    // **which sum** it selects over as its `out_ty` — that is how the emitter
    // finds the enum to `match` when the sum is a field rather than the whole
    // returned value. Nothing looks up a converter for it (`has_converter()` is
    // false for a `SumTag`): there is no value to convert, the emitter assigns
    // the tag literal per arm. Its wire is a `jint` by definition.
    let enum_ident = &item_enum.ident;
    let mut leaves = vec![UnfoldLeaf {
        name: SUM_TAG_LEAF.to_string(),
        path: Vec::new(),
        out_ty: syn::parse_quote!(#enum_ident),
        identity: false,
        nullable: false,
        source: LeafSource::SumTag,
        group: None,
    }];
    for variant in &spec.variants {
        let kotlin_name = ext.sum_variant_class_name(sum_cfg, &variant.ident);
        for field in &variant.fields {
            let prop = sum_field_prop_name(field);
            leaves.push(UnfoldLeaf {
                name: sum_slot_fragment(&kotlin_name, &prop),
                path: Vec::new(),
                out_ty: field.ty.clone(),
                identity: false,
                nullable: false,
                source: LeafSource::VariantField {
                    variant: variant.ident.clone(),
                    member: field.member.clone(),
                },
                group: Some(variant.tag),
            });
        }
    }
    leaves
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
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    leaves: &[crate::api::core::unfold::UnfoldLeaf],
    obj_idents: &[syn::Ident],
    matched: TokenStream,
    fail: &dyn Fn(TokenStream) -> TokenStream,
) -> (TokenStream, Vec<TokenStream>) {
    use crate::api::core::unfold::LeafSource;

    // Which sum this is comes from the selector leaf, not from the plan's
    // source: the plan's source is the *containing* value when the sum is a
    // field of a value form.
    let tag_leaf = leaves
        .iter()
        .find(|l| l.source == LeafSource::SumTag)
        .expect("a sum segment carries its selector leaf");
    let ident = bare_path_ident(&tag_leaf.out_ty).unwrap_or_else(|| {
        panic!(
            "jnigen sum unfold: selector type `{}` is not a path type",
            TypeKey::from_type(&tag_leaf.out_ty)
        )
    });
    let module = ext.fn_module(registry, &ident);
    let source: syn::Path = syn::parse_quote!(#module::#ident);

    // Per-leaf slot shape: a primitive-wire leaf occupies a typed `jvalue`
    // (its raw primitive rides the typed `run`), everything else a `JObject`.
    // Derived from the SAME `leaf_is_prim` the argument expressions below and
    // the interface descriptor use, so the three cannot disagree.
    struct Slot {
        prim: bool,
        ty: TokenStream,
        default: TokenStream,
    }
    let slots: Vec<Slot> = leaves
        .iter()
        .map(|leaf| {
            if !leaf_is_prim(registry, leaf) {
                return Slot {
                    prim: false,
                    ty: quote!(jni::objects::JObject),
                    default: quote!(jni::objects::JObject::null()),
                };
            }
            // The tag is synthesized, so it has no converter to read a wire
            // from — it is a `jint` by definition.
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
        })
        .collect();

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
    // enum's own variants, not by the grouped leaves.
    let (item_enum, _) = registry.enums.get(&ident).unwrap_or_else(|| {
        panic!("jnigen sum unfold: no indexed enum `{ident}` for the decomposed sum")
    });
    let spec = crate::api::core::types_util::SumSpec::from_item_enum(item_enum);

    let arms: Vec<TokenStream> = spec
        .variants
        .iter()
        .map(|variant| {
            let group: Vec<usize> = leaves
                .iter()
                .enumerate()
                .filter(|(_, l)| l.group == Some(variant.tag))
                .map(|(i, _)| i)
                .collect();
            let binds: Vec<syn::Ident> = group
                .iter()
                .enumerate()
                .map(|(k, _)| format_ident!("__sv{}", k))
                .collect();
            let vident = &variant.ident;
            let pattern = match variant.fields.first().map(|f| &f.member) {
                None => quote!(#source::#vident),
                Some(syn::Member::Named(_)) => {
                    let pairs = variant.fields.iter().zip(&binds).map(|(f, b)| {
                        let syn::Member::Named(n) = &f.member else {
                            unreachable!("variant field shapes are uniform")
                        };
                        quote!(#n: #b)
                    });
                    quote!(#source::#vident { #(#pairs),* })
                }
                Some(syn::Member::Unnamed(_)) => quote!(#source::#vident(#(#binds),*)),
            };
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
            let tag_lit = proc_macro2::Literal::i32_unsuffixed(variant.tag);
            quote! {
                #pattern => {
                    #live
                    #tag_id = jni::sys::jvalue { i: #tag_lit };
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
    registry: &Registry<KotlinMeta>,
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
            TypeKey::from_type(&leaf.out_ty)
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
