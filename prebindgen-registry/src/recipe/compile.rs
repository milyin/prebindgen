//! Turning recipes into whatever an adapter emits from.
//!
//! [`Recipes`] says how a value gets across in terms of its parts; an adapter
//! says what that costs on the wire. Its answer for one recipe row is a **fragment**,
//! a type the adapter defines and this module never looks inside. A fragment is
//! not a wire value: it may occupy none, one or several, and the count is never
//! asked for.
//!
//! An adapter writes [`Compile`], one hook per recipe shape. [`Compiler`] drives
//! the recursion over the table, builds each recipe's fragment **once**, and hands
//! every hook the fragments its parts already produced — so an adapter says
//! what one crossing means and never writes the walk.
//!
//! # Per crossing, not per site
//!
//! A fragment is built once per crossing and reused at every site that crossing
//! appears at. That is what makes emitted code shared without a converter
//! table, and it rests on one assumption: **a fragment is context-free**. Where
//! an adapter appears to need site context, the site is wrapping the fragment
//! rather than the fragment differing, and [`Compile::plan`] — the one hook
//! called per site — is where that wrapping belongs.
//!
//! A crossing here is the type **as the site spelled it**, which is finer than
//! the [`RecipeKey`] a row is declared under. `Sample`, `&Sample` and `Box<Sample>`
//! find one row, because the same row assembles all three — and they get
//! three fragments, because taking a value out of a pointer, borrowing through
//! one, and rebuilding a `Box` are three different pieces of Rust. Sharing the
//! recipe is the declaration's business; sharing the code is not.

use std::{collections::HashMap, fmt, rc::Rc};

use super::{
    Bindings, Bound, Construct, Crossing, Deconstruct, Direction, Mode, Reach, Recipe, RecipeError,
    RecipeKey, RecipeName, Recipes, Role, Shape, Site,
};
use crate::{
    flat::{Alternative, Field, Flat, Function, Type, TypeKey, TypeKind, TypeRef},
    Emit, FragmentId,
};

/// What a fragment produces, which is the only thing the registry reads out of
/// one.
pub trait Carrier {
    /// The Rust value this fragment yields.
    fn yields(&self) -> Yield;
}

/// The Rust value a fragment produces.
#[derive(Clone, Debug)]
pub struct Yield {
    /// The type produced.
    pub ty: TypeKey,
    /// Handed over, or reached through a borrow.
    pub mode: Mode,
    /// How long what the fragment produces stays usable.
    pub validity: Validity,
}

/// How long what a fragment produces stays usable.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Validity {
    /// The foreign side may keep it. Destroying it needs nothing else alive.
    SelfSufficient,
    /// Valid only while the value it was read from is alive.
    Borrowed,
}

impl Validity {
    /// Whether a value of this validity can be used where `needed` is required.
    ///
    /// Only one direction fails: something the foreign side will keep cannot be
    /// borrowed from a value the call is about to drop.
    pub fn satisfies(self, needed: Validity) -> bool {
        needed == Validity::Borrowed || self == Validity::SelfSufficient
    }
}

impl fmt::Display for Validity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Validity::SelfSufficient => "self-sufficient",
            Validity::Borrowed => "borrowed",
        })
    }
}

/// Where a hook is: which recipe it is compiling.
#[derive(Copy, Clone, Debug)]
pub struct At<'a> {
    /// The crossing the recipe answers.
    pub crossing: &'a Crossing,
    /// The globally unique row being compiled.
    pub recipe: &'a RecipeKey,
}

/// What every hook receives: a read-only view of the model and the table, and
/// the capability to emit Rust.
///
/// A hook asks the registry nothing and records nothing in it. What it produces
/// is its fragment, and what the registry reads of that is
/// [`Carrier::yields`].
pub struct Cx<'a> {
    model: &'a Flat,
    recipes: &'a Recipes,
    emit: &'a Emit,
}

impl Cx<'_> {
    /// The Rust-emission capability.
    ///
    /// A fragment is generated Rust, so compiling one is an emission callback
    /// and is handed the key exactly as `Prebindgen`'s `on_*` methods are.
    pub fn emit(&self) -> &Emit {
        self.emit
    }

    /// The model every structural fact is read off.
    pub fn model(&self) -> &Flat {
        self.model
    }

    /// The table, for asking about a crossing without demanding it.
    pub fn table(&self) -> &Recipes {
        self.recipes
    }

    /// Which reusable recipe names are declared under a crossing key.
    ///
    /// Demands nothing, so an adapter may ask about an alternative it will not
    /// take. Empty for a crossing nobody declared, which still has a derived
    /// recipe.
    pub fn recipe_names(&self, crossing: &Crossing) -> Vec<&RecipeName> {
        self.recipes.names_of(&crossing.key())
    }
}

