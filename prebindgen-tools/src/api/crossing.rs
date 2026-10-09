use std::fmt;

use prebindgen_flat::flat::{Alternative, Field, Function, Struct, TypeRef, Variant};

use crate::{
    api::{
        shape::{shape, Access, SequenceKind, Shape},
        ways::{Choices, Way, Ways},
    },
    names, Place, ResolvedConversion, Seg, Wire, WireType,
};

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::In {}
    impl Sealed for super::Out {}
}

/// Into Rust: wires → a source value.
#[derive(Debug)]
pub enum In {}
/// Out of Rust: a source value → wires.
#[derive(Debug)]
pub enum Out {}

/// Which way a value crosses. A [`Crossing`] is typed by it, so a node only
/// one direction can have — a value built by a function, say — cannot
/// appear in the other.
pub trait Direction: sealed::Sealed + Sized + 'static {
    /// A value built from crossing arguments: [`Constructed`] into Rust,
    /// [`Never`] out of it.
    type Constructed<'f, W: WireType>: fmt::Debug;
    #[doc(hidden)]
    const IN: bool;
    #[doc(hidden)]
    const NAME: &'static str;
    #[doc(hidden)]
    fn constructed<'f, W: WireType>(
        func: &'f Function,
        args: Vec<Crossing<'f, W, Self>>,
    ) -> Option<Self::Constructed<'f, W>>;
    #[doc(hidden)]
    fn args<'s, 'f: 's, W: WireType>(
        c: &'s Self::Constructed<'f, W>,
    ) -> &'s [Crossing<'f, W, Self>];
}

/// A node that cannot exist in this direction.
#[derive(Debug)]
pub enum Never {}

impl Direction for In {
    type Constructed<'f, W: WireType> = Constructed<'f, W>;
    const IN: bool = true;
    const NAME: &'static str = "into Rust";
    fn constructed<'f, W: WireType>(
        func: &'f Function,
        args: Vec<Crossing<'f, W, In>>,
    ) -> Option<Constructed<'f, W>> {
        Some(Constructed { func, args })
    }
    fn args<'s, 'f: 's, W: WireType>(c: &'s Constructed<'f, W>) -> &'s [Crossing<'f, W, In>] {
        &c.args
    }
}

impl Direction for Out {
    type Constructed<'f, W: WireType> = Never;
    const IN: bool = false;
    const NAME: &'static str = "out of Rust";
    fn constructed<'f, W: WireType>(
        _: &'f Function,
        _: Vec<Crossing<'f, W, Out>>,
    ) -> Option<Never> {
        None
    }
    fn args<'s, 'f: 's, W: WireType>(c: &'s Never) -> &'s [Crossing<'f, W, Out>] {
        match *c {}
    }
}

/// How one occurrence of a type crosses in direction `D`: the decision made
/// at every layer of the type, down to the wires.
///
/// A crossing is the model's type tree with a decision installed at each
/// node. Only [`resolve`] builds one, by walking the type and taking the
/// [`Choices`] made for each place, so its structure is the type's: a
/// [`Node::Fields`] has one child per field of its struct, a
/// [`Node::Optional`] one inner value of the option's inner type. Read it
/// to write either side of the boundary; turn it into Rust with
/// [`Crossing::decode`] or [`Crossing::encode`].
#[derive(Debug)]
pub struct Crossing<'f, W: WireType, D: Direction> {
    ty: TypeRef,
    place: Place,
    node: Node<'f, W, D>,
}

