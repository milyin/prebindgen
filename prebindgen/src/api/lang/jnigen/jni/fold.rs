//! Projection / `FoldStrategy` folding helpers and Kotlin type-shape
//! probes for the JNI back-end's Kotlin emitter.
//!
//! Carved from the former `jni_kotlin_ext.rs`; shares the `jni` namespace
//! via `use super::*`.

use super::*;

/// Peel a leading `&`/`&mut` and an `Option<…>` layer to expose the inner type
/// used for enum detection. So `&Priority`, `Priority`, and `Option<Priority>`
/// all probe as `Priority` — letting nullable enum params (`Option<enum>`) wire
/// as `Int?` + `?.value` just like a non-null enum wires as `Int` + `.value`,
/// instead of leaking the enum object to the (boxed-int-expecting) Rust converter.
pub(crate) fn enum_probe_type(ty: &syn::Type) -> syn::Type {
    let stripped = match ty {
        syn::Type::Reference(r) => (*r.elem).clone(),
        other => other.clone(),
    };
    match crate::api::lang::jnigen::jni::option_inner_type(&stripped) {
        Some(inner) => match inner {
            syn::Type::Reference(r) => (*r.elem).clone(),
            other => other,
        },
        None => stripped,
    }
}

// The bottom-up layer fold is the shared `crate::api::core::shape::fold_shape`
// (its `on_optional` receives the layer's `&NullableKind` + the wrapped
// `&FoldStrategy`, so callers can special-case e.g. a `Niche` layer sitting
// directly over the `Base` leaf). Used by the **type-name** folds
// (`handle_kt_type` / `projection_wire_return`). The **expression** folds
// (`render_handle_close` / `fold_projection_wrap`) are deliberately *not*
// expressed through it: they fold the other direction (threading a `receiver` /
// fresh lambda variable top-down rather than combining a bottom-up result), so
// a shared combinator would obscure rather than simplify them.
use crate::api::core::shape::fold_shape;

/// The Kotlin type for a closeable handle reached through the folded
/// [`FoldStrategy`] layers, given the leaf typed-handle type (e.g.
/// `ZKeyExpr`): `Direct → ZKeyExpr`, `Nullable(inner) → <inner>?`,
/// `Iterable(inner) → List<<inner>>`.
pub(crate) fn handle_kt_type(strategy: &FoldStrategy, leaf: &kt::KtType) -> kt::KtType {
    fold_shape(
        strategy,
        &|| leaf.clone(),
        // The declared Kotlin projection type is `T?` regardless of how null
        // is represented over the wire — the wrap fold and the wire-return
        // helper read the kind to handle the wire shape separately.
        &|inner, _kind, _inner_strategy| inner.nullable(),
        &|inner| kt::KtType::generic("List", [inner]),
    )
}

/// Typed Kotlin leaf of a projection. Declared handle/value-blob projections
/// take their configured class FQN; the built-in `u64` projection is Kotlin's
/// stable unsigned scalar type.
pub(crate) fn projection_leaf_kt(ext: &JniGen, proj: &Projection) -> Option<kt::KtType> {
    match proj.kind {
        ProjectionKind::Handle | ProjectionKind::ValueBlob => {
            ext.kotlin_fqn(&proj.leaf_key).map(kt::KtType::cls)
        }
        ProjectionKind::Unsigned64 => Some(kt::KtType::cls("ULong")),
    }
}

/// Wrap one raw projection leaf into its typed Kotlin form.
pub(crate) fn projection_wrap_expr(kind: &ProjectionKind, short: &str, raw: &str) -> String {
    match kind {
        ProjectionKind::Handle | ProjectionKind::ValueBlob => format!("{short}({raw})"),
        ProjectionKind::Unsigned64 => format!("{raw}.toULong()"),
    }
}