/// One part, as the registry resolved it.
///
/// Nothing here was stated by the recipe: every field was read off the model,
/// which is why an adapter can use it without checking it against anything.
#[derive(Clone, Debug)]
pub struct Part<'a> {
    /// Where the part came from, and what to call or read to get it.
    pub from: PartSource<'a>,
    /// The part's Rust type.
    pub ty: TypeRef,
    /// Handed over, or reached through a borrow.
    pub mode: Mode,
    /// The segment an adapter mangles into a destination-language name.
    pub name: String,
}

/// Where one part comes from.
#[derive(Copy, Clone, Debug)]
pub enum PartSource<'a> {
    /// Parameter `index` of the constructor being called.
    Argument {
        /// Position in the constructor's parameter list.
        index: usize,
    },
    /// Field `index` of the value, read directly.
    Field {
        /// Position in the struct or in the arm's payload.
        index: usize,
        /// The field itself.
        field: &'a Field,
    },
    /// This accessor call.
    Accessor {
        /// The function to call.
        func: &'a Function,
    },
}

/// A product's parts, each already compiled.
pub type Parts<'a, C> = &'a [(Part<'a>, &'a <C as Compile>::Fragment)];

/// Shorthand for what every fragment-producing hook returns.
pub type Frag<C> = Result<<C as Compile>::Fragment, <C as Compile>::Error>;

/// The adapter's half of code generation: what one crossing costs on the wire.
///
/// One hook per recipe shape, and one per operation, so an adapter never matches
/// on [`Construct`] or [`Deconstruct`] — the registry calls only the hook that
/// suits the recipe. Every hook is handed model elements rather than a copy of
/// what the recipe said about them.
pub trait Compile {
    /// The adapter's answer for one recipe: what crosses, how it is encoded, what
    /// it costs to clean up.
    ///
    /// Opaque to the registry, which only ever asks it what Rust value it
    /// yields. Not a wire value — a fragment may occupy none, one or several.
    type Fragment: Carrier;
    /// The adapter's answer for one site: a root fragment plus the signature,
    /// the call and the cleanup around it.
    type Plan;
    /// What the adapter reports when it cannot answer.
    type Error;

    /// No parts: the adapter emits the conversion itself.
    fn atomic(&mut self, cx: &mut Cx<'_>, at: At<'_>) -> Frag<Self>;

    /// Absent, or the inner fragment's value.
    fn optional(&mut self, cx: &mut Cx<'_>, at: At<'_>, inner: &Self::Fragment) -> Frag<Self>;

    /// A run of the inner fragment's value.
    ///
    /// `elements` is derived from the collection type, not declared, so an
    /// adapter reads it rather than checking it against anything.
    fn sequence(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        elements: Mode,
        inner: &Self::Fragment,
    ) -> Frag<Self>;

    /// Call `func` to assemble the value. Its parameters are `args`.
    fn construct(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        func: &Function,
        args: Parts<'_, Self>,
    ) -> Frag<Self>;

