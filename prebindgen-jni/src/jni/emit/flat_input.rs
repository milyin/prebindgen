//! Flattenable `data_class` inputs: leaf plans, Kotlin destructure
//! expressions, and the Rust-side reconstruct.

// `flat` as a module for `TypeKind`: the bare name in this scope is jnigen's own
// classifier (via `use super::*`), and an explicit import would shadow it.
use prebindgen_registry::{
    flat::{self, TypeRef},
    Conversions,
};

use super::*;
use crate::jni::trait_impl::build_through_erased_wrappers;

/// Takes the **element**, not the `syn::ItemStruct` it was parsed from (#289):
/// `flat::Field::ty` is already a `TypeRef`, so every peel below is the model's
/// answer rather than a last-path-segment test on tokens that had a reading one
/// level up. Same move `build_flat_struct_node` made for the flatten path; this
/// is the whole-object `.jobject_input()` decoder.
pub(crate) fn struct_input_body(
    ext: &Declarations,
    s: &flat::Struct,
    registry: &impl Conversions,
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
        field_entry.activate();
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
                        let #fname_ident = #field_conv;
                    });
                }
                ProjectionKind::Unsigned64 => {
                    if field_optional {
                        let niche = matches!(
                            proj.strategy,
                            FoldStrategy::Optional(NullableKind::Niche, _)
                        );
                        let inner_entry = ext.in_frag(inner)?;
                        inner_entry.activate();
                        let inner_conv =
                            composed_entry_decode(&inner_entry, &raw_ident, &fname_ident);
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
        // (The generic converters can't be used here: both are jint-keyed,
        // while this Kotlin property is an enum object or null.)
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
                let inner_entry = ext.in_frag(inner)?;
                inner_entry.activate();
                let inner_conv = composed_entry_decode(&inner_entry, &raw_ident, &fname_ident);
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
    registry: &impl Conversions,
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
            let (pre, value) =
                read_kotlin_property(ext, &quote!(__obj), &prop, &field.ty, &bind, &err_prefix)?;
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
) -> Option<(TokenStream, TokenStream)> {
    // The payload's own reading straight to its entry, and the layer questions
    // below asked of it once — `option_inner_type` compared the last path
    // segment, so a payload spelled `Box<Option<T>>` answered "not optional"
    // four separate times here (#289).
    let entry = ext.in_frag(reading)?;
    entry.activate();
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
                    let #bind = #conv;
                },
                quote!(#bind),
            ));
        }
    }
    // An enum property — bare or under `Option` — is stored as the Kotlin
    // **enum object**, so it is read through `getValue`, NOT through the
    // generic converters: both are `jint`-keyed, and neither matches what the
    // JVM slot actually holds. `struct_input_body` makes the same
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
            let inner_entry = ext.in_frag(inner)?;
            inner_entry.activate();
            let inner_conv = composed_entry_decode(&inner_entry, &raw, bind);
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
/// stopping at [`ConverterImpl::converter_ident`] would bind the *representation*
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
/// boundary as its own wrapper parameter, and the three coordinated sites — the
/// native wrapper signature, the `JNINative` extern declaration and the Kotlin
/// call-site destructure — read it so they cannot drift in order, type, or
/// nullability.
///
/// One JNI parameter a flattened `data_class` occupies, as the site names it.
///
/// A projection of the fragment's [`Wire`](crate::jni::compile::Wire) and
/// nothing more: the wire says what crosses and how Kotlin reaches it, and this
/// pairs that with the one thing a wire cannot know — the parameter name the
/// site hangs it off.
pub(crate) struct FlatLeaf {
    /// Native wrapper parameter ident — also the decode source.
    pub native_ident: syn::Ident,
    /// Kotlin `external fun` parameter name (camelCase).
    pub kt_name: String,
    /// The wire itself.
    pub wire: crate::jni::compile::Wire,
}

impl FlatLeaf {
    /// This wire as one parameter of the site named `param`.
    fn of(param: &syn::Ident, wire: &crate::jni::compile::Wire) -> Self {
        let native = format!("{param}_{}", wire.path.replace('.', "_"));
        Self {
            native_ident: format_ident!("{native}"),
            kt_name: snake_to_camel(&native),
            wire: wire.clone(),
        }
    }

    /// Native wire type, lifetime-annotated for object wires.
    pub fn native_wire_ty(&self) -> TokenStream {
        annotate_jobject_with_lifetime(&self.wire.ty, "a").to_token_stream()
    }

    /// Kotlin `external fun` parameter type (incl. a trailing `?`).
    pub fn kt_wire_ty(&self) -> &str {
        &self.wire.kt_ty
    }

    /// Per-field input converter ident (`None` for a synthetic gate or tag).
    pub fn conv(&self) -> Option<&syn::Ident> {
        self.wire.conv()
    }

    /// The complete conversion, stages included, or `None` for a gate or tag.
    pub fn entry(&self) -> Option<&prebindgen_registry::ConverterImpl<KotlinMeta>> {
        self.wire.entry.as_ref()
    }

