//! Cbindgen's per-type policies, lowered into the registry's row vocabulary.
//!
//! A build script writes `.opaque_ptr()`, `.data_struct()`, `.enum_type()`,
//! `.tagged_union()` or `.repr_c_struct()`, each saying what C shape one Rust
//! type takes. This turns those into rows in a [`Recipes`] table: the shared
//! statement of **which parts** a value gets across in, with nothing about the
//! C wire in it.
//!
//! There is one row per declared type and job, and every crossing nobody
//! declared takes the row the registry derives from its kind. So the chain of
//! `or_else` guesses this replaced — try a custom conversion, then a handle,
//! then a data struct, then an enum, … — is now a lookup, and the answer is
//! whatever the build script said rather than whichever guess fired first.

use prebindgen_registry::{
    flat::Flat,
    recipe::{Constructing, Deconstructing, RecipeError, RecipeId, Recipes},
};

use super::*;

/// The row a type with no parts takes: the adapter emits the conversion itself.
fn whole() -> RecipeId {
    RecipeId::new("whole")
}

impl CbindgenBuilder {
    /// Every row this binding's declarations state.
    ///
    /// A type declared but absent from the model is skipped rather than
    /// refused: the scan already reports it, and reporting it twice in
    /// different words helps nobody.
    pub(crate) fn recipes(&self, model: &Flat) -> Result<Recipes, Vec<RecipeError>> {
        let mut rows = Recipes::builder();
        // Every per-type policy, and every `convert!`-declared conversion. The
        // second matters as much as the first: a conversion may be declared on
        // a type the registry would otherwise read as an arity layer, and
        // `convert!(Option<Duration> => ..)` means the adapter emits that
        // optional's conversion itself rather than wrapping `Duration`'s.
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
            // The declarator's own tokens, re-parsed: a declared type may be
            // an alias the model holds as an `Extern`, which carries no reading
            // of its own to borrow.
            let Ok(spelled) = syn::parse2::<syn::Type>(origin.declared_spelling()) else {
                continue;
            };
            let Ok(ty) = model.classify(&spelled) else {
                continue;
            };
            // Every declared C shape is one row with no parts today, including
            // the two that plainly have some: `in_data_struct` walks a struct's
            // fields inside one generated function, and `in_tagged_union` walks
            // an arm's payload inside another. `Atomic` is what a row says about
            // that — the adapter emits the conversion itself — so it describes
            // the adapter as it stands rather than as it will be. Stating those
            // parts is what lets the two walks be deleted, and is the next
            // stage.
            rows.declare(ty.clone(), whole(), Deconstructing::Atomic)
                .declare(ty, whole(), Constructing::Atomic);
        }
        rows.build(model)
    }
}
