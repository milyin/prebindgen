//! JNI adapter metadata propagated through `TypeEntry`.

use super::kt;

/// How a `Shape::Optional` fold layer represents `None` over the JNI wire —
/// the per-layer payload `N` of the JNI adapter's [`FoldStrategy`].
///
/// The choice is made at the point the `Option<_>` wrapper folds the layer onto
/// a projection's `FoldStrategy`, and only depends on whether `option_output`
/// rode the inner converter's niche (wire stayed identical to the inner's wire)
/// or boxed the primitive into `java.lang.<Box>` (wire widened to `JObject`).
/// The renderer reads this to pick the matching Kotlin shape — without it, a
/// primitive-wired `Option<Handle>` would be declared as nullable `Long?` even
/// though the wire is a non-nullable `jlong` whose `0L` *is* the null.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NullableKind {
    /// Wire kept the inner converter's encoding; `None` is the carved niche
    /// slot's `value` (e.g. `0L` for a handle's `jlong`). On Kotlin the
    /// declared wire is non-nullable; the wrapper body converts the sentinel
    /// to `null` explicitly via an `if (it == <sentinel>) null else W(it)`
    /// pattern.
    Niche,
    /// Wire widened to `JObject`; `None` is JVM `null`. On Kotlin the
    /// declared wire is nullable and `?.let { W(it) }` works directly. This
    /// is also the rendering object-shaped niches (`JByteArray::null` /
    /// `JString::null`) collapse onto — Kotlin's `T?` already maps to JVM
    /// reference-null at no extra cost.
    Boxed,
}

/// The JNI adapter's nullability / collection layer stack over a handle or
/// value-class leaf, on the unified [`Shape`](crate::api::core::shape::Shape)
/// with [`NullableKind`] as the per-`Optional`-layer payload:
///   * `Base` — the receiver *is* the handle;
///   * `Optional(kind, inner)` — `T?`; `kind` records how null is represented
///     over the wire (see [`NullableKind`]);
///   * `Iterable(inner)` — `List<T>`. EXTENSION POINT: no `Vec<Handle>` shape
///     exists today, so the emitters guard this arm loudly rather than silently
///     mis-generating.
pub type FoldStrategy = crate::api::core::shape::Shape<NullableKind>;

/// Which flavor of Kotlin newtype a [`Projection`] surfaces. Both share the
/// same "wire != declared Kotlin type, wrap as `W(wire)`, fold through
/// `Option`/`Vec`" shape; they differ only in how a struct field stores them
/// and whether they own a closeable resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionKind {
    /// Opaque native handle (`ptr_class`). Wire is `jlong`; a struct field
    /// stores the **boxed** handle object (`L<fqn>;`); closeable when owned.
    Handle,
    /// Kotlin `@JvmInline value class` wrapping a **`Copy` value-blob**
    /// (`value_blob`). Its inner is always a raw `ByteArray` (`[B`) — there is
    /// no Rust struct field to resolve. The typed class has a single
    /// `bytes: ByteArray` field; the wire is `JByteArray`. Never closeable.
    ValueBlob,
    /// Rust `u64`: raw JNI `jlong` bit pattern with a typed Kotlin `ULong`
    /// surface. It owns no resource; wrapping/unwrapping is
    /// `Long.toULong()` / `ULong.toLong()`.
    Unsigned64,
}

/// Folded description of a Kotlin newtype projection (opaque handle or value
/// class) reached through zero or more wrapper layers. Set at the leaf,
/// transformed by each wrapper as the type folds (see [`FoldStrategy`]), and
/// read by every typed-surface emitter (data-class fields, struct
/// encode/decode, `classify_return`, param classification) so "what Kotlin
/// class does this surface, how do I wrap/fold it, do I close it" has one
/// source of truth instead of a parallel ad-hoc decision tree.
#[derive(Clone, Debug)]
pub struct Projection {
    /// Canonical key of the leaf type (e.g. `ZKeyExpr`, `ZenohId`); derive
    /// the typed Kotlin FQN via `JniGen::kotlin_fqn` — a typed key, so the
    /// lookup cannot drift from the declaration table's constructor.
    pub leaf_key: crate::api::core::registry::TypeKey,
    /// `false` for `&T` borrows of a handle — still a projection (param
    /// classification needs this), but not the holder's to close, so
    /// `close()` emission skips it. Always `false` for [`ProjectionKind::ValueBlob`].
    pub owned: bool,
    /// Nullability / collection layers.
    pub strategy: FoldStrategy,
    /// Handle vs value class — see [`ProjectionKind`].
    pub kind: ProjectionKind,
    /// Kotlin literals for representation-domain niches, in carve order.
    /// Empty for ordinary projections; bounded u64 conversions populate it.
    pub niche_sentinels: Vec<String>,
}

/// Per-converter language-specific extras carried by every converter this
/// adapter produces. Filled by the same handler that builds the wire/body,
/// propagated by the resolver into
/// [`crate::api::core::registry::TypeEntry::metadata`], and read directly by
/// the Kotlin emitter — so cross-language facts flow through the existing
/// wrapper machinery rather than a parallel side channel.
#[derive(Clone, Debug, Default)]
pub struct KotlinMeta {
    /// Value-context Kotlin type, structured ([`kt::KtType`]). `Long` for
    /// opaque handles (jlong wire mention), the FQN class
    /// (`io.zenoh.jni.JNIEncoding`) for user-declared decoder types whose
    /// wire isn't primitive, a composed `List<ByteArray>` when a wrapper
    /// wraps an inner. Leaves carry FQNs; the Kotlin renderer's `ImportSet`
    /// shortens them at render time. `None` only for entries that must not
    /// appear in any Kotlin signature — the emitter treats that as a hard
    /// error.
    pub kotlin_name: Option<kt::KtType>,
    /// For wrapper converters whose Kotlin projection is the *inner*
    /// type's projection (e.g. `ZResult<Publisher>` → `Publisher`),
    /// this carries the inner Rust type's canonical key so downstream
    /// emitters (typed-handle constructor lookup in `classify_return`) can find
    /// the wrapped value's identity without baking in any specific shape.
    /// Populated with `args[0]`'s canonical key for arity-1 wrappers, and
    /// inherited by the built-in `Option<_>` / `Vec<_>` / `&_` wrappers from
    /// their inner type's metadata. `None` for plain values and arity-0
    /// converters. A typed key — readers get the type via `to_type()`, no
    /// reparse.
    pub value_rust_key: Option<crate::api::core::registry::TypeKey>,
    /// Present iff this (possibly wrapped) value is an opaque native handle. Set
    /// at the opaque-handle leaf and folded outward by the `&_` / `Option<_>`
    /// wrappers and the `lookup_*` composed branches. The single source of truth
    /// for typed-handle rendering and `close()` generation — see [`Projection`].
    pub projection: Option<Projection>,
}

impl KotlinMeta {
    pub fn from_name(name: impl Into<String>) -> Self {
        Self {
            kotlin_name: Some(kt::KtType::cls(name)),
            value_rust_key: None,
            projection: None,
        }
    }

    /// True iff this (input-direction) converter decodes a directly-consumable
    /// owned opaque handle — i.e. its projection is a bare `Handle` leaf with no
    /// `Option`/`Vec` fold. Replaces the former `converter_returns_owned_object`
    /// return-type AST sniff; the two are equivalent for every input converter
    /// this adapter produces.
    pub(crate) fn is_direct_handle(&self) -> bool {
        self.projection.as_ref().is_some_and(|p| {
            p.kind == ProjectionKind::Handle && matches!(p.strategy, FoldStrategy::Base)
        })
    }
}
