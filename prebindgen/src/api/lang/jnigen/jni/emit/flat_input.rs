//! Flattenable `data_class` inputs: leaf plans, Kotlin destructure
//! expressions, and the Rust-side reconstruct.

use super::*;

pub(crate) fn struct_input_body(
    ext: &JniGen,
    s: &syn::ItemStruct,
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr)> {
    let struct_name = s.ident.to_string();
    let struct_module = struct_module_path(ext, registry, s);
    let struct_ident = &s.ident;

    let syn::Fields::Named(named) = &s.fields else {
        return None;
    };

    let mut field_preludes: Vec<TokenStream> = Vec::new();
    let mut field_init: Vec<TokenStream> = Vec::new();

    for field in &named.named {
        let fname_ident = field.ident.as_ref().unwrap().clone();
        let fname = fname_ident.to_string();
        let camel = snake_to_camel(&fname);
        let err_prefix = format!("{struct_name}.{camel}: {{}}");
        let raw_ident = format_ident!("__{}_raw", fname_ident);

        // Defer if any field's input converter isn't resolved yet — the
        // fixed-point loop will retry on the next iteration.
        let field_entry = registry.input_entry(&field.ty)?;
        let field_wire = field_entry.destination.clone();
        let field_conv = field_entry.function.sig.ident.clone();

        // Projection fields — mirror of `struct_output_body`'s kind branch:
        //  * Handle: read the JNINativeHandle object from the JVM slot,
        //    `peek()` the raw jlong, then run the per-field input converter
        //    (jlong-keyed; null handle ⇒ jlong 0 ⇒ `None` via the niche path).
        //  * ValueBlob: the class is JVM-erased to its `bytes: ByteArray`, so
        //    the slot is the `[B` descriptor; read it as a JObject, coerce to
        //    the inner wire, and run the per-field converter. (Without this
        //    branch a value-blob field would be mis-decoded as a handle —
        //    peeking a non-handle object.)
        if let Some(proj) = &field_entry.metadata.projection {
            match proj.kind {
                ProjectionKind::Handle => {
                    let java_path = handle_field_fqn(ext, proj).replace('.', "/");
                    let sig = format!("L{};", java_path);
                    let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                    // Struct fields are owned, so a non-`Option` handle field
                    // owns its native object: decode by consuming
                    // (`Box::from_raw` → owned `T`), mirroring
                    // `struct_output_body`'s `Box::into_raw`. The borrow
                    // converter would yield `OwnedObject<T>`, which can't
                    // populate an owned field. `Option<_>` handle fields keep
                    // the niche-aware converter (jlong 0 ⇒ `None`).
                    let field_ty = &field.ty;
                    let field_is_option = matches!(
                        field_ty,
                        syn::Type::Path(p) if p.path.segments.last()
                            .map(|s| s.ident == "Option").unwrap_or(false)
                    );
                    let decode = if field_is_option {
                        quote! { let #fname_ident = #field_conv(env, &#raw_ident)?; }
                    } else {
                        quote! {
                            // Null or closed handle in a required field —
                            // reject before any dereference (`peek()`
                            // normalizes closed handles to 0).
                            if #raw_ident == 0 || (#raw_ident & 1) == 1 {
                                return ::core::result::Result::Err(
                                    <__JniErr as ::core::convert::From<String>>::from(
                                        "Operation on a closed native handle.".to_string(),
                                    ),
                                );
                            }
                            let #fname_ident: #field_ty = unsafe {
                                *std::boxed::Box::from_raw(#raw_ident as *mut #field_ty)
                            };
                        }
                    };
                    field_preludes.push(quote! {
                        let #tmp_ident: jni::objects::JObject = env.get_field(v, #camel, #sig)
                            .and_then(|val| val.l())
                            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                        let #raw_ident: jni::sys::jlong = if #tmp_ident.is_null() {
                            0
                        } else {
                            env.call_method(&#tmp_ident, "peek", "()J", &[])
                                .and_then(|val| val.j())
                                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?
                        };
                        #decode
                    });
                }
                ProjectionKind::ValueBlob => {
                    let descriptor = "[B";
                    let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                    field_preludes.push(quote! {
                        let #tmp_ident: jni::objects::JObject = env.get_field(v, #camel, #descriptor)
                            .and_then(|val| val.l())
                            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                        let #raw_ident: #field_wire = #tmp_ident.into();
                        let #fname_ident = #field_conv(env, &#raw_ident)?;
                    });
                }
            }
            field_init.push(quote!(#fname_ident));
            continue;
        }

        // Enum-typed field (bare or `Option`-wrapped): the Kotlin data class
        // stores the TYPED enum object (`Priority` / `Priority?`), so read the
        // slot with the enum-class descriptor and decode the discriminant via
        // its `value` getter (`getValue()I`); a null object is the `None` arm.
        // (The generic converters can't be used here: the bare-enum one is
        // jint-keyed, the `Option<enum>` one unboxes `java.lang.Integer`.)
        let f_inner = option_inner_type(&field.ty).unwrap_or_else(|| field.ty.clone());
        if ext.is_kotlin_enum(&f_inner) {
            if let Some(fqn) = bare_path_ident(&f_inner)
                .and_then(|n| ext.kotlin_fqn(&n.to_string()))
                .map(|v| v.to_string())
            {
                let sig = format!("L{};", fqn.replace('.', "/"));
                let inner_conv = registry.input_entry(&f_inner)?.function.sig.ident.clone();
                let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                let decode = if option_inner_type(&field.ty).is_some() {
                    quote! {
                        let #fname_ident = if #tmp_ident.is_null() {
                            ::core::option::Option::None
                        } else {
                            let #raw_ident: jni::sys::jint = env.call_method(&#tmp_ident, "getValue", "()I", &[])
                                .and_then(|val| val.i())
                                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                            ::core::option::Option::Some(#inner_conv(env, &#raw_ident)?)
                        };
                    }
                } else {
                    quote! {
                        let #raw_ident: jni::sys::jint = env.call_method(&#tmp_ident, "getValue", "()I", &[])
                            .and_then(|val| val.i())
                            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                        let #fname_ident = #inner_conv(env, &#raw_ident)?;
                    }
                };
                field_preludes.push(quote! {
                    let #tmp_ident: jni::objects::JObject = env.get_field(v, #camel, #sig)
                        .and_then(|val| val.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    #decode
                });
                field_init.push(quote!(#fname_ident));
                continue;
            }
        }

        match jni_field_access(&field_wire) {
            Some((sig, accessor, false)) => {
                field_preludes.push(quote! {
                    let #raw_ident: #field_wire = env.get_field(v, #camel, #sig)
                        .and_then(|val| val.#accessor())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))? as _;
                    let #fname_ident = #field_conv(env, &#raw_ident)?;
                });
            }
            Some((sig, _, true)) => {
                let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                field_preludes.push(quote! {
                    let #tmp_ident: jni::objects::JObject = env.get_field(v, #camel, #sig)
                        .and_then(|val| val.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    let #raw_ident: #field_wire = #tmp_ident.into();
                    let #fname_ident = #field_conv(env, &#raw_ident)?;
                });
            }
            None => {
                // Wire is JObject — fetch via .l() and pass by reference. JNI
                // `GetFieldID` needs the slot's EXACT static descriptor: the
                // box class for an `Option`-boxed primitive, the registered
                // Kotlin class for a nested data-class field (Option-stripped
                // — a nullable field keeps the same descriptor), `List` for a
                // `Vec` field.
                let slot_ty = option_inner_type(&field.ty).unwrap_or_else(|| field.ty.clone());
                let sig = registry
                    .input_entry(&slot_ty)
                    .and_then(|e| jni_field_access(&e.destination))
                    .and_then(|(sig, _, is_obj)| {
                        if is_obj {
                            Some(sig.to_string())
                        } else {
                            box_descriptor_for_primitive(sig).map(str::to_string)
                        }
                    })
                    .or_else(|| {
                        bare_path_ident(&slot_ty).and_then(|name| {
                            ext.kotlin_fqn(&name.to_string())
                                .map(|v| format!("L{};", v.replace('.', "/")))
                        })
                    })
                    .or_else(|| {
                        if pat_match_top(&slot_ty, "Vec") {
                            Some("Ljava/util/List;".to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "Ljava/lang/Object;".to_string());
                field_preludes.push(quote! {
                    let #raw_ident: jni::objects::JObject = env.get_field(v, #camel, #sig)
                        .and_then(|val| val.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    let #fname_ident = #field_conv(env, &#raw_ident)?;
                });
            }
        }
        field_init.push(quote!(#fname_ident));
    }

    let body: syn::Expr = syn::parse_quote!({
        #(#field_preludes)*
        #struct_module::#struct_ident { #(#field_init),* }
    });
    Some((syn::parse_quote!(jni::objects::JObject), body))
}

// ──────────────────────────────────────────────────────────────────────
// Struct input flattening (pass a data_class param as its leaf fields)
// ──────────────────────────────────────────────────────────────────────

/// One flattened leaf of a struct **input** param. The mirror of
/// [`EncSlot`] for the input direction: instead of reading the field with
/// `env.get_field(...)` out of a single `JObject`, the leaf crosses the JNI
/// boundary as its own wrapper parameter. Carries every fact the three
/// coordinated sites (native wrapper signature, `JNINative` extern decl,
/// Kotlin call-site destructure) need so they cannot drift in order, type, or
/// nullability.
pub(crate) struct FlatLeaf {
    /// Native wrapper parameter ident — also the decode source.
    pub native_ident: syn::Ident,
    /// Native wire type (lifetime-annotated for object wires).
    pub native_wire_ty: TokenStream,
    /// Kotlin `external fun` parameter name (camelCase).
    pub kt_name: String,
    /// Kotlin `external fun` parameter type (incl. a trailing `?`).
    pub kt_wire_ty: String,
    /// Kotlin call-site destructure expression feeding this leaf.
    pub kt_access: String,
    /// Per-field input converter ident (`None` for the synthetic present flag).
    pub conv: Option<syn::Ident>,
    /// Struct field this leaf populates. `None` for the struct-level present
    /// flag of an `Option<struct>` param. `Some` for ordinary field leaves AND
    /// for a per-field present flag (Phase 5 `Option<primitive>` field).
    pub field: Option<syn::Ident>,
    /// `true` for a synthetic `…Present: Boolean` gate leaf: the struct-level
    /// gate of an `Option<struct>` param (`field == None`) or a per-field gate
    /// of an `Option<primitive>`/`Option<enum>` field (`field == Some`).
    pub is_present_flag: bool,
    /// `true` for the **value** leaf of an `Option<primitive>`/`Option<enum>`
    /// field — its `field`'s reconstruct is `if <field>_present { Some(conv(v)) }
    /// else { None }`, gated by the matching per-field present leaf. (Phase 5.)
    pub opt_scalar: bool,
}

/// A flattened plan for one struct input parameter. Built once by
/// [`build_flat_input_plan`] and consumed by all three codegen sites.
pub(crate) struct FlatInputPlan {
    pub leaves: Vec<FlatLeaf>,
    /// Module path the struct lives under (`zenoh_flat`).
    pub struct_module: syn::Path,
    /// Struct ident (`Encoding`).
    pub struct_ident: syn::Ident,
    /// `true` when the original param was `Option<…>` — leaves are gated on a
    /// `present` flag and decoded lazily.
    pub optional: bool,
    /// `true` when the source fn takes `&Struct` — the call site passes `&arg`.
    pub by_ref: bool,
    /// The present-flag param ident (`Some` iff `optional`).
    pub present_ident: Option<syn::Ident>,
}

/// Extract `S` from an `impl Into<S> + …` parameter type.
pub(crate) fn impl_into_target(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::ImplTrait(it) = ty else {
        return None;
    };
    for b in &it.bounds {
        if let syn::TypeParamBound::Trait(tb) = b {
            if let Some(seg) = tb.path.segments.last() {
                if seg.ident == "Into" {
                    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(t)) = ab.args.first() {
                            return Some(t.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Peel a leading `&`/`&mut` then an `Option<…>` to expose the inner type used
/// for enum/struct detection (`&Priority`, `Option<Priority>` → `Priority`).
pub(crate) fn flat_probe_inner(ty: &syn::Type) -> syn::Type {
    let stripped = match ty {
        syn::Type::Reference(r) => (*r.elem).clone(),
        other => other.clone(),
    };
    option_inner_type(&stripped).unwrap_or(stripped)
}

/// Kotlin literal that fills a leaf slot when its `Option<struct>` parent is
/// absent (the `present` flag tells Rust to ignore it). `None` for nullable
/// leaves, which simply ride a JVM `null`. Mirrors
/// [`primitive_default_for_descriptor`] on the Rust side.
pub(crate) fn kt_leaf_default(sig: &str, nullable: bool) -> Option<String> {
    if nullable {
        return None;
    }
    Some(
        match sig {
            "Z" => "false",
            "B" | "S" | "I" => "0",
            "C" => "'\\u0000'",
            "J" => "0L",
            "F" => "0.0f",
            "D" => "0.0",
            "Ljava/lang/String;" => "\"\"",
            "[B" => "ByteArray(0)",
            _ => "null",
        }
        .to_string(),
    )
}

/// Decompose an `Option<primitive>` / `Option<enum>` **struct field** into a
/// `(<field>_present: Boolean, <field>_value: <prim>)` [`FlatLeaf`] pair, or
/// `None` to keep the boxed-`JObject` decline. The field-level dual of
/// [`build_option_scalar_input_plan`]: same boxed-fallback condition (primitive
/// inner wire, no niche, no projection, no pre-stages). `kt_param` is the call
/// object expression and `optional` whether the enclosing struct param is itself
/// `Option<struct>` — when so, the access safe-navigates (`obj?.field`), so an
/// absent struct yields `present = false` for every field.
#[allow(clippy::too_many_arguments)]
pub(crate) fn option_scalar_field_leaves(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    param_name: &syn::Ident,
    kt_param: &str,
    optional: bool,
    fident: &syn::Ident,
    fcamel: &str,
    field_ty: &syn::Type,
) -> Option<Vec<FlatLeaf>> {
    let inner = option_inner_type(field_ty)?;
    if matches!(inner, syn::Type::Reference(_)) {
        return None;
    }
    let ie = registry.input_entry(&inner)?;
    let value_wire = ie.destination.clone();
    let prim = JniPrim::from_wire(&value_wire)?;
    if ie.niches.clone().carve().is_some()
        || ie.metadata.projection.is_some()
        || !ie.pre_stages.is_empty()
    {
        return None;
    }
    let is_enum = ext.is_kotlin_enum(&inner);
    // `obj.field` / `obj?.field` (safe-nav under an absent `Option<struct>`).
    let field_ref = if optional {
        format!("{kt_param}?.{fcamel}")
    } else {
        format!("{kt_param}.{fcamel}")
    };
    let present_access = format!("{field_ref} != null");
    let value_access = if is_enum {
        format!("{field_ref}?.value ?: {}", prim.kotlin_zero())
    } else {
        format!("{field_ref} ?: {}", prim.kotlin_zero())
    };
    let pres_ident = format_ident!("{}_{}_present", param_name, fident);
    let val_ident = format_ident!("{}_{}_value", param_name, fident);
    Some(vec![
        FlatLeaf {
            native_ident: pres_ident,
            native_wire_ty: quote!(jni::sys::jboolean),
            kt_name: snake_to_camel(&format!("{}_{}_present", param_name, fident)),
            kt_wire_ty: "Boolean".to_string(),
            kt_access: present_access,
            conv: None,
            field: Some(fident.clone()),
            is_present_flag: true,
            opt_scalar: false,
        },
        FlatLeaf {
            native_ident: val_ident,
            native_wire_ty: quote!(#value_wire),
            kt_name: snake_to_camel(&format!("{}_{}_value", param_name, fident)),
            kt_wire_ty: prim.kotlin_type().to_string(),
            kt_access: value_access,
            conv: Some(ie.function.sig.ident.clone()),
            field: Some(fident.clone()),
            is_present_flag: false,
            opt_scalar: true,
        },
    ])
}

/// Build a [`FlatInputPlan`] for a struct input parameter, or `None` to keep
/// the existing single-`JObject` path. Returns `None` (safe fallback) for any
/// shape outside the conservative v1 leaf set — handle/value projections,
/// enums, nested data classes, `Vec<non-u8>`,
/// converters with `pre_stages`, and `impl Into<S>` dispatch (`Any`). This is
/// the single source of truth shared by the native wrapper signature, the
/// `JNINative` extern declaration, and the Kotlin call-site destructure.
pub(crate) fn build_flat_input_plan(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    param_name: &syn::Ident,
    arg_ty: &syn::Type,
    kt_base: &str,
) -> Option<FlatInputPlan> {
    // 1. Resolve the struct target through `&`, `Option<…>`, and `impl Into<S>`.
    let (by_ref, t1) = match arg_ty {
        syn::Type::Reference(r) => (true, (*r.elem).clone()),
        other => (false, other.clone()),
    };
    let (optional, inner) = match option_inner_type(&t1) {
        Some(i) => (true, i),
        None => (false, t1.clone()),
    };
    let struct_ty = impl_into_target(&inner).unwrap_or_else(|| inner.clone());
    let name = bare_path_ident(&struct_ty)?;
    let (st, _) = registry.structs.get(&name)?;
    let key = TypeKey::from_type(&struct_ty);
    let cfg = ext.types.get(&key);
    // Exclude value-blob / enum structs — they have their own
    // erasure and are not field-flattened here.
    if matches!(
        ext.type_kind(registry, &struct_ty),
        TypeKind::ValueBlob | TypeKind::Enum
    ) {
        return None;
    }
    // Identity / pass-through guard: the resolved param must decode to the
    // struct itself, not an opaque handle / value projection (`projection`
    // present) and not a multi-source / non-identity `impl Into<S>` (which
    // surfaces as `"Any"` Dispatch or a foreign source type). The resolved
    // param's Kotlin type (compared by short name, since metadata carries the
    // FQN) must equal the struct's data-class name.
    let entry = registry.input_entry(arg_ty)?;
    if entry.metadata.projection.is_some() {
        return None;
    }
    let dc_short = cfg
        .and_then(|c| c.name_spec.as_ref())
        .map(|s| ext.fqn_of(s))
        .map(|fqn| fqn.rsplit('.').next().unwrap_or(&fqn).to_string())
        .unwrap_or_else(|| name.to_string());
    let entry_short = entry
        .metadata
        .kotlin_name
        .as_ref()
        .and_then(|t| t.simple_name());
    if entry_short != Some(dc_short.as_str()) {
        return None;
    }

    // 2. Named fields only.
    let syn::Fields::Named(named) = &st.fields else {
        return None;
    };

    // 3. Classify every field as a simple leaf, else fall back.
    let struct_module = struct_module_path(ext, registry, st);
    // `kt_base` is the Kotlin expression for the object at the call site —
    // normally the camelCase param name, or `this` for a promoted instance
    // receiver. The native param idents / extern names stay keyed on
    // `param_name` so the wire signature is independent of the call form.
    let kt_param = kt_base.to_string();
    let mut leaves: Vec<FlatLeaf> = Vec::new();

    // Present gate for `Option<struct>` (first leaf, mirrors the output
    // `Option<nested>` `present: jboolean` slot).
    let present_ident = if optional {
        let id = format_ident!("{}_present", param_name);
        leaves.push(FlatLeaf {
            native_ident: id.clone(),
            native_wire_ty: quote!(jni::sys::jboolean),
            kt_name: snake_to_camel(&format!("{}_present", param_name)),
            kt_wire_ty: "Boolean".to_string(),
            kt_access: format!("{kt_param} != null"),
            conv: None,
            field: None,
            is_present_flag: true,
            opt_scalar: false,
        });
        Some(id)
    } else {
        None
    };

    for field in &named.named {
        let fident = field.ident.clone()?;
        let fcamel = snake_to_camel(&fident.to_string());

        // Phase 5: an `Option<primitive>` / `Option<enum>` field that would
        // otherwise box into a `java.lang.*` `JObject` (declining the whole
        // struct) crosses instead as a `(<field>_present: Boolean,
        // <field>_value: <prim>)` leaf pair — the field dual of the param-level
        // `OptionScalarInputPlan`. The Rust reconstruct rebuilds the `Option`
        // from the two raw scalars (no `intValue()` unbox). Detected before the
        // enum-decline guard below so `Option<enum>` fields take this path too.
        if let Some(pair) = option_scalar_field_leaves(
            ext, registry, param_name, &kt_param, optional, &fident, &fcamel, &field.ty,
        ) {
            leaves.extend(pair);
            continue;
        }

        let fentry = registry.input_entry(&field.ty)?;
        // Reject anything outside the simple-leaf set (keeps the object path).
        if !fentry.pre_stages.is_empty() {
            return None;
        }
        if fentry.metadata.projection.is_some() {
            return None;
        }
        if ext.is_kotlin_enum(&flat_probe_inner(&field.ty)) {
            return None;
        }
        let wire = &fentry.destination;
        let (sig, _accessor, _is_obj) = jni_field_access(wire)?;
        let f_opt = option_inner_type(&field.ty).is_some();
        let kt = fentry.metadata.kotlin_name.clone()?;
        let kt_wire_ty = format!("{}{}", kt, if f_opt { "?" } else { "" });
        let native_ident = format_ident!("{}_{}", param_name, fident);
        let native_wire_ty = annotate_jobject_with_lifetime(wire, "a").to_token_stream();
        let kt_name = snake_to_camel(&format!("{}_{}", param_name, fident));

        // Destructure expression. Under an absent `Option<struct>` parent the
        // leaf still needs a value on the wire (`present` makes Rust ignore
        // it): nullable leaves ride JVM null, non-null leaves a typed default.
        let kt_access = if optional {
            let base = format!("{kt_param}?.{fcamel}");
            match kt_leaf_default(sig, f_opt) {
                Some(def) => format!("{base} ?: {def}"),
                None => base,
            }
        } else {
            format!("{kt_param}.{fcamel}")
        };

        leaves.push(FlatLeaf {
            native_ident,
            native_wire_ty,
            kt_name,
            kt_wire_ty,
            kt_access,
            conv: Some(fentry.function.sig.ident.clone()),
            field: Some(fident),
            is_present_flag: false,
            opt_scalar: false,
        });
    }

    Some(FlatInputPlan {
        leaves,
        struct_module,
        struct_ident: st.ident.clone(),
        optional,
        by_ref,
        present_ident,
    })
}

/// Render the native reconstruct for a [`FlatInputPlan`]: decode each leaf
/// param with its per-field converter (lazily, inside the `present` branch for
/// an `Option<struct>`) and bind the rebuilt struct to `arg_ident`. Each decode
/// failure routes through `signal_error` (the per-call sink) and returns the
/// function `on_err` sentinel. Returns the prelude statements and the call
/// argument (`arg` or `&arg`).
pub(crate) fn render_flat_input_decode(
    plan: &FlatInputPlan,
    arg_ident: &syn::Ident,
    on_err: &TokenStream,
) -> (TokenStream, TokenStream) {
    let module = &plan.struct_module;
    let sid = &plan.struct_ident;
    // Per-field present gates (Phase 5 `Option<primitive>` fields): map the
    // field ident to its `<field>_present` native param ident.
    let field_present: std::collections::HashMap<String, syn::Ident> = plan
        .leaves
        .iter()
        .filter(|l| l.is_present_flag && l.field.is_some())
        .map(|l| (l.field.clone().unwrap().to_string(), l.native_ident.clone()))
        .collect();
    let mut field_decodes: Vec<TokenStream> = Vec::new();
    let mut field_inits: Vec<TokenStream> = Vec::new();
    for leaf in &plan.leaves {
        if leaf.is_present_flag {
            continue;
        }
        let conv = leaf.conv.as_ref().unwrap();
        let wid = &leaf.native_ident;
        let fid = leaf.field.clone().unwrap();
        let tmp = format_ident!("__{}_{}", arg_ident, fid);
        let decode_value = quote! {
            match #conv(&mut env, &#wid) {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__e) => {
                    let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                    return #on_err;
                }
            }
        };
        if leaf.opt_scalar {
            // `Option<primitive>` field: gate the decode on the per-field present
            // flag; an absent field skips the converter entirely.
            let present = field_present.get(&fid.to_string()).unwrap_or_else(|| {
                panic!("opt_scalar value leaf for field `{fid}` has no matching present leaf")
            });
            field_decodes.push(quote! {
                let #tmp = if #present != 0u8 {
                    ::core::option::Option::Some(#decode_value)
                } else {
                    ::core::option::Option::None
                };
            });
        } else {
            field_decodes.push(quote! {
                let #tmp = #decode_value;
            });
        }
        field_inits.push(quote!(#fid: #tmp));
    }
    let build = quote!(#module::#sid { #(#field_inits),* });
    let prelude = if plan.optional {
        let present = plan.present_ident.as_ref().unwrap();
        quote! {
            let #arg_ident = if #present != 0u8 {
                #(#field_decodes)*
                Some(#build)
            } else {
                None
            };
        }
    } else {
        quote! {
            #(#field_decodes)*
            let #arg_ident = #build;
        }
    };
    let call_arg = if plan.by_ref {
        quote!(&#arg_ident)
    } else {
        quote!(#arg_ident)
    };
    (prelude, call_arg)
}

// ──────────────────────────────────────────────────────────────────────
// Bare `Option<primitive>` / `Option<enum>` input → (present, value) leaves
// ──────────────────────────────────────────────────────────────────────

/// A decomposed plan for an `Option<primitive>` / `Option<enum>` **input**
/// parameter that would otherwise box into a `java.lang.*` and cross as a
/// single `JObject` (decoded with a reflective `intValue()`/`longValue()`
/// unbox). Instead the value crosses as a
/// `(<param>_present: jboolean, <param>_value: <wire>)` pair — no boxed object
/// on the wire, and the Rust side reassembles the `Option` from two raw scalars
/// with zero `env.call_method(...)`. The single-scalar dual of
/// [`FlatInputPlan`]'s `Option<struct>` present-gate path.
pub(crate) struct OptionScalarInputPlan {
    /// Native `<param>_present: jboolean` ident.
    pub present_ident: syn::Ident,
    /// Native `<param>_value: <wire>` ident.
    pub value_ident: syn::Ident,
    /// JNI primitive wire of the inner value (`jint`/`jlong`/`jboolean`/…).
    pub value_wire: syn::Type,
    /// Inner converter (`<wire> -> T`), called inside the `present` branch.
    pub inner_conv: syn::Ident,
    /// Kotlin camelCase extern param name for the present flag.
    pub present_kt: String,
    /// Kotlin camelCase extern param name for the value.
    pub value_kt: String,
    /// Non-null Kotlin type of the value leaf (`Int`/`Long`/…) for the extern.
    pub value_kt_type: String,
    /// Kotlin zero literal filling the value leaf when the option is absent.
    pub value_kt_zero: String,
    /// `true` when the inner is an `enum_class` — the call site reads `?.value`.
    pub is_enum: bool,
}

/// Build an [`OptionScalarInputPlan`] for a bare `Option<primitive>` /
/// `Option<enum>` parameter, or `None` to keep the existing single-`JObject`
/// boxed path. Mirrors exactly the boxed-fallback condition of [`option_input`]
/// (primitive inner wire, no niche, no projection, no composed pre-stages) so
/// only the cases that *would* box are intercepted — niche cases (already
/// unboxed / ABI-clean) and opaque/value projections are left untouched.
pub(crate) fn build_option_scalar_input_plan(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    param_name: &syn::Ident,
    arg_ty: &syn::Type,
) -> Option<OptionScalarInputPlan> {
    let inner = option_inner_type(arg_ty)?;
    // `Option<&T>` is the nullable-borrow / handle path, not a scalar.
    if matches!(inner, syn::Type::Reference(_)) {
        return None;
    }
    let inner_entry = registry.input_entry(&inner)?;
    let value_wire = inner_entry.destination.clone();
    // Only the boxed-primitive fallback shape: primitive wire, no niche,
    // no projection, no composed pre-stages.
    let prim = JniPrim::from_wire(&value_wire)?;
    if inner_entry.niches.clone().carve().is_some() {
        return None;
    }
    if inner_entry.metadata.projection.is_some() {
        return None;
    }
    if !inner_entry.pre_stages.is_empty() {
        return None;
    }
    let is_enum = ext.is_kotlin_enum(&inner);
    Some(OptionScalarInputPlan {
        present_ident: format_ident!("{}_present", param_name),
        value_ident: format_ident!("{}_value", param_name),
        value_wire,
        inner_conv: inner_entry.function.sig.ident.clone(),
        present_kt: snake_to_camel(&format!("{}_present", param_name)),
        value_kt: snake_to_camel(&format!("{}_value", param_name)),
        value_kt_type: prim.kotlin_type().to_string(),
        value_kt_zero: prim.kotlin_zero().to_string(),
        is_enum,
    })
}

// ──────────────────────────────────────────────────────────────────────
// Slice / Vec input → Rust-side Vec handle (built by pushing leaves)
// ──────────────────────────────────────────────────────────────────────
