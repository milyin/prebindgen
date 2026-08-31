//! JniGen's Kotlin class declarations, lowered into the registry's recipe
//! vocabulary.
//!
//! A build script writes `ptr_class!`, `data_class!`, `enum_class!` or
//! `sealed_class!`, each naming the Kotlin type one Rust type becomes, and
//! `convert!` for a conversion the binding supplies itself. This turns those
//! into recipes in a [`Recipes`] table: the shared statement of **which parts** a
//! value gets across in, with nothing about the JNI wire in it.
//!
//! One recipe per declared type and direction, and every crossing nobody declared
//! takes
//! the recipe the registry derives from its kind. The chain of guesses this
//! replaced — try a terminal, then a `Result`, then an optional, then a run,
//! then a borrow, then a transparent bridge — is a lookup for the declared
//! half, and the derived half is the registry's.

use std::collections::BTreeMap;

use prebindgen_registry::{
    flat::{Flat, TypeRef},
    recipe::{
        Arm, Construct, Constructing, Deconstruct, Deconstructing, Reach, RecipeError, RecipeName,
        Recipes,
    },
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
/// reports that, and a recipe over no arms would report it a second time.
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

/// One arm per alternative of a declared sum, each taken apart into its own
/// payload fields.
///
/// The deconstructing twin of [`alternatives`]. Separate because the two directions
/// name different operations — building an alternative is `Construct::Fields`,
/// reading one is `Deconstruct::Fields` over the reaches that get there — and
/// a shared helper would have to be generic over an operation neither side
/// chooses.
fn out_alternatives(model: &Flat, ty: &TypeRef) -> Vec<Arm<Deconstruct>> {
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
            op: Deconstruct::Fields((0..alt.fields.len()).map(Reach::Field).collect()),
        })
        .collect()
}

impl Declarations {
    /// How many values this type hands out, or 0 if it hands out none.
    ///
    /// Model and declaration only, like the compositions it asks — so a
    /// declaration-time caller may ask it, which is what binding a return site
    /// needs.
    fn out_wire_count(
        &self,
        registry: &impl prebindgen_registry::Conversions,
        ty: &TypeRef,
    ) -> usize {
        let prebindgen_registry::flat::TypeKind::Named { id, .. } = ty.unwrapped().kind() else {
            return 0;
        };
        let Some(ident) = id.ident() else { return 0 };
        match registry.flat().declared_type(&ident) {
            Some(prebindgen_registry::flat::Type::Variant(_)) => self
                .sum_out_wires(registry, &ident, ty)
                .map_or(0, |w| w.len()),
            Some(prebindgen_registry::flat::Type::Struct(_)) => {
                self.struct_out_wires(registry, ty).map_or(0, |w| w.len())
            }
            _ => 0,
        }
    }

    /// The accessor a type's value form names, and one reach per field of the
    /// struct it returns.
    ///
    /// `None` unless the declaration is exactly one `.fields(fields!(f))` whose
    /// every field is a plain leaf. A record that states its own decomposition
    /// — a per-field `expand_return!` override, an inlined nested class, a sum
    /// laid out as a selector and its groups — is a shape this recipe does not
    /// state yet, and stating half of one would describe a boundary nothing
    /// emits.
    fn value_form_of(
        &self,
        model: &Flat,
        registry: &impl prebindgen_registry::Conversions,
        ty: &TypeRef,
    ) -> Option<(syn::Ident, Vec<Reach>)> {
        use prebindgen_registry::unfold::{FieldDecon, FieldRecord};
        let decl = self
            .return_expand_decls
            .iter()
            .find(|d| *d.key() == ty.stripped_key())?;
        let [crate::jni::LocalField::Fields(fields)] = decl.field_list() else {
            return None;
        };
        let records: Vec<FieldRecord> = self.lower_value_form(registry, decl.key(), fields);
        let ret = model.function(fields.func())?.ret.clone();
        let ret = ret.borrow_target().unwrap_or(&ret);
        let prebindgen_registry::flat::TypeKind::Named { id, .. } = ret.unwrapped().kind() else {
            return None;
        };
        let prebindgen_registry::flat::Type::Struct(st) =
            id.ident().and_then(|i| model.declared_type(&i))?
        else {
            return None;
        };
        let mut reaches = Vec::new();
        for record in &records {
            if !matches!(record.decon, FieldDecon::Default) {
                return None;
            }
            let [member] = record.members.as_slice() else {
                return None;
            };
            let index = st
                .fields
                .iter()
                .position(|f| f.name.as_ref() == Some(member))?;
            // A field that reaches the type being taken apart. The leaf
            // synthesis treats `Box<T>` as a spelling of its own and stops
            // there; a recipe keys a crossing by the value that crosses, so
            // `Box<ZSample>` inside `ZSample`'s own value form is that recipe
            // reaching itself. Refused here rather than by `Recipes::build`,
            // which would refuse the whole binding over a shape the leaf
            // synthesis handles.
            if st.fields[index].ty.stripped_key() == ty.stripped_key() {
                return None;
            }
            reaches.push(Reach::Field(index));
        }
        Some((fields.func().clone(), reaches))
    }
}

