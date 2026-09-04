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
    Bindings, Bound, Construct, Crossing, Deconstruct, Direction, Mode, Origin, Reach, Recipe,
    RecipeError, RecipeKey, RecipeName, Recipes, Role, Shape, Site,
};
use crate::{
    flat::{Alternative, Field, Function, Type, TypeKey, TypeKind, TypeRef},
    generation::{ChoiceArity, FixedArity, FragmentUse, OperationId, ShapePlan},
    Conversions, FragmentId,
};

/// What a fragment produces, which is the only thing the registry reads out of
/// one.
pub trait Carrier {
    /// The Rust value this fragment yields.
    fn yields(&self) -> Yield;

    /// The crossings this fragment is built out of, which the completeness check
    /// follows to decide what the binding must be able to make. Identities, not
    /// spellings, so `T`, `&T` and `Box<T>` name one crossing.
    ///
    /// `parts` is what the registry handed the hook, which is the right answer
    /// for most fragments. An adapter narrows where a part is no crossing of its
    /// own — an opaque union payload, a boxed inner — and the `atomic` hook has
    /// no parts, so a fragment that delegates from there must answer.
    fn delegates_to(&self, parts: &[TypeKey]) -> Vec<TypeKey> {
        parts.to_vec()
    }

    /// Take the shape the registry composed for this fragment.
    ///
    /// The registry knows which shape it is compiling and which child fragments
    /// it handed to the hook, so it builds the [`ShapePlan`] rather than each
    /// hook stating one. The fragment carries it as far as the `FragmentPlan`
    /// the adapter freezes.
    fn composed(&mut self, shape: ShapePlan);
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

impl At<'_> {
    /// Stable registry identity of the fragment being compiled.
    pub fn fragment_id(&self) -> FragmentId {
        FragmentId::new(self.crossing.spelled().key(), self.recipe.clone())
    }

    /// Sequence-converter identity for a model carrier deliberately shared by
    /// owned collections and borrowed views.
    pub fn sequence_converter_for(&self, carrier: &TypeRef) -> crate::OperationId {
        crate::OperationId::sequence_converter(carrier, self.crossing.direction())
    }
}

/// What every hook receives: a read-only view of the model, of the binding, and
/// of what has compiled so far. A hook asks the registry nothing and records
/// nothing in it; what it produces is its fragment.
pub struct Cx<'a, F> {
    conversions: &'a dyn Conversions,
    compiled: &'a Compiled<F>,
}

impl<F> Cx<'_, F> {
    /// What the binding says about a type, and the model every structural fact
    /// is read off: the same view an emitter reads after the fill, lent per call
    /// so an adapter need not hold one itself.
    pub fn conversions(&self) -> &dyn Conversions {
        self.conversions
    }

    /// The fragments compiled so far, this recipe's parts among them. A shared
    /// borrow is enough: the parts are compiled before the hook runs, and
    /// nothing writes to the store while one is running.
    pub fn compiled(&self) -> &Compiled<F> {
        self.compiled
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
/// `Clone` and not `Copy`: `Path` carries its access chain, and a derived
/// recipe is owned and dropped inside the compile, so the chain cannot be
/// borrowed from it (#613 step 10).
#[derive(Clone, Debug)]
pub enum PartSource<'a> {
    /// Parameter `index` of the constructor being called.
    Argument {
        /// Position in the constructor's parameter list.
        index: usize,
    },
    /// A chain of field accesses, outermost first — an inlined nested class's
    /// leaf. The one-hop case is [`Self::Field`] (#613 step 10).
    Path {
        /// Each hop's position in the struct it indexes. Borrowed from the
        /// recipe row, which outlives the compile.
        indices: Vec<usize>,
        /// The field the chain arrives at.
        field: &'a Field,
    },
    /// Field `index` of the value, read directly.
    Field {
        /// Position in the struct or in the arm's payload.
        index: usize,
        /// The field itself.
        field: &'a Field,
    },
    /// The value itself, as one part — the compiled form of
    /// [`Reach::Identity`]. Nothing is indexed and nothing is called.
    Identity,
    /// This accessor call.
    Accessor {
        /// The function to call.
        func: &'a Function,
    },
}

/// A product's parts, each already compiled.
pub type Parts<'a, C> = &'a [(Part<'a>, &'a <C as Compile>::Fragment)];

