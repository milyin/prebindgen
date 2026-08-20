//! One table of crossings, one recipe per row.
//!
//! A **crossing** is one Rust type plus one of two jobs — *construct* a Rust
//! value out of the wire values that arrived, or *deconstruct* one into the
//! wire values that leave. [`Recipes`] is the table that answers it, and a
//! **recipe** is one answer: which `#[prebindgen]` constructor assembles the
//! value, or which accessors take it apart.
//!
//! A recipe says nothing about the wire. How many wire values a crossing costs,
//! how they are encoded and what they cost to clean up is the adapter's answer,
//! and no part of it is stored here.
//!
//! A recipe also says nothing the model already says. Part count, part types,
//! ownership, names and fallibility are all read off [`flat`](crate::flat), so
//! a recipe cannot contradict the Rust it describes. What is left is short: a
//! whole product is one identifier.
//!
//! # What a crossing is keyed by
//!
//! A borrow is not part of the crossing. `Sample`, `&Sample` and `Box<Sample>`
//! are one row, because the same recipe assembles all three; whether the value
//! is handed over or reached through a borrow is [`Crossing::mode`], read off
//! the type. So an adapter declares `Sample` once and every site finds it.
//!
//! # Rows nobody declares
//!
//! A type with no declared row still gets one, derived from its kind:
//! `Option<T>` yields [`Shape::Optional`], `Vec<T>` and `&[T]` and `[T; N]`
//! yield [`Shape::Sequence`], and everything else yields [`Shape::Atomic`].
//! Nesting needs no rule of its own: a row names one layer, and the layer
//! inside it is a crossing with a row of its own.
//!
//! A callback has no row at all. Taking one apart into the values that pass
//! through it is the only thing any adapter can do with it, so there is no
//! decision to record and [`RecipesBuilder::build`] refuses a declaration for
//! one.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt,
};

use crate::flat::{Field, Flat, Function, Type, TypeKey, TypeKind, TypeRef};

#[cfg(test)]
mod tests;

// ── The two jobs ──────────────────────────────────────────────────────────

/// Which of the two jobs a crossing is, as a value.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Assembly {
    /// Build a Rust value out of the wire values that arrived.
    Construct,
    /// Take a Rust value apart into the wire values that leave.
    Deconstruct,
}

impl Assembly {
    /// The other job. Only a callback swaps: the Rust side holds the values its
    /// arguments carry and pushes them out through the call.
    pub fn swap(self) -> Self {
        match self {
            Assembly::Construct => Assembly::Deconstruct,
            Assembly::Deconstruct => Assembly::Construct,
        }
    }
}

impl fmt::Display for Assembly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Assembly::Construct => "construct",
            Assembly::Deconstruct => "deconstruct",
        })
    }
}

/// The same two jobs at the type level, stated on the operations themselves.
///
/// An adapter never writes this bound and never names an implementor: `OP` is
/// inferred from the shape handed to [`RecipesBuilder::declare`], which is what
/// files a row under the right job without anything stating it twice.
pub trait Operation: Sized {
    /// Which job an operation of this type does.
    const ASSEMBLY: Assembly;

    /// Erase the job so the table can hold both. Not part of the surface an
    /// adapter writes against.
    #[doc(hidden)]
    fn into_row(shape: Shape<Self>) -> Row;
}

/// What assembles a value from its parts.
#[derive(Clone, Debug)]
pub enum Construct {
    /// Call this `#[prebindgen]` constructor. **Its parameters are the parts** —
    /// how many, in what order, named what, typed what, taken by value or by
    /// reference. All of that is the model's
    /// [`Function`], so none of it is restated here, and
    /// whether the call is fallible is its return type.
    Call(syn::Ident),
    /// The single part named here already is the value.
    ///
    /// Boxed because a `TypeRef` is many times the size of the identifier
    /// [`Call`](Self::Call) carries, and a call is the common form. The same
    /// trade-off the model makes for an array's extent.
    Identity(Box<TypeRef>),
}

impl Operation for Construct {
    const ASSEMBLY: Assembly = Assembly::Construct;

    fn into_row(shape: Shape<Self>) -> Row {
        Row::Constructing(shape)
    }
}

