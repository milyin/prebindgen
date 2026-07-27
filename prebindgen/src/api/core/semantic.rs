//! Tier 0 — the language-neutral **semantic shape** of a boundary value.
//!
//! One interned graph describing what the *source Rust type says*, shared by
//! every adapter. Stage 1 (`lang::Cbindgen`) and Stage 3 (`lang::JniGen`) both
//! build their own Tier 1 plans **over** this tier; without a single owner each
//! would grow a private copy of the same traversal, which is the failure mode
//! [#187](https://github.com/milyin/prebindgen/issues/187) exists to prevent.
//!
//! # What this tier may say
//!
//! Structure and use. A struct is a [`SemanticShape::Product`], an enum is a
//! [`SemanticShape::Choice`], `Option` and the sequence types are layers, and
//! **every child edge is use-qualified** — [`SemanticUse`] records whether the
//! parent holds that child by value, by shared reference, or by exclusive
//! reference.
//!
//! That last part is the reason a root-only crossing context is not enough:
//! `Vec<&T>` is a container held by value whose *elements* are shared-borrowed,
//! and there is nowhere to write that down without an edge qualifier.
//!
//! # What this tier may not say
//!
//! > Tier 0 may not name a JVM descriptor, a C ABI type, a Kotlin class, or a
//! > delivery protocol.
//!
//! [`SourceUse`] records what the source type says; what that obliges either
//! side to *release* is a Tier 1 decision. The module-boundary test in
//! `semantic/tests.rs` checks this mechanically against this file's own text,
//! because Rust's module system cannot express "no adapter concept is
//! reachable from here".
//!
//! Two consequences worth stating, since both look like omissions:
//!
//! - **`Box<T>` is transparent.** A `Box` is heap indirection, which is a
//!   representation fact — whether it becomes a pointer on the wire is Tier 1's
//!   call. It is also what makes `Option<Box<Node>>` a legal recursion rather
//!   than an infinitely-sized type.
//! - **`Result<T, E>` is an ordinary [`Choice`](SemanticShape::Choice).** Both
//!   adapters route it to an error channel, but *that* is the Tier 1 decision;
//!   at the source it is exactly a two-alternative sum, and modelling it as an
//!   opaque leaf would hide `T` and `E` from the tier whose job is structure.
//!
//! # What is deliberately *not* decomposed
//!
//! A [`Leaf`](SemanticShape::Leaf) says "this tier does not decompose it", and
//! two cases take that answer on purpose, because the alternative is a node
//! that looks decomposed and is wrong:
//!
//! - **A type-or-const-generic declared item** ([`generic_over_types`]).
//!   `struct Wrap<T> { value: T }` interned as `Wrap<u8>` would otherwise be a
//!   product keyed `Wrap<u8>` whose field is still the *parameter* `T`.
//!   Instantiating the root's arguments is the other half of the answer and
//!   lands the day an adapter needs it.
//! - **A foreign-qualified path** ([`source_item_ident`]). Ingest normalizes
//!   every captured source spelling to a bare ident, so anything still
//!   qualified is not the registry's own item — matching on the tail would give
//!   `foreign::Rec` the local `Rec`'s fields under the key `foreign::Rec`.
//!
//! # Cycles are represented, never cut
//!
//! `Node { children: Vec<Node> }` interns to a finite graph with a back-edge,
//! because interning happens **before** a node's children are built. Where a
//! cycle may be cut — and what that costs on the wire — is Tier 1 policy, so
//! this tier offers no stopping-rule hook; a hook here would be the same policy
//! leak the tier boundary exists to prevent. [`ShapeGraph::is_recursive`] is a
//! *query* over the finished graph, not a knob.

// This tier lands before either adapter reads it: Stage 1 (#192) and Stage 3
// (#193) are its consumers, and #187's whole point is that neither of them owns
// it. So the graph is exercised by its own tests and nothing else yet — the
// same gap `SumSpec` carries for the same reason, and it goes away with the
// first adapter that plans over a shape.
#![allow(dead_code)]

use std::collections::HashMap;

use super::{
    registry::{Registry, TypeKey},
    types_util::{EnumShape, SumSpec},
};

