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

/// One flattened leaf wire slot of a struct's recursive `fromParts` encode
/// (see [`flatten_struct_encode`]). `ident` holds the encoded wire after the
/// preludes run; `default` is the value used for this slot when it sits under
/// an absent `Option<nested>` parent.
pub(crate) struct EncSlot {
    ident: proc_macro2::Ident,
    /// How deeply nested the field this slot came from is: 0 for the struct's
    /// own field, one more per inlined `data_class`.
    ///
    /// Read only by [`encode_leaves`], which
    /// `JniGen::assert_leaf_derivations_agree` compares against the
    /// registry-facing decomposition's own nesting — that one spells it with
    /// the reserved `__` separator, this one counts it. The encode itself
    /// needs the number while it recurses, not after, which is why nothing
    /// else reads the field.
    #[allow(dead_code)]
    depth: usize,
    wire_ty: TokenStream,
    descriptor: String,
    is_object: bool,
    default: TokenStream,
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

/// The [`LeafSource::Reach`](prebindgen_registry::unfold::LeafSource) leaves of
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
) -> Option<Vec<prebindgen_registry::unfold::UnfoldLeaf>> {
    use prebindgen_registry::unfold::{LeafSource, UnfoldLeaf};
    Some(
        ext.struct_out_wires_of(registry, &s.name)?
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

/// Walk a [`StructPlan`] emitting the Rust-side wire encode: per leaf a
/// prelude statement binding `__<prefix>_<field>` to the converted wire and
/// an [`EncSlot`] describing its `JValue` slot. `access` is the Rust
/// expression yielding the current struct value (`v`, `v.field`, or the
/// matched `__cN` under an Option); `prefix` namespaces the generated idents.
/// Walk the struct's fields and encode each leaf into a `fromParts` argument.
///
/// This is a source-value walk that the registry does not perform, and what
/// keeps it here is now only that nothing has moved it. The coverage reason is
/// gone: #616, #617 and #618 taught the registry-facing decomposition
/// (`Declarations::struct_out_wires_at`) every gated shape this encode
/// supports — a sum field as a tag over one group per variant, an optional
/// nested class as a presence flag over a defaulted group, and one selector
/// inside another — so the two now cover the same shapes rather than differing.
///
/// Which makes this the duplication #613 step 3 exists to remove: two walks
/// deriving one leaf list. `JniGen::assert_leaf_derivations_agree` is what says
/// they still agree, on every binding a test writes, and #619 is where this
/// walk goes and that check with it.
/// The leaves this encode emits, flattened — the `fromParts` argument list,
/// in order, each with how deeply nested the field it came from is.
///
/// Test support: the registry-facing decomposition of the same struct must
/// agree with this wherever it exists, and #603 recorded that agreement as
/// measured rather than checked. `JniGen::write_rust` checks it now.
/// Kept, and kept out of the encode, for one reason: the Kotlin `fromParts`
/// signature is still derived from [`StructPlan`], and this is what says that
/// derivation still agrees with the decomposition the Rust side now renders
/// from. It goes when the Kotlin half moves too — with
/// `JniGen::assert_leaf_derivations_agree`, `encode_plan`, `encode_field` and
/// the conversion half of `StructPlan` that nothing else reads (#619).
#[allow(dead_code)]
pub(crate) fn encode_leaves(
    plan: &StructPlan,
    emit: &prebindgen_registry::RustWriter,
) -> Vec<(String, usize)> {
    let (_, slots) = encode_plan(plan, &quote!(v), "", 0, &quote!(env), emit);
    slots
        .iter()
        .map(|slot| {
            (
                slot.ident.to_string().trim_start_matches('_').to_string(),
                slot.depth,
            )
        })
        .collect()
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

fn arm_local_base(tag: usize, base: &str) -> String {
    format!("arm{tag}_{base}")
}

/// The outer slot ident for an arm-local binding produced under
/// [`arm_local_base`].
fn outer_of(tag: usize, inner: &proc_macro2::Ident) -> proc_macro2::Ident {
    let name = inner.to_string();
    let prefix = format!("__arm{tag}_");
    let rest = name.strip_prefix(&prefix).unwrap_or_else(|| {
        panic!("an arm-local binding starts with the prefix its outer slot drops: `{name}`")
    });
    format_ident!("__{rest}")
}

fn encode_plan(
    plan: &StructPlan,
    access: &TokenStream,
    prefix: &str,
    depth: usize,
    env_expr: &TokenStream,
    emit: &prebindgen_registry::RustWriter,
) -> (TokenStream, Vec<EncSlot>) {
    let mut preludes = TokenStream::new();
    let mut slots: Vec<EncSlot> = Vec::new();

    for f in &plan.fields {
        let fname = &f.fname;
        let base = format!("{}_{}", prefix, fname);
        let value = quote! { #access.#fname };
        let (pre, sl) = encode_field(&f.kind, &value, &base, depth, env_expr, emit);
        preludes.extend(pre);
        slots.extend(sl);
    }
    (preludes, slots)
}

/// Emit the Rust-side wire encode of ONE value position — a struct field or a
/// sum's variant payload. `value` is the Rust expression yielding it (`v.mode`
/// at a struct field, the bound pattern variable inside a variant arm), which
/// is what lets a sum reuse this for its payloads: a payload is encoded by the
/// same code as a field of the same type, not by a parallel walk.
fn encode_field(
    kind: &PlanFieldKind,
    value: &TokenStream,
    base: &str,
    depth: usize,
    env_expr: &TokenStream,
    emit: &prebindgen_registry::RustWriter,
) -> (TokenStream, Vec<EncSlot>) {
    let mut preludes = TokenStream::new();
    let mut slots: Vec<EncSlot> = Vec::new();
    {
        let id = format_ident!("__{}", base);
        // The leaf's COMPLETE chain, not just its wire-facing converter: a
        // `convert!`-declared type (`Duration`) reaches the wire through its
        // rust-side stages first (`Duration → u64 → jlong`).
        let conv_value =
            |conv: &ConvChain| -> TokenStream { conv.call(env_expr, value, base, emit) };
        match kind {
            // Projection leaf (opaque handle → jlong, `ULong` → jlong).
            PlanFieldKind::Projection { conv, proj, .. } => {
                let value_expr = conv_value(conv);
                match proj.kind {
                    ProjectionKind::Handle => {
                        preludes.extend(quote! { let #id: jni::sys::jlong = #value_expr; });
                        slots.push(EncSlot {
                            depth,
                            ident: id,
                            wire_ty: quote!(jni::sys::jlong),
                            descriptor: "J".to_string(),
                            is_object: false,
                            default: quote!(0i64),
                        });
                    }
                    ProjectionKind::Unsigned64 => match proj.strategy {
                        FoldStrategy::Base => {
                            preludes.extend(quote! { let #id: jni::sys::jlong = #value_expr; });
                            slots.push(EncSlot {
                                depth,
                                ident: id,
                                wire_ty: quote!(jni::sys::jlong),
                                descriptor: "J".to_string(),
                                is_object: false,
                                default: quote!(0i64),
                            });
                        }
                        FoldStrategy::Optional(NullableKind::Niche, _) => {
                            preludes.extend(quote! { let #id: jni::sys::jlong = #value_expr; });
                            slots.push(EncSlot {
                                depth,
                                ident: id,
                                wire_ty: quote!(jni::sys::jlong),
                                descriptor: "J".to_string(),
                                is_object: false,
                                default: quote!(0i64),
                            });
                        }
                        FoldStrategy::Optional(NullableKind::Boxed, _) => {
                            preludes
                                .extend(quote! { let #id: jni::objects::JObject = #value_expr; });
                            slots.push(EncSlot {
                                depth,
                                ident: id,
                                wire_ty: quote!(jni::objects::JObject),
                                descriptor: "Ljava/lang/Long;".to_string(),
                                is_object: true,
                                default: quote!(jni::objects::JObject::null()),
                            });
                        }
                        FoldStrategy::Iterable(_) => unreachable!(
                            "projection collection fields are rejected by build_struct_plan"
                        ),
                    },
                }
            }
            // Enum leaf → jint discriminant (Kotlin `fromParts` calls `fromInt`).
            PlanFieldKind::Enum { conv, .. } => {
                let value_expr = conv_value(conv);
                preludes.extend(quote! { let #id: jni::sys::jint = #value_expr; });
                slots.push(EncSlot {
                    depth,
                    ident: id,
                    wire_ty: quote!(jni::sys::jint),
                    descriptor: "I".to_string(),
                    is_object: false,
                    default: quote!(0i32),
                });
            }
            // `Option<enum>` uses the enum's frozen primitive niche. Keep the
            // boxed Integer branch for plans produced without niche metadata.
            PlanFieldKind::OptionEnum { conv, niche, .. } => {
                let value_expr = conv_value(conv);
                if niche.is_some() {
                    preludes.extend(quote! { let #id: jni::sys::jint = #value_expr; });
                    slots.push(EncSlot {
                        depth,
                        ident: id,
                        wire_ty: quote!(jni::sys::jint),
                        descriptor: "I".to_string(),
                        is_object: false,
                        default: quote!(0i32),
                    });
                } else {
                    preludes.extend(quote! { let #id: jni::objects::JObject = #value_expr; });
                    slots.push(EncSlot {
                        depth,
                        ident: id,
                        wire_ty: quote!(jni::objects::JObject),
                        descriptor: "Ljava/lang/Integer;".to_string(),
                        is_object: true,
                        default: quote!(jni::objects::JObject::null()),
                    });
                }
            }
            // Nested data-class: inline the child's leaves; under `Option` add
            // a `present` flag and default the child slots in the `None` arm.
            PlanFieldKind::Nested {
                optional,
                plan: child,
                ..
            } => {
                if !*optional {
                    let (child_pre, child_slots) =
                        encode_plan(child, value, base, depth + 1, env_expr, emit);
                    preludes.extend(child_pre);
                    slots.extend(child_slots);
                } else {
                    let cbind = format_ident!("__c{}", depth);
                    let child_access = quote! { #cbind };
                    // The child encodes under an arm-local base, and the outer
                    // slots drop that prefix: the two are the same leaf, named
                    // the same way the decomposition names it, and one scope
                    // assigns from the other.
                    let (child_pre, child_slots) = encode_plan(
                        child,
                        &child_access,
                        &arm_local_base(PRESENT_ARM as usize, base),
                        depth + 1,
                        env_expr,
                        emit,
                    );
                    let flag_id = format_ident!("__{}_present", base);
                    let outer_ids: Vec<proc_macro2::Ident> = child_slots
                        .iter()
                        .map(|slot| outer_of(PRESENT_ARM as usize, &slot.ident))
                        .collect();
                    let outer_tys: Vec<TokenStream> =
                        child_slots.iter().map(|sl| sl.wire_ty.clone()).collect();
                    let inner_ids: Vec<proc_macro2::Ident> =
                        child_slots.iter().map(|sl| sl.ident.clone()).collect();
                    let defaults: Vec<TokenStream> =
                        child_slots.iter().map(|sl| sl.default.clone()).collect();
                    // Destructured through a coercion site: `kind` says this
                    // field is optional, and how Rust spells that is the
                    // source's business (#268).
                    let obind = format_ident!("__on{}", depth);
                    let coerce =
                        prebindgen_registry::unfold::bind_as_option(&quote!(&#value), &obind);
                    preludes.extend(quote! {
                        let #flag_id: jni::sys::jboolean;
                        #( let #outer_ids: #outer_tys; )*
                        #coerce
                        match #obind {
                            ::core::option::Option::Some(#cbind) => {
                                #child_pre
                                #flag_id = 1u8;
                                #( #outer_ids = #inner_ids; )*
                            }
                            ::core::option::Option::None => {
                                #flag_id = 0u8;
                                #( #outer_ids = #defaults; )*
                            }
                        }
                    });
                    // The flag sits one level inside the struct, with the
                    // group it gates: it is reached THROUGH this field, which
                    // is what the decomposition's `__` join says.
                    slots.push(EncSlot {
                        depth: depth + 1,
                        ident: flag_id,
                        wire_ty: quote!(jni::sys::jboolean),
                        descriptor: "Z".to_string(),
                        is_object: false,
                        default: quote!(0u8),
                    });
                    for (i, sl) in child_slots.iter().enumerate() {
                        slots.push(EncSlot {
                            depth: sl.depth,
                            ident: outer_ids[i].clone(),
                            wire_ty: sl.wire_ty.clone(),
                            descriptor: sl.descriptor.clone(),
                            is_object: sl.is_object,
                            default: sl.default.clone(),
                        });
                    }
                }
            }
            // Data-carrying enum: one `match` binds the tag and EVERY group's
            // slots — the live group from its payload, the rest from the same
            // defaults an absent `Option<nested>` uses. One crossing, no JVM
            // object built for the sum.
            PlanFieldKind::Sum {
                source,
                optional,
                variants,
                ..
            } => {
                let tag_id = format_ident!("__{}__tag", base);

                // Encode each variant's group once, against its own pattern
                // bindings. `arms` keeps them aligned with the flat slot list.
                struct Arm {
                    pattern: TokenStream,
                    preludes: TokenStream,
                    slots: Vec<EncSlot>,
                }
                let mut arms: Vec<Arm> = Vec::new();
                for (tag, v) in variants.iter().enumerate() {
                    let vident = &v.rust_ident;
                    let binds: Vec<syn::Ident> = (0..v.fields.len())
                        .map(|i| format_ident!("__s{}_{}", depth, i))
                        .collect();
                    let mut vpre = TokenStream::new();
                    let mut vslots: Vec<EncSlot> = Vec::new();
                    for (f, bind) in v.fields.iter().zip(&binds) {
                        // The arm's own bindings carry a marker segment the
                        // outer slots drop: the two are the same leaf and want
                        // the same name, and one scope assigns from the other.
                        let fbase = arm_local_base(tag, &format!("{base}_{}", f.slot));
                        let bind_expr = quote!(#bind);
                        let (p, s) =
                            encode_field(&f.kind, &bind_expr, &fbase, depth + 1, env_expr, emit);
                        vpre.extend(p);
                        vslots.extend(s);
                    }
                    // Bind every payload field, shaped like the variant.
                    let pattern = match v.fields.first().map(|f| &f.member) {
                        None => quote!(#source::#vident),
                        Some(syn::Member::Named(_)) => {
                            let pairs = v.fields.iter().zip(&binds).map(|(f, b)| {
                                let syn::Member::Named(n) = &f.member else {
                                    unreachable!("variant field shapes are uniform")
                                };
                                quote!(#n: #b)
                            });
                            quote!(#source::#vident { #(#pairs),* })
                        }
                        Some(syn::Member::Unnamed(_)) => quote!(#source::#vident(#(#binds),*)),
                    };
                    arms.push(Arm {
                        pattern,
                        preludes: vpre,
                        slots: vslots,
                    });
                }

                // Outer bindings: the tag plus every group's slots, side by
                // side in variant order. Each arm assigns its own group from
                // the values it just computed and defaults all the others.
                let all: Vec<&EncSlot> = arms.iter().flat_map(|a| a.slots.iter()).collect();
                // Named after the leaf each slot carries — `__outcome_found_v0`,
                // not `__outcome_g0`. The inner bindings already use that
                // naming, and it is what the registry-facing decomposition
                // calls the same leaf, so the two derivations agree by
                // construction rather than by a positional coincidence. The
                // inner binding of the same name lives inside its match arm
                // and shadows nothing the outer tuple reads.
                let outer_ids: Vec<proc_macro2::Ident> = arms
                    .iter()
                    .enumerate()
                    .flat_map(|(tag, arm)| {
                        arm.slots.iter().map(move |slot| outer_of(tag, &slot.ident))
                    })
                    .collect();
                let outer_tys: Vec<TokenStream> = all.iter().map(|s| s.wire_ty.clone()).collect();
                let defaults: Vec<TokenStream> = all.iter().map(|s| s.default.clone()).collect();

                let mut offset = 0usize;
                let arm_code: Vec<TokenStream> = arms
                    .iter()
                    .enumerate()
                    .map(|(tag, a)| {
                        let n = a.slots.len();
                        let live_outer = &outer_ids[offset..offset + n];
                        let live_inner: Vec<proc_macro2::Ident> =
                            a.slots.iter().map(|s| s.ident.clone()).collect();
                        // Every slot outside this arm's own group is inert.
                        let inert_outer: Vec<proc_macro2::Ident> = outer_ids
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| *i < offset || *i >= offset + n)
                            .map(|(_, id)| id.clone())
                            .collect();
                        let inert_defaults: Vec<TokenStream> = defaults
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| *i < offset || *i >= offset + n)
                            .map(|(_, d)| d.clone())
                            .collect();
                        offset += n;
                        let pattern = &a.pattern;
                        let pre = &a.preludes;
                        let tag_lit = proc_macro2::Literal::i32_unsuffixed(tag as i32);
                        quote! {
                            #pattern => {
                                #pre
                                #tag_id = #tag_lit;
                                #( #live_outer = #live_inner; )*
                                #( #inert_outer = #inert_defaults; )*
                            }
                        }
                    })
                    .collect();

                let decls = quote! {
                    let #tag_id: jni::sys::jint;
                    #( let #outer_ids: #outer_tys; )*
                };
                if !*optional {
                    preludes.extend(quote! {
                        #decls
                        match &#value { #(#arm_code)* }
                    });
                } else {
                    // `Option<sum>` keeps its own present flag ahead of the
                    // tag: optionality and choice are independent facts.
                    let flag_id = format_ident!("__{}_present", base);
                    let sbind = format_ident!("__o{}", depth);
                    let inner_arms: Vec<TokenStream> =
                        arm_code.iter().map(|a| quote! { #a }).collect();
                    // Destructured through a coercion site: `kind` says this
                    // field is optional, and how Rust spells that is the
                    // source's business (#268).
                    let obind = format_ident!("__oc{}", depth);
                    let coerce =
                        prebindgen_registry::unfold::bind_as_option(&quote!(&#value), &obind);
                    preludes.extend(quote! {
                        let #flag_id: jni::sys::jboolean;
                        #decls
                        #coerce
                        match #obind {
                            ::core::option::Option::Some(#sbind) => {
                                #flag_id = 1u8;
                                match #sbind { #(#inner_arms)* }
                            }
                            ::core::option::Option::None => {
                                #flag_id = 0u8;
                                #tag_id = 0i32;
                                #( #outer_ids = #defaults; )*
                            }
                        }
                    });
                    // One level inside the struct, like the tag it gates and
                    // like an optional nested class's own flag: it is reached
                    // THROUGH this field. The decomposition says so by joining
                    // the names with `__`, and this said `depth` — a
                    // disagreement no shape reached while `Option<sum>` was
                    // refused a decomposition to disagree with.
                    slots.push(EncSlot {
                        depth: depth + 1,
                        ident: flag_id,
                        wire_ty: quote!(jni::sys::jboolean),
                        descriptor: "Z".to_string(),
                        is_object: false,
                        default: quote!(0u8),
                    });
                }
                // A sum field's slots sit one level inside the struct, the
                // same as an inlined nested class's: the tag and the groups
                // are reached THROUGH the field. The decomposition says so by
                // joining the names with `__`, and depth is that count.
                slots.push(EncSlot {
                    depth: depth + 1,
                    ident: tag_id,
                    wire_ty: quote!(jni::sys::jint),
                    descriptor: "I".to_string(),
                    is_object: false,
                    default: quote!(0i32),
                });
                for (i, sl) in all.iter().enumerate() {
                    slots.push(EncSlot {
                        depth: sl.depth,
                        ident: outer_ids[i].clone(),
                        wire_ty: sl.wire_ty.clone(),
                        descriptor: sl.descriptor.clone(),
                        is_object: sl.is_object,
                        default: sl.default.clone(),
                    });
                }
            }
            // Simple leaf: bind per the plan's wire form.
            PlanFieldKind::Leaf {
                conv,
                wire,
                form,
                descriptor,
                ..
            } => {
                let value_expr = conv_value(conv);
                match form {
                    LeafForm::Prim => {
                        preludes.extend(quote! { let #id: #wire = #value_expr; });
                        slots.push(EncSlot {
                            depth,
                            ident: id,
                            wire_ty: quote!(#wire),
                            descriptor: descriptor.clone(),
                            is_object: false,
                            default: primitive_default_for_descriptor(descriptor),
                        });
                    }
                    LeafForm::IntoObject => {
                        preludes.extend(
                            quote! { let #id: jni::objects::JObject = #value_expr.into(); },
                        );
                        slots.push(EncSlot {
                            depth,
                            ident: id,
                            wire_ty: quote!(jni::objects::JObject),
                            descriptor: descriptor.clone(),
                            is_object: true,
                            default: quote!(jni::objects::JObject::null()),
                        });
                    }
                    LeafForm::Object => {
                        preludes.extend(quote! { let #id: jni::objects::JObject = #value_expr; });
                        slots.push(EncSlot {
                            depth,
                            ident: id,
                            wire_ty: quote!(jni::objects::JObject),
                            descriptor: descriptor.clone(),
                            is_object: true,
                            default: quote!(jni::objects::JObject::null()),
                        });
                    }
                }
            }
        }
    }
    (preludes, slots)
}

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
