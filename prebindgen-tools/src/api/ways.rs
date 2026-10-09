use std::{collections::HashMap, fmt, marker::PhantomData};

use prebindgen_flat::flat::{Function, Struct, TypeKind, TypeRef, Variant};

use crate::{
    api::crossing::{Direction, In, Out},
    Place, ResolvedConversion,
};

/// A way that serves both directions.
pub enum Both {}

/// One way a type may cross, as registered in [`Ways`].
///
/// Each variant holds the model item that fixes the structure below it, so
/// what a way can contain is decided by the model, never by the adapter:
/// [`Way::Fields`] has one part per field of its struct, [`Way::Constructed`]
/// one per parameter of its function.
pub enum Way<'f, L> {
    /// Whole, on one wire; `L` is the adapter's declaration (a handle, an
    /// enum, a mirror) that its lowering turns into the wire.
    Whole(L),
    /// As the fields of this struct.
    Fields(&'f Struct),
    /// As a tag and the fields of every alternative of this sum.
    Alternatives(&'f Variant),
    /// As the representation this conversion declares.
    Converted(Box<ResolvedConversion>),
    /// Into Rust only: as the arguments of this function, which builds the
    /// value.
    Constructed(&'f Function),
}

/// A registered way, typed by the directions it can serve: [`Both`], or only
/// [`In`] or [`Out`]. A `WayId<Both>` converts into either.
pub struct WayId<D> {
    index: usize,
    _direction: PhantomData<fn() -> D>,
}

impl<D> WayId<D> {
    fn new(index: usize) -> Self {
        Self {
            index,
            _direction: PhantomData,
        }
    }
}

impl<D> Clone for WayId<D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D> Copy for WayId<D> {}

impl<D> fmt::Debug for WayId<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WayId({})", self.index)
    }
}

impl From<WayId<Both>> for WayId<In> {
    fn from(id: WayId<Both>) -> Self {
        Self::new(id.index)
    }
}

impl From<WayId<Both>> for WayId<Out> {
    fn from(id: WayId<Both>) -> Self {
        Self::new(id.index)
    }
}

struct Registered<'f, L> {
    /// The type the way is for.
    key: String,
    way: Way<'f, L>,
    /// Whether it builds a value and so serves [`In`] only.
    in_only: bool,
}

/// Every way each type may cross, registered once and checked against the
/// model when registered. Which one an occurrence takes is a [`Choices`].
pub struct Ways<'f, L> {
    ways: Vec<Registered<'f, L>>,
}

impl<L> Default for Ways<'_, L> {
    fn default() -> Self {
        Self { ways: Vec::new() }
    }
}

/// The identity a type is registered under: a named type by its name, any
/// other type by its canonical form.
pub(crate) fn key_of(ty: &TypeRef) -> String {
    match ty.kind() {
        TypeKind::Named { id, .. } => id.name.clone(),
        TypeKind::String => "String".into(),
        _ => ty.key().as_str().to_string(),
    }
}

impl<'f, L> Ways<'f, L> {
    pub fn new() -> Self {
        Self::default()
    }

    fn push<D>(&mut self, ty: &TypeRef, way: Way<'f, L>, in_only: bool) -> WayId<D> {
        self.ways.push(Registered {
            key: key_of(ty),
            way,
            in_only,
        });
        WayId::new(self.ways.len() - 1)
    }

    /// `ty` crosses whole, as `leaf` says.
    pub fn whole(&mut self, ty: &TypeRef, leaf: L) -> WayId<Both> {
        self.push(ty, Way::Whole(leaf), false)
    }

    /// The struct crosses as its fields.
    pub fn fields(&mut self, s: &'f Struct) -> WayId<Both> {
        self.push(s.type_ref(), Way::Fields(s), false)
    }

    /// The sum crosses as a tag and its alternatives' fields.
    pub fn alternatives(&mut self, v: &'f Variant) -> WayId<Both> {
        self.push(v.type_ref(), Way::Alternatives(v), false)
    }

    /// The conversion's target crosses as its representation.
    pub fn converted(&mut self, conversion: ResolvedConversion) -> WayId<Both> {
        let ty = conversion.target().clone();
        self.push(&ty, Way::Converted(Box::new(conversion)), false)
    }

