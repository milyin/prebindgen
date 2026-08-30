//! Projection / `FoldStrategy` folding helpers and Kotlin type-shape
//! probes for the JNI back-end's Kotlin emitter.
//!
//! Carved from the former `jni_kotlin_ext.rs`; shares the `jni` namespace
//! via `use super::*`.

use kotlin_codegen::KtType;
use prebindgen_registry::flat::TypeRef;

use super::*;

/// Peel the layers that never change whether a value's core is a Kotlin enum,
/// **off the model**: a borrow and an optional, in any nesting. So `&Priority`,
/// `Priority`, `Option<Priority>` and `Option<&Priority>` all probe as
/// `Priority` — letting nullable enum params (`Option<enum>`) wire as the raw
/// `Int` discriminant plus an allocated niche, instead of leaking the enum
/// object to the Rust converter.
///
/// A **run is not peeled**. `Vec<Priority>` is a `List<Priority>`, not an enum,
/// so this is deliberately not [`TypeRef::layer_stack`], which strips the
/// sequence layer too.
///
/// Borrowing rather than composing is not a shortcut: every layer of a reading
/// already holds the next as a `TypeRef` of its own, so there is nothing to
/// mint — which is also why this needs no registry. What it returns spells
/// itself (`spell()`) and classifies itself (`kind`), and the two cannot
/// disagree.
pub(crate) fn enum_probe(reading: &TypeRef) -> &TypeRef {
    let mut cur = reading;
    while let Some(inner) = cur.borrow_target().or_else(|| cur.optional_inner()) {
        cur = inner;
    }
    cur
}

// The bottom-up layer fold is the shared `prebindgen_registry::shape::fold_shape`
// (its `on_optional` receives the layer's `&NullableKind` + the wrapped
// `&FoldStrategy`, so callers can special-case e.g. a `Niche` layer sitting
// directly over the `Base` leaf). Used by the **type-name** folds
// (`handle_kt_type` / `projection_wire_return`). The **expression** folds
// (`render_handle_close` / `fold_projection_wrap`) are deliberately *not*
// expressed through it: they fold the other direction (threading a `receiver` /
// fresh lambda variable top-down rather than combining a bottom-up result), so
// a shared combinator would obscure rather than simplify them.
use prebindgen_registry::shape::fold_shape;

/// The Kotlin type for a closeable handle reached through the folded
/// [`FoldStrategy`] layers, given the leaf typed-handle type (e.g.
/// `ZKeyExpr`): `Direct → ZKeyExpr`, `Nullable(inner) → <inner>?`,
/// `Iterable(inner) → List<<inner>>`.
pub(crate) fn handle_kt_type(strategy: &FoldStrategy, leaf: &KtType) -> KtType {
    fold_shape(
        strategy,
        &|| leaf.clone(),
        // The declared Kotlin projection type is `T?` regardless of how null
        // is represented over the wire — the wrap fold and the wire-return
        // helper read the kind to handle the wire shape separately.
        &|inner, _kind, _inner_strategy| inner.nullable(),
        &|inner| KtType::generic("List", [inner]),
    )
}

/// Typed Kotlin leaf of a projection. Declared handle projections
/// take their configured class FQN; the built-in `u64` projection is Kotlin's
/// stable unsigned scalar type.
pub(crate) fn projection_leaf_kt(ext: &Declarations, proj: &Projection) -> Option<KtType> {
    match proj.kind {
        ProjectionKind::Handle => ext.kotlin_fqn(&proj.leaf_key).map(KtType::cls),
        ProjectionKind::Unsigned64 => Some(KtType::cls("ULong")),
    }
}

/// Wrap one raw projection leaf into its typed Kotlin form.
pub(crate) fn projection_wrap_expr(kind: &ProjectionKind, short: &str, raw: &str) -> String {
    match kind {
        ProjectionKind::Handle => handle_from_raw(short, raw),
        ProjectionKind::Unsigned64 => format!("{raw}.toULong()"),
    }
}

/// The Kotlin side of one struct's `fromParts` bridge — see
/// [`flatten_struct_factory`], whose returned tuple this names.
pub(crate) type StructFactory = (Vec<(String, KtType)>, String, bool);

