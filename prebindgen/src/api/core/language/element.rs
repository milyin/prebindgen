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

use super::{origin::Origin, ty::Type};
use crate::SourceLocation;

/// One structure of the prebindgen source language.
///
/// The four modelled kinds, plus [`Element::Unsupported`] for anything the
/// language cannot express. There is no verbatim-passthrough variant: a
/// `#[prebindgen]` crate marks the items that cross the boundary, and the
/// supporting code around them is the consumer crate's job — the proc-macro
/// enforces that already, refusing to mark a `use`, `mod`, `impl` or
/// `macro_rules!` at all.
#[derive(Clone, Debug)]
pub enum Element {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Const(Const),
    /// An item the language cannot express — a parameter type outside the
    /// grammar, a `self` receiver, or a whole item kind it does not model such
    /// as a `union`.
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
            Element::Struct(s) => Some(&s.name),
            Element::Enum(e) => Some(&e.name),
            Element::Const(c) => Some(&c.name),
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
            Element::Struct(s) => &s.origin.location,
            Element::Enum(e) => &e.origin.location,
            Element::Const(c) => &c.origin.location,
            Element::Unsupported(u) => &u.origin.location,
        }
    }

    /// The whole item as the source wrote it.
    pub fn syntax(&self) -> syn::Item {
        match self {
            Element::Function(f) => syn::Item::Fn(f.origin.syntax.clone()),
            Element::Struct(s) => syn::Item::Struct(s.origin.syntax.clone()),
            Element::Enum(e) => syn::Item::Enum(e.origin.syntax.clone()),
            Element::Const(c) => syn::Item::Const(c.origin.syntax.clone()),
            Element::Unsupported(u) => u.origin.syntax.clone(),
        }
    }
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
    pub ret: Type,
    /// The whole item: attributes, `cfg`, doc comments, body.
    pub origin: Origin<syn::ItemFn>,
}

/// One parameter of a [`Function`].
#[derive(Clone, Debug)]
pub struct Param {
    pub name: syn::Ident,
    pub ty: Type,
    /// The parameter as written — `mode: Mode`.
    pub origin: Origin<syn::PatType>,
}

/// A `#[prebindgen]` struct: a product of fields, or an opaque one.
#[derive(Clone, Debug)]
pub struct Struct {
    pub name: syn::Ident,
    /// The fields, when they are a boundary surface — `Some(vec![])` for a
    /// struct with none.
    ///
    /// `None` means **opaque**: the contents are not part of the boundary and
    /// are deliberately not lowered, so a field type outside the grammar is not
    /// an error. That is today's tuple struct — usable as a handle, its fields
    /// never crossed by any adapter — and lowering them would turn types that
    /// are ignored now into refusals.
    ///
    /// Whether a shape has named or positional fields is not recorded here: a
    /// [`Field`] already knows its own address, and the delimiters are
    /// spelling, read off `syntax` by
    /// [`spell::fields`](super::spell::fields).
    pub fields: Option<Vec<Field>>,
    pub origin: Origin<syn::ItemStruct>,
}

impl Struct {
    /// The modelled fields — empty when the struct is opaque.
    pub fn fields(&self) -> &[Field] {
        self.fields.as_deref().unwrap_or(&[])
    }
}

/// A `#[prebindgen]` enum: a choice of one alternative, each with its own field
/// group.
///
/// One model covers both enum shapes on purpose. A fieldless enum is the
/// degenerate sum whose every group is empty, so a lowering written for the
/// general case collapses to "just the selector" for it.
#[derive(Clone, Debug)]
pub struct Enum {
    pub name: syn::Ident,
    /// Variants in declaration order; `variants[i].index == i`.
    pub variants: Vec<Variant>,
    pub origin: Origin<syn::ItemEnum>,
}

impl Enum {
    /// True when every variant is fieldless, i.e. the value is exactly its
    /// discriminant.
    pub fn is_unit(&self) -> bool {
        self.variants.iter().all(Variant::is_unit)
    }

