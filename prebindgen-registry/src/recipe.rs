//! The table of crossings, and the recipes that answer them.
//!
//! A **crossing** is one Rust type plus one of two directions — *construct* a Rust
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
//! [`RecipeName`] is reusable adapter vocabulary such as `whole` or `parts`.
//! [`RecipeKey`] pairs that name with a [`CrossingKey`] and uniquely identifies
//! one row throughout the table. Resolved bindings and compiled fragments carry
//! the full key rather than a context-dependent name.
//!
//! # What a crossing is keyed by
//!
//! A borrow is not part of the crossing. `Sample`, `&Sample` and `Box<Sample>`
//! are one recipe, because the same recipe assembles all three; whether the value
//! is handed over or reached through a borrow is [`Crossing::mode`], read off
//! the type. So an adapter declares `Sample` once and every site finds it.
//!
//! # Recipes nobody declares
//!
//! A type with no declared recipe still gets one, derived from its kind:
//! `Option<T>` yields [`Shape::Optional`], `Vec<T>` and `&[T]` and `[T; N]`
//! yield [`Shape::Sequence`], and everything else yields [`Shape::Atomic`].
//! Nesting needs no rule of its own: a recipe names one layer, and the layer
//! inside it is a crossing with a recipe of its own.
//!
//! A callback type yields [`Shape::Invoke`], and that is the only shape such a
//! crossing takes: converting the arguments is what makes the callable
//! callable, so there is no second answer to choose between.
//! [`RecipesBuilder::build`] refuses any other shape there, and an `Invoke` recipe's
//! parts are the callback's arguments, which take the other direction.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt,
};

use crate::flat::{Field, Flat, Function, Type, TypeKey, TypeKind, TypeRef};

mod compile;
mod site;
#[cfg(test)]
mod tests;

pub use self::{
    compile::{
        At, Carrier, Compile, CompileError, Compiled, Compiler, Cx, Frag, Part, PartSource, Parts,
        Validity, Yield,
    },
    site::{Ask, Bindings, BindingsBuilder, Bound, Origin, Role, Site},
};

// ── The two directions ──────────────────────────────────────────────────────────

/// Which of the two directions a crossing is, as a value.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Direction {
    /// Build a Rust value out of the wire values that arrived.
    Construct,
    /// Take a Rust value apart into the wire values that leave.
    Deconstruct,
}

impl Direction {
    /// The other direction. Only a callback swaps: the Rust side holds the values
    /// arguments carry and pushes them out through the call.
    pub fn swap(self) -> Self {
        match self {
            Direction::Construct => Direction::Deconstruct,
            Direction::Deconstruct => Direction::Construct,
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Direction::Construct => "construct",
            Direction::Deconstruct => "deconstruct",
        })
    }
}

/// The same two directions at the type level, stated on the operations themselves.
///
/// An adapter never writes this bound and never names an implementor: `OP` is
/// inferred from the shape handed to [`RecipesBuilder::declare`], which is what
/// files a recipe under the right direction without anything stating it twice.
pub trait Operation: Sized {
    /// Which direction an operation of this type takes.
    const DIRECTION: Direction;

    /// Erase the direction so the table can hold both. Not part of the surface an
    /// adapter writes against.
    #[doc(hidden)]
    fn into_recipe(shape: Shape<Self>) -> Recipe;
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
    /// Write the value's own fields. **They are the parts**, in the model's
    /// order, and every one of them contributes.
    ///
    /// The adapter-synthesized case, where nothing is called: a `#[repr(C)]`
    /// mirror of a struct with public fields is rebuilt as a struct literal,
    /// with no constructor in the source crate to name. Unlike
    /// [`Deconstruct::Fields`] it names no list, because a value cannot be
    /// built with one of its fields missing, and which fields there are is the
    /// model's answer.
    ///
    /// Inside a [`Shape::Choice`] arm the fields are that alternative's.
    Fields,
}

impl Operation for Construct {
    const DIRECTION: Direction = Direction::Construct;

    fn into_recipe(shape: Shape<Self>) -> Recipe {
        Recipe::Constructing(shape)
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
        /// The accessor to call, whose first parameter is the recipe's own type.
        func: syn::Ident,
        /// Where each part of the bound result comes from.
        parts: Vec<Reach>,
    },
}

impl Operation for Deconstruct {
    const DIRECTION: Direction = Direction::Deconstruct;