/// Interned identity of a node in a [`ShapeGraph`].
///
/// Ids are only meaningful within the graph that issued them. Two graphs may
/// hand out the same numeric id for unrelated shapes, so an id is never
/// compared across graphs — the accessors all take `&self` for that reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShapeId(usize);

impl ShapeId {
    /// The raw index, for diagnostics and test assertions.
    pub fn index(self) -> usize {
        self.0
    }
}

/// How a parent holds a child — the edge qualifier that makes `Vec<&T>`
/// expressible.
///
/// This records the **source** relationship only. That a `SharedRef` element
/// cannot be handed to a consuming converter, or that a `Value` element obliges
/// the receiver to release it, are Tier 1 conclusions drawn *from* this, not
/// facts stored here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceUse {
    /// Held by value — `T`.
    Value,
    /// Shared borrow — `&T`.
    SharedRef,
    /// Exclusive borrow — `&mut T`.
    ExclusiveRef,
}

/// The declared length of a [`SequenceKind::Array`], as the source spells it.
///
/// Kept in the shape rather than left for a consumer to re-read off the type,
/// because the length is *needed*: packing a small fixed array into scalar
/// slots ([#208](https://github.com/milyin/prebindgen/issues/208)) is a Tier 1
/// decision that cannot be taken without it.
///
/// Split by what a consumer can actually do with it. A literal is actionable
/// immediately; a named const has to be resolved against the registry first —
/// `[u8; ZENOH_ID_MAX_SIZE]` is the real spelling in zenoh-flat, so "the length
/// is a literal" is not an assumption this tier may make.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArrayLen {
    /// `[u8; 16]`.
    Literal(usize),
    /// `[u8; ZENOH_ID_MAX_SIZE]` — a const the registry may index.
    Named(syn::Ident),
    /// Anything else (an arithmetic expression, a generic const parameter),
    /// recorded verbatim so nothing is silently dropped.
    Other(String),
}

/// Which sequence spelling a [`SemanticShape::Sequence`] layer came from.
///
/// Kept distinct because they differ in what the *source* says about ownership
/// of the backing storage — and, for [`Array`](Self::Array), about its length —
/// which is an input to Tier 1's decision even though the decision itself is
/// not made here.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SequenceKind {
    /// `Vec<T>` — owned, growable.
    Vec,
    /// `[T]`, reached through `&[T]` — borrowed storage.
    Slice,
    /// `Cow<'_, [T]>` — borrowed or owned, decided at runtime.
    CowSlice,
    /// `[T; N]` — owned, inline, statically sized.
    ///
    /// Added after the fact: Stage T's spec enumerated only the three
    /// unbounded spellings, so a fixed array fell through to a
    /// [`Leaf`](SemanticShape::Leaf) — which hid its element type from the tier
    /// whose job is structure, the same objection that put `Result` in as a
    /// `Choice` rather than an opaque leaf.
    ///
    /// It was invisible while nothing crossed the boundary as an array;
    /// [#209](https://github.com/milyin/prebindgen/pull/209) makes `[T; N]` a
    /// first-class boundary shape (a Kotlin primitive array), so the gap became
    /// load-bearing.
    Array(ArrayLen),
}

/// A use-qualified edge to a child shape.
///
/// Every child edge in this tier is one of these; there is no unqualified
/// `ShapeId` reference in a node, so a traversal cannot silently lose the
/// distinction between `Vec<T>` and `Vec<&T>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticUse {
    /// The child node.
    pub shape: ShapeId,
    /// How the parent holds it.
    pub source: SourceUse,
}

/// One field of a [`Product`](SemanticShape::Product), or one payload slot of a
/// [`VariantUse`].
#[derive(Clone, Debug)]
pub struct FieldUse {
    /// How the field is *addressed*: `Named(ident)` for a record field,
    /// `Unnamed(index)` for a tuple slot.
    ///
    /// Retained as a [`syn::Member`] rather than degraded to a string on
    /// purpose. A `Member` distinguishes `.0` from `."0"` and survives
    /// round-tripping into generated code; rebuilding an access from a string
    /// is [#186](https://github.com/milyin/prebindgen/issues/186)'s textual
    /// problem one tier lower, and the point of doing Tier 0 at all is not to
    /// recreate it.
    pub member: syn::Member,
    /// A human-readable name for diagnostics and for adapters that need a
    /// flattened leaf name. **Not** an identity — two fields may share one, and
    /// nothing may key on it.
    pub diagnostic_name: String,
    /// The field's shape and how this parent holds it.
    pub uses: SemanticUse,
}