/// What takes a value apart.
#[derive(Clone, Debug)]
pub enum Deconstruct {
    /// Read the parts off the value where it stands.
    Fields(Vec<Reach>),
    /// Call this accessor once, bind its result, and read the parts off that
    /// binding. Whether the accessor consumes the value is read from its own
    /// first parameter, not stated again.
    ValueForm {
        /// The accessor to call, whose first parameter is the row's own type.
        func: syn::Ident,
        /// Where each part of the bound result comes from.
        parts: Vec<Reach>,
    },
}

impl Operation for Deconstruct {
    const ASSEMBLY: Assembly = Assembly::Deconstruct;

    fn into_row(shape: Shape<Self>) -> Row {
        Row::Deconstructing(shape)
    }
}

/// Where one part comes from.
///
/// Each form yields the part's type, its ownership and its name, so a recipe
/// never repeats any of the three.
#[derive(Clone, Debug)]
pub enum Reach {
    /// Field `index` of the model's [`Struct`](crate::flat::Struct) — or,
    /// inside an arm, of that alternative. The adapter-synthesized case, where
    /// nothing is called.
    Field(usize),
    /// A `#[prebindgen]` accessor `fn(&T) -> P`, or `fn(T) -> P` to consume.
    Accessor(syn::Ident),
    /// This position contributes nothing.
    Omit,
}

/// Whether a value is handed over, or reached through a borrow.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Handed over. The receiver may keep it.
    Owned,
    /// Reached through `&T`.
    Shared,
    /// Reached through `&mut T`.
    Exclusive,
}

impl Mode {
    /// Whether a part yielded in this mode can be consumed where `wanted` is
    /// required. An edge declared `&T` accepts a borrow or an owned value; one
    /// declared `T` accepts only an owned value.
    pub fn satisfies(self, wanted: Mode) -> bool {
        match wanted {
            Mode::Owned => self == Mode::Owned,
            Mode::Shared => self != Mode::Exclusive,
            Mode::Exclusive => self == Mode::Exclusive,
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Mode::Owned => "owned",
            Mode::Shared => "&",
            Mode::Exclusive => "&mut",
        })
    }
}

// ── The crossing ──────────────────────────────────────────────────────────

/// One Rust type and one of the two jobs: the question the table answers.
#[derive(Clone, Debug)]
pub struct Crossing {
    ty: TypeRef,
    assembly: Assembly,
}

impl Crossing {
    /// The crossing of `ty`, doing `assembly`.
    ///
    /// `ty` is kept exactly as the site spelled it, borrow and transparent
    /// wrappers included; what is normalized away is only the key
    /// [`Self::key`] derives from it.
    pub fn new(ty: TypeRef, assembly: Assembly) -> Self {
        Self { ty, assembly }
    }

    /// The type as the site spelled it.
    pub fn spelled(&self) -> &TypeRef {
        &self.ty
    }

    /// The Rust value that crosses: the spelled type with a borrow peeled off.
    pub fn value(&self) -> &TypeRef {
        self.ty.borrow_target().unwrap_or(&self.ty)
    }

    /// Which job this crossing is.
    pub fn assembly(&self) -> Assembly {
        self.assembly
    }

    /// Whether the value is handed over or reached through a borrow.
    pub fn mode(&self) -> Mode {
        match self.ty.unwrapped().kind() {
            TypeKind::Ref { mutable: true, .. } => Mode::Exclusive,
            TypeKind::Ref { .. } => Mode::Shared,
            _ => Mode::Owned,
        }
    }

    /// The erased form, for maps and diagnostics.
    pub fn key(&self) -> CrossingKey {
        CrossingKey {
            ty: self.value().stripped_key(),
            assembly: self.assembly,
        }
    }
}

/// A crossing identified rather than described.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CrossingKey {
    /// The Rust value that crosses, with borrow and transparent wrappers gone.
    pub ty: TypeKey,
    /// Which of the two jobs.
    pub assembly: Assembly,
}

impl fmt::Display for CrossingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.ty, self.assembly)
    }
}

/// Names one of several answers a crossing may have.
///
/// Adapters mint these; the table attaches no meaning to any particular name.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecipeId(String);

