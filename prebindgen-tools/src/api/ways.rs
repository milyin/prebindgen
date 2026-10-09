use std::{
    collections::HashMap,
    fmt,
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};

use prebindgen_flat::flat::{Function, GenericArg, Struct, TypeKind, TypeRef, Variant};

use crate::{
    api::crossing::{Direction, In, Out},
    Place, ResolvedConversion,
};

/// A way that serves both directions.
pub enum Both {}

const BOTH: (bool, bool) = (true, true);

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
///
/// An id is only valid with the [`Ways`] that issued it: [`Choices`] refuses
/// one from another registry.
pub struct WayId<D> {
    registry: usize,
    index: usize,
    _direction: PhantomData<fn() -> D>,
}

impl<D> WayId<D> {
    fn new(registry: usize, index: usize) -> Self {
        Self {
            registry,
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
        write!(f, "WayId({}.{})", self.registry, self.index)
    }
}

impl From<WayId<Both>> for WayId<In> {
    fn from(id: WayId<Both>) -> Self {
        Self::new(id.registry, id.index)
    }
}

impl From<WayId<Both>> for WayId<Out> {
    fn from(id: WayId<Both>) -> Self {
        Self::new(id.registry, id.index)
    }
}

struct Registered<'f, L> {
    /// The type the way is for.
    key: String,
    way: Way<'f, L>,
    /// The directions it serves: into Rust, out of Rust.
    serves: (bool, bool),
}

/// Every way each type may cross, registered once and checked against the
/// model when registered. Which one an occurrence takes is a [`Choices`].
pub struct Ways<'f, L> {
    /// This registry's identity, which its ids carry.
    id: usize,
    ways: Vec<Registered<'f, L>>,
}

impl<L> Default for Ways<'_, L> {
    fn default() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            ways: Vec::new(),
        }
    }
}

/// The identity a type is registered under: a named type by its name and
/// type arguments (`Foo<u8>` is not `Foo<u64>`; lifetimes do not count),
/// any other type by its canonical form.
pub(crate) fn key_of(ty: &TypeRef) -> String {
    match ty.kind() {
        TypeKind::Named { id, args } => {
            let args: Vec<String> = args
                .iter()
                .filter_map(|a| match a {
                    GenericArg::Type(t) => Some(key_of(t)),
                    GenericArg::Lifetime(_) => None,
                })
                .collect();
            if args.is_empty() {
                id.name.clone()
            } else {
                format!("{}<{}>", id.name, args.join(", "))
            }
        }
        TypeKind::String => "String".into(),
        _ => ty.key().as_str().to_string(),
    }
}

impl<'f, L> Ways<'f, L> {
    pub fn new() -> Self {
        Self::default()
    }

    fn push<D>(&mut self, ty: &TypeRef, way: Way<'f, L>, serves: (bool, bool)) -> WayId<D> {
        self.ways.push(Registered {
            key: key_of(ty),
            way,
            serves,
        });
        WayId::new(self.id, self.ways.len() - 1)
    }

    /// `ty` crosses whole, as `leaf` says.
    pub fn whole(&mut self, ty: &TypeRef, leaf: L) -> WayId<Both> {
        self.push(ty, Way::Whole(leaf), BOTH)
    }

    /// The struct crosses as its fields.
    pub fn fields(&mut self, s: &'f Struct) -> WayId<Both> {
        self.push(s.type_ref(), Way::Fields(s), BOTH)
    }

    /// The sum crosses as a tag and its alternatives' fields.
    pub fn alternatives(&mut self, v: &'f Variant) -> WayId<Both> {
        self.push(v.type_ref(), Way::Alternatives(v), BOTH)
    }

    /// The conversion's target crosses as its representation, both ways.
    /// Fails unless the conversion declares both an input and an output.
    pub fn converted(&mut self, conversion: ResolvedConversion) -> Result<WayId<Both>, String> {
        self.conversion(conversion, BOTH)
    }

    /// The same, into Rust only: the conversion must declare an input.
    pub fn converted_in(&mut self, conversion: ResolvedConversion) -> Result<WayId<In>, String> {
        self.conversion(conversion, (true, false))
    }

    /// The same, out of Rust only: the conversion must declare an output.
    pub fn converted_out(&mut self, conversion: ResolvedConversion) -> Result<WayId<Out>, String> {
        self.conversion(conversion, (false, true))
    }

