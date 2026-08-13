//! The type axis, taken from the model rather than restated.
//!
//! [`TypeKind`] is the closed set of Rust type forms a `#[prebindgen]` crate
//! may write. The matrix must cover all of them, and the only way to keep that
//! true as the language grows is to make the enumerator *fail to compile* when
//! a form is added — which is what [`tag_of`] is for.

use prebindgen_flat::flat::{GenericArg, TypeKind, TypeRef};

/// One accepted Rust type form.
///
/// Exactly one variant per [`TypeKind`] variant. It exists separately only
/// because a tag has to be namable, comparable and printable without carrying a
/// type; adding one here without adding it to `TypeKind` is meaningless, and
/// the reverse is a compile error in [`tag_of`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TypeTag {
    Scalar,
    Str,
    String,
    Optional,
    Vec,
    Slice,
    Array,
    Fallible,
    Named,
    Ref,
    Boxed,
    Cow,
    Uninit,
    Callback,
    Unit,
}

impl TypeTag {
    /// Every form, in the order the report lists them.
    pub const ALL: &'static [TypeTag] = &[
        TypeTag::Scalar,
        TypeTag::Str,
        TypeTag::String,
        TypeTag::Optional,
        TypeTag::Vec,
        TypeTag::Slice,
        TypeTag::Array,
        TypeTag::Fallible,
        TypeTag::Named,
        TypeTag::Ref,
        TypeTag::Boxed,
        TypeTag::Cow,
        TypeTag::Uninit,
        TypeTag::Callback,
        TypeTag::Unit,
    ];

    /// The form as the report names it — the Rust spelling, not the variant.
    pub fn as_str(self) -> &'static str {
        match self {
            TypeTag::Scalar => "scalar",
            TypeTag::Str => "str",
            TypeTag::String => "String",
            TypeTag::Optional => "Option<T>",
            TypeTag::Vec => "Vec<T>",
            TypeTag::Slice => "[T]",
            TypeTag::Array => "[T; N]",
            TypeTag::Fallible => "Result<T, E>",
            TypeTag::Named => "named type",
            TypeTag::Ref => "&T / &mut T",
            TypeTag::Boxed => "Box<T>",
            TypeTag::Cow => "Cow<'a, T>",
            TypeTag::Uninit => "MaybeUninit<T>",
            TypeTag::Callback => "impl Fn(..)",
            TypeTag::Unit => "()",
        }
    }
}

/// The gate.
///
/// A new [`TypeKind`] variant stops this compiling, which stops the matrix
/// building, until the form has a tag — and then the completeness test
/// (`every_type_form_is_covered`) fails until it also has a fixture. That chain
/// is the whole point: the previous version of this matrix carried a *hand
/// written* grammar of shapes, which had already drifted to 8 of the 15 forms,
/// and the same drift shipped once before — a spec listing `Vec | [T] | Cow<[T]>`
/// as the sequence forms let `[u8; 16]` degrade silently to an opaque leaf
/// (#190, corrected only when something finally needed it).
pub fn tag_of(kind: &TypeKind) -> TypeTag {
    match kind {
        TypeKind::Scalar(_) => TypeTag::Scalar,
        TypeKind::Str => TypeTag::Str,
        TypeKind::String => TypeTag::String,
        TypeKind::Optional(_) => TypeTag::Optional,
        TypeKind::Vec(_) => TypeTag::Vec,
        TypeKind::Slice(_) => TypeTag::Slice,
        TypeKind::Array { .. } => TypeTag::Array,
        TypeKind::Fallible { .. } => TypeTag::Fallible,
        TypeKind::Named { .. } => TypeTag::Named,
        TypeKind::Ref { .. } => TypeTag::Ref,
        TypeKind::Boxed(_) => TypeTag::Boxed,
        TypeKind::Cow { .. } => TypeTag::Cow,
        TypeKind::Uninit(_) => TypeTag::Uninit,
        TypeKind::Callback { .. } => TypeTag::Callback,
        TypeKind::Unit => TypeTag::Unit,
    }
}

/// The types written *inside* this one, one level down.
///
/// The second gate, and the reason this is not
/// [`TypeRef::walk`](prebindgen_flat::flat::TypeRef::walk): that walk descends
/// through transparent wrappers on purpose, so `Box<Vec<T>>` reaches `T`
/// without ever yielding the `Vec` node. Coverage has to see every form the
/// fixture *writes*, transparent or not.
fn children(kind: &TypeKind) -> Vec<&TypeRef> {
    match kind {
        TypeKind::Optional(t)
        | TypeKind::Vec(t)
        | TypeKind::Slice(t)
        | TypeKind::Boxed(t)
        | TypeKind::Uninit(t)
        | TypeKind::Cow { inner: t, .. }
        | TypeKind::Ref { inner: t, .. } => vec![t],
        TypeKind::Array { elem, .. } => vec![elem],
        TypeKind::Fallible { ok, err } => vec![ok, err],
        TypeKind::Callback { args } => args.iter().collect(),
        TypeKind::Named { args, .. } => args
            .iter()
            .filter_map(|a| match a {
                GenericArg::Type(t) => Some(&**t),
                GenericArg::Lifetime(_) => None,
            })
            .collect(),
        TypeKind::Scalar(_) | TypeKind::Str | TypeKind::String | TypeKind::Unit => vec![],
    }
}

/// Every form appearing anywhere in `ty`, root included.
///
/// A fixture covers the forms it *contains*, not just the one it starts with:
/// `&str` is a borrow of a `str`, and writing it once covers both.
pub fn tags_in(ty: &TypeRef) -> Vec<TypeTag> {
    let mut tags = Vec::new();
    collect(ty, &mut tags);
    tags.sort();
    tags.dedup();
    tags
}

fn collect(ty: &TypeRef, out: &mut Vec<TypeTag>) {
    out.push(tag_of(ty.kind()));
    for child in children(ty.kind()) {
        collect(child, out);
    }
}
