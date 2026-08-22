//! Cbindgen's per-type policies, lowered into the registry's recipe vocabulary.
//!
//! A build script writes `.opaque_ptr()`, `.data_struct()`, `.enum_type()`,
//! `.tagged_union()` or `.repr_c_struct()`, each saying what C shape one Rust
//! type takes. This turns those into recipes in a [`Recipes`] table: the shared
//! statement of **which parts** a value gets across in, with nothing about the
//! C wire in it.
//!
//! There is one recipe per declared type and direction, and every crossing nobody
//! declared takes the recipe the registry derives from its kind. So the chain of
//! `or_else` guesses this replaced — try a custom conversion, then a handle,
//! then a data struct, then an enum, … — is now a lookup, and the answer is
//! whatever the build script said rather than whichever guess fired first.

use prebindgen_registry::{
    flat::{Flat, Type, TypeRef},
    recipe::{
        Arm, Construct, Constructing, Deconstruct, Deconstructing, Direction, Reach, RecipeError,
        RecipeId, Recipes,
    },
};

use super::*;

/// The recipe a type with no parts takes: the adapter emits the conversion itself.
fn whole() -> RecipeId {
    RecipeId::new("whole")
}

/// The recipe a type crossing field by field takes.
pub(crate) fn parts() -> RecipeId {
    RecipeId::new("parts")
}

/// The recipe a value takes as a **tagged union's payload**, where it rides
/// differently from both of the readings below.
///
/// A `Box<T>` or `Option<Box<T>>` over a declared `opaque_ptr` rides in the
/// union as a bare `*mut t_t` the C side owns. That is neither the whole-value
/// reading — a handle parameter is spelled `T` and reclaimed from its own
/// pointer — nor a struct field's.
///
/// The recipe is filed on the **stripped** crossing, because `Crossing::key` peels
/// `Box`: `Box<Blob>` and `Blob` share one crossing and are told apart by the
/// site that picks between their recipes, and by the fragment, which is keyed by
/// the spelling.
pub(crate) fn payload() -> RecipeId {
    RecipeId::new("payload")
}

/// The recipe a value takes **inside a `data_struct`'s mirror**, where its wire
/// differs from the one it takes on its own.
///
/// One crossing read two ways is two recipes, and the site picks — which is what
/// `declare_default` and `Ask::Recipe` exist for. Two types need it.
///
/// `bool`: a field arrives from C by value and may hold any byte until it is
/// normalised, so it crosses as `MaybeUninit<bool>`, while a `bool` returned
/// from Rust is already one of two values and passes through.
///
/// `String`: a field decodes a null pointer to an empty string, where a
/// `String` parameter refuses one. Both readings were in the hand-written field
/// walk; the recipe is what makes the difference visible.
pub(crate) fn in_field() -> RecipeId {
    RecipeId::new("field")
}