/// One alternative of a [`Choice`](SemanticShape::Choice).
#[derive(Clone, Debug)]
pub struct VariantUse {
    /// The variant ident **as declared in the source**, independent of any
    /// destination-language naming. An adapter that renames a variant records
    /// that in its own tier; this stays the Rust spelling, so a construction
    /// path built from it is always valid.
    pub ident: syn::Ident,
    /// Declaration-order tag, `0..N-1`. A payload enum's alternatives are
    /// identified by this, never by a `#[repr]` discriminant.
    pub tag: i32,
    /// The alternative's payload in declaration order. **Empty for a unit
    /// variant** — the group that contributes nothing but its tag.
    pub fields: Vec<FieldUse>,
}

/// The language-neutral shape of one type.
///
/// Interned in a [`ShapeGraph`]; children are reached through [`SemanticUse`]
/// edges rather than by containment, so a recursive type is a finite graph.
#[derive(Clone, Debug)]
pub enum SemanticShape {
    /// A type with no structure this tier decomposes: a scalar, a `String`, a
    /// callback, or any type the registry did not index as a struct or enum.
    ///
    /// "Leaf" is a statement about *this graph's* knowledge, not about the
    /// type — an opaque handle and an `i32` are both leaves here, and what
    /// separates them is a Tier 1 declaration.
    Leaf(TypeKey),
    /// A struct: all of these fields, together.
    Product {
        /// Canonical key of the struct type.
        key: TypeKey,
        /// Fields in declaration order.
        fields: Vec<FieldUse>,
    },
    /// An enum: exactly one of these alternatives.
    ///
    /// **Every enum is a `Choice`, unit-only included** — a unit enum is the
    /// degenerate sum whose every variant group is empty, which is exactly the
    /// tag-only lowering. An adapter may collapse an all-empty choice to a
    /// tag-only leaf; this tier does not, because "is it a sum" and "does it
    /// carry payloads" are different questions and only the second is
    /// [`EnumShape`]'s.
    Choice {
        /// Canonical key of the enum type.
        key: TypeKey,
        /// Alternatives in declaration order.
        variants: Vec<VariantUse>,
    },
    /// An `Option<…>` layer.
    Optional(SemanticUse),
    /// A `Vec<…>` / `[…]` / `Cow<'_, […]>` layer.
    Sequence {
        /// Which spelling it came from.
        kind: SequenceKind,
        /// The element edge — this is where `Vec<&T>` records that its elements
        /// are shared-borrowed while the container is held by value.
        elem: SemanticUse,
    },
}

impl SemanticShape {
    /// Every child edge, in a single uniform accessor, so a traversal cannot
    /// forget a variant.
    pub fn children(&self) -> Vec<SemanticUse> {
        match self {
            SemanticShape::Leaf(_) => Vec::new(),
            SemanticShape::Product { fields, .. } => fields.iter().map(|f| f.uses).collect(),
            SemanticShape::Choice { variants, .. } => variants
                .iter()
                .flat_map(|v| v.fields.iter().map(|f| f.uses))
                .collect(),
            SemanticShape::Optional(u) => vec![*u],
            SemanticShape::Sequence { elem, .. } => vec![*elem],
        }
    }

    /// [`EnumShape`] as a **classifier over** a `Choice` — not a second
    /// modelling of one. `None` for anything that is not a choice.
    ///
    /// The distinction exists because the *declarators* differ: `enum_class!` /
    /// `.enum_type()` accept only the degenerate case, and handing them a
    /// payload enum is an error naming the sum declarator. It is not a second
    /// shape.
    pub fn enum_shape(&self) -> Option<EnumShape> {
        match self {
            SemanticShape::Choice { variants, .. } => {
                Some(if variants.iter().all(|v| v.fields.is_empty()) {
                    EnumShape::Unit
                } else {
                    EnumShape::Sum
                })
            }
            _ => None,
        }
    }
}

