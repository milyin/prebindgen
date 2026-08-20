//! Which row a given place in the generated API uses.
//!
//! [`Recipes`] says what rows a crossing has; this says which of them the
//! second parameter of `z_put`, or the `Err` arm of a return, or one part of
//! another row actually takes. A declaration states that through
//! [`BindingsBuilder::bind`], and [`Bindings::resolve`] answers it for one
//! site.
//!
//! A site nobody binds uses its crossing's default row, so the common case is
//! declared nowhere.

use std::{collections::HashMap, fmt};

use super::{Crossing, CrossingKey, RecipeError, RecipeId, Recipes};

/// One place in the generated API where a value crosses.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Site {
    /// The item the site belongs to: the exported function for a parameter or
    /// a return, the crossed type for a part.
    pub owner: syn::Ident,
    /// Which place within that item.
    pub role: Role,
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
    Param {
        /// Position in the parameter list.
        index: usize,
        /// The parameter's name, for diagnostics.
        name: String,
    },
    /// The value a method is called on.
    Receiver,
    /// What the function returns.
    Return,
    /// The `Err` arm of a `Result`.
    Error,
    /// One argument the Rust side passes out through a callback.
    ///
    /// Swaps jobs: Rust holds the value and pushes it out, so the argument is
    /// deconstructed even though it sits in a parameter list.
    CallbackArg {
        /// Which parameter of the exported function is the callback.
        param: usize,
        /// Which argument of that callback.
        arg: usize,
    },
    /// One part of another crossing's recipe.
    Part {
        /// The crossing whose row names this part.
        of: CrossingKey,
        /// Which of that crossing's rows.
        recipe: RecipeId,
        /// The part's position within the row.
        index: usize,
    },
    /// A `#[prebindgen]` constant's value.
    Const,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Param { index, name } => write!(f, "parameter {index} (`{name}`)"),
            Role::Receiver => f.write_str("receiver"),
            Role::Return => f.write_str("return value"),
            Role::Error => f.write_str("error arm"),
            Role::CallbackArg { param, arg } => {
                write!(f, "argument {arg} of the callback in parameter {param}")
            }
            Role::Part { of, recipe, index } => {
                write!(f, "part {index} of row `{recipe}` of {of}")
            }
            Role::Const => f.write_str("value"),
        }
    }
}

/// What a declaration asks for at one site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// The crossing's default row.
    Default,
    /// This named row of the crossing.
    Recipe(RecipeId),
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
    /// A per-part declaration, written against one part of one row.
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
    /// Which of the crossing's rows this site takes.
    pub recipe: RecipeId,
    /// Which declaration won, so a diagnostic can say why this row applies.
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
    /// [`Self::build`] rather than resolved by declaration order.
    conflicts: Vec<(Site, Origin)>,
}

impl BindingsBuilder {
    /// State which row one site takes.
    ///
    /// Where two declarations bind one site, the higher-precedence `origin`
    /// wins — with its `crossing` and its `ask` together, since the two are one
    /// declaration's answer. Two declarations of equal precedence asking for
    /// different rows is an error, and two asking for the same row is not.
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
                if held.ask != binding.ask {
                    self.conflicts.push((site, origin));
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
    /// The one check here is that a site naming a row asks for one the crossing
    /// has. Whether a crossing with several rows says which of them is the
    /// default is [`RecipesBuilder::build`](super::RecipesBuilder::build)'s.
    pub fn build(self, recipes: &Recipes) -> Result<Bindings, Vec<RecipeError>> {
        let mut errors: Vec<RecipeError> = self
            .conflicts
            .into_iter()
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
                Ask::Default => recipes.row(&binding.crossing).0,
                Ask::Recipe(id) => {
                    if recipes.get(&key, id).is_none() {
                        errors.push(RecipeError::UnknownRow {
                            site,
                            crossing: key,
                            recipe: id.clone(),
                        });
                        continue;
                    }
                    id.clone()
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

    /// The row this site takes.
    ///
    /// `None` when a declaration bound the site to [`Ask::Omit`]. A site nobody
    /// bound takes its crossing's default row, attributed to
    /// [`Origin::Adapter`], so an adapter asks here for every site rather than
    /// checking first whether one was declared.
    pub fn resolve(&self, site: &Site, crossing: &Crossing, recipes: &Recipes) -> Option<Bound> {
        match self.bound.get(site) {
            Some(bound) => bound.clone(),
            None => Some(Bound {
                site: site.clone(),
                recipe: recipes.row(crossing).0,
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