    /// Read the parts off the value where it stands.
    fn fields(&mut self, cx: &mut Cx<'_>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self>;

    /// Call `func` once, then read the parts off what it returned.
    fn value_form(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        func: &Function,
        parts: Parts<'_, Self>,
    ) -> Frag<Self>;

    /// Every arm, always.
    ///
    /// A choice is a run-time alternative, so the adapter is never offered one
    /// of them to pick. Each arm arrives already composed by one of the four
    /// hooks above, which is why this hook needs no direction of its own.
    fn choice(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        arms: &[(&Alternative, &Self::Fragment)],
    ) -> Frag<Self>;

    /// A callback, taken apart into the values that pass through it.
    ///
    /// `result` is `None` for every callback the model can describe today; it
    /// is the position issue #216's return value lands in.
    fn callback(
        &mut self,
        cx: &mut Cx<'_>,
        at: At<'_>,
        args: &[&Self::Fragment],
        result: Option<&Self::Fragment>,
    ) -> Frag<Self>;

    /// The weakest validity this role accepts in **this** target.
    ///
    /// Not the registry's to decide, because it follows from the target's
    /// ownership model. C hands out a `*const T` for a zero-copy accessor and
    /// its contract says the caller neither frees nor outlives it, so a
    /// borrowed return is correct there. The JVM keeps what it is given, so
    /// JniGen clones instead and a borrowed return would be a use-after-free.
    ///
    /// The default is the strict reading: anything the foreign side may keep
    /// past the call must be self-sufficient, and only a position that lives
    /// for the duration of the call may borrow.
    fn tolerates(&self, role: &Role) -> Validity {
        match role {
            Role::Return | Role::Error | Role::Const => Validity::SelfSufficient,
            Role::Param { .. } | Role::Receiver | Role::CallbackArg { .. } | Role::Part { .. } => {
                Validity::Borrowed
            }
        }
    }

    /// Wrap the site's root fragment into a plan — the signature, the call, the
    /// cleanup.
    ///
    /// The only hook called once per site; every hook above is called once per
    /// recipe, however many sites reuse it.
    fn plan(
        &mut self,
        cx: &mut Cx<'_>,
        bound: &Bound,
        root: &Self::Fragment,
    ) -> Result<Self::Plan, Self::Error>;
}

/// What a compilation reports when it cannot finish.
#[derive(Debug)]
pub enum CompileError<E> {
    /// The adapter could not answer for a recipe or a site.
    Adapter(E),
    /// The table and the model disagree with what a site asked for.
    ///
    /// Boxed: a `RecipeError` names a site and a crossing, which makes it much
    /// the larger of the two variants, and this is the rare one.
    Recipe(Box<RecipeError>),
}

impl<E: fmt::Display> fmt::Display for CompileError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Adapter(e) => write!(f, "{e}"),
            CompileError::Recipe(e) => write!(f, "{e}"),
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for CompileError<E> {}

impl<E> From<RecipeError> for CompileError<E> {
    fn from(e: RecipeError) -> Self {
        CompileError::Recipe(Box::new(e))
    }
}

type Built<C> = Result<Rc<<C as Compile>::Fragment>, CompileError<<C as Compile>::Error>>;

/// What a compilation has produced, apart from its plans.
///
/// Carried between runs by an adapter whose view of the model is lent to it per
/// callback rather than held: such an adapter is a different type on every call
/// and so cannot keep one [`Compiler`], but the fragments it built borrow
/// nothing and outlive any of them.
pub struct Compiled<F> {
    fragments: HashMap<FragmentId, Rc<F>>,
    /// Which row answered when a crossing was compiled **as a whole**, rather
    /// than as one part of a container. Recorded by [`Compiler::crossing`],
    /// which is the only entry point that consults the crossing's default —
    /// so [`Compiled::fragment`] can give back that same answer instead of
    /// choosing between the recipes a crossing happens to have.
    defaults: HashMap<(TypeKey, Direction), RecipeKey>,
}

impl<F> Default for Compiled<F> {
    fn default() -> Self {
        Self {
            fragments: HashMap::new(),
            defaults: HashMap::new(),
        }
    }
}

/// Cheap: a fragment sits behind an `Rc`, so a clone shares the fragments
/// rather than copying them, and needs nothing of `F`.
impl<F> Clone for Compiled<F> {
    fn clone(&self) -> Self {
        Self {
            fragments: self.fragments.clone(),
            defaults: self.defaults.clone(),
        }
    }
}

impl<F> Compiled<F> {
    /// How many fragments have been built, making memoization observable.
    pub fn len(&self) -> usize {
        self.fragments.len()
    }

    /// Whether nothing has been compiled yet.
    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    /// The fragment for one crossing, compiled **as a whole**.
    ///
    /// What an adapter's emitters ask once they stop reading the converter
    /// table. The answer is the one [`Compiler::crossing`] built — the
    /// crossing's default recipe — and not one of the recipes that answer for this
    /// type only as a part of some container. A caller that wants a particular
    /// recipe has the recipe and asks through [`Self::recipe_fragment`].
    ///
    /// `None` means no site ever crossed this type in this direction.
    ///
    /// Handed back as the `Rc` it is stored under: an adapter that reads its
    /// store while compilation is still filling it cannot hold a borrow across
    /// the next write, and a fragment holds a whole `syn::ItemFn` that a
    /// recursive lookup should not be copying.
    pub fn fragment(&self, ty: &TypeKey, direction: Direction) -> Option<Rc<F>> {
        let recipe = self.defaults.get(&(ty.clone(), direction))?;
        self.recipe_fragment(ty, recipe)
    }

    /// Record a fragment an adapter built **without** the compiler, as this
    /// crossing's whole answer.
    ///
    /// The escape hatch for a crossing the adapter still answers by hand:
    /// `Compiler::crossing` never saw it, so nothing would file it, and
    /// [`Self::fragment`] would have no answer to give. Recording it keeps the
    /// adapter's emitters on one lookup instead of a per-site fall-back to
    /// whatever else knows.
    pub fn record(&mut self, recipe: RecipeKey, fragment: F) {
        let ty = recipe.crossing().ty.clone();
        let direction = recipe.crossing().direction;
        self.fragments.insert(
            FragmentId::from_parts(ty.clone(), recipe.clone()),
            std::rc::Rc::new(fragment),
        );
        self.defaults.insert((ty, direction), recipe);
    }

    /// The fragment for one spelled type and one row key.
    pub fn recipe_fragment(&self, ty: &TypeKey, recipe: &RecipeKey) -> Option<Rc<F>> {
        self.fragments
            .get(&FragmentId::from_parts(ty.clone(), recipe.clone()))
            .cloned()
    }