/// An interned graph of [`SemanticShape`]s.
///
/// Interning is keyed by the **full [`TypeKey`]**, never by a bare identifier.
/// That is the structural fix for
/// [#136](https://github.com/milyin/prebindgen/issues/136): a bare-ident cycle
/// stack conflates two distinct types that happen to share a short name, and no
/// amount of care at the use site recovers the difference once the key has
/// thrown it away.
#[derive(Debug, Default)]
pub struct ShapeGraph {
    nodes: Vec<SemanticShape>,
    by_key: HashMap<TypeKey, ShapeId>,
    /// Reverse index, so a layer node can report the spelling it came from.
    keys: Vec<TypeKey>,
}

impl ShapeGraph {
    /// An empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern the shape of `ty` **as a root**, returning its use-qualified
    /// edge. A root is use-qualified like any other edge: a `&T` parameter is a
    /// shared-ref use of `T`, not a distinct shape.
    pub fn intern_root<M>(&mut self, ty: &syn::Type, registry: &Registry<M>) -> SemanticUse {
        self.intern_use(ty, registry)
    }

    /// The node behind an id.
    pub fn get(&self, id: ShapeId) -> &SemanticShape {
        &self.nodes[id.0]
    }

    /// The key a node was interned under.
    pub fn key(&self, id: ShapeId) -> &TypeKey {
        &self.keys[id.0]
    }

    /// The id already interned for `key`, if any.
    pub fn id_of(&self, key: &TypeKey) -> Option<ShapeId> {
        self.by_key.get(key).copied()
    }

    /// Number of interned nodes — the finiteness a recursion test asserts.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Every interned id, in interning order.
    pub fn ids(&self) -> impl Iterator<Item = ShapeId> {
        (0..self.nodes.len()).map(ShapeId)
    }

    /// Project the `Optional` / `Sequence` layers above `id` onto the existing
    /// [`Shape`](super::shape::Shape) stack, stopping at the first node that is
    /// not a layer.
    ///
    /// This is the answer to "does `Shape<N>` survive Tier 0": **it survives as
    /// a derived view, not as a second model.** `Shape<N>`'s `N` is the
    /// null-representation choice, which is a wire fact and belongs to Tier 1;
    /// the *layer stack itself* is semantic and is now owned here. So the
    /// engines that fold over layers keep their algebra and stop deriving the
    /// stack themselves.
    pub fn wrapper_projection(&self, id: ShapeId) -> super::shape::Shape {
        match self.get(id) {
            SemanticShape::Optional(inner) => {
                super::shape::Shape::optional((), self.wrapper_projection(inner.shape))
            }
            SemanticShape::Sequence { elem, .. } => {
                super::shape::Shape::iterable(self.wrapper_projection(elem.shape))
            }
            _ => super::shape::Shape::Base,
        }
    }

    /// The node the layer stack above `id` bottoms out at — the leaf, product
    /// or choice [`Self::wrapper_projection`] stopped at.
    pub fn base_of(&self, id: ShapeId) -> ShapeId {
        match self.get(id) {
            SemanticShape::Optional(inner) => self.base_of(inner.shape),
            SemanticShape::Sequence { elem, .. } => self.base_of(elem.shape),
            _ => id,
        }
    }

    /// Whether `id` is reachable from itself — i.e. it participates in a cycle.
    ///
    /// A **query** over the finished graph, not a stopping rule. Which cycles
    /// may be cut, and how, is Tier 1 policy; offering that decision here would
    /// be the policy leak this tier exists to prevent.
    pub fn is_recursive(&self, id: ShapeId) -> bool {
        let mut seen = vec![false; self.nodes.len()];
        let mut stack: Vec<ShapeId> = self.get(id).children().iter().map(|u| u.shape).collect();
        while let Some(next) = stack.pop() {
            if next == id {
                return true;
            }
            if std::mem::replace(&mut seen[next.0], true) {
                continue;
            }
            stack.extend(self.get(next).children().iter().map(|u| u.shape));
        }
        false
    }

    /// Peel the source's use qualifier off `ty` and intern what remains.
    fn intern_use<M>(&mut self, ty: &syn::Type, registry: &Registry<M>) -> SemanticUse {
        // Transparent wrappers are stripped on **both** sides of the peel, not
        // just one:
        //
        //   `&Box<T>`  needs the strip AFTER  — peel yields `Box<T>`;
        //   `Box<&T>`  needs the strip BEFORE — otherwise the `&` is still
        //              there when the key is formed, and the reference ends up
        //              *inside* the interned node instead of on the edge.
        //
        // That second case is what makes the order load-bearing rather than
        // cosmetic: a node keyed `& T` contradicts this tier's own rule that a
        // use qualifier lives on the edge, and `build` would classify it as a
        // `Leaf` — silently losing the structure of a declared `T`.
        let outer = strip_transparent(ty);
        let (source, inner) = peel_source_use(&outer);
        let inner = strip_transparent(&inner);
        SemanticUse {
            shape: self.intern(&inner, registry),
            source,
        }
    }

