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
use crate::api::core::registry::Conversions;

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

/// The COMPLETE Rust → wire conversion of one leaf: the rust-side stages a
/// custom [`convert!`](crate::convert) declaration inserts (`Duration → u64`)
/// followed by the wire-facing converter (`u64 → jlong`).
///
/// A leaf must carry the whole chain, not just
/// [`TypeEntry::converter_ident`](crate::core::TypeEntry::converter_ident):
/// calling only the wire-facing function would hand it the *semantic* value
/// (a `Duration`) where it expects the *representation* (a `u64`), which does
/// not compile. Structural wrappers (`Option<_>`, `Vec<_>`) already compose
/// the chain; this is the same composition for the positions the flattened
/// `fromParts` bridge encodes itself.
pub(crate) struct ConvChain {
    /// Rust-side stages in output execution order — each consumes the
    /// previous one's result, the first consumes the Rust value.
    pub stages: Vec<syn::Ident>,
    /// The wire-facing converter, applied last.
    pub function: syn::Ident,
}

impl ConvChain {
    /// Read the chain off a resolved output entry.
    fn of(entry: &crate::core::TypeEntry<KotlinMeta>) -> Self {
        ConvChain {
            stages: entry
                .output_stage_order()
                .map(|(_, stage)| stage.function.sig.ident.clone())
                .collect(),
            function: entry.converter_ident().clone(),
        }
    }