    fn into_recipe(shape: Shape<Self>) -> Recipe {
        Recipe::Deconstructing(shape)
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
    /// A field of a field: the access chain an inlined nested class needs,
    /// outermost first. `Field(i)` is the one-element case, kept separate
    /// because it is the overwhelming majority and reads better.
    ///
    /// The form a spliced `FieldRecord` states: its `members` is exactly this
    /// chain, and without it a `parts` row for a value form that inlines a
    /// nested declared class cannot be spelled at all (#613 step 10).
    Path(Vec<usize>),
    /// Field `index`, taken apart **here** by `shape` rather than by whatever
    /// row its own type has.
    ///
    /// **A STRUCT-shaped field only.** `shape` is a [`Deconstruct`], which is
    /// `Fields` or `ValueForm` — both read parts off a product — so the field
    /// must be one. A sum-typed field is refused rather than silently
    /// contributing nothing, which is what an empty field list would otherwise
    /// do (#658 review).
    ///
    /// It does NOT yet serve the case #613 step 10 ends at. A `sealed_class`
    /// has no deconstructing whole-value crossing (`prebindgen-jni` states that
    /// contract), so a sum-typed field's LEAVES are what cross, and reaching
    /// them is a `Choice` — which lives on [`Shape`], not on `Deconstruct`.
    /// Carrying one means this reach holding a `Shape<Deconstruct>` and the
    /// compiler composing a nested choice into a part, neither of which exists
    /// here.
    ///
    /// **Which of the two ways to carry it is settled** (#660 item 6). A part's
    /// fragment normally comes from resolving the part's own crossing, and a
    /// sum-typed field has none to resolve, so something has to compose it. The
    /// two candidates were a [`Part`] — the resolved description of one product
    /// position — carrying a pre-built fragment, and the product's member list
    /// admitting a member the compiler composes in place. **It is the second**, because of where the
    /// two steps sit: reading the parts off the model is a `&self` read with no
    /// adapter in reach, and composing anything needs one. Pre-building a
    /// fragment would have to happen during that read, which means handing the
    /// adapter to it — turning the one step that is purely a model question into
    /// a second composition site. Composing in place happens where the product
    /// is already composed, one call away from the loop that composes a
    /// top-level `Choice`.
    ///
    /// The adapter's own hooks do not change: a composed member is still paired
    /// with a `Part` describing the field it came from, so `fields`,
    /// `construct` and `value_form` keep seeing one list of
    /// `(Part, &Fragment)`. What changes is this reach's `shape`, the member
    /// list the parts reader returns, and one branch in the product path.
    ///
    /// Building it is worth doing **with** the reader that needs it, not
    /// before: the case is a callback argument whose row is
    /// `Deconstructing::Atomic` today, and `effective_callback_plan` is what
    /// routes a callback to such a row. The row's shape and that function are
    /// one mechanism, so the mechanism here and its removal are one change.
    Nested {
        /// Position of the field in the struct being taken apart.
        index: usize,
        /// How that field comes apart, compiled in place.
        shape: Box<Deconstruct>,
    },
    /// The value itself, as one part — cloned from a borrow, moved from an
    /// owned receiver. The form `DeconRecord::Identity` states and no reach
    /// could: `Field` indexes into a product and `Accessor` calls out of one,
    /// while a handle leaf is the whole value with nothing between (#613 step
    /// 10).
    ///
    /// The part's type is the receiver, so it resolves through the crossing's
    /// DEFAULT row rather than the row being compiled — otherwise the compiler
    /// re-enters the same crossing and recurses until the stack runs out. That
    /// default is the value's own converter, which is exactly what a handle
    /// leaf delivers.
    ///
    /// So an identity row works **beside** a default row and not instead of
    /// one: `whole` stays the default and `handle` states `Identity`, and a
    /// site asks for `handle`. A type whose ONLY row is the identity one has a
    /// default that is itself, which is genuinely circular and is refused as a
    /// cycle.
    Identity,
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
    /// How a value of mode `self` is held when it is reached **through** a
    /// container held as `outer`.
    ///
    /// Both layers decide it, and neither alone: reading through a shared
    /// `&Option<T>` can only ever lend its value, however the value is spelled,
    /// and a `Vec<&T>` hands its elements over while what it hands over is a
    /// borrow. Composing them is what a nested reading needs and what asking
    /// either layer on its own gets wrong in both directions — refusing the one
    /// correct fragment, or accepting one the hook cannot be fed.
    ///
    /// | held through | this mode | reached as |
    /// |---|---|---|
    /// | owned | any | itself — the container gives its contents up |
    /// | `&` | any | `&` — a shared view yields nothing stronger |
    /// | `&mut` | `&` | `&` — an exclusive view of a shared reference is still shared |
    /// | `&mut` | owned or `&mut` | `&mut` |
    pub fn through(self, outer: Mode) -> Mode {
        match outer {
            Mode::Owned => self,
            Mode::Shared => Mode::Shared,
            Mode::Exclusive => match self {
                Mode::Shared => Mode::Shared,
                Mode::Owned | Mode::Exclusive => Mode::Exclusive,
            },
        }
    }

    /// Whether a part produced in this mode can be consumed where `wanted` is
    /// required.
    ///
    /// **Owning it is enough for anything**: a value handed over can be
    /// consumed, lent, or lent mutably, so an owned production serves every
    /// edge. A borrow serves only its own kind — a `&T` cannot be consumed and
    /// cannot be written through, and a `&mut T` is not what an edge asking for
    /// `&T` was written against.
    ///
    /// The rule fires only where an adapter's fragment **disagrees** with how
    /// its crossing is spelled, since both sides are otherwise read off the same
    /// type. `prebindgen-jni`'s borrowed opaque output is the live example: it
    /// clones its referent into a fresh handle, so it produces owned where the
    /// crossing says `&T`.
    pub fn satisfies(self, wanted: Mode) -> bool {
        self == Mode::Owned || self == wanted
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

/// One Rust type and one of the two directions: the question the table answers.
#[derive(Clone, Debug)]
pub struct Crossing {
    ty: TypeRef,
    direction: Direction,
}

impl Crossing {
    /// The crossing of `ty`, doing `direction`.
    ///
    /// `ty` is kept exactly as the site spelled it, borrow and transparent
    /// wrappers included; what is normalized away is only the key
    /// [`Self::key`] derives from it.
    pub fn new(ty: TypeRef, direction: Direction) -> Self {
        Self { ty, direction }
    }

    /// The type as the site spelled it.
    pub fn spelled(&self) -> &TypeRef {
        &self.ty
    }

    /// The Rust value that crosses: the spelled type with a borrow peeled off.
    pub fn value(&self) -> &TypeRef {
        self.ty.borrow_target().unwrap_or(&self.ty)
    }

    /// Which direction this crossing takes.
    pub fn direction(&self) -> Direction {
        self.direction
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
            direction: self.direction,
        }
    }

    /// The row named `name` under this crossing's normalized key.
    pub fn row(&self, name: RecipeName) -> RecipeKey {
        self.key().row(name)
    }
}

/// A crossing identified rather than described.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CrossingKey {
    /// The Rust value that crosses, with borrow and transparent wrappers gone.
    pub ty: TypeKey,
    /// Which of the two directions.
    pub direction: Direction,
}

impl CrossingKey {
    /// The row named `name` under this crossing key.
    pub fn row(&self, name: RecipeName) -> RecipeKey {
        RecipeKey::new(self.clone(), name)
    }
}

impl fmt::Display for CrossingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.ty, self.direction)
    }
}