impl RecipeId {
    /// The name of one answer.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name the table gives the row it derives for an undeclared crossing.
    pub fn derived() -> Self {
        Self("derived".to_owned())
    }

    /// The name as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RecipeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── What a recipe says ────────────────────────────────────────────────────

/// How a value gets across, in terms of its parts.
///
/// `OP` is the one thing a recipe states differently between the two jobs:
/// what assembles the parts into a value, or takes them out of it. Nothing
/// below the top restates the job, because a part, an optional's value and a
/// sequence's element all do the same job as the row.
#[derive(Clone, Debug)]
pub enum Shape<OP> {
    /// No parts. The adapter emits the conversion itself; how many wire values
    /// that costs is the adapter's business and the table never asks.
    Atomic,
    /// Absent, or the inner type.
    Optional {
        /// The type an `Option` wraps.
        inner: TypeRef,
    },
    /// A run of the inner type.
    ///
    /// Whether iterating the run yields owned values or borrows is the
    /// collection's business, not the recipe's, so it is derived rather than
    /// stated: `Vec<T>` gives its elements up, `&[T]` and `Cow<'_, [T]>` lend
    /// them.
    Sequence {
        /// The element type.
        inner: TypeRef,
    },
    /// Every part contributes.
    Product(OP),
    /// Exactly one arm is live at run time. Every arm still compiles.
    Choice {
        /// One entry per alternative that crosses.
        arms: Vec<Arm<OP>>,
    },
}

/// One alternative of the model's [`Variant`](crate::flat::Variant).
///
/// The alternative's name and its position come with it, and how the foreign
/// side is told which arm is live is the adapter's choice, which is why no tag
/// is stated here.
#[derive(Clone, Debug)]
pub struct Arm<OP> {
    /// Position of the alternative within its sum.
    pub alternative: usize,
    /// What assembles this arm's payload from its parts, or takes it out.
    pub op: OP,
}

/// A recipe that builds a Rust value out of what arrived.
pub type Constructing = Shape<Construct>;
/// A recipe that takes a Rust value apart into what leaves.
pub type Deconstructing = Shape<Deconstruct>;

/// One row's recipe, under whichever job it does.
///
/// The table holds rows in this form so both jobs share one map. An adapter
/// writes a [`Constructing`] or a [`Deconstructing`] and never builds or
/// matches on a `Row`.
#[derive(Clone, Debug)]
pub enum Row {
    /// A row filed under [`Assembly::Construct`].
    Constructing(Constructing),
    /// A row filed under [`Assembly::Deconstruct`].
    Deconstructing(Deconstructing),
    /// A callback, taken apart into the values that pass through it.
    ///
    /// Derived from the type's kind and never declared, because there is no
    /// decision to record: a callable that crossed whole would not be callable
    /// from Rust. Its parts are the callback's arguments, and they do the other
    /// job — the Rust side holds those values and pushes them out through the
    /// call. The [`Assembly`] here is the row's own, not its arguments'.
    Callback(Assembly),
}

impl Row {
    /// Which job this row does.
    pub fn assembly(&self) -> Assembly {
        match self {
            Row::Constructing(_) => Assembly::Construct,
            Row::Deconstructing(_) => Assembly::Deconstruct,
            Row::Callback(assembly) => *assembly,
        }
    }
}

// ── The table ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Entry {
    id: RecipeId,
    row: Row,
    /// The type as the declaration spelled it, which is what the checks read
    /// fields and alternatives off.
    ty: TypeRef,
    default: bool,
}

/// The resolved table. Built by [`RecipesBuilder`], checked once, then
/// immutable.
#[derive(Clone, Debug, Default)]
pub struct Recipes {
    rows: HashMap<CrossingKey, Vec<Entry>>,
}

impl Recipes {
    /// Start describing a table.
    pub fn builder() -> RecipesBuilder {
        RecipesBuilder::default()
    }

    /// Which rows this crossing has, in declaration order.
    ///
    /// Empty for a crossing nobody declared, which still has the row
    /// [`Self::row`] derives.
    pub fn rows(&self, key: &CrossingKey) -> Vec<&RecipeId> {
        self.rows
            .get(key)
            .map(|rows| rows.iter().map(|e| &e.id).collect())
            .unwrap_or_default()
    }