/// Recursively build the Kotlin `fromParts` factory for a data class — the
/// mirror of the native `flatten_struct_encode` (in the [`jni`](super)
/// module). Both walk the same [`build_struct_plan`], so the leaf order and
/// slot types agree by construction.
/// Returns `(params, reconstruct, mints_handle)`:
/// * `params` — the flattened `(name, kotlin_type)` list (one per transitive
///   leaf wire; nested data-class fields are inlined, `Option<nested>` prepends
///   a `…__present: Boolean` flag). Order/types match the native call's JVM
///   descriptor positionally.
/// * `reconstruct` — the Kotlin expression building this struct:
///   `Class(<part per constructor field>)`, where a nested field reconstructs
///   via `Child.fromParts(<child param names>)` (`if (present) … else null` when
///   optional) and a leaf reconstructs with its wrap.
/// * `mints_handle` — whether any transitive leaf is an opaque handle, i.e.
///   whether this factory takes a raw native pointer and so needs the
///   raw-pointer guard (see [`plan_mints_handle`]).
#[allow(clippy::too_many_arguments)]
pub(crate) fn flatten_struct_factory(
    ext: &Declarations,
    registry: &Registry,
    s: &prebindgen_registry::flat::Struct,
    prefix: &str,
    class_name: &str,
    imports: &mut BTreeSet<String>,
    depth: usize,
) -> Option<StructFactory> {
    let _ = (prefix, depth);
    let leaves = ext.struct_out_frozen(registry, s)?;
    let (params, reconstruct) = factory_from_leaves(ext, registry, &leaves, class_name, imports)?;
    // Whether this factory takes a raw native pointer, and so needs the
    // raw-pointer guard: asked of the same leaves, through the wrap that says
    // a slot is a handle.
    let mints_handle = crate::jni::iface::plan_leaf_params(ext, &leaves)?
        .iter()
        .any(|param| param.wrap.class_fqn().is_some());
    Some((params, reconstruct, mints_handle))
}

/// The Kotlin `fromParts` side, derived from the struct's **decomposition** —
/// the same leaf list the Rust encode renders from and the same one a builder
/// interface declares.
///
/// One derivation, so the factory's parameters cannot drift from the slots the
/// encoder fills. What each part of the expression needs, the leaves already
/// say: `reach` which field a leaf came from, `through` which class a boundary
/// reassembles through, `groups` and the selectors which arm gates it.
///
/// Returns `(params, reconstruct)`, where `reconstruct` is
/// `Class(<one part per constructor field>)`.
fn factory_from_leaves(
    ext: &Declarations,
    registry: &Registry,
    leaves: &[crate::jni::compile::OutWire],
    class_name: &str,
    imports: &mut BTreeSet<String>,
) -> Option<(Vec<(String, KtType)>, String)> {
    let params = crate::jni::iface::plan_leaf_params(ext, leaves)?;
    let names = crate::jni::render::plan_leaf_names(leaves);
    let declared: Vec<(String, KtType)> = names
        .iter()
        .zip(&params)
        .map(|(name, param)| (name.clone(), param.raw.clone()))
        .collect();

    // One part per constructor field, and a field is what a leaf's reach
    // starts with — a nested class's leaves, a sum's tag and groups, and an
    // optional's flag all reach through the one field that carries them.
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < leaves.len() {
        let field = |j: usize| {
            leaves[j]
                .reach()
                .first()
                .map(|step| step.ident().to_string())
        };
        let head = field(i);
        let end = (i..leaves.len())
            .take_while(|&j| field(j) == head)
            .last()
            .map_or(i + 1, |j| j + 1);
        parts.push(factory_part(
            ext,
            registry,
            &leaves[i..end],
            &params[i..end],
            &names[i..end],
            None,
            imports,
        )?);
        i = end;
    }
    Some((declared, format!("{class_name}({})", parts.join(", "))))
}