/// For a projection (handle / value-class / value-blob) **struct field**,
/// compute the `(wire_param_type, wrap_expr)` the data class's `fromParts`
/// factory uses: the wire param type matches the leaf wire
/// `struct_output_body` passes (handle → `Long` jlong sentinel, value class /
/// blob → `ByteArray`), and the wrap reconstructs the typed value in JVM
/// bytecode (`Short(arg)`, with null mapped from the `0L` sentinel for handles
/// or JVM null for value classes). Only the `Direct` and `Nullable{Direct}`
/// shapes a scalar projection field can take are supported — a collection
/// (`Vec<projection>`) field is rejected (matching the struct bridge's
/// scalar-only guard).
pub(crate) fn factory_projection_wire_wrap(
    kind: &crate::api::lang::jnigen::jni::ProjectionKind,
    strategy: &crate::api::lang::jnigen::jni::FoldStrategy,
    short: &str,
    name: &str,
) -> (kt::KtType, String) {
    use crate::api::{core::shape::Shape::*, lang::jnigen::jni::ProjectionKind::*};
    let direct = |kind: &crate::api::lang::jnigen::jni::ProjectionKind| match kind {
        Handle => (kt::KtType::long(), format!("{short}({name})")),
        ValueBlob => (kt::KtType::byte_array(), format!("{short}({name})")),
        Unsigned64 => (kt::KtType::long(), format!("{name}.toULong()")),
    };
    match strategy {
        Base => direct(kind),
        Optional(_, inner) => {
            if !matches!(**inner, Base) {
                panic!(
                    "factory_projection_wire_wrap: only `Nullable<Direct>` projection struct \
                     fields are supported (field `{name}`)"
                );
            }
            match kind {
                // Handle null rides the `0L` jlong sentinel.
                Handle => (
                    kt::KtType::long(),
                    format!("if ({name} == 0L) null else {short}({name})"),
                ),
                // Value-blob null rides JVM-null of the `ByteArray` slot.
                ValueBlob => (
                    kt::KtType::byte_array().nullable(),
                    format!("{name}?.let {{ {short}(it) }}"),
                ),
                // `Option<u64>` has no spare bit pattern, so its primitive
                // wire is boxed and JVM null represents `None`.
                Unsigned64 => (kt::KtType::long().nullable(), format!("{name}?.toULong()")),
            }
        }
        Iterable(_) => panic!(
            "factory_projection_wire_wrap: collection (`Vec<projection>`) struct fields are not \
             supported by the fromParts factory (field `{name}`)"
        ),
    }
}

/// True for the Kotlin types that map to JVM **primitives** (never null over
/// the JNI boundary). Used to decide which flattened `Option<nested>` leaf
/// params must be made nullable in the parent factory signature.
pub(crate) fn is_kotlin_primitive_ty(t: &kt::KtType) -> bool {
    !t.is_nullable()
        && t.leaf_name().is_some_and(|n| {
            matches!(
                n,
                "Long" | "Int" | "Boolean" | "Double" | "Float" | "Byte" | "Short" | "Char"
            )
        })
}

/// Recursively build the Kotlin `fromParts` factory for a data class — the
/// mirror of the native `flatten_struct_encode` (in the [`jni`](super)
/// module). Both walk the same [`build_struct_plan`], so the leaf order and
/// slot types agree by construction.
/// Returns `(params, reconstruct)`:
/// * `params` — the flattened `(name, kotlin_type)` list (one per transitive
///   leaf wire; nested data-class fields are inlined, `Option<nested>` prepends
///   a `…__present: Boolean` flag). Order/types match the native call's JVM
///   descriptor positionally.
/// * `reconstruct` — the Kotlin expression building this struct:
///   `Class(<part per constructor field>)`, where a nested field reconstructs
///   via `Child.fromParts(<child param names>)` (`if (present) … else null` when
///   optional) and a leaf reconstructs with its wrap.
#[allow(clippy::too_many_arguments)]
pub(crate) fn flatten_struct_factory(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    s: &syn::ItemStruct,
    prefix: &str,
    class_name: &str,
    imports: &mut BTreeSet<String>,
    depth: usize,
) -> Option<(Vec<(String, kt::KtType)>, String)> {
    let plan = build_struct_plan(ext, registry, s, depth)?;
    factory_from_plan(&plan, prefix, class_name, imports)
}