/// The adapter-chosen name of a recipe row within one crossing key.
///
/// Names such as `whole` and `parts` are deliberately reusable across types and
/// directions. Use [`RecipeKey`] where the identity of one table row is needed.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RecipeName(String);

impl RecipeName {
    /// A reusable row name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The name the table gives the recipe it derives for an undeclared crossing.
    pub fn derived() -> Self {
        Self("derived".to_owned())
    }

    /// The name as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RecipeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The globally unique position of one row in a [`Recipes`] table.
///
/// A [`RecipeKey`] identifies that position whether or not the table currently
/// holds a row there. A row name is meaningful only under the crossing key that
/// owns it, so the table's primary key is the pair rather than an
/// insertion-order surrogate.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RecipeKey {
    crossing: CrossingKey,
    name: RecipeName,
}

impl RecipeKey {
    /// Identify the row named `name` under `crossing`.
    fn new(crossing: CrossingKey, name: RecipeName) -> Self {
        Self { crossing, name }
    }

    /// The crossing key under which the row is filed.
    pub fn crossing(&self) -> &CrossingKey {
        &self.crossing
    }

    /// The adapter-chosen name within that crossing key.
    pub fn name(&self) -> &RecipeName {
        &self.name
    }

    fn derived(crossing: CrossingKey) -> Self {
        Self::new(crossing, RecipeName::derived())
    }
}

impl fmt::Display for RecipeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "recipe `{}` of {}", self.name, self.crossing)
    }
}

// ── What a recipe says ────────────────────────────────────────────────────

/// How a value gets across, in terms of its parts.
///
/// `OP` is the one thing a recipe states differently between the two directions:
/// what assembles the parts into a value, or takes them out of it. Nothing
/// below the top restates the direction, because a part, an optional's value and
/// a sequence's element all take the same direction as the recipe.
#[derive(Clone, Debug)]
pub enum Shape<OP> {
    /// No parts. The adapter emits the conversion itself; how many wire values
    /// that costs is the adapter's business and the table never asks.
    Atomic,
    /// Absent, or the value the `Option` wraps.
    ///
    /// The inner type is the crossing's own — the payload of `Option<T>` is
    /// `T` — so it is read off the type rather than stated here.
    Optional,
    /// A run of the collection's element.
    ///
    /// The element type is the crossing's own, and whether iterating yields
    /// owned values or borrows is the collection's business too: `Vec<T>` gives
    /// its elements up, `&[T]` and `Cow<'_, [T]>` lend them.
    Sequence,
    /// Every part contributes.
    Product(OP),
    /// Exactly one arm is live at run time. Every arm still compiles.
    Choice {
        /// One entry per alternative that crosses.
        arms: Vec<Arm<OP>>,
    },
    /// A callable the foreign side supplied, taken apart into the values that
    /// pass through it.
    ///
    /// The only shape a crossing of a callback type may be declared with:
    /// converting the arguments is what makes the callable callable, so there
    /// is no second way for such a crossing to be answered. It names no
    /// arguments — they are the ones the crossing's own type carries — and
    /// their direction is the recipe's swapped, which the table applies rather than any
    /// declaration stating it.
    Invoke,
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

/// One recipe, under whichever direction it takes.
///
/// The table holds recipes in this form so both directions share one map. An adapter
/// writes a [`Constructing`] or a [`Deconstructing`] and never builds or
/// matches on a `Recipe`.
#[derive(Clone, Debug)]
pub enum Recipe {
    /// A recipe filed under [`Direction::Construct`].
    Constructing(Constructing),
    /// A recipe filed under [`Direction::Deconstruct`].
    Deconstructing(Deconstructing),
}

impl Recipe {
    /// Whether this recipe takes a callable apart — the one shape whose parts do
    /// the other direction.
    pub fn is_invoke(&self) -> bool {
        matches!(
            self,
            Recipe::Constructing(Shape::Invoke) | Recipe::Deconstructing(Shape::Invoke)
        )
    }