    /// Every fragment this compilation built, in a deterministic order.
    ///
    /// What an adapter emits from once it stops routing its conversions back
    /// through the converter table — handed to
    /// [`write_rust`](crate::write::write_rust) directly. The order is by
    /// crossing key and then by recipe name, so a file written from this is
    /// stable across runs.
    pub fn fragments(&self) -> Vec<&F> {
        let mut keyed: Vec<(&FragmentId, &Rc<F>)> = self.fragments.iter().collect();
        keyed.sort_by(|a, b| {
            (a.0.spelling(), a.0.direction(), a.0.recipe().name()).cmp(&(
                b.0.spelling(),
                b.0.direction(),
                b.0.recipe().name(),
            ))
        });
        keyed.into_iter().map(|(_, f)| &**f).collect()
    }
}

/// Drives an adapter over the table: one fragment per spelled crossing and
/// globally identified row, one plan per site.
pub struct Compiler<'a, C: Compile> {
    model: &'a Flat,
    recipes: &'a Recipes,
    bindings: &'a Bindings,
    compiled: Compiled<C::Fragment>,
    emit: Emit,
}

impl<'a, C: Compile> Compiler<'a, C> {
    /// Drive `adapter` over this table.
    pub fn new(model: &'a Flat, recipes: &'a Recipes, bindings: &'a Bindings) -> Self {
        Self::resume(model, recipes, bindings, Compiled::default())
    }

    /// [`Self::new`], carrying on from what an earlier run built.
    pub fn resume(
        model: &'a Flat,
        recipes: &'a Recipes,
        bindings: &'a Bindings,
        compiled: Compiled<C::Fragment>,
    ) -> Self {
        Self {
            model,
            recipes,
            bindings,
            compiled,
            emit: Emit::new(),
        }
    }

    /// Hand back what this run built, to carry into the next one.
    pub fn finish(self) -> Compiled<C::Fragment> {
        self.compiled
    }

    /// How many fragments have been built, making memoization observable.
    pub fn compiled_fragments(&self) -> usize {
        self.compiled.fragments.len()
    }

    /// Compile one site: pick its recipe, build the fragment, wrap it in a plan.
    ///
    /// `None` when a declaration bound the site to
    /// [`Ask::Omit`](super::Ask::Omit).
    pub fn site(
        &mut self,
        adapter: &mut C,
        site: Site,
        crossing: Crossing,
    ) -> Result<Option<C::Plan>, CompileError<C::Error>> {
        let Some(bound) = self.bindings.resolve(&site, &crossing, self.recipes) else {
            return Ok(None);
        };
        let root = self.recipe(adapter, &bound.crossing, &bound.recipe)?;
        let needed = adapter.tolerates(&bound.site.role);
        let got = root.yields().validity;
        if !got.satisfies(needed) {
            return Err(RecipeError::Validity {
                site: bound.site,
                needed,
                got,
            }
            .into());
        }
        let mut cx = Cx {
            model: self.model,
            recipes: self.recipes,
            emit: &self.emit,
        };
        adapter
            .plan(&mut cx, &bound, &root)
            .map(Some)
            .map_err(CompileError::Adapter)
    }

    /// The fragment for one crossing's default recipe.
    ///
    /// The per-recipe half of [`Self::site`], for an adapter that composes the
    /// per-site wrapping itself. Built once and reused, like every other recipe.
    pub fn crossing(
        &mut self,
        adapter: &mut C,
        crossing: &Crossing,
    ) -> Result<Rc<C::Fragment>, CompileError<C::Error>> {
        let recipe = self.recipes.recipe(crossing).0;
        let fragment = self.recipe(adapter, crossing, &recipe)?;
        // The whole-crossing answer, so the emitters can ask for it back
        // without re-deriving which recipe a crossing defaults to.
        self.compiled
            .defaults
            .insert((crossing.spelled().key(), crossing.direction()), recipe);
        Ok(fragment)
    }

    /// The recipes this compilation was built against.
    ///
    /// So a caller can ask whether a crossing has a recipe before compiling it by
    /// name — the check [`Self::recipe_of`] enforces, available ahead of the
    /// error rather than only through it.
    pub fn recipes(&self) -> &Recipes {
        self.recipes
    }

    /// The fragment for one crossing under a **named** recipe, rather than the one
    /// the crossing defaults to.
    ///
    /// For an adapter that states more than one recipe for a type and wants both:
    /// the default answers every site, and this compiles the other so it can be
    /// checked, or read through
    /// [`Compiled::recipe_fragment`](Compiled::recipe_fragment), before any site
    /// takes it.
    ///
    /// Refuses a name the crossing does not have. Compiling a recipe falls back to
    /// the derived recipe when the lookup misses, which is right for the two
    /// callers that pass a name they got **from** the table — a crossing's
    /// default, or a site's binding — and wrong here, where the name is the
    /// caller's own claim. Without the check a conditionally-declared recipe that was not
    /// declared compiles the default instead and files it under the asked-for
    /// name, leaving [`Compiled::recipe_fragment`] to answer for a recipe that does
    /// not exist.
    pub fn recipe_of(
        &mut self,
        adapter: &mut C,
        crossing: &Crossing,
        name: &RecipeName,
    ) -> Result<Rc<C::Fragment>, CompileError<C::Error>> {
        let crossing_key = crossing.key();
        let Some(recipe) = self.recipes.key_of(&crossing_key, name).cloned() else {
            return Err(RecipeError::NoSuchRecipe {
                recipe: crossing_key.row(name.clone()),
            }
            .into());
        };
        self.recipe(adapter, crossing, &recipe)
    }