    /// Intern `ty`'s shape, reusing an existing node when its key is already
    /// present.
    ///
    /// The id is reserved **before** the children are built, which is what
    /// makes a recursive type terminate: the recursive occurrence finds the
    /// reserved id in `by_key` and becomes a back-edge instead of recursing
    /// forever.
    /// `ty` must already be normalized by [`Self::intern_use`] — no use
    /// qualifier and no transparent wrapper — so an interning key can never
    /// contain a reference.
    fn intern<M>(&mut self, ty: &syn::Type, registry: &Registry<M>) -> ShapeId {
        debug_assert!(
            !matches!(ty, syn::Type::Reference(_)),
            "intern received an unpeeled reference `{}` — the use qualifier belongs on the edge",
            quote::ToTokens::to_token_stream(ty)
        );
        let key = TypeKey::from_type(ty);
        if let Some(id) = self.by_key.get(&key) {
            return *id;
        }
        let id = ShapeId(self.nodes.len());
        // Placeholder, so the key is claimed before `build` can recurse into
        // it. Overwritten unconditionally below.
        self.nodes.push(SemanticShape::Leaf(key.clone()));
        self.keys.push(key.clone());
        self.by_key.insert(key.clone(), id);

        let built = self.build(ty, &key, registry);
        self.nodes[id.0] = built;
        id
    }

    /// Classify one type into a node, interning its children.
    fn build<M>(&mut self, ty: &syn::Type, key: &TypeKey, registry: &Registry<M>) -> SemanticShape {
        use super::types_util::{first_type_arg, is_option_type, is_vec_type, result_parts};

        if is_option_type(ty) {
            if let Some(inner) = first_type_arg(ty) {
                return SemanticShape::Optional(self.intern_use(&inner, registry));
            }
        }
        if is_vec_type(ty) {
            if let Some(inner) = first_type_arg(ty) {
                return SemanticShape::Sequence {
                    kind: SequenceKind::Vec,
                    elem: self.intern_use(&inner, registry),
                };
            }
        }
        if let syn::Type::Slice(s) = ty {
            return SemanticShape::Sequence {
                kind: SequenceKind::Slice,
                elem: self.intern_use(&s.elem, registry),
            };
        }
        if let syn::Type::Array(a) = ty {
            return SemanticShape::Sequence {
                kind: SequenceKind::Array(array_len(&a.len)),
                elem: self.intern_use(&a.elem, registry),
            };
        }
        if let Some(inner) = cow_slice_elem(ty) {
            return SemanticShape::Sequence {
                kind: SequenceKind::CowSlice,
                elem: self.intern_use(&inner, registry),
            };
        }
        // `Result<T, E>` is a two-alternative sum at the source. Whether it
        // becomes an error channel is Tier 1's decision, not this tier's.
        if let Some((ok, err)) = result_parts(ty) {
            return SemanticShape::Choice {
                key: key.clone(),
                variants: vec![
                    self.synthetic_variant("Ok", 0, &ok, registry),
                    self.synthetic_variant("Err", 1, &err, registry),
                ],
            };
        }

        // A declared struct or enum — but only through a **canonical source
        // path**. See `source_item_ident`: matching on the tail ident alone
        // would give `foreign::Rec` the local `Rec`'s fields while keeping
        // `foreign::Rec` as its key, which contradicts both the full-`TypeKey`
        // rule and `normalize_type`'s own statement that an unknown crate path
        // is never reduced because its spelling is its identity.
        let Some(ident) = source_item_ident(ty) else {
            return SemanticShape::Leaf(key.clone());
        };
        if let Some((item, _)) = registry.structs.get(&ident) {
            if generic_over_types(&item.generics) {
                return SemanticShape::Leaf(key.clone());
            }
            let item = item.clone();
            return SemanticShape::Product {
                key: key.clone(),
                fields: self.product_fields(&item, registry),
            };
        }
        if let Some((item, _)) = registry.enums.get(&ident) {
            if generic_over_types(&item.generics) {
                return SemanticShape::Leaf(key.clone());
            }
            let item = item.clone();
            return SemanticShape::Choice {
                key: key.clone(),
                variants: self.choice_variants(&item, registry),
            };
        }
        SemanticShape::Leaf(key.clone())
    }

