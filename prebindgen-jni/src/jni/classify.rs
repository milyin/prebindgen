//! One-stop classification of how a bare Rust type is declared to this
//! adapter — the single precedence every emitter agrees on instead of each
//! re-deriving it from `TypeConfig` flags and `registry.flat()` type probes.

use prebindgen_registry::Conversions;

use super::*;

/// The adapter-declared kind of a **bare** (already `Option`/`&`-stripped)
/// Rust type: the declared [`DeclaredKind`] when the type is declared to this
/// adapter, else a registered source struct, else everything else.
///
/// The three special kinds cannot overlap — a type stores exactly one
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
    /// A `#[prebindgen]` struct from the source crate that is none of the
    /// special kinds; flattens field-by-field when emitters support it.
    ///
    /// The **element**, not its `syn::ItemStruct`. A flattening emitter wants
    /// each field's reading, and the model already decided one per field; going
    /// through the syntax means asking some other authority for it again. An
    /// emitter that only re-emits the struct reads `st.origin.as_syn()`.
    DataStruct {
        st: &'r prebindgen_registry::flat::Struct,
        cfg: Option<&'c TypeConfig>,
    },
    /// Scalars, `String`, undeclared / non-path types.
    Other,
}

impl TypeConfig {
    /// Declared as one of the three non-data-class kinds (`ptr_class` /
    /// `enum_class` / `sealed_class`) — types with their own
    /// dedicated Kotlin emitters, never flattened as data classes.
    pub(crate) fn special_decl(&self) -> bool {
        !matches!(self.kind, DeclaredKind::Data)
    }
}

impl Declarations {
    /// Classify `bare` against the declared-type table and the registry's
    /// captured structs. Callers strip `Option<_>` / `&_` layers first —
    /// wrapper folding is the resolver's business, not this table's.
    pub(crate) fn type_kind<'r, 'c>(
        &'c self,
        registry: &'r impl Conversions,
        bare: &TypeKey,
    ) -> TypeKind<'r, 'c> {
        let cfg = self.types.get(bare);
        if let Some(c) = cfg {
            match c.kind {
                DeclaredKind::Ptr(_) => return TypeKind::Handle,
                DeclaredKind::Enum(_) => return TypeKind::Enum,
                DeclaredKind::Sealed(_) => return TypeKind::Sum,
                // A data class is exactly a declared source struct — fall
                // through to the registry probe below, which supplies the
                // element its emitters flatten.
                DeclaredKind::Data => {}
            }
        }
        // The key IS the canonical spelling, so a key that is one identifier is
        // exactly what `bare_path_ident` used to fish out of the node — and this
        // function never wanted anything else from it.
        if let Some(name) = bare.ident() {
            if let Some(st) = registry.flat().struct_type(&name) {
                return TypeKind::DataStruct { st, cfg };
            }
        }
        TypeKind::Other
    }
}