    /// One named row of a crossing, or `None` if it was never declared.
    pub fn get(&self, key: &CrossingKey, id: &RecipeId) -> Option<&Row> {
        self.rows
            .get(key)?
            .iter()
            .find(|e| &e.id == id)
            .map(|e| &e.row)
    }

    /// The row a site uses when it names none.
    ///
    /// With one declared row that row is the default; with several it is the
    /// one declared through [`RecipesBuilder::declare_default`]. `None` for a
    /// crossing nobody declared.
    pub fn default_of(&self, key: &CrossingKey) -> Option<&RecipeId> {
        let rows = self.rows.get(key)?;
        match rows.as_slice() {
            [only] => Some(&only.id),
            many => many.iter().find(|e| e.default).map(|e| &e.id),
        }
    }

    /// The default row for a crossing: the declared one, or the row derived
    /// from the type's kind.
    ///
    /// A callback derives [`Row::Callback`], which nothing can declare, so an
    /// adapter reaches one here the same way it reaches every other row.
    pub fn row(&self, crossing: &Crossing) -> (RecipeId, Cow<'_, Row>) {
        let key = crossing.key();
        match self.default_of(&key) {
            Some(id) => {
                let id = id.clone();
                let row = self.get(&key, &id).expect("default names a declared row");
                (id, Cow::Borrowed(row))
            }
            None => (RecipeId::derived(), Cow::Owned(derive(crossing))),
        }
    }
}

/// The arity row a crossing gets when nobody declared one.
fn derive(crossing: &Crossing) -> Row {
    let value = crossing.value();
    if value.callback_args().is_some() {
        return Row::Callback(crossing.assembly);
    }
    let shape = if let Some(inner) = value.optional_inner() {
        Some((inner.clone(), true))
    } else {
        sequence_elem(value).map(|inner| (inner.clone(), false))
    };
    match crossing.assembly {
        Assembly::Construct => Row::Constructing(match shape {
            Some((inner, true)) => Shape::Optional { inner },
            Some((inner, false)) => Shape::Sequence { inner },
            None => Shape::Atomic,
        }),
        Assembly::Deconstruct => Row::Deconstructing(match shape {
            Some((inner, true)) => Shape::Optional { inner },
            Some((inner, false)) => Shape::Sequence { inner },
            None => Shape::Atomic,
        }),
    }
}

/// The element of a run, including the fixed-size array the model's own
/// [`TypeRef::sequence_elem`](crate::flat::TypeRef::sequence_elem) leaves out.
fn sequence_elem(ty: &TypeRef) -> Option<&TypeRef> {
    if let Some(elem) = ty.sequence_elem() {
        return Some(elem);
    }
    match ty.unwrapped().kind() {
        TypeKind::Array { elem, .. } => Some(elem),
        _ => None,
    }
}

/// Describes a [`Recipes`] table.
#[derive(Default)]
pub struct RecipesBuilder {
    rows: HashMap<CrossingKey, Vec<Entry>>,
    /// A row declared twice under one name, reported by [`Self::build`] rather
    /// than by overwriting silently.
    duplicates: Vec<(CrossingKey, RecipeId)>,
}

impl RecipesBuilder {
    /// Add one row for `ty`.
    ///
    /// Which job the row is filed under is the shape's own, so nothing states
    /// it twice and the two cannot disagree. Declaring a second row for one
    /// crossing is how a type offers a choice, and the site is what picks
    /// between them — at which point one of the rows has to be declared through
    /// [`Self::declare_default`].
    pub fn declare<OP: Operation>(
        &mut self,
        ty: TypeRef,
        id: RecipeId,
        shape: Shape<OP>,
    ) -> &mut Self {
        self.insert(ty, id, OP::into_row(shape), false)
    }

    /// Add one row and make it the row used where a site names none.
    ///
    /// Needed only once a crossing has more than one row: with a single row
    /// that row is the default, so the common case never says so.
    pub fn declare_default<OP: Operation>(
        &mut self,
        ty: TypeRef,
        id: RecipeId,
        shape: Shape<OP>,
    ) -> &mut Self {
        self.insert(ty, id, OP::into_row(shape), true)
    }