    /// The expression converting `value` (a Rust value expression) to this
    /// leaf's wire form, propagating any stage error with `?`.
    pub(crate) fn call(&self, env: &TokenStream, value: &TokenStream, base: &str) -> TokenStream {
        let function = &self.function;
        if self.stages.is_empty() {
            return quote! { #function(#env, #value.clone())? };
        }
        let mut body = TokenStream::new();
        let mut previous = quote!(#value.clone());
        for (order, stage) in self.stages.iter().enumerate() {
            let next = format_ident!("__{}_s{}", base, order);
            body.extend(quote! {
                let #next = #stage(#env, #previous)
                    .map_err(|__e| <__JniErr as ::core::convert::From<String>>::from(
                        __e.to_string()))?;
            });
            previous = quote!(#next);
        }
        quote!({ #body #function(#env, #previous)? })
    }
}

pub(crate) enum PlanFieldKind {
    /// Projection leaf (opaque handle / `ULong`). Wire slot: `jlong` (`"J"`);
    /// the factory rebuilds the typed value from `fqn`.
    Projection {
        conv: ConvChain,
        proj: Projection,
        fqn: String,
    },
    /// Bare enum → `jint` discriminant (`"I"`); factory calls `fromInt`.
    Enum { conv: ConvChain, kotlin: kt::KtType },
    /// `Option<enum>` → `box_jint`-boxed discriminant
    /// (`"Ljava/lang/Integer;"`, JVM null = `None`); factory takes `Int?`.
    OptionEnum { conv: ConvChain, kotlin: kt::KtType },
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
    /// Data-carrying enum (`sealed_class`): an `Int` **tag** slot naming the
    /// live alternative, followed by one **leaf group per variant** laid side
    /// by side. Exactly one group is live; the rest are wire-defaulted and
    /// the tag tells both sides which to read.
    ///
    /// This is [`Self::Nested`]'s `optional` gating with `N` groups instead
    /// of one and an `Int` tag instead of a `Boolean` flag — a unit-only
    /// enum would degenerate to "just a tag", which is why `enum_class`
    /// keeps its own simpler path.
    Sum {
        /// Path to the source enum, for the encoder's match arms.
        source: syn::Path,
        /// Kotlin FQN of the sealed interface, for the factory's `when`.
        kotlin_fqn: String,
        /// `Option<E>` keeps its own `present` flag ahead of the tag; the tag
        /// domain is never overloaded with an "absent" value.
        optional: bool,
        /// Variants in declaration order; index == tag.
        variants: Vec<SumPlanVariant>,
    },
    /// Simple leaf with its own output converter.
    Leaf {
        conv: ConvChain,
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

/// One alternative of a [`PlanFieldKind::Sum`].
pub(crate) struct SumPlanVariant {
    /// Variant ident as declared in Rust — the encoder's match pattern.
    pub rust_ident: syn::Ident,
    /// Variant class name in Kotlin (after any `variant!(V).name(...)`).
    pub kotlin_name: String,
    /// This variant's payload, in declaration order. Empty for a unit
    /// variant — the group that contributes nothing but its tag.
    pub fields: Vec<SumPlanField>,
}

/// One payload field of a [`SumPlanVariant`]. Classified by exactly the same
/// [`classify_field`] a struct field goes through, so a payload and a struct
/// field of the same Rust type get the same slot, wire and Kotlin type.
pub(crate) struct SumPlanField {
    /// How the field is addressed in the encoder's match pattern.
    pub member: syn::Member,
    /// Slot-name fragment, `<variantCamel>_<prop>` (`exact_v0`). The Kotlin
    /// property name it embeds is recomputed where needed from
    /// [`sum_field_prop_name`], so there is one derivation rather than a
    /// stored copy that could disagree with it.
    pub slot: String,
    pub kind: PlanFieldKind,
}

/// Classify `s`'s fields into the shared bridge plan. `None` aborts the
/// whole-value bridge (an unresolved field converter or a missing Kotlin
/// name) — consistently for BOTH sides, where the former parallel walks
/// could silently diverge on such edge cases.
pub(crate) fn build_struct_plan(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
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
        let owner = format!("{}.{}", s.ident, fname);
        let kind = classify_field(ext, registry, &field.ty, &owner, depth)?;
        fields.push(PlanField { fname, kind });
    }
    Some(StructPlan { fields })
}

/// Classify ONE value position — a struct field or a sum's variant payload —
/// into its bridge slot. Both callers go through here so a payload and a
/// struct field of the same Rust type get the same slot, wire, descriptor and
/// Kotlin type; the alternative (a second classification walk) is exactly the
/// drift `StructPlan` exists to prevent.
///
/// `owner` is the dotted path used in diagnostics (`Config.mode`,
/// `Reading::Exact.v0`).
pub(crate) fn classify_field(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    ty: &syn::Type,
    owner: &str,
    depth: usize,
) -> Option<PlanFieldKind> {
    let effective_ty = ty.clone();

    // A sum is classified FIRST, because it is the one kind with no converter
    // of its own: it crosses as a tag plus one leaf group per variant, never
    // as a single wire, which is why `sealed_class` types are declared
    // boundary-only. Demanding an output entry before this point would send
    // every sum-typed field down the `None` path and fail the whole parent's
    // plan with an unresolved-converter error naming the wrong thing.
    // `Vec` is peeled alongside `Option` here purely to CLASSIFY: `type_kind`
    // answers about a bare ident, so it reports `Vec<Reading>` as `Other` and
    // a rejection guarded on the unpeeled type could never fire. Peeling first
    // is what makes the `Vec<sum>` error reachable at all.
    let bare = option_inner_type(&effective_ty).unwrap_or_else(|| effective_ty.clone());
    let core = vec_inner_type(&bare).unwrap_or_else(|| bare.clone());
    if matches!(ext.type_kind(registry, &core), TypeKind::Sum) {
        // A `Vec` of tag-gated groups has variable arity, exactly like a `Vec`
        // of nested data classes — the flattened bridge is fixed-layout by
        // construction.
        if vec_inner_type(&bare).is_some() {
            panic!(
                "fromParts bridge: `Vec<{}>` sealed-class field (`{owner}`) is not supported \
                 (variable arity)",
                core.to_token_stream(),
            );
        }
        return sum_plan_kind(
            ext,
            registry,
            &bare,
            owner,
            option_inner_type(&effective_ty).is_some(),
            depth,
        );
    }

    let field_entry = registry.output_entry(&effective_ty)?;
    let conv = ConvChain::of(field_entry);

    {
        // Projection leaf (opaque handle / `ULong`).
        if let Some(proj) = field_entry.metadata.projection.clone() {
            if matches!(proj.strategy, FoldStrategy::Iterable(_)) {
                panic!(
                    "fromParts bridge: collection (`Vec<projection>`) field `{owner}` is not \
                     supported — add array codegen to lift this guard"
                );
            }
            let fqn = projection_leaf_kt(ext, &proj)?.to_string();
            return Some(PlanFieldKind::Projection { conv, proj, fqn });
        }
        // Bare enum leaf.
        if ext.is_kotlin_enum(&effective_ty) {
            let kotlin = field_entry.metadata.kotlin_name.clone()?;
            return Some(PlanFieldKind::Enum { conv, kotlin });
        }
        // `Option<enum>` leaf.
        if let Some(inner) = option_inner_type(&effective_ty) {
            if ext.is_kotlin_enum(&inner) {
                let kotlin = registry
                    .output_entry(&inner)?
                    .metadata
                    .kotlin_name
                    .clone()?;
                return Some(PlanFieldKind::OptionEnum { conv, kotlin });
            }
        }
        // Nested plain data-class (optionally under `Option`).
        let inner_ty = bare.clone();
        if let TypeKind::DataStruct { st, cfg } = ext.type_kind(registry, &inner_ty) {
            if pat_match_top(&effective_ty, "Vec") {
                panic!(
                    "fromParts bridge: `Vec<{}>` data-class field (`{owner}`) is not supported \
                     (variable arity)",
                    inner_ty.to_token_stream(),
                );
            }
            let child_fqn = cfg
                .and_then(|c| c.name_spec.as_ref())
                .map(|s| ext.fqn_of(s));
            let plan = build_struct_plan(ext, registry, &st.origin.syntax, depth + 1)?;
            return Some(PlanFieldKind::Nested {
                optional: option_inner_type(&effective_ty).is_some(),
                child_fqn,
                plan,
            });
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
                            ext.kotlin_fqn(&TypeKey::from_ident(&name))
                                .map(|v| format!("L{};", v.replace('.', "/")))
                        })
                    })
                    .or_else(|| {
                        if pat_match_top(&slot_ty, "Vec") {
                            Some("Ljava/util/List;".to_string())
                        } else {
                            // The wire table already names every reference wire's
                            // descriptor — String and the eight primitive arrays.
                            jni_field_access(&wire).map(|(sig, _, _)| sig.to_string())
                        }
                    })
                    .unwrap_or_else(|| "Ljava/lang/Object;".to_string());
                (LeafForm::Object, descriptor)
            }
        };
        let nullable = is_option_type(&effective_ty) && !is_jni_primitive(&wire);
        Some(PlanFieldKind::Leaf {
            conv,
            wire: Box::new(wire),
            form,
            descriptor,
            kotlin,
            nullable,
        })
    }
}

impl PlanFieldKind {
    /// The Kotlin type of the `data class` **constructor property** this field
    /// becomes.
    ///
    /// The class declaration, the `fromParts` factory and the Rust encoder are
    /// three views of one classification, so all three read it from here — the
    /// module docs' "agree by construction instead of by hand-synchronized
    /// parallel walks" applied to the declaration too (#156). Deriving it
    /// separately is what let a property's type disagree with its own
    /// factory parameter.
    ///
    /// `owner` is the dotted path used in diagnostics.
    pub(crate) fn property_type(&self, owner: &str) -> kt::KtType {
        match self {
            // A projection's typed surface is its folded shape over the leaf
            // class (`ZKeyExpr?`, `List<ZKeyExpr>`, `ULong`), which the plan
            // already resolved into `fqn`.
            PlanFieldKind::Projection { proj, fqn, .. } => {
                handle_kt_type(&proj.strategy, &kt::KtType::cls(fqn))
            }
            PlanFieldKind::Enum { kotlin, .. } => kotlin.clone(),
            PlanFieldKind::OptionEnum { kotlin, .. } => kotlin.clone().nullable(),
            PlanFieldKind::Nested {
                optional,
                child_fqn,
                ..
            } => {
                let fqn = child_fqn.as_ref().unwrap_or_else(|| {
                    panic!(
                        "data class property `{owner}`: nested data-class field has no \
                         registered Kotlin class — declare the child type in a package"
                    )
                });
                let t = kt::KtType::cls(fqn);
                if *optional {
                    t.nullable()
                } else {
                    t
                }
            }
            PlanFieldKind::Sum {
                kotlin_fqn,
                optional,
                ..
            } => {
                let t = kt::KtType::cls(kotlin_fqn);
                if *optional {
                    t.nullable()
                } else {
                    t
                }
            }
            // `nullable` is the plan's own rule — an `Option` field whose wire
            // is object-shaped. An `Option` over a PRIMITIVE wire stays
            // non-null, because the encoder passes the bare primitive with a
            // sentinel and the JVM slot must match (`J`, not `Ljava/lang/Long;`).
            PlanFieldKind::Leaf {
                kotlin, nullable, ..
            } => {
                if *nullable {
                    kotlin.clone().nullable()
                } else {
                    kotlin.clone()
                }
            }
        }
    }