/// The context one hook receives, named by the adapter. [`Cx`] is generic over
/// the **fragment** so an adapter that delegates to another can hand its own
/// context straight through.
pub type Ctx<'a, C> = Cx<'a, <C as Compile>::Fragment>;

/// Why a hook has no fragment: an ordinary **gap** — nothing for this crossing,
/// which the scan's over-approximation makes common and which matters only where
/// an exported function reaches it — or an **error**, a wrong declaration,
/// reported either way. Both were one answer before, which both adapters turned
/// into a gap with `.ok()?`.
#[derive(Debug)]
pub enum Refusal<E> {
    /// Nothing for this crossing, and why. A diagnostic: nothing fails on it.
    Gap(String),
    /// The declaration or the composition is wrong. Always reported.
    Error(E),
}

/// So a hook can `?` its own error type: a helper that failed has said
/// something is wrong, not that the crossing is unanswered.
impl<E> From<E> for Refusal<E> {
    fn from(error: E) -> Self {
        Refusal::Error(error)
    }
}

/// Shorthand for what every fragment-producing hook returns.
pub type Frag<C> = Result<<C as Compile>::Fragment, Refusal<<C as Compile>::Error>>;

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
    fn atomic(&mut self, cx: &mut Ctx<'_, Self>, at: At<'_>) -> Frag<Self>;

    /// Absent, or the inner fragment's value.
    fn optional(
        &mut self,
        cx: &mut Ctx<'_, Self>,
        at: At<'_>,
        inner: &Self::Fragment,
    ) -> Frag<Self>;

    /// A run of the inner fragment's value.
    ///
    /// `elements` is derived from the collection type, not declared, so an
    /// adapter reads it rather than checking it against anything.
    fn sequence(
        &mut self,
        cx: &mut Ctx<'_, Self>,
        at: At<'_>,
        elements: Mode,
        inner: &Self::Fragment,
    ) -> Frag<Self>;

    /// Call `func` to assemble the value. Its parameters are `args`.
    fn construct(
        &mut self,
        cx: &mut Ctx<'_, Self>,
        at: At<'_>,
        func: &Function,
        args: Parts<'_, Self>,
    ) -> Frag<Self>;

    /// Read the parts off the value where it stands.
    fn fields(&mut self, cx: &mut Ctx<'_, Self>, at: At<'_>, parts: Parts<'_, Self>) -> Frag<Self>;

    /// Call `func` once, then read the parts off what it returned.
    fn value_form(
        &mut self,
        cx: &mut Ctx<'_, Self>,
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
        cx: &mut Ctx<'_, Self>,
        at: At<'_>,
        arms: &[(Option<&Alternative>, &Self::Fragment)],
    ) -> Frag<Self>;

    /// A callback, taken apart into the values that pass through it.
    ///
    /// `result` is `None` for every callback the model can describe today; it
    /// is the position issue #216's return value lands in.
    fn callback(
        &mut self,
        cx: &mut Ctx<'_, Self>,
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
            Role::Param { .. }
            | Role::Receiver
            | Role::CallbackArg { .. }
            | Role::Part { .. }
            // A leaf of an expanded parameter lives exactly as long as the
            // parameter it came from, so it borrows on the same terms.
            | Role::ExpansionLeaf { .. } => Validity::Borrowed,
        }
    }

    /// Whether this adapter plans this site at all.
    ///
    /// The registry enumerates the positions a value *can* cross at, which is
    /// the model's answer. Whether a given target has anything to do with one is
    /// not: `prebindgen-c` plans nothing for a `()` return because C hands
    /// nothing back there, while a JVM binding does; and JniGen answers a
    /// callback parameter whole, so planning it as a site would freeze the same
    /// position twice.
    ///
    /// The default is that every enumerated site is planned. An adapter that
    /// says no gets no plan and no refusal — a position it has nothing to say
    /// about is not a failure.
    fn plans_site(&self, _site: &Site, _crossing: &Crossing) -> bool {
        true
    }

    /// Which recipe this site takes, where the binding table cannot say.
    ///
    /// `None` — the default, and the answer for every site of most adapters —
    /// means the binding decides. An adapter answers otherwise where the choice
    /// follows from something compiled rather than from the model: JniGen picks
    /// its `pair` row for an optional whose payload turned out to be a
    /// niche-free primitive, which is a fact about the compiled payload and not
    /// about the declaration.
    ///
    /// Asked before the root fragment is built, so the answer chooses which
    /// fragment that is. The row must still be declared, so choosing one cannot
    /// invent a representation outside the table.
    fn site_recipe(&mut self, _cx: &mut Ctx<'_, Self>, _bound: &Bound) -> Option<RecipeName> {
        None
    }

    /// Wrap the site's root fragment into a plan — the signature, the call, the
    /// cleanup.
    ///
    /// The only hook called once per site; every hook above is called once per
    /// recipe, however many sites reuse it.
    fn plan(
        &mut self,
        cx: &mut Ctx<'_, Self>,
        bound: &Bound,
        root: &Self::Fragment,
    ) -> Result<Self::Plan, Self::Error>;
}

