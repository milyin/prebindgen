//! Struct outputs: `fromParts` leaf encoding and the value-struct
//! synthesis probe.

use super::*;

/// Resolve the typed-handle Kotlin FQN for a handle-bearing struct field
/// and assert its folded strategy is one the struct encode/decode bridge
/// supports. Today only scalar handle slots (`Direct`, optionally wrapped
/// in `Nullable`) are encodable as a single `L<FQN>;` ctor arg; a
/// collection layer (`Iterable`, i.e. `Vec<Handle>`) would need array
/// codegen and is a loud build-time error until implemented.
pub(crate) fn handle_field_fqn(ext: &JniGen, h: &Projection) -> String {
    fn assert_scalar(s: &FoldStrategy) {
        match s {
            FoldStrategy::Base => {}
            FoldStrategy::Optional(_, inner) => assert_scalar(inner),
            FoldStrategy::Iterable(_) => panic!(
                "struct handle field: collection (Vec<Handle>) layers are not yet \
                 supported by the struct encode/decode bridge — add array codegen \
                 to struct_output_body/struct_input_body to lift this guard"
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

/// Synthesize the [`LeafSource::Field`](crate::api::core::unfold::LeafSource)
/// leaves of a by-value `data_class` for the fixed-builder output/callback path
/// — the pre-resolve analog of [`flatten_struct_encode`] (which runs at emit
/// time). Each named field becomes one field-access leaf
/// (`name`, `path = [..field idents]`, `out_ty = <field type>`); a non-optional
/// nested data-class field recurses (inlined), so the whole graph crosses as
/// decoupled leaves the foreign side reassembles.
///
/// Returns `None` (⇒ the type keeps the whole-value `fromParts` path) when a
/// field needs a transform this fixed builder can't yet forward verbatim — a
/// **projection** (opaque handle / value blob), an **enum**, or a nested
/// data-class behind `Option` / `Vec`. (Those are handled by the slower
/// [`struct_output_body`] until the synthesizer is widened to wrap them.)
///
/// Classification reads only `ext.types` (`opaque`/`enum_cfg`/`value_blob`) and
/// `registry.structs` — both populated before `resolve` — never the output
/// converter table (not yet built at this stage).
pub(crate) fn synth_value_struct_leaves(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    s: &syn::ItemStruct,
    path_prefix: &[syn::Ident],
    name_prefix: &str,
    depth: usize,
) -> Option<Vec<crate::api::core::unfold::UnfoldLeaf>> {
    use crate::api::core::unfold::{LeafSource, UnfoldLeaf};
    if depth > 16 {
        return None;
    }
    let syn::Fields::Named(named) = &s.fields else {
        return None;
    };
    let mut leaves: Vec<UnfoldLeaf> = Vec::new();
    for field in &named.named {
        let fname = field.ident.as_ref()?.clone();
        let effective_ty = field.ty.clone();
        let camel = kt_snake_to_camel(&fname.to_string());
        let leaf_name = if name_prefix.is_empty() {
            camel
        } else {
            format!("{name_prefix}__{camel}")
        };
        let mut path = path_prefix.to_vec();
        path.push(fname);

        // A projection field (opaque handle / `value_blob`) or an enum field
        // is delivered with a transform the fixed builder can't forward yet.
        // A nested data-class field (a *declared* plain struct) inlines when
        // non-optional (recurse); `Option`/`Vec`-wrapped nesting is deferred
        // to the whole-value path.
        let probe = option_inner_type(&effective_ty).unwrap_or_else(|| effective_ty.clone());
        let nested = match ext.type_kind(registry, &probe) {
            TypeKind::Handle | TypeKind::Enum | TypeKind::ValueBlob => return None,
            TypeKind::DataStruct { st, cfg: Some(_) } => Some(st.clone()),
            _ => None,
        };
        if let Some(child) = nested {
            if option_inner_type(&effective_ty).is_some() || pat_match_top(&effective_ty, "Vec") {
                return None;
            }
            let child_leaves =
                synth_value_struct_leaves(ext, registry, &child, &path, &leaf_name, depth + 1)?;
            leaves.extend(child_leaves);
            continue;
        }

        // Simple leaf: scalar / String / Option<Box<String>> / ByteArray / Vec.
        // The field's own output converter (resolved later) encodes it; the
        // foreign `fromParts` forwards it verbatim. Nullability is carried by
        // the converter (e.g. `Option<Box<String>>` → `String?`), so the leaf
        // itself isn't path-nullable.
        leaves.push(UnfoldLeaf {
            name: leaf_name,
            path,
            out_ty: effective_ty,
            identity: false,
            nullable: false,
            source: LeafSource::Field,
        });
    }
    Some(leaves)
}

/// Recursively flatten a struct's output encode into a list of leaf wire
/// slots plus the preludes that compute them, so the whole object graph can
/// be built by a **single** Kotlin `fromParts` call (no per-nested-struct
/// `call_static_method`). Nested non-optional data-class fields are inlined;
/// nested `Option<data-class>` fields emit a `present` `jboolean` slot followed
/// by the child's leaves (encoded in the `Some` arm, defaulted in the `None`
/// arm). Leaves (primitives, handles→`jlong`, value classes/blobs→`ByteArray`,
/// enums→`jint`, strings, `Vec`) terminate the recursion.
///
/// The field classification is the shared [`build_struct_plan`] — the same
/// plan `flatten_struct_factory` walks for the Kotlin side, so the slot
/// order and JVM descriptors agree by construction.
pub(crate) fn flatten_struct_encode(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    s: &syn::ItemStruct,
    access: &TokenStream,
    prefix: &str,
    depth: usize,
    env_expr: &TokenStream,
) -> Option<(TokenStream, Vec<EncSlot>)> {
    let plan = build_struct_plan(ext, registry, s, depth)?;
    Some(encode_plan(&plan, access, prefix, depth, env_expr))
}

/// Walk a [`StructPlan`] emitting the Rust-side wire encode: per leaf a
/// prelude statement binding `__<prefix>_<field>` to the converted wire and
/// an [`EncSlot`] describing its `JValue` slot. `access` is the Rust
/// expression yielding the current struct value (`v`, `v.field`, or the
/// matched `__cN` under an Option); `prefix` namespaces the generated idents.
fn encode_plan(
    plan: &StructPlan,
    access: &TokenStream,
    prefix: &str,
    depth: usize,
    env_expr: &TokenStream,
) -> (TokenStream, Vec<EncSlot>) {
    let mut preludes = TokenStream::new();
    let mut slots: Vec<EncSlot> = Vec::new();

    for f in &plan.fields {
        let fname = &f.fname;
        let base = format!("{}_{}", prefix, fname);
        let id = format_ident!("__{}", base);
        let conv_value = |conv: &syn::Ident| -> TokenStream {
            quote! { #conv(#env_expr, #access.#fname.clone())? }
        };
        match &f.kind {
            // Projection leaf (opaque handle → jlong, value class / blob → ByteArray).
            PlanFieldKind::Projection { conv, proj, .. } => {
                let value_expr = conv_value(conv);
                match proj.kind {
                    ProjectionKind::Handle => {
                        preludes.extend(quote! { let #id: jni::sys::jlong = #value_expr; });
                        slots.push(EncSlot {
                            ident: id,
                            wire_ty: quote!(jni::sys::jlong),
                            descriptor: "J".to_string(),
                            is_object: false,
                            default: quote!(0i64),
                        });
                    }
                    ProjectionKind::ValueBlob => {
                        preludes.extend(
                            quote! { let #id: jni::objects::JObject = { #value_expr }.into(); },
                        );
                        slots.push(EncSlot {
                            ident: id,
                            wire_ty: quote!(jni::objects::JObject),
                            descriptor: "[B".to_string(),
                            is_object: true,
                            default: quote!(jni::objects::JObject::null()),
                        });
                    }
                }
            }
            // Enum leaf → jint discriminant (Kotlin `fromParts` calls `fromInt`).
            PlanFieldKind::Enum { conv, .. } => {
                let value_expr = conv_value(conv);
                preludes.extend(quote! { let #id: jni::sys::jint = #value_expr; });
                slots.push(EncSlot {
                    ident: id,
                    wire_ty: quote!(jni::sys::jint),
                    descriptor: "I".to_string(),
                    is_object: false,
                    default: quote!(0i32),
                });
            }
            // `Option<enum>` leaf → the converter delivers the `box_jint`-boxed
            // discriminant (JVM null = `None`); the slot is the box class.
            PlanFieldKind::OptionEnum { conv, .. } => {
                let value_expr = conv_value(conv);
                preludes.extend(quote! { let #id: jni::objects::JObject = #value_expr; });
                slots.push(EncSlot {
                    ident: id,
                    wire_ty: quote!(jni::objects::JObject),
                    descriptor: "Ljava/lang/Integer;".to_string(),
                    is_object: true,
                    default: quote!(jni::objects::JObject::null()),
                });
            }
            // Nested data-class: inline the child's leaves; under `Option` add
            // a `present` flag and default the child slots in the `None` arm.
            PlanFieldKind::Nested {
                optional,
                plan: child,
                ..
            } => {
                if !*optional {
                    let child_access = quote! { #access.#fname };
                    let (child_pre, child_slots) =
                        encode_plan(child, &child_access, &base, depth + 1, env_expr);
                    preludes.extend(child_pre);
                    slots.extend(child_slots);
                } else {
                    let cbind = format_ident!("__c{}", depth);
                    let child_access = quote! { #cbind };
                    let (child_pre, child_slots) =
                        encode_plan(child, &child_access, &base, depth + 1, env_expr);
                    let flag_id = format_ident!("__{}_present", base);
                    let outer_ids: Vec<proc_macro2::Ident> = (0..child_slots.len())
                        .map(|i| format_ident!("__{}_o{}", base, i))
                        .collect();
                    let outer_tys: Vec<TokenStream> =
                        child_slots.iter().map(|sl| sl.wire_ty.clone()).collect();
                    let inner_ids: Vec<proc_macro2::Ident> =
                        child_slots.iter().map(|sl| sl.ident.clone()).collect();
                    let defaults: Vec<TokenStream> =
                        child_slots.iter().map(|sl| sl.default.clone()).collect();
                    preludes.extend(quote! {
                        let #flag_id: jni::sys::jboolean;
                        #( let #outer_ids: #outer_tys; )*
                        match &#access.#fname {
                            Some(#cbind) => {
                                #child_pre
                                #flag_id = 1u8;
                                #( #outer_ids = #inner_ids; )*
                            }
                            None => {
                                #flag_id = 0u8;
                                #( #outer_ids = #defaults; )*
                            }
                        }
                    });
                    slots.push(EncSlot {
                        ident: flag_id,
                        wire_ty: quote!(jni::sys::jboolean),
                        descriptor: "Z".to_string(),
                        is_object: false,
                        default: quote!(0u8),
                    });
                    for (i, sl) in child_slots.iter().enumerate() {
                        slots.push(EncSlot {
                            ident: outer_ids[i].clone(),
                            wire_ty: sl.wire_ty.clone(),
                            descriptor: sl.descriptor.clone(),
                            is_object: sl.is_object,
                            default: sl.default.clone(),
                        });
                    }
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

pub(crate) fn struct_output_body(
    ext: &JniGen,
    s: &syn::ItemStruct,
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr)> {
    let struct_name = s.ident.to_string();
    // Prefer the registered Kotlin FQN (`io.zenoh.jni.JniSample`) so the
    // mangle closure flows through; fall back to the bare struct ident
    // qualified with the package when no `data_class` /
    // `ptr_class` declaration exists for this Rust type.
    let struct_ident = &s.ident;
    let struct_ty: syn::Type = syn::parse_quote!(#struct_ident);
    let registered_fqn = ext
        .types
        .get(&TypeKey::from_type(&struct_ty))
        .and_then(|cfg| cfg.kotlin_name.clone());
    let java_class_name = if let Some(fqn) = registered_fqn {
        fqn.replace('.', "/")
    } else if ext.java_class_prefix.is_empty() {
        struct_name.clone()
    } else {
        format!("{}/{}", ext.java_class_prefix, struct_name)
    };

    // Recursively flatten the whole object graph into leaf wires, then build it
    // with ONE `call_static_method("fromParts", …)` — no per-nested-struct JNI
    // crossing. The Kotlin `fromParts` factory (recursively flattened the same
    // way in `render_data_class_source`) reassembles the graph in bytecode.
    let access = quote!(v);
    let (preludes, slots) = flatten_struct_encode(ext, registry, s, &access, "", 0, &quote!(env))?;

    let mut sig = String::from("(");
    let mut args: Vec<TokenStream> = Vec::new();
    for sl in &slots {
        sig.push_str(&sl.descriptor);
        let id = &sl.ident;
        if sl.is_object {
            args.push(quote!(jni::objects::JValue::Object(&#id)));
        } else {
            args.push(quote!(jni::objects::JValue::from(#id)));
        }
    }
    sig.push_str(&format!(")L{};", java_class_name));
    let factory_sig_lit = syn::LitStr::new(&sig, Span::call_site());

    let body: syn::Expr = syn::parse_quote!({
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
    });
    Some((syn::parse_quote!(jni::objects::JObject), body))
}

pub(crate) fn struct_module_path(ext: &JniGen, s: &syn::ItemStruct) -> syn::Path {
    // Place the struct under <source_module>::<file_stem>::<Name>. Today's
    // pipeline derives the module from the source file stem; here we ride
    // on the same convention by inspecting the SourceLocation. Without a
    // location handy at this stage we fall back to <source_module>::<Name>.
    // In practice the actual file stem is added in the compose step at the
    // call site by the consuming crate when needed.
    let _ = s;
    ext.source_module.clone()
}

// ──────────────────────────────────────────────────────────────────────
// Enum rank-0 bodies
// ──────────────────────────────────────────────────────────────────────