    /// Whether this is a synthetic `…Present: Boolean` gate.
    pub fn is_present_flag(&self) -> bool {
        self.wire.is_present_flag()
    }

    /// Whether the handle access can be null, either because the field itself
    /// is optional or because an optional ancestor gates it.
    pub fn handle_nullable(&self) -> bool {
        self.wire.handle_nullable
    }

    /// Kotlin call-site destructure expression feeding this leaf, rooted at
    /// `base` — the object expression at this call site (the camelCase param
    /// name, `this` for a promoted receiver, `__e` for the vec-build loop
    /// variable).
    pub fn kt_access(&self, base: &str) -> String {
        self.wire.access.render(base)
    }

    /// The Kotlin expression yielding the handle this leaf carries, rooted at
    /// `base` — the thing the lock scaffold locks and `markConsumed()`s.
    /// `None` when the leaf is not a handle.
    pub fn kt_handle_target(&self, base: &str) -> Option<String> {
        self.wire
            .handle_target
            .as_ref()
            .map(|walk| crate::jni::compile::reached(base, walk))
    }

    /// Native call argument for this leaf. Handle pointers are bound under
    /// the unified lock scaffold; every other leaf is read directly from the
    /// Kotlin object graph.
    pub fn kt_call_arg(&self, base: &str) -> String {
        if self.wire.handle_target.is_some() {
            format!("{}_ptr", self.kt_name)
        } else {
            self.kt_access(base)
        }
    }
}

/// The outer layer a flattened call argument restores after its registry chain
/// returns the value represented by the crossing.
///
/// Product/Optional/Choice construction belongs to the registry chain. The
/// renderer only adds the source call's borrow; wrappers outside that borrow
/// still belong afterwards (`Box<&S>` -> `Box::new(&value)`).
pub(crate) struct RebuildTarget {
    /// Wrappers over the borrow, if there is one — the `Box` of `Box<&S>`.
    arg: Vec<&'static str>,
    borrow: RebuildBorrow,
}

#[derive(Clone, Copy)]
enum RebuildBorrow {
    None,
    Outer { mutable: bool },
    OptionalInner { mutable: bool },
}

