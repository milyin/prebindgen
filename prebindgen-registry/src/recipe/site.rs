//! Which recipe-table row a given place in the generated API uses.
//!
//! [`Recipes`] says what rows a crossing key has; this says which row the
//! second parameter of `z_put`, or the `Err` arm of a return, or one part of
//! another recipe actually takes. A declaration states that through
//! [`BindingsBuilder::bind`], and [`Bindings::resolve`] answers it for one
//! site.
//!
//! Declarations select by reusable [`RecipeName`]; resolution stores the full
//! [`RecipeKey`]. A site nobody binds uses its crossing's default row, so the
//! common case is declared nowhere.

use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};

use super::{Crossing, RecipeError, RecipeKey, RecipeName, Recipes};

/// One place in the generated API where a value crosses.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Site {
    /// The item the site belongs to: the exported function for a parameter or
    /// a return, the crossed type for a part.
    pub owner: syn::Ident,
    /// Which place within that item.
    pub role: Role,
}

impl Site {
    /// The site of one part: which crossing the part belongs to, which of that
    /// crossing's recipes names it, and where in that recipe it sits.
    ///
    /// The one `Site` an adapter must be able to build **exactly**, because the
    /// driver builds it too and the two have to meet: a per-part binding is
    /// found by this key or not at all. Its `owner` is derived from the crossed
    /// type rather than chosen, so composing one by hand is a guess — this is
    /// the answer instead.
    pub fn part(recipe: &RecipeKey, index: usize) -> Self {
        Self::arm_part(recipe, None, index)
    }

    /// [`Self::part`], for a part that sits inside one alternative.
    ///
    /// A part of a [`Shape::Choice`](super::Shape::Choice) recipe needs the
    /// alternative stated, because every arm numbers its parts from zero.
    pub fn arm_part(recipe: &RecipeKey, arm: Option<usize>, index: usize) -> Self {
        Self {
            owner: recipe
                .crossing()
                .ty
                .ident()
                .unwrap_or_else(|| syn::Ident::new("_", proc_macro2::Span::call_site())),
            role: Role::Part {
                recipe: recipe.clone(),
                arm,
                index,
            },
        }
    }
}

impl fmt::Display for Site {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}'s {}", self.owner, self.role)
    }
}

/// Which place within an item a site is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    /// One parameter of an exported function.
    ///
    /// Identified by position and not by name: a `Site` is a map key, so
    /// anything on it has to be something two callers cannot spell differently.
    /// The parameter's name is the model's, reached through `owner`.
    Param {
        /// Position in the parameter list.
        index: usize,
    },
    /// The value a method is called on.
    Receiver,
    /// What the function returns.
    Return,
    /// The `Err` arm of a `Result`.
    Error,
    /// One argument the Rust side passes out through a callback.
    ///
    /// Swaps direction: Rust holds the value and pushes it out, so the argument is
    /// deconstructed even though it sits in a parameter list.
    ///
    /// A **root** role, like [`Return`](Self::Return) and
    /// [`Param`](Self::Param): it names an argument of the callback in one
    /// exported function, so it is what an adapter passes to
    /// [`Compiler::site`](super::Compiler::site) when it compiles that argument
    /// itself. It is deliberately not what the driver asks at while compiling a
    /// callback recipe — a recipe is shared by every function whose callback has the
    /// same signature, so a per-function answer could not apply to it. Inside a
    /// recipe the driver asks at [`Part`](Self::Part), which is keyed by the recipe.
    CallbackArg {
        /// Which parameter of the exported function is the callback.
        param: usize,
        /// Which argument of that callback.
        arg: usize,
    },
    /// One leaf of an **expanded** parameter.
    ///
    /// An `expand_param!` declaration replaces one source parameter with
    /// several values, each crossing on its own. They are not
    /// [`Param`](Self::Param)s: that index is the position in the source
    /// parameter list, and an expansion's leaves all belong to the one
    /// parameter that expanded — so numbering them as parameters would name
    /// positions the function does not have and attach one parameter's site to
    /// another's crossing (#622 review).
    ///
    /// Shaped like [`CallbackArg`](Self::CallbackArg) because the question is
    /// the same one: which parameter, and which value within it.
    ExpansionLeaf {
        /// Which parameter of the exported function expanded.
        param: usize,
        /// Which leaf of that expansion.
        leaf: usize,
    },
    /// One part of another crossing's recipe.
    Part {
        /// The globally unique row that names this part.
        recipe: RecipeKey,
        /// Which alternative, for a part inside a [`Shape::Choice`](super::Shape::Choice)
        /// arm; `None` for a product's own part.
        ///
        /// Load-bearing rather than decorative: **every arm numbers its parts
        /// from zero**, so `part 0` of a two-armed sum names two different
        /// parts of two different types. Without the alternative there is no
        /// key that tells them apart, and a binding written for one silently
        /// collides with the other.
        arm: Option<usize>,
        /// The part's position within the recipe — or within its arm.
        index: usize,
    },
    /// A `#[prebindgen]` constant's value.
    Const,
}