    fn insert(&mut self, ty: TypeRef, id: RecipeId, row: Row, default: bool) -> &mut Self {
        let key = Crossing::new(ty.clone(), row.assembly()).key();
        let entries = self.rows.entry(key.clone()).or_default();
        if entries.iter().any(|e| e.id == id) {
            self.duplicates.push((key, id));
            return self;
        }
        entries.push(Entry {
            id,
            row,
            ty,
            default,
        });
        self
    }

    /// Check the table and freeze it.
    ///
    /// Every problem is reported, not just the first. The checks are the ones
    /// no type can express: whether a recipe names something the model has,
    /// whether a crossing with several rows says which of them wins, and
    /// whether a row reaches its own crossing.
    pub fn build(self, model: &Flat) -> Result<Recipes, Vec<RecipeError>> {
        let table = Recipes { rows: self.rows };
        let mut errors: Vec<RecipeError> = self
            .duplicates
            .into_iter()
            .map(|(crossing, recipe)| RecipeError::Duplicate { crossing, recipe })
            .collect();

        for (key, entries) in &table.rows {
            if entries.len() > 1 {
                let defaults: Vec<RecipeId> = entries
                    .iter()
                    .filter(|e| e.default)
                    .map(|e| e.id.clone())
                    .collect();
                if defaults.len() != 1 {
                    errors.push(RecipeError::NoDefault {
                        crossing: key.clone(),
                        defaults,
                    });
                }
            }
            for entry in entries {
                let crossing = Crossing::new(entry.ty.clone(), entry.row.assembly());
                if crossing.value().callback_args().is_some() {
                    errors.push(RecipeError::CallbackDeclared {
                        row: key.clone(),
                        recipe: entry.id.clone(),
                    });
                    continue;
                }
                let mut check = Check {
                    model,
                    row: key,
                    recipe: &entry.id,
                    errors: &mut errors,
                    arm_fields: None,
                };
                check.row(&entry.ty, &entry.row);
            }
        }

        cycles(model, &table, &mut errors);

        if errors.is_empty() {
            Ok(table)
        } else {
            Err(errors)
        }
    }
}

// ── Checking one row ──────────────────────────────────────────────────────

struct Check<'a, 'e> {
    model: &'a Flat,
    row: &'a CrossingKey,
    recipe: &'a RecipeId,
    errors: &'e mut Vec<RecipeError>,
    /// The payload fields of the arm being checked, which is what a
    /// [`Reach::Field`] indexes inside a [`Shape::Choice`].
    arm_fields: Option<Vec<Field>>,
}