/// What a compilation reports when it cannot finish.
#[derive(Debug)]
pub enum CompileError<E> {
    /// The adapter refused a recipe or a site, possibly through a part rather
    /// than through what was asked for. A [`Refusal::Gap`] is a failure only
    /// where something has already reached the crossing — at a **site**, say.
    Adapter(Refusal<E>),
    /// The table and the model disagree with what a site asked for.
    ///
    /// Boxed: a `RecipeError` names a site and a crossing, which makes it much
    /// the larger of the two variants, and this is the rare one.
    Recipe(Box<RecipeError>),
}

impl<E: fmt::Display> fmt::Display for CompileError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Adapter(Refusal::Gap(why)) => f.write_str(why),
            CompileError::Adapter(Refusal::Error(e)) => write!(f, "{e}"),
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

/// One compiled part: its fragment, and the edge to it the shape records.
type Edge<C> =
    Result<(Rc<<C as Compile>::Fragment>, FragmentUse), CompileError<<C as Compile>::Error>>;

/// A fragment and the shape the registry composed for it.
type Composed<C> =
    Result<(<C as Compile>::Fragment, ShapePlan), CompileError<<C as Compile>::Error>>;

/// A product's fragment and the edges to its parts, which the shape of the
/// product — or of the choice whose arm it is — records.
type Assembled<C> =
    Result<(<C as Compile>::Fragment, Vec<FragmentUse>), CompileError<<C as Compile>::Error>>;

/// Every composite shape's bridge is the fragment's own converter identity: the
/// shape says how the parts go together, and the identity says which generated
/// function does it.
fn bridge(at: At<'_>) -> OperationId {
    OperationId::converter(at.fragment_id())
}

fn product_shape(at: At<'_>, parts: Vec<FragmentUse>) -> ShapePlan {
    ShapePlan::Product {
        bridge: FixedArity::new(parts.len(), bridge(at)),
        parts,
    }
}

/// [`Built`], beside the crossings the fragment delegates to.
type Answered<C> =
    Result<(Rc<<C as Compile>::Fragment>, Vec<TypeKey>), CompileError<<C as Compile>::Error>>;

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
    /// the next write, and a fragment may hold a nontrivial semantic plan that
    /// a recursive lookup should not copy.
    pub fn fragment(&self, ty: &TypeKey, direction: Direction) -> Option<Rc<F>> {
        let recipe = self.defaults.get(&(ty.clone(), direction))?;
        self.recipe_fragment(ty, recipe)
    }

    /// The fragment for one spelled type and one row key.
    pub fn recipe_fragment(&self, ty: &TypeKey, recipe: &RecipeKey) -> Option<Rc<F>> {
        self.fragments
            .get(&FragmentId::new(ty.clone(), recipe.clone()))
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
    conversions: &'a dyn Conversions,
    recipes: &'a Recipes,
    bindings: &'a Bindings,
    compiled: Compiled<C::Fragment>,
    /// Per fragment, the crossings it is built out of: the parts the registry
    /// compiled for it while the row was being built, replaced by the
    /// fragment's own [`Carrier::delegates_to`] once the hook has answered.
    delegations: HashMap<FragmentId, Vec<TypeKey>>,
}

impl<'a, C: Compile> Compiler<'a, C> {
    /// Drive `adapter` over this table.
    pub fn new(
        conversions: &'a dyn Conversions,
        recipes: &'a Recipes,
        bindings: &'a Bindings,
    ) -> Self {
        Self::resume(conversions, recipes, bindings, Compiled::default())
    }

