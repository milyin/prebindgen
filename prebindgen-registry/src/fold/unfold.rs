//! The deconstructing half of the view: leaves read off a row that takes a
//! value apart.
//!
//! The constructing half is in the parent module, and the two answer the same
//! question in opposite directions — what a row costs on the foreign signature,
//! in order, with names. What a row does not say, the target does, through an
//! [`UnfoldPolicy`]: what a selector and a presence flag are, what one part is
//! called, and how a nested part's name joins its parent's.
//!
//! Which parts are taken apart further is a **binding**, not a property of the
//! row: a part whose type states a row still crosses whole unless something
//! says to read it in pieces. That is #701's design decision 3, and it is what
//! `Bindings` carries here.
//!
//! Only what a row can already state is here. `Reach::Path` and `Reach::Nested`
//! are the two forms step 3 replaces with part bindings; until they are, a row
//! using one is refused rather than read as something else.

use prebindgen_flat::flat::TypeRef;

use super::Folding;
use crate::{
    leaf::{Hoist, LeafSource, PathStep, UnfoldLeaf},
    recipe::{Deconstruct, Reach, Recipe, RecipeName, Shape},
};

/// What a target decides when a row is taken apart into leaves.
///
/// A row states which parts there are and how each is reached. It does not
/// state what any of them is *called* — an accessor's leaf name is the
/// declaration's, not the function's — nor what a synthesized leaf carries.
pub trait UnfoldPolicy {
    /// The leaf saying which alternative of a sum is live.
    ///
    /// Its type is the sum itself rather than the integer it crosses as: what
    /// the emitter needs is which sum it is choosing between.
    fn selector(&self, source: &TypeRef) -> UnfoldLeaf;

    /// The leaf saying whether an optional value the decomposition looks
    /// through is present.
    fn presence(&self, name: &str) -> UnfoldLeaf;

    /// Whether the parts being named are a value form's.
    ///
    /// A value form's parts are named by the declaration that lists them, and a
    /// product's own by whatever the target calls the field. The two are
    /// different lists, and only the caller knows which is being walked.
    fn value_form_part(&self, _index: usize) -> Option<String> {
        None
    }

    /// What the part reached by `reach` is called.
    ///
    /// The declaration's answer, which is why the row cannot give it: two
    /// bindings may read one accessor under two names. `field` is the model's
    /// name for the field a [`Reach::Field`] reads, which a target that names
    /// its slots after the struct's own fields needs and one that names them
    /// after the declaration ignores.
    fn part_name(&self, reach: &Reach, index: usize, field: Option<&syn::Ident>) -> String;

    /// What one payload of one alternative is called.
    ///
    /// Not [`Self::part_name`] with the alternative prefixed: a target names an
    /// arm's payload after what the alternative is called on ITS side, which is
    /// not the Rust variant's name, and after how it addresses the field. Both
    /// are the target's, and neither is in the row.
    fn arm_part_name(&self, variant: &syn::Ident, member: &syn::Member, index: usize) -> String;

    /// What the value itself is called when it is one of the parts.
    fn identity_name(&self) -> String;

    /// How a nested part's name joins the name of the part it was reached
    /// through.
    fn nest(&self, outer: &str, inner: &str) -> String;
}

/// Why a row could not be taken apart into leaves.
#[derive(Debug)]
pub enum UnfoldViewError {
    /// A row named a function the model does not have.
    UnknownAccessor(syn::Ident),
    /// A row states a form the view does not read yet.
    NotYetReadable {
        /// The crossing whose row was walked.
        crossing: String,
        /// The form it used.
        form: &'static str,
    },
    /// More than one part of one level is the value itself.
    ManyIdentities {
        /// The value being taken apart.
        ty: String,
    },
    /// Two leaves of one decomposition carry the same name.
    DuplicateName {
        /// The value being taken apart.
        ty: String,
        /// The name written twice.
        name: String,
    },
    /// A part's row reaches the value being taken apart.
    Cycle {
        /// The type the walk came back to.
        ty: String,
    },
}