    fn conversion<D>(
        &mut self,
        conversion: ResolvedConversion,
        serves: (bool, bool),
    ) -> Result<WayId<D>, String> {
        let declared = (conversion.has_input(), conversion.has_output());
        for (needed, has, what) in [
            (serves.0, declared.0, "input"),
            (serves.1, declared.1, "output"),
        ] {
            if needed && !has {
                return Err(format!(
                    "convert!({}) declares no {what}",
                    conversion.target()
                ));
            }
        }
        let ty = conversion.target().clone();
        Ok(self.push(&ty, Way::Converted(Box::new(conversion)), serves))
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
        Ok(self.push(&ret.clone(), Way::Constructed(f), (true, false)))
    }

    /// The way `id` names, or an error if another registry issued it.
    pub fn get<D>(&self, id: WayId<D>) -> Result<&Way<'f, L>, String> {
        self.index(id.registry, id.index).map(|i| &self.ways[i].way)
    }

    fn index(&self, registry: usize, index: usize) -> Result<usize, String> {
        if registry != self.id {
            return Err("a way id issued by another `Ways` registry".into());
        }
        Ok(index)
    }

    /// The ways registered for `ty`, else for the type it borrows (`true`).
    fn of<D: Direction>(&self, ty: &TypeRef) -> (Vec<usize>, bool) {
        let keys = [
            (Some(key_of(ty)), false),
            (ty.borrow_target().map(key_of), true),
        ];
        for (key, borrowed) in keys {
            let Some(key) = key else { continue };
            let ids: Vec<usize> = (0..self.ways.len())
                .filter(|&i| self.ways[i].key == key && self.serves::<D>(i))
                .collect();
            if !ids.is_empty() {
                return (ids, borrowed);
            }
        }
        (Vec::new(), false)
    }

    fn serves<D: Direction>(&self, index: usize) -> bool {
        let (into, out_of) = self.ways[index].serves;
        if D::IN {
            into
        } else {
            out_of
        }
    }

    /// Whether way `index` is for `ty` (`Some(false)`) or for the type it
    /// borrows (`Some(true)`).
    fn is_for(&self, index: usize, ty: &TypeRef) -> Option<bool> {
        let key = &self.ways[index].key;
        if *key == key_of(ty) {
            Some(false)
        } else if ty.borrow_target().is_some_and(|t| *key == key_of(t)) {
            Some(true)
        } else {
            None
        }
    }
}

/// The way an occurrence takes, and whether it is the way of the type the
/// occurrence borrows rather than of its own type.
pub(crate) struct Picked<'w, 'f, L> {
    /// Its position in the registry, which identifies it.
    pub(crate) index: usize,
    pub(crate) way: &'w Way<'f, L>,
    pub(crate) through_borrow: bool,
}

/// Which registered way each occurrence takes, in direction `D`: a default
/// per type, replaced at chosen places.
///
/// A type with a single way registered for `D` needs no choice.
pub struct Choices<D> {
    /// Way ids, as `(registry, index)`.
    defaults: HashMap<String, (usize, usize)>,
    places: HashMap<Place, (usize, usize)>,
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
    /// Fails if `ways` did not issue `way`.
    pub fn choose<L>(
        &mut self,
        ways: &Ways<'_, L>,
        way: impl Into<WayId<D>>,
    ) -> Result<(), String> {
        let id = way.into();
        let index = ways.index(id.registry, id.index)?;
        self.defaults
            .insert(ways.ways[index].key.clone(), (id.registry, index));
        Ok(())
    }

    /// The occurrence at `place` takes `way`. Resolving fails if the type
    /// at `place` is not the way's.
    pub fn choose_at(&mut self, place: Place, way: impl Into<WayId<D>>) {
        let id = way.into();
        self.places.insert(place, (id.registry, id.index));
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
    ) -> Result<Option<Picked<'w, 'f, L>>, String> {
        let picked = |i: usize, through_borrow| Picked {
            index: i,
            way: &ways.ways[i].way,
            through_borrow,
        };
        if let Some(&(registry, i)) = self.places.get(place) {
            let i = ways
                .index(registry, i)
                .map_err(|e| format!("{place}: {e}"))?;
            let Some(through_borrow) = ways.is_for(i, ty) else {
                return Err(format!(
                    "{place}: the way chosen here is for `{}`, the type here is `{ty}`",
                    ways.ways[i].key
                ));
            };
            return Ok(Some(picked(i, through_borrow)));
        }
        let (ids, through_borrow) = ways.of::<D>(ty);
        if let Some(&i) = ids
            .iter()
            .find(|&&i| self.defaults.get(&ways.ways[i].key) == Some(&(ways.id, i)))
        {
            return Ok(Some(picked(i, through_borrow)));
        }
        match ids.as_slice() {
            [] => Ok(None),
            [i] => Ok(Some(picked(*i, through_borrow))),
            _ => Err(format!(
                "{place}: `{ty}` has {} ways to cross {}; choose one",
                ids.len(),
                D::NAME
            )),
        }
    }
}