impl Declarations {
    /// What a value form calls each of its fields, by the Rust field ident.
    ///
    /// The declaration's answer, so a `.name(..)` rename carries through to the
    /// builder parameter. `None` for a type with no value form, or one whose
    /// records this recipe does not state — the same test [`Self::value_form_of`]
    /// makes when it declares the recipe.
    pub(crate) fn value_form_names(
        &self,
        registry: &impl prebindgen_registry::Conversions,
        ty: &TypeRef,
    ) -> Option<std::collections::HashMap<String, String>> {
        use prebindgen_registry::unfold::FieldDecon;
        let decl = self
            .return_expand_decls
            .iter()
            .find(|d| *d.key() == ty.stripped_key())?;
        let [crate::jni::LocalField::Fields(fields)] = decl.field_list() else {
            return None;
        };
        self.lower_value_form(registry, decl.key(), fields)
            .into_iter()
            .map(|r| match (r.members.as_slice(), &r.decon) {
                ([member], FieldDecon::Default) => Some((member.to_string(), r.name)),
                _ => None,
            })
            .collect()
    }
}

/// One reach per field of a declared struct, in the model's order.
///
/// Empty for anything the model does not hold as a struct, which includes a
/// `data_class!` over a type the scan dropped — the scan reports that, and a
/// recipe over no fields would report it a second time.
fn out_fields(model: &Flat, ty: &TypeRef) -> Vec<Reach> {
    (0..fields_of(model, ty).len()).map(Reach::Field).collect()
}

/// The fields of a declared struct, or none for anything the model does not
/// hold as one.
fn fields_of<'a>(model: &'a Flat, ty: &TypeRef) -> &'a [prebindgen_registry::flat::Field] {
    let prebindgen_registry::flat::TypeKind::Named { id, .. } = ty.unwrapped().kind() else {
        return &[];
    };
    match id.ident().and_then(|ident| model.declared_type(&ident)) {
        Some(prebindgen_registry::flat::Type::Struct(s)) => &s.fields,
        _ => &[],
    }
}

/// The recipe a type with no parts takes: the adapter emits the conversion itself.
fn whole() -> RecipeName {
    RecipeName::new("whole")
}

/// The recipe a `data_class` takes when it crosses **as its fields**.
///
/// Not the default yet. `whole()` still answers for every site, and this one is
/// compiled only where something asks for it by name — which is what lets the
/// composed wire list be checked against the walk it will replace before
/// anything depends on it.
pub(crate) fn parts() -> RecipeName {
    RecipeName::new("parts")
}

/// The allocation-free `(present, value)` input row for an Optional whose
/// destination payload is a JNI primitive.
pub(crate) fn pair() -> RecipeName {
    RecipeName::new("pair")
}