    /// The close strategy when this field owns a native handle, so the class
    /// implements `AutoCloseable` and `close()` walks it. Only an **owned**
    /// `Handle` projection qualifies: a `ULong` owns nothing, and a borrowed
    /// handle is not this object's to release.
    pub(crate) fn destructible(&self) -> Option<FoldStrategy> {
        match self {
            PlanFieldKind::Projection { proj, .. }
                if matches!(proj.kind, ProjectionKind::Handle) && proj.owned =>
            {
                Some(proj.strategy.clone())
            }
            _ => None,
        }
    }
}

/// Build the [`PlanFieldKind::Sum`] for a `sealed_class`-declared enum: one
/// leaf group per variant, each payload classified through
/// [`classify_field`].
///
/// Recursion is bounded by this function's own depth guard (see below) — a
/// sum reaching its own type has no `jobject_input`-style escape hatch, since
/// the flatten plan is finite by construction.
///
/// `None` propagates the resolver's **deferral** protocol: a payload whose
/// converter has not resolved *yet* means "retry on the next fixed-point
/// iteration", exactly as [`classify_field`] signals it for a struct field.
/// Panicking there instead would turn a transient state into a build failure
/// whenever a payload's converter happened to resolve later than this plan
/// was first attempted.
fn sum_plan_kind(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    ty: &syn::Type,
    owner: &str,
    optional: bool,
    depth: usize,
) -> Option<PlanFieldKind> {
    use crate::api::core::types_util::SumSpec;

    // Sum expansion needs its OWN depth guard. A sum whose payload is a sum
    // never passes through `build_struct_plan`, so that function's assert —
    // the only one on this recursion before now — cannot see a chain made
    // purely of sums. Rust's sizedness rules make an unindirected cycle
    // impossible to declare, and every indirection either classifies as
    // `Other` (`Box<E>` is not a bare ident) or is already rejected
    // (`Vec<E>`), so this is defence in depth rather than a reachable path
    // today. It costs one comparison and makes the bound true for every
    // future shape instead of true-by-accident.
    assert!(
        depth <= 16,
        "fromParts bridge: sealed-class expansion too deep at `{owner}` (recursive sum?)"
    );
    let ident = bare_path_ident(ty).unwrap_or_else(|| {
        panic!("fromParts bridge: sealed-class field `{owner}` is not a path type")
    });
    let item_enum = registry.flat().enum_item(&ident).unwrap_or_else(|| {
        panic!("fromParts bridge: sealed-class field `{owner}` has no indexed enum `{ident}`")
    });
    let key = TypeKey::from_ident(&ident);
    let cfg = ext
        .types
        .get(&key)
        .unwrap_or_else(|| panic!("fromParts bridge: `{ident}` is not declared"));
    let sum_cfg = cfg
        .sum()
        .unwrap_or_else(|| panic!("fromParts bridge: `{ident}` is not a sealed class"));
    let kotlin_fqn = cfg
        .name_spec
        .as_ref()
        .map(|s| ext.fqn_of(s))
        .unwrap_or_else(|| panic!("fromParts bridge: sealed class `{ident}` has no Kotlin name"));

    let spec = SumSpec::from_item_enum(item_enum);
    let mut variants: Vec<SumPlanVariant> = Vec::new();
    for (v, item_variant) in spec.variants.iter().zip(&item_enum.variants) {
        let kotlin_name = ext.sum_variant_class_name(sum_cfg, &v.ident);
        let mut fields: Vec<SumPlanField> = Vec::new();
        for (f, item_field) in v.fields.iter().zip(item_variant.fields.iter()) {
            let prop = sum_field_prop_name(f);
            let slot = sum_slot_fragment(&kotlin_name, &prop);
            let owner = format!("{ident}::{}.{prop}", v.ident);
            // `?` — a payload whose converter has not resolved yet defers the
            // whole plan to the next iteration, it does not fail the build.
            let kind = classify_field(ext, registry, &item_field.ty, &owner, depth + 1)?;
            fields.push(SumPlanField {
                member: f.member.clone(),
                slot,
                kind,
            });
        }
        variants.push(SumPlanVariant {
            rust_ident: v.ident.clone(),
            kotlin_name,
            fields,
        });
    }

    Some(PlanFieldKind::Sum {
        source: {
            let module = ext.fn_module(registry, &ident);
            syn::parse_quote!(#module::#ident)
        },
        kotlin_fqn,
        optional,
        variants,
    })
}

/// Kotlin property name of one sum payload field — a named field keeps its
/// camelCased name, a tuple field becomes `v0`, `v1`. Must agree with the
/// sealed-interface emitter, which is why both call this.
pub(crate) fn sum_field_prop_name(field: &crate::api::core::types_util::SumField) -> String {
    match &field.member {
        syn::Member::Named(id) => mangle_kotlin_ident(&kt_snake_to_camel(&id.to_string())),
        syn::Member::Unnamed(i) => format!("v{}", i.index),
    }
}

/// Slot-name fragment for one variant field: `<variantCamel>_<prop>`. Keyed
/// on the **Kotlin** variant name so a `variant!(V).name(...)` rename carries
/// through to the slots.
pub(crate) fn sum_slot_fragment(kotlin_variant: &str, prop: &str) -> String {
    let mut chars = kotlin_variant.chars();
    let head: String = match chars.next() {
        Some(c) => c.to_lowercase().collect(),
        None => String::new(),
    };
    format!("{head}{}_{prop}", chars.as_str())
}