impl<'a> Check<'a, '_> {
    fn push(&mut self, error: RecipeError) {
        self.errors.push(error);
    }

    fn out_of_range(&mut self, index: usize, len: usize) {
        self.push(RecipeError::OutOfRange {
            row: self.row.clone(),
            recipe: self.recipe.clone(),
            index,
            len,
        });
    }

    fn not_a_product(&mut self) {
        self.push(RecipeError::NotAProduct {
            row: self.row.clone(),
            recipe: self.recipe.clone(),
        });
    }

    /// Every part crossing this row reaches, checking what it names on the way.
    fn row(&mut self, ty: &TypeRef, row: &Row) -> Vec<Crossing> {
        let (assembly, parts) = match row {
            Row::Constructing(shape) => (Assembly::Construct, self.constructing(ty, shape)),
            Row::Deconstructing(shape) => (Assembly::Deconstruct, self.deconstructing(ty, shape)),
            // The one place the two jobs swap.
            Row::Callback(assembly) => {
                let args = Crossing::new(ty.clone(), *assembly)
                    .value()
                    .callback_args()
                    .unwrap_or_default()
                    .to_vec();
                (assembly.swap(), args)
            }
        };
        parts
            .into_iter()
            .map(|ty| Crossing::new(ty, assembly))
            .collect()
    }

    fn constructing(&mut self, ty: &TypeRef, shape: &Constructing) -> Vec<TypeRef> {
        match shape {
            Shape::Atomic => Vec::new(),
            Shape::Optional { inner } | Shape::Sequence { inner } => vec![inner.clone()],
            Shape::Product(op) => self.construct(op),
            Shape::Choice { arms } => {
                // An arm's constructor names the alternative it builds, so no
                // field list has to be in scope for one.
                let Some(alternatives) = self.alternatives(ty) else {
                    self.not_a_product();
                    return Vec::new();
                };
                let mut parts = Vec::new();
                for arm in arms {
                    if arm.alternative >= alternatives.len() {
                        self.out_of_range(arm.alternative, alternatives.len());
                        continue;
                    }
                    parts.extend(self.construct(&arm.op));
                }
                parts
            }
        }
    }

    fn deconstructing(&mut self, ty: &TypeRef, shape: &Deconstructing) -> Vec<TypeRef> {
        match shape {
            Shape::Atomic => Vec::new(),
            Shape::Optional { inner } | Shape::Sequence { inner } => vec![inner.clone()],
            Shape::Product(op) => self.deconstruct(ty, op),
            Shape::Choice { arms } => {
                let Some(alternatives) = self.alternatives(ty) else {
                    self.not_a_product();
                    return Vec::new();
                };
                let mut parts = Vec::new();
                for arm in arms {
                    let Some(fields) = alternatives.get(arm.alternative) else {
                        self.out_of_range(arm.alternative, alternatives.len());
                        continue;
                    };
                    // The arm's payload stands in for the sum's own fields,
                    // which the model gives a sum none of.
                    let outer = self.arm_fields.replace(fields.clone());
                    parts.extend(self.deconstruct(ty, &arm.op));
                    self.arm_fields = outer;
                }
                parts
            }
        }
    }

    fn construct(&mut self, op: &Construct) -> Vec<TypeRef> {
        match op {
            Construct::Identity(inner) => vec![(**inner).clone()],
            Construct::Call(func) => match self.function(func) {
                Some(f) => f.params.iter().map(|p| p.ty.clone()).collect(),
                None => Vec::new(),
            },
        }
    }

    fn deconstruct(&mut self, ty: &TypeRef, op: &Deconstruct) -> Vec<TypeRef> {
        match op {
            Deconstruct::Fields(reaches) => self.reaches(ty, reaches),
            Deconstruct::ValueForm { func, parts } => {
                let Some(f) = self.function(func) else {
                    return Vec::new();
                };
                let bound = f.ret.clone();
                if !accessor_of(f, ty) {
                    self.push(RecipeError::NotAnAccessor {
                        row: self.row.clone(),
                        recipe: self.recipe.clone(),
                        func: func.clone(),
                    });
                }
                // Parts are read off the bound result, so the arm's payload no
                // longer stands in for them.
                let outer = self.arm_fields.take();
                let parts = self.reaches(&bound, parts);
                self.arm_fields = outer;
                parts
            }
        }
    }

    fn reaches(&mut self, ty: &TypeRef, reaches: &[Reach]) -> Vec<TypeRef> {
        let mut out = Vec::new();
        for reach in reaches {
            match reach {
                Reach::Omit => {}
                Reach::Accessor(func) => {
                    let Some(f) = self.function(func) else {
                        continue;
                    };
                    let ret = f.ret.clone();
                    if !accessor_of(f, ty) {
                        self.push(RecipeError::NotAnAccessor {
                            row: self.row.clone(),
                            recipe: self.recipe.clone(),
                            func: func.clone(),
                        });
                    }
                    out.push(ret);
                }
                Reach::Field(index) => {
                    let Some(fields) = self.fields(ty) else {
                        self.not_a_product();
                        continue;
                    };
                    match fields.get(*index) {
                        Some(field) => out.push(field.ty.clone()),
                        None => self.out_of_range(*index, fields.len()),
                    }
                }
            }
        }
        out
    }

    fn function(&mut self, name: &syn::Ident) -> Option<&'a Function> {
        match self.model.function(name) {
            Some(f) => Some(f),
            None => {
                let error = RecipeError::UnknownFunction {
                    row: self.row.clone(),
                    recipe: self.recipe.clone(),
                    func: name.clone(),
                };
                self.push(error);
                None
            }
        }
    }

    /// The fields a [`Reach::Field`] indexes: an arm's payload where one is in
    /// scope, else the type's own.
    fn fields(&self, ty: &TypeRef) -> Option<Vec<Field>> {
        match &self.arm_fields {
            Some(fields) => Some(fields.clone()),
            None => match declared(self.model, ty)? {
                Type::Struct(s) => Some(s.fields.clone()),
                _ => None,
            },
        }
    }

    fn alternatives(&self, ty: &TypeRef) -> Option<Vec<Vec<Field>>> {
        match declared(self.model, ty)? {
            Type::Variant(v) => Some(v.alternatives.iter().map(|a| a.fields.clone()).collect()),
            _ => None,
        }
    }
}