    /// The first payload-carrying variant — the offender an adapter names when
    /// refusing a sum where only a fieldless enum is accepted.
    pub fn first_payload_variant(&self) -> Option<&Variant> {
        self.variants.iter().find(|v| !v.is_unit())
    }

    /// Every variant paired with the value Rust assigns it, or the first
    /// variant whose discriminant could not be evaluated.
    ///
    /// This is the numbering a destination language needs when it has no way to
    /// reference a Rust constant: a Kotlin `enum class` entry is `NAME(3)`, and
    /// the generated `int → variant` decode matches on the same numbers, so both
    /// come from here and cannot drift. An `Err` is a refusal for *that*
    /// consumer only — one that re-emits the source spelling never asks.
    pub fn discriminant_values(&self) -> Result<Vec<(&syn::Ident, i64)>, &syn::Ident> {
        self.variants
            .iter()
            .map(|v| match v.discriminant {
                Some(n) => Ok((&v.name, n)),
                None => Err(&v.name),
            })
            .collect()
    }
}

/// One alternative of an [`Enum`].
#[derive(Clone, Debug)]
pub struct Variant {
    pub name: syn::Ident,
    /// Position within its enum, `0..N-1` — the same fact a [`Field`] carries,
    /// for the same reason: a node handed out on its own still knows where it
    /// sits.
    ///
    /// This is **not** the discriminant. A position is what the source *wrote*;
    /// a discriminant is the value Rust *assigns*, and the two are independent
    /// — see [`Self::discriminant`].
    ///
    /// What a destination language does with the position is its own business:
    /// one may transmit it to say which alternative is live, another may send a
    /// name instead. The language says only where the variant sits.
    pub index: usize,
    /// The value Rust assigns this variant — an explicit `= N` sets it, an
    /// implicit variant takes the previous value plus one, starting at 0.
    ///
    /// `None` once a spelling the frontend cannot evaluate (a `const`, a `cfg`,
    /// arithmetic) has broken the chain, or once the chain has run out of `i64`.
    /// That is not a failure: only a consumer that needs the *number* is
    /// affected, and one that re-emits the *spelling* reads
    /// [`Self::origin`]`.syntax.discriminant` instead.
    pub discriminant: Option<i64>,
    /// The variant's payload, in declaration order.
    pub fields: Vec<Field>,
    /// The variant as written: delimiters, `= 0x07`, attributes, doc comments.
    pub origin: Origin<syn::Variant>,
}

/// One field of a [`Struct`] or of a [`Variant`].
#[derive(Clone, Debug)]
pub struct Field {
    /// The field's name, or `None` for a positional one.
    pub name: Option<syn::Ident>,
    /// Position within its struct or variant, `0..N-1` — the same fact a
    /// [`Variant`] carries.
    ///
    /// The address of a positional field. A named field has one too, and simply
    /// does not need it: it is addressed by name, so this is available rather
    /// than used — the same way it carries its item's location.
    pub index: usize,
    pub ty: Type,
    /// The field as written — `pub id: u64`, attributes and docs included.
    pub origin: Origin<syn::Field>,
}

impl Variant {
    /// True when this variant carries no payload.
    ///
    /// The *group* question, not the syntax one: `B`, `B()` and `B {}` are all
    /// unit by this test, and
    /// [`spell::fields`](super::spell::fields) is what keeps their delimiters
    /// apart.
    pub fn is_unit(&self) -> bool {
        self.fields.is_empty()
    }
}

/// A `#[prebindgen]` const.
///
/// Also the home of the unnamed `const _` feature guard each source injects: it
/// is a const, so it is modelled as one, and [`Element::name`] returning `None`
/// for `_` is what keeps several of them from colliding in the flat namespace.
#[derive(Clone, Debug)]
pub struct Const {
    pub name: syn::Ident,
    pub ty: Type,
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