/// The decision at one node of a [`Crossing`].
#[derive(Debug)]
pub enum Node<'f, W: WireType, D: Direction> {
    /// Whole, on one wire.
    Wire(Wire<W>),
    /// Nothing crosses: `()`.
    Unit,
    /// A struct, as its fields.
    Fields(Fields<'f, W, D>),
    /// A sum, as a tag and every alternative's fields.
    Alternatives(Alternatives<'f, W, D>),
    /// A converted type, as its representation.
    Converted(Converted<'f, W, D>),
    /// `Option<T>`, as a presence test and `T`.
    Optional(Optional<'f, W, D>),
    /// `Vec<T>` or `[T]`, on the adapter's wires, each element as `T`.
    Sequence(Sequence<'f, W, D>),
    /// `Box<T>`, `&T`, `Cow<T>`, as `T`.
    Wrapped(Wrapped<'f, W, D>),
    /// A value built by a function from its arguments.
    Constructed(D::Constructed<'f, W>),
}

/// A struct crossing as its fields.
#[derive(Debug)]
pub struct Fields<'f, W: WireType, D: Direction> {
    item: &'f Struct,
    fields: Vec<Crossing<'f, W, D>>,
}

/// The fields of one alternative of a sum.
#[derive(Debug)]
pub struct Arm<'f, W: WireType, D: Direction> {
    alt: &'f Alternative,
    fields: Vec<Crossing<'f, W, D>>,
}

/// A sum crossing as a tag and the fields of every alternative: the taken
/// one's carry the value, the others' placeholders.
#[derive(Debug)]
pub struct Alternatives<'f, W: WireType, D: Direction> {
    item: &'f Variant,
    tag: Wire<W>,
    arms: Vec<Arm<'f, W, D>>,
}

/// A type crossing as the representation its conversion declares.
#[derive(Debug)]
pub struct Converted<'f, W: WireType, D: Direction> {
    conversion: Box<ResolvedConversion>,
    repr: Box<Crossing<'f, W, D>>,
}

/// How an optional value says it is absent.
#[derive(Debug)]
pub enum Presence<W> {
    /// A flag wire, before the inner value's wires.
    Flag(Wire<W>),
    /// The inner value's wires: absent is their placeholders.
    Niche,
}

/// `Option<T>` crossing as a presence and `T`.
#[derive(Debug)]
pub struct Optional<'f, W: WireType, D: Direction> {
    presence: Presence<W>,
    inner: Box<Crossing<'f, W, D>>,
}

/// A sequence on the adapter's wires; each element crosses as `elem`, whose
/// wires are the per-element locals the adapter's loop binds.
#[derive(Debug)]
pub struct Sequence<'f, W: WireType, D: Direction> {
    kind: SequenceKind,
    access: Access,
    wires: Vec<Wire<W>>,
    elem: Box<Crossing<'f, W, D>>,
}

/// What a [`Wrapped`] node wraps its inner value in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrapper {
    Box,
    Ref(Access),
    Cow,
}

/// A transparent wrapper crossing as what it wraps.
#[derive(Debug)]
pub struct Wrapped<'f, W: WireType, D: Direction> {
    wrapper: Wrapper,
    inner: Box<Crossing<'f, W, D>>,
}

/// A value built by calling a function on crossing arguments, one per
/// parameter.
#[derive(Debug)]
pub struct Constructed<'f, W: WireType> {
    func: &'f Function,
    args: Vec<Crossing<'f, W, In>>,
}

impl<'f, W: WireType, D: Direction> Crossing<'f, W, D> {
    /// The source type of this occurrence.
    pub fn ty(&self) -> &TypeRef {
        &self.ty
    }

    /// Where it occurs.
    pub fn place(&self) -> &Place {
        &self.place
    }

    /// The decision made here.
    pub fn node(&self) -> &Node<'f, W, D> {
        &self.node
    }

    /// The boundary wires, in the order the Rust side reads or produces them
    /// and the foreign side passes or unpacks them.
    pub fn wires(&self) -> Vec<&Wire<W>> {
        let mut out = Vec::new();
        self.collect(&mut out);
        out
    }

    fn collect<'s>(&'s self, out: &mut Vec<&'s Wire<W>>) {
        match &self.node {
            Node::Wire(w) => out.push(w),
            Node::Unit => {}
            Node::Fields(f) => f.fields.iter().for_each(|c| c.collect(out)),
            Node::Alternatives(a) => {
                out.push(&a.tag);
                for arm in &a.arms {
                    arm.fields.iter().for_each(|c| c.collect(out));
                }
            }
            Node::Converted(c) => c.repr.collect(out),
            Node::Optional(o) => {
                if let Presence::Flag(w) = &o.presence {
                    out.push(w);
                }
                o.inner.collect(out)
            }
            Node::Sequence(s) => out.extend(&s.wires),
            Node::Wrapped(w) => w.inner.collect(out),
            Node::Constructed(c) => D::args(c).iter().for_each(|c| c.collect(out)),
        }
    }
}

impl<'f, W: WireType, D: Direction> Fields<'f, W, D> {
    pub fn item(&self) -> &'f Struct {
        self.item
    }
    /// Each field with its crossing, in source order.
    pub fn fields(&self) -> impl Iterator<Item = (&'f Field, &Crossing<'f, W, D>)> {
        self.item.fields.iter().zip(&self.fields)
    }
}

impl<'f, W: WireType, D: Direction> Arm<'f, W, D> {
    pub fn alternative(&self) -> &'f Alternative {
        self.alt
    }
    /// Each field with its crossing, in source order.
    pub fn fields(&self) -> impl Iterator<Item = (&'f Field, &Crossing<'f, W, D>)> {
        self.alt.fields.iter().zip(&self.fields)
    }
    /// The wires of this alternative's fields, in order.
    pub fn wires(&self) -> Vec<&Wire<W>> {
        self.fields.iter().flat_map(|c| c.wires()).collect()
    }
}