impl Role {
    /// The source parameter position this role sits at, for the roles that have
    /// one. `None` for a return, an error arm, a receiver, a constant or a part.
    pub fn param_position(&self) -> Option<usize> {
        match self {
            Role::Param { index }
            | Role::ExpansionLeaf { param: index, .. }
            | Role::CallbackArg { param: index, .. } => Some(*index),
            _ => None,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Param { index } => write!(f, "parameter {index}"),
            Role::Receiver => f.write_str("receiver"),
            Role::Return => f.write_str("return value"),
            Role::Error => f.write_str("error arm"),
            Role::CallbackArg { param, arg } => {
                write!(f, "argument {arg} of the callback in parameter {param}")
            }
            Role::ExpansionLeaf { param, leaf } => {
                write!(f, "leaf {leaf} of the expansion of parameter {param}")
            }
            Role::Part { recipe, arm, index } => match arm {
                Some(arm) => write!(f, "part {index} of arm {arm} of {recipe}"),
                None => write!(f, "part {index} of {recipe}"),
            },
            Role::Const => f.write_str("value"),
        }
    }
}

/// What a declaration asks for at one site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// The crossing's default recipe.
    Default,
    /// This named recipe under the crossing key.
    Recipe(RecipeName),
    /// This site contributes nothing.
    Omit,
}

/// Which declaration made the ask.
///
/// Ordered highest precedence first, so the winner of two asks for one site is
/// the smaller of the two.
///
/// Not to be confused with [`flat::Origin`](crate::flat::Origin), which is a
/// captured item's own syntax.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Origin {
    /// A per-function override, written against one exported function.
    Function,
    /// A per-part declaration, written against one part of one recipe.
    Part,
    /// The type's own default, written against the type.
    Type,
    /// What the adapter would otherwise have picked.
    Adapter,
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Origin::Function => "a per-function declaration",
            Origin::Part => "a per-part declaration",
            Origin::Type => "the type's own declaration",
            Origin::Adapter => "the adapter",
        })
    }
}

/// What the table hands an adapter for one site. Immutable.
#[derive(Clone, Debug)]
pub struct Bound {
    /// Where the value crosses.
    pub site: Site,
    /// What crosses, and which way.
    pub crossing: Crossing,
    /// The globally unique row this site takes.
    pub recipe: RecipeKey,
    /// Which declaration won, so a diagnostic can say why this recipe applies.
    pub origin: Origin,
}

#[derive(Clone, Debug)]
struct Binding {
    crossing: Crossing,
    ask: Ask,
    origin: Origin,
}

/// Describes the [`Bindings`] of one binding crate.
#[derive(Default)]
pub struct BindingsBuilder {
    asks: HashMap<Site, Binding>,
    /// Two declarations of equal precedence that disagree, reported by
    /// [`Self::build`] rather than resolved by declaration order. One entry per
    /// site, so a site declared three ways is reported once.
    conflicts: BTreeMap<String, (Site, Origin)>,
}

impl BindingsBuilder {
    /// State which recipe one site takes.
    ///
    /// Where two declarations bind one site, the higher-precedence `origin`
    /// wins — with its `crossing` and its `ask` together, since the two are one
    /// declaration's answer. Two declarations of equal precedence asking for
    /// different recipes is an error, and two asking for the same recipe is not.
    pub fn bind(&mut self, site: Site, crossing: Crossing, ask: Ask, origin: Origin) -> &mut Self {
        let binding = Binding {
            crossing,
            ask,
            origin,
        };
        match self.asks.get(&site) {
            // A weaker declaration never displaces a stronger one.
            Some(held) if held.origin < binding.origin => {}
            Some(held) if held.origin == binding.origin => {
                // The whole answer has to match, not only the ask: two
                // declarations naming different crossings disagree just as much
                // as two naming different recipes.
                if held.ask != binding.ask || held.crossing.key() != binding.crossing.key() {
                    self.conflicts.insert(site.to_string(), (site, origin));
                }
            }
            _ => {
                self.asks.insert(site, binding);
            }
        }
        self
    }