    /// The fields of a declared struct, in declaration order.
    fn product_fields<M>(
        &mut self,
        item: &syn::ItemStruct,
        registry: &Registry<M>,
    ) -> Vec<FieldUse> {
        item.fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let (member, diagnostic_name) = match &f.ident {
                    Some(id) => (syn::Member::Named(id.clone()), id.to_string()),
                    None => (syn::Member::Unnamed(syn::Index::from(i)), i.to_string()),
                };
                FieldUse {
                    member,
                    diagnostic_name,
                    uses: self.intern_use(&f.ty, registry),
                }
            })
            .collect()
    }

    /// The alternatives of a declared enum.
    ///
    /// Built **through [`SumSpec`]**, which is this tier's `Choice`
    /// constructor rather than a parallel description of the same enum: it
    /// already assigns declaration-order tags, retains `syn::Member`, and
    /// derives the nested-prefix leaf names, and having two places compute
    /// those is the duplication Tier 0 exists to remove.
    fn choice_variants<M>(
        &mut self,
        item: &syn::ItemEnum,
        registry: &Registry<M>,
    ) -> Vec<VariantUse> {
        let spec = SumSpec::from_item_enum(item);
        spec.variants
            .iter()
            .map(|v| VariantUse {
                ident: v.ident.clone(),
                tag: v.tag,
                fields: v
                    .fields
                    .iter()
                    .map(|f| FieldUse {
                        member: f.member.clone(),
                        diagnostic_name: f.name.clone(),
                        uses: self.intern_use(&f.ty, registry),
                    })
                    .collect(),
            })
            .collect()
    }

    /// One alternative of a sum that has no `syn::ItemEnum` behind it (today,
    /// `Result`'s two arms), carrying a single unnamed payload.
    fn synthetic_variant<M>(
        &mut self,
        ident: &str,
        tag: i32,
        payload: &syn::Type,
        registry: &Registry<M>,
    ) -> VariantUse {
        let uses = self.intern_use(payload, registry);
        VariantUse {
            ident: syn::Ident::new(ident, proc_macro2::Span::call_site()),
            tag,
            // A unit payload carries nothing, so the alternative is a unit
            // variant — the same shape a fieldless declared variant has.
            fields: if super::types_util::is_unit(payload) {
                Vec::new()
            } else {
                vec![FieldUse {
                    member: syn::Member::Unnamed(syn::Index::from(0usize)),
                    diagnostic_name: super::types_util::pascal_to_snake(ident),
                    uses,
                }]
            },
        }
    }
}

/// The ident of a **canonical source-item path**, or `None` for anything that
/// is not one.
///
/// The registry indexes source items by bare ident, and ingest normalizes every
/// captured spelling to that form (`crate::Foo`, `myflat::Foo` → `Foo`). So a
/// path that is still qualified after normalization denotes something the
/// registry does *not* own — `normalize_type` says exactly this about unknown
/// crate paths: they are never reduced, because `a::KeyExpr` and `b::KeyExpr`
/// may be genuinely distinct types and their spelling is their identity.
///
/// Matching on the tail ident instead would hand `foreign::Rec` the local
/// `Rec`'s fields while keeping `foreign::Rec` as the node's key — a product
/// assembled from an unrelated item, and the full-`TypeKey` invariant broken
/// from the other direction.
///
/// Lifetime arguments are allowed (`Ref<'a>` is still `Ref`): a lifetime cannot
/// appear in a field's *type structure*, so decomposition stays sound. Type and
/// const arguments are not — see [`generic_over_types`].
fn source_item_ident(ty: &syn::Type) -> Option<syn::Ident> {
    let syn::Type::Path(tp) = ty else { return None };
    if tp.qself.is_some() || tp.path.leading_colon.is_some() || tp.path.segments.len() != 1 {
        return None;
    }
    let seg = tp.path.segments.first()?;
    match &seg.arguments {
        syn::PathArguments::None => Some(seg.ident.clone()),
        syn::PathArguments::AngleBracketed(ab) => ab
            .args
            .iter()
            .all(|a| matches!(a, syn::GenericArgument::Lifetime(_)))
            .then(|| seg.ident.clone()),
        syn::PathArguments::Parenthesized(_) => None,
    }
}

