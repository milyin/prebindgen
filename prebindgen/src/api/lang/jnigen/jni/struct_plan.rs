//! The shared recursive leaf plan of the data-class `fromParts` bridge.
//!
//! A whole-value struct crossing Rust→Kotlin is flattened into leaf wire
//! slots: the Rust side encodes them and makes ONE
//! `call_static_method("fromParts", …)`
//! ([`flatten_struct_encode`](super::flatten_struct_encode)); the Kotlin side
//! declares the matching `fromParts` factory that reassembles the object in
//! bytecode ([`flatten_struct_factory`](super::flatten_struct_factory)). Both
//! sides must enumerate the same leaves, in the same order, with matching
//! wire slots and JVM descriptors.
//!
//! This module holds that agreement: [`build_struct_plan`] classifies every
//! field ONCE, in one fixed priority order (projection → enum →
//! `Option<enum>` → nested data-class → simple leaf), and both emitters walk
//! the resulting [`StructPlan`] — so the two sides agree by construction
//! instead of by hand-synchronized parallel walks.

use super::*;

/// The flattened `fromParts` bridge plan of one struct.
pub(crate) struct StructPlan {
    pub fields: Vec<PlanField>,
}

/// One classified field of a [`StructPlan`]. Each side derives its own
/// naming from `fname` (camelCase Kotlin params, snake Rust idents); the
/// classification fixes the wire slot both sides use.
pub(crate) struct PlanField {
    pub fname: syn::Ident,
    pub kind: PlanFieldKind,
}

/// How a Rust-side simple leaf binds its encoded wire into the `JValue` slot.
pub(crate) enum LeafForm {
    /// Primitive wire: bind as the wire type, pass via `JValue::from`.
    Prim,
    /// `JString` / `JByteArray`: bind as `JObject` via `.into()`.
    IntoObject,
    /// Already-`JObject` wire (boxed `Option`, `List`, …): bind directly.
    Object,
}

pub(crate) enum PlanFieldKind {
    /// Opaque-handle / value-blob leaf. Wire slot: `jlong` (`"J"`) for a
    /// handle, `ByteArray` (`"[B"`) for a blob; the factory rebuilds the
    /// typed value from `fqn`.
    Projection {
        conv: syn::Ident,
        proj: Projection,
        fqn: String,
    },
    /// Bare enum → `jint` discriminant (`"I"`); factory calls `fromInt`.
    Enum {
        conv: syn::Ident,
        kotlin: kt::KtType,
    },
    /// `Option<enum>` → `box_jint`-boxed discriminant
    /// (`"Ljava/lang/Integer;"`, JVM null = `None`); factory takes `Int?`.
    OptionEnum {
        conv: syn::Ident,
        kotlin: kt::KtType,
    },
    /// Nested plain data-class: its leaves inline here. `optional` prepends
    /// a `present: Boolean` flag (`"Z"`) and defaults the child slots in the
    /// `None` arm; the factory guards `Child.fromParts(…)` on the flag.
    Nested {
        optional: bool,
        /// The child's registered Kotlin FQN (its `fromParts` owner). `None`
        /// for an undeclared struct: the Rust encode can still inline it,
        /// but the Kotlin factory (which must name the child class) aborts.
        child_fqn: Option<String>,
        plan: StructPlan,
    },
    /// Simple leaf with its own output converter.
    Leaf {
        conv: syn::Ident,
        /// The converter's destination wire type (boxed: `syn::Type` is the
        /// enum's size outlier).
        wire: Box<syn::Type>,
        form: LeafForm,
        /// JVM descriptor of the slot (must match the factory param's type).
        descriptor: String,
        kotlin: kt::KtType,
        /// Kotlin-side `?` (an `Option` field whose wire is object-shaped).
        nullable: bool,
    },
}

