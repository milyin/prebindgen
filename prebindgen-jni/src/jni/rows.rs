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
    recipe::{Arm, Construct, Constructing, Deconstructing, RecipeError, RecipeId, Recipes},
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

/// One arm per alternative of a declared sum, each assembled from its own
/// payload fields.
///
/// Empty for anything the model does not hold as a data-carrying enum, which
/// includes a `sealed_class!` declared over a type the scan dropped — the scan
/// reports that, and a row over no arms would report it a second time.
fn alternatives(model: &Flat, ty: &TypeRef) -> Vec<Arm<Construct>> {
    let prebindgen_registry::flat::TypeKind::Named { id, .. } = ty.unwrapped().kind() else {
        return Vec::new();
    };
    let Some(prebindgen_registry::flat::Type::Variant(v)) =
        id.ident().and_then(|ident| model.declared_type(&ident))
    else {
        return Vec::new();
    };
    v.alternatives
        .iter()
        .map(|alt| Arm {
            alternative: alt.index,
            op: Construct::Fields,
        })
        .collect()
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
            match self.types.get(&ty.key()).map(|c| &c.kind) {
                Some(DeclaredKind::Data) => {
                    rows.declare_default(ty.clone(), whole(), Constructing::Atomic)
                        .declare(ty, parts(), Constructing::Product(Construct::Fields));
                }
                // A `sealed_class` has one too, and it is a choice rather than
                // a product: exactly one alternative is live, every one of them
                // still crosses, and the tag says which. The arms are the
                // model's — a row states which parts, never what they are.
                Some(DeclaredKind::Sealed(_)) => {
                    let arms = alternatives(model, &ty);
                    rows.declare_default(ty.clone(), whole(), Constructing::Atomic);
                    if !arms.is_empty() {
                        rows.declare(ty, parts(), Constructing::Choice { arms });
                    }
                }
                _ => {
                    rows.declare(ty, whole(), Constructing::Atomic);
                }
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
        // The field's own key, not the stripped one: a `Box<Leaf>` field
        // crosses as one value through `Box<Leaf>`'s conversion, and only a
        // parameter spelled that way is peeled back to the class. Binding the
        // stripped key here would flatten a field the rebuild has no node for.
        self.types.get(&ty.key()).is_some_and(|c| match c.kind {
            DeclaredKind::Data => !c.jobject_input,
            // A `sealed_class` field crosses as a tag plus every alternative's
            // slots. Whether it *can* is the adapter's answer at compile time,
            // not a declaration: a payload with no slot form leaves the sum
            // object-shaped, which is the fragment `Compile::choice` hands
            // back. So the site asks, and the row answers.
            DeclaredKind::Sealed(_) => true,
            _ => false,
        })
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

        // Every optional over a flattenable value, wherever the model spells
        // one: a parameter, a field, a callback argument. A `Site` keys a part
        // by the crossing's **stripped** key, so `Option<Payload>` and
        // `Box<Option<Payload>>` are one site and binding it once answers for
        // both spellings.
        //
        // Enumerated from the model rather than from the declarations, for the
        // same reason the array rows are: what has to be bound is what the
        // model names, and a declaration says nothing about where its type is
        // used.
        let mut optionals: BTreeMap<TypeKey, (&TypeRef, &TypeRef)> = BTreeMap::new();
        for ty in model
            .elements()
            .flat_map(element_types)
            .flat_map(|t| t.walk())
        {
            let Some(inner) = ty.optional_inner() else {
                continue;
            };
            if self.field_crosses_as_its_fields(inner) {
                optionals.entry(ty.stripped_key()).or_insert((ty, inner));
            }
        }
        for (outer, inner) in optionals.into_values() {
            // The optional keeps the row the registry derived from its shape —
            // it has no `parts` row of its own — and it is the value one layer
            // in that crosses as its parts.
            bound.bind(
                Site::part(
                    &Crossing::new(outer.clone(), Assembly::Construct),
                    &RecipeId::derived(),
                    0,
                ),
                Crossing::new(inner.clone(), Assembly::Construct),
                Ask::Recipe(parts()),
                Origin::Part,
            );
        }

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
                // An `Option<D>` field reaches D through the optional's own
                // part site, which the model-wide scan above already bound. What
                // is left is the field that IS the class: its part takes the
                // `parts` row.
                if field.ty.optional_inner().is_none() {
                    bound.bind(
                        Site::part(&of, &parts(), index),
                        Crossing::new(field.ty.clone(), Assembly::Construct),
                        Ask::Recipe(parts()),
                        Origin::Part,
                    );
                }
            }
        }
        bound.build(recipes)
    }
}