/// Whether `f` reads a value of `ty` — `fn(&T) -> P` or `fn(T) -> P`.
fn accessor_of(f: &Function, ty: &TypeRef) -> bool {
    let Some(first) = f.params.first() else {
        return false;
    };
    value_key(&first.ty) == value_key(ty)
}

/// The type's identity with a borrow and any transparent wrapper gone — what
/// [`Crossing::key`] keys a row by, without a job attached.
fn value_key(ty: &TypeRef) -> TypeKey {
    ty.borrow_target().unwrap_or(ty).stripped_key()
}

/// The `#[prebindgen]` type declaration `ty` names, if it names one.
fn declared<'a>(model: &'a Flat, ty: &TypeRef) -> Option<&'a Type> {
    let value = ty.borrow_target().unwrap_or(ty).unwrapped();
    match value.kind() {
        TypeKind::Named { id, .. } => model.resolve(id),
        _ => None,
    }
}

// ── The one structural rule ───────────────────────────────────────────────

/// A row may not reach its own crossing, directly or through any chain of
/// parts.
///
/// The traversal unrolls the table's graph statically, so a cycle would not
/// terminate. That is a mechanical limit rather than a semantic one, which is
/// why a self-referential Rust type is refused here and is not otherwise
/// ill-behaved.
fn cycles(model: &Flat, table: &Recipes, errors: &mut Vec<RecipeError>) {
    let mut settled: HashSet<CrossingKey> = HashSet::new();
    let mut reported: HashSet<CrossingKey> = HashSet::new();
    let roots: Vec<Crossing> = table
        .rows
        .values()
        .flat_map(|entries| {
            entries
                .iter()
                .map(|e| Crossing::new(e.ty.clone(), e.row.assembly()))
        })
        .collect();
    for root in roots {
        let mut path = Vec::new();
        walk(
            model,
            table,
            &root,
            &mut path,
            &mut settled,
            &mut reported,
            errors,
        );
    }
}

fn walk(
    model: &Flat,
    table: &Recipes,
    crossing: &Crossing,
    path: &mut Vec<CrossingKey>,
    settled: &mut HashSet<CrossingKey>,
    reported: &mut HashSet<CrossingKey>,
    errors: &mut Vec<RecipeError>,
) {
    let key = crossing.key();
    if settled.contains(&key) {
        return;
    }
    if let Some(start) = path.iter().position(|k| k == &key) {
        if reported.insert(key.clone()) {
            let mut cycle = path[start..].to_vec();
            cycle.push(key);
            errors.push(RecipeError::Cycle { path: cycle });
        }
        return;
    }
    path.push(key.clone());
    for part in successors(model, table, crossing) {
        walk(model, table, &part, path, settled, reported, errors);
    }
    path.pop();
    settled.insert(key);
}

/// Every crossing a crossing's rows reach.
///
/// Every row counts, not only the default: a site may name any of them, so a
/// cycle through the row nobody happens to default to is still a cycle.
fn successors(model: &Flat, table: &Recipes, crossing: &Crossing) -> Vec<Crossing> {
    let key = crossing.key();
    let rows: Vec<(RecipeId, TypeRef, Row)> = match table.rows.get(&key) {
        Some(entries) => entries
            .iter()
            .map(|e| (e.id.clone(), e.ty.clone(), e.row.clone()))
            .collect(),
        None => vec![(
            RecipeId::derived(),
            crossing.spelled().clone(),
            derive(crossing),
        )],
    };
    // Anything a row names wrongly is reported by the per-row pass; here the
    // walk only needs the parts a well-formed reading of the row reaches.
    let mut discarded = Vec::new();
    rows.into_iter()
        .flat_map(|(id, ty, row)| {
            let mut check = Check {
                model,
                row: &key,
                recipe: &id,
                errors: &mut discarded,
                arm_fields: None,
            };
            check.row(&ty, &row)
        })
        .collect()
}

