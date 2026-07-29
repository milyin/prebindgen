//! The elements: one variant per structure the source language allows.
//!
//! Every node — an item, a parameter, a field, a variant, a type — carries one
//! [`Origin`], so generated Rust names the source by re-emitting what the source
//! wrote, nothing re-parses a whole item to find a part of it, and no level has
//! to copy a piece of provenance down from the level above.
//!
//! **Structure only.** Everything that turns an element back into Rust tokens
//! lives in [`spell`](super::spell), so the shape of an element says nothing
//! about the language it came from.

use super::{origin::Origin, ty::TypeRef};
use crate::SourceLocation;

/// One member of the flat API.
///
/// Three modelled kinds — a function, a type, a constant — plus
/// [`Element::Unsupported`] for anything the language cannot express. There is no
/// verbatim-passthrough variant: a `#[prebindgen]` crate marks the items that
/// cross the boundary, and the supporting code around them is the consumer
/// crate's job — the proc-macro enforces that already, refusing to mark a `use`,
/// `mod`, `impl` or `macro_rules!` at all.
#[derive(Clone, Debug)]
pub enum Element {
    Function(Function),
    /// A type declaration: a struct, either enum shape, or an opaque handle.
    Type(Type),
    Constant(Constant),
    /// An item the language cannot express — a parameter type outside the
    /// grammar, a `self` receiver, a reference to a type the flat API never
    /// declares, or a whole item kind it does not model such as a `union`.
    ///
    /// Inert: it is indexed under its name so nothing else can claim it, and
    /// the diagnosis rides along, to be raised by whatever declares it. See the
    /// [module docs](super) on where acceptance is enforced.
    Unsupported(Unsupported),
}

impl Element {
    /// The item's name, which is also its address: `#[prebindgen]` names live
    /// in one flat namespace across every ingested source crate.
    ///
    /// `None` when the item has no address — an unnamed `const _` (each
    /// source's injected feature guard, so several may coexist), or an item
    /// kind with no identifier at all.
    pub fn name(&self) -> Option<&syn::Ident> {
        let named = match self {
            Element::Function(f) => Some(&f.name),
            Element::Type(t) => Some(t.name()),
            Element::Constant(c) => Some(&c.name),
            Element::Unsupported(u) => u.name.as_ref(),
        };
        named.filter(|id| *id != "_")
    }

    /// Where the item was captured, including the crate that marked it.
    ///
    /// The same location every component of this item carries — they share one
    /// [`Origin::location`], because one captured record is one item.
    pub fn location(&self) -> &SourceLocation {
        match self {
            Element::Function(f) => &f.origin.location,
            Element::Type(t) => t.location(),
            Element::Constant(c) => &c.origin.location,
            Element::Unsupported(u) => &u.origin.location,
        }
    }

    /// The whole item as the source wrote it.
    pub fn syntax(&self) -> syn::Item {
        match self {
            Element::Function(f) => syn::Item::Fn(f.origin.syntax.clone()),
            Element::Type(t) => t.syntax(),
            Element::Constant(c) => syn::Item::Const(c.origin.syntax.clone()),
            Element::Unsupported(u) => u.origin.syntax.clone(),
        }
    }
}

/// A type the flat API declares.
///
/// Four shapes, and the classification is what a destination language acts on: a
/// product of fields, a sum, a named set of integers, or a handle whose contents
/// do not cross.
#[derive(Clone, Debug)]
pub enum Type {
    Struct(Struct),
    /// An enum whose alternatives carry payloads — a sum type.
    Variant(Variant),
    /// An enum whose every alternative is fieldless — a named set of integers.
    Enum(Enum),
    Opaque(Opaque),
}

impl Type {
    pub fn name(&self) -> &syn::Ident {
        match self {
            Type::Struct(s) => &s.name,
            Type::Variant(v) => &v.name,
            Type::Enum(e) => &e.name,
            Type::Opaque(o) => &o.name,
        }
    }

    pub fn location(&self) -> &SourceLocation {
        match self {
            Type::Struct(s) => &s.origin.location,
            Type::Variant(v) => &v.origin.location,
            Type::Enum(e) => &e.origin.location,
            Type::Opaque(o) => &o.origin.location,
        }
    }

