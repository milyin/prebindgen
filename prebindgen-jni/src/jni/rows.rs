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
    recipe::{Construct, Constructing, Deconstructing, RecipeError, RecipeId, Recipes},
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

/// The row a `data_class` takes when it crosses **as its fields**.
///
/// Not the default yet. `whole()` still answers for every site, and this one is
/// compiled only where something asks for it by name — which is what lets the
/// composed wire list be checked against the walk it will replace before
/// anything depends on it.
pub(crate) fn parts() -> RecipeId {
    RecipeId::new("parts")
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
            rows.declare(ty.clone(), whole(), Deconstructing::Atomic);
            // A `data_class` also has a row that says what it is made of, so
            // its constructing side names which of the two a site takes by
            // default. See [`parts`] for why that is still `whole`.
            if matches!(
                self.types.get(&ty.key()).map(|c| &c.kind),
                Some(DeclaredKind::Data)
            ) {
                rows.declare_default(ty.clone(), whole(), Constructing::Atomic)
                    .declare(ty, parts(), Constructing::Product(Construct::Fields));
            } else {
                rows.declare(ty, whole(), Constructing::Atomic);
            }
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

impl Declarations {
    /// Which row each part of a `data_class` takes.
    ///
    /// A field that is itself a `data_class` crosses as **its** fields too, so
    /// the part site asks for [`parts`] rather than letting the field take its
    /// own default. That is the one thing a site can say that a row cannot: the
    /// same crossing is read one way on its own and another inside a product.
    ///
    /// Without it a nested class contributes a single wire and the flattening
    /// stops one layer down.
    /// Whether a `data_class` field is itself flattened, or stays one value.
    ///
    /// A `data_class` declared `.jobject_input()` crosses Kotlin → Rust as a
    /// single `JObject` — that is the whole point of the opt-in — so binding
    /// its parts would flatten a boundary the declaration drew.
    fn field_crosses_as_its_fields(&self, ty: &TypeRef) -> bool {
        self.types
            .get(&ty.stripped_key())
            .is_some_and(|c| matches!(c.kind, DeclaredKind::Data) && !c.jobject_input)
    }

    pub(crate) fn bindings(
        &self,
        model: &Flat,
        recipes: &prebindgen_registry::recipe::Recipes,
    ) -> Result<prebindgen_registry::recipe::Bindings, Vec<RecipeError>> {
        use prebindgen_registry::recipe::{
            Ask, Assembly, Bindings, Crossing, Origin, RecipeId, Site,
        };

        let mut bound = Bindings::builder();
        let mut declared: Vec<TypeKey> = self
            .types
            .iter()
            .filter(|(_, c)| matches!(c.kind, DeclaredKind::Data))
            .map(|(k, _)| k.clone())
            .collect();
        declared.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        for key in declared {
            let Some(ident) = key.ident() else { continue };
            let Some(prebindgen_registry::flat::Type::Struct(s)) = model.declared_type(&ident)
            else {
                continue;
            };
            let Ok(ty) = model.classify(&syn::parse_quote!(#ident)) else {
                continue;
            };
            let of = Crossing::new(ty, Assembly::Construct);
            for (index, field) in s.fields.iter().enumerate() {
                // An `Option<D>` field reaches D through the optional's own
                // row, so the part bound here is the optional and the inner is
                // bound below.
                let target = field.ty.optional_inner().unwrap_or(&field.ty);
                if !self.field_crosses_as_its_fields(target) {
                    continue;
                }
                match field.ty.optional_inner() {
                    // The field IS the class: its part takes the `parts` row.
                    None => bound.bind(
                        Site::part(&of, &parts(), index),
                        Crossing::new(field.ty.clone(), Assembly::Construct),
                        Ask::Recipe(parts()),
                        Origin::Part,
                    ),
                    // The field is an `Option<D>`. That crossing keeps the row
                    // the registry derived from its shape — it has no `parts`
                    // row of its own — and it is D, one layer in, that takes
                    // `parts`.
                    Some(_) => bound.bind(
                        Site::part(
                            &Crossing::new(field.ty.clone(), Assembly::Construct),
                            &RecipeId::derived(),
                            0,
                        ),
                        Crossing::new(target.clone(), Assembly::Construct),
                        Ask::Recipe(parts()),
                        Origin::Part,
                    ),
                };
            }
        }
        bound.build(recipes)
    }
}