/// The expression rebuilding ONE constructor field from the leaves that carry
/// it: a plain leaf through its own wrap, a nested class through its
/// `fromParts`, a sum through the `when` over its tag, and any of those behind
/// a presence flag when the field is optional.
#[allow(clippy::too_many_arguments)]
fn factory_part(
    ext: &Declarations,
    registry: &Registry,
    leaves: &[crate::jni::compile::OutWire],
    params: &[crate::jni::IfaceParam],
    names: &[String],
    // `ungated`: the same leaves as the value's OWN class declares them, when a
    // presence flag above made the parent's copies nullable. `None` when
    // nothing gates them, in which case the two are the same.
    ungated: Option<&[crate::jni::IfaceParam]>,
    imports: &mut BTreeSet<String>,
) -> Option<String> {
    // The class boundary is asked FIRST. A nested child may itself begin with
    // a selector — an `Option<Mid>` whose `Mid` starts with an optional field,
    // or with a sum — and that selector belongs to the CHILD's signature, not
    // to this one. Reading `from` before `through` consumed it here and called
    // the child factory without it, which is a `fromParts` arity mismatch the
    // JVM only reports at the call (#620 review).
    // A nested class: its leaves are the parent's, put back through the class
    // the decomposition recorded when it flattened them. A gated one receives
    // wire defaults when absent, so its object slots are nullable in the
    // parent's signature and re-asserted inside the guard the flag opened.
    if let Some(fqn) = leaves[0].through.first() {
        let short = register_fqn(fqn, imports);
        // Re-asserted only where the NULL came from the gate, never where the
        // value is legitimately absent: a leaf whose own type is optional is
        // nullable on its own account, and `!!` there would throw on a value
        // the class is meant to hold. The same distinction `sum_ctor_arg`
        // makes for an inert group's payload.
        let forwarded: Vec<String> = names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                // Re-asserted only where the null is the GATE's doing: the
                // parent declares the slot nullable so the encoder can default
                // it when absent, while the class itself declares it non-null.
                // A slot the class already declares nullable — its own type is
                // optional, or it belongs to a sum group that is inert whenever
                // another alternative is live — is forwarded untouched, because
                // `!!` there would throw on a value the class is meant to hold.
                let gate_added = params[i].raw.is_nullable()
                    && !ungated.is_none_or(|own| own[i].raw.is_nullable());
                match gate_added {
                    true => format!("{name}!!"),
                    false => name.clone(),
                }
            })
            .collect();
        return Some(format!("{short}.fromParts({})", forwarded.join(", ")));
    }
    // A presence flag gates everything after it, and contributes no value of
    // its own: the field is what its group rebuilds, or null.
    if matches!(leaves[0].from, crate::jni::compile::OutFrom::Present) {
        // Inside the guard the gate's own arm is no longer in question, so the
        // group it opened is stripped: what is left is the value as its own
        // class declares it, which is what says whether a null here is this
        // gate's doing or the value's own.
        let inner: Vec<crate::jni::compile::OutWire> = leaves[1..]
            .iter()
            .map(|leaf| {
                let mut leaf = leaf.clone();
                leaf.groups = leaf
                    .groups
                    .split_first()
                    .map_or_else(Vec::new, |(_, rest)| rest.to_vec());
                leaf
            })
            .collect();
        let inner_params = crate::jni::iface::plan_leaf_params(ext, &inner)?;
        let gated = factory_part(
            ext,
            registry,
            &leaves[1..],
            &params[1..],
            &names[1..],
            Some(&inner_params),
            imports,
        )?;
        return Some(format!("if ({}) {gated} else null", names[0]));
    }
    if leaves[0].is_tag() {
        let (_, when) = ext.sum_reconstruct(
            registry,
            &leaves[0].out_ty.unwrapped().key(),
            leaves,
            params,
            names,
            imports,
        );
        return Some(when);
    }
    debug_assert_eq!(
        leaves.len(),
        1,
        "an unselected field with no class path is one leaf"
    );
    Some(ext.sum_ctor_arg(&leaves[0], &params[0], &names[0], imports))
}