    /// The whole item as the source wrote it.
    pub fn syntax(&self) -> syn::Item {
        match self {
            Type::Struct(s) => syn::Item::Struct(s.origin.syntax.clone()),
            Type::Variant(v) => syn::Item::Enum(v.origin.syntax.clone()),
            Type::Enum(e) => syn::Item::Enum(e.origin.syntax.clone()),
            Type::Opaque(o) => o.origin.syntax.clone(),
        }
    }
}

/// A type whose contents do not cross the boundary — a handle.
///
/// Two spellings declare one thing, because the model records the *fact* rather
/// than the Rust shape that carried it:
///
/// * `#[prebindgen] pub type X = path::To<Thing>;` — the way to give a foreign or
///   crate-private type a name in the flat API. This is how a handle is declared
///   deliberately.
/// * `#[prebindgen] pub struct X(..);` — a tuple struct, whose fields no adapter
///   has ever crossed.
///
/// Either way the adapter decides what the handle becomes: an opaque pointer, a
/// `ptr_class`, a `convert!` target.
#[derive(Clone, Debug)]
pub struct Opaque {
    pub name: syn::Ident,
    /// The declaring item — a type alias or a tuple struct.
    pub origin: Origin<syn::Item>,
}

/// A `#[prebindgen]` free function.
#[derive(Clone, Debug)]
pub struct Function {
    pub name: syn::Ident,
    /// Parameters in declaration order.
    pub params: Vec<Param>,
    /// What the function returns. An elided return is
    /// [`TypeKind::Unit`](super::TypeKind), exactly as a written `-> ()` is:
    /// they mean the same thing, differ only in spelling, and every consumer
    /// today already normalizes one to the other on the spot.
    pub ret: TypeRef,
    /// The whole item: attributes, `cfg`, doc comments, body.
    pub origin: Origin<syn::ItemFn>,
}

/// One parameter of a [`Function`].
#[derive(Clone, Debug)]
pub struct Param {
    pub name: syn::Ident,
    pub ty: TypeRef,
    /// The parameter as written — `mode: Mode`.
    pub origin: Origin<syn::PatType>,
}

/// A `#[prebindgen]` struct: a product of fields that cross the boundary.
///
/// A struct whose contents do *not* cross is an [`Opaque`], not a `Struct` with
/// nothing in it — so `fields` is a plain list, and empty means the source wrote
/// a struct with no fields.
///
/// Whether the fields are named or positional is not recorded: a [`Field`]
/// already knows its own address, and the delimiters are spelling, read off the
/// syntax by [`spell::fields`](super::spell::fields).
#[derive(Clone, Debug)]
pub struct Struct {
    pub name: syn::Ident,
    pub fields: Vec<Field>,
    pub origin: Origin<syn::ItemStruct>,
}

/// A `#[prebindgen]` enum whose alternatives carry payloads — a sum type.
///
/// Distinct from [`Enum`], which is the fieldless shape, because the two are
/// consumed as different constructs and **numbered differently**. A sum's
/// alternatives are identified by position: the mirror an adapter builds carries
/// no `repr` and numbers its own arms, so a Rust discriminant would be the wrong
/// answer here — which is why there is no slot for one.
///
/// Both shapes are spelled `enum` in Rust and both keep a `syn::ItemEnum` in
/// their origin. Which one an item *is* is the classification, and it is decided
/// once: any alternative with a field makes it a `Variant`.
#[derive(Clone, Debug)]
pub struct Variant {
    pub name: syn::Ident,
    /// Alternatives in declaration order; `alternatives[i].index == i`.
    pub alternatives: Vec<Alternative>,
    pub origin: Origin<syn::ItemEnum>,
}

/// One alternative of a [`Variant`].
#[derive(Clone, Debug)]
pub struct Alternative {
    pub name: syn::Ident,
    /// Position within its sum, `0..N-1` — the same fact a [`Field`] carries,
    /// for the same reason: a node handed out on its own still knows where it
    /// sits.
    ///
    /// This is the *only* numbering a sum has. What a destination language does
    /// with it is its own business: one may transmit it to say which alternative
    /// is live, another may send a name instead.
    pub index: usize,
    /// The alternative's payload, in declaration order. May be empty — a sum can
    /// mix payload-carrying and payload-free alternatives, and only the presence
    /// of *some* payload makes the type a `Variant`.
    pub fields: Vec<Field>,
    /// The alternative as written: delimiters, attributes, doc comments.
    pub origin: Origin<syn::Variant>,
}