impl std::fmt::Display for UnfoldViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnfoldViewError::UnknownAccessor(name) => {
                write!(f, "no `#[prebindgen]` function named `{name}`")
            }
            UnfoldViewError::NotYetReadable { crossing, form } => write!(
                f,
                "`{crossing}`'s row uses {form}, which the leaf view does not read yet"
            ),
            UnfoldViewError::ManyIdentities { ty } => write!(
                f,
                "`{ty}` hands itself over more than once, and it has only one to give"
            ),
            UnfoldViewError::DuplicateName { ty, name } => {
                write!(f, "`{ty}` gives two of its leaves the name `{name}`")
            }
            UnfoldViewError::Cycle { ty } => {
                write!(
                    f,
                    "`{ty}` is taken apart into a part that is taken apart into it"
                )
            }
        }
    }
}

/// What one walk carries: the target's answers, the row name a nested part is
/// looked up under, what has been read so far, and the chain of types being
/// taken apart, which is what catches a value reached from itself.
struct Walk<'w> {
    policy: &'w dyn UnfoldPolicy,
    bindings: &'w crate::recipe::Bindings,
    row: &'w RecipeName,
    leaves: Vec<UnfoldLeaf>,
    hoists: Vec<Hoist>,
    reading: Vec<String>,
}

/// Where one level of the walk stands: the path reached so far, the name
/// prefix, and whether anything above was optional.
#[derive(Clone)]
struct At {
    path: Vec<PathStep>,
    name: Option<String>,
    nullable: bool,
    /// Whether these parts are a value form's, which names them itself.
    value_form: bool,
}