impl<'f, W: WireType, D: Direction> Alternatives<'f, W, D> {
    pub fn item(&self) -> &'f Variant {
        self.item
    }
    pub fn tag(&self) -> &Wire<W> {
        &self.tag
    }
    /// One arm per alternative, in source order.
    pub fn arms(&self) -> &[Arm<'f, W, D>] {
        &self.arms
    }
}

impl<'f, W: WireType, D: Direction> Converted<'f, W, D> {
    pub fn conversion(&self) -> &ResolvedConversion {
        &self.conversion
    }
    pub fn repr(&self) -> &Crossing<'f, W, D> {
        &self.repr
    }
}

impl<'f, W: WireType, D: Direction> Optional<'f, W, D> {
    pub fn presence(&self) -> &Presence<W> {
        &self.presence
    }
    pub fn inner(&self) -> &Crossing<'f, W, D> {
        &self.inner
    }
}

impl<'f, W: WireType, D: Direction> Sequence<'f, W, D> {
    pub fn kind(&self) -> SequenceKind {
        self.kind
    }
    pub fn access(&self) -> Access {
        self.access
    }
    /// The wires the whole sequence crosses on.
    pub fn wires(&self) -> &[Wire<W>] {
        &self.wires
    }
    /// How one element crosses; its wires are the loop's locals.
    pub fn elem(&self) -> &Crossing<'f, W, D> {
        &self.elem
    }
}

impl<'f, W: WireType, D: Direction> Wrapped<'f, W, D> {
    pub fn wrapper(&self) -> Wrapper {
        self.wrapper
    }
    pub fn inner(&self) -> &Crossing<'f, W, D> {
        &self.inner
    }
}

impl<'f, W: WireType> Constructed<'f, W> {
    pub fn func(&self) -> &'f Function {
        self.func
    }
    /// Each parameter with its argument's crossing, in order.
    pub fn args(
        &self,
    ) -> impl Iterator<Item = (&'f prebindgen_flat::flat::Param, &Crossing<'f, W, In>)> {
        self.func.params.iter().zip(&self.args)
    }
}

/// What resolving asks the adapter: the choices made, and how its boundary
/// carries what the model leaves open — which values cross whole and on what
/// wire, how an option says it is absent, which wires carry a sequence.
pub trait Lower<'f, D: Direction> {
    /// The adapter's wire types.
    type Wire: WireType;
    /// The adapter's declaration for a type that crosses whole.
    type Leaf;
    type Error: From<String>;

    /// Every way each type may cross.
    fn ways(&self) -> &Ways<'f, Self::Leaf>;
    /// Which way each occurrence takes.
    fn choices(&self) -> &Choices<D>;

    /// The wire the value at `place` crosses whole on, or `None` to cross
    /// it by its structure. A type whose chosen way is [`Way::Whole`]
    /// arrives as [`Shape::Declared`] with that declaration, and must get a
    /// wire.
    fn whole(
        &self,
        ty: &TypeRef,
        shape: Shape<'_, &Self::Leaf>,
        place: &Place,
    ) -> Result<Option<Wire<Self::Wire>>, Self::Error>;

    /// How the option at `place`, whose inner value crosses as `inner`,
    /// says it is absent.
    fn presence(
        &self,
        inner: &Crossing<'f, Self::Wire, D>,
        place: &Place,
    ) -> Result<Presence<Self::Wire>, Self::Error>;

    /// The wires the sequence at `place` crosses on, its element crossing
    /// as `elem`.
    fn sequence(
        &self,
        elem: &Crossing<'f, Self::Wire, D>,
        place: &Place,
    ) -> Result<Vec<Wire<Self::Wire>>, Self::Error>;

    /// The tag wire of the sum at `place`.
    fn tag(&self, place: &Place) -> Wire<Self::Wire>;
}

/// Resolve how the occurrence of `ty` at `place` crosses: at each layer,
/// the way the choices pick for it, or else its structure.
pub fn resolve<'f, D: Direction, L: Lower<'f, D>>(
    lower: &L,
    ty: &TypeRef,
    place: Place,
) -> Result<Crossing<'f, L::Wire, D>, L::Error> {
    let node = match lower.choices().pick(lower.ways(), ty, &place)? {
        Some(way) => by_way(lower, ty, way, &place)?,
        None => by_structure(lower, ty, &place)?,
    };
    Ok(Crossing {
        ty: ty.clone(),
        place,
        node,
    })
}