impl RebuildTarget {
    /// Put back the wrappers over the **borrow** — the `Box` of `Box<&S>`,
    /// which goes on after the call site has added its `&`.
    ///
    /// A no-op when the parameter is not a borrow: wrappers at or below the
    /// crossing are already restored by the registry chain.
    pub fn call_arg(&self, ident: &syn::Ident) -> TokenStream {
        let value = match self.borrow {
            RebuildBorrow::None => quote!(#ident),
            RebuildBorrow::Outer { mutable: false } => quote!(&#ident),
            RebuildBorrow::Outer { mutable: true } => quote!(&mut #ident),
            RebuildBorrow::OptionalInner { mutable: false } => quote!(#ident.as_ref()),
            RebuildBorrow::OptionalInner { mutable: true } => quote!(#ident.as_mut()),
        };
        if !matches!(self.borrow, RebuildBorrow::Outer { .. }) {
            return value;
        }
        crate::jni::trait_impl::build_through_wrappers(&self.arg, value)
            .expect("every layer was checked buildable when the plan was built")
    }

    fn mutable_binding(&self) -> bool {
        matches!(
            self.borrow,
            RebuildBorrow::Outer { mutable: true } | RebuildBorrow::OptionalInner { mutable: true }
        )
    }
}

/// Descend an outer borrow, an `Option`, or the borrow inside an `Option` to the
/// struct a specialized lowering will **rebuild**, keeping each layer's reading
/// so its spelling can be restored.
///
/// One function because it is one rule, and the layers are checked **on the way
/// down**: an erasure sits outside the layer it wraps, so `Box<&S>` classifies
/// as `Ref` and interpreting `kind` before asking would discard the `Box`.
///
/// The only refusal left is a wrapper the adapter cannot **build** — `Cow`, by
/// policy rather than by impossibility (see its `WRAPPER_OPS` recipe). A wrapped
/// spelling that declines here keeps the general converter path, which is
/// correct if less direct.
fn rebuildable_target(arg: &TypeRef) -> Option<(RebuildTarget, &TypeRef)> {
    // A probe per layer: "can this spelling be rebuilt at all", asked before the
    // peel that would hide it. The token is irrelevant — only the `Option` is.
    let buildable = |t: &TypeRef| build_through_erased_wrappers(t, quote!(__probe)).map(|_| ());
    buildable(arg)?;
    let (borrow, t1) = match arg.unwrapped().kind() {
        flat::TypeKind::Ref { mutable, inner, .. } => {
            (RebuildBorrow::Outer { mutable: *mutable }, inner.as_ref())
        }
        _ => (RebuildBorrow::None, arg),
    };
    buildable(t1)?;
    let mut inner = t1.optional_inner().unwrap_or(t1);
    // The chain must also be able to rebuild the crossing's inner spelling.
    buildable(inner)?;
    let borrow = if matches!(borrow, RebuildBorrow::None) {
        match inner.kind() {
            flat::TypeKind::Ref {
                mutable,
                inner: target,
                ..
            } if arg.erased_wrappers().is_empty() => {
                inner = target.as_ref();
                buildable(inner)?;
                RebuildBorrow::OptionalInner { mutable: *mutable }
            }
            _ => borrow,
        }
    } else {
        borrow
    };
    // Only wrappers outside the borrow are kept. Inner wrappers are already
    // part of the registry chain's source policy.
    Some((
        RebuildTarget {
            arg: arg.erased_wrappers(),
            borrow,
        },
        inner,
    ))
}

/// A flattened plan for one struct input parameter. Built once by
/// [`build_flat_input_plan`] and consumed by all three codegen sites.
pub(crate) struct FlatInputPlan {
    pub leaves: Vec<FlatLeaf>,
    /// Registry-composed source converter over those leaves. Planning verifies
    /// its presence and leaf arity; cross-artifact and runtime tests verify that
    /// the ordered leaves have the same meaning on both sides of JNI. There is
    /// no adapter-side reconstruction path.
    pub chain: crate::jni::compile::ComposedChain,
    /// Source identity retained only for the deliberately non-recursive Vec
    /// push helper, which constructs one simple element literal.
    pub struct_module: syn::Path,
    pub struct_ident: syn::Ident,
    /// Whether any Product child is itself composed. The specialized Vec
    /// helper only understands a Product of terminal leaves.
    pub contains_composed_child: bool,
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

pub(crate) fn wire_kotlin_type(entry: &prebindgen_registry::ConverterImpl<KotlinMeta>) -> String {
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

pub(crate) fn build_flat_input_plan(
    ext: &Declarations,
    registry: &impl Conversions,
    param_name: &syn::Ident,
    arg: &TypeRef,
) -> Result<Option<FlatInputPlan>, FlatInputError> {
    // 1. Resolve the struct target through `&` and `Option<…>` — off the model,
    //    keeping each layer's reading so the rebuild can restore its spelling.
    let Some((target, inner)) = rebuildable_target(arg) else {
        return Ok(None);
    };
    // `impl Into<S>` is NOT peeled here, and cannot be: the model refuses
    // `impl Trait` that is not the callback form (`DisallowedImplTrait`), so a
    // parameter spelled that way never becomes a reading and never reaches this
    // function.
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
    // `Payload`'s data-class declaration.
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
    drop(entry);

    // 2. The parameters this crossing occupies, as the recipe states them. What
    //    used to be walked and named here is composed once per crossing, so
    //    the only thing left for the site to say is which parameter each wire
    //    hangs off.
    let Some(wires) = ext.wires_of(arg) else {
        return Ok(None);
    };
    let leaves: Vec<FlatLeaf> = wires.iter().map(|w| FlatLeaf::of(param_name, w)).collect();
    let chain = ext
        .composed_chain(arg, prebindgen_registry::recipe::Direction::Construct)
        .filter(|chain| chain.layout.leaf_count() == leaves.len())
        .ok_or_else(|| {
            flat_error(
                &key,
                &param_name.to_string(),
                "the registry-composed layout arity does not match the declared JNI wires",
            )
        })?;
    chain.activate();
    let contains_composed_child = match &chain.layout {
        crate::jni::compile::JLayout::Product(parts) => {
            parts.iter().any(crate::jni::compile::JLayout::is_composed)
        }
        _ => true,
    };
    Ok(Some(FlatInputPlan {
        leaves,
        chain,
        struct_module: struct_module_path(ext, registry, &st.name),
        struct_ident: st.name.clone(),
        contains_composed_child,
        target,
    }))
}

/// Render native reconstruction for a [`FlatInputPlan`] through its registry
/// chain. Building the Product/Optional/Choice tree is entirely a registry
/// responsibility; this site only supplies the JNI wire leaves.
/// Failures route through `signal_error` and return the function `on_err`
/// sentinel. Returns the prelude and call argument (`arg` or `&arg`).
pub(crate) fn render_flat_input_decode(
    plan: &FlatInputPlan,
    arg_ident: &syn::Ident,
    on_err: &TokenStream,
) -> (TokenStream, TokenStream) {
    let leaves: Vec<syn::Ident> = plan
        .leaves
        .iter()
        .map(|leaf| leaf.native_ident.clone())
        .collect();
    let intermediate = plan.chain.layout.expression(&leaves);
    let converter = &plan.chain.ident;
    let binding = plan.target.mutable_binding().then(|| quote!(mut));
    let prelude = quote! {
        let #binding #arg_ident = match #converter(&mut env, #intermediate) {
            ::core::result::Result::Ok(__value) => __value,
            ::core::result::Result::Err(__error) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__error.to_string(),
                );
                return #on_err;
            }
        };
    };
    (prelude, plan.target.call_arg(arg_ident))
}

// ──────────────────────────────────────────────────────────────────────
// Slice / Vec input → Rust-side Vec handle (built by pushing leaves)
// ──────────────────────────────────────────────────────────────────────