impl CbindgenBuilder {
    /// Every recipe this binding's declarations state.
    ///
    /// A type declared but absent from the model is skipped rather than
    /// refused: the scan already reports it, and reporting it twice in
    /// different words helps nobody.
    pub(crate) fn recipes(&self, model: &Flat) -> Result<Recipes, Vec<RecipeError>> {
        let mut recipes = Recipes::builder();
        // The crossings a second recipe will also land on, so the loop below knows
        // to make the whole-value one the default rather than leaving two recipes
        // with no answer between them.
        //
        // Two sources. A payload recipe shares a crossing with its handle's own,
        // because `Crossing::key` peels `Box` — right for lookup, and it means
        // `Blob` and `Box<Blob>` are one crossing. And `bool` and `String` each
        // get a field recipe unconditionally, which collides with a whole-value
        // recipe a binding declared itself.
        let mut shared: std::collections::HashSet<TypeKey> = self
            .boxed_payloads(model)
            .iter()
            .map(|t| t.borrow_target().unwrap_or(t).stripped_key())
            .collect();
        for builtin in [syn::parse_quote!(bool), syn::parse_quote!(String)] {
            if let Ok(ty) = model.classify(&builtin) {
                shared.insert(ty.key());
            }
        }
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
            // A tagged union is a tag plus a union, rebuilt one arm at a time.
            // Neither the tag nor the selector is a crossing, so neither is
            // declared: how the C side is told which arm is live is the
            // adapter's business.
            if self.tagged_unions.contains_key(&_key) {
                let Some(alternatives) = alternative_field_counts(model, &_key) else {
                    continue;
                };
                let out: Vec<Arm<Deconstruct>> = alternatives
                    .iter()
                    .enumerate()
                    .map(|(alternative, count)| Arm {
                        alternative,
                        op: Deconstruct::Fields((0..*count).map(Reach::Field).collect()),
                    })
                    .collect();
                let into: Vec<Arm<Construct>> = alternatives
                    .iter()
                    .enumerate()
                    .map(|(alternative, _)| Arm {
                        alternative,
                        op: Construct::Fields,
                    })
                    .collect();
                recipes
                    .declare(ty.clone(), parts(), Deconstructing::Choice { arms: out })
                    .declare(ty, parts(), Constructing::Choice { arms: into });
                continue;
            }
            // A `data_struct` converts each field and reassembles the
            // converted fields into one C struct — many parts, one wire value,
            // which is the pair #450 keeps apart. Everything else declared here
            // is one wire value with nothing inside it that crosses on its own:
            // an opaque handle, a value-opaque mirror, a C enum.
            if let Some(count) = self
                .data
                .contains_key(&_key)
                .then(|| field_count(model, &_key))
            {
                let Some(count) = count else { continue };
                let reaches: Vec<Reach> = (0..count).map(Reach::Field).collect();
                recipes
                    .declare(
                        ty.clone(),
                        parts(),
                        Deconstructing::Product(Deconstruct::Fields(reaches)),
                    )
                    .declare(ty, parts(), Constructing::Product(Construct::Fields));
                continue;
            }
            if shared.contains(&ty.borrow_target().unwrap_or(&ty).stripped_key()) {
                recipes
                    .declare_default(ty.clone(), whole(), Deconstructing::Atomic)
                    .declare_default(ty, whole(), Constructing::Atomic);
            } else {
                recipes
                    .declare(ty.clone(), whole(), Deconstructing::Atomic)
                    .declare(ty, whole(), Constructing::Atomic);
            }
        }
        // The payload recipes, on the stripped crossing each `Box`-over-handle
        // shares with its own handle type. Only the types a union arm actually
        // carries.
        for ty in self.boxed_payloads(model) {
            // The handle's own recipe is already filed on this crossing, so one of
            // the two has to say which a site gets when it names neither.
            recipes
                .declare(ty.clone(), payload(), Deconstructing::Atomic)
                .declare(ty, payload(), Constructing::Atomic);
        }

        // The field recipes. Declared for every binding rather than only where a
        // struct has such a field: a recipe for a crossing nobody uses is inert,
        // and the alternative is walking every declared struct twice to find
        // out.
        //
        // **Always**, whether or not the type's whole-value recipe was declared
        // above. The two are independent readings — the hand-written walks kept
        // them so — and a binding may declare the whole-value one itself:
        // `convert!` refuses a builtin, but `opaque_ptr(String)` is accepted and
        // `out_terminal` has an arm for it. Suppressing the field recipe there
        // stranded every string field, because `bindings` asks each one for it.
        let boolean = model
            .classify(&syn::parse_quote!(bool))
            .expect("bool is a scalar the model always classifies");
        // A whole-value recipe the loop already filed stays as it is and only has
        // to become the default; otherwise it is declared here.
        if !declared_keys.contains(&boolean.key()) {
            recipes
                .declare_default(boolean.clone(), whole(), Deconstructing::Atomic)
                .declare_default(boolean.clone(), whole(), Constructing::Atomic);
        }
        recipes
            .declare(boolean.clone(), in_field(), Deconstructing::Atomic)
            .declare(boolean, in_field(), Constructing::Atomic);
        if let Ok(string) = model.classify(&syn::parse_quote!(String)) {
            // Output only ever has one reading, so `String` gets a second recipe in
            // the constructing direction alone.
            if !declared_keys.contains(&string.key()) {
                recipes.declare_default(string.clone(), whole(), Constructing::Atomic);
            }
            recipes.declare(string, in_field(), Constructing::Atomic);
        }