    /// The fragment for one recipe, built once and reused.
    fn recipe(&mut self, adapter: &mut C, crossing: &Crossing, recipe: &RecipeKey) -> Built<C> {
        debug_assert_eq!(recipe.crossing(), &crossing.key());
        // The row has global identity, while the fragment also needs the spelling:
        // one row can serve `T`, `&T` and `Box<T>`, whose Rust differs.
        let key = FragmentId::new(crossing, recipe.clone())
            .expect("compiler recipe always answers its crossing");
        if let Some(built) = self.compiled.fragments.get(&key) {
            return Ok(built.clone());
        }
        let chosen = match self.recipes.get(recipe) {
            Some(chosen) => chosen.clone(),
            None => {
                debug_assert_eq!(recipe.name(), &RecipeName::derived());
                super::derive(crossing)
            }
        };
        let at = At { crossing, recipe };
        let fragment = match &chosen {
            Recipe::Constructing(shape) => self.constructing(adapter, at, shape)?,
            Recipe::Deconstructing(shape) => self.deconstructing(adapter, at, shape)?,
        };
        let fragment = Rc::new(fragment);
        self.compiled.fragments.insert(key, fragment.clone());
        Ok(fragment)
    }

    /// The fragment for one part of the recipe being compiled.
    ///
    /// Which row the part takes is the site machinery's answer, asked at
    /// [`Role::Part`] — keyed by the recipe, because that is what compilation is
    /// per. A root role such as [`Role::CallbackArg`] names a place in one
    /// exported function, so it cannot answer for a recipe every function with
    /// that signature shares; an adapter that wants a per-function answer
    /// compiles that position as its own root site through [`Self::site`].
    /// `direction` is the recipe's own except for a callback, whose arguments do
    /// the other direction — Rust holds those values and pushes them out through
    /// call. The **site** is still the recipe's either way: a part is identified
    /// by the recipe that names it and its index, and a binding written against
    /// that recipe has to match here, so only the part's `Crossing` carries the
    /// swap.
    ///
    /// `wanted` is what the edge needs, and every edge states it: a product's
    /// declared parameter or field mode, an optional's value, the mode a
    /// collection lends its elements in, a callback argument's own.
    fn part(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
        direction: Direction,
        index: usize,
        ty: &TypeRef,
        wanted: Mode,
    ) -> Built<C> {
        self.part_of(adapter, at, direction, None, index, ty, wanted)
    }

    /// [`Self::part`] for a part inside a [`Shape::Choice`] arm, which numbers
    /// its parts from zero like every other arm.
    #[allow(clippy::too_many_arguments)]
    fn part_of(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
        direction: Direction,
        arm: Option<usize>,
        index: usize,
        ty: &TypeRef,
        wanted: Mode,
    ) -> Built<C> {
        let crossing = Crossing::new(ty.clone(), direction);
        let site = Site::arm_part(at.recipe, arm, index);
        let Some(bound) = self.bindings.resolve(&site, &crossing, self.recipes) else {
            return Err(RecipeError::UnknownRecipe {
                site,
                recipe: crossing.row(super::RecipeName::new("<omitted>")),
            }
            .into());
        };
        let fragment = self.recipe(adapter, &bound.crossing, &bound.recipe)?;
        // Both contracts, here rather than at each caller: an optional's value,
        // a run's element and a callback's argument are parts as much as a
        // product's are, and a check written per edge is a check an edge can be
        // added without.
        let produced = fragment.yields();
        let expected = part_key(ty);
        if produced.ty != expected {
            return Err(RecipeError::ComposedType {
                site,
                part: index,
                wanted: expected,
                got: produced.ty,
            }
            .into());
        }
        if !produced.mode.satisfies(wanted) {
            return Err(RecipeError::Composition {
                site,
                part: index,
                wanted,
                got: produced.mode,
            }
            .into());
        }
        Ok(fragment)
    }

