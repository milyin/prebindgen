//! One-stop classification of how a bare Rust type is declared to this
//! adapter — the single precedence every emitter agrees on instead of each
//! re-deriving it from `TypeConfig` flags and `registry.structs` probes.

use super::*;

/// The adapter-declared kind of a **bare** (already `Option`/`&`-stripped)
/// Rust type, in fixed precedence: opaque handle → enum class → value blob
/// → registered source struct → everything else.
///
/// The three special kinds are mutually exclusive by builder enforcement;
/// the precedence order only pins down behavior if that invariant is ever
/// violated. `DataStruct` is any struct captured from the source crate —
/// `cfg` tells whether it was also declared to the builder (a `data_class`
/// candidate) or is merely known to the registry.
pub(crate) enum TypeKind<'r, 'c> {
    /// Declared via `ptr_class` — jlong wire, typed-handle Kotlin class.
    Handle,
    /// Declared via `enum_class` — jint wire, Kotlin `enum class`.
    Enum,
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
    /// Declared as one of the three non-data-class kinds (`ptr_class` /
    /// `enum_class` / `value_class`) — types with their own dedicated
    /// Kotlin emitters, never flattened as data classes.
    pub(crate) fn special_decl(&self) -> bool {
        self.opaque.is_some() || self.enum_cfg.is_some() || self.value_blob
    }
}

impl<S> JniGen<S> {
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
            if c.opaque.is_some() {
                return TypeKind::Handle;
            }
            if c.enum_cfg.is_some() {
                return TypeKind::Enum;
            }
            if c.value_blob {
                return TypeKind::ValueBlob;
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