    /// Which direction this recipe takes.
    pub fn direction(&self) -> Direction {
        match self {
            Recipe::Constructing(_) => Direction::Construct,
            Recipe::Deconstructing(_) => Direction::Deconstruct,
        }
    }
}

// ── The table ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Entry {
    key: RecipeKey,
    recipe: Recipe,
    /// The type as the declaration spelled it, which is what the checks read
    /// fields and alternatives off.
    ty: TypeRef,
    default: bool,
}

/// The resolved table. Built by [`RecipesBuilder`], checked once, then
/// immutable.
#[derive(Clone, Debug, Default)]
pub struct Recipes {
    recipes: HashMap<CrossingKey, Vec<Entry>>,
}

impl Recipes {
    /// Start describing a table.
    pub fn builder() -> RecipesBuilder {
        RecipesBuilder::default()
    }

    /// Which recipe names this crossing has, in declaration order.
    ///
    /// Empty for a crossing nobody declared, which still has the recipe
    /// [`Self::recipe`] derives.
    pub fn names_of(&self, key: &CrossingKey) -> Vec<&RecipeName> {
        self.recipes
            .get(key)
            .map(|recipes| recipes.iter().map(|e| e.key.name()).collect())
            .unwrap_or_default()
    }

    /// The key of one named row under a crossing, or `None` if it was never declared.
    pub fn key_of(&self, crossing: &CrossingKey, name: &RecipeName) -> Option<&RecipeKey> {
        self.recipes
            .get(crossing)?
            .iter()
            .find(|e| e.key.name() == name)
            .map(|e| &e.key)
    }

    /// One row by its globally unique key.
    pub fn get(&self, key: &RecipeKey) -> Option<&Recipe> {
        self.recipes
            .get(key.crossing())?
            .iter()
            .find(|e| &e.key == key)
            .map(|e| &e.recipe)
    }

    /// The recipe a site uses when it names none.
    ///
    /// With one declared recipe that recipe is the default; with several it is the
    /// one declared through [`RecipesBuilder::declare_default`]. `None` for a
    /// crossing nobody declared.
    pub fn default_of(&self, key: &CrossingKey) -> Option<&RecipeKey> {
        let recipes = self.recipes.get(key)?;
        match recipes.as_slice() {
            [only] => Some(&only.key),
            many => many.iter().find(|e| e.default).map(|e| &e.key),
        }
    }

    /// The default recipe for a crossing: the declared one, or the recipe derived
    /// from the type's kind.
    ///
    /// A callback type derives [`Shape::Invoke`], the only shape such a
    /// crossing takes, so an adapter reaches one here the same way it reaches
    /// every other recipe.
    pub fn recipe(&self, crossing: &Crossing) -> (RecipeKey, Cow<'_, Recipe>) {
        let key = crossing.key();
        match self.default_of(&key) {
            Some(recipe_key) => {
                let recipe_key = recipe_key.clone();
                let recipe = self
                    .get(&recipe_key)
                    .expect("default names a declared recipe");
                (recipe_key, Cow::Borrowed(recipe))
            }
            None => (RecipeKey::derived(key), Cow::Owned(derive(crossing))),
        }
    }
}

/// The arity recipe a crossing gets when nobody declared one.
fn derive(crossing: &Crossing) -> Recipe {
    let value = crossing.value();
    let kind = if value.callback_args().is_some() {
        DerivedKind::Invoke
    } else if value.optional_inner().is_some() {
        DerivedKind::Optional
    } else if sequence_elem(value).is_some() {
        DerivedKind::Sequence
    } else {
        DerivedKind::Atomic
    };
    match crossing.direction {
        Direction::Construct => Recipe::Constructing(kind.shape()),
        Direction::Deconstruct => Recipe::Deconstructing(kind.shape()),
    }
}

/// The shape a type's own kind implies, before either direction is chosen. Shared so
/// the two arms of [`derive`] cannot drift apart.
#[derive(Copy, Clone)]
enum DerivedKind {
    Atomic,
    Optional,
    Sequence,
    Invoke,
}

impl DerivedKind {
    fn shape<OP>(self) -> Shape<OP> {
        match self {
            DerivedKind::Atomic => Shape::Atomic,
            DerivedKind::Optional => Shape::Optional,
            DerivedKind::Sequence => Shape::Sequence,
            DerivedKind::Invoke => Shape::Invoke,
        }
    }
}