/// Render the Kotlin `close()` expression for a handle `receiver` through
/// the folded [`FoldStrategy`] layers. Fresh lambda variable per nesting
/// level avoids `it` shadowing; the common single-layer cases are
/// special-cased for readable output (`x?.close()`, `x.forEach { it.close() }`).
pub(crate) fn render_handle_close(strategy: &crate::jni::FoldStrategy, receiver: &str) -> String {
    use prebindgen_registry::shape::Shape::*;
    fn go(strategy: &crate::jni::FoldStrategy, receiver: &str, depth: usize) -> String {
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
    strategy: &crate::jni::FoldStrategy,
    receiver: &str,
    kind: &crate::jni::ProjectionKind,
    wrap_class: &str,
    niche_sentinel: Option<&str>,
) -> String {
    use prebindgen_registry::shape::Shape::*;

    use crate::jni::NullableKind;
    fn go(
        s: &crate::jni::FoldStrategy,
        r: &str,
        kind: &crate::jni::ProjectionKind,
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
/// is the inner converter's destination Kotlin name — `Long` for both
/// projection kinds (a handle's pointer, a `ULong`'s raw bit pattern). The
/// fold honours
/// [`NullableKind`] so the declared wire matches the runtime ABI:
/// `Niche+primitive` keeps the layer non-nullable on the wire (the sentinel
/// represents null); `Niche+object` and `Boxed` add `?`.
pub(crate) fn projection_wire_return(proj: &crate::jni::Projection) -> KtType {
    use crate::jni::{FoldStrategy, NullableKind, ProjectionKind};
    let (inner_wire, inner_is_primitive) = match proj.kind {
        ProjectionKind::Handle => (KtType::long(), true),
        ProjectionKind::Unsigned64 => (KtType::long(), true),
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
        &|inner| KtType::generic("List", [inner]),
    )
}

/// Kotlin null-sentinel literal for the *leaf wire* of a projection. Read
/// at the wrapper-body call site and forwarded to [`fold_projection_wrap`];
/// `None` when the leaf wire has no primitive null sentinel, where
/// `?.let { }` covers the JVM-null case directly.
pub(crate) fn projection_leaf_sentinel(proj: &crate::jni::Projection) -> Option<String> {
    if let Some(sentinel) = proj.niche_sentinels.first() {
        return Some(sentinel.clone());
    }
    use crate::jni::ProjectionKind;
    let leaf_wire: syn::Type = match proj.kind {
        ProjectionKind::Handle => syn::parse_quote!(jni::sys::jlong),
        // No niche exists for `u64`; `Option<u64>` uses the boxed path, so a
        // primitive sentinel must never be synthesized.
        ProjectionKind::Unsigned64 => return None,
    };
    kotlin_null_sentinel(&leaf_wire).map(|s| s.to_string())
}

/// The sentinel a **wrap** should test, given the projection and whether an
/// ancestor makes the leaf nullable — the one rule both derivations of that
/// wrap read (`unfold_leaf_kt` in `render.rs`, `leaf_iface_param` in
/// `iface.rs`), so it cannot drift between them again (#142).
///
/// A sentinel is the leaf's **own** `None` representation, so it belongs to the
/// leaf's own type and to nothing above it. Two independent facts meet here and
/// only the first one answers:
///
/// * the leaf's type carries a niche — `Option<Duration>` over a bounded
///   `convert!`, whose strategy is `Optional(Niche, _)`. Its `None` IS the
///   sentinel, and the test stays whatever the ancestor does.
/// * an **ancestor** can be absent (a conditional value form, an
///   `Option<sum>`/`Option<nested>` field). That widens the wire — the Rust
///   side boxes any nullable leaf, see
///   [`leaf_is_prim`](crate::jni::emit::leaf_is_prim) — and `?.` alone carries
///   it. It grants no sentinel.
///
/// [`projection_leaf_sentinel`] answers off `niche_sentinels`, which
/// `attach_domain_sentinels` puts on the **bare** type's converter as well as
/// the `Option` one. Asking it without the `Base` check therefore handed a
/// sentinel to a leaf that has no niche encoding at all, splicing
/// `?.let { if (it == -1L) … }` into a wrap whose own encoder can never emit
/// `-1`. Harmless at runtime — the value is outside the declared range — and
/// wrong on its face.
pub(crate) fn wrap_sentinel(proj: &crate::jni::Projection, nullable: bool) -> Option<String> {
    // `Base` means the leaf has no `Option` of its own: whatever absence it can
    // express is the ancestor's, and `?.` already expresses that.
    if nullable && matches!(proj.strategy, crate::jni::FoldStrategy::Base) {
        return None;
    }
    projection_leaf_sentinel(proj)
}

/// Kotlin literal for the null-sentinel of a primitive wire — used by
/// [`fold_projection_wrap`] when a `Niche` layer covers a primitive wire and
/// can't carry JVM null. Mirrors `jni_field_access`'s primitive descriptors.
/// Returns `None` for object-shaped wires (where JVM null *is* the null
/// representation and `?.let` is the right pattern).
pub(crate) fn kotlin_null_sentinel(wire: &syn::Type) -> Option<&'static str> {
    let (_, _, is_object) = crate::jni::wire_access::jni_field_access(wire)?;
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