/// Walk a [`StructPlan`] emitting the Kotlin `fromParts` side: the flattened
/// factory params and the reconstruct expression.
fn factory_from_plan(
    plan: &StructPlan,
    prefix: &str,
    class_name: &str,
    imports: &mut BTreeSet<String>,
) -> Option<(Vec<(String, kt::KtType)>, String)> {
    let mut params: Vec<(String, kt::KtType)> = Vec::new();
    let mut parts: Vec<String> = Vec::new();

    for f in &plan.fields {
        let camel = mangle_kotlin_ident(&kt_snake_to_camel(&f.fname.to_string()));
        let base = if prefix.is_empty() {
            camel.clone()
        } else {
            format!("{prefix}_{camel}")
        };
        match &f.kind {
            // Projection leaf (handle / value class / blob).
            PlanFieldKind::Projection { proj, fqn, .. } => {
                let short = register_fqn(fqn, imports);
                let (wire_ty, wrap) =
                    factory_projection_wire_wrap(&proj.kind, &proj.strategy, &short, &base);
                params.push((base.clone(), wire_ty));
                parts.push(wrap);
            }
            // Enum leaf → `Int`, rebuilt via `Enum.fromInt(i)` (raw text: the
            // enum's short name, its FQN collected as a body import).
            PlanFieldKind::Enum { kotlin, .. } => {
                let short = register_fqn(kotlin.leaf_name()?, imports);
                params.push((base.clone(), kt::KtType::int()));
                parts.push(format!("{short}.fromInt({base})"));
            }
            // `Option<enum>` leaf: the native encoder delivers the discriminant
            // `box_jint`-boxed (JVM `Ljava/lang/Integer;`, null for `None`), so
            // the factory takes `Int?` and rebuilds the nullable enum.
            PlanFieldKind::OptionEnum { kotlin, .. } => {
                let short = register_fqn(kotlin.leaf_name()?, imports);
                params.push((base.clone(), kt::KtType::int().nullable()));
                parts.push(format!("{base}?.let {{ {short}.fromInt(it) }}"));
            }
            // Nested data-class field — inline its leaves and reconstruct via
            // the child's own `fromParts` (in bytecode, no JNI crossing).
            PlanFieldKind::Nested {
                optional,
                child_fqn,
                plan: child,
            } => {
                let child_fqn = child_fqn.as_ref()?;
                let child_short = register_fqn(child_fqn, imports);
                let (child_params, _child_reconstruct) =
                    factory_from_plan(child, &base, &child_short, imports)?;
                let child_names = child_params
                    .iter()
                    .map(|(n, _)| n.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                if !*optional {
                    params.extend(child_params);
                    parts.push(format!("{child_short}.fromParts({child_names})"));
                } else {
                    // `Option<nested>`: the parent receives default-null object wires
                    // for the child's leaves when absent (the native `None` arm), so
                    // every object-typed child param must be NULLABLE in the parent
                    // signature. Inside the `if (present)` guard the values are
                    // non-null again, so forward them to the child's (non-null)
                    // `fromParts` with `!!`. Primitive params (Long/Int/Boolean)
                    // can't be null and are forwarded as-is; already-nullable params
                    // stay nullable.
                    let flag = format!("{base}__present");
                    let mut fwd_names: Vec<String> = Vec::with_capacity(child_params.len());
                    params.push((flag.clone(), kt::KtType::boolean()));
                    for (n, t) in &child_params {
                        if is_kotlin_primitive_ty(t) || t.is_nullable() {
                            params.push((n.clone(), t.clone()));
                            fwd_names.push(n.clone());
                        } else {
                            params.push((n.clone(), t.clone().nullable()));
                            fwd_names.push(format!("{n}!!"));
                        }
                    }
                    parts.push(format!(
                        "if ({flag}) {child_short}.fromParts({}) else null",
                        fwd_names.join(", ")
                    ));
                }
            }
            // Leaf primitive / object (string, byte array, Vec, …) — forwarded
            // unchanged to the constructor. Full-FQN param type; the
            // render-time `ImportSet` shortens it.
            PlanFieldKind::Leaf {
                kotlin, nullable, ..
            } => {
                let ty = if *nullable {
                    kotlin.clone().nullable()
                } else {
                    kotlin.clone()
                };
                params.push((base.clone(), ty));
                parts.push(base);
            }
        }
    }

    let reconstruct = format!("{class_name}({})", parts.join(", "));
    Some((params, reconstruct))
}

/// Render the Kotlin `close()` expression for a handle `receiver` through
/// the folded [`FoldStrategy`] layers. Fresh lambda variable per nesting
/// level avoids `it` shadowing; the common single-layer cases are
/// special-cased for readable output (`x?.close()`, `x.forEach { it.close() }`).
pub(crate) fn render_handle_close(
    strategy: &crate::api::lang::jnigen::jni::FoldStrategy,
    receiver: &str,
) -> String {
    use crate::api::core::shape::Shape::*;
    fn go(
        strategy: &crate::api::lang::jnigen::jni::FoldStrategy,
        receiver: &str,
        depth: usize,
    ) -> String {
        match strategy {
            Base => format!("{receiver}.close()"),
            // The Kotlin-side receiver is already nullable (`handle_kt_type`
            // emits `T?` for both niche and boxed kinds), so `?.close()` covers
            // both wire representations.
            Optional(_, inner) => match &**inner {
                Base => format!("{receiver}?.close()"),
                _ => {
                    let v = format!("e{depth}");
                    format!("{receiver}?.let {{ {v} -> {} }}", go(inner, &v, depth + 1))
                }
            },
            Iterable(inner) => {
                let v = format!("e{depth}");
                format!(
                    "{receiver}.forEach {{ {v} -> {} }}",
                    go(inner, &v, depth + 1)
                )
            }
        }
    }
    go(strategy, receiver, 0)
}

/// Fold the projection wrap call `W(receiver)` through the
/// [`FoldStrategy`] layers:
/// * `Direct`         → `W(x)`
/// * `Nullable{Boxed}` → `x?.let { W(it) }` (JVM-null at the wire)
/// * `Nullable{Niche}` over a primitive wire (e.g. `jlong`) →
///   `x.let { if (it == <sentinel>) null else W(it) }`
/// * `Nullable{Niche}` over an object wire (e.g. `JByteArray`) →
///   `x?.let { W(it) }` (the wire is already a nullable reference)
/// * `Iterable`       → `x.map { W(it) }`
///
/// `niche_sentinel` is the Kotlin literal to compare against for the
/// `Niche+primitive` arm (e.g. `"0L"` for `jlong`-wired handles). When the
/// wire is object-shaped the sentinel is unused — `null` is the wire-level
/// representation and `?.let` is a no-cost null check.
pub(crate) fn fold_projection_wrap(
    strategy: &crate::api::lang::jnigen::jni::FoldStrategy,
    receiver: &str,
    kind: &crate::api::lang::jnigen::jni::ProjectionKind,
    wrap_class: &str,
    niche_sentinel: Option<&str>,
) -> String {
    use crate::api::{core::shape::Shape::*, lang::jnigen::jni::NullableKind};
    fn go(
        s: &crate::api::lang::jnigen::jni::FoldStrategy,
        r: &str,
        kind: &crate::api::lang::jnigen::jni::ProjectionKind,
        w: &str,
        sentinel: Option<&str>,
        depth: usize,
    ) -> String {
        match s {
            Base => projection_wrap_expr(kind, w, r),
            Optional(nullable_kind, inner) => match (nullable_kind, &**inner) {
                // Primitive-wired niche → can't carry null on the wire, so
                // compare against the sentinel and synthesize null on the
                // Kotlin side.
                (NullableKind::Niche, Base) if sentinel.is_some() => {
                    let s = sentinel.unwrap();
                    let wrapped = projection_wrap_expr(kind, w, "it");
                    format!("{r}.let {{ if (it == {s}) null else {wrapped} }}")
                }
                // Object-wired niche or fully boxed Nullable → `?.let { W(it) }`.
                (_, Base) => {
                    let wrapped = projection_wrap_expr(kind, w, "it");
                    format!("{r}?.let {{ {wrapped} }}")
                }
                // Deeper nesting. The niche/boxed distinction is only
                // observable at the outermost layer covering a `Direct`
                // leaf; intermediate layers (nullable-of-iterable etc.)
                // can keep the simple form because Kotlin's `?.` chain
                // already represents the layered null.
                _ => {
                    let v = format!("e{depth}");
                    format!(
                        "{r}?.let {{ {v} -> {} }}",
                        go(inner, &v, kind, w, sentinel, depth + 1)
                    )
                }
            },
            Iterable(inner) => match &**inner {
                Base => {
                    let wrapped = projection_wrap_expr(kind, w, "it");
                    format!("{r}.map {{ {wrapped} }}")
                }
                _ => {
                    let v = format!("e{depth}");
                    format!(
                        "{r}.map {{ {v} -> {} }}",
                        go(inner, &v, kind, w, sentinel, depth + 1)
                    )
                }
            },
        }
    }
    go(strategy, receiver, kind, wrap_class, niche_sentinel, 0)
}

/// JNI extern's declared Kotlin wire-return for a projection. The leaf wire
/// is the inner converter's destination Kotlin name: `Long` for handles
/// (boxed jlong), the inner field's converter result for value classes (e.g.
/// `ByteArray` for `ZenohId`/`ZBytes`). The fold honours
/// [`NullableKind`] so the declared wire matches the runtime ABI:
/// `Niche+primitive` keeps the layer non-nullable on the wire (the sentinel
/// represents null); `Niche+object` and `Boxed` add `?`.
pub(crate) fn projection_wire_return(
    proj: &crate::api::lang::jnigen::jni::Projection,
) -> kt::KtType {
    use crate::api::lang::jnigen::jni::{FoldStrategy, NullableKind, ProjectionKind};
    let (inner_wire, inner_is_primitive) = match proj.kind {
        ProjectionKind::Handle => (kt::KtType::long(), true),
        // Value-blob's inner wire is always `ByteArray` (object-shaped).
        ProjectionKind::ValueBlob => (kt::KtType::byte_array(), false),
        ProjectionKind::Unsigned64 => (kt::KtType::long(), true),
    };
    fold_shape(
        &proj.strategy,
        &|| inner_wire.clone(),
        &|inner, kind, inner_strategy| {
            // A niche layer over a primitive wire keeps the wire non-nullable —
            // the sentinel value is the null representation. Object-wired niches
            // and full-boxed Nullables both add `?` (JVM null on the reference).
            match (kind, inner_strategy) {
                (NullableKind::Niche, FoldStrategy::Base) if inner_is_primitive => inner,
                _ => inner.nullable(),
            }
        },
        &|inner| kt::KtType::generic("List", [inner]),
    )
}

/// Kotlin null-sentinel literal for the *leaf wire* of a projection. Read
/// at the wrapper-body call site and forwarded to [`fold_projection_wrap`];
/// `None` for object-wired leaves (e.g. value classes over `ByteArray`),
/// where `?.let { }` covers the JVM-null case directly.
pub(crate) fn projection_leaf_sentinel(
    proj: &crate::api::lang::jnigen::jni::Projection,
) -> Option<String> {
    use crate::api::lang::jnigen::jni::ProjectionKind;
    let leaf_wire: syn::Type = match proj.kind {
        ProjectionKind::Handle => syn::parse_quote!(jni::sys::jlong),
        // Value-blob leaf wire is always `JByteArray` (object-shaped) — no
        // primitive sentinel; JVM `null` represents the absent value, so
        // `?.let` covers nullability.
        ProjectionKind::ValueBlob => syn::parse_quote!(jni::objects::JByteArray),
        // No niche exists for `u64`; `Option<u64>` uses the boxed path, so a
        // primitive sentinel must never be synthesized.
        ProjectionKind::Unsigned64 => return None,
    };
    kotlin_null_sentinel(&leaf_wire).map(|s| s.to_string())
}

/// Kotlin literal for the null-sentinel of a primitive wire — used by
/// [`fold_projection_wrap`] when a `Niche` layer covers a primitive wire and
/// can't carry JVM null. Mirrors `jni_field_access`'s primitive descriptors.
/// Returns `None` for object-shaped wires (where JVM null *is* the null
/// representation and `?.let` is the right pattern).
pub(crate) fn kotlin_null_sentinel(wire: &syn::Type) -> Option<&'static str> {
    let (_, _, is_object) = crate::api::lang::jnigen::jni::wire_access::jni_field_access(wire)?;
    if is_object {
        return None;
    }
    let syn::Type::Path(tp) = wire else {
        return None;
    };
    let last = tp.path.segments.last()?;
    Some(match last.ident.to_string().as_str() {
        "jlong" => "0L",
        "jint" | "jshort" | "jbyte" | "jchar" => "0",
        "jfloat" => "0.0f",
        "jdouble" => "0.0",
        "jboolean" => "false",
        _ => return None,
    })
}

/// Shorten a class FQN to its simple name for use in **raw body text** (a
/// `fromParts` reconstruct fragment: `Child.fromParts(…)`, `Enum.fromInt(…)`,
/// `ZenohId(bytes)`), registering the FQN into `used` — the body `Code`'s own
/// import list. A non-dotted name (a Kotlin builtin like `String`) needs no
/// import and passes through. Signature types are NOT shortened this way —
/// those are full-FQN `KtType`s in the AST, shortened + imported by the
/// render-time `ImportSet`.
pub(crate) fn register_fqn(fqn: &str, used: &mut BTreeSet<String>) -> String {
    if fqn.contains('.') {
        used.insert(fqn.to_string());
        fqn.rsplit('.').next().unwrap_or(fqn).to_string()
    } else {
        fqn.to_string()
    }
}