impl Folding<'_> {
    /// The leaves one value is taken apart into, and the value forms to bind
    /// once on the way.
    ///
    /// `reading` is the value's own type. The row walked is the one named,
    /// because taking a value apart is what a site opts into — the crossing's
    /// default row is the value crossing whole.
    pub fn unfold(
        &self,
        policy: &dyn UnfoldPolicy,
        bindings: &crate::recipe::Bindings,
        reading: &TypeRef,
        row: &RecipeName,
    ) -> Result<(Vec<UnfoldLeaf>, Vec<Hoist>), UnfoldViewError> {
        let shape = self
            .deconstructing(reading, row)
            .ok_or_else(|| UnfoldViewError::NotYetReadable {
                crossing: reading.to_string(),
                form: "no row that takes the value apart",
            })?
            .clone();
        let mut walk = Walk {
            policy,
            bindings,
            row,
            leaves: Vec::new(),
            hoists: Vec::new(),
            reading: vec![reading.key().to_string()],
        };
        let at = At {
            path: Vec::new(),
            name: None,
            nullable: false,
            value_form: false,
        };
        self.level(&mut walk, reading, &shape, &at)?;
        unique_names(&walk.leaves, reading)?;
        Ok((walk.leaves, walk.hoists))
    }

    /// One row, at one place in the value.
    fn level(
        &self,
        walk: &mut Walk<'_>,
        source: &TypeRef,
        shape: &Shape<Deconstruct>,
        at: &At,
    ) -> Result<(), UnfoldViewError> {
        match shape {
            Shape::Product(Deconstruct::Fields(reaches)) => self.parts(walk, source, reaches, at),
            Shape::Product(Deconstruct::ValueForm { func, parts }) => {
                // The form is called once and every part hangs off that one
                // call, which is what a hoist records.
                let mut root = at.path.clone();
                root.push(PathStep::call(func.clone(), false, true));
                walk.hoists.push(Hoist {
                    prefix: root.clone(),
                    consuming: self.consumes(func)?,
                });
                let inner = At {
                    path: root,
                    name: at.name.clone(),
                    nullable: at.nullable,
                    value_form: true,
                };
                // The parts are read off what the call RETURNS, not off the
                // value it was called on: `Reach::Field(0)` here is the first
                // field of the bound struct.
                let bound = self.returns(func)?;
                self.parts(walk, &bound, parts, &inner)
            }
            Shape::Choice { arms } => {
                // The selector first, then one group per arm. Every arm's
                // leaves are written, and the selector says which of them is
                // live — an arm the value did not take fills its slots with the
                // wire's default rather than being absent.
                let mut tag = walk.policy.selector(source);
                // A sum reached AS A PART carries that part's path and name:
                // its leaves are read out of the field the walk arrived
                // through, not off the value the row belongs to.
                tag.path = at.path.clone();
                tag.nullable = at.nullable;
                if let Some(outer) = &at.name {
                    tag.name = walk.policy.nest(outer, &tag.name);
                }
                walk.leaves.push(tag);
                for arm in arms {
                    let Some(alternative) = arm.alternative else {
                        return Err(UnfoldViewError::NotYetReadable {
                            crossing: source.to_string(),
                            form: "an arm naming no alternative, which only a parameter builds",
                        });
                    };
                    let Deconstruct::Fields(reaches) = &arm.op else {
                        return Err(UnfoldViewError::NotYetReadable {
                            crossing: source.to_string(),
                            form: "an arm read through a value form",
                        });
                    };
                    let Some((variant, fields)) = self.alternative(source, alternative) else {
                        return Err(UnfoldViewError::NotYetReadable {
                            crossing: source.to_string(),
                            form: "an alternative the model does not have",
                        });
                    };
                    for (index, reach) in reaches.iter().enumerate() {
                        let Reach::Field(field) = reach else {
                            return Err(UnfoldViewError::NotYetReadable {
                                crossing: source.to_string(),
                                form: reach_form(reach),
                            });
                        };
                        let Some(payload) = fields.get(*field) else {
                            return Err(UnfoldViewError::NotYetReadable {
                                crossing: source.to_string(),
                                form: "a payload field the alternative does not have",
                            });
                        };
                        // Bound by a pattern in the arm rather than reached off
                        // the value, so the leaf carries no path.
                        // Named under the ALTERNATIVE: every arm writes its
                        // own slots, so two arms reading their first payload
                        // field would otherwise be two leaves of one name. What
                        // that name is, is the target's — see
                        // [`UnfoldPolicy::arm_part_name`].
                        let member = member_of(payload, *field);
                        let name = walk.policy.arm_part_name(variant, &member, index);
                        walk.leaves.push(UnfoldLeaf {
                            name: match &at.name {
                                Some(outer) => walk.policy.nest(outer, &name),
                                None => name,
                            },
                            path: at.path.clone(),
                            out_ty: payload.ty.clone(),
                            identity: false,
                            nullable: at.nullable,
                            source: LeafSource::VariantField {
                                variant: variant.clone(),
                                member,
                            },
                            groups: vec![alternative as i32],
                        });
                    }
                }
                Ok(())
            }
            Shape::Atomic => Ok(()),
            other => Err(UnfoldViewError::NotYetReadable {
                crossing: source.to_string(),
                form: name_of(other),
            }),
        }
    }

    /// Every part of one product.
    fn parts(
        &self,
        walk: &mut Walk<'_>,
        source: &TypeRef,
        reaches: &[Reach],
        at: &At,
    ) -> Result<(), UnfoldViewError> {
        // One value to give, so one part may be it. It keeps the position the
        // row puts it in: what goes last is its EMISSION, after every borrow
        // taken off the value has ended, and that is the emitter's ordering
        // rather than the leaf list's.
        let mut seen_identity = false;
        for (index, reach) in reaches.iter().enumerate() {
            if !matches!(reach, Reach::Identity) {
                self.part(walk, source, reach, index, at)?;
                continue;
            }
            if seen_identity {
                return Err(UnfoldViewError::ManyIdentities {
                    ty: source.key().to_string(),
                });
            }
            seen_identity = true;
            let name = match &at.name {
                Some(outer) => outer.clone(),
                None => walk.policy.identity_name(),
            };
            walk.leaves.push(UnfoldLeaf {
                name,
                path: at.path.clone(),
                // The value as it stands: owned where it is ours to give,
                // borrowed where it is lent and the leaf clones through its own
                // converter.
                out_ty: if at.path.is_empty() {
                    source.clone()
                } else {
                    source.borrowed()
                },
                identity: true,
                nullable: at.nullable,
                source: LeafSource::Reach,
                groups: Vec::new(),
            });
        }
        Ok(())
    }

    /// One part, spliced through its own row when it has one.
    fn part(
        &self,
        walk: &mut Walk<'_>,
        source: &TypeRef,
        reach: &Reach,
        index: usize,
        at: &At,
    ) -> Result<(), UnfoldViewError> {
        let name = match at
            .value_form
            .then(|| walk.policy.value_form_part(index))
            .flatten()
        {
            Some(name) => name,
            // A value form that names no part at this position falls back to
            // the ordinary answer, the same as a product's own part.
            None => walk
                .policy
                .part_name(reach, index, self.field_name(source, reach)),
        };
        let full = match &at.name {
            Some(outer) => walk.policy.nest(outer, &name),
            None => name,
        };
        let (step, ty) = match reach {
            Reach::Omit => return Ok(()),
            Reach::Accessor(func) => {
                let ret = self.returns(func)?;
                let optional = ret.optional_inner().is_some();
                let core = ret.optional_inner().unwrap_or(&ret);
                let owned = core.borrow_target().is_none();
                (
                    PathStep::call(func.clone(), optional, owned),
                    core.borrow_target().unwrap_or(core).clone(),
                )
            }
            Reach::Field(field) => {
                let ty = self.field_ty(source, *field).ok_or_else(|| {
                    UnfoldViewError::NotYetReadable {
                        crossing: source.to_string(),
                        form: "a field the model does not have",
                    }
                })?;
                let ident = self.field_ident(source, *field);
                (PathStep::field(ident, false), ty)
            }
            Reach::Identity => unreachable!("handled by the caller"),
            other => {
                return Err(UnfoldViewError::NotYetReadable {
                    crossing: source.to_string(),
                    form: reach_form(other),
                })
            }
        };
        let optional = step.is_optional();
        let mut path = at.path.clone();
        path.push(step);

        // A part is taken apart further only where a binding says so. Having a
        // row of its own is not enough — a `data_class` states one and still
        // crosses whole as an accessor's return, which is the difference a
        // binding carries and a row cannot (#701 decision 3).
        if let Some(child) = self.bound_row(walk, source, index, &ty).cloned() {
            let key = ty.key().to_string();
            if walk.reading.contains(&key) {
                return Err(UnfoldViewError::Cycle { ty: key });
            }
            walk.reading.push(key);
            let inner = At {
                path,
                name: Some(full),
                nullable: at.nullable || optional,
                value_form: false,
            };
            self.level(walk, &ty, &child, &inner)?;
            walk.reading.pop();
            return Ok(());
        }

        walk.leaves.push(UnfoldLeaf {
            name: full,
            path,
            out_ty: self.leaf_ty(reach, source, index),
            identity: false,
            nullable: at.nullable,
            source: LeafSource::Reach,
            groups: Vec::new(),
        });
        Ok(())
    }

    /// The model's name for the field a reach reads, when it reads one.
    fn field_name(&self, source: &TypeRef, reach: &Reach) -> Option<&syn::Ident> {
        let Reach::Field(index) = reach else {
            return None;
        };
        self.fields_of(source).get(*index)?.name.as_ref()
    }

    /// The row a part is taken apart by, if a binding names one.
    ///
    /// The site is the part's own position in the row being walked, which is
    /// the key `Compiler::part_of` builds for the same part — so an adapter
    /// writes one binding and both the compiler and this view find it.
    fn bound_row(
        &self,
        walk: &Walk<'_>,
        source: &TypeRef,
        index: usize,
        part: &TypeRef,
    ) -> Option<&Shape<Deconstruct>> {
        use crate::recipe::{Crossing, Direction, Site};

        let owner = Crossing::new(source.clone(), Direction::Deconstruct);
        let site = Site::arm_part(&owner.row(walk.row.clone()), None, index);
        let crossing = Crossing::new(part.clone(), Direction::Deconstruct);
        let bound = walk.bindings.resolve(&site, &crossing, self.recipes())?;
        match self.recipes().get(&bound.recipe)? {
            Recipe::Deconstructing(shape) => Some(shape),
            Recipe::Constructing(_) => None,
        }
    }

    /// One alternative of a declared sum: its name, and its payload fields.
    fn alternative(
        &self,
        source: &TypeRef,
        index: usize,
    ) -> Option<(&syn::Ident, &[prebindgen_flat::flat::Field])> {
        let value = source.borrow_target().unwrap_or(source).unwrapped();
        let prebindgen_flat::flat::TypeKind::Named { id, .. } = value.kind() else {
            return None;
        };
        let prebindgen_flat::flat::Type::Variant(v) = self.model().resolve(id)? else {
            return None;
        };
        let alternative = v.alternatives.get(index)?;
        Some((&alternative.name, alternative.fields.as_slice()))
    }

    /// This crossing's deconstructing row under `row`, if it states one.
    fn deconstructing(&self, ty: &TypeRef, row: &RecipeName) -> Option<&Shape<Deconstruct>> {
        let crossing =
            crate::recipe::Crossing::new(ty.clone(), crate::recipe::Direction::Deconstruct).key();
        match self.recipes().get(self.recipes().key_of(&crossing, row)?)? {
            Recipe::Deconstructing(shape) => Some(shape),
            Recipe::Constructing(_) => None,
        }
    }

    /// What a part's own leaf carries, when it is not spliced.
    fn leaf_ty(&self, reach: &Reach, source: &TypeRef, index: usize) -> TypeRef {
        match reach {
            Reach::Accessor(func) => self.returns(func).unwrap_or_else(|_| source.clone()),
            Reach::Field(field) => self
                .field_ty(source, *field)
                .unwrap_or_else(|| source.clone()),
            _ => {
                let _ = index;
                source.clone()
            }
        }
    }

    fn returns(&self, func: &syn::Ident) -> Result<TypeRef, UnfoldViewError> {
        self.model()
            .function(func)
            .map(|f| f.ret.clone())
            .ok_or_else(|| UnfoldViewError::UnknownAccessor(func.clone()))
    }

    fn consumes(&self, func: &syn::Ident) -> Result<bool, UnfoldViewError> {
        let f = self
            .model()
            .function(func)
            .ok_or_else(|| UnfoldViewError::UnknownAccessor(func.clone()))?;
        Ok(f.params
            .first()
            .is_some_and(|p| p.ty.borrow_target().is_none()))
    }

    fn field_ty(&self, source: &TypeRef, index: usize) -> Option<TypeRef> {
        self.fields_of(source).get(index).map(|f| f.ty.clone())
    }

    /// How a field is addressed: its name, or its position for a tuple struct.
    fn field_ident(&self, source: &TypeRef, index: usize) -> syn::Ident {
        self.fields_of(source)
            .get(index)
            .and_then(|f| f.name.clone())
            .unwrap_or_else(|| {
                syn::Ident::new(&format!("_{index}"), proc_macro2::Span::call_site())
            })
    }
}