    /// The type `f` returns (directly or as the `Ok` of a `Result`) is built
    /// by calling `f` on its arguments.
    pub fn constructed(&mut self, f: &'f Function) -> Result<WayId<In>, String> {
        let ret = match f.ret.kind() {
            TypeKind::Fallible { ok, .. } => &**ok,
            _ => &f.ret,
        };
        if !matches!(ret.kind(), TypeKind::Named { .. }) {
            return Err(format!(
                "constructor `{}` must return a named type, returns `{ret}`",
                f.name
            ));
        }
        Ok(self.push(&ret.clone(), Way::Constructed(f), true))
    }

    /// The way `id` names.
    pub fn get<D>(&self, id: WayId<D>) -> &Way<'f, L> {
        &self.ways[id.index].way
    }

    /// The ways registered for `ty`, or for the type it borrows.
    fn of<D: Direction>(&self, ty: &TypeRef) -> Vec<usize> {
        let keys = [Some(key_of(ty)), ty.borrow_target().map(key_of)];
        for key in keys.iter().flatten() {
            let ids: Vec<usize> = (0..self.ways.len())
                .filter(|&i| &self.ways[i].key == key && (D::IN || !self.ways[i].in_only))
                .collect();
            if !ids.is_empty() {
                return ids;
            }
        }
        Vec::new()
    }

    /// Whether way `index` is for `ty` or the type it borrows.
    fn is_for(&self, index: usize, ty: &TypeRef) -> bool {
        let key = &self.ways[index].key;
        *key == key_of(ty) || ty.borrow_target().is_some_and(|t| *key == key_of(t))
    }
}

/// Which registered way each occurrence takes, in direction `D`: a default
/// per type, replaced at chosen places.
///
/// A type with a single way registered for `D` needs no choice.
pub struct Choices<D> {
    defaults: HashMap<String, usize>,
    places: HashMap<Place, usize>,
    _direction: PhantomData<fn() -> D>,
}

impl<D> Default for Choices<D> {
    fn default() -> Self {
        Self {
            defaults: HashMap::new(),
            places: HashMap::new(),
            _direction: PhantomData,
        }
    }
}

impl<D: Direction> Choices<D> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Every occurrence of the way's type takes it, unless a place chooses
    /// otherwise.
    pub fn choose<L>(&mut self, ways: &Ways<'_, L>, way: impl Into<WayId<D>>) {
        let index = way.into().index;
        self.defaults.insert(ways.ways[index].key.clone(), index);
    }

    /// The occurrence at `place` takes `way`. Resolving fails if the type
    /// at `place` is not the way's.
    pub fn choose_at(&mut self, place: Place, way: impl Into<WayId<D>>) {
        self.places.insert(place, way.into().index);
    }

    /// The way the occurrence of `ty` at `place` takes: the place's choice,
    /// else the type's default, else its only way. `None` when the type has
    /// no way registered; an error when the choice is ambiguous or names
    /// another type's way.
    pub(crate) fn pick<'w, 'f, L>(
        &self,
        ways: &'w Ways<'f, L>,
        ty: &TypeRef,
        place: &Place,
    ) -> Result<Option<&'w Way<'f, L>>, String> {
        if let Some(&i) = self.places.get(place) {
            if !ways.is_for(i, ty) {
                return Err(format!(
                    "{place}: the way chosen here is for `{}`, the type here is `{ty}`",
                    ways.ways[i].key
                ));
            }
            return Ok(Some(&ways.ways[i].way));
        }
        let ids = ways.of::<D>(ty);
        if let Some(&i) = ids
            .iter()
            .find(|&&i| self.defaults.get(&ways.ways[i].key) == Some(&i))
        {
            return Ok(Some(&ways.ways[i].way));
        }
        match ids.as_slice() {
            [] => Ok(None),
            [i] => Ok(Some(&ways.ways[*i].way)),
            _ => Err(format!(
                "{place}: `{ty}` has {} ways to cross {}; choose one",
                ids.len(),
                D::NAME
            )),
        }
    }
}