    /// Check every ask against the table and freeze the result.
    ///
    /// The one check here is that a site naming a recipe asks for one the crossing
    /// has. Whether a crossing with several recipes says which of them is the
    /// default is [`RecipesBuilder::build`](super::RecipesBuilder::build)'s.
    pub fn build(self, recipes: &Recipes) -> Result<Bindings, Vec<RecipeError>> {
        let mut errors: Vec<RecipeError> = self
            .conflicts
            .into_values()
            .map(|(site, origin)| RecipeError::Rebound { site, origin })
            .collect();
        let mut bound = HashMap::new();
        for (site, binding) in self.asks {
            let key = binding.crossing.key();
            let recipe = match &binding.ask {
                Ask::Omit => {
                    bound.insert(site, None);
                    continue;
                }
                Ask::Default => recipes.recipe(&binding.crossing).0,
                Ask::Recipe(name) => {
                    let Some(recipe) = recipes.key_of(&key, name) else {
                        errors.push(RecipeError::UnknownRecipe {
                            site,
                            recipe: key.row(name.clone()),
                        });
                        continue;
                    };
                    recipe.clone()
                }
            };
            bound.insert(
                site.clone(),
                Some(Bound {
                    site,
                    crossing: binding.crossing,
                    recipe,
                    origin: binding.origin,
                }),
            );
        }
        if errors.is_empty() {
            Ok(Bindings { bound })
        } else {
            Err(errors)
        }
    }
}

/// Which row every declared site takes. Built by [`BindingsBuilder`], checked
/// once, then immutable.
#[derive(Debug, Default)]
pub struct Bindings {
    /// `None` for a site a declaration omitted, which is a different answer
    /// from a site nobody bound.
    bound: HashMap<Site, Option<Bound>>,
}

impl Bindings {
    /// Start describing the sites of one binding crate.
    pub fn builder() -> BindingsBuilder {
        BindingsBuilder::default()
    }

    /// The crossing a declaration bound this site to, where one did.
    ///
    /// A site's crossing is usually the model's — a parameter crosses its own
    /// type. It is not always: a declaration may bind a return to the value its
    /// decomposition produces rather than to what the signature says. So a
    /// caller enumerating sites asks here first and falls back to the model,
    /// which is what keeps the two answers from disagreeing.
    pub fn crossing_of(&self, site: &Site) -> Option<Crossing> {
        self.bound
            .get(site)?
            .as_ref()
            .map(|bound| bound.crossing.clone())
    }

    /// The recipe this site takes.
    ///
    /// `None` when a declaration bound the site to [`Ask::Omit`]. A site nobody
    /// bound takes its crossing's default recipe, attributed to
    /// [`Origin::Adapter`], so an adapter asks here for every site rather than
    /// checking first whether one was declared.
    pub fn resolve(&self, site: &Site, crossing: &Crossing, recipes: &Recipes) -> Option<Bound> {
        match self.bound.get(site) {
            Some(bound) => {
                // A site is one place, so the crossing a caller asks with must
                // be the one the declaration bound. Getting a plausible answer
                // back for the wrong type would hide the caller's bug.
                debug_assert!(
                    bound
                        .as_ref()
                        .is_none_or(|b| b.crossing.key() == crossing.key()),
                    "{site} was bound as {} and is being resolved as {}",
                    bound
                        .as_ref()
                        .map(|b| b.crossing.key().to_string())
                        .unwrap_or_default(),
                    crossing.key(),
                );
                bound.clone()
            }
            None => Some(Bound {
                site: site.clone(),
                recipe: recipes.recipe(crossing).0,
                crossing: crossing.clone(),
                origin: Origin::Adapter,
            }),
        }
    }

    /// Whether any declaration named this site.
    pub fn is_declared(&self, site: &Site) -> bool {
        self.bound.contains_key(site)
    }
}