impl Declarations {
    /// Every recipe this binding's declarations state.
    ///
    /// A type declared but absent from the model is skipped rather than
    /// refused: the scan already reports it, and reporting it twice in
    /// different words helps nobody.
    /// This type's `parts` decomposition, read off its `expand_return!`
    /// declaration in the recipe table's own vocabulary.
    ///
    /// `None` when the declaration states a shape a `Deconstruct` cannot hold —
    /// a value-form record mixed with others, since `ValueForm` is the whole
    /// row while `LocalField::Fields` is one record among many. No declaration
    /// in this workspace has that shape (measured: every `Fields` record stands
    /// alone), so the caller's `Atomic` fallback is a guard rather than a path
    /// anything takes.
    fn parts_deconstruct(&self, ty: &TypeRef) -> Option<Deconstruct> {
        let decl = self
            .return_expand_decls
            .iter()
            .find(|d| *d.key() == ty.stripped_key())?;
        let fields = decl.field_list();
        if let [crate::jni::LocalField::Fields(value_form)] = fields {
            return Some(Deconstruct::ValueForm {
                func: value_form.func().clone(),
                parts: Vec::new(),
            });
        }
        let mut reaches = Vec::with_capacity(fields.len());
        for field in fields {
            reaches.push(match field {
                crate::jni::LocalField::Named(func, _) => Reach::Accessor(func.clone()),
                // The handle itself — the leaf `Reach::Identity` was added for.
                crate::jni::LocalField::SelfField => Reach::Identity,
                // A binding-local accessor resolves by its path's last segment:
                // `local_functions()` registers it under exactly that ident.
                crate::jni::LocalField::Local { path, .. } => {
                    Reach::Accessor(path.segments.last()?.ident.clone())
                }
                crate::jni::LocalField::Fields(_) => return None,
            });
        }
        Some(Deconstruct::Fields(reaches))
    }

    pub(crate) fn recipes(
        &self,
        model: &Flat,
        expansion_leaves: &[TypeRef],
        registry: &impl prebindgen_registry::Conversions,
    ) -> Result<Recipes, Vec<RecipeError>> {
        let mut recipes = Recipes::builder();
        // Which crossings already state a deconstructing `parts` row, so the
        // decomposition block below adds one only where none was declared.
        let mut parts_out: std::collections::HashSet<TypeKey> = std::collections::HashSet::new();
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
            // Every declared Kotlin shape is one recipe with no parts today,
            // including the two that plainly have some: a `data_class` arrives
            // as separate JNI parameters and is rebuilt inside one generated
            // function, and a `sealed_class` is a selector plus the live arm's
            // slots inside another. `Atomic` is what a recipe says about that —
            // the adapter emits the conversion itself, and how many wire values
            // that costs is its own business — so it describes the adapter as it
            // stands rather than as it will be.
            // A `sealed_class` hands its value out as a tag plus every
            // alternative's payloads, laid side by side — the deconstructing
            // twin of the constructing recipe below, and the direction that
            // actually has no alternative: a sum has no whole-value output
            // conversion, because a tag plus groups is not one wire.
            match self.types.get(&ty.key()).map(|c| &c.kind) {
                Some(DeclaredKind::Sealed(_)) => {
                    let arms = out_alternatives(model, &ty);
                    recipes.declare_default(ty.clone(), whole(), Deconstructing::Atomic);
                    if !arms.is_empty() {
                        recipes.declare(ty.clone(), parts(), Deconstructing::Choice { arms });
                        parts_out.insert(ty.stripped_key());
                    }
                }
                // A `data_class` hands its value out as its fields too, so the
                // foreign side reassembles the object and nothing builds one on
                // the Rust side.
                Some(DeclaredKind::Data) => {
                    let reaches = out_fields(model, &ty);
                    recipes.declare_default(ty.clone(), whole(), Deconstructing::Atomic);
                    if !reaches.is_empty() {
                        recipes.declare(
                            ty.clone(),
                            parts(),
                            Deconstructing::Product(Deconstruct::Fields(reaches)),
                        );
                        parts_out.insert(ty.stripped_key());
                    }
                }
                _ => {
                    recipes.declare_default(ty.clone(), whole(), Deconstructing::Atomic);
                }
            }
            // A value form: `expand_return!(T).fields(fields!(f))` calls `f`
            // once and hands out the fields of the struct it returns. The one
            // declaration shape that names its own accessor, and the reason
            // `Deconstruct::ValueForm` exists.
            if let Some((func, reaches)) = self.value_form_of(model, registry, &ty) {
                recipes.declare(
                    ty.clone(),
                    parts(),
                    Deconstructing::Product(Deconstruct::ValueForm {
                        func,
                        parts: reaches,
                    }),
                );
                parts_out.insert(ty.stripped_key());
            }
            // A `data_class` also has a recipe that says what it is made of, so
            // its constructing side names which of the two a site takes by
            // default. See [`parts`] for why that is still `whole`.
            match self.types.get(&ty.key()).map(|c| &c.kind) {
                // A struct with no fields is nothing to be made of, so it
                // states no `parts` recipe — the same condition the deconstructing
                // side applies, and the reason `recipe_of` may be asked for the
                // recipe by name.
                Some(DeclaredKind::Data) if !fields_of(model, &ty).is_empty() => {
                    recipes
                        .declare_default(ty.clone(), whole(), Constructing::Atomic)
                        .declare(ty, parts(), Constructing::Product(Construct::Fields));
                }
                // A `sealed_class` has one too, and it is a choice rather than
                // a product: exactly one alternative is live, every one of them
                // still crosses, and the tag says which. The arms are the
                // model's — a recipe states which parts, never what they are.
                Some(DeclaredKind::Sealed(_)) => {
                    let arms = alternatives(model, &ty);
                    recipes.declare_default(ty.clone(), whole(), Constructing::Atomic);
                    if !arms.is_empty() {
                        recipes.declare(ty, parts(), Constructing::Choice { arms });
                    }
                }
                _ => {
                    recipes.declare(ty, whole(), Constructing::Atomic);
                }
            }
        }

