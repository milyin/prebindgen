//! Struct outputs: `fromParts` leaf encoding and the value-struct
//! synthesis probe.

use prebindgen_registry::Conversions;

use super::*;

/// Resolve the typed-handle Kotlin FQN for a handle-bearing struct field
/// and assert its folded strategy is one the struct encode/decode bridge
/// supports. Today only scalar handle slots (`Direct`, optionally wrapped
/// in `Nullable`) are encodable as a single `L<FQN>;` ctor arg; a
/// collection layer (`Iterable`, i.e. `Vec<Handle>`) would need array
/// codegen and is a loud build-time error until implemented.
pub(crate) fn handle_field_fqn(ext: &Declarations, h: &Projection) -> String {
    fn assert_scalar(s: &FoldStrategy) {
        match s {
            FoldStrategy::Base => {}
            FoldStrategy::Optional(_, inner) => assert_scalar(inner),
            FoldStrategy::Iterable(_) => panic!(
                "struct handle field: collection (Vec<Handle>) layers are not yet \
                 supported by the struct encode/decode bridge — add array codegen \
                 to the late struct output/input plans to lift this guard"
            ),
        }
    }
    assert_scalar(&h.strategy);
    ext.kotlin_fqn(&h.leaf_key)
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            panic!(
                "struct handle field: leaf `{}` has no Kotlin FQN registered \
                 (ptr_class)",
                h.leaf_key
            )
        })
}

/// Zero/null wire value for a JVM descriptor — used to fill an absent
/// `Option<nested>`'s leaf slots (the Kotlin `present` flag tells the factory
/// to ignore them).
pub(crate) fn primitive_default_for_descriptor(sig: &str) -> TokenStream {
    match sig {
        "Z" => quote!(0u8),
        "B" => quote!(0i8),
        "C" => quote!(0u16),
        "S" => quote!(0i16),
        "I" => quote!(0i32),
        "J" => quote!(0i64),
        "F" => quote!(0.0f32),
        "D" => quote!(0.0f64),
        _ => quote!(jni::objects::JObject::null()),
    }
}

/// The [`LeafSource::Reach`](crate::unfold::LeafSource) leaves of
/// a by-value `data_class`, as the fixed-builder output and callback paths want
/// them.
///
/// The list is [`Declarations::struct_out_wires`]', mapped. Runs BEFORE
/// `resolve`, which is exactly why it shares that composition rather than
/// walking the struct a second time: the leaf names reach the foreign
/// `fromParts` parameters and the recipe identically, and two walks agreeing was a
/// property nothing checked.
///
/// `None` — the type keeps the whole-value `fromParts` path — when a field is
/// one the decomposition cannot state at all. Those are structural: a repeated
/// nested class, a field the model does not hold as a named one, nesting past
/// 16. Not a missing transform — a handle, an `enum_class`, a sum, an optional
/// nested class and a selector inside a gated group all state fine, each
/// carrying its own conversion (#602). See [`Declarations::struct_out_wires`]
/// for the list and why one such field declines the whole value.
pub(crate) fn synth_value_struct_leaves(
    ext: &Declarations,
    registry: &impl Conversions,
    s: &prebindgen_registry::flat::Struct,
) -> Option<Vec<crate::unfold::UnfoldLeaf>> {
    use crate::unfold::{LeafSource, UnfoldLeaf};
    Some(
        ext.struct_out_wires_of(registry.flat(), &s.name)?
            .into_iter()
            .map(|w| UnfoldLeaf {
                name: w.name,
                path: w.reach,
                out_ty: w.out_ty,
                identity: false,
                nullable: w.nullable,
                // A selector is not read off a place: the emitter assigns the
                // live alternative's number in each arm of its `match`, which
                // is what `LeafSource::SumTag` says and what keeps
                // `OutWire::from_leaf` round-tripping it back to a tag rather
                // than trying to compile the sum type as a crossing.
                source: match w.from {
                    crate::jni::compile::OutFrom::Tag => LeafSource::SumTag,
                    crate::jni::compile::OutFrom::Present => LeafSource::Presence,
                    _ => LeafSource::Reach,
                },
                groups: w.groups,
            })
            .collect(),
    )
}

/// The prefix an arm-local binding carries, and the outer slot it feeds.
///
/// A leaf's name is the same on both sides — it is one leaf — so the two would
/// collide in one function. The arm-local name is the outer name with a
/// **prefix** this module adds, which is why recovering the outer one is
/// stripping a known prefix at a known position rather than searching for a
/// marker: `base` and every slot fragment come from source identifiers, and
/// any sentinel spelled inside one of them can appear in the middle of a name
/// too (#616 review).
/// The one "arm" an optional value has: present. Absence contributes no
/// bindings of its own, only the defaults its group carries.
pub(crate) const PRESENT_ARM: i32 = 0;

/// Assemble the Rust→JVM half of a frozen whole-struct codec.
///
/// Classification, child selection, JVM descriptors, and the target Kotlin
/// class are all frozen before this point. This function only turns that plan
/// into the final converter body; it performs no registry or declaration
/// lookup and cannot rediscover source-type facts.
pub(crate) fn render_struct_output_body(
    delivery: &crate::jni::emit::FrozenDelivery,
    java_class_name: &str,
    emit: &prebindgen_registry::RustWriter,
) -> syn::Expr {
    // The whole object graph flattened into leaf wires, then built with ONE
    // `call_static_method("fromParts", …)` — no per-nested-struct JNI crossing.
    // The Kotlin `fromParts` factory reassembles the graph in bytecode.
    //
    // The flattening is the registry's walk, the same one a fixed-builder site
    // delivers through: `v` is borrowed and each leaf cloned out of it, which
    // is what `is_field_read` decides, and a gated group is one `match` the
    // walk emits rather than an arm this module builds.
    let n = delivery.wire_count();
    let obj_idents: Vec<syn::Ident> = (0..n).map(|i| format_ident!("__obj{}", i)).collect();
    // A converter returns its failure; there is no error sink at this site.
    let fail = |msg: TokenStream| -> TokenStream {
        quote! {
            return ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<String>>::from(#msg),
            );
        }
    };
    let (preludes, args, _) = crate::jni::emit::encode_plan_leaves(
        delivery,
        delivery.delivered(),
        &obj_idents,
        &quote!(&v),
        &fail,
        emit,
    );
    let factory_sig_lit = syn::LitStr::new(
        &delivery.factory_signature(java_class_name),
        Span::call_site(),
    );

    syn::parse_quote!({
        #preludes
        let __obj = env.call_static_method(
            #java_class_name,
            "fromParts",
            #factory_sig_lit,
            &[#(#args),*],
        )
        .and_then(|__v| __v.l())
        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}

pub(crate) fn struct_module_path(
    ext: &Declarations,
    registry: &impl Conversions,
    name: &syn::Ident,
) -> syn::Path {
    // The module the struct is reachable under from the generated file: its
    // origin crate (multi-source registries) or the default module. Takes the
    // NAME, which is all it needs — so it serves a caller holding the element
    // and one still holding the item.
    ext.fn_module(registry, name)
}

// ──────────────────────────────────────────────────────────────────────
// Enum rank-0 bodies
// ──────────────────────────────────────────────────────────────────────