        recipes.build(model)
    }

    /// Which recipe a union payload of this type takes, and in which directions,
    /// where it is not the type's own default.
    ///
    /// Every `Box`-over-handle a union arm carries, keyed once.
    fn boxed_payloads(&self, model: &Flat) -> Vec<TypeRef> {
        let mut payloads: Vec<TypeRef> = Vec::new();
        for (key, _) in sorted_by_key(&self.tagged_unions) {
            for fields in alternatives_of(model, key).unwrap_or_default() {
                for field in fields {
                    if self.declared_opaque_payload_inner(&field.ty).is_some()
                        || r_boxed_inner(&field.ty).is_some()
                    {
                        payloads.push(field.ty.clone());
                    }
                }
            }
        }
        payloads.sort_by_key(|t| t.key().as_str().to_owned());
        payloads.dedup_by_key(|t| t.key().as_str().to_owned());
        payloads
    }

    /// `bool` and `String` read as they do inside a struct; a `Box`-over-handle
    /// needs [`payload`], the one reading neither of the other two covers.
    fn payload_reading(&self, fty: &TypeRef) -> Option<(RecipeId, &'static [Direction])> {
        use prebindgen_registry::flat::{ScalarKind, TypeKind};
        if self.declared_opaque_payload_inner(fty).is_some() || r_boxed_inner(fty).is_some() {
            return Some((payload(), &[Direction::Construct, Direction::Deconstruct]));
        }
        match fty.unwrapped().kind() {
            TypeKind::Scalar(ScalarKind::Bool) => {
                Some((in_field(), &[Direction::Construct, Direction::Deconstruct]))
            }
            TypeKind::String => Some((in_field(), &[Direction::Construct])),
            _ => None,
        }
    }

    /// Which recipe each part of a declared product takes, where it is not the
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
        use prebindgen_registry::recipe::{Ask, Bindings, Crossing, Origin as Asked, Site};

        let mut bound = Bindings::builder();
        // A union's payload reads like a struct's field for `bool` and
        // `String`, and needs a reading of its own where it is a `Box` over a
        // declared handle — see [`payload`].
        for (key, cfg) in sorted_by_key(&self.tagged_unions) {
            let Ok(spelled) = syn::parse2::<syn::Type>(cfg.rust_type.declared_spelling()) else {
                continue;
            };
            let Ok(ty) = model.classify(&spelled) else {
                continue;
            };
            let Some(alternatives) = alternatives_of(model, key) else {
                continue;
            };
            // Every arm numbers its parts from zero, so a payload is addressed
            // by its alternative as well as its index — `part 0` alone names
            // one part per arm.
            for (arm, fields) in alternatives.iter().enumerate() {
                for (index, field) in fields.iter().enumerate() {
                    let Some((recipe, directions)) = self.payload_reading(&field.ty) else {
                        continue;
                    };
                    for &direction in directions {
                        let of = Crossing::new(ty.clone(), direction);
                        bound.bind(
                            Site::arm_part(&of, &parts(), Some(arm), index),
                            Crossing::new(field.ty.clone(), direction),
                            Ask::Recipe(recipe.clone()),
                            Asked::Part,
                        );
                    }
                }
            }
        }
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
                let directions: &[Direction] = if is_bool {
                    &[Direction::Construct, Direction::Deconstruct]
                } else {
                    &[Direction::Construct]
                };
                for &direction in directions {
                    let of = Crossing::new(ty.clone(), direction);
                    bound.bind(
                        Site::part(&of, &parts(), index),
                        Crossing::new(field.ty.clone(), direction),
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

/// The alternatives of this declared sum, with their payload fields.
fn alternatives_of(
    model: &Flat,
    key: &TypeKey,
) -> Option<Vec<Vec<prebindgen_registry::flat::Field>>> {
    match model.declared_type(&key.ident()?)? {
        Type::Variant(v) => Some(v.alternatives.iter().map(|a| a.fields.clone()).collect()),
        _ => None,
    }
}

/// How many fields each alternative of this declared sum carries.
fn alternative_field_counts(model: &Flat, key: &TypeKey) -> Option<Vec<usize>> {
    match model.declared_type(&key.ident()?)? {
        Type::Variant(v) => Some(v.alternatives.iter().map(|a| a.fields.len()).collect()),
        _ => None,
    }
}

/// How many fields the model gives this declared struct.
fn field_count(model: &Flat, key: &TypeKey) -> Option<usize> {
    Some(struct_fields(model, key)?.len())
}