/// Classify `s`'s fields into the shared bridge plan. `None` aborts the
/// whole-value bridge (an unresolved field converter or a missing Kotlin
/// name) — consistently for BOTH sides, where the former parallel walks
/// could silently diverge on such edge cases.
pub(crate) fn build_struct_plan(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    s: &syn::ItemStruct,
    depth: usize,
) -> Option<StructPlan> {
    assert!(
        depth <= 16,
        "struct fromParts plan: recursion too deep at struct `{}` (cyclic data_class?)",
        s.ident
    );
    let syn::Fields::Named(named) = &s.fields else {
        return None;
    };
    let mut fields: Vec<PlanField> = Vec::new();
    for field in &named.named {
        let fname = field.ident.as_ref()?.clone();
        let effective_ty = field.ty.clone();
        let field_entry = registry.output_entry(&effective_ty)?;
        let conv = field_entry.function.sig.ident.clone();

        // Projection leaf (opaque handle / value blob).
        if let Some(proj) = field_entry.metadata.projection.clone() {
            if matches!(proj.strategy, FoldStrategy::Iterable(_)) {
                panic!(
                    "fromParts bridge: collection (`Vec<projection>`) field `{}.{}` is not \
                     supported — add array codegen to lift this guard",
                    s.ident, fname
                );
            }
            let fqn = ext.kotlin_fqn(&proj.leaf_key).map(|v| v.to_string())?;
            fields.push(PlanField {
                fname,
                kind: PlanFieldKind::Projection { conv, proj, fqn },
            });
            continue;
        }
        // Bare enum leaf.
        if ext.is_kotlin_enum(&effective_ty) {
            let kotlin = field_entry.metadata.kotlin_name.clone()?;
            fields.push(PlanField {
                fname,
                kind: PlanFieldKind::Enum { conv, kotlin },
            });
            continue;
        }
        // `Option<enum>` leaf.
        if let Some(inner) = option_inner_type(&effective_ty) {
            if ext.is_kotlin_enum(&inner) {
                let kotlin = registry
                    .output_entry(&inner)?
                    .metadata
                    .kotlin_name
                    .clone()?;
                fields.push(PlanField {
                    fname,
                    kind: PlanFieldKind::OptionEnum { conv, kotlin },
                });
                continue;
            }
        }
        // Nested plain data-class (optionally under `Option`).
        let inner_ty = option_inner_type(&effective_ty).unwrap_or_else(|| effective_ty.clone());
        if let TypeKind::DataStruct { st, cfg } = ext.type_kind(registry, &inner_ty) {
            if pat_match_top(&effective_ty, "Vec") {
                panic!(
                    "fromParts bridge: `Vec<{}>` data-class field (`{}.{}`) is not supported \
                     (variable arity)",
                    inner_ty.to_token_stream(),
                    s.ident,
                    fname
                );
            }
            let child_fqn = cfg
                .and_then(|c| c.name_spec.as_ref())
                .map(|s| ext.fqn_of(s));
            let plan = build_struct_plan(ext, registry, &st.clone(), depth + 1)?;
            fields.push(PlanField {
                fname,
                kind: PlanFieldKind::Nested {
                    optional: option_inner_type(&effective_ty).is_some(),
                    child_fqn,
                    plan,
                },
            });
            continue;
        }
        // Simple leaf: derive the slot descriptor and the Rust binding form
        // from the converter's wire — the one place this decision is made.
        let wire = field_entry.destination.clone();
        let kotlin = field_entry.metadata.kotlin_name.clone()?;
        let (form, descriptor) = match jni_field_access(&wire) {
            Some((sig, _, false)) => (LeafForm::Prim, sig.to_string()),
            Some((sig, _, true)) => (LeafForm::IntoObject, sig.to_string()),
            None => {
                // Object-shaped wire with no fixed descriptor; the JVM slot
                // must be the field's actual declared type (Option-stripped).
                let slot_ty =
                    option_inner_type(&effective_ty).unwrap_or_else(|| effective_ty.clone());
                let descriptor = registry
                    .output_entry(&slot_ty)
                    .and_then(|e| jni_field_access(&e.destination))
                    .and_then(|(sig, _, is_obj)| {
                        if is_obj {
                            Some(sig.to_string())
                        } else {
                            // The inner type's own wire is a primitive, so
                            // this field is an `Option<primitive-wire>` whose
                            // converter delivers the `box_j*`-boxed OBJECT
                            // (null for `None`) — the JVM slot is the box
                            // class, not the primitive.
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
                        } else if let syn::Type::Path(tp) = &wire {
                            tp.path.segments.last().and_then(|seg| {
                                match seg.ident.to_string().as_str() {
                                    "JString" => Some("Ljava/lang/String;".to_string()),
                                    "JByteArray" => Some("[B".to_string()),
                                    _ => None,
                                }
                            })
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "Ljava/lang/Object;".to_string());
                (LeafForm::Object, descriptor)
            }
        };
        let nullable = is_option_type(&effective_ty) && !is_jni_primitive(&wire);
        fields.push(PlanField {
            fname,
            kind: PlanFieldKind::Leaf {
                conv,
                wire: Box::new(wire),
                form,
                descriptor,
                kotlin,
                nullable,
            },
        });
    }
    Some(StructPlan { fields })
}
