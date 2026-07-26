//! One-stop classification of how a bare Rust type is declared to this
//! adapter — the single precedence every emitter agrees on instead of each
//! re-deriving it from `TypeConfig` flags and `registry.structs` probes.

use super::*;

/// The adapter-declared kind of a **bare** (already `Option`/`&`-stripped)
/// Rust type: the declared [`DeclaredKind`] when the type is declared to this
/// adapter, else a registered source struct, else everything else.
///
/// The four special kinds cannot overlap — a type stores exactly one
/// [`DeclaredKind`], so this is a lookup, not a precedence chain.
/// `DataStruct` is any struct captured from the source crate — `cfg` tells
/// whether it was also declared to the builder (a `data_class` candidate) or
/// is merely known to the registry.
pub(crate) enum TypeKind<'r, 'c> {
    /// Declared via `ptr_class` — jlong wire, typed-handle Kotlin class.
    Handle,
    /// Declared via `enum_class` — jint wire, Kotlin `enum class`.
    Enum,
    /// Declared via `sealed_class` — a data-carrying enum: an `Int` tag plus
    /// one leaf group per variant, surfacing as a Kotlin `sealed interface`.
    /// It has no single wire of its own; it crosses flattened.
    Sum,
    /// Declared via `value_class` — raw-memory `JByteArray` wire.
    ValueBlob,
    /// A `#[prebindgen]` struct from the source crate that is none of the
    /// special kinds; flattens field-by-field when emitters support it.
    DataStruct {
        st: &'r syn::ItemStruct,
        cfg: Option<&'c TypeConfig>,
    },
    /// Scalars, `String`, undeclared / non-path types.
    Other,
}

impl TypeConfig {
    /// Declared as one of the four non-data-class kinds (`ptr_class` /
    /// `enum_class` / `sealed_class` / `value_class`) — types with their own
    /// dedicated Kotlin emitters, never flattened as data classes.
    pub(crate) fn special_decl(&self) -> bool {
        !matches!(self.kind, DeclaredKind::Data)
    }
}

impl JniGen {
    /// Classify `bare` against the declared-type table and the registry's
    /// captured structs. Callers strip `Option<_>` / `&_` layers first —
    /// wrapper folding is the resolver's business, not this table's.
    pub(crate) fn type_kind<'r, 'c>(
        &'c self,
        registry: &'r Registry<KotlinMeta>,
        bare: &syn::Type,
    ) -> TypeKind<'r, 'c> {
        let cfg = self.types.get(&TypeKey::from_type(bare));
        if let Some(c) = cfg {
            match c.kind {
                DeclaredKind::Ptr(_) => return TypeKind::Handle,
                DeclaredKind::Enum(_) => return TypeKind::Enum,
                DeclaredKind::Sealed(_) => return TypeKind::Sum,
                DeclaredKind::Value => return TypeKind::ValueBlob,
                // A data class is exactly a declared source struct — fall
                // through to the registry probe below, which supplies the
                // `syn::ItemStruct` its emitters flatten.
                DeclaredKind::Data => {}
            }
        }
        if let Some(name) = bare_path_ident(bare) {
            if let Some((st, _)) = registry.structs.get(&name) {
                return TypeKind::DataStruct { st, cfg };
            }
        }
        TypeKind::Other
    }
}
