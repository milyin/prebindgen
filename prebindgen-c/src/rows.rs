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
    flat::{Flat, Type},
    recipe::{
        Construct, Constructing, Deconstruct, Deconstructing, Reach, RecipeError, RecipeId, Recipes,
    },
};

use super::*;

/// The row a type with no parts takes: the adapter emits the conversion itself.
fn whole() -> RecipeId {
    RecipeId::new("whole")
}

/// The row a type crossing field by field takes.
pub(crate) fn parts() -> RecipeId {
    RecipeId::new("parts")
}

/// The row a value takes **inside a `data_struct`'s mirror**, where its wire
/// differs from the one it takes on its own.
///
/// One crossing read two ways is two rows, and the site picks — which is what
/// `declare_default` and `Ask::Recipe` exist for. Two types need it.
///
/// `bool`: a field arrives from C by value and may hold any byte until it is
/// normalised, so it crosses as `MaybeUninit<bool>`, while a `bool` returned
/// from Rust is already one of two values and passes through.
///
/// `String`: a field decodes a null pointer to an empty string, where a
/// `String` parameter refuses one. Both readings were in the hand-written field
/// walk; the row is what makes the difference visible.
pub(crate) fn in_field() -> RecipeId {
    RecipeId::new("field")
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
        let declared_keys: std::collections::HashSet<TypeKey> =
            declared.iter().map(|(k, _)| k.clone()).collect();

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
            // A `data_struct` converts each field and reassembles the
            // converted fields into one C struct — many parts, one wire value,
            // which is the pair #450 keeps apart. Everything else declared here
            // is one wire value with nothing inside it that crosses on its own:
            // an opaque handle, a value-opaque mirror, a C enum. A tagged union
            // plainly has arms and is still `Atomic`, because `in_tagged_union`
            // walks an arm's payload inside one generated function; stating
            // those arms is the next stage.
            if let Some(count) = self
                .data
                .contains_key(&_key)
                .then(|| field_count(model, &_key))
            {
                let Some(count) = count else { continue };
                let reaches: Vec<Reach> = (0..count).map(Reach::Field).collect();
                rows.declare(
                    ty.clone(),
                    parts(),
                    Deconstructing::Product(Deconstruct::Fields(reaches)),
                )
                .declare(ty, parts(), Constructing::Product(Construct::Fields));
                continue;
            }
            rows.declare(ty.clone(), whole(), Deconstructing::Atomic)
                .declare(ty, whole(), Constructing::Atomic);
        }
        // `bool`'s second row. Declared for every binding rather than only
        // where a struct has such a field: a row for a crossing nobody uses is
        // inert, and the alternative is walking every declared struct twice to
        // find out.
        let boolean = model
            .classify(&syn::parse_quote!(bool))
            .expect("bool is a scalar the model always classifies");
        rows.declare_default(boolean.clone(), whole(), Deconstructing::Atomic)
            .declare(boolean.clone(), in_field(), Deconstructing::Atomic)
            .declare_default(boolean.clone(), whole(), Constructing::Atomic)
            .declare(boolean, in_field(), Constructing::Atomic);
        // `String`'s second row, unless the binding already declared one for it
        // — a `convert!(String => ..)` states the whole-value reading itself,
        // and the loop above has already filed it.
        if let Ok(string) = model.classify(&syn::parse_quote!(String)) {
            if !declared_keys.contains(&string.key()) {
                // Output only ever has one reading, so `String` gets a second
                // row in the constructing direction alone.
                rows.declare_default(string.clone(), whole(), Constructing::Atomic)
                    .declare(string, in_field(), Constructing::Atomic);
            }
        }

        rows.build(model)
    }

    /// Which row each part of a declared product takes, where it is not the
    /// part type's own default.
    ///
    /// Only `bool` inside a `data_struct` today — see [`in_field`].
    pub(crate) fn bindings(
        &self,
        model: &Flat,
        recipes: &Recipes,
    ) -> Result<prebindgen_registry::recipe::Bindings, Vec<RecipeError>> {
        // `recipe::Origin` is which declaration asked; `flat::Origin` is a
        // captured item's own syntax. Both are in play here, so the one that
        // arrives by glob keeps its name.
        use prebindgen_registry::recipe::{
            Ask, Assembly, Bindings, Crossing, Origin as Asked, Site,
        };

        let mut bound = Bindings::builder();
        let mut declared: Vec<(TypeKey, Origin<syn::Type>)> = self
            .data
            .iter()
            .map(|(k, c)| (k.clone(), c.rust_type.clone()))
            .collect();
        declared.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        for (key, origin) in declared {
            let Ok(spelled) = syn::parse2::<syn::Type>(origin.declared_spelling()) else {
                continue;
            };
            let Ok(ty) = model.classify(&spelled) else {
                continue;
            };
            let Some(fields) = struct_fields(model, &key) else {
                continue;
            };
            for (index, field) in fields.iter().enumerate() {
                let kind = field.ty.unwrapped().kind();
                let is_bool = matches!(
                    kind,
                    prebindgen_registry::flat::TypeKind::Scalar(
                        prebindgen_registry::flat::ScalarKind::Bool
                    )
                );
                let is_string = matches!(kind, prebindgen_registry::flat::TypeKind::String);
                if !is_bool && !is_string {
                    continue;
                }
                // A `String` reads differently only on the way in.
                let directions: &[Assembly] = if is_bool {
                    &[Assembly::Construct, Assembly::Deconstruct]
                } else {
                    &[Assembly::Construct]
                };
                for &assembly in directions {
                    let of = Crossing::new(ty.clone(), assembly);
                    bound.bind(
                        Site::part(&of, &parts(), index),
                        Crossing::new(field.ty.clone(), assembly),
                        Ask::Recipe(in_field()),
                        Asked::Part,
                    );
                }
            }
        }
        bound.build(recipes)
    }
}

/// The fields the model gives this declared struct.
fn struct_fields(model: &Flat, key: &TypeKey) -> Option<Vec<prebindgen_registry::flat::Field>> {
    match model.declared_type(&key.ident()?)? {
        Type::Struct(s) => Some(s.fields.clone()),
        _ => None,
    }
}

/// How many fields the model gives this declared struct.
fn field_count(model: &Flat, key: &TypeKey) -> Option<usize> {
    Some(struct_fields(model, key)?.len())
}