// ── What the table refuses ────────────────────────────────────────────────

/// A problem [`RecipesBuilder::build`] found.
#[derive(Debug)]
pub enum RecipeError {
    /// A row's parts reach the row's own crossing.
    ///
    /// The path is a chain of keys because it may pass through a callback, and
    /// so swap jobs.
    Cycle {
        /// The chain, starting and ending at the crossing that repeats.
        path: Vec<CrossingKey>,
    },
    /// A crossing has several rows and none, or more than one, was declared the
    /// default.
    NoDefault {
        /// The crossing whose rows disagree.
        crossing: CrossingKey,
        /// The rows that claimed to be the default.
        defaults: Vec<RecipeId>,
    },
    /// One name was declared twice for one crossing.
    Duplicate {
        /// The crossing declared twice.
        crossing: CrossingKey,
        /// The name used twice.
        recipe: RecipeId,
    },
    /// A recipe named a constructor or accessor the model does not have.
    UnknownFunction {
        /// The crossing the recipe answers.
        row: CrossingKey,
        /// Which of the crossing's rows named it.
        recipe: RecipeId,
        /// The name that resolved to nothing.
        func: syn::Ident,
    },
    /// An accessor was named where its first parameter is not the row's type.
    NotAnAccessor {
        /// The crossing the recipe answers.
        row: CrossingKey,
        /// Which of the crossing's rows named it.
        recipe: RecipeId,
        /// The function whose first parameter does not match.
        func: syn::Ident,
    },
    /// A [`Reach::Field`] or an [`Arm::alternative`] is past the end of what the
    /// model holds for that type.
    OutOfRange {
        /// The crossing the recipe answers.
        row: CrossingKey,
        /// Which of the crossing's rows names the index.
        recipe: RecipeId,
        /// The index the recipe named.
        index: usize,
        /// How many the model holds.
        len: usize,
    },
    /// A product or choice recipe was declared for a type the model gives no
    /// parts.
    NotAProduct {
        /// The crossing the recipe answers.
        row: CrossingKey,
        /// Which of the crossing's rows was declared.
        recipe: RecipeId,
    },
    /// A row was declared for a callback, which has no decision to record.
    CallbackDeclared {
        /// The callback crossing.
        row: CrossingKey,
        /// The row that was declared for it.
        recipe: RecipeId,
    },
}

impl fmt::Display for RecipeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecipeError::Cycle { path } => {
                let chain: Vec<String> = path.iter().map(|k| k.to_string()).collect();
                write!(
                    f,
                    "a recipe reaches its own crossing: {}",
                    chain.join(" -> ")
                )
            }
            RecipeError::NoDefault { crossing, defaults } => write!(
                f,
                "{crossing} has several rows and {} of them is the default; \
                 declare exactly one with `declare_default`",
                if defaults.is_empty() {
                    "none".to_owned()
                } else {
                    format!("{}", defaults.len())
                }
            ),
            RecipeError::Duplicate { crossing, recipe } => {
                write!(f, "{crossing} declares the row `{recipe}` twice")
            }
            RecipeError::UnknownFunction { row, recipe, func } => write!(
                f,
                "row `{recipe}` of {row} names `{func}`, which no #[prebindgen] \
                 source declares"
            ),
            RecipeError::NotAnAccessor { row, recipe, func } => write!(
                f,
                "row `{recipe}` of {row} reaches a part through `{func}`, whose \
                 first parameter is not that type"
            ),
            RecipeError::OutOfRange {
                row,
                recipe,
                index,
                len,
            } => write!(
                f,
                "row `{recipe}` of {row} names index {index}, and the model holds {len}"
            ),
            RecipeError::NotAProduct { row, recipe } => write!(
                f,
                "row `{recipe}` takes {row} apart, and the model gives that type no parts"
            ),
            RecipeError::CallbackDeclared { row, recipe } => write!(
                f,
                "row `{recipe}` was declared for the callback {row}; a callback is \
                 always taken apart into its arguments, so it has no row to declare"
            ),
        }
    }
}

impl std::error::Error for RecipeError {}