/// The element of a run, including the fixed-size array the model's own
/// [`TypeRef::sequence_elem`](crate::flat::TypeRef::sequence_elem) leaves out.
pub(crate) fn sequence_elem(ty: &TypeRef) -> Option<&TypeRef> {
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
    recipes: HashMap<CrossingKey, Vec<Entry>>,
    /// A recipe declared twice under one name, reported by [`Self::build`] rather
    /// than by overwriting silently.
    duplicates: Vec<RecipeKey>,
}

impl RecipesBuilder {
    /// Add one recipe for `ty`.
    ///
    /// Which direction the recipe is filed under is the shape's own, so nothing states
    /// it twice and the two cannot disagree. Declaring a second recipe for one
    /// crossing is how a type offers a choice, and the site is what picks
    /// between them — at which point one of the recipes has to be declared through
    /// [`Self::declare_default`].
    pub fn declare<OP: Operation>(
        &mut self,
        ty: TypeRef,
        name: RecipeName,
        shape: Shape<OP>,
    ) -> &mut Self {
        self.insert(ty, name, OP::into_recipe(shape), false)
    }

    /// Add one recipe and make it the recipe used where a site names none.
    ///
    /// Needed only once a crossing has more than one recipe: with a single recipe
    /// that recipe is the default, so the common case never says so.
    pub fn declare_default<OP: Operation>(
        &mut self,
        ty: TypeRef,
        name: RecipeName,
        shape: Shape<OP>,
    ) -> &mut Self {
        self.insert(ty, name, OP::into_recipe(shape), true)
    }

    fn insert(
        &mut self,
        ty: TypeRef,
        name: RecipeName,
        recipe: Recipe,
        default: bool,
    ) -> &mut Self {
        let crossing = Crossing::new(ty.clone(), recipe.direction()).key();
        let key = crossing.row(name);
        let entries = self.recipes.entry(crossing).or_default();
        if entries.iter().any(|e| e.key == key) {
            self.duplicates.push(key);
            return self;
        }
        entries.push(Entry {
            key,
            recipe,
            ty,
            default,
        });
        self
    }