impl Alternative {
    /// True when this alternative carries no payload.
    ///
    /// The *group* question, not the syntax one: `B`, `B()` and `B {}` are all
    /// empty by this test, and [`spell::fields`](super::spell::fields) is what
    /// keeps their delimiters apart.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// A `#[prebindgen]` enum whose every alternative is fieldless — the C-style
/// shape, a named set of integers.
///
/// Distinct from [`Variant`] because the identity of a member here is the value
/// Rust **assigns** it, not where it sits: a C header re-states each `= expr`
/// and a Kotlin `enum class` entry is `NAME(7)`. A sum has no such value, which
/// is why the two are separate entities rather than one with a dead field each.
#[derive(Clone, Debug)]
pub struct Enum {
    pub name: syn::Ident,
    /// Values in declaration order; `values[i].index == i`.
    pub values: Vec<EnumValue>,
    pub origin: Origin<syn::ItemEnum>,
}

impl Enum {
    /// Every value paired with the number Rust assigns it, or the first value
    /// whose discriminant could not be evaluated.
    ///
    /// This is the numbering a destination language needs when it has no way to
    /// reference a Rust constant: a Kotlin `enum class` entry is `NAME(3)`, and
    /// the generated `int → value` decode matches on the same numbers, so both
    /// come from here and cannot drift. An `Err` is a refusal for *that*
    /// consumer only — one that re-emits the source spelling never asks.
    pub fn discriminant_values(&self) -> Result<Vec<(&syn::Ident, i64)>, &syn::Ident> {
        self.values
            .iter()
            .map(|v| match v.discriminant {
                Some(n) => Ok((&v.name, n)),
                None => Err(&v.name),
            })
            .collect()
    }
}

/// One named value of an [`Enum`].
#[derive(Clone, Debug)]
pub struct EnumValue {
    pub name: syn::Ident,
    /// Position within its enum, `0..N-1`. Not the identity — see
    /// [`Self::discriminant`] — but the same "where it sits" fact every node in
    /// an ordered list carries, and what a consumer falls back to when a
    /// discriminant cannot be evaluated.
    pub index: usize,
    /// The value Rust assigns — an explicit `= N` sets it, an implicit value
    /// takes the previous plus one, starting at 0. **This shape's identity.**
    ///
    /// `None` once a spelling the frontend cannot evaluate (a `const`, a `cfg`,
    /// arithmetic) has broken the chain, or once the chain has run out of `i64`.
    /// That is not a failure: only a consumer that needs the *number* is
    /// affected, and one that re-emits the *spelling* reads
    /// [`Self::origin`]`.syntax.discriminant` instead.
    pub discriminant: Option<i64>,
    /// The value as written: `= 0x07`, attributes, doc comments — and its
    /// delimiters, since `B` and `B()` are both fieldless and still spelled
    /// differently.
    pub origin: Origin<syn::Variant>,
}

/// One field of a [`Struct`] or of an [`Alternative`].
#[derive(Clone, Debug)]
pub struct Field {
    /// The field's name, or `None` for a positional one.
    pub name: Option<syn::Ident>,
    /// Position within its struct or alternative, `0..N-1` — the same fact an
    /// [`Alternative`] carries.
    ///
    /// The address of a positional field. A named field has one too, and simply
    /// does not need it: it is addressed by name, so this is available rather
    /// than used — the same way it carries its item's location.
    pub index: usize,
    pub ty: TypeRef,
    /// The field as written — `pub id: u64`, attributes and docs included.
    pub origin: Origin<syn::Field>,
}

/// A `#[prebindgen]` constant.
///
/// Also the home of the unnamed `const _` feature guard each source injects: it
/// is a constant, so it is modelled as one, and [`Element::name`] returning
/// `None` for `_` is what keeps several of them from colliding in the flat
/// namespace.
#[derive(Clone, Debug)]
pub struct Constant {
    pub name: syn::Ident,
    pub ty: TypeRef,
    /// The whole item — the initializer expression included, which is where a
    /// consumer that re-emits the value reads it from.
    pub origin: Origin<syn::ItemConst>,
}

/// An item the language cannot express.
#[derive(Clone, Debug)]
pub struct Unsupported {
    /// The item's identifier, or `None` for an item kind that has none.
    pub name: Option<syn::Ident>,
    /// What could not be expressed, ready to be raised by whatever declares
    /// this item. Boxed: it is the size outlier among the elements, and this
    /// one is the rare variant.
    pub error: Box<super::ItemError>,
    /// The item as written, so a diagnosis can quote the source.
    pub origin: Origin<syn::Item>,
}