        // A type a callback delivers by taking it apart states that as a row,
        // so the argument's site has a recipe to name. Without one the site
        // could only be fabricated — a `Bound` no binding answered — which is
        // what #622's first two attempts produced and why they were withdrawn.
        //
        // `Atomic` for the reason the declared-type loop above gives for the
        // two shapes that plainly have parts: the adapter emits this
        // conversion itself, and how many wire values that costs is its own
        // business. The parts are stated by the deconstructor declaration the
        // registry already resolved into an `UnfoldPlan`, which this row does
        // not restate — it names the crossing that plan decomposes. Deleting
        // that second statement is #613 step 5b's, not this row's.
        //
        // The source is the plan's own owned core, so a plan keyed under `&T`
        // and one keyed under `T` state one row; a type that already declared
        // `parts` — a `data_class`, a `sealed_class`, a value form — keeps it.
        //
        // A **whole-element fold** is not one of these and must not earn a row.
        // `apply_leaf_vec_folds` files a plan for `impl Fn(&[T])` under `&[T]`
        // whose `source` is the ELEMENT and whose `decon` is `None`: nothing is
        // taken apart, each element crosses whole through its own converter.
        // Declaring `parts` off it would state a decomposition of `T` that does
        // not exist — and, worse, a scalar `T` argument elsewhere in the same
        // model would then bind to that row and name a fragment compiled under
        // a different recipe (#623 review). `decon` is the gate the model
        // already carries for this: `None` only for the whole-element arm.
        let mut decomposed: BTreeMap<TypeKey, TypeRef> = BTreeMap::new();
        for plan in registry.callback_arg_plans().values() {
            if plan.decon.is_none() {
                continue;
            }
            let key = plan.source.stripped_key();
            if parts_out.contains(&key) {
                continue;
            }
            decomposed.entry(key).or_insert_with(|| plan.source.clone());
        }
        for ty in decomposed.into_values() {
            // A real row where the declaration can state one. #622 wrote
            // `Atomic` here because `Reach` could not spell an identity leaf;
            // it can now (#635), so the row says how the value comes apart
            // instead of only existing to be selected (#613 step 10).
            match self.parts_deconstruct(&ty) {
                Some(deconstruct) => {
                    recipes.declare(ty, parts(), Deconstructing::Product(deconstruct));
                }
                None => {
                    recipes.declare(ty, parts(), Deconstructing::Atomic);
                }
            }
        }

