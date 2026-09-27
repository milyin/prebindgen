use prebindgen_flat::flat::{ScalarKind, TypeKind, TypeRef};

/// How a value is held where it appears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    /// By value. This describes access, not whether the type is sized or
    /// supported at a binding boundary.
    Owned,
    /// `&T`.
    Shared,
    /// `&mut T`.
    Exclusive,
}

impl Access {
    /// Whether `ty` is owned, shared, or exclusively borrowed at its outer layer.
    pub fn of(ty: &TypeRef) -> Self {
        match ty.kind() {
            TypeKind::Ref { mutable: true, .. } => Self::Exclusive,
            TypeKind::Ref { .. } => Self::Shared,
            _ => Self::Owned,
        }
    }
}

/// The source text container, independently of how it is accessed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextKind {
    /// The growable, owned string container.
    String,
    /// The unsized string slice.
    Str,
}

/// The source sequence container, independently of how it is accessed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SequenceKind {
    /// The growable vector container.
    Vec,
    /// The unsized slice.
    Slice,
}

/// The outer layer of a model type, with any declaration the adapter made for it.
///
/// Returned by [`shape()`]. Child types borrow the input model; they have not
/// been classified recursively. The adapter matches this enum to choose a
/// representation and calls `shape()` on whichever children it needs.
/// `D` is the adapter's declaration type, often borrowed from its registry.
#[derive(Debug)]
pub enum Shape<'t, D> {
    /// `()`.
    Unit,
    /// A primitive.
    Scalar(ScalarKind),
    /// A string container or string slice, by value or borrowed.
    Str {
        /// Whether the source names `String` or `str`.
        kind: TextKind,
        /// By value, shared borrow, or exclusive borrow.
        access: Access,
    },
    /// A vector or slice, by value or borrowed.
    Seq {
        /// The sequence element to lower recursively.
        elem: &'t TypeRef,
        /// Whether the source names `Vec<T>` or `[T]`.
        kind: SequenceKind,
        /// By value, shared borrow, or exclusive borrow.
        access: Access,
    },
    /// `[T; N]`.
    Array {
        /// The array element to lower recursively.
        elem: &'t TypeRef,
        /// The fixed number of elements.
        len: usize,
    },
    /// `Option<T>`.
    Option(&'t TypeRef),
    /// `Box<T>`.
    Boxed(&'t TypeRef),
    /// `Cow<T>`, including `Cow<str>` and `Cow<[T]>`. The child is `T`.
    Cow(&'t TypeRef),
    /// A borrow of anything [`Shape::Str`], [`Shape::Seq`] and
    /// [`Shape::Declared`] do not cover.
    Ref {
        /// The referenced type to lower recursively.
        inner: &'t TypeRef,
        /// Shared or exclusive access.
        access: Access,
    },
    /// A type the adapter declared, with its original ownership or borrow.
    Declared {
        /// The original type, including generic arguments and any borrow.
        ty: &'t TypeRef,
        /// The adapter's declaration returned by `shape_declared`.
        declaration: D,
    },
    /// A named type for which the adapter returned no declaration.
    Undeclared(&'t str),
    /// `impl Fn(A, B)`.
    Callback(&'t [TypeRef]),
    /// `Result<T, E>`.
    Result {
        /// Successful value to lower recursively.
        ok: &'t TypeRef,
        /// Error value to lower recursively.
        err: &'t TypeRef,
    },
    /// `&mut MaybeUninit<T>`: a slot the callee fills.
    Out(&'t TypeRef),
}

/// Describe one outer layer of a source type for a language adapter.
///
/// Pass the [`TypeRef`] being converted and `shape_declared`, a callback that
/// returns the adapter's declaration for a type or `None`. The callback sees
/// the complete type, so an adapter can distinguish `Vec<u8>` from
/// `Vec<Payload>`. A returned declaration makes the result
/// [`Shape::Declared`], which retains the original `ty`.
///
/// `shape` asks about the exact `ty` first. For a borrow with no exact
/// declaration, it also asks about the borrowed inner type, so a declaration
/// for `Payload` still covers `&Payload`. An exact declaration for
/// `&Payload` takes precedence. Bare `MaybeUninit<T>` and the
/// `&mut MaybeUninit<T>` output slot are handled before the callback.
///
/// # Walking into child types
///
/// An adapter can get `ty` from a function parameter in the flat model, then
/// call `shape(ty, shape_declared)` and match the returned [`Shape`]. Some
/// variants hold a child [`TypeRef`]; call `shape` again with that child and
/// the same closure. This example shows each call for `Option<Vec<Payload>>`:
///
/// ```
/// use std::collections::HashMap;
/// use prebindgen::SourceLocation;
/// use prebindgen_flat::Flat;
/// use prebindgen_tools::{shape, Access, SequenceKind, Shape};
///
/// let source = syn::parse_file(
///     "pub struct Payload; pub fn send(value: Option<Vec<Payload>>, bytes: Vec<u8>) {}"
/// ).unwrap();
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|item| (item, SourceLocation::default())))
///     .build().unwrap();
/// let ty = &flat.function("send").unwrap().params[0].ty;
/// let declarations = HashMap::from([("Payload".to_owned(), "opaque handle")]);
/// let shape_declared = |candidate: &prebindgen_flat::flat::TypeRef| {
///     match candidate.kind() {
///         prebindgen_flat::flat::TypeKind::Named { id, .. } =>
///             declarations.get(&id.name).copied(),
///         _ => None,
///     }
/// };
///
/// let Shape::Option(inner) = shape(ty, shape_declared).unwrap() else { panic!() };
/// let Shape::Seq { elem, kind: SequenceKind::Vec, access: Access::Owned } = shape(inner, shape_declared).unwrap()
///     else { panic!() };
/// assert!(matches!(
///     shape(elem, shape_declared).unwrap(),
///     Shape::Declared {
///         declaration: "opaque handle", ty: declared_ty
///     } if std::ptr::eq(declared_ty, elem)
/// ));
///
/// // A declaration may target a complete generic type, too.
/// let bytes = &flat.function("send").unwrap().params[1].ty;
/// let bytes_key = bytes.key();
/// assert!(matches!(
///     shape(bytes, |candidate| (candidate.key() == bytes_key).then_some("blob"))
///         .unwrap(),
///     Shape::Declared { ty, declaration: "blob" } if std::ptr::eq(ty, bytes)
/// ));
/// ```
///
/// The first two calls return built-in shapes because the callback returns
/// `None` for `Option<Vec<Payload>>` and `Vec<Payload>`. The third returns
/// [`Shape::Declared`] for `Payload`. The adapter decides how to represent
/// each layer on its boundary and which children to visit.
/// A successful call describes only the current layer; it does not establish
/// that the adapter can convert the complete type.
///
/// # Container kind and access
///
/// Without an adapter declaration, container kinds and access remain explicit:
///
/// | Source type | Shape |
/// |---|---|
/// | `String` | `Str { kind: String, access: Owned }` |
/// | `&String` | `Str { kind: String, access: Shared }` |
/// | `&str` | `Str { kind: Str, access: Shared }` |
/// | `Vec<T>` | `Seq { elem: T, kind: Vec, access: Owned }` |
/// | `&Vec<T>` | `Seq { elem: T, kind: Vec, access: Shared }` |
/// | `&[T]` | `Seq { elem: T, kind: Slice, access: Shared }` |
/// | `&mut [T]` | `Seq { elem: T, kind: Slice, access: Exclusive }` |
/// | `Cow<str>` / `Cow<[T]>` | `Cow(str)` / `Cow([T])` |
/// | `[T; N]` | `Array { elem: T, len: N }`, using the resolved length |
/// | `&mut MaybeUninit<T>` | `Out(T)`, a slot the callee fills |
///
/// Mutable borrows of `String`, `str`, `Vec<T>`, and `[T]` have the same
/// container kind as shared borrows, with [`Access::Exclusive`]. `Cow`
/// remains a wrapper: its runtime ownership is not an access mode.
/// `Option`, `Box`, `Cow`, `Result`, and callbacks expose their children
/// without visiting them. Lifetimes remain on the original [`TypeRef`].
///
/// Bare `str` and `[T]` classify as unsized containers with [`Access::Owned`].
/// This lets an adapter inspect a `Cow` child without inventing another type.
/// It does not make those types valid by-value parameters: the adapter must
/// reject unsupported uses, including mutable containers it cannot implement.
///
/// # Errors
///
/// Bare `MaybeUninit<T>` is refused; only `&mut MaybeUninit<T>` is recognized
/// as an output slot. A different borrow initially returns `Ref`; visiting
/// its child then encounters the bare-`MaybeUninit` error.
///
/// See the [`shape` module](mod@crate::shape) for a runnable recursive example.
pub fn shape<D>(
    ty: &TypeRef,
    shape_declared: impl Fn(&TypeRef) -> Option<D>,
) -> Result<Shape<'_, D>, String> {
    match ty.kind() {
        TypeKind::Uninit(_) => {
            return Err("`MaybeUninit<T>` crosses only as `&mut MaybeUninit<T>`".into())
        }
        TypeKind::Ref {
            mutable: true,
            inner,
            ..
        } => {
            if let TypeKind::Uninit(t) = inner.kind() {
                return Ok(Shape::Out(t));
            }
        }
        _ => {}
    }
    if let Some(declaration) = shape_declared(ty) {
        return Ok(Shape::Declared { ty, declaration });
    }
    if let TypeKind::Ref { inner, .. } = ty.kind() {
        if !matches!(inner.kind(), TypeKind::Uninit(_)) {
            if let Some(declaration) = shape_declared(inner) {
                return Ok(Shape::Declared { ty, declaration });
            }
        }
    }
    Ok(match ty.kind() {
        TypeKind::Unit => Shape::Unit,
        TypeKind::Scalar(k) => Shape::Scalar(*k),
        TypeKind::String => Shape::Str {
            kind: TextKind::String,
            access: Access::Owned,
        },
        TypeKind::Str => Shape::Str {
            kind: TextKind::Str,
            access: Access::Owned,
        },
        TypeKind::Slice(elem) => Shape::Seq {
            elem,
            kind: SequenceKind::Slice,
            access: Access::Owned,
        },
        TypeKind::Optional(t) => Shape::Option(t),
        TypeKind::Vec(t) => Shape::Seq {
            elem: t,
            kind: SequenceKind::Vec,
            access: Access::Owned,
        },
        TypeKind::Array { elem, extent } => Shape::Array {
            elem,
            len: extent.value,
        },
        TypeKind::Boxed(t) => Shape::Boxed(t),
        TypeKind::Cow { inner, .. } => Shape::Cow(inner),
        TypeKind::Ref { mutable, inner, .. } => {
            let access = Access::of(ty);
            match inner.kind() {
                TypeKind::Str => Shape::Str {
                    kind: TextKind::Str,
                    access,
                },
                TypeKind::Slice(elem) => Shape::Seq {
                    elem,
                    kind: SequenceKind::Slice,
                    access,
                },
                TypeKind::Vec(elem) => Shape::Seq {
                    elem,
                    kind: SequenceKind::Vec,
                    access,
                },
                TypeKind::Uninit(t) if *mutable => Shape::Out(t),
                TypeKind::Named { id, .. } => Shape::Undeclared(&id.name),
                TypeKind::String => Shape::Str {
                    kind: TextKind::String,
                    access,
                },
                _ => Shape::Ref { inner, access },
            }
        }
        TypeKind::Named { id, .. } => Shape::Undeclared(&id.name),
        TypeKind::Callback { args } => Shape::Callback(args),
        TypeKind::Fallible { ok, err } => Shape::Result { ok, err },
        TypeKind::Uninit(_) => {
            return Err("`MaybeUninit<T>` crosses only as `&mut MaybeUninit<T>`".into())
        }
    })
}
