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

use kotlin_codegen::KtType;
use prebindgen_registry::Conversions;

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

/// The COMPLETE Rust → wire conversion of one leaf: the rust-side stages a
/// custom [`convert!`](prebindgen_registry::convert) declaration inserts (`Duration → u64`)
/// followed by the wire-facing converter (`u64 → jlong`).
///
/// A leaf must carry the whole chain, not just
/// [`converter_ident`](prebindgen_registry::ConverterImpl::converter_ident):
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
    fn of(entry: &prebindgen_registry::ConverterImpl<KotlinMeta>) -> Self {
        ConvChain {
            stages: entry
                .output_stage_order()
                .map(|(_, stage)| stage.converter.clone())
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
    Enum { conv: ConvChain, kotlin: KtType },
    /// `Option<enum>` → primitive discriminant when the enum contributes a
    /// niche, boxed `Integer` otherwise. The frozen Kotlin sentinel is the
    /// same slot the registry-composed converter carved.
    OptionEnum {
        conv: ConvChain,
        kotlin: KtType,
        niche: Option<String>,
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
        kotlin: KtType,
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
    registry: &impl Conversions,
    s: &prebindgen_registry::flat::Struct,
    depth: usize,
) -> Option<StructPlan> {
    assert!(
        depth <= 16,
        "struct fromParts plan: recursion too deep at struct `{}` (cyclic data_class?)",
        s.name
    );
    let mut fields: Vec<PlanField> = Vec::new();
    for field in &s.fields {
        // A tuple struct is an `Extern` in the model, never a `Struct`, so a
        // nameless field cannot reach here.
        let fname = field.name.as_ref()?.clone();
        let owner = format!("{}.{}", s.name, fname);
        let kind = classify_field(ext, registry, &field.ty, &owner, depth)?;
        fields.push(PlanField { fname, kind });
    }
    Some(StructPlan { fields })
}

impl Declarations {
    /// The one whole-value struct layout used by Rust encoding and Kotlin
    /// declaration/factory emission. During resolution it is memoized; after
    /// resolution the lookup is served exclusively by
    /// [`crate::jni::generation::JniGenerationPlan`].
    pub(crate) fn struct_plan(
        &self,
        registry: &impl Conversions,
        s: &prebindgen_registry::flat::Struct,
        depth: usize,
    ) -> Option<std::rc::Rc<StructPlan>> {
        let key = s.type_ref().key();
        if let Some(generation) = &self.generation {
            return generation.struct_plan(&key);
        }
        if let Some(hit) = self.struct_plans.borrow().get(&key) {
            return hit.clone();
        }
        let plan = build_struct_plan(self, registry, s, depth).map(std::rc::Rc::new);
        self.struct_plans.borrow_mut().insert(key, plan.clone());
        plan
    }
}

/// True when this plan's flattened `fromParts` takes at least one **raw
/// native pointer** leaf — an opaque-handle projection, whose wire slot is a
/// bare `jlong` the factory mints a handle from.
///
/// This is what decides whether that factory needs the raw-pointer guard.
/// Marking every `fromParts` was over-broad: a pointer-free factory such as
/// `Timestamp.fromParts(Long, ByteArray)` cannot forge anything, and guarding
/// it only removed a safe factory from Java and forced unrelated consumers to
/// opt into a contract it does not have.
///
/// Recursive for the same reason the factory is: a nested data class inlines
/// its leaves into the parent's signature, so a handle two levels down still
/// arrives as a raw `Long` parameter of the *parent's* `fromParts`.
pub(crate) fn plan_mints_handle(plan: &StructPlan) -> bool {
    plan.fields.iter().any(|f| kind_mints_handle(&f.kind))
}

fn kind_mints_handle(kind: &PlanFieldKind) -> bool {
    match kind {
        PlanFieldKind::Projection { proj, .. } => proj.kind == ProjectionKind::Handle,
        PlanFieldKind::Nested { plan, .. } => plan_mints_handle(plan),
        PlanFieldKind::Sum { variants, .. } => variants
            .iter()
            .flat_map(|v| &v.fields)
            .any(|f| kind_mints_handle(&f.kind)),
        // `ULong` is a value, not a pointer; the rest are enums and plain
        // leaves that carry no address.
        PlanFieldKind::Enum { .. }
        | PlanFieldKind::OptionEnum { .. }
        | PlanFieldKind::Leaf { .. } => false,
    }
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
    registry: &impl Conversions,
    reading: &prebindgen_registry::flat::TypeRef,
    owner: &str,
    depth: usize,
) -> Option<PlanFieldKind> {
    // The **reading**, not a spelling. Every layer question below is answered
    // from `kind` and cannot fail: holding a `TypeRef` is proof the model
    // classified this type. Taking a `syn::Type` meant asking the registry per
    // question, and a type it had never seen answered "no layer" rather than
    // saying so — which is the missing `?` of #273 waiting to happen again.

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
    // Every layer question below is the MODEL's, asked once: a field spelled
    // `Box<Option<T>>` is `Optional` and must classify, nest and render exactly
    // as `Option<T>` does. Peeling by path segment answered "not optional" for
    // it, and the seven peels in this function would then disagree with each
    // other about the same field (#273).
    let optional_inner = reading.optional_inner();
    let bare_ref = optional_inner.unwrap_or(reading);
    let seq_elem = bare_ref.sequence_elem();
    let core = seq_elem.unwrap_or(bare_ref);
    if matches!(ext.type_kind(registry, &core.key()), TypeKind::Sum) {
        // A `Vec` of tag-gated groups has variable arity, exactly like a `Vec`
        // of nested data classes — the flattened bridge is fixed-layout by
        // construction.
        if seq_elem.is_some() {
            panic!(
                "fromParts bridge: `Vec<{}>` sealed-class field (`{owner}`) is not supported \
                 (variable arity)",
                core,
            );
        }
        return sum_plan_kind(
            ext,
            registry,
            bare_ref,
            owner,
            optional_inner.is_some(),
            depth,
        );
    }

    let field_entry = ext.out_frag(reading)?;
    field_entry.activate();
    let conv = ConvChain::of(&field_entry);

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
        // Enum leaf, bare or under `Option` — asked ONCE, of the model, and of
        // the already-peeled reading beside us.
        //
        // It used to ask `is_kotlin_enum` twice, of two spellings. That answers
        // about the WRAPPER: `builder.rs` documents `Box<Priority>` as `false`
        // for it and `true` for the reading form, so a wrapped enum field fell
        // through to the plain-leaf arm and rendered as its wire instead of the
        // Kotlin enum class — the #273 family, output-side. `flat_input.rs` had
        // already moved to the reading; this is the other half of the same
        // question finally giving the same answer.
        //
        // Optionality stays the CALLER's fact rather than the probe's:
        // `enum_probe` peels `Option` as well as borrows, so asking it about
        // the unpeeled reading would make `Priority` and `Option<Priority>`
        // indistinguishable and collapse the two arms into one.
        if ext.is_kotlin_enum_reading(bare_ref) {
            return match optional_inner {
                None => {
                    let kotlin = field_entry.metadata.kotlin_name.clone()?;
                    Some(PlanFieldKind::Enum { conv, kotlin })
                }
                Some(inner) => {
                    let kotlin = ext.out_frag(inner)?.metadata.kotlin_name.clone()?;
                    let niche = crate::jni::compile::option_enum_niche(
                        ext,
                        reading,
                        prebindgen_registry::recipe::Direction::Deconstruct,
                    );
                    Some(PlanFieldKind::OptionEnum {
                        conv,
                        kotlin,
                        niche,
                    })
                }
            };
        }
        // Nested plain data-class (optionally under `Option`).
        //
        // A `Vec<data class>` does NOT arrive here and needs no guard of its
        // own (#217). `type_kind` answers `DataStruct` only for a key that is a
        // single identifier, which a `Vec<_>` key never is — so this branch
        // cannot be entered with a sequence in hand, and the field falls
        // through to the simple-leaf arm below. That is the right answer rather
        // than a missed one: the field stays ONE slot whose own converter is
        // the element's fixed folder, so the bridge keeps its fixed slot count
        // and the elements still cross as raw leaves. #217 expected this to
        // need array codegen — a count slot plus a per-element sub-plan in all
        // three producers — and it does not, because the sequence never has to
        // enter the fixed layout at all.
        //
        // The `Vec<sum>` refusal above is a different question and stays: a sum
        // has no converter of its own, so there is no single slot to fall
        // through to.
        let inner_ty = bare_ref;
        if let TypeKind::DataStruct { st, cfg } = ext.type_kind(registry, &inner_ty.key()) {
            let child_fqn = cfg
                .and_then(|c| c.name_spec.as_ref())
                .map(|s| ext.fqn_of(s));
            let plan = build_struct_plan(ext, registry, st, depth + 1)?;
            return Some(PlanFieldKind::Nested {
                optional: optional_inner.is_some(),
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
                // Option-stripped off the MODEL: `optional_inner` is the
                // layer's own reading, so there is nothing to re-look-up.
                let slot = optional_inner.unwrap_or(reading);
                let descriptor = ext
                    .out_frag(slot)
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
                        // The NAME off the classification, not off the last
                        // path segment: `Box<T>` IS `T` here, and taking the
                        // spelling apart would answer about the wrapper.
                        match slot.unwrapped().kind() {
                            prebindgen_registry::flat::TypeKind::Named { id, .. } => id.ident(),
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
                        if slot.sequence_elem().is_some() {
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
        let nullable = optional_inner.is_some() && !is_jni_primitive(&wire);
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
    pub(crate) fn property_type(&self, owner: &str) -> KtType {
        match self {
            // A projection's typed surface is its folded shape over the leaf
            // class (`ZKeyExpr?`, `List<ZKeyExpr>`, `ULong`), which the plan
            // already resolved into `fqn`.
            PlanFieldKind::Projection { proj, fqn, .. } => {
                handle_kt_type(&proj.strategy, &KtType::cls(fqn))
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
                let t = KtType::cls(fqn);
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
                let t = KtType::cls(kotlin_fqn);
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

    /// The close strategy when this field **reaches** an owned native handle,
    /// so the class implements `AutoCloseable` and `close()` walks it.
    ///
    /// Only an **owned** `Handle` projection is a handle: a `ULong` owns
    /// nothing, and a borrowed handle is not this object's to release. But
    /// reaching one is not the same as *being* one — a handle held inside a sum
    /// payload or a nested data class is just as much this object's to release,
    /// and used to fall through to `None` (#218). That made ownership depend on
    /// how the field happened to be spelled: swapping a handle field for an
    /// enum carrying that handle silently moved the free onto the consumer,
    /// with nothing in the generated Kotlin saying so.
    ///
    /// The wrapped cases fold as `Base`/`Optional` over the field itself rather
    /// than over the handle inside it, because the field's own generated type
    /// is `AutoCloseable` too — a sum's `close()` is the `when` over its
    /// alternatives (`Declarations::build_sealed_class`), a nested data
    /// class's is this same cascade one level down. So every
    /// container emits the same plain `field.close()`, and the walk into the
    /// wrapper lives once, in the wrapped type, instead of at each use site.
    pub(crate) fn destructible(&self) -> Option<FoldStrategy> {
        match self {
            PlanFieldKind::Projection { proj, .. }
                if matches!(proj.kind, ProjectionKind::Handle) && proj.owned =>
            {
                Some(proj.strategy.clone())
            }
            // `sequence: false` is `classify_field`'s guarantee, not an
            // assumption: it rejects `Vec<sum>` and `Vec<data class>` outright,
            // so a classified field of either kind is never a sequence.
            PlanFieldKind::Sum {
                optional, variants, ..
            } if variants.iter().any(SumPlanVariant::destructible) => {
                Some(whole_value_close(*optional, false))
            }
            PlanFieldKind::Nested { optional, plan, .. } if plan.destructible() => {
                Some(whole_value_close(*optional, false))
            }
            _ => None,
        }
    }
}

impl StructPlan {
    /// Whether closing this struct has anything to do — any field reaching an
    /// owned handle. The recursion is [`PlanFieldKind::destructible`]'s, and is
    /// bounded by the same depth guards that bound plan construction.
    pub(crate) fn destructible(&self) -> bool {
        self.fields.iter().any(|f| f.kind.destructible().is_some())
    }
}

impl SumPlanVariant {
    /// Whether this alternative's payload reaches an owned handle — so its
    /// generated variant class needs a `close()` body rather than a no-op one.
    pub(crate) fn destructible(&self) -> bool {
        self.fields.iter().any(|f| f.kind.destructible().is_some())
    }
}

/// The fold of a wrapper that closes itself: the value is closed as a whole,
/// `?.`-guarded when it is optional and `forEach`-ed when it is a sequence.
/// Shared by the two forms of the reaches-a-handle question below so they
/// cannot answer differently.
///
/// [`NullableKind::Boxed`] is not a guess here. The receiver is always a
/// generated Kotlin *reference* — a sum or a data class — whose absent form is
/// a JVM null; a niche encoding is a wire fact of a handle projection, and
/// those come back carrying their own strategy without passing through this.
fn whole_value_close(optional: bool, sequence: bool) -> FoldStrategy {
    let mut fold = FoldStrategy::Base;
    if sequence {
        fold = FoldStrategy::Iterable(Box::new(fold));
    }
    if optional {
        fold = FoldStrategy::Optional(NullableKind::Boxed, Box::new(fold));
    }
    fold
}

/// How to close a value of this type, or `None` when it **reaches** no owned
/// native handle and so has nothing to release —
/// [`PlanFieldKind::destructible`]'s question asked of a type rather than of an
/// already-classified plan field.
///
/// Two callers hold a [`TypeRef`](prebindgen_registry::flat::TypeRef) and no
/// plan: the sealed-interface emitter, deciding whether a sum is
/// `AutoCloseable` and what each variant class's `close()` body does; and the
/// callback interface builder, deciding whether a reassembled whole value is
/// the proxy's to close after `run`.
///
/// The two forms must **agree wherever both answer**, and that is a tested
/// invariant, not a structural one: `a_types_close_answer_matches_its_plans`
/// asserts `type_close_strategy(ty).is_some() == plan(ty).destructible()` over
/// every field of every declared shape in a set covering all the ways one can
/// reach a handle. It is worth pinning because the walks are not identical.
/// Two places they differ:
///
/// * **Order.** [`classify_field`] classifies a `Sum` *before* it consults
///   `output_entry` (deliberately — see its comment); this asks `output_entry`
///   first. Safe only while sums carry no converter of their own, which is a
///   precondition rather than a construction.
/// * **Totality.** A field [`classify_field`] refuses collapses the whole
///   struct's plan to `None`, while this refuses nothing. So on a subtree the
///   bridge rejects, the two answer differently *by design* — the plan
///   builders are what diagnose those, with the path, and "is there anything
///   to close" still has a defensible answer for every type.
///
/// A disagreement costs a Kotlin compile error in one direction and a silent
/// leak — #218 again, at this seam — in the other.
///
/// `depth` bounds the same recursion `build_struct_plan` and `sum_plan_kind`
/// bound, and asserts on the same bound rather than answering: a cycle deep
/// enough to trip this is a declaration the plan builders already refuse
/// loudly, and returning `None` for it would report "nothing to close" — the
/// leak direction — for a shape nobody can compile anyway.
pub(crate) fn type_close_strategy(
    ext: &Declarations,
    registry: &impl Conversions,
    ty: &prebindgen_registry::flat::TypeRef,
    depth: usize,
) -> Option<FoldStrategy> {
    assert!(
        depth <= 16,
        "close-strategy walk: recursion too deep at type `{}` (cyclic data_class?)",
        ty
    );
    // An owned `Handle` projection is the one thing that actually owns
    // something: a `ULong` owns nothing, and a borrowed handle is not ours to
    // release. Asked of the whole reading, so the `Option`/`Vec` folds the
    // projection carries come back in its own strategy.
    if let Some(proj) = ext.out_frag(ty).and_then(|e| e.metadata.projection.clone()) {
        return (matches!(proj.kind, ProjectionKind::Handle) && proj.owned)
            .then(|| proj.strategy.clone());
    }
    // Peel the layers the model names, exactly as `classify_field` does: what
    // a `Box<Option<T>>` reaches is what `T` reaches.
    let bare = ty.optional_inner().unwrap_or(ty);
    let core = bare.sequence_elem().unwrap_or(bare);
    let reaches = match ext.type_kind(registry, &core.key()) {
        TypeKind::Sum => core
            .key()
            .ident()
            .and_then(|ident| registry.flat().declared_type(&ident))
            .is_some_and(|ty| match ty {
                prebindgen_registry::flat::Type::Variant(sum) => {
                    sum.alternatives.iter().any(|alt| {
                        alt.fields
                            .iter()
                            .any(|f| type_close_strategy(ext, registry, &f.ty, depth + 1).is_some())
                    })
                }
                _ => false,
            }),
        TypeKind::DataStruct { st, .. } => st
            .fields
            .iter()
            .any(|f| type_close_strategy(ext, registry, &f.ty, depth + 1).is_some()),
        TypeKind::Handle | TypeKind::Enum | TypeKind::Other => false,
    };
    // Put back exactly the layers peeled above. `reaches` was answered about
    // the ELEMENT, so a `Vec<sum-that-reaches-a-handle>` must close each
    // element — `close()` on the `List` itself would not compile. The field
    // bridge rejects that shape, but this predicate exists precisely for the
    // callers that hold no plan and so meet no rejection.
    reaches.then(|| {
        whole_value_close(
            ty.optional_inner().is_some(),
            bare.sequence_elem().is_some(),
        )
    })
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
    registry: &impl Conversions,
    ty: &prebindgen_registry::flat::TypeRef,
    owner: &str,
    optional: bool,
    depth: usize,
) -> Option<PlanFieldKind> {
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
    // The key's ident: a sum is a declared name, and the key is that name when
    // the key is one identifier.
    let ident = ty.key().ident().unwrap_or_else(|| {
        panic!("fromParts bridge: sealed-class field `{owner}` is not a path type")
    });
    // The sum as the MODEL holds it: its alternatives' payloads are `TypeRef`s
    // already, so classifying one asks nothing and cannot be asked about a type
    // the model never saw. One lookup, not two — the `enum_item` that used to
    // sit beside this only fed a `SumSpec` of what the element already says.
    let Some(prebindgen_registry::flat::Type::Variant(sum)) = registry.flat().declared_type(&ident)
    else {
        panic!("fromParts bridge: sealed-class field `{owner}`: `{ident}` is not an indexed sum")
    };
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

    let mut variants: Vec<SumPlanVariant> = Vec::new();
    for alt in &sum.alternatives {
        let kotlin_name = ext.sum_variant_class_name(sum_cfg, &alt.name);
        let mut fields: Vec<SumPlanField> = Vec::new();
        for field in &alt.fields {
            let member = field.member();
            let prop = sum_field_prop_name(&member);
            let slot = sum_slot_fragment(&kotlin_name, &prop);
            let owner = format!("{ident}::{}.{prop}", alt.name);
            // `?` — a payload whose converter has not resolved yet defers the
            // whole plan to the next iteration, it does not fail the build.
            let kind = classify_field(ext, registry, &field.ty, &owner, depth + 1)?;
            fields.push(SumPlanField { member, slot, kind });
        }
        variants.push(SumPlanVariant {
            rust_ident: alt.name.clone(),
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
///
/// Takes the **member**, which is the whole of what the name depends on: every
/// caller holds a `flat::Field` and asks `Field::member()`. It took a
/// `types_util::SumField` when a second description of a sum still existed
/// beside the model's (#289).
pub(crate) fn sum_field_prop_name(member: &syn::Member) -> String {
    match member {
        syn::Member::Named(id) => mangle_kotlin_ident(&kt_snake_to_camel(&id.to_string())),
        syn::Member::Unnamed(i) => format!("v{}", i.index),
    }
}

/// The wire tag of one alternative: its declaration-order index, as the `jint`
/// the selector leaf carries.
///
/// One place, because the tag has to agree in three: the leaf's `group`, the
/// Kotlin `when` arm, and the Rust `match` arm. Three separate `as i32` casts
/// agreed by coincidence rather than by construction.
///
/// Deliberately **not** a checked conversion. `usize` → `i32` can truncate in
/// general, but not here: the index counts alternatives of one enum, and an
/// enum with `i32::MAX` variants is not a thing rustc can be handed. A
/// `try_from(..).expect(..)` would put a panic in the working path for a state
/// the compiler cannot produce, which is the shape this crate avoids.
pub(crate) fn sum_tag(alt: &prebindgen_registry::flat::Alternative) -> i32 {
    alt.index as i32
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
