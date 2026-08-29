//! Sum outputs: the leaf synthesis for a sum that IS a function's own return
//! (or a callback argument), and the `match` that encodes it.
//!
//! A sum is the one decomposition that is not a deterministic product: only
//! ONE alternative's leaves are live per value. Core models that with a
//! synthesized [`LeafSource::SumTag`] selector plus per-leaf
//! [`UnfoldLeaf::group`] membership
//! ([`apply_sum_returns`](prebindgen_registry::unfold::apply_sum_returns)); this
//! module is the JNI adapter's two ends of it — [`synth_sum_leaves`] builds the
//! leaf list before `resolve`, [`encode_sum_leaves`] emits the single `match`
//! that fills every slot at emit time.
//!
//! The wire layout is the same one a sum-typed **struct field** gets
//! ([`PlanFieldKind::Sum`](super::super::PlanFieldKind::Sum)): a tag slot
//! followed by one leaf group per variant, laid side by side, inert groups
//! wire-defaulted. Only the delivery differs — a field's slots ride the
//! parent's `fromParts`, a return's ride the hoisted builder singleton.

use prebindgen_registry::Conversions;

use super::*;

/// Leaf name of the synthesized selector. Distinct from every group slot by
/// construction: a group slot always contains the `_` that separates its
/// variant fragment from its property.
pub(crate) const SUM_TAG_LEAF: &str = "tag";

/// The leaf-name fragment of a presence flag — what an optional nested class
/// contributes ahead of the group it gates.
pub(crate) const PRESENT_LEAF: &str = "present";