    /// [`Self::new`], carrying on from what an earlier run built.
    pub fn resume(
        conversions: &'a dyn Conversions,
        recipes: &'a Recipes,
        bindings: &'a Bindings,
        compiled: Compiled<C::Fragment>,
    ) -> Self {
        Self {
            conversions,
            recipes,
            bindings,
            compiled,
            delegations: HashMap::new(),
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
        // The adapter's own answer takes precedence, and only where it has one:
        // a choice that follows from a compiled fragment is one the binding
        // table cannot express.
        let chosen = {
            let mut cx = self.cx();
            adapter.site_recipe(&mut cx, &bound)
        };
        let bound = match chosen {
            None => bound,
            Some(name) => {
                let crossing_key = bound.crossing.key();
                let Some(recipe) = self.recipes.key_of(&crossing_key, &name).cloned() else {
                    return Err(RecipeError::NoSuchRecipe {
                        recipe: crossing_key.row(name),
                    }
                    .into());
                };
                Bound {
                    recipe,
                    origin: Origin::Adapter,
                    ..bound
                }
            }
        };
        self.plan(adapter, bound)
    }

    fn plan(
        &mut self,
        adapter: &mut C,
        bound: Bound,
    ) -> Result<Option<C::Plan>, CompileError<C::Error>> {
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
        let mut cx = self.cx();
        adapter
            .plan(&mut cx, &bound, &root)
            .map(Some)
            .map_err(|e| CompileError::Adapter(e.into()))
    }

    /// The fragment for one crossing's default recipe.
    ///
    /// The per-recipe half of [`Self::site`], for an adapter that composes the
    /// per-site wrapping itself. Built once and reused, like every other recipe.
    /// Beside the fragment, the crossings it delegates to: what the registry's
    /// reachability walk follows, and the reason [`Carrier::delegates_to`]
    /// exists.
    pub fn crossing(&mut self, adapter: &mut C, crossing: &Crossing) -> Answered<C> {
        let recipe = self.recipes.recipe(crossing).0;
        let fragment = self.recipe(adapter, crossing, &recipe)?;
        let id = FragmentId::new(crossing.spelled().key(), recipe.clone());
        // The whole-crossing answer, so the emitters can ask for it back
        // without re-deriving which recipe a crossing defaults to.
        self.compiled
            .defaults
            .insert((crossing.spelled().key(), crossing.direction()), recipe);
        let delegates = self.delegations.get(&id).cloned().unwrap_or_default();
        Ok((fragment, delegates))
    }

    /// The fragment for one crossing under a row of the table, rather than the
    /// one the crossing defaults to.
    ///
    /// For a type whose table states more than one recipe: the default answers
    /// every site, and this builds the other so it can be read through
    /// [`Compiled::recipe_fragment`] before any site takes it. A key comes only
    /// from [`Recipes::key_of`](super::Recipes::key_of), so this cannot reach
    /// `recipe`'s fallback to the derived recipe — right for a name read *from*
    /// the table, and wrong for a caller's own claim.
    pub fn row(
        &mut self,
        adapter: &mut C,
        crossing: &Crossing,
        row: &RecipeKey,
    ) -> Result<Rc<C::Fragment>, CompileError<C::Error>> {
        self.recipe(adapter, crossing, row)
    }

    /// The fragment for one recipe, built once and reused.
    fn recipe(&mut self, adapter: &mut C, crossing: &Crossing, recipe: &RecipeKey) -> Built<C> {
        debug_assert_eq!(recipe.crossing(), &crossing.key());
        // The row has global identity, while the fragment also needs the spelling:
        // one row can serve `T`, `&T` and `Box<T>`, whose Rust differs.
        let key = FragmentId::new(crossing.spelled().key(), recipe.clone());
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
        let (mut fragment, shape) = match &chosen {
            Recipe::Constructing(shape) => self.constructing(adapter, at, shape)?,
            Recipe::Deconstructing(shape) => self.deconstructing(adapter, at, shape)?,
        };
        fragment.composed(shape);
        // What the hook was handed, now what the fragment says it delegates to.
        let parts = self.delegations.remove(&key).unwrap_or_default();
        self.delegations
            .insert(key.clone(), fragment.delegates_to(&parts));
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
    ) -> Edge<C> {
        self.part_of(adapter, at, direction, None, index, ty, wanted, false)
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
        identity: bool,
    ) -> Edge<C> {
        let crossing = Crossing::new(ty.clone(), direction);
        // Every edge the registry compiles for this row, which is the default
        // answer to `Carrier::delegates_to`. An arm's parts land on the choice
        // that owns them, because the arm has no fragment identity of its own.
        self.delegations
            .entry(at.fragment_id())
            .or_default()
            .push(ty.key());
        let site = Site::arm_part(at.recipe, arm, index);
        // An identity part IS its receiver, so resolving it the ordinary way
        // finds the row being compiled and recurses without bound. It takes the
        // crossing's DEFAULT row instead — the value's own converter, which is
        // what a handle leaf delivers. A default that is the row being compiled
        // is a cycle the declaration actually wrote, and says so (#613 step 10).
        if identity {
            let (row, _) = self.recipes.recipe(&crossing);
            if row == *at.recipe {
                return Err(RecipeError::Cycle {
                    path: vec![at.crossing.key(), crossing.key()],
                }
                .into());
            }
            let fragment = self.recipe(adapter, &crossing, &row)?;
            let edge = FragmentUse::new(
                FragmentId::new(crossing.spelled().key(), row),
                fragment.yields(),
            );
            return Ok((fragment, edge));
        }
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
        let edge = FragmentUse::new(
            FragmentId::new(bound.crossing.spelled().key(), bound.recipe),
            produced,
        );
        Ok((fragment, edge))
    }

    fn constructing(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
        shape: &Shape<Construct>,
    ) -> Composed<C> {
        match shape {
            Shape::Atomic => self.atomic(adapter, at),
            Shape::Optional => self.optional(adapter, at),
            Shape::Sequence => self.sequence(adapter, at),
            Shape::Invoke => self.invoke(adapter, at),
            Shape::Product(op) => {
                let (kind, parts) = self.construct_parts(at, op, None)?;
                let (fragment, uses) = self.product(adapter, at, None, kind, parts)?;
                Ok((fragment, product_shape(at, uses)))
            }
            Shape::Choice { arms } => {
                let mut built = Vec::new();
                let mut arm_uses = Vec::new();
                // Numbered by POSITION, not by the alternative an arm may name.
                // An arm that names none still has to key its parts' bindings,
                // and for a sum every arm names its own position anyway.
                for (at_arm, arm) in arms.iter().enumerate() {
                    let alternative = self.alternative_of(at, arm.alternative)?;
                    let (kind, parts) =
                        self.construct_parts(at, &arm.op, alternative.map(|a| &*a.fields))?;
                    let (fragment, uses) = self.product(adapter, at, Some(at_arm), kind, parts)?;
                    built.push((alternative, fragment));
                    arm_uses.push(uses);
                }
                self.choice(adapter, at, built, arm_uses)
            }
        }
    }

    fn deconstructing(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
        shape: &Shape<Deconstruct>,
    ) -> Composed<C> {
        match shape {
            Shape::Atomic => self.atomic(adapter, at),
            Shape::Optional => self.optional(adapter, at),
            Shape::Sequence => self.sequence(adapter, at),
            Shape::Invoke => self.invoke(adapter, at),
            Shape::Product(op) => {
                let (kind, parts) = self.deconstruct_parts(at, op, None)?;
                let (fragment, uses) = self.product(adapter, at, None, kind, parts)?;
                Ok((fragment, product_shape(at, uses)))
            }
            Shape::Choice { arms } => {
                let mut built = Vec::new();
                let mut arm_uses = Vec::new();
                for (at_arm, arm) in arms.iter().enumerate() {
                    let alternative = self.alternative_of(at, arm.alternative)?;
                    let (kind, parts) =
                        self.deconstruct_parts(at, &arm.op, alternative.map(|a| &*a.fields))?;
                    let (fragment, uses) = self.product(adapter, at, Some(at_arm), kind, parts)?;
                    built.push((alternative, fragment));
                    arm_uses.push(uses);
                }
                self.choice(adapter, at, built, arm_uses)
            }
        }
    }

    // ── The hooks ─────────────────────────────────────────────────────────

    fn atomic(&mut self, adapter: &mut C, at: At<'_>) -> Composed<C> {
        let mut cx = self.cx();
        let fragment = adapter.atomic(&mut cx, at).map_err(CompileError::Adapter)?;
        Ok((fragment, ShapePlan::Atomic(bridge(at))))
    }

    fn optional(&mut self, adapter: &mut C, at: At<'_>) -> Composed<C> {
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
        let (inner, value) = self.part(adapter, at, at.crossing.direction(), 0, inner, wanted)?;
        let mut cx = self.cx();
        let fragment = adapter
            .optional(&mut cx, at, &inner)
            .map_err(CompileError::Adapter)?;
        Ok((
            fragment,
            ShapePlan::Optional {
                bridge: bridge(at),
                value,
            },
        ))
    }

    fn sequence(&mut self, adapter: &mut C, at: At<'_>) -> Composed<C> {
        // The element is the crossing's own, for the reason `optional` gives.
        let Some(inner) = super::sequence_elem(at.crossing.value()).cloned() else {
            return Err(wrong_shape(at, "Sequence", "a `Vec`, slice or array"));
        };
        let inner = &inner;
        // A run's element is held the way the collection lends it, which is
        // what the adapter is told and so what its fragment must satisfy.
        let elements = element_mode(at.crossing, inner);
        let (inner, element) =
            self.part(adapter, at, at.crossing.direction(), 0, inner, elements)?;
        let mut cx = self.cx();
        let fragment = adapter
            .sequence(&mut cx, at, elements, &inner)
            .map_err(CompileError::Adapter)?;
        Ok((
            fragment,
            ShapePlan::Sequence {
                bridge: bridge(at),
                element,
            },
        ))
    }

    fn invoke(&mut self, adapter: &mut C, at: At<'_>) -> Composed<C> {
        let direction = at.crossing.direction();
        let Some(args) = at.crossing.value().callback_args().map(<[_]>::to_vec) else {
            return Err(wrong_shape(at, "Invoke", "a callback type"));
        };
        // The one place the two directions swap: Rust holds these values and pushes
        // them out through the call. The swap is the argument's, not the site's
        // — the parts still belong to the callback recipe that names them.
        let mut built = Vec::new();
        let mut arguments = Vec::new();
        for (index, arg) in args.iter().enumerate() {
            let wanted = mode_of(arg);
            let (fragment, edge) = self.part(adapter, at, direction.swap(), index, arg, wanted)?;
            built.push(fragment);
            arguments.push(edge);
        }
        let refs: Vec<&C::Fragment> = built.iter().map(|f| &**f).collect();
        let mut cx = self.cx();
        let fragment = adapter
            .callback(&mut cx, at, &refs, None)
            .map_err(CompileError::Adapter)?;
        Ok((
            fragment,
            ShapePlan::Invoke {
                bridge: FixedArity::new(arguments.len(), bridge(at)),
                arguments,
            },
        ))
    }

    /// One product, whichever of the four hooks composes it.
    fn product<'p>(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
        arm: Option<usize>,
        kind: ProductKind<'p>,
        parts: Vec<Part<'p>>,
    ) -> Assembled<C> {
        let mut parts = parts;
        if at.crossing.direction() == Direction::Deconstruct {
            for part in &mut parts {
                part.mode = part.mode.through(at.crossing.mode());
            }
        }
        let mut built = Vec::new();
        let mut uses = Vec::new();
        for (index, part) in parts.iter().enumerate() {
            // `part.mode` rather than the type's own spelling: a product edge
            // states what it needs — a constructor parameter, a field, an
            // accessor's receiver — and `part` checks against that.
            let (fragment, edge) = self.part_of(
                adapter,
                at,
                at.crossing.direction(),
                arm,
                index,
                &part.ty,
                part.mode,
                matches!(part.from, PartSource::Identity),
            )?;
            built.push(fragment);
            uses.push(edge);
        }
        let paired: Vec<(Part<'p>, &C::Fragment)> =
            parts.into_iter().zip(built.iter().map(|f| &**f)).collect();
        let mut cx = self.cx();
        let fragment = match kind {
            ProductKind::Construct(func) => adapter.construct(&mut cx, at, func, &paired),
            ProductKind::Fields => adapter.fields(&mut cx, at, &paired),
            ProductKind::ValueForm(func) => adapter.value_form(&mut cx, at, func, &paired),
        }
        .map_err(CompileError::Adapter)?;
        Ok((fragment, uses))
    }

    fn choice(
        &mut self,
        adapter: &mut C,
        at: At<'_>,
        arms: Vec<(Option<&'a Alternative>, C::Fragment)>,
        arm_uses: Vec<Vec<FragmentUse>>,
    ) -> Composed<C> {
        let paired: Vec<(Option<&Alternative>, &C::Fragment)> =
            arms.iter().map(|(a, f)| (*a, f)).collect();
        let mut cx = self.cx();
        let fragment = adapter
            .choice(&mut cx, at, &paired)
            .map_err(CompileError::Adapter)?;
        Ok((
            fragment,
            ShapePlan::Choice {
                bridge: ChoiceArity::new(arm_uses.iter().map(Vec::len).collect(), bridge(at)),
                arms: arm_uses,
            },
        ))
    }

    // ── Reading the parts off the model ───────────────────────────────────

    fn cx(&self) -> Cx<'_, C::Fragment> {
        Cx {
            conversions: self.conversions,
            compiled: &self.compiled,
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
            Construct::Identity => {
                // The value itself, exactly as `Reach::Identity` states it on
                // the other side: the SPELLED crossing's mode, so a borrowed
                // crossing records `Borrowed` and the adapter clones rather
                // than moving out of a reference.
                let parts = vec![Part {
                    from: PartSource::Identity,
                    mode: at.crossing.mode(),
                    ty: at.crossing.value().clone(),
                    name: "self".to_string(),
                }];
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
                // The nested shape's own parts, compiled against the field's
                // type. `deconstruct_parts` is the same entry the outer shape
                // took, one level in — so a nested `Choice` composes exactly as
                // a top-level one does (#613 step 10).
                Reach::Nested { index, shape } => {
                    let field = fields.get(*index).ok_or_else(|| {
                        CompileError::Recipe(Box::new(RecipeError::OutOfRange {
                            recipe: at.recipe.clone(),
                            index: *index,
                            len: fields.len(),
                        }))
                    })?;
                    let inner = self.fields_of(&field.ty);
                    let (_, mut inner_parts) = self.deconstruct_parts(at, shape, Some(inner))?;
                    parts.append(&mut inner_parts);
                }
                Reach::Path(indices) => {
                    // Walk the chain against the model, as validation did.
                    let mut here: &[Field] = fields;
                    let mut field: Option<&Field> = None;
                    for index in indices {
                        let hop = here.get(*index).ok_or_else(|| {
                            CompileError::Recipe(Box::new(RecipeError::OutOfRange {
                                recipe: at.recipe.clone(),
                                index: *index,
                                len: here.len(),
                            }))
                        })?;
                        field = Some(hop);
                        here = self.fields_of(&hop.ty);
                    }
                    let field = field.ok_or_else(|| {
                        CompileError::Recipe(Box::new(RecipeError::OutOfRange {
                            recipe: at.recipe.clone(),
                            index: 0,
                            len: 0,
                        }))
                    })?;
                    parts.push(Part {
                        from: PartSource::Path {
                            indices: indices.clone(),
                            field,
                        },
                        mode: mode_of(&field.ty),
                        ty: field.ty.clone(),
                        name: field_name(field, *indices.last().unwrap_or(&0)),
                    });
                }
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
                Reach::Identity => parts.push(Part {
                    from: PartSource::Identity,
                    // From the SPELLED crossing, not `value()`: the latter
                    // strips `&`/`&mut`, which would record a borrowed
                    // identity row as `Owned` and lose the clone-for-borrow /
                    // move-for-owned distinction this form exists to carry
                    // (#635 review).
                    mode: at.crossing.mode(),
                    ty: at.crossing.value().clone(),
                    name: "self".to_string(),
                }),
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
        self.conversions.flat().function(name).ok_or_else(|| {
            CompileError::Recipe(Box::new(RecipeError::UnknownFunction {
                recipe: at.recipe.clone(),
                func: name.clone(),
            }))
        })
    }

    /// The alternative an arm names, or `None` when it names none.
    fn alternative_of(
        &self,
        at: At<'_>,
        index: Option<usize>,
    ) -> Result<Option<&'a Alternative>, CompileError<C::Error>> {
        index.map(|i| self.alternative(at, i)).transpose()
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
            TypeKind::Named { id, .. } => self.conversions.flat().resolve(id),
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