    fn constructing(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
        shape: &Shape<Construct>,
    ) -> Result<C::Fragment, CompileError<C::Error>> {
        match shape {
            Shape::Atomic => self.atomic(adapter, at),
            Shape::Optional => self.optional(adapter, at),
            Shape::Sequence => self.sequence(adapter, at),
            Shape::Invoke => self.invoke(adapter, at),
            Shape::Product(op) => {
                let (kind, parts) = self.construct_parts(at, op, None)?;
                self.product(adapter, at, None, kind, parts)
            }
            Shape::Choice { arms } => {
                let mut built = Vec::new();
                for arm in arms {
                    let alternative = self.alternative(at, arm.alternative)?;
                    let (kind, parts) =
                        self.construct_parts(at, &arm.op, Some(&alternative.fields))?;
                    let at_arm = Some(arm.alternative);
                    built.push((alternative, self.product(adapter, at, at_arm, kind, parts)?));
                }
                self.choice(adapter, at, built)
            }
        }
    }

    fn deconstructing(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
        shape: &Shape<Deconstruct>,
    ) -> Result<C::Fragment, CompileError<C::Error>> {
        match shape {
            Shape::Atomic => self.atomic(adapter, at),
            Shape::Optional => self.optional(adapter, at),
            Shape::Sequence => self.sequence(adapter, at),
            Shape::Invoke => self.invoke(adapter, at),
            Shape::Product(op) => {
                let (kind, parts) = self.deconstruct_parts(at, op, None)?;
                self.product(adapter, at, None, kind, parts)
            }
            Shape::Choice { arms } => {
                let mut built = Vec::new();
                for arm in arms {
                    let alternative = self.alternative(at, arm.alternative)?;
                    let (kind, parts) =
                        self.deconstruct_parts(at, &arm.op, Some(&alternative.fields))?;
                    let at_arm = Some(arm.alternative);
                    built.push((alternative, self.product(adapter, at, at_arm, kind, parts)?));
                }
                self.choice(adapter, at, built)
            }
        }
    }

    // ── The hooks ─────────────────────────────────────────────────────────

    fn atomic(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
    ) -> Result<C::Fragment, CompileError<C::Error>> {
        let mut cx = self.cx();
        adapter.atomic(&mut cx, at).map_err(CompileError::Adapter)
    }

    fn optional(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
    ) -> Result<C::Fragment, CompileError<C::Error>> {
        // The payload is the crossing's own — `Option<T>` gives `T` — so the
        // recipe states no inner type and there is none to disagree with.
        let Some(inner) = at.crossing.value().optional_inner().cloned() else {
            return Err(wrong_shape(at, "Optional", "an `Option`"));
        };
        let inner = &inner;
        // Both layers: an `Option<&T>` holds a borrow, and a `&Option<T>` can
        // only lend whatever it holds. Reading either alone is wrong in a
        // different direction — `&Option<T>` would demand an owned `T` that
        // reading through the shared optional cannot produce, and
        // `&mut Option<T>` would demand an owned one where it lends `&mut T`.
        let wanted = mode_of(inner).through(at.crossing.mode());
        let inner = self.part(adapter, at, at.crossing.direction(), 0, inner, wanted)?;
        let mut cx = self.cx();
        adapter
            .optional(&mut cx, at, &inner)
            .map_err(CompileError::Adapter)
    }

    fn sequence(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
    ) -> Result<C::Fragment, CompileError<C::Error>> {
        // The element is the crossing's own, for the reason `optional` gives.
        let Some(inner) = super::sequence_elem(at.crossing.value()).cloned() else {
            return Err(wrong_shape(at, "Sequence", "a `Vec`, slice or array"));
        };
        let inner = &inner;
        // A run's element is held the way the collection lends it, which is
        // what the adapter is told and so what its fragment must satisfy.
        let elements = element_mode(at.crossing, inner);
        let inner = self.part(adapter, at, at.crossing.direction(), 0, inner, elements)?;
        let mut cx = self.cx();
        adapter
            .sequence(&mut cx, at, elements, &inner)
            .map_err(CompileError::Adapter)
    }

    fn invoke(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
    ) -> Result<C::Fragment, CompileError<C::Error>> {
        let direction = at.crossing.direction();
        let Some(args) = at.crossing.value().callback_args().map(<[_]>::to_vec) else {
            return Err(wrong_shape(at, "Invoke", "a callback type"));
        };
        // The one place the two directions swap: Rust holds these values and pushes
        // them out through the call. The swap is the argument's, not the site's
        // — the parts still belong to the callback recipe that names them.
        let mut built = Vec::new();
        for (index, arg) in args.iter().enumerate() {
            let wanted = mode_of(arg);
            built.push(self.part(adapter, at, direction.swap(), index, arg, wanted)?);
        }
        let refs: Vec<&C::Fragment> = built.iter().map(|f| &**f).collect();
        let mut cx = self.cx();
        adapter
            .callback(&mut cx, at, &refs, None)
            .map_err(CompileError::Adapter)
    }