/// One alternative of `v`, as a record of its fields: for an adapter that
/// writes a mirror of the sum rather than crossing it whole.
pub fn resolve_arm<'f, D: Direction, L: Lower<'f, D>>(
    lower: &L,
    v: &'f Variant,
    alt: &'f Alternative,
    place: &Place,
) -> Result<Arm<'f, L::Wire, D>, L::Error> {
    if !v.alternatives.iter().any(|a| std::ptr::eq(a, alt)) {
        return Err(format!("`{}` is not an alternative of `{}`", alt.name, v.name).into());
    }
    let at = place.at(Seg::Alt(names::bare(&alt.name)));
    Ok(Arm {
        alt,
        fields: fields(lower, &alt.fields, &at)?,
    })
}

fn fields<'f, D: Direction, L: Lower<'f, D>>(
    lower: &L,
    fields: &'f [Field],
    place: &Place,
) -> Result<Vec<Crossing<'f, L::Wire, D>>, L::Error> {
    fields
        .iter()
        .map(|f| resolve(lower, &f.ty, place.at(Seg::field(f))))
        .collect()
}

fn by_way<'f, D: Direction, L: Lower<'f, D>>(
    lower: &L,
    ty: &TypeRef,
    way: &Way<'f, L::Leaf>,
    place: &Place,
) -> Result<Node<'f, L::Wire, D>, L::Error> {
    Ok(match way {
        Way::Whole(leaf) => {
            let shape = Shape::Declared {
                ty,
                declaration: leaf,
            };
            match lower.whole(ty, shape, place)? {
                Some(w) => Node::Wire(w),
                None => {
                    return Err(format!(
                        "{place}: `{ty}` is declared to cross whole, but has no wire here"
                    )
                    .into())
                }
            }
        }
        Way::Fields(s) => Node::Fields(Fields {
            item: s,
            fields: fields(lower, &s.fields, place)?,
        }),
        Way::Alternatives(v) => Node::Alternatives(Alternatives {
            item: v,
            tag: lower.tag(place),
            arms: v
                .alternatives
                .iter()
                .map(|alt| resolve_arm(lower, v, alt, place))
                .collect::<Result<_, _>>()?,
        }),
        Way::Converted(c) => Node::Converted(Converted {
            conversion: c.clone(),
            repr: Box::new(resolve(lower, c.repr(), place.at(Seg::Repr))?),
        }),
        Way::Constructed(func) => {
            let args = func
                .params
                .iter()
                .map(|p| resolve(lower, &p.ty, place.at(Seg::Param(names::bare(&p.name)))))
                .collect::<Result<_, _>>()?;
            match D::constructed(func, args) {
                Some(c) => Node::Constructed(c),
                None => unreachable!("a constructor is only picked into Rust"),
            }
        }
    })
}

fn by_structure<'f, D: Direction, L: Lower<'f, D>>(
    lower: &L,
    ty: &TypeRef,
    place: &Place,
) -> Result<Node<'f, L::Wire, D>, L::Error> {
    let layer = || shape(ty, |_| None::<&L::Leaf>).map_err(|e| format!("{place}: {e}"));
    if let Some(w) = lower.whole(ty, layer()?, place)? {
        return Ok(Node::Wire(w));
    }
    let wrapped = |wrapper, inner: &TypeRef| -> Result<Node<'f, L::Wire, D>, L::Error> {
        Ok(Node::Wrapped(Wrapped {
            wrapper,
            inner: Box::new(resolve(lower, inner, place.clone())?),
        }))
    };
    Ok(match layer()? {
        Shape::Unit => Node::Unit,
        Shape::Option(inner) => {
            let inner = resolve(lower, inner, place.at(Seg::Some))?;
            Node::Optional(Optional {
                presence: lower.presence(&inner, place)?,
                inner: Box::new(inner),
            })
        }
        Shape::Seq { elem, kind, access } => {
            let elem = resolve(lower, elem, place.at(Seg::Elem))?;
            Node::Sequence(Sequence {
                kind,
                access,
                wires: lower.sequence(&elem, place)?,
                elem: Box::new(elem),
            })
        }
        Shape::Boxed(inner) => wrapped(Wrapper::Box, inner)?,
        Shape::Cow(inner) => wrapped(Wrapper::Cow, inner)?,
        Shape::Ref { inner, access } => wrapped(Wrapper::Ref(access), inner)?,
        Shape::Undeclared(name) => {
            return Err(format!("{place}: `{name}` has no way to cross; declare one").into())
        }
        _ => return Err(format!("{place}: `{ty}` cannot cross here").into()),
    })
}