        // Every implicit Optional keeps its established whole/default input
        // row and also offers the allocation-free primitive `pair` row. A
        // parameter site selects `pair` only after the compiled payload proves
        // that it is one JNI primitive with no niche, projection, or stage.
        //
        // Product-shaped payloads additionally offer `parts`; that row is
        // selected by the existing flattening bindings below.
        let mut implicit: BTreeMap<TypeKey, (TypeRef, TypeRef)> = self
            .implicit_optionals(model)
            .into_iter()
            .map(|(key, (outer, inner))| (key, (outer.clone(), inner.clone())))
            .collect();
        // A combined constructor expansion represents an inactive arm by
        // Option-wrapping each required leaf. Those readings are synthesized
        // after the Flat model was built, so a model walk cannot discover
        // them. File them in the same table as source-written implicit
        // Optionals: otherwise a nullable primitive leaf falls back to a boxed
        // JObject and Rust has to call `intValue()`/`longValue()` across JNI.
        for leaf in expansion_leaves {
            let Some(inner) = leaf.optional_inner() else {
                continue;
            };
            if !self.explicitly_declares(leaf) {
                implicit
                    .entry(leaf.stripped_key())
                    .or_insert_with(|| (leaf.clone(), inner.clone()));
            }
        }
        for (outer, _) in implicit.values() {
            recipes
                .declare_default(outer.clone(), whole(), Constructing::Optional)
                .declare(outer.clone(), pair(), Constructing::Optional)
                .declare(outer.clone(), parts(), Constructing::Optional);
        }
        for (outer, _) in implicit.into_values() {
            recipes
                .declare_default(outer.clone(), whole(), Deconstructing::Optional)
                .declare(outer.clone(), parts(), Deconstructing::Optional);
        }

        // A fixed-size array of JNI primitives is one Kotlin `ByteArray` or
        // `LongArray`, bulk-copied with nothing boxed — one wire value, not a
        // run this adapter walks. The registry reads `[T; N]` as a run unless
        // something says otherwise, and this is JniGen saying otherwise for
        // every array the model holds. Nothing is enumerated twice: a recipe for
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
            recipes
                .declare(ty.clone(), whole(), Deconstructing::Atomic)
                .declare(ty.clone(), whole(), Constructing::Atomic);
        }

        recipes.build(model)
    }
}

