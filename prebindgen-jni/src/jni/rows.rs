//! JniGen's Kotlin class declarations, lowered into the registry's row
//! vocabulary.
//!
//! A build script writes `ptr_class!`, `data_class!`, `enum_class!` or
//! `sealed_class!`, each naming the Kotlin type one Rust type becomes, and
//! `convert!` for a conversion the binding supplies itself. This turns those
//! into rows in a [`Recipes`] table: the shared statement of **which parts** a
//! value gets across in, with nothing about the JNI wire in it.
//!
//! One row per declared type and job, and every crossing nobody declared takes
//! the row the registry derives from its kind. The chain of guesses this
//! replaced — try a terminal, then a `Result`, then an optional, then a run,
//! then a borrow, then a transparent bridge — is a lookup for the declared
//! half, and the derived half is the registry's.

use std::collections::BTreeMap;

use prebindgen_registry::{
    flat::{Flat, TypeRef},
    recipe::{Constructing, Deconstructing, RecipeError, RecipeId, Recipes},
};

use super::*;

/// Every type one captured item names, so a reading nested inside it can be
/// found without the caller knowing which kind of item it came from.
fn element_types(element: &prebindgen_registry::Element) -> Vec<&TypeRef> {
    use prebindgen_registry::flat::Type;
    match element {
        prebindgen_registry::Element::Function(f) => f
            .params
            .iter()
            .map(|p| &p.ty)
            .chain(std::iter::once(&f.ret))
            .collect(),
        prebindgen_registry::Element::Type(Type::Struct(s)) => {
            s.fields.iter().map(|f| &f.ty).collect()
        }
        prebindgen_registry::Element::Type(Type::Variant(v)) => v
            .alternatives
            .iter()
            .flat_map(|a| a.fields.iter().map(|f| &f.ty))
            .collect(),
        prebindgen_registry::Element::Constant(c) => vec![&c.ty],
        _ => Vec::new(),
    }
}

/// The row a type with no parts takes: the adapter emits the conversion itself.
fn whole() -> RecipeId {
    RecipeId::new("whole")
}

impl Declarations {
    /// Every row this binding's declarations state.
    ///
    /// A type declared but absent from the model is skipped rather than
    /// refused: the scan already reports it, and reporting it twice in
    /// different words helps nobody.
    pub(crate) fn recipes(&self, model: &Flat) -> Result<Recipes, Vec<RecipeError>> {
        let mut rows = Recipes::builder();
        // Every Kotlin class declaration, and every `convert!`-declared
        // conversion. The second matters as much as the first: a conversion may
        // be declared on a type the registry would otherwise read as an arity
        // layer, and `convert!(Option<Duration> => ..)` means the adapter emits
        // that optional's conversion itself rather than wrapping `Duration`'s.
        let mut declared: Vec<(TypeKey, Origin<syn::Type>)> = self
            .declared_types()
            .into_iter()
            .chain(
                self.convert_decls
                    .iter()
                    .map(|d| (d.key().clone(), d.rust_type().clone())),
            )
            .collect();
        declared.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        declared.dedup_by(|a, b| a.0 == b.0);

        for (_key, origin) in declared {
            // The declarator's own tokens, re-parsed: a declared type may be an
            // alias the model holds as an `Extern`, which carries no reading of
            // its own to borrow.
            let Ok(spelled) = syn::parse2::<syn::Type>(origin.declared_spelling()) else {
                continue;
            };
            let Ok(ty) = model.classify(&spelled) else {
                continue;
            };
            // Every declared Kotlin shape is one row with no parts today,
            // including the two that plainly have some: a `data_class` arrives
            // as separate JNI parameters and is rebuilt inside one generated
            // function, and a `sealed_class` is a selector plus the live arm's
            // slots inside another. `Atomic` is what a row says about that —
            // the adapter emits the conversion itself, and how many wire values
            // that costs is its own business — so it describes the adapter as it
            // stands rather than as it will be.
            rows.declare(ty.clone(), whole(), Deconstructing::Atomic)
                .declare(ty, whole(), Constructing::Atomic);
        }

        // A fixed-size array of JNI primitives is one Kotlin `ByteArray` or
        // `LongArray`, bulk-copied with nothing boxed — one wire value, not a
        // run this adapter walks. The registry reads `[T; N]` as a run unless
        // something says otherwise, and this is JniGen saying otherwise for
        // every array the model holds. Nothing is enumerated twice: a row for
        // a crossing nobody uses is inert.
        // Keyed by identity in a `BTreeMap`, so each array type is keyed once
        // and the declaration order is the key order — no sort, no second pass
        // rebuilding a key it already has, and nothing cloned before the entry
        // turns out to be new.
        let mut arrays: BTreeMap<TypeKey, &TypeRef> = BTreeMap::new();
        for ty in model
            .elements()
            .flat_map(element_types)
            .flat_map(|ty| ty.walk())
            .filter(|ty| matches!(ty.kind(), prebindgen_registry::flat::TypeKind::Array { .. }))
        {
            arrays.entry(ty.key()).or_insert(ty);
        }
        for ty in arrays.into_values() {
            rows.declare(ty.clone(), whole(), Deconstructing::Atomic)
                .declare(ty.clone(), whole(), Constructing::Atomic);
        }

        rows.build(model)
    }
}