/// Whether a declared item is parameterized by **types or consts** — in which
/// case this tier refuses to decompose it.
///
/// `struct Wrap<T> { value: T }` interned as `Wrap<u8>` would otherwise become a
/// product keyed `Wrap<u8>` whose field is still the *parameter* `T`, not `u8`:
/// a node that looks decomposed and is wrong, which is worse than one that
/// admits it does not know. Recursive generic types compound it, since the
/// back-edge would point at the uninstantiated body rather than the concrete
/// root.
///
/// Substituting the root's arguments is the other half of the answer and is
/// deliberately **not** attempted here: doing it properly means defaults,
/// const generics, partial application and nested parameter references, and a
/// half-done substitution reintroduces exactly the silently-wrong node this
/// guard exists to prevent. A [`Leaf`](SemanticShape::Leaf) is the honest
/// answer — "this tier does not decompose it" — and an adapter that meets one
/// rejects it as unsupported instead of mis-planning it. The day an adapter
/// needs generic source items, instantiation lands here with its own tests.
///
/// Lifetime parameters do not count: they cannot appear in a field's type
/// structure.
fn generic_over_types(generics: &syn::Generics) -> bool {
    generics
        .params
        .iter()
        .any(|p| matches!(p, syn::GenericParam::Type(_) | syn::GenericParam::Const(_)))
}

/// Split `ty` into how it is held and what is held.
///
/// `&[T]` peels to `(SharedRef, [T])`, so the borrow lands on the edge and the
/// slice becomes an ordinary sequence node — which is what lets `&[T]` and
/// `Vec<T>` share one traversal.
fn peel_source_use(ty: &syn::Type) -> (SourceUse, syn::Type) {
    match ty {
        syn::Type::Reference(r) => {
            let source = if r.mutability.is_some() {
                SourceUse::ExclusiveRef
            } else {
                SourceUse::SharedRef
            };
            (source, (*r.elem).clone())
        }
        other => (SourceUse::Value, other.clone()),
    }
}

/// Peel wrappers that carry no semantic structure. `Box<T>` is heap
/// indirection — a representation fact Tier 1 decides about — and treating it
/// as transparent is also what makes `Option<Box<Node>>` a legal recursion.
fn strip_transparent(ty: &syn::Type) -> syn::Type {
    if super::types_util::path_tail_ident(ty)
        .map(|i| i == "Box")
        .unwrap_or(false)
    {
        if let Some(inner) = super::types_util::first_type_arg(ty) {
            return strip_transparent(&inner);
        }
    }
    ty.clone()
}

/// Classify an array-length expression by what a consumer can do with it — see
/// [`ArrayLen`].
fn array_len(expr: &syn::Expr) -> ArrayLen {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(i),
            ..
        }) => i
            .base10_parse::<usize>()
            .map(ArrayLen::Literal)
            .unwrap_or_else(|_| {
                ArrayLen::Other(quote::ToTokens::to_token_stream(expr).to_string())
            }),
        // A bare const ident. A qualified path is `Other`: this tier does not
        // guess which namespace it names, the same rule `source_item_ident`
        // applies to types.
        syn::Expr::Path(p)
            if p.qself.is_none()
                && p.path.leading_colon.is_none()
                && p.path.segments.len() == 1
                && matches!(p.path.segments[0].arguments, syn::PathArguments::None) =>
        {
            ArrayLen::Named(p.path.segments[0].ident.clone())
        }
        other => ArrayLen::Other(quote::ToTokens::to_token_stream(other).to_string()),
    }
}

/// The element of a `Cow<'_, [E]>`, if `ty` is one.
fn cow_slice_elem(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Cow" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    args.args.iter().find_map(|arg| match arg {
        syn::GenericArgument::Type(syn::Type::Slice(slice)) => Some((*slice.elem).clone()),
        _ => None,
    })
}

#[cfg(test)]
mod tests;