impl Declarations {
    /// Which recipe each part of a `data_class` takes.
    ///
    /// A field that is itself a `data_class` crosses as **its** fields too, so
    /// the part site asks for [`parts`] rather than letting the field take its
    /// own default. That is the one thing a site can say that a recipe cannot: the
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
            // back. So the site asks, and the recipe answers.
            DeclaredKind::Sealed(_) => true,
            _ => false,
        })
    }

    /// Optional crossings whose payload can use the shared `parts` recipe.
    ///
    /// Explicit declarations keep full control of the outer crossing: a
    /// `convert!(Option<T> => ..)` is an atomic recipe, not an implicit request
    /// to flatten `T`.
    fn implicit_optionals<'a>(
        &self,
        model: &'a Flat,
    ) -> BTreeMap<TypeKey, (&'a TypeRef, &'a TypeRef)> {
        let mut optionals = BTreeMap::new();
        for ty in model
            .elements()
            .flat_map(element_types)
            .flat_map(|ty| ty.walk())
        {
            let Some(inner) = ty.optional_inner() else {
                continue;
            };
            if !self.explicitly_declares(ty) {
                optionals.entry(ty.stripped_key()).or_insert((ty, inner));
            }
        }
        optionals
    }

    fn explicitly_declares(&self, ty: &TypeRef) -> bool {
        self.types.contains_key(&ty.key())
            || self
                .convert_decls
                .iter()
                .any(|decl| decl.key() == &ty.key() || decl.key() == &ty.stripped_key())
    }

    pub(crate) fn bindings(
        &self,
        model: &Flat,
        registry: &impl prebindgen_registry::Conversions,
        recipes: &prebindgen_registry::recipe::Recipes,
    ) -> Result<prebindgen_registry::recipe::Bindings, Vec<RecipeError>> {
        use prebindgen_registry::recipe::{
            Ask, Bindings, Crossing, Direction, Origin, RecipeName, Site,
        };

        let mut bound = Bindings::builder();
        let mut declared: Vec<TypeKey> = self
            .types
            .iter()
            .filter(|(_, c)| matches!(c.kind, DeclaredKind::Data))
            .map(|(k, _)| k.clone())
            .collect();
        declared.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        // The named Optional row is the representation suitable inside another
        // composed shape. Where its child has a Product row it delegates to that
        // row; a scalar child stays on its default row and lets the JNI compiler
        // choose a niche or a `(present, value)` intermediate from the compiled
        // child facts. The whole/default Optional row remains untouched.
        //
        // Bind both directions: function inputs construct the Rust optional,
        // while returns and callback arguments deconstruct it.
        for (outer, inner) in self.implicit_optionals(model).into_values() {
            for direction in [Direction::Construct, Direction::Deconstruct] {
                let outer = Crossing::new(outer.clone(), direction);
                let row = outer.row(parts());
                let inner = Crossing::new(inner.clone(), direction);
                bound.bind(
                    Site::part(&row, 0),
                    inner.clone(),
                    if recipes.key_of(&inner.key(), &parts()).is_some() {
                        Ask::Recipe(parts())
                    } else {
                        Ask::Default
                    },
                    Origin::Part,
                );
            }
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
            let building = Crossing::new(ty.clone(), Direction::Construct);
            let handing_out = Crossing::new(ty, Direction::Deconstruct);
            for (index, field) in s.fields.iter().enumerate() {
                // A field selects `parts` exactly when its crossing declares
                // that row AND the declaration says this occurrence decomposes.
                // Direct classes retain the existing `.jobject_input()` and
                // erased-wrapper boundaries; every implicit Optional selects its
                // part representation, which decides whether the child is
                // another composition or one scalar intermediate.
                if field.ty.optional_inner().is_none()
                    && !self.field_crosses_as_its_fields(&field.ty)
                {
                    continue;
                }
                for (of, direction) in [
                    (&building, Direction::Construct),
                    (&handing_out, Direction::Deconstruct),
                ] {
                    let row = of.row(parts());
                    let field_crossing = Crossing::new(field.ty.clone(), direction);
                    if recipes.key_of(&field_crossing.key(), &parts()).is_none() {
                        continue;
                    }
                    bound.bind(
                        Site::part(&row, index),
                        field_crossing,
                        Ask::Recipe(parts()),
                        Origin::Part,
                    );
                }
            }
        }
        // Every function whose return the binding takes apart, bound to the
        // `parts` recipe of its return type.
        //
        // This is what a site naming several values is: `Ask::Recipe` already
        // lets a site pick a recipe, and a `parts` fragment already occupies
        // several wires — the two facts stages 3 and 4 established, meeting.
        // The registry needs nothing new to express a decomposed return.
        for f in model.functions() {
            // The value that crosses, through the layers a decomposition looks
            // through: a `&T` return decomposes T, and so does an `Option<T>`
            // or a `Vec<T>` — the shape rides the delivery, not the recipe.
            let ret = f.ret.borrow_target().unwrap_or(&f.ret);
            let ret = ret.optional_inner().unwrap_or(ret);
            let ret = ret.sequence_elem().unwrap_or(ret);
            let ret = ret.borrow_target().unwrap_or(ret);
            let crossing = Crossing::new(ret.clone(), Direction::Deconstruct);
            // Only where the type states one. A `sealed_class` always does; a
            // `data_class` states one unless a field of it declines, and a
            // return whose type states none crosses whole.
            if recipes.key_of(&crossing.key(), &parts()).is_none() {
                continue;
            }
            // And only where it is genuinely several. A decomposition that
            // yields ONE value takes `Delivery::Return`: the wrapper hands that
            // value back through its own conversion rather than through a
            // builder, so the return site's crossing is the value's, not this
            // type's. Asked of the composition, which is model and declaration
            // only and so answerable here — the same property that let one
            // composition serve both sides of `resolve`.
            if self.out_wire_count(registry, ret) < 2 {
                continue;
            }
            bound.bind(
                Site {
                    owner: f.name.clone(),
                    role: prebindgen_registry::recipe::Role::Return,
                },
                crossing,
                Ask::Recipe(parts()),
                Origin::Adapter,
            );
        }

        // A callback's `Invoke` recipe owns its argument conversions. Bind a
        // model-derived data-class argument to `parts` so the Product fragment
        // reaches `Compile::callback` before the trampoline is rendered.
        for f in model.functions() {
            for param in &f.params {
                let prebindgen_registry::flat::TypeKind::Callback { args } =
                    param.ty.unwrapped().kind()
                else {
                    continue;
                };
                let callback = Crossing::new(param.ty.clone(), Direction::Construct);
                let row = callback.row(RecipeName::derived());
                for (index, arg) in args.iter().enumerate() {
                    let crossed = arg.borrow_target().unwrap_or(arg);
                    let core = crossed.optional_inner().unwrap_or(crossed);
                    if let Some(element) = core.sequence_elem() {
                        let element = element.borrow_target().unwrap_or(element);
                        if !self.field_crosses_as_its_fields(element) {
                            continue;
                        }
                        let sequence = Crossing::new(arg.clone(), Direction::Deconstruct);
                        let element_crossing =
                            Crossing::new(element.borrowed(), Direction::Deconstruct);
                        if recipes.key_of(&element_crossing.key(), &parts()).is_none() {
                            continue;
                        }
                        bound.bind(
                            Site::part(&sequence.row(RecipeName::derived()), 0),
                            element_crossing,
                            Ask::Recipe(parts()),
                            Origin::Adapter,
                        );
                        continue;
                    }
                    // A model-derived data class, or a type whose deconstructor
                    // declaration earned the `parts` row above. Both take that
                    // row here, so the fragment the trampoline delivers and the
                    // one the argument's own `Role::CallbackArg` site names are
                    // one fragment rather than two rows over one crossing.
                    if !self.field_crosses_as_its_fields(core)
                        && registry.callback_arg_plan(&arg.key()).is_none()
                    {
                        continue;
                    }
                    let crossing = Crossing::new(arg.clone(), Direction::Deconstruct);
                    if recipes.key_of(&crossing.key(), &parts()).is_none() {
                        continue;
                    }
                    bound.bind(
                        Site::part(&row, index),
                        crossing,
                        Ask::Recipe(parts()),
                        Origin::Adapter,
                    );
                }
            }
        }

        // Each value a callback delivers is a root site of its own — the
        // function-unique `Role::CallbackArg` the registry names, not the
        // `Role::Part` above, which is keyed by the callback recipe every
        // function with that signature shares.
        //
        // Bound to `parts` exactly where the argument's crossing states that
        // row, which is now every argument the trampoline takes apart: a
        // `data_class`, a `sealed_class`, an implicit Optional over one of
        // those, or a type whose deconstructor declaration earned the row
        // declared above. An argument that crosses whole names nothing here and
        // takes its crossing's default, attributed to the adapter.
        for f in model.functions() {
            for (param, p) in f.params.iter().enumerate() {
                let prebindgen_registry::flat::TypeKind::Callback { args } =
                    p.ty.unwrapped().kind()
                else {
                    continue;
                };
                for (arg, ty) in args.iter().enumerate() {
                    let crossing = Crossing::new(ty.clone(), Direction::Deconstruct);
                    if recipes.key_of(&crossing.key(), &parts()).is_none() {
                        continue;
                    }
                    bound.bind(
                        Site {
                            owner: f.name.clone(),
                            role: prebindgen_registry::recipe::Role::CallbackArg { param, arg },
                        },
                        crossing,
                        Ask::Recipe(parts()),
                        Origin::Adapter,
                    );
                }
            }
        }

        bound.build(recipes)
    }
}