/// The leaves of one `sealed_class`-declared sum, as the expansion plans want
/// them: the [`LeafSource::SumTag`] selector followed by one
/// [`LeafSource::VariantField`] leaf per payload field, in tag order, each
/// carrying its variant's tag as its [`group`](UnfoldLeaf::group).
///
/// The list is `Declarations::sum_out_wires`', mapped. Runs BEFORE `resolve`,
/// which is exactly why it shares that composition rather than walking the
/// enum a second time: a `variant!(V).name(..)` rename and the slot naming have
/// to reach the builder's parameter names and the recipe identically, and two
/// walks agreeing was a property nothing checked.
pub(crate) fn synth_sum_leaves(
    ext: &Declarations,
    registry: &impl Conversions,
    ident: &syn::Ident,
    sum: &prebindgen_registry::flat::Variant,
) -> Vec<prebindgen_registry::unfold::UnfoldLeaf> {
    use prebindgen_registry::unfold::{LeafSource, UnfoldLeaf};
    ext.sum_out_wires(registry, ident, sum.type_ref())
        .unwrap_or_default()
        .into_iter()
        .map(|w| UnfoldLeaf {
            name: w.name,
            path: Vec::new(),
            out_ty: w.out_ty,
            identity: false,
            nullable: w.nullable,
            source: match w.from {
                // Nothing looks up a converter for the selector
                // (`has_converter()` is false for a `SumTag`): there is no
                // value to convert, the emitter assigns the tag literal per
                // arm. Its wire is a `jint` by definition.
                crate::jni::compile::OutFrom::Tag => LeafSource::SumTag,
                crate::jni::compile::OutFrom::Payload { variant, member } => {
                    LeafSource::VariantField {
                        variant: variant.expect("a composed payload names its alternative"),
                        member,
                    }
                }
                crate::jni::compile::OutFrom::Place => LeafSource::Reach,
                crate::jni::compile::OutFrom::Present => LeafSource::Presence,
            },
            group: w.group,
        })
        .collect()
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
    context: &crate::jni::emit::delivery::FrozenDelivery,
    leaf: &crate::jni::compile::OutWire,
) -> Slot {
    if !context.leaf_is_prim(leaf) {
        return Slot {
            prim: false,
            ty: quote!(jni::objects::JObject),
            default: quote!(jni::objects::JObject::null()),
        };
    }
    // The tag is synthesized, so it has no converter to read a wire from — it
    // is a `jint` by definition.
    let (sig, letter) = if leaf.is_tag() {
        ("I", format_ident!("i"))
    } else {
        let wire = context.leaf_wire(leaf);
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

/// Whether the delivered value **is** a sum, rather than merely containing one.
///
/// The tag comes first and reaches nothing: it selects over the whole value.
/// A struct with a sum-typed FIELD also carries a tag — reached through that
/// field — and is a product whose fields include a segment. The two were the
/// same question only while a field could not decompose to a tag, which is
/// what #602 changed: the predicate this replaced answered "any leaf is a
/// tag", and a builder asking it tried to build a sealed-class builder for a
/// struct.
pub(crate) fn is_whole_sum_row(leaves: &[crate::jni::compile::OutWire]) -> bool {
    leaves
        .first()
        .is_some_and(|leaf| leaf.is_tag() && leaf.reach.is_empty())
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
    context: &crate::jni::emit::delivery::FrozenDelivery,
    leaves: &[crate::jni::compile::OutWire],
    obj_idents: &[syn::Ident],
    matched: TokenStream,
    fail: &dyn Fn(TokenStream) -> TokenStream,
    emit: &prebindgen_registry::RustWriter,
) -> (TokenStream, Vec<TokenStream>) {
    // Which sum this is comes from the selector leaf, not from the plan's
    // source: the plan's source is the *containing* value when the sum is a
    // field of a value form.
    let tag_leaf = leaves
        .iter()
        .find(|l| l.is_tag())
        .expect("a sum segment carries its selector leaf");
    let (source, sum) = prebindgen_registry::unfold::DeliveryBridge::sum(context, tag_leaf);

    let slots: Vec<Slot> = leaves.iter().map(|l| leaf_slot(context, l)).collect();

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
        .position(|l| l.is_tag())
        .expect("a sum plan carries its selector leaf");
    let tag_id = &obj_idents[tag_idx];
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
            let pattern = emit.shape_alternative(alt, quote!(#source::#vident), &parts);
            // The live group: convert each payload through its own output
            // converter, exactly as a struct field of the same type would be.
            let live: TokenStream = group
                .iter()
                .zip(&binds)
                .map(|(&idx, bind)| {
                    encode_group_leaf(
                        context,
                        &leaves[idx],
                        &obj_idents[idx],
                        slots[idx].prim,
                        bind,
                        fail,
                        emit,
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
    context: &crate::jni::emit::delivery::FrozenDelivery,
    leaf: &crate::jni::compile::OutWire,
    obj_ident: &syn::Ident,
    prim: bool,
    bind: &syn::Ident,
    fail: &dyn Fn(TokenStream) -> TokenStream,
    emit: &prebindgen_registry::RustWriter,
) -> TokenStream {
    let frozen_pipeline = match &leaf.abi {
        Some(crate::jni::compile::OutAbi::Value(value)) => &value.pipeline,
        Some(crate::jni::compile::OutAbi::Tag) | Some(crate::jni::compile::OutAbi::Present) => {
            unreachable!("a segment's payload is not its selector")
        }
        None => panic!(
            "jnigen sum delivery: payload leaf `{}` reached Rust rendering without a frozen output ABI",
            leaf.name
        ),
    };
    let wire = context.leaf_wire(leaf);
    let conv_fail = fail(quote!(__e.to_string()));
    let enc = format_ident!("__enc_{}", obj_ident);
    let mut encode = TokenStream::new();
    let call = frozen_pipeline.invoke(quote!(#bind.clone()), emit);
    encode.extend(quote! {
        let #enc = match #call {
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

/// The encode of a **presence** segment: an optional nested value's flag and
/// the leaves it gates, from the value the walk has already unwrapped.
///
/// [`prebindgen_registry::unfold::segment`] supplies the gate — one tuple bind
/// whose absent arm carries every slot's default — and calls this for the
/// present arm, so what is here is the flag and the child's own leaves read
/// off the unwrapped value. The sum twin next to it answers the same shape
/// with alternatives in place of presence.
pub(crate) fn encode_presence_group(
    context: &crate::jni::emit::delivery::FrozenDelivery,
    leaves: &[crate::jni::compile::OutWire],
    obj_idents: &[syn::Ident],
    matched: TokenStream,
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    fail: &dyn Fn(TokenStream) -> TokenStream,
    emit: &prebindgen_registry::RustWriter,
) -> (TokenStream, Vec<TokenStream>) {
    use prebindgen_registry::unfold::DeliveryBridge;

    let flag = &obj_idents[0];
    let flag_slot = leaf_slot(context, &leaves[0]);
    let flag_ty = &flag_slot.ty;
    let mut stmts = quote! { let #flag: #flag_ty = jni::sys::jvalue { z: 1u8 }; };
    let mut args: Vec<TokenStream> = vec![quote!(#flag)];

    // The prefix the presence flag reaches is already consumed: the walk bound
    // what it found there, so each gated leaf reaches on from that binding.
    let consumed = leaves[0].reach.len();
    for (index, leaf) in leaves.iter().enumerate().skip(1) {
        let slot = &obj_idents[index];
        let tail: Vec<prebindgen_registry::unfold::PathStep> =
            leaf.reach.iter().skip(consumed).cloned().collect();
        let matched = matched.clone();
        let reach = |body: &dyn Fn(TokenStream) -> TokenStream| -> TokenStream {
            prebindgen_registry::unfold::reach_leaf(
                qualify,
                prebindgen_registry::unfold::LeafAt {
                    leaf,
                    path: &tail,
                    base: matched.clone(),
                    base_is_ref: true,
                    consuming: false,
                    unwrap_last: false,
                },
                Some(&|| DeliveryBridge::absent(context)),
                body,
            )
        };
        stmts.extend(context.encode(leaf, index, slot, &reach, fail, emit));
        args.push(context.argument(leaf, slot));
    }
    (stmts, args)
}