    /// One product, whichever of the four hooks composes it.
    fn product<'p>(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
        arm: Option<usize>,
        kind: ProductKind<'p>,
        parts: Vec<Part<'p>>,
    ) -> Result<C::Fragment, CompileError<C::Error>> {
        let mut parts = parts;
        if at.crossing.direction() == Direction::Deconstruct {
            for part in &mut parts {
                part.mode = part.mode.through(at.crossing.mode());
            }
        }
        let mut built = Vec::new();
        for (index, part) in parts.iter().enumerate() {
            // `part.mode` rather than the type's own spelling: a product edge
            // states what it needs — a constructor parameter, a field, an
            // accessor's receiver — and `part` checks against that.
            built.push(self.part_of(
                adapter,
                at,
                at.crossing.direction(),
                arm,
                index,
                &part.ty,
                part.mode,
            )?);
        }
        let paired: Vec<(Part<'p>, &C::Fragment)> =
            parts.into_iter().zip(built.iter().map(|f| &**f)).collect();
        let mut cx = Cx {
            model: self.model,
            recipes: self.recipes,
            emit: &self.emit,
        };
        match kind {
            ProductKind::Construct(func) => adapter.construct(&mut cx, at, func, &paired),
            ProductKind::Fields => adapter.fields(&mut cx, at, &paired),
            ProductKind::ValueForm(func) => adapter.value_form(&mut cx, at, func, &paired),
        }
        .map_err(CompileError::Adapter)
    }

    fn choice(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
        arms: Vec<(&'a Alternative, C::Fragment)>,
    ) -> Result<C::Fragment, CompileError<C::Error>> {
        let paired: Vec<(&Alternative, &C::Fragment)> = arms.iter().map(|(a, f)| (*a, f)).collect();
        let mut cx = Cx {
            model: self.model,
            recipes: self.recipes,
            emit: &self.emit,
        };
        adapter
            .choice(&mut cx, at, &paired)
            .map_err(CompileError::Adapter)
    }

    // ── Reading the parts off the model ───────────────────────────────────

    fn cx(&mut self) -> Cx<'_> {
        Cx {
            model: self.model,
            recipes: self.recipes,
            emit: &self.emit,
        }
    }

    fn construct_parts(
        &self,
        at: At<'_>,
        op: &Construct,
        arm: Option<&'a [Field]>,
    ) -> Result<(ProductKind<'a>, Vec<Part<'a>>), CompileError<C::Error>> {
        match op {
            Construct::Fields => {
                let fields = match arm {
                    Some(fields) => fields,
                    None => self.fields_of(at.crossing.value()),
                };
                let parts = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| Part {
                        from: PartSource::Field { index, field },
                        mode: mode_of(&field.ty),
                        ty: field.ty.clone(),
                        name: field_name(field, index),
                    })
                    .collect();
                Ok((ProductKind::Fields, parts))
            }
            Construct::Call(name) => {
                let func = self.function(at, name)?;
                let parts = func
                    .params
                    .iter()
                    .enumerate()
                    .map(|(index, param)| Part {
                        from: PartSource::Argument { index },
                        mode: mode_of(&param.ty),
                        ty: param.ty.clone(),
                        name: param.name.to_string(),
                    })
                    .collect();
                Ok((ProductKind::Construct(func), parts))
            }
        }
    }

    fn deconstruct_parts(
        &self,
        at: At<'_>,
        op: &Deconstruct,
        arm: Option<&'a [Field]>,
    ) -> Result<(ProductKind<'a>, Vec<Part<'a>>), CompileError<C::Error>> {
        match op {
            Deconstruct::Fields(reaches) => {
                let fields = match arm {
                    Some(fields) => fields,
                    None => self.fields_of(at.crossing.value()),
                };
                Ok((ProductKind::Fields, self.reaches(at, reaches, fields)?))
            }
            Deconstruct::ValueForm { func, parts } => {
                let func = self.function(at, func)?;
                let fields = self.fields_of(&func.ret);
                Ok((
                    ProductKind::ValueForm(func),
                    self.reaches(at, parts, fields)?,
                ))
            }
        }
    }

    fn reaches(
        &self,
        at: At<'_>,
        reaches: &[Reach],
        fields: &'a [Field],
    ) -> Result<Vec<Part<'a>>, CompileError<C::Error>> {
        let mut parts = Vec::new();
        for reach in reaches {
            match reach {
                Reach::Omit => {}
                Reach::Field(index) => {
                    let field = fields.get(*index).ok_or_else(|| {
                        CompileError::Recipe(Box::new(RecipeError::OutOfRange {
                            recipe: at.recipe.clone(),
                            index: *index,
                            len: fields.len(),
                        }))
                    })?;
                    parts.push(Part {
                        from: PartSource::Field {
                            index: *index,
                            field,
                        },
                        mode: mode_of(&field.ty),
                        ty: field.ty.clone(),
                        name: field_name(field, *index),
                    });
                }
                Reach::Accessor(name) => {
                    let func = self.function(at, name)?;
                    parts.push(Part {
                        from: PartSource::Accessor { func },
                        mode: mode_of(&func.ret),
                        ty: func.ret.clone(),
                        name: func.name.to_string(),
                    });
                }
            }
        }
        Ok(parts)
    }

    fn function(
        &self,
        at: At<'_>,
        name: &syn::Ident,
    ) -> Result<&'a Function, CompileError<C::Error>> {
        self.model.function(name).ok_or_else(|| {
            CompileError::Recipe(Box::new(RecipeError::UnknownFunction {
                recipe: at.recipe.clone(),
                func: name.clone(),
            }))
        })
    }

    fn alternative(
        &self,
        at: At<'_>,
        index: usize,
    ) -> Result<&'a Alternative, CompileError<C::Error>> {
        let alternatives = match self.declared(at.crossing.value()) {
            Some(Type::Variant(v)) => v.alternatives.as_slice(),
            _ => &[],
        };
        alternatives.get(index).ok_or_else(|| {
            CompileError::Recipe(Box::new(RecipeError::OutOfRange {
                recipe: at.recipe.clone(),
                index,
                len: alternatives.len(),
            }))
        })
    }

    fn fields_of(&self, ty: &TypeRef) -> &'a [Field] {
        match self.declared(ty) {
            Some(Type::Struct(s)) => s.fields.as_slice(),
            _ => &[],
        }
    }

    fn declared(&self, ty: &TypeRef) -> Option<&'a Type> {
        let value = ty.borrow_target().unwrap_or(ty).unwrapped();
        match value.kind() {
            TypeKind::Named { id, .. } => self.model.resolve(id),
            _ => None,
        }
    }
}