/// Two leaves of one decomposition may not share a name: the foreign signature
/// names its arguments, and two of one name is one the emitter cannot write.
fn unique_names(leaves: &[UnfoldLeaf], source: &TypeRef) -> Result<(), UnfoldViewError> {
    let mut seen = std::collections::HashSet::new();
    for leaf in leaves {
        if !seen.insert(leaf.name.clone()) {
            return Err(UnfoldViewError::DuplicateName {
                ty: source.key().to_string(),
                name: leaf.name.clone(),
            });
        }
    }
    Ok(())
}

fn name_of(shape: &Shape<Deconstruct>) -> &'static str {
    match shape {
        Shape::Atomic => "atomic",
        Shape::Optional => "an optional layer",
        Shape::Sequence => "a sequence",
        Shape::Invoke => "a callable",
        Shape::Product(_) => "a product",
        Shape::Choice { .. } => "a choice",
    }
}

/// How a payload field is addressed in its arm's pattern: by name, or by
/// position for a tuple variant.
fn member_of(field: &prebindgen_flat::flat::Field, index: usize) -> syn::Member {
    match &field.name {
        Some(name) => syn::Member::Named(name.clone()),
        None => syn::Member::Unnamed(syn::Index::from(index)),
    }
}

fn reach_form(reach: &Reach) -> &'static str {
    match reach {
        Reach::Path(_) => "a field-of-a-field chain",
        Reach::Nested { .. } => "a field taken apart in place",
        _ => "a reach the view does not read",
    }
}

#[cfg(test)]
mod tests;