    /// Check the table and freeze it.
    ///
    /// Every problem is reported, not just the first. The checks are the ones
    /// no type can express: whether a recipe names something the model has,
    /// whether a crossing with several recipes says which of them wins, and
    /// whether a recipe reaches its own crossing.
    pub fn build(self, model: &Flat) -> Result<Recipes, Vec<RecipeError>> {
        let table = Recipes {
            recipes: self.recipes,
        };
        let mut errors: Vec<RecipeError> = self
            .duplicates
            .into_iter()
            .map(|recipe| RecipeError::Duplicate { recipe })
            .collect();

        for (key, entries) in &table.recipes {
            if entries.len() > 1 {
                let defaults: Vec<RecipeName> = entries
                    .iter()
                    .filter(|e| e.default)
                    .map(|e| e.key.name().clone())
                    .collect();
                if defaults.len() != 1 {
                    errors.push(RecipeError::NoDefault {
                        crossing: key.clone(),
                        defaults,
                    });
                }
            }
            for entry in entries {
                let crossing = Crossing::new(entry.ty.clone(), entry.recipe.direction());
                // A callback type takes `Invoke` and nothing else: converting
                // the arguments is what makes the callable callable, so there
                // is no second answer for such a crossing.
                if crossing.value().callback_args().is_some() && !entry.recipe.is_invoke() {
                    errors.push(RecipeError::CallbackShape {
                        recipe: entry.key.clone(),
                    });
                    continue;
                }
                let mut check = Check {
                    model,
                    recipe: &entry.key,
                    errors: &mut errors,
                    arm_fields: None,
                };
                check.recipe(&entry.ty, &entry.recipe);
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

// ── Checking one recipe ──────────────────────────────────────────────────────

struct Check<'a, 'e> {
    model: &'a Flat,
    recipe: &'a RecipeKey,
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
            recipe: self.recipe.clone(),
            index,
            len,
        });
    }

    fn not_a_product(&mut self) {
        self.push(RecipeError::NotAProduct {
            recipe: self.recipe.clone(),
        });
    }

    fn not_a_callback(&mut self) {
        self.push(RecipeError::WrongShape {
            recipe: self.recipe.clone(),
            shape: "Invoke",
            wanted: "a callback type",
        });
    }

    fn wrong_arity(&mut self, optional: bool) {
        let (shape, wanted) = if optional {
            ("Optional", "an `Option`")
        } else {
            ("Sequence", "a `Vec`, slice or array")
        };
        self.push(RecipeError::WrongShape {
            recipe: self.recipe.clone(),
            shape,
            wanted,
        });
    }

    /// Every part crossing this recipe reaches, checking what it names on the way.
    fn recipe(&mut self, ty: &TypeRef, recipe: &Recipe) -> Vec<Crossing> {
        let direction = recipe.direction();
        // The one place the two directions swap: the Rust side holds the values a
        // callback's arguments carry and pushes them out through the call.
        if recipe.is_invoke() {
            let value = Crossing::new(ty.clone(), direction);
            let Some(args) = value.value().callback_args().map(<[_]>::to_vec) else {
                self.not_a_callback();
                return Vec::new();
            };
            return args
                .into_iter()
                .map(|ty| Crossing::new(ty, direction.swap()))
                .collect();
        }
        let parts = match recipe {
            Recipe::Constructing(shape) => self.constructing(ty, shape),
            Recipe::Deconstructing(shape) => self.deconstructing(ty, shape),
        };
        parts
            .into_iter()
            .map(|ty| Crossing::new(ty, direction))
            .collect()
    }

    /// The inner crossing an arity shape reaches, or a refusal naming the
    /// shape the type does not have.
    ///
    /// Stated nowhere in the recipe: the payload of `Option<T>` is `T` and the
    /// element of `Vec<T>` is `T`, so declaring either would only be a fact
    /// that could disagree with the type.
    fn arity_inner(&mut self, ty: &TypeRef, optional: bool) -> Vec<TypeRef> {
        let inner = if optional {
            ty.optional_inner()
        } else {
            sequence_elem(ty)
        };
        match inner {
            Some(inner) => vec![inner.clone()],
            None => {
                self.wrong_arity(optional);
                Vec::new()
            }
        }
    }

    fn constructing(&mut self, ty: &TypeRef, shape: &Constructing) -> Vec<TypeRef> {
        match shape {
            Shape::Atomic => Vec::new(),
            Shape::Optional => self.arity_inner(ty, true),
            Shape::Sequence => self.arity_inner(ty, false),
            // Reached through `is_invoke` above, never here.
            Shape::Invoke => Vec::new(),
            Shape::Product(op) => self.construct(ty, op),
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
                    parts.extend(self.construct(ty, &arm.op));
                    self.arm_fields = outer;
                }
                parts
            }
        }
    }

    fn deconstructing(&mut self, ty: &TypeRef, shape: &Deconstructing) -> Vec<TypeRef> {
        match shape {
            Shape::Atomic => Vec::new(),
            Shape::Optional => self.arity_inner(ty, true),
            Shape::Sequence => self.arity_inner(ty, false),
            // Reached through `is_invoke` above, never here.
            Shape::Invoke => Vec::new(),
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

    fn construct(&mut self, ty: &TypeRef, op: &Construct) -> Vec<TypeRef> {
        match op {
            Construct::Call(func) => match self.function(func) {
                Some(f) => {
                    let parts = f.params.iter().map(|p| p.ty.clone()).collect();
                    if !constructor_of(f, ty) {
                        self.push(RecipeError::NotAConstructor {
                            recipe: self.recipe.clone(),
                            func: func.clone(),
                        });
                    }
                    parts
                }
                None => Vec::new(),
            },
            Construct::Fields => match self.fields(ty) {
                Some(fields) => fields.into_iter().map(|f| f.ty).collect(),
                None => {
                    self.not_a_product();
                    Vec::new()
                }
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
                // The part IS the receiver, so it reaches no new type: there
                // is no accessor to check and nothing further to resolve. It
                // must not push `ty` either — that reads as the crossing
                // depending on itself, and validation reports a cycle. The
                // value's own converter comes from its `whole` row, which is a
                // different recipe.
                Reach::Identity => {}
                Reach::Accessor(func) => {
                    let Some(f) = self.function(func) else {
                        continue;
                    };
                    let ret = f.ret.clone();
                    if !accessor_of(f, ty) {
                        self.push(RecipeError::NotAnAccessor {
                            recipe: self.recipe.clone(),
                            func: func.clone(),
                        });
                    }
                    out.push(ret);
                }
                // Each hop resolves against the previous field's type, so a
                // chain is validated exactly as the accesses it renders.
                // The field's own type is reached, and the nested shape's
                // reaches are validated against it — the same walk one level in.
                Reach::Nested { index, shape } => {
                    let Some(fields) = self.fields(ty) else {
                        self.not_a_product();
                        continue;
                    };
                    match fields.get(*index) {
                        Some(field) => {
                            let inner = field.ty.clone();
                            // A `Deconstruct` reads parts off a product. A
                            // field that is not one contributes nothing at all
                            // through this reach, so it is refused here rather
                            // than silently dropping its leaves (#658 review).

                            match shape.as_ref() {
                                Deconstruct::Fields(inner_reaches) => {
                                    out.extend(self.reaches(&inner, inner_reaches));
                                }
                                Deconstruct::ValueForm { parts, .. } => {
                                    out.extend(self.reaches(&inner, parts));
                                }
                            }
                        }
                        None => self.out_of_range(*index, fields.len()),
                    }
                }
                Reach::Path(indices) => {
                    let mut at: TypeRef = ty.clone();
                    let mut ok = true;
                    for index in indices {
                        let Some(fields) = self.fields(&at) else {
                            self.not_a_product();
                            ok = false;
                            break;
                        };
                        match fields.get(*index) {
                            Some(field) => at = field.ty.clone(),
                            None => {
                                self.out_of_range(*index, fields.len());
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok {
                        out.push(at);
                    }
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

/// Whether `f` builds a value of `ty` — `fn(..) -> T`, or `fn(..) -> Result<T, E>`
/// for a construction that can fail.
///
/// The constructing twin of [`accessor_of`]. A recipe names the function and the
/// model supplies everything else, so the one thing left to check is that the
/// function it names produces the type the recipe is filed under.
fn constructor_of(f: &Function, ty: &TypeRef) -> bool {
    // A fallible constructor is recognised by its return type, which is the same
    // place the parts' fallibility is read from — the recipe never states it.
    let built = match f.ret.fallible_parts() {
        Some((ok, _err)) => ok,
        None => &f.ret,
    };
    value_key(built) == value_key(ty)
}

/// The type's identity with a borrow and any transparent wrapper gone — what
/// [`Crossing::key`] keys a recipe by, without a direction attached.
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

/// A recipe may not reach its own crossing, directly or through any chain of
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
        .recipes
        .values()
        .flat_map(|entries| {
            entries
                .iter()
                .map(|e| Crossing::new(e.ty.clone(), e.recipe.direction()))
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

/// Every crossing a crossing's recipes reach.
///
/// Every recipe counts, not only the default: a site may name any of them, so a
/// cycle through the recipe nobody happens to default to is still a cycle.
fn successors(model: &Flat, table: &Recipes, crossing: &Crossing) -> Vec<Crossing> {
    let key = crossing.key();
    let recipes: Vec<(RecipeKey, TypeRef, Recipe)> = match table.recipes.get(&key) {
        Some(entries) => entries
            .iter()
            .map(|e| (e.key.clone(), e.ty.clone(), e.recipe.clone()))
            .collect(),
        None => vec![(
            key.row(RecipeName::derived()),
            crossing.spelled().clone(),
            derive(crossing),
        )],
    };
    // Anything a recipe names wrongly is reported by the per-recipe pass; here the
    // walk only needs the parts a well-formed reading of the recipe reaches.
    let mut discarded = Vec::new();
    recipes
        .into_iter()
        .flat_map(|(recipe_key, ty, recipe)| {
            let mut check = Check {
                model,
                recipe: &recipe_key,
                errors: &mut discarded,
                arm_fields: None,
            };
            check.recipe(&ty, &recipe)
        })
        .collect()
}

// ── What the table refuses ────────────────────────────────────────────────

/// A problem [`RecipesBuilder::build`] found.
#[derive(Debug)]
pub enum RecipeError {
    /// A recipe's parts reach the recipe's own crossing.
    ///
    /// The path is a chain of keys because it may pass through a callback, and
    /// so swap directions.
    Cycle {
        /// The chain, starting and ending at the crossing that repeats.
        path: Vec<CrossingKey>,
    },
    /// A crossing has several recipes and none, or more than one, was declared the
    /// default.
    NoDefault {
        /// The crossing whose recipes disagree.
        crossing: CrossingKey,
        /// The recipes that claimed to be the default.
        defaults: Vec<RecipeName>,
    },
    /// One row was declared twice.
    Duplicate {
        /// The row declared twice.
        recipe: RecipeKey,
    },
    /// A recipe named a constructor or accessor the model does not have.
    UnknownFunction {
        /// The row that named it.
        recipe: RecipeKey,
        /// The name that resolved to nothing.
        func: syn::Ident,
    },
    /// A constructor was named where what it returns is not the recipe type.
    ///
    /// The constructing twin of [`NotAnAccessor`](Self::NotAnAccessor). A
    /// `Result<T, E>` return counts as building a `T`, because that is where a
    /// construction's fallibility is read from.
    NotAConstructor {
        /// The row that named the function.
        recipe: RecipeKey,
        /// The function whose return type does not match.
        func: syn::Ident,
    },
    /// An accessor was named where its first parameter is not the recipe type.
    NotAnAccessor {
        /// The row that named the function.
        recipe: RecipeKey,
        /// The function whose first parameter does not match.
        func: syn::Ident,
    },
    /// A [`Reach::Field`] or an [`Arm::alternative`] is past the end of what the
    /// model holds for that type.
    OutOfRange {
        /// The row that names the index.
        recipe: RecipeKey,
        /// The index the recipe named.
        index: usize,
        /// How many the model holds.
        len: usize,
    },
    /// A shape was declared for a type that cannot take it: `Optional` on a
    /// type that is not an `Option`, `Sequence` on one that is not a run, or
    /// `Invoke` on one that is not a callback.
    ///
    /// The arity shapes read their inner type off the crossing rather than
    /// stating it, so this is where a mismatch surfaces.
    WrongShape {
        /// The row whose shape is wrong.
        recipe: RecipeKey,
        /// The shape as declared.
        shape: &'static str,
        /// What the type would have had to be.
        wanted: &'static str,
    },
    /// A product or choice recipe was declared for a type the model gives no parts.
    NotAProduct {
        /// The row whose type has no parts.
        recipe: RecipeKey,
    },
    /// A site asked for a recipe row that was never declared.
    UnknownRecipe {
        /// Where the value crosses.
        site: Site,
        /// The complete key the site asked for.
        recipe: RecipeKey,
    },
    /// A caller named a recipe row the table does not have.
    ///
    /// Distinct from [`UnknownRecipe`](Self::UnknownRecipe), which is a **site**
    /// asking for one. This is a caller compiling a named recipe directly through
    /// [`Compiler::recipe_of`](crate::recipe::Compiler::recipe_of), where there is no
    /// site to name — an adapter checking a recipe it declared conditionally, and
    /// getting the condition wrong.
    NoSuchRecipe {
        /// The complete key the caller asked for.
        recipe: RecipeKey,
    },
    /// Two declarations of equal precedence bound one site to different recipes.
    Rebound {
        /// The site both named.
        site: Site,
        /// The precedence they share.
        origin: Origin,
    },
    /// A part yields a different Rust type from the one its edge needs.
    ///
    /// The type half of the composition contract; [`Composition`](Self::Composition)
    /// is the ownership half. Both are checked at every part, this one first: a
    /// part producing the wrong value is wrong however it is held.
    ///
    /// **Every** part, in the full sense — a product's fields and constructor
    /// arguments, an optional's value, a run's element, and a callback's
    /// arguments. Each is a crossing the recipe reaches, and each is checked in
    /// the one place the driver reaches one.
    ComposedType {
        /// The part's own site.
        site: Site,
        /// Which part of the recipe.
        part: usize,
        /// What the edge requires: a constructor's parameter, a field, an
        /// accessor's receiver, an optional's value, a run's element, or a
        /// callback argument.
        wanted: TypeKey,
        /// What the part's fragment says it produces.
        got: TypeKey,
    },
    /// A part yields something the edge it feeds cannot consume.
    Composition {
        /// The part's own site.
        site: Site,
        /// Which part of the recipe.
        part: usize,
        /// How the edge requires it held: a product's declared parameter or
        /// field mode, an optional's value, the mode a collection lends its
        /// elements in, or a callback argument's own.
        wanted: Mode,
        /// What the part's fragment produces.
        got: Mode,
    },
    /// A site needs a value that outlives the call and got a borrowed one.
    Validity {
        /// Where the value crosses.
        site: Site,
        /// The weakest validity the site's role accepts.
        needed: crate::recipe::Validity,
        /// What the root fragment produces.
        got: crate::recipe::Validity,
    },
    /// A callback crossing was declared with a shape other than
    /// [`Shape::Invoke`].
    ///
    /// Converting the arguments is what makes a callable callable, so taking it
    /// apart is the only thing an adapter can do with one.
    CallbackShape {
        /// The invalid callback row.
        recipe: RecipeKey,
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
                "{crossing} has several recipes and {} of them is the default; \
                 declare exactly one with `declare_default`",
                if defaults.is_empty() {
                    "none".to_owned()
                } else {
                    format!("{}", defaults.len())
                }
            ),
            RecipeError::Duplicate { recipe } => {
                write!(f, "{recipe} is declared twice")
            }
            RecipeError::UnknownFunction { recipe, func } => write!(
                f,
                "{recipe} names `{func}`, which no #[prebindgen] source declares"
            ),
            RecipeError::NotAConstructor { recipe, func } => write!(
                f,
                "{recipe} builds the value with `{func}`, which does not return that type"
            ),
            RecipeError::NotAnAccessor { recipe, func } => write!(
                f,
                "{recipe} reaches a part through `{func}`, whose first parameter is not that type"
            ),
            RecipeError::OutOfRange { recipe, index, len } => {
                write!(f, "{recipe} names index {index}, and the model holds {len}")
            }
            RecipeError::WrongShape {
                recipe,
                shape,
                wanted,
            } => write!(
                f,
                "{recipe} declares `{shape}`, but the type is not {wanted}"
            ),
            RecipeError::NotAProduct { recipe } => write!(
                f,
                "{recipe} takes its type apart, and the model gives that type no parts"
            ),
            RecipeError::NoSuchRecipe { recipe } => write!(
                f,
                "{recipe} does not exist — it was compiled by name, not through a site"
            ),
            RecipeError::UnknownRecipe { site, recipe } => {
                write!(f, "{site} asks for {recipe}, which does not exist")
            }
            RecipeError::Rebound { site, origin } => write!(
                f,
                "{site} is bound to two different recipes by {origin}; one of them has to \
                 be written at a higher precedence"
            ),
            RecipeError::ComposedType {
                site,
                part,
                wanted,
                got,
            } => write!(
                f,
                "part {part} of {site} needs a `{wanted}` and its fragment produces a `{got}`"
            ),
            RecipeError::Composition {
                site,
                part,
                wanted,
                got,
            } => write!(
                f,
                "part {part} of {site} needs `{wanted}` and its fragment produces `{got}`"
            ),
            RecipeError::Validity { site, needed, got } => write!(
                f,
                "{site} needs a {needed} value and its fragment produces a {got} one"
            ),
            RecipeError::CallbackShape { recipe } => write!(
                f,
                "{recipe} is not `Invoke`; a callback is always taken apart into its \
                 arguments, so that is the only shape such a crossing takes"
            ),
        }
    }
}

impl std::error::Error for RecipeError {}
