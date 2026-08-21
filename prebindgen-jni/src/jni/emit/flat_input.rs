//! Flattenable `data_class` inputs: leaf plans, Kotlin destructure
//! expressions, and the Rust-side reconstruct.

// `flat` as a module for `TypeKind`: the bare name in this scope is jnigen's own
// classifier (via `use super::*`), and an explicit import would shadow it.
use prebindgen_registry::{
    flat::{self, TypeRef},
    Conversions,
};

use super::*;
use crate::jni::trait_impl::{build_through_erased_wrappers, build_through_wrappers};

/// Takes the **element**, not the `syn::ItemStruct` it was parsed from (#289):
/// `flat::Field::ty` is already a `TypeRef`, so every peel below is the model's
/// answer rather than a last-path-segment test on tokens that had a reading one
/// level up. Same move `build_flat_struct_node` made for the flatten path; this
/// is the whole-object `.jobject_input()` decoder.
pub(crate) fn struct_input_body(
    ext: &Declarations,
    s: &flat::Struct,
    registry: &impl Conversions<KotlinMeta>,
    emit: &prebindgen_registry::Emit,
) -> Option<(syn::Type, syn::Expr)> {
    let struct_name = s.name.to_string();
    let struct_module = struct_module_path(ext, registry, &s.name);
    let struct_ident = &s.name;

    let mut field_preludes: Vec<TokenStream> = Vec::new();
    let mut field_init: Vec<TokenStream> = Vec::new();

    for field in &s.fields {
        // A positional field has no name to read a JVM slot by, which is what
        // the `syn::Fields::Named` guard used to say one level up. Said per
        // field now, because the element models a field list rather than a
        // `syn::Fields` shape.
        let fname_ident = field.name.clone()?;
        // The name the property was DECLARED with — `GetFieldID` takes the
        // slot's exact name, so this cannot be derived a second way.
        let camel = kotlin_property_name(&fname_ident);
        let err_prefix = format!("{struct_name}.{camel}: {{}}");
        let raw_ident = format_ident!("__{}_raw", fname_ident);

        // Defer if any field's input converter isn't resolved yet — the
        // fixed-point loop will retry on the next iteration. The field's own
        // reading straight to its entry — the `reading_of` hop only ever
        // recovered what the field already carried.
        let field_entry = ext.in_frag(&field.ty)?;
        // The optional layer off the MODEL, asked once and reused: every site
        // below that wants "is this field optional" reads this, so they cannot
        // disagree with each other the way four independent path-segment tests
        // could (#273). `option_inner_type` compared the last path segment, so
        // a field spelled `Box<Option<T>>` answered "not optional" here.
        let field_optional = field.ty.optional_inner().is_some();
        let inner = field.ty.optional_inner().unwrap_or(&field.ty);
        let field_wire = field_entry.destination.clone();
        // The field's COMPLETE decode, stages included — a `convert!` type
        // reaches its Rust value through them (`jlong -> u64 -> Duration`).
        let field_conv = composed_entry_decode(&field_entry, &raw_ident, &fname_ident);

        // Projection fields — mirror of `struct_output_body`'s kind branch:
        //  * Handle: read the JNINativeHandle object from the JVM slot,
        //    `peek()` the raw jlong, then run the per-field input converter
        //    (jlong-keyed; null handle ⇒ jlong 0 ⇒ `None` via the niche path).
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
                    let field_ty = emit.spell(&field.ty);
                    let decode = if field_optional {
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
                ProjectionKind::Unsigned64 => {
                    if field_optional {
                        let niche = matches!(
                            proj.strategy,
                            FoldStrategy::Optional(NullableKind::Niche, _)
                        );
                        let inner_conv =
                            composed_entry_decode(&ext.in_frag(inner)?, &raw_ident, &fname_ident);
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
        if ext.is_kotlin_enum_reading(inner) {
            // The NAME off the classification, not off the last path segment:
            // `Box<T>` IS `T` here, and taking the spelling apart would answer
            // about the wrapper.
            if let Some(fqn) = match inner.unwrapped().kind() {
                flat::TypeKind::Named { id, .. } => id.ident(),
                _ => None,
            }
            .and_then(|n| ext.kotlin_fqn(&TypeKey::from_ident(&n)))
            .map(|v| v.to_string())
            {
                let sig = format!("L{};", fqn.replace('.', "/"));
                let inner_conv =
                    composed_entry_decode(&ext.in_frag(inner)?, &raw_ident, &fname_ident);
                let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                let decode = if field_optional {
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
                let sig = ext
                    .in_frag(inner)
                    .and_then(|e| jni_field_access(&e.destination))
                    .and_then(|(sig, _, is_obj)| {
                        if is_obj {
                            Some(sig.to_string())
                        } else {
                            box_descriptor_for_primitive(sig).map(str::to_string)
                        }
                    })
                    .or_else(|| {
                        // The NAME off the classification, not off the last
                        // path segment.
                        match inner.unwrapped().kind() {
                            flat::TypeKind::Named { id, .. } => id.ident(),
                            _ => None,
                        }
                        .and_then(|name| {
                            ext.kotlin_fqn(&TypeKey::from_ident(&name))
                                .map(|v| format!("L{};", v.replace('.', "/")))
                        })
                    })
                    .or_else(|| {
                        // A run of values is what `kind` says it is.
                        // `pat_match_top(.., "Vec")` compared the last path
                        // segment, so a `Box<Vec<T>>` answered false.
                        if inner.sequence_elem().is_some() {
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

    // The struct's OWN delimiters, from the one place that chooses them.
    // `flat::Struct` does not record whether its fields were named — that is
    // spelling — so hard-coding braces here emitted `Unit {}` for
    // `struct Unit;` and `Empty {}` for `struct Empty()`, neither of which is
    // Rust. The `syn::Fields::Named` guard this walk replaced happened to
    // refuse both; the per-field name check cannot, because an empty struct
    // has no field to refuse. `Struct::spell` is the dual of the
    // `Alternative::spell` the sum decoder uses for exactly this.
    let ctor = emit.shape_struct(s, quote!(#struct_module::#struct_ident), &field_init);
    let body: syn::Expr = syn::parse_quote!({
        #(#field_preludes)*
        #ctor
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
/// Takes the **element**, not the `syn::ItemEnum` it was parsed from (#289):
/// `Alternative::fields` carries a `TypeRef` per payload, so the property read
/// below asks the model instead of peeling tokens. It also retires the two zips
/// this used to run — a `SumSpec` derived from the item, paired back against the
/// item it came from — because `Alternative` already is that pairing.
pub(crate) fn sum_input_body(
    ext: &Declarations,
    v: &flat::Variant,
    registry: &impl Conversions<KotlinMeta>,
    emit: &prebindgen_registry::Emit,
) -> Option<(syn::Type, syn::Expr)> {
    let key = TypeKey::from_ident(&v.name);
    let cfg = ext.types.get(&key)?;
    let sum_cfg = cfg.sum()?;
    let iface_fqn = cfg.name_spec.as_ref().map(|s| ext.fqn_of(s))?;
    let iface_path = iface_fqn.replace('.', "/");
    let source_module = ext.fn_module(registry, &v.name);
    let enum_ident = &v.name;
    let enum_name = v.name.to_string();

    let mut arms: Vec<TokenStream> = Vec::new();
    for alt in &v.alternatives {
        let vident = &alt.name;
        let kotlin_name = ext.sum_variant_class_name(sum_cfg, vident);
        // A variant class is NESTED in the interface, so its JVM binary name
        // is `Outer$Variant`.
        let jvm_class = format!("{iface_path}${kotlin_name}");

        let mut preludes: Vec<TokenStream> = Vec::new();
        let mut inits: Vec<TokenStream> = Vec::new();
        for field in &alt.fields {
            let prop = crate::jni::struct_plan::sum_field_prop_name(&field.member());
            let bind = format_ident!("__p_{}", prop);
            let err_prefix = format!("{enum_name}.{kotlin_name}.{prop}: {{}}");
            let (pre, value) = read_kotlin_property(
                ext,
                &quote!(__obj),
                &prop,
                &field.ty,
                &bind,
                &err_prefix,
                emit,
            )?;
            preludes.push(pre);
            inits.push(field.bind(&value));
        }
        // The alternative's OWN delimiters, from the one place that chooses
        // them. `B()` carries no payload and still must be written `E::B()` —
        // a three-arm `syn::Fields` match here would have had to re-derive
        // that, and `Alternative::is_empty()` cannot: `B`, `B()` and `B {}`
        // are all empty by it.
        let ctor =
            emit.shape_alternative(alt, quote!(#source_module::#enum_ident::#vident), &inits);
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
#[allow(clippy::too_many_arguments)]
fn read_kotlin_property(
    ext: &Declarations,
    receiver: &TokenStream,
    prop: &str,
    reading: &TypeRef,
    bind: &syn::Ident,
    err_prefix: &str,
    emit: &prebindgen_registry::Emit,
) -> Option<(TokenStream, TokenStream)> {
    // The payload's own reading straight to its entry, and the layer questions
    // below asked of it once — `option_inner_type` compared the last path
    // segment, so a payload spelled `Box<Option<T>>` answered "not optional"
    // four separate times here (#289).
    let entry = ext.in_frag(reading)?;
    let ty = emit.spell(reading);
    let optional = reading.optional_inner().is_some();
    let inner = reading.optional_inner().unwrap_or(reading);
    let wire = entry.destination.clone();
    let raw = format_ident!("{}_raw", bind);
    // The COMPLETE wire → Rust chain, not just the wire-facing converter: a
    // `convert!`-declared type reaches its Rust value through the rust-side
    // stages that follow (`jlong → u64 → Duration`). Stage bindings are named
    // off `bind`, so two payloads of the same type in one variant do not
    // collide.
    let conv = composed_property_decode(&entry, bind);

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
            let decode = if optional {
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
    if ext.is_kotlin_enum_reading(inner) {
        // The NAME off the classification, not off the last path segment.
        let fqn = match inner.unwrapped().kind() {
            flat::TypeKind::Named { id, .. } => id.ident(),
            _ => None,
        }
        .and_then(|n| ext.kotlin_fqn(&TypeKey::from_ident(&n)))
        .map(|v| v.to_string())?;
        let sig = format!("L{};", fqn.replace('.', "/"));
        let obj = format_ident!("{}_obj", bind);
        // Under `Option`, JVM null is `None` and the INNER converter decodes
        // the discriminant; the outer converter would expect a boxed Integer.
        let decode = if optional {
            let inner_conv = composed_entry_decode(&ext.in_frag(inner)?, &raw, bind);
            quote! {
                let #bind = if #obj.is_null() {
                    ::core::option::Option::None
                } else {
                    let #raw: jni::sys::jint = env.call_method(&#obj, "getValue", "()I", &[])
                        .and_then(|val| val.i())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    ::core::option::Option::Some(#inner_conv)
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
            let sig = match inner.unwrapped().kind() {
                // The NAME off the classification, not off the last path
                // segment: `Box<T>` IS `T` here.
                flat::TypeKind::Named { id, .. } => id.ident(),
                _ => None,
            }
            .and_then(|name| ext.kotlin_fqn(&TypeKey::from_ident(&name)))
            .map(|v| format!("L{};", v.replace('.', "/")))
            .or_else(|| {
                // A run of values is what `kind` says it is.
                // `pat_match_top(.., "Vec")` compared the last path segment, so
                // a `Box<Vec<T>>` answered false.
                inner
                    .sequence_elem()
                    .is_some()
                    .then(|| "Ljava/util/List;".to_string())
            })
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
/// stages a custom [`convert!`](prebindgen_registry::convert) declaration inserts.
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
    entry: &prebindgen_registry::ConverterImpl<KotlinMeta>,
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
    entry: &prebindgen_registry::ConverterImpl<KotlinMeta>,
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
    pub entry: Option<prebindgen_registry::ConverterImpl<KotlinMeta>>,
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
        /// `Some(target)` iff this field crosses as a raw handle jlong, where
        /// `target` is the type the `Box` points at — the field's own type with
        /// its optional layer peeled, **taken off the model at plan time**.
        ///
        /// Paired rather than a `bool` beside a spelling the renderer re-peels:
        /// `option_inner_type` compared the last path segment, so a field
        /// spelled `Box<Option<T>>` would have handed `Box::from_raw` the wrong
        /// target. There is no reading here to ask — `FlatFieldNode` is an
        /// emission IR and tokens are what it is for — so the answer travels
        /// from where the reading was (#289).
        /// The handle type a leaf reconstructs by `Box::from_raw` — the
        /// reading, spelled at the emit site like every other generated type.
        direct_handle: Option<Box<prebindgen_registry::flat::TypeRef>>,
        optional_handle: bool,
        rust_ty: Box<prebindgen_registry::flat::TypeRef>,
        /// The transparent wrappers this field's spelling adds over its
        /// classification, outermost first — put back wherever the decode
        /// **rebuilds** the value (an `Option::Some`/`None` literal) rather than
        /// running the field's own converter, which already yields the spelling.
        wrappers: Vec<&'static str>,
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
        rust_ty: Box<prebindgen_registry::flat::TypeRef>,
        /// The transparent wrappers this field's spelling adds over its
        /// classification, outermost first — put back wherever the decode
        /// **rebuilds** the value (an `Option::Some`/`None` literal) rather than
        /// running the field's own converter, which already yields the spelling.
        wrappers: Vec<&'static str>,
    },
}

/// One alternative of a [`FlatFieldNode::Sum`].
pub(crate) struct FlatSumVariant {
    pub rust_ident: syn::Ident,
    /// This variant's payload: how each field is addressed when rebuilding
    /// it, paired with the leaf carrying its value. Empty for a unit variant.
    pub fields: Vec<(syn::Member, usize)>,
}

/// The three layers a specialized struct lowering descends through, each paired
/// with the reading whose spelling it must satisfy.
///
/// `kind` decides what the destination sees; the **conversion** follows the
/// syntax, and this lowering does not decode its parameter — it emits a literal
/// `S { .. }`, wraps it in `Option::Some`, and hands it to the source function.
/// Rebuilding from the classification alone produces the *stripped* type, so a
/// parameter spelled `Box<Option<S>>` would receive an `Option<S>`: `E0308` in
/// the generated crate.
///
/// So each layer keeps its own reading, and the emitter puts that layer's
/// wrappers back as it builds outward — see [`RebuildTarget::wrap_core`] and its
/// siblings. Collected on the way **down** because an erasure sits *outside* the
/// layer it wraps: `Box<&S>` classifies as `Ref`, and reading `kind` first would
/// leave the `Box` unreachable.
pub(crate) struct RebuildTarget {
    /// Wrappers over the borrow, if there is one — the `Box` of `Box<&S>`.
    arg: Vec<&'static str>,
    /// Wrappers over the `Option` — the `Box` of `Box<Option<S>>`.
    under_borrow: Vec<&'static str>,
    /// Wrappers over the `S { .. }` literal — the `Box` of `Option<Box<S>>`.
    core: Vec<&'static str>,
    /// `true` when the source fn takes `&Struct`.
    pub by_ref: bool,
    /// `true` when the value is `Option`-wrapped.
    pub optional: bool,
}

impl RebuildTarget {
    /// Put back the wrappers standing over the `S { .. }` literal —
    /// `Option<Box<S>>` wraps here, not at [`Self::wrap_optional`].
    pub fn wrap_core(&self, e: TokenStream) -> TokenStream {
        Self::wrap(&self.core, e)
    }

    /// Put back the wrappers over the `Option<..>` — the `Box` of
    /// `Box<Option<S>>`.
    ///
    /// A no-op when the parameter is not optional, for the same reason
    /// [`Self::wrap_arg`] is one when it is not a borrow: with no `Option` to
    /// peel, `under_borrow` and `core` are the *same reading*, and wrapping at
    /// both would apply one layer twice.
    ///
    /// Stated once as the rule the three share: **a layer's wrappers are
    /// applied only where that layer exists**, and the innermost always applies.
    pub fn wrap_optional(&self, e: TokenStream) -> TokenStream {
        if !self.optional {
            return e;
        }
        Self::wrap(&self.under_borrow, e)
    }

    /// Put back the wrappers over the **borrow** — the `Box` of `Box<&S>`,
    /// which goes on after the call site has added its `&`.
    ///
    /// A no-op when the parameter is not a borrow, and that is not an
    /// optimisation: with no `&` to peel, `arg` and `under_borrow` are the *same
    /// reading*, so wrapping at both would apply one layer twice —
    /// `Box::new(Box::new(v))` for a `Box<Option<S>>` parameter.
    pub fn wrap_arg(&self, e: TokenStream) -> TokenStream {
        if !self.by_ref {
            return e;
        }
        Self::wrap(&self.arg, e)
    }

    /// Every wrap goes through the one helper, and every layer was proved
    /// buildable by [`rebuildable_target`] before a plan existed — so a `None`
    /// here would mean the descent and the emission disagree about the same
    /// reading, which is a bug in this file rather than an unsupported source.
    fn wrap(names: &[&'static str], e: TokenStream) -> TokenStream {
        build_through_wrappers(names, e)
            .expect("every layer was checked buildable when the plan was built")
    }
}

/// Descend `&` then `Option<…>` off the model to the struct a specialized
/// lowering will **rebuild**, keeping each layer's reading so its spelling can
/// be restored.
///
/// One function because it is one rule, and the layers are checked **on the way
/// down**: an erasure sits outside the layer it wraps, so `Box<&S>` classifies
/// as `Ref` and interpreting `kind` before asking would discard the `Box`.
///
/// The only refusal left is a wrapper the adapter cannot **build** — `Cow`, by
/// policy rather than by impossibility (see its `WRAPPER_OPS` row). A wrapped
/// spelling that declines here keeps the general converter path, which is
/// correct if less direct.
fn rebuildable_target(arg: &TypeRef) -> Option<(RebuildTarget, &TypeRef)> {
    // A probe per layer: "can this spelling be rebuilt at all", asked before the
    // peel that would hide it. The token is irrelevant — only the `Option` is.
    let buildable = |t: &TypeRef| build_through_erased_wrappers(t, quote!(__probe)).map(|_| ());
    buildable(arg)?;
    let by_ref = arg.borrow_target().is_some();
    let t1 = arg.borrow_target().unwrap_or(arg);
    buildable(t1)?;
    let optional = t1.optional_inner().is_some();
    let inner = t1.optional_inner().unwrap_or(t1);
    // The struct is rebuilt BY NAME (`S { .. }`), and its own spelling may add a
    // wrapper over that name — `Box<S>` gets its `Box::new` at `wrap_core`.
    buildable(inner)?;
    // Only the wrapper LISTS are kept: they are all a rebuild uses, and a
    // `TypeRef` apiece would put ~800 bytes into every `InputKind`.
    Some((
        RebuildTarget {
            arg: arg.erased_wrappers(),
            under_borrow: t1.erased_wrappers(),
            core: inner.erased_wrappers(),
            by_ref,
            optional,
        },
        inner,
    ))
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
    /// The layer readings the rebuild has to satisfy — carried rather than
    /// re-derived at the emission sites, so the descent is stated once.
    pub target: RebuildTarget,
}

// `impl_into_target` lived here: it extracted `S` from an `impl Into<S> + …`
// spelling for `build_flat_input_plan`'s struct-target peel. It is gone because
// that peel now takes a reading, and the model REFUSES `impl Trait` that is not
// the callback form (`UnsupportedTypeReason::DisallowedImplTrait`) — so a
// parameter spelled `impl Into<S>` never becomes a `TypeRef` and never reached
// the call. `cargo check` confirmed it dead rather than the reasoning alone.
// jnigen's actual `impl Into<…>` support is elsewhere: plugin wrapper exts build
// a `ConverterImpl::function` by hand via `Declarations::input_converter_name`,
// which never consults this.
// `flat_probe_inner` lived here: it peeled `&` then `Option` off a SPELLING to
// reach the type an enum probe should ask about. Its last caller now asks
// `is_kotlin_enum_reading`, whose `enum_probe` peels the same two layers off the
// model — so `Box<Priority>` probes as `Priority` where this answered about the
// wrapper (#289).

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
            other => {
                // An inert primitive-array slot is an EMPTY array of its own
                // type, never null — the slot is non-nullable in the factory.
                if let Some(n) = crate::jni::wire_access::kotlin_array_of_descriptor(other) {
                    return Some(format!("{n}(0)"));
                }
                "null"
            }
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

fn wire_kotlin_type(entry: &prebindgen_registry::ConverterImpl<KotlinMeta>) -> String {
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
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
    sum_reading: &TypeRef,
    field: syn::Ident,
    optional: bool,
    native_prefix: &str,
    field_ref: &str,
    nullable_access: bool,
    field_reading: &TypeRef,
    leaves: &mut Vec<FlatLeaf>,
) -> Option<FlatFieldNode> {
    let rust_ty = field_reading;

    // The NAME off the classification, and then the ELEMENT — `enum_item`
    // hands back only the `syn::ItemEnum`, deliberately, so a consumer that
    // acts on the Variant/Enum distinction asks `declared_type` (#289).
    let ident = match sum_reading.unwrapped().kind() {
        flat::TypeKind::Named { id, .. } => id.ident(),
        _ => None,
    }?;
    let flat::Type::Variant(sum) = registry.flat().declared_type(&ident)? else {
        return None;
    };
    let cfg = ext.types.get(&TypeKey::from_ident(&ident))?;
    let sum_cfg = cfg.sum()?;
    let iface_fqn = cfg.name_spec.as_ref().map(|s| ext.fqn_of(s))?;

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
        entry: prebindgen_registry::ConverterImpl<KotlinMeta>,
        access_tail: String,
        nullable_wire: bool,
    }
    let mut planned: Vec<Planned> = Vec::new();
    for alt in &sum.alternatives {
        let kotlin = ext.sum_variant_class_name(sum_cfg, &alt.name);
        let mut fields = Vec::new();
        for field in &alt.fields {
            // The payload's own reading straight to its entry.
            let entry = ext.in_frag(&field.ty)?;
            // A projection payload (handle) carries ownership
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
            let member = field.member();
            let prop = crate::jni::struct_plan::sum_field_prop_name(&member);
            let slot = crate::jni::struct_plan::sum_slot_fragment(&kotlin, &prop);
            // `(<base>.field as? io.x.E.V)?.prop` — inert groups yield null,
            // so a primitive slot takes its zero and an object slot stays
            // nullable. The Rust side only converts the live group.
            //
            // An `enum_class` payload is a Kotlin enum object whose wire is
            // the `jint` discriminant, so the access reads `.value` — without
            // it the slot would be `Priority?` where the wire wants `Int`.
            let read = if ext.is_kotlin_enum_reading(&field.ty) {
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
                member,
                PlannedLeaf {
                    native: format!("{native_prefix}_{slot}"),
                    entry: entry.clone(),
                    access_tail,
                    nullable_wire,
                },
            ));
        }
        planned.push(Planned {
            rust_ident: alt.name.clone(),
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
        wrappers: field_reading.erased_wrappers(),
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
    entry: &prebindgen_registry::ConverterImpl<KotlinMeta>,
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
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
    param_name: &syn::Ident,
    arg: &TypeRef,
) -> Result<Option<FlatInputPlan>, FlatInputError> {
    // 1. Resolve the struct target through `&` and `Option<…>` — off the model,
    //    keeping each layer's reading so the rebuild can restore its spelling.
    let Some((target, inner)) = rebuildable_target(arg) else {
        return Ok(None);
    };
    let (by_ref, optional) = (target.by_ref, target.optional);
    // `impl Into<S>` is NOT peeled here, and cannot be: the model refuses
    // `impl Trait` that is not the callback form (`DisallowedImplTrait`), so a
    // parameter spelled that way never becomes a reading and never reaches this
    // function. The former `impl_into_target` call was already unreachable from
    // every caller — see the sibling helper's doc.
    // The name off the classification, not off the last path segment: `Box<S>`
    // IS `S` here, and taking the spelling apart would answer about the wrapper.
    let flat::TypeKind::Named { id, .. } = inner.unwrapped().kind() else {
        return Ok(None);
    };
    let Some(name) = id.ident() else {
        return Ok(None);
    };
    // The ELEMENT, not the item it was parsed from: its fields already carry
    // readings, which is the whole of #289.
    let Some(st) = registry.flat().struct_type(&name) else {
        return Ok(None);
    };
    // The DECLARATION is keyed by the type, not by the spelling: a
    // `Box<Payload>` parameter is a `Payload` to Kotlin and must find
    // `Payload`'s data-class declaration. Keying by spelling looked up
    // `Box < Payload >`, found nothing, and silently dropped the parameter to
    // the general converter — the flatten lowering was unreachable for every
    // wrapped core.
    let key = inner.stripped_key();
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
    // The parameter's own reading straight to its entry — no spell-and-look-back.
    let Some(entry) = ext.in_frag(arg) else {
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
        target,
    }))
}

#[allow(clippy::too_many_arguments)]
/// Takes the **element**, not the `syn::ItemStruct` it was parsed from (#289):
/// `flat::Field::ty` is already a `TypeRef`, so every peel below is the model's
/// answer rather than a last-path-segment test on tokens that had a reading one
/// level up.
///
/// That matters here and not only on principle. `option_inner_type` reads the
/// last path segment, so a field spelled `Box<Option<T>>` answered "not
/// optional" and crossed as one boxed object; the model says `Optional` and it
/// takes the decoupled `(present, value)` pair like its bare twin. The emitter
/// then has to put the `Box` back — which is why this migration could not land
/// before the rebuild did.
fn build_flat_struct_node(
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
    st: &flat::Struct,
    optional: bool,
    native_prefix: &str,
    access_prefix: &str,
    nullable_context: bool,
    root: &TypeKey,
    stack: &mut Vec<TypeKey>,
    leaves: &mut Vec<FlatLeaf>,
) -> Result<FlatStructNode, FlatInputError> {
    let node_key = TypeKey::from_ident(&st.name);
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
    stack.push(node_key);
    let present_ident = if optional {
        let native = format!("{native_prefix}_present");
        push_present_leaf(leaves, &native, format!("{access_prefix} != null"), None);
        Some(format_ident!("{native}"))
    } else {
        None
    };
    let mut fields = Vec::new();
    for field in &st.fields {
        // A positional field has no name to derive a Kotlin property from, which
        // is what "only named-field structs can flatten" used to say one level
        // up. Said per field now, because the element models a field list rather
        // than a `syn::Fields` shape.
        let Some(fident) = field.name.clone() else {
            return Err(flat_error(
                root,
                native_prefix,
                "only named-field structs can flatten",
            ));
        };
        let fcamel = kotlin_property_name(&fident);
        let child_native = format!("{native_prefix}_{}", fident);
        let field_ref = if nullable_context {
            format!("{access_prefix}?.{fcamel}")
        } else {
            format!("{access_prefix}.{fcamel}")
        };
        // The optional layer off the MODEL, asked once and reused: every site
        // below that wants "is this field optional" reads this, so they cannot
        // disagree with each other the way seven independent path-segment tests
        // could (#273).
        let field_optional = field.ty.optional_inner().is_some();
        let nested = field.ty.optional_inner().unwrap_or(&field.ty);
        // A data-carrying enum flattens into a tag plus one group per variant.
        // `None` means some payload is not leaf-shaped — fall through and let
        // it cross as one object through its own converter.
        if matches!(ext.type_kind(registry, &nested.key()), TypeKind::Sum) {
            if let Some(node) = build_flat_sum_field(
                ext,
                registry,
                nested,
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
        } = ext.type_kind(registry, &nested.key())
        {
            if cfg.name_spec.is_some() && !cfg.special_decl() && !cfg.jobject_input {
                let child_optional = field_optional;
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
        // The field's own reading straight to its entry — the `reading_of` hop
        // only ever recovered what the field already carried.
        let Some(fentry) = ext.in_frag(&field.ty) else {
            return Err(flat_error(
                root,
                &path,
                format!("field type `{}` has no input converter", field.ty.key()),
            ));
        };

        // Nullable primitive/enum with no niche: keep the allocation-free
        // `(present, value)` representation at every recursion depth.
        if let Some(inner_reading) = field.ty.optional_inner() {
            if inner_reading.borrow_target().is_none() {
                if let Some(inner) = ext.in_frag(inner_reading) {
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
                            let value_access = if ext.is_kotlin_enum_reading(inner_reading) {
                                format!("{field_ref}?.value ?: {}", prim.kotlin_zero())
                            } else {
                                format!("{field_ref} ?: {}", prim.kotlin_zero())
                            };
                            let value_index = push_value_leaf(
                                leaves,
                                &format!("{child_native}_value"),
                                fident.clone(),
                                &inner,
                                value_access,
                                false,
                            );
                            fields.push(FlatFieldNode::Value {
                                field: fident,
                                value_leaf: value_index,
                                present_leaf: Some(present_index),
                                direct_handle: None,
                                optional_handle: false,
                                rust_ty: Box::new(field.ty.clone()),
                                wrappers: field.ty.erased_wrappers(),
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
                if let Some(inner_reading) = field.ty.optional_inner() {
                    if JniPrim::from_wire(&fentry.destination).is_none() {
                        let inner = ext.in_frag(inner_reading).ok_or_else(|| {
                            flat_error(
                                root,
                                &path,
                                format!(
                                    "unsigned field representation `{}` has no input converter",
                                    inner_reading.key()
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
                            &inner,
                            format!("{field_ref}?.toLong() ?: 0L"),
                            false,
                        );
                        fields.push(FlatFieldNode::Value {
                            field: fident,
                            value_leaf: value_index,
                            present_leaf: Some(present_index),
                            direct_handle: None,
                            optional_handle: false,
                            rust_ty: Box::new(field.ty.clone()),
                            wrappers: field.ty.erased_wrappers(),
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
                    let optional_handle = field_optional;
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
                        direct_handle: Some(Box::new(nested.clone())),
                        optional_handle,
                        rust_ty: Box::new(field.ty.clone()),
                        wrappers: field.ty.erased_wrappers(),
                    });
                    continue;
                }
                ProjectionKind::Unsigned64 => {
                    let is_opt = field_optional;
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
                        &fentry,
                        access,
                        false,
                    );
                    fields.push(FlatFieldNode::Value {
                        field: fident,
                        value_leaf: value_index,
                        present_leaf: None,
                        direct_handle: None,
                        optional_handle: false,
                        rust_ty: Box::new(field.ty.clone()),
                        wrappers: field.ty.erased_wrappers(),
                    });
                    continue;
                }
            }
        }

        let field_is_option = field_optional;
        // The enum branch is self-contained: when it coalesces (`?.value ?: 0`)
        // it already yields a non-null `Int`, so block (B) below must not append
        // a second default (which produced the dead `?: 0 ?: 0`, issue #144).
        let mut enum_coalesced = false;
        // The enum probe off the MODEL (`enum_probe` peels the same `&`/`Option`
        // layers `flat_probe_inner` peeled off tokens), so a `Box<Priority>`
        // field answers as a `Priority` does.
        let mut access = if ext.is_kotlin_enum_reading(&field.ty) {
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
            &fentry,
            access,
            (field_is_option || nullable_context) && is_jobject_shaped_wire(&fentry.destination),
        );
        fields.push(FlatFieldNode::Value {
            field: fident,
            value_leaf: value_index,
            present_leaf: None,
            direct_handle: None,
            optional_handle: false,
            rust_ty: Box::new(field.ty.clone()),
            wrappers: field.ty.erased_wrappers(),
        });
    }
    stack.pop();
    Ok(FlatStructNode {
        struct_module: struct_module_path(ext, registry, &st.name),
        struct_ident: st.name.clone(),
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
    emit: &prebindgen_registry::Emit,
) -> (TokenStream, TokenStream) {
    let reconstruct = render_flat_struct_node(plan, &plan.root, Some(&plan.target), on_err, emit);
    let root_binding = &plan.root.binding;
    let prelude = quote! {
        #reconstruct
        let #arg_ident = #root_binding;
    };
    // The borrow, then the wrappers standing OVER it — `Box<&S>` is
    // `Box::new(&arg)`, in that order, because the erasure sits outside the
    // layer it wraps and the `&` is that layer.
    let borrowed = if plan.by_ref {
        quote!(&#arg_ident)
    } else {
        quote!(#arg_ident)
    };
    (prelude, plan.target.wrap_arg(borrowed))
}

fn render_entry_decode(
    entry: &prebindgen_registry::ConverterImpl<KotlinMeta>,
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

/// `target` is `Some` for the parameter's ROOT node, whose spelling may add
/// transparent wrappers the rebuild has to restore, and `None` for a nested one
/// — a nested struct is reached through a field, and a field's own wrappers are
/// applied where that field is decoded.
fn render_flat_struct_node(
    plan: &FlatInputPlan,
    node: &FlatStructNode,
    target: Option<&RebuildTarget>,
    on_err: &TokenStream,
    emit: &prebindgen_registry::Emit,
) -> TokenStream {
    let mut decodes = TokenStream::new();
    let mut inits = Vec::new();
    for field in &node.fields {
        match field {
            FlatFieldNode::Nested { field, node: child } => {
                decodes.extend(render_flat_struct_node(plan, child, None, on_err, emit));
                let child_binding = &child.binding;
                inits.push(quote!(#field: #child_binding));
            }
            // Tag-gated groups: one `match` over the tag rebuilds the live
            // variant. ONLY that arm's leaves are converted — the inert
            // groups carry wire defaults nobody reads.
            FlatFieldNode::Sum {
                wrappers,
                field,
                tag_leaf,
                present_leaf,
                source,
                variants,
                rust_ty,
            } => {
                let tmp = format_ident!("{}_{}", node.binding, field);
                // The slot's ascription, spelled from the reading the node
                // carries — see the comment below on why the type is written.
                let rust_ty = emit.spell(rust_ty);
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
                // The rebuilt value, then the wrappers this FIELD's spelling
                // adds — the slot is ascribed `#rust_ty`, so a `Box<Option<T>>`
                // field needs its `Box` back. Only the rebuilding arms wrap: the
                // fall-through below runs the field's own converter, which
                // already yields the spelling.
                let wrap = |e: TokenStream| {
                    build_through_wrappers(wrappers, e)
                        .expect("a field spelling the plan accepted is buildable")
                };
                if let Some(p) = present_leaf {
                    let present = &plan.leaves[*p].native_ident;
                    let gated = wrap(quote! {
                        if #present != 0u8 {
                            ::core::option::Option::Some(#build)
                        } else {
                            ::core::option::Option::None
                        }
                    });
                    decodes.extend(quote! { let #tmp: #rust_ty = #gated; });
                } else {
                    let built = wrap(build);
                    decodes.extend(quote! { let #tmp: #rust_ty = #built; });
                }
                inits.push(quote!(#field: #tmp));
            }
            FlatFieldNode::Value {
                wrappers,
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
                let rust_ty = emit.spell(rust_ty);
                let wrap = |e: TokenStream| {
                    build_through_wrappers(wrappers, e)
                        .expect("a field spelling the plan accepted is buildable")
                };
                if let Some(target) = direct_handle {
                    let target_ty = emit.spell(target);
                    if *optional_handle {
                        let gated = wrap(quote! {
                            if #wire == 0 {
                                ::core::option::Option::None
                            } else {
                                if (#wire & 1) == 1 {
                                    signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, "Operation on a closed native handle.");
                                    return #on_err;
                                }
                                ::core::option::Option::Some(unsafe {
                                    *::std::boxed::Box::from_raw(#wire as *mut #target_ty)
                                })
                            }
                        });
                        decodes.extend(quote! { let #tmp: #rust_ty = #gated; });
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
                        let gated = wrap(quote! {
                            if #present != 0u8 {
                                #decode
                                ::core::option::Option::Some(#inner_tmp)
                            } else {
                                ::core::option::Option::None
                            }
                        });
                        decodes.extend(quote! { let #tmp = #gated; });
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
    // The struct literal, then the wrappers the CORE spelling adds over it —
    // `Option<Box<S>>` gets its `Box::new` here, inside the present gate, not
    // around it. `None` for a nested node, whose own layers are its field's
    // question rather than the parameter's.
    let built = match target {
        Some(t) => t.wrap_core(quote!(#module::#sid { #(#inits),* })),
        None => quote!(#module::#sid { #(#inits),* }),
    };
    // …and the wrappers over the `Option` (or over the bare value) go around
    // the whole gate — the `Box` of `Box<Option<S>>`.
    let outer = |e: TokenStream| match target {
        Some(t) => t.wrap_optional(e),
        None => e,
    };
    if node.optional {
        let present = node.present_ident.as_ref().expect("optional node has gate");
        // `#decodes` belongs **inside** the true arm, and that is a correctness
        // requirement rather than a tidiness one: when the Kotlin object is null
        // its leaves carry inert placeholders, and decoding them is not
        // side-effect-free. A required handle field arrives as pointer `0`, so
        // an unconditional direct-handle decode calls `signal_binding_error` and
        // returns instead of delivering `None`; an enum with no discriminant `0`
        // and a fallible custom converter fail on their placeholders the same
        // way.
        //
        // The wrapper goes around the whole conditional, which is what the
        // `Option` layer wraps — so `outer` applies to the `if`, never between
        // it and the decodes.
        let gate = outer(quote! {
            if #present != 0u8 {
                #decodes
                ::core::option::Option::Some(#built)
            } else {
                ::core::option::Option::None
            }
        });
        quote! {
            let #binding = #gate;
        }
    } else {
        let value = outer(built);
        quote! {
            #decodes
            let #binding = #value;
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
    /// The transparent wrappers the parameter's spelling adds over `Optional`,
    /// outermost first — what the emitter puts back.
    ///
    /// This plan **rebuilds** its parameter — the emitter writes a literal
    /// `Option::Some(v)` / `Option::None` and hands it to the source fn — so a
    /// parameter spelled `Box<Option<T>>` must receive a `Box`, not the bare
    /// `Option` the classification names. Carried rather than re-derived at the
    /// two emission sites, which would be the same rule stated twice; the list
    /// rather than the reading, because that is all a rebuild uses and this
    /// plan sits in `InputKind`, whose size every variant pays.
    pub arg_wrappers: Vec<&'static str>,
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
    ext: &Declarations,
    param_name: &syn::Ident,
    arg: &TypeRef,
) -> Option<OptionScalarInputPlan> {
    // The wrappers this spelling adds over `Optional` have to be BUILDABLE, not
    // absent: the emitter rebuilds a bare `Option::Some(v)` and hands it to the
    // source fn, so a parameter spelled `Box<Option<T>>` receives a `Box`.
    // Asked here, before the peel, because an erasure sits outside the layer it
    // wraps — and asked as "can I build it" rather than "is there one", so the
    // only refusal left is a wrapper `WRAPPER_OPS` declines (`Cow`).
    build_through_erased_wrappers(arg, quote!(__probe))?;
    let inner = arg.optional_inner()?;
    // `Option<&T>` is the nullable-borrow / handle path, not a scalar.
    if inner.borrow_target().is_some() {
        return None;
    }
    // The layer's own reading straight to its entry — no spell-and-look-back.
    let inner_entry = ext.in_frag(inner)?;
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
    // The reading-taking probe: `Option<Box<Priority>>` now answers TRUE, where
    // keying on the spelling asked about `Box < Priority >` and found nothing —
    // the #270/#272 family again. The probe also peels an optional, which cannot
    // matter here: a nested `Option<Option<enum>>` has a BOXED wire, and
    // `JniPrim::from_wire` above accepts only the eight `j*` primitives, so it
    // has already returned by this line.
    let is_enum = ext.is_kotlin_enum_reading(inner);
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
        arg_wrappers: arg.erased_wrappers(),
    })
}

// ──────────────────────────────────────────────────────────────────────
// Slice / Vec input → Rust-side Vec handle (built by pushing leaves)
// ──────────────────────────────────────────────────────────────────────