/// A part's type as a fragment must answer for it.
///
/// The same normalization [`Crossing::key`] uses, so the two cannot disagree
/// about what a fragment for a given part is called: a part spelled `&T` or
/// `Box<T>` is answered by a fragment yielding `T`, and whether that fragment
/// may be *held* the way the part needs is [`Mode`]'s question.
fn part_key(ty: &TypeRef) -> TypeKey {
    ty.borrow_target().unwrap_or(ty).stripped_key()
}

/// A field's name, or its position when the struct is positional.
fn field_name(field: &Field, index: usize) -> String {
    field
        .name
        .as_ref()
        .map(|n| n.to_string())
        .unwrap_or_else(|| index.to_string())
}

/// Which of the four product hooks composes a set of parts.
enum ProductKind<'a> {
    Construct(&'a Function),
    Fields,
    ValueForm(&'a Function),
}

/// Whether a value is handed over or reached through a borrow.
fn mode_of(ty: &TypeRef) -> Mode {
    match ty.unwrapped().kind() {
        TypeKind::Ref { mutable: true, .. } => Mode::Exclusive,
        TypeKind::Ref { .. } => Mode::Shared,
        _ => Mode::Owned,
    }
}

/// How an element of a run is held once you have it.
///
/// Two facts, and both belong to the type rather than to the recipe.
///
/// How the collection hands an element over: `Vec<T>` gives its elements up,
/// `&[T]` and `Cow<'_, [T]>` lend them, and `&mut [T]` lends them exclusively —
/// a run reached through a borrow lends the same way it was reached, so
/// `&mut Vec<T>` yields `&mut T` and not `&T`.
///
/// And whether the element **is** a borrow: a `Vec<&T>` gives its elements up,
/// and what it gives up is a reference. So an element spelled `&T` is held as a
/// borrow however the collection hands it over, and reading only the collection
/// would call that element owned.
fn element_mode(crossing: &Crossing, elem: &TypeRef) -> Mode {
    // How the collection hands an element over, before the element's own
    // spelling is considered: a run reached through a borrow lends the same way
    // it was reached, and a bare `[T]` is only ever reached through one.
    let lent_as = match crossing.mode() {
        borrowed @ (Mode::Shared | Mode::Exclusive) => borrowed,
        Mode::Owned => match crossing.value().unwrapped().kind() {
            TypeKind::Slice(_) => Mode::Shared,
            _ => Mode::Owned,
        },
    };
    // Then the element's own, held through that. A `Vec<&T>` gives its elements
    // up and what it gives up is a borrow; a `&[&mut T]` yields `&&mut T` and
    // so cannot hand over the `&mut T` its element is spelled as.
    mode_of(elem).through(lent_as)
}

/// A shape declared for a type that cannot take it.
///
/// The arity shapes and `Invoke` read what they need off the crossing rather
/// than stating it, so a recipe declaring one on the wrong type is caught here.
fn wrong_shape<E>(at: At<'_>, shape: &'static str, wanted: &'static str) -> CompileError<E> {
    RecipeError::WrongShape {
        recipe: at.recipe.clone(),
        shape,
        wanted,
    }
    .into()
}
