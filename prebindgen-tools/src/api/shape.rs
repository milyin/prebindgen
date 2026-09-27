use prebindgen_flat::flat::{ScalarKind, TypeKind, TypeRef};

/// How a value is held where it appears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    /// By value: the callee owns it.
    Owned,
    /// `&T`.
    Shared,
    /// `&mut T`.
    Exclusive,
}

/// How a run of values (text or a sequence) is held.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holding {
    /// `String`, `Vec<T>`.
    Owned,
    /// `&str`, `&[T]`.
    Borrowed,
    /// `&Vec<T>`.
    BorrowedVec,
    /// `Cow<str>`, `Cow<[T]>`.
    Cow,
}

/// A type, one layer deep.
#[derive(Debug)]
pub enum Shape<'t, S> {
    /// `()`.
    Unit,
    /// A primitive.
    Scalar(ScalarKind),
    /// `String`, `&str`, `Cow<str>`.
    Str(Holding),
    /// `Vec<T>`, `&[T]`, `&Vec<T>`, `Cow<[T]>`.
    Seq { elem: &'t TypeRef, holding: Holding },
    /// `[T; N]`.
    Array { elem: &'t TypeRef, len: usize },
    /// `Option<T>`.
    Option(&'t TypeRef),
    /// `Box<T>`.
    Boxed(&'t TypeRef),
    /// `Cow<T>` over anything but `str` or a slice.
    Cow(&'t TypeRef),
    /// A borrow of anything [`Shape::Str`], [`Shape::Seq`] and
    /// [`Shape::Declared`] do not cover.
    Ref { inner: &'t TypeRef, access: Access },
    /// A named type the adapter has a setting for, held by value or
    /// borrowed.
    Declared {
        name: &'t str,
        setting: S,
        access: Access,
    },
    /// A named type the adapter has no setting for.
    Undeclared(&'t str),
    /// `impl Fn(A, B)`.
    Callback(&'t [TypeRef]),
    /// `Result<T, E>`.
    Result { ok: &'t TypeRef, err: &'t TypeRef },
    /// `&mut MaybeUninit<T>`: a slot the callee fills.
    Out(&'t TypeRef),
}

/// The outermost layer of `ty`. `lookup` gives the adapter's setting for a
/// named type (`String` included, so an adapter may declare it); a borrow of
/// a named type is that type with its [`Access`].
///
/// A bare `str` or slice, and a `MaybeUninit` outside `&mut`, are refused:
/// no value of them can cross.
pub fn shape<'t, S>(
    ty: &'t TypeRef,
    lookup: impl Fn(&str) -> Option<S>,
) -> Result<Shape<'t, S>, String> {
    let named = |name: &'t str, access: Access| match lookup(name) {
        Some(setting) => Shape::Declared {
            name,
            setting,
            access,
        },
        None => Shape::Undeclared(name),
    };
    Ok(match ty.kind() {
        TypeKind::Unit => Shape::Unit,
        TypeKind::Scalar(k) => Shape::Scalar(*k),
        TypeKind::String => match lookup("String") {
            Some(setting) => Shape::Declared {
                name: "String",
                setting,
                access: Access::Owned,
            },
            None => Shape::Str(Holding::Owned),
        },
        TypeKind::Str => return Err("a bare `str` cannot cross; use `&str` or `String`".into()),
        TypeKind::Slice(_) => {
            return Err("a bare slice cannot cross; use `&[T]` or `Vec<T>`".into())
        }
        TypeKind::Optional(t) => Shape::Option(t),
        TypeKind::Vec(t) => Shape::Seq {
            elem: t,
            holding: Holding::Owned,
        },
        TypeKind::Array { elem, extent } => Shape::Array {
            elem,
            len: extent.value,
        },
        TypeKind::Boxed(t) => Shape::Boxed(t),
        TypeKind::Cow { inner, .. } => match inner.kind() {
            TypeKind::Str => Shape::Str(Holding::Cow),
            TypeKind::Slice(e) => Shape::Seq {
                elem: e,
                holding: Holding::Cow,
            },
            _ => Shape::Cow(inner),
        },
        TypeKind::Ref { mutable, inner, .. } => {
            let access = if *mutable {
                Access::Exclusive
            } else {
                Access::Shared
            };
            match inner.kind() {
                TypeKind::Str if !mutable => Shape::Str(Holding::Borrowed),
                TypeKind::Slice(e) if !mutable => Shape::Seq {
                    elem: e,
                    holding: Holding::Borrowed,
                },
                TypeKind::Vec(e) if !mutable => Shape::Seq {
                    elem: e,
                    holding: Holding::BorrowedVec,
                },
                TypeKind::Uninit(t) if *mutable => Shape::Out(t),
                TypeKind::Named { id, .. } => named(&id.name, access),
                TypeKind::String => match lookup("String") {
                    Some(setting) => Shape::Declared {
                        name: "String",
                        setting,
                        access,
                    },
                    None => Shape::Ref { inner, access },
                },
                _ => Shape::Ref { inner, access },
            }
        }
        TypeKind::Named { id, .. } => named(&id.name, Access::Owned),
        TypeKind::Callback { args } => Shape::Callback(args),
        TypeKind::Fallible { ok, err } => Shape::Result { ok, err },
        TypeKind::Uninit(_) => {
            return Err("`MaybeUninit<T>` crosses only as `&mut MaybeUninit<T>`".into())
        }
    })
}
