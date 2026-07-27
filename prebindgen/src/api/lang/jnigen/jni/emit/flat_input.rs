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
        let camel = mangle_kotlin_ident(&snake_to_camel(&fname));
        let err_prefix = format!("{struct_name}.{camel}: {{}}");
        let raw_ident = format_ident!("__{}_raw", fname_ident);

        // Defer if any field's input converter isn't resolved yet — the
        // fixed-point loop will retry on the next iteration.
        let field_entry = registry.input_entry(&field.ty)?;
        let field_wire = field_entry.destination.clone();
        // The field's COMPLETE decode, stages included — a `convert!` type
        // reaches its Rust value through them (`jlong -> u64 -> Duration`).
        let field_conv = composed_entry_decode(field_entry, &raw_ident, &fname_ident);

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
                        quote! { let #fname_ident = #field_conv; }
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
                        let #fname_ident = #field_conv;
                    });
                }
                ProjectionKind::Unsigned64 => {
                    if let Some(inner_ty) = option_inner_type(&field.ty) {
                        let niche = matches!(
                            proj.strategy,
                            FoldStrategy::Optional(NullableKind::Niche, _)
                        );
                        let inner_conv = composed_entry_decode(
                            registry.input_entry(&inner_ty)?,
                            &raw_ident,
                            &fname_ident,
                        );
                        let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                        let decode = if niche {
                            // The Kotlin data-class property is still `ULong?`
                            // (and therefore boxed in object storage), but its
                            // JNI converter is niche-keyed on primitive jlong.
                            // Run the complete field converter so every custom
                            // semantic stage (e.g. u64 -> Duration) is applied.
                            quote! { #field_conv }
                        } else {
                            quote! {
                                ::core::option::Option::Some(#inner_conv)
                            }
                        };
                        field_preludes.push(quote! {
                            let #tmp_ident: jni::objects::JObject = env
                                .get_field(v, #camel, "Lkotlin/ULong;")
                                .and_then(|val| val.l())
                                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                            let #fname_ident = if #tmp_ident.is_null() {
                                ::core::option::Option::None
                            } else {
                                let #raw_ident: jni::sys::jlong = env
                                    .call_method(&#tmp_ident, "unbox-impl", "()J", &[])
                                    .and_then(|val| val.j())
                                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                                #decode
                            };
                        });
                    } else {
                        field_preludes.push(quote! {
                            let #raw_ident: jni::sys::jlong = env
                                .get_field(v, #camel, "J")
                                .and_then(|val| val.j())
                                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                            let #fname_ident = #field_conv;
                        });
                    }
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
                .and_then(|n| ext.kotlin_fqn(&TypeKey::from_ident(&n)))
                .map(|v| v.to_string())
            {
                let sig = format!("L{};", fqn.replace('.', "/"));
                let inner_conv = composed_entry_decode(
                    registry.input_entry(&f_inner)?,
                    &raw_ident,
                    &fname_ident,
                );
                let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                let decode = if option_inner_type(&field.ty).is_some() {
                    quote! {
                        let #fname_ident = if #tmp_ident.is_null() {
                            ::core::option::Option::None
                        } else {
                            let #raw_ident: jni::sys::jint = env.call_method(&#tmp_ident, "getValue", "()I", &[])
                                .and_then(|val| val.i())
                                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                            ::core::option::Option::Some(#inner_conv)
                        };
                    }
                } else {
                    quote! {
                        let #raw_ident: jni::sys::jint = env.call_method(&#tmp_ident, "getValue", "()I", &[])
                            .and_then(|val| val.i())
                            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                        let #fname_ident = #inner_conv;
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
                    let #fname_ident = #field_conv;
                });
            }
            Some((sig, _, true)) => {
                let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                field_preludes.push(quote! {
                    let #tmp_ident: jni::objects::JObject = env.get_field(v, #camel, #sig)
                        .and_then(|val| val.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    let #raw_ident: #field_wire = #tmp_ident.into();
                    let #fname_ident = #field_conv;
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
                            ext.kotlin_fqn(&TypeKey::from_ident(&name))
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
                    let #fname_ident = #field_conv;
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

/// Whole-object **input** decode for a `sealed_class` sum: a `JObject` of the
/// sealed interface → the Rust enum, by `instanceof` dispatch over the nested
/// variant classes and a property read per payload.
///
/// This is the counterpart of a nested data class's own input converter, not
/// a second mechanism: a sum reached as a **field** sits inside a parent
/// `JObject` that is already being read field-by-field, so the parent's
/// generic field branch just delegates here — the same delegation
/// `Annotated.payload` uses. The tag-gated *flattened* form is the separate
/// parameter path.
///
/// The output direction deliberately has no such converter: a sum crosses
/// Rust → Kotlin **flattened, always** (`PlanFieldKind::Sum`), which is the
/// design's rejected-alternative note about per-crossing JVM objects. Reading
/// one field out of a `JObject` the caller already handed us costs nothing
/// extra, so the asymmetry is real rather than an oversight.
pub(crate) fn sum_input_body(
    ext: &JniGen,
    e: &syn::ItemEnum,
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr)> {
    use crate::api::core::types_util::SumSpec;

    let key = TypeKey::from_ident(&e.ident);
    let cfg = ext.types.get(&key)?;
    let sum_cfg = cfg.sum()?;
    let iface_fqn = cfg.name_spec.as_ref().map(|s| ext.fqn_of(s))?;
    let iface_path = iface_fqn.replace('.', "/");
    let source_module = ext.fn_module(registry, &e.ident);
    let enum_ident = &e.ident;
    let enum_name = e.ident.to_string();

    let spec = SumSpec::from_item_enum(e);
    let mut arms: Vec<TokenStream> = Vec::new();
    for (v, item_variant) in spec.variants.iter().zip(&e.variants) {
        let vident = &v.ident;
        let kotlin_name = ext.sum_variant_class_name(sum_cfg, vident);
        // A variant class is NESTED in the interface, so its JVM binary name
        // is `Outer$Variant`.
        let jvm_class = format!("{iface_path}${kotlin_name}");

        let mut preludes: Vec<TokenStream> = Vec::new();
        let mut inits: Vec<TokenStream> = Vec::new();
        for (f, item_field) in v.fields.iter().zip(item_variant.fields.iter()) {
            let prop = crate::api::lang::jnigen::jni::struct_plan::sum_field_prop_name(f);
            let bind = format_ident!("__p_{}", prop);
            let err_prefix = format!("{enum_name}.{kotlin_name}.{prop}: {{}}");
            let (pre, value) = read_kotlin_property(
                ext,
                registry,
                &quote!(__obj),
                &prop,
                &item_field.ty,
                &bind,
                &err_prefix,
            )?;
            preludes.push(pre);
            match &f.member {
                syn::Member::Named(n) => inits.push(quote!(#n: #value)),
                syn::Member::Unnamed(_) => inits.push(quote!(#value)),
            }
        }
        let ctor = match item_variant.fields {
            syn::Fields::Unit => quote!(#source_module::#enum_ident::#vident),
            syn::Fields::Named(_) => {
                quote!(#source_module::#enum_ident::#vident { #(#inits),* })
            }
            syn::Fields::Unnamed(_) => {
                quote!(#source_module::#enum_ident::#vident(#(#inits),*))
            }
        };
        arms.push(quote! {
            if env.is_instance_of(__obj, #jvm_class)
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(
                    format!(concat!(#enum_name, ": instanceof ", #jvm_class, ": {}"), e)))?
            {
                #(#preludes)*
                return ::core::result::Result::Ok(#ctor);
            }
        });
    }

    // No arm matched: the JVM object is not one of the declared variants —
    // a binding error through the ordinary channel, never a panic.
    let no_match = format!("{enum_name}: value is not one of its declared variants");
    // NULL must be rejected BEFORE the dispatch: JNI specifies that
    // `IsInstanceOf(NULL, any)` is true ("a NULL object can be cast to any
    // class"), so a null would match the first arm and silently decode as
    // that variant — for a unit first variant, without even a failed field
    // read to give it away. An `Option<sum>` never reaches here with null:
    // its wrapper carves the null niche first and yields `None`.
    let null_msg = format!("{enum_name}: null value where a variant was required");
    let body: syn::Expr = syn::parse_quote!({
        let __obj = v;
        (|| -> ::core::result::Result<#source_module::#enum_ident, __JniErr> {
            if __obj.is_null() {
                return ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<String>>::from(#null_msg.to_string()),
                );
            }
            #(#arms)*
            ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<String>>::from(#no_match.to_string()),
            )
        })()?
    });
    Some((syn::parse_quote!(jni::objects::JObject), body))
}

/// Read one Kotlin property off `receiver` and decode it to its Rust value,
/// binding the result to `bind`. Mirrors the per-field decode
/// [`struct_input_body`] performs, for the positions that are properties of a
/// generated class rather than fields of a data class.
fn read_kotlin_property(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    receiver: &TokenStream,
    prop: &str,
    ty: &syn::Type,
    bind: &syn::Ident,
    err_prefix: &str,
) -> Option<(TokenStream, TokenStream)> {
    let entry = registry.input_entry(ty)?;
    let wire = entry.destination.clone();
    let raw = format_ident!("{}_raw", bind);
    // The COMPLETE wire → Rust chain, not just the wire-facing converter: a
    // `convert!`-declared type reaches its Rust value through the rust-side
    // stages that follow (`jlong → u64 → Duration`). Stage bindings are named
    // off `bind`, so two payloads of the same type in one variant do not
    // collide.
    let conv = composed_property_decode(entry, bind);

    // A handle property is a `NativeHandle` object whose raw pointer comes
    // from `peek()`; an enum property is the Kotlin enum class, decoded
    // through its `getValue`. Both mirror `struct_input_body`.
    if let Some(proj) = &entry.metadata.projection {
        if matches!(proj.kind, ProjectionKind::Handle) {
            let fqn = handle_field_fqn(ext, proj).replace('.', "/");
            let sig = format!("L{fqn};");
            let obj = format_ident!("{}_obj", bind);
            // A required (non-`Option`) handle payload is OWNED by the variant
            // it builds, so it is decoded by CONSUMING the native object
            // (`Box::from_raw`) — the mirror of the output side's
            // `Box::into_raw`. The borrow converter would yield
            // `OwnedObject<T>`, which cannot populate an owned field. Same rule
            // (and same reasoning) as an owned handle field of a data class;
            // `Option<_>` keeps the niche-aware converter (jlong 0 ⇒ `None`).
            let closed_msg = "Operation on a closed native handle.";
            let decode = if option_inner_type(ty).is_some() {
                quote! { let #bind = #conv; }
            } else {
                quote! {
                    if #raw == 0 || (#raw & 1) == 1 {
                        return ::core::result::Result::Err(
                            <__JniErr as ::core::convert::From<String>>::from(
                                #closed_msg.to_string(),
                            ),
                        );
                    }
                    let #bind: #ty = unsafe {
                        *std::boxed::Box::from_raw(#raw as *mut #ty)
                    };
                }
            };
            return Some((
                quote! {
                    let #obj: jni::objects::JObject = env.get_field(#receiver, #prop, #sig)
                        .and_then(|val| val.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    let #raw: jni::sys::jlong = if #obj.is_null() {
                        0
                    } else {
                        env.call_method(&#obj, "peek", "()J", &[])
                            .and_then(|val| val.j())
                            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?
                    };
                    #decode
                },
                quote!(#bind),
            ));
        }
    }
    // An enum property — bare or under `Option` — is stored as the Kotlin
    // **enum object**, so it is read through `getValue`, NOT through the
    // generic converters: the bare-enum one is `jint`-keyed and the
    // `Option<enum>` one unboxes a `java.lang.Integer`, and neither matches
    // what the JVM slot actually holds. `struct_input_body` makes the same
    // distinction for data-class fields; this is that logic for a property.
    let enum_inner = option_inner_type(ty).unwrap_or_else(|| ty.clone());
    if ext.is_kotlin_enum(&enum_inner) {
        let fqn = bare_path_ident(&enum_inner)
            .and_then(|n| ext.kotlin_fqn(&TypeKey::from_ident(&n)))
            .map(|v| v.to_string())?;
        let sig = format!("L{};", fqn.replace('.', "/"));
        let obj = format_ident!("{}_obj", bind);
        // Under `Option`, JVM null is `None` and the INNER converter decodes
        // the discriminant; the outer converter would expect a boxed Integer.
        let decode = if option_inner_type(ty).is_some() {
            let inner_conv = registry
                .input_entry(&enum_inner)?
                .function
                .sig
                .ident
                .clone();
            quote! {
                let #bind = if #obj.is_null() {
                    ::core::option::Option::None
                } else {
                    let #raw: jni::sys::jint = env.call_method(&#obj, "getValue", "()I", &[])
                        .and_then(|val| val.i())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    ::core::option::Option::Some(#inner_conv(env, &#raw)?)
                };
            }
        } else {
            quote! {
                let #raw: jni::sys::jint = env.call_method(&#obj, "getValue", "()I", &[])
                    .and_then(|val| val.i())
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                let #bind = #conv;
            }
        };
        return Some((
            quote! {
                let #obj: jni::objects::JObject = env.get_field(#receiver, #prop, #sig)
                    .and_then(|val| val.l())
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                #decode
            },
            quote!(#bind),
        ));
    }
    match jni_field_access(&wire) {
        Some((sig, accessor, false)) => Some((
            quote! {
                let #raw: #wire = env.get_field(#receiver, #prop, #sig)
                    .and_then(|val| val.#accessor())
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))? as _;
                let #bind = #conv;
            },
            quote!(#bind),
        )),
        Some((sig, _, true)) => {
            let obj = format_ident!("{}_obj", bind);
            Some((
                quote! {
                    let #obj: jni::objects::JObject = env.get_field(#receiver, #prop, #sig)
                        .and_then(|val| val.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    let #raw: #wire = #obj.into();
                    let #bind = #conv;
                },
                quote!(#bind),
            ))
        }
        None => {
            // Object-shaped wire with no fixed descriptor (a nested data
            // class, another sum, a `List`): the slot's descriptor is the
            // registered Kotlin class and the value decodes through its own
            // converter — the same delegation the data-class path uses.
            let slot_ty = option_inner_type(ty).unwrap_or_else(|| ty.clone());
            let sig = bare_path_ident(&slot_ty)
                .and_then(|name| ext.kotlin_fqn(&TypeKey::from_ident(&name)))
                .map(|v| format!("L{};", v.replace('.', "/")))
                .or_else(|| pat_match_top(&slot_ty, "Vec").then(|| "Ljava/util/List;".to_string()))
                .unwrap_or_else(|| "Ljava/lang/Object;".to_string());
            Some((
                quote! {
                    let #raw: jni::objects::JObject = env.get_field(#receiver, #prop, #sig)
                        .and_then(|val| val.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    let #bind = #conv;
                },
                quote!(#bind),
            ))
        }
    }
}

/// The complete `wire -> Rust` decode of one value read out of a JVM object:
/// the wire-facing converter applied to `raw`, followed by the rust-side
/// stages a custom [`convert!`](crate::convert) declaration inserts.
///
/// The mirror of [`ConvChain::call`](super::super::struct_plan::ConvChain) on
/// the output side, and of the structural wrappers' own chain composition:
/// stopping at [`TypeEntry::converter_ident`] would bind the *representation*
/// (`u64`) where the Rust value (`Duration`) is required, which does not
/// compile.
///
/// Every converter invocation in this module's whole-object decoders goes
/// through here — a data-class field, a sealed-class property, and the inner
/// converter an `Option`/enum slot delegates to — so a chain cannot be dropped
/// by one branch happening not to have been the one under test. Stage bindings
/// derive from `stage_base`, so two values of the same type in one scope get
/// distinct names.
fn composed_entry_decode(
    entry: &crate::core::TypeEntry<KotlinMeta>,
    raw: &syn::Ident,
    stage_base: &syn::Ident,
) -> TokenStream {
    let converter = entry.converter_ident();
    if entry.pre_stages.is_empty() {
        return quote!(#converter(env, &#raw)?);
    }
    let s0 = format_ident!("{}_s0", stage_base);
    let mut body = quote! { let #s0 = #converter(env, &#raw)?; };
    let mut previous = s0;
    for (order, (_, stage)) in entry.input_stage_order().enumerate() {
        let stage_fn = &stage.function.sig.ident;
        let next = format_ident!("{}_s{}", stage_base, order + 1);
        body.extend(quote! {
            let #next = #stage_fn(env, #previous)
                .map_err(|__e| <__JniErr as ::core::convert::From<String>>::from(
                    __e.to_string()))?;
        });
        previous = next;
    }
    quote!({ #body #previous })
}

/// [`composed_entry_decode`] for a sealed-class property, whose raw binding is
/// `<bind>_raw` by construction.
fn composed_property_decode(
    entry: &crate::core::TypeEntry<KotlinMeta>,
    bind: &syn::Ident,
) -> TokenStream {
    composed_entry_decode(entry, &format_ident!("{}_raw", bind), bind)
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
    /// Call-site destructure expression **tail** — everything after the
    /// object expression (`.field ?: 0`, `?.seq != null`, ` != null`). The
    /// full access is composed per site via [`Self::kt_access`], so the plan
    /// itself stays independent of the call form (`payload`, `this`, `__e`).
    pub kt_access_tail: String,
    /// Text placed **before** the object expression, so the access is a
    /// template (`prefix + base + tail`) rather than a suffix.
    ///
    /// Empty for every ordinary leaf, whose access really is a suffix. A
    /// tag-gated variant slot is not: it needs
    /// `(<base>.field as? Reading.Exact)?.v0 ?: 0L`, where the base sits in
    /// the middle. Both [`Self::kt_access`] and the handle-target composition
    /// go through it, because a variant slot can equally be a handle — and a
    /// handle target that ignored the prefix would silently lock and consume
    /// the wrong expression.
    pub kt_access_prefix: String,
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
    /// Complete converter entry for an ordinary value leaf. Present flags and
    /// direct owned-handle leaves have no entry here.
    pub entry: Option<crate::api::core::registry::TypeEntry<KotlinMeta>>,
    /// A nested owned handle crosses as a raw pointer under the same Kotlin
    /// locking/consume scaffold as a top-level handle. This stores the typed
    /// property access tail (`.child.handle` / `?.handle`) used to collect it.
    pub handle_target_tail: Option<String>,
    /// Whether the handle access can be null, either because the field itself
    /// is optional or because an optional ancestor gates it.
    pub handle_nullable: bool,
}

impl FlatLeaf {
    /// Kotlin call-site destructure expression feeding this leaf, rooted at
    /// `base` — the object expression at this call site (the camelCase param
    /// name, `this` for a promoted receiver, `__e` for the vec-build loop
    /// variable).
    pub fn kt_access(&self, base: &str) -> String {
        format!("{}{base}{}", self.kt_access_prefix, self.kt_access_tail)
    }

    /// The Kotlin expression yielding the handle this leaf carries, rooted at
    /// `base` — the thing the lock scaffold locks and `markConsumed()`s.
    /// `None` when the leaf is not a handle. Composed through the same
    /// template as [`Self::kt_access`].
    pub fn kt_handle_target(&self, base: &str) -> Option<String> {
        let tail = self.handle_target_tail.as_ref()?;
        Some(format!("{}{base}{tail}", self.kt_access_prefix))
    }

    /// Native call argument for this leaf. Handle pointers are bound under
    /// the unified lock scaffold; every other leaf is read directly from the
    /// Kotlin object graph.
    pub fn kt_call_arg(&self, base: &str) -> String {
        if self.handle_target_tail.is_some() {
            format!("{}_ptr", self.kt_name)
        } else {
            self.kt_access(base)
        }
    }
}

pub(crate) struct FlatStructNode {
    pub struct_module: syn::Path,
    pub struct_ident: syn::Ident,
    pub binding: syn::Ident,
    pub optional: bool,
    pub present_ident: Option<syn::Ident>,
    pub fields: Vec<FlatFieldNode>,
}

pub(crate) enum FlatFieldNode {
    Value {
        field: syn::Ident,
        value_leaf: usize,
        present_leaf: Option<usize>,
        direct_handle: bool,
        optional_handle: bool,
        rust_ty: Box<syn::Type>,
    },
    Nested {
        field: syn::Ident,
        node: Box<FlatStructNode>,
    },
    /// A data-carrying enum crossing as an `Int` **tag** leaf plus one leaf
    /// group per variant, inert groups filled with their wire defaults — the
    /// N-way form of the `present` gating `Option` already uses.
    Sum {
        field: syn::Ident,
        /// Index of the synthetic tag leaf.
        tag_leaf: usize,
        /// Index of the `present` gate when the field is `Option<sum>`.
        /// Optionality and choice stay independent facts: the tag domain is
        /// never overloaded with an "absent" value.
        present_leaf: Option<usize>,
        /// Qualified path to the source enum, for the reconstruct's arms.
        source: syn::Path,
        /// Variants in declaration order; index == tag.
        variants: Vec<FlatSumVariant>,
        rust_ty: Box<syn::Type>,
    },
}

/// One alternative of a [`FlatFieldNode::Sum`].
pub(crate) struct FlatSumVariant {
    pub rust_ident: syn::Ident,
    /// This variant's payload: how each field is addressed when rebuilding
    /// it, paired with the leaf carrying its value. Empty for a unit variant.
    pub fields: Vec<(syn::Member, usize)>,
}

/// A flattened plan for one struct input parameter. Built once by
/// [`build_flat_input_plan`] and consumed by all three codegen sites.
pub(crate) struct FlatInputPlan {
    pub leaves: Vec<FlatLeaf>,
    pub root: FlatStructNode,
    /// `true` when the source fn takes `&Struct` — the call site passes `&arg`.
    pub by_ref: bool,
    /// Vec/slice element lowering deliberately retains its previous
    /// non-recursive ABI; callers use this bit to decline recursive plans.
    pub contains_nested: bool,
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

#[derive(Clone, Debug)]
pub(crate) struct FlatInputError {
    pub root: TypeKey,
    pub path: String,
    pub reason: String,
}

impl FlatInputError {
    pub fn message(&self) -> String {
        format!(
            "data-class input `{}` cannot be flattened at `{}`: {} — fixed-layout data classes must flatten completely; declare `data_class!({}).jobject_input()` to opt this type into an explicit JObject boundary",
            self.root, self.path, self.reason, self.root
        )
    }
}

fn flat_error(root: &TypeKey, path: &str, reason: impl Into<String>) -> FlatInputError {
    FlatInputError {
        root: root.clone(),
        path: path.to_string(),
        reason: reason.into(),
    }
}

fn wire_kotlin_type(entry: &crate::api::core::registry::TypeEntry<KotlinMeta>) -> String {
    if let Some(p) = JniPrim::from_wire(&entry.destination) {
        return p.kotlin_type().to_string();
    }
    if let syn::Type::Path(tp) = &entry.destination {
        if let Some(last) = tp.path.segments.last() {
            return match last.ident.to_string().as_str() {
                "JString" => "String".to_string(),
                "JByteArray" => "ByteArray".to_string(),
                _ => entry
                    .metadata
                    .kotlin_name
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "Any".to_string()),
            };
        }
    }
    if matches!(entry.destination, syn::Type::Ptr(_)) {
        "Long".to_string()
    } else {
        entry
            .metadata
            .kotlin_name
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "Any".to_string())
    }
}

/// Plan a **data-carrying enum** field as a tag leaf plus one leaf group per
/// variant, so the sum crosses flattened instead of as one `JObject` per call.
///
/// Returns `None` when some payload cannot be expressed as a flat leaf — a
/// handle, say, whose ownership the group-gated form does not yet model. That
/// is deliberately a **graceful degradation, not a rejection**: the caller
/// falls through to the ordinary value leaf, and the sum crosses as a whole
/// object through its own converter, exactly as it did before this path
/// existed. Refusing instead would turn a working binding into a build error
/// for a shape the generator can already handle.
///
/// Kotlin references are emitted **fully qualified** (`is io.x.Reading.Exact`)
/// so the call site needs no import bookkeeping — these expressions are raw
/// text spliced into a wrapper whose import set this plan does not own.
#[allow(clippy::too_many_arguments)]
fn build_flat_sum_field(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    sum_ty: &syn::Type,
    field: syn::Ident,
    optional: bool,
    native_prefix: &str,
    field_ref: &str,
    nullable_access: bool,
    rust_ty: &syn::Type,
    leaves: &mut Vec<FlatLeaf>,
) -> Option<FlatFieldNode> {
    use crate::api::core::types_util::SumSpec;

    let ident = bare_path_ident(sum_ty)?;
    let (item_enum, _) = registry.enums.get(&ident)?;
    let cfg = ext.types.get(&TypeKey::from_ident(&ident))?;
    let sum_cfg = cfg.sum()?;
    let iface_fqn = cfg.name_spec.as_ref().map(|s| ext.fqn_of(s))?;
    let spec = SumSpec::from_item_enum(item_enum);

    // Plan every group first: a single unflattenable payload means the whole
    // sum stays object-shaped, so nothing may be pushed until all of them are
    // known good.
    struct Planned {
        rust_ident: syn::Ident,
        kotlin: String,
        fields: Vec<(syn::Member, PlannedLeaf)>,
    }
    struct PlannedLeaf {
        native: String,
        entry: crate::api::core::registry::TypeEntry<KotlinMeta>,
        access_tail: String,
        nullable_wire: bool,
    }
    let mut planned: Vec<Planned> = Vec::new();
    for (v, item_variant) in spec.variants.iter().zip(&item_enum.variants) {
        let kotlin = ext.sum_variant_class_name(sum_cfg, &v.ident);
        let mut fields = Vec::new();
        for (f, item_field) in v.fields.iter().zip(item_variant.fields.iter()) {
            let entry = registry.input_entry(&item_field.ty)?;
            // A projection payload (handle / value blob) carries ownership
            // and locking rules the tag-gated group does not model yet.
            if entry.metadata.projection.is_some() {
                return None;
            }
            // Nested objects would need their own recursive group; today only
            // leaf-shaped payloads flatten.
            let prim = JniPrim::from_wire(&entry.destination);
            let is_string_like = matches!(&entry.destination, syn::Type::Path(tp)
                if tp.path.segments.last().is_some_and(|s| s.ident == "JString"));
            if prim.is_none() && !is_string_like {
                return None;
            }
            let prop = crate::api::lang::jnigen::jni::struct_plan::sum_field_prop_name(f);
            let slot =
                crate::api::lang::jnigen::jni::struct_plan::sum_slot_fragment(&kotlin, &prop);
            // `(<base>.field as? io.x.E.V)?.prop` — inert groups yield null,
            // so a primitive slot takes its zero and an object slot stays
            // nullable. The Rust side only converts the live group.
            //
            // An `enum_class` payload is a Kotlin enum object whose wire is
            // the `jint` discriminant, so the access reads `.value` — without
            // it the slot would be `Priority?` where the wire wants `Int`.
            let read = if ext.is_kotlin_enum(&item_field.ty) {
                format!("{prop}?.value")
            } else {
                prop.clone()
            };
            let cast = format!("{field_ref} as? {iface_fqn}.{kotlin})?.{read}");
            let (access_tail, nullable_wire) = match &prim {
                Some(p) => (format!("{cast} ?: {}", p.kotlin_zero()), false),
                None => (cast, true),
            };
            fields.push((
                f.member.clone(),
                PlannedLeaf {
                    native: format!("{native_prefix}_{slot}"),
                    entry: entry.clone(),
                    access_tail,
                    nullable_wire,
                },
            ));
        }
        planned.push(Planned {
            rust_ident: v.ident.clone(),
            kotlin,
            fields,
        });
    }

    // Everything flattens — now push the leaves, tag first.
    let present_leaf = optional.then(|| {
        push_present_leaf(
            leaves,
            &format!("{native_prefix}_present"),
            format!("{field_ref} != null"),
            Some(field.clone()),
        )
    });

    // The tag is computed once per call site by matching the value against
    // its own variant classes. A nullable access adds the `null` arm the
    // present flag already gates on.
    let mut arms: Vec<String> = Vec::new();
    if nullable_access || optional {
        arms.push("null -> 0".to_string());
    }
    for (tag, p) in planned.iter().enumerate() {
        arms.push(format!("is {iface_fqn}.{} -> {tag}", p.kotlin));
    }
    let tag_leaf = leaves.len();
    leaves.push(FlatLeaf {
        native_ident: format_ident!("{}__tag", native_prefix),
        native_wire_ty: quote!(jni::sys::jint),
        kt_name: snake_to_camel(&format!("{native_prefix}__tag")),
        kt_wire_ty: "Int".to_string(),
        kt_access_tail: format!("{field_ref}) {{ {} }}", arms.join("; ")),
        kt_access_prefix: "when (".to_string(),
        conv: None,
        field: Some(field.clone()),
        is_present_flag: false,
        entry: None,
        handle_target_tail: None,
        handle_nullable: false,
    });

    let variants = planned
        .into_iter()
        .map(|p| FlatSumVariant {
            rust_ident: p.rust_ident,
            fields: p
                .fields
                .into_iter()
                .map(|(member, l)| {
                    let idx = push_value_leaf(
                        leaves,
                        &l.native,
                        field.clone(),
                        &l.entry,
                        l.access_tail,
                        l.nullable_wire,
                    );
                    // The access is a cast expression, so the base sits in the
                    // middle rather than at the front.
                    leaves[idx].kt_access_prefix = "(".to_string();
                    (member, idx)
                })
                .collect(),
        })
        .collect();

    let module = ext.fn_module(registry, &ident);
    Some(FlatFieldNode::Sum {
        field,
        tag_leaf,
        present_leaf,
        source: syn::parse_quote!(#module::#ident),
        variants,
        rust_ty: Box::new(rust_ty.clone()),
    })
}

fn push_present_leaf(
    leaves: &mut Vec<FlatLeaf>,
    native: &str,
    access: String,
    field: Option<syn::Ident>,
) -> usize {
    let index = leaves.len();
    leaves.push(FlatLeaf {
        native_ident: format_ident!("{native}"),
        native_wire_ty: quote!(jni::sys::jboolean),
        kt_name: snake_to_camel(native),
        kt_wire_ty: "Boolean".to_string(),
        kt_access_tail: access,
        kt_access_prefix: String::new(),
        conv: None,
        field,
        is_present_flag: true,
        entry: None,
        handle_target_tail: None,
        handle_nullable: false,
    });
    index
}

fn push_value_leaf(
    leaves: &mut Vec<FlatLeaf>,
    native: &str,
    field: syn::Ident,
    entry: &crate::api::core::registry::TypeEntry<KotlinMeta>,
    access: String,
    nullable_wire: bool,
) -> usize {
    let wire = &entry.destination;
    let mut kt_wire_ty = wire_kotlin_type(entry);
    if nullable_wire && !kt_wire_ty.ends_with('?') {
        kt_wire_ty.push('?');
    }
    let index = leaves.len();
    leaves.push(FlatLeaf {
        native_ident: format_ident!("{native}"),
        native_wire_ty: annotate_jobject_with_lifetime(wire, "a").to_token_stream(),
        kt_name: snake_to_camel(native),
        kt_wire_ty,
        kt_access_tail: access,
        kt_access_prefix: String::new(),
        conv: Some(entry.function.sig.ident.clone()),
        field: Some(field),
        is_present_flag: false,
        entry: Some(entry.clone()),
        handle_target_tail: None,
        handle_nullable: false,
    });
    index
}

fn push_handle_leaf(
    leaves: &mut Vec<FlatLeaf>,
    native: &str,
    field: syn::Ident,
    target: String,
    nullable: bool,
) -> usize {
    let index = leaves.len();
    leaves.push(FlatLeaf {
        native_ident: format_ident!("{native}"),
        native_wire_ty: quote!(jni::sys::jlong),
        kt_name: snake_to_camel(native),
        kt_wire_ty: "Long".to_string(),
        kt_access_tail: target.clone(),
        kt_access_prefix: String::new(),
        conv: None,
        field: Some(field),
        is_present_flag: false,
        entry: None,
        handle_target_tail: Some(target),
        handle_nullable: nullable,
    });
    index
}

/// Build the one shared recursive Kotlin→Rust plan. `Ok(None)` means the
/// parameter is not an unmarked declared data class (including the explicit
/// `.jobject_input()` opt-in); an unmarked data class either returns a complete
/// plan or a validation error — never a silent object fallback.
pub(crate) fn build_flat_input_plan(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    param_name: &syn::Ident,
    arg_ty: &syn::Type,
) -> Result<Option<FlatInputPlan>, FlatInputError> {
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
    let Some(name) = bare_path_ident(&struct_ty) else {
        return Ok(None);
    };
    let Some((st, _)) = registry.structs.get(&name) else {
        return Ok(None);
    };
    let key = TypeKey::from_type(&struct_ty);
    let Some(cfg) = ext.types.get(&key) else {
        return Ok(None);
    };
    if cfg.special_decl() || cfg.name_spec.is_none() || cfg.jobject_input {
        return Ok(None);
    }
    // Identity / pass-through guard: the resolved param must decode to the
    // struct itself, not an opaque handle / value projection (`projection`
    // present) and not a multi-source / non-identity `impl Into<S>` (which
    // surfaces as `"Any"` Dispatch or a foreign source type). The resolved
    // param's Kotlin type (compared by short name, since metadata carries the
    // FQN) must equal the struct's data-class name.
    let Some(entry) = registry.input_entry(arg_ty) else {
        return Ok(None);
    };
    if entry.metadata.projection.is_some() {
        return Ok(None);
    }
    let dc_short = cfg
        .name_spec
        .as_ref()
        .map(|s| ext.fqn_of(s))
        .map(|fqn| fqn.rsplit('.').next().unwrap_or(&fqn).to_string())
        .unwrap_or_else(|| name.to_string());
    let entry_short = entry
        .metadata
        .kotlin_name
        .as_ref()
        .and_then(|t| t.simple_name());
    if entry_short != Some(dc_short.as_str()) {
        return Ok(None);
    }
    let mut leaves: Vec<FlatLeaf> = Vec::new();
    let mut stack = Vec::new();
    let root = build_flat_struct_node(
        ext,
        registry,
        st,
        optional,
        &param_name.to_string(),
        "",
        optional,
        &key,
        &mut stack,
        &mut leaves,
    )?;
    let contains_nested = root
        .fields
        .iter()
        .any(|f| matches!(f, FlatFieldNode::Nested { .. }));
    Ok(Some(FlatInputPlan {
        leaves,
        root,
        by_ref,
        contains_nested,
    }))
}

#[allow(clippy::too_many_arguments)]
fn build_flat_struct_node(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    st: &syn::ItemStruct,
    optional: bool,
    native_prefix: &str,
    access_prefix: &str,
    nullable_context: bool,
    root: &TypeKey,
    stack: &mut Vec<TypeKey>,
    leaves: &mut Vec<FlatLeaf>,
) -> Result<FlatStructNode, FlatInputError> {
    let node_key = TypeKey::from_ident(&st.ident);
    if stack.contains(&node_key) {
        return Err(flat_error(
            root,
            native_prefix,
            "recursive data-class cycle",
        ));
    }
    if stack.len() >= 16 {
        return Err(flat_error(
            root,
            native_prefix,
            "recursive flattening exceeds depth 16",
        ));
    }
    let syn::Fields::Named(named) = &st.fields else {
        return Err(flat_error(
            root,
            native_prefix,
            "only named-field structs can flatten",
        ));
    };
    stack.push(node_key);
    let present_ident = if optional {
        let native = format!("{native_prefix}_present");
        push_present_leaf(leaves, &native, format!("{access_prefix} != null"), None);
        Some(format_ident!("{native}"))
    } else {
        None
    };
    let mut fields = Vec::new();
    for field in &named.named {
        let Some(fident) = field.ident.clone() else {
            return Err(flat_error(root, native_prefix, "unnamed field"));
        };
        let fcamel = mangle_kotlin_ident(&snake_to_camel(&fident.to_string()));
        let child_native = format!("{native_prefix}_{}", fident);
        let field_ref = if nullable_context {
            format!("{access_prefix}?.{fcamel}")
        } else {
            format!("{access_prefix}.{fcamel}")
        };
        let nested_ty = option_inner_type(&field.ty).unwrap_or_else(|| field.ty.clone());
        // A data-carrying enum flattens into a tag plus one group per variant.
        // `None` means some payload is not leaf-shaped — fall through and let
        // it cross as one object through its own converter.
        if matches!(ext.type_kind(registry, &nested_ty), TypeKind::Sum) {
            let field_optional = option_inner_type(&field.ty).is_some();
            if let Some(node) = build_flat_sum_field(
                ext,
                registry,
                &nested_ty,
                fident.clone(),
                field_optional,
                &child_native,
                &field_ref,
                nullable_context,
                &field.ty,
                leaves,
            ) {
                fields.push(node);
                continue;
            }
        }
        if let TypeKind::DataStruct {
            st: child,
            cfg: Some(cfg),
        } = ext.type_kind(registry, &nested_ty)
        {
            if cfg.name_spec.is_some() && !cfg.special_decl() && !cfg.jobject_input {
                let child_optional = option_inner_type(&field.ty).is_some();
                let node = build_flat_struct_node(
                    ext,
                    registry,
                    child,
                    child_optional,
                    &child_native,
                    &field_ref,
                    nullable_context || child_optional,
                    root,
                    stack,
                    leaves,
                )?;
                fields.push(FlatFieldNode::Nested {
                    field: fident,
                    node: Box::new(node),
                });
                continue;
            }
        }

        let path = child_native.clone();
        let Some(fentry) = registry.input_entry(&field.ty) else {
            return Err(flat_error(
                root,
                &path,
                format!(
                    "field type `{}` has no input converter",
                    TypeKey::from_type(&field.ty)
                ),
            ));
        };

        // Nullable primitive/enum with no niche: keep the allocation-free
        // `(present, value)` representation at every recursion depth.
        if let Some(inner_ty) = option_inner_type(&field.ty) {
            if !matches!(inner_ty, syn::Type::Reference(_)) {
                if let Some(inner) = registry.input_entry(&inner_ty) {
                    if let Some(prim) = JniPrim::from_wire(&inner.destination) {
                        if inner.niches.clone().carve().is_none()
                            && inner.metadata.projection.is_none()
                            && inner.pre_stages.is_empty()
                        {
                            let present_index = push_present_leaf(
                                leaves,
                                &format!("{child_native}_present"),
                                format!("{field_ref} != null"),
                                Some(fident.clone()),
                            );
                            let value_access = if ext.is_kotlin_enum(&inner_ty) {
                                format!("{field_ref}?.value ?: {}", prim.kotlin_zero())
                            } else {
                                format!("{field_ref} ?: {}", prim.kotlin_zero())
                            };
                            let value_index = push_value_leaf(
                                leaves,
                                &format!("{child_native}_value"),
                                fident.clone(),
                                inner,
                                value_access,
                                false,
                            );
                            fields.push(FlatFieldNode::Value {
                                field: fident,
                                value_leaf: value_index,
                                present_leaf: Some(present_index),
                                direct_handle: false,
                                optional_handle: false,
                                rust_ty: Box::new(field.ty.clone()),
                            });
                            continue;
                        }
                    }
                }
            }
        }

        if let Some(proj) = &fentry.metadata.projection {
            // `Option<u64>` has no natural niche and its ordinary converter is
            // object-shaped. Preserve the allocation-free field ABI by
            // splitting it into presence + raw `jlong`, just like optional
            // signed primitives. Bounded custom representations whose range
            // provides a niche already have a primitive destination and stay
            // a single leaf below.
            if proj.kind == ProjectionKind::Unsigned64 {
                if let Some(inner_ty) = option_inner_type(&field.ty) {
                    if JniPrim::from_wire(&fentry.destination).is_none() {
                        let inner = registry.input_entry(&inner_ty).ok_or_else(|| {
                            flat_error(
                                root,
                                &path,
                                format!(
                                    "unsigned field representation `{}` has no input converter",
                                    TypeKey::from_type(&inner_ty)
                                ),
                            )
                        })?;
                        let present_index = push_present_leaf(
                            leaves,
                            &format!("{child_native}_present"),
                            format!("{field_ref} != null"),
                            Some(fident.clone()),
                        );
                        let value_index = push_value_leaf(
                            leaves,
                            &format!("{child_native}_value"),
                            fident.clone(),
                            inner,
                            format!("{field_ref}?.toLong() ?: 0L"),
                            false,
                        );
                        fields.push(FlatFieldNode::Value {
                            field: fident,
                            value_leaf: value_index,
                            present_leaf: Some(present_index),
                            direct_handle: false,
                            optional_handle: false,
                            rust_ty: Box::new(field.ty.clone()),
                        });
                        continue;
                    }
                }
            }
            match proj.kind {
                ProjectionKind::Handle => {
                    if matches!(proj.strategy, FoldStrategy::Iterable(_)) {
                        return Err(flat_error(
                            root,
                            &path,
                            "collections of handles retain their collection boundary",
                        ));
                    }
                    let optional_handle = option_inner_type(&field.ty).is_some();
                    let value_index = push_handle_leaf(
                        leaves,
                        &child_native,
                        fident.clone(),
                        field_ref,
                        nullable_context || optional_handle,
                    );
                    fields.push(FlatFieldNode::Value {
                        field: fident,
                        value_leaf: value_index,
                        present_leaf: None,
                        direct_handle: true,
                        optional_handle,
                        rust_ty: Box::new(field.ty.clone()),
                    });
                    continue;
                }
                ProjectionKind::ValueBlob => {
                    let is_opt = option_inner_type(&field.ty).is_some();
                    let mut access = if is_opt || nullable_context {
                        format!("{field_ref}?.bytes")
                    } else {
                        format!("{field_ref}.bytes")
                    };
                    if nullable_context && !is_opt {
                        access.push_str(" ?: ByteArray(0)");
                    }
                    let value_index = push_value_leaf(
                        leaves,
                        &child_native,
                        fident.clone(),
                        fentry,
                        access,
                        is_opt,
                    );
                    fields.push(FlatFieldNode::Value {
                        field: fident,
                        value_leaf: value_index,
                        present_leaf: None,
                        direct_handle: false,
                        optional_handle: false,
                        rust_ty: Box::new(field.ty.clone()),
                    });
                    continue;
                }
                ProjectionKind::Unsigned64 => {
                    let is_opt = option_inner_type(&field.ty).is_some();
                    let access = if is_opt || nullable_context {
                        let sentinel = proj
                            .niche_sentinels
                            .first()
                            .cloned()
                            .unwrap_or_else(|| "0L".to_string());
                        format!("{field_ref}?.toLong() ?: {sentinel}")
                    } else {
                        format!("{field_ref}.toLong()")
                    };
                    let value_index = push_value_leaf(
                        leaves,
                        &child_native,
                        fident.clone(),
                        fentry,
                        access,
                        false,
                    );
                    fields.push(FlatFieldNode::Value {
                        field: fident,
                        value_leaf: value_index,
                        present_leaf: None,
                        direct_handle: false,
                        optional_handle: false,
                        rust_ty: Box::new(field.ty.clone()),
                    });
                    continue;
                }
            }
        }

        let field_is_option = option_inner_type(&field.ty).is_some();
        // The enum branch is self-contained: when it coalesces (`?.value ?: 0`)
        // it already yields a non-null `Int`, so block (B) below must not append
        // a second default (which produced the dead `?: 0 ?: 0`, issue #144).
        let mut enum_coalesced = false;
        let mut access = if ext.is_kotlin_enum(&flat_probe_inner(&field.ty)) {
            if field_is_option || nullable_context {
                enum_coalesced = true;
                format!("{field_ref}?.value ?: 0")
            } else {
                format!("{field_ref}.value")
            }
        } else {
            field_ref.clone()
        };
        if nullable_context && !field_is_option && !enum_coalesced {
            if let Some((sig, _, _)) = jni_field_access(&fentry.destination) {
                if let Some(default) = kt_leaf_default(sig, false) {
                    access = format!("{access} ?: {default}");
                }
            }
        }
        let value_index = push_value_leaf(
            leaves,
            &child_native,
            fident.clone(),
            fentry,
            access,
            (field_is_option || nullable_context) && is_jobject_shaped_wire(&fentry.destination),
        );
        fields.push(FlatFieldNode::Value {
            field: fident,
            value_leaf: value_index,
            present_leaf: None,
            direct_handle: false,
            optional_handle: false,
            rust_ty: Box::new(field.ty.clone()),
        });
    }
    stack.pop();
    Ok(FlatStructNode {
        struct_module: struct_module_path(ext, registry, st),
        struct_ident: st.ident.clone(),
        binding: format_ident!("__flat_{native_prefix}"),
        optional,
        present_ident,
        fields,
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
    let reconstruct = render_flat_struct_node(plan, &plan.root, on_err);
    let root_binding = &plan.root.binding;
    let prelude = quote! {
        #reconstruct
        let #arg_ident = #root_binding;
    };
    let call_arg = if plan.by_ref {
        quote!(&#arg_ident)
    } else {
        quote!(#arg_ident)
    };
    (prelude, call_arg)
}

fn render_entry_decode(
    entry: &crate::api::core::registry::TypeEntry<KotlinMeta>,
    wire_ident: &syn::Ident,
    out_ident: &syn::Ident,
    on_err: &TokenStream,
) -> TokenStream {
    let conv = entry.converter_ident();
    let decode_call = if matches!(entry.destination, syn::Type::Ptr(_)) {
        quote!(#conv(&mut env, #wire_ident))
    } else {
        quote!(#conv(&mut env, &#wire_ident))
    };
    let route = |expr: TokenStream| {
        quote! {
            match #expr {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
                    return #on_err;
                }
            }
        }
    };
    if entry.pre_stages.is_empty() {
        let decoded = route(decode_call);
        return quote!(let #out_ident = #decoded;);
    }
    let stage0 = format_ident!("{}_s0", out_ident);
    let decoded = route(decode_call);
    let mut body = quote!(let #stage0 = #decoded;);
    let mut previous = stage0;
    let n = entry.pre_stages.len();
    for (idx, stage) in entry.input_stage_order() {
        let stage_fn = &stage.function.sig.ident;
        let next = if idx == 0 {
            out_ident.clone()
        } else {
            format_ident!("{}_s{}", out_ident, n - idx)
        };
        let converted = route(quote!(#stage_fn(&mut env, #previous)));
        body.extend(quote!(let #next = #converted;));
        previous = next;
    }
    body
}

fn render_flat_struct_node(
    plan: &FlatInputPlan,
    node: &FlatStructNode,
    on_err: &TokenStream,
) -> TokenStream {
    let mut decodes = TokenStream::new();
    let mut inits = Vec::new();
    for field in &node.fields {
        match field {
            FlatFieldNode::Nested { field, node: child } => {
                decodes.extend(render_flat_struct_node(plan, child, on_err));
                let child_binding = &child.binding;
                inits.push(quote!(#field: #child_binding));
            }
            // Tag-gated groups: one `match` over the tag rebuilds the live
            // variant. ONLY that arm's leaves are converted — the inert
            // groups carry wire defaults nobody reads.
            FlatFieldNode::Sum {
                field,
                tag_leaf,
                present_leaf,
                source,
                variants,
                rust_ty,
            } => {
                let tmp = format_ident!("{}_{}", node.binding, field);
                let tag = &plan.leaves[*tag_leaf].native_ident;
                let arms = variants.iter().enumerate().map(|(t, v)| {
                    let vident = &v.rust_ident;
                    let tag_lit = proc_macro2::Literal::i32_unsuffixed(t as i32);
                    let mut pre = TokenStream::new();
                    let mut inits: Vec<TokenStream> = Vec::new();
                    for (member, leaf_idx) in &v.fields {
                        let leaf = &plan.leaves[*leaf_idx];
                        let entry = leaf.entry.as_ref().expect("sum payload leaf has an entry");
                        let wire = &leaf.native_ident;
                        let bind = format_ident!("{}_{}", tmp, wire);
                        pre.extend(render_entry_decode(entry, wire, &bind, on_err));
                        match member {
                            syn::Member::Named(n) => inits.push(quote!(#n: #bind)),
                            syn::Member::Unnamed(_) => inits.push(quote!(#bind)),
                        }
                    }
                    let ctor = if v.fields.is_empty() {
                        quote!(#source::#vident)
                    } else if matches!(v.fields[0].0, syn::Member::Named(_)) {
                        quote!(#source::#vident { #(#inits),* })
                    } else {
                        quote!(#source::#vident(#(#inits),*))
                    };
                    quote! { #tag_lit => { #pre #ctor } }
                });
                // A tag outside `0..N-1` is a binding error through the
                // ordinary channel — never a panic across the boundary.
                let bad_tag = format!(
                    "{}: invalid tag",
                    source
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default()
                );
                let build = quote! {
                    match #tag {
                        #(#arms)*
                        _ => {
                            signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, #bad_tag);
                            return #on_err;
                        }
                    }
                };
                if let Some(p) = present_leaf {
                    let present = &plan.leaves[*p].native_ident;
                    decodes.extend(quote! {
                        let #tmp: #rust_ty = if #present != 0u8 {
                            ::core::option::Option::Some(#build)
                        } else {
                            ::core::option::Option::None
                        };
                    });
                } else {
                    decodes.extend(quote! { let #tmp: #rust_ty = #build; });
                }
                inits.push(quote!(#field: #tmp));
            }
            FlatFieldNode::Value {
                field,
                value_leaf,
                present_leaf,
                direct_handle,
                optional_handle,
                rust_ty,
            } => {
                let leaf = &plan.leaves[*value_leaf];
                let wire = &leaf.native_ident;
                let tmp = format_ident!("{}_{}", node.binding, field);
                if *direct_handle {
                    let target = option_inner_type(rust_ty).unwrap_or_else(|| (**rust_ty).clone());
                    if *optional_handle {
                        decodes.extend(quote! {
                            let #tmp: #rust_ty = if #wire == 0 {
                                ::core::option::Option::None
                            } else {
                                if (#wire & 1) == 1 {
                                    signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, "Operation on a closed native handle.");
                                    return #on_err;
                                }
                                ::core::option::Option::Some(unsafe {
                                    *::std::boxed::Box::from_raw(#wire as *mut #target)
                                })
                            };
                        });
                    } else {
                        decodes.extend(quote! {
                            if #wire == 0 || (#wire & 1) == 1 {
                                signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, "Operation on a closed native handle.");
                                return #on_err;
                            }
                            let #tmp: #rust_ty = unsafe {
                                *::std::boxed::Box::from_raw(#wire as *mut #rust_ty)
                            };
                        });
                    }
                } else {
                    let entry = leaf
                        .entry
                        .as_ref()
                        .expect("ordinary leaf has converter entry");
                    if let Some(present_index) = present_leaf {
                        let present = &plan.leaves[*present_index].native_ident;
                        let inner_tmp = format_ident!("{}_value", tmp);
                        let decode = render_entry_decode(entry, wire, &inner_tmp, on_err);
                        decodes.extend(quote! {
                            let #tmp = if #present != 0u8 {
                                #decode
                                ::core::option::Option::Some(#inner_tmp)
                            } else {
                                ::core::option::Option::None
                            };
                        });
                    } else {
                        decodes.extend(render_entry_decode(entry, wire, &tmp, on_err));
                    }
                }
                inits.push(quote!(#field: #tmp));
            }
        }
    }
    let module = &node.struct_module;
    let sid = &node.struct_ident;
    let binding = &node.binding;
    let built = quote!(#module::#sid { #(#inits),* });
    if node.optional {
        let present = node.present_ident.as_ref().expect("optional node has gate");
        quote! {
            let #binding = if #present != 0u8 {
                #decodes
                ::core::option::Option::Some(#built)
            } else {
                ::core::option::Option::None
            };
        }
    } else {
        quote! {
            #decodes
            let #binding = #built;
        }
    }
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
