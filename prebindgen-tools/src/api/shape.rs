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

/// The outer layer of a model type, with the adapter's setting for named types.
///
/// Returned by [`shape()`]. Child types borrow the input model; they have not
/// been classified recursively. The adapter matches this enum to choose a
/// representation and calls `shape()` on whichever children it needs.
/// `S` is the adapter's own setting type, often a borrowed declaration.
#[derive(Debug)]
pub enum Shape<'t, S> {
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
    /// A named type the adapter has a setting for, held by value or
    /// borrowed.
    Declared {
        /// Name in the flat namespace.
        name: &'t str,
        /// The adapter's setting returned by the `lookup` callback to [`shape()`].
        setting: S,
        /// Whether this use owns or borrows the named type.
        access: Access,
    },
    /// A named type the adapter has no setting for.
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

/// Classify one outer layer of `ty` for an adapter to lower.
///
/// This is the common first step in deciding how a source value crosses a
/// binding boundary. It describes the source shape and attaches an adapter
/// setting when available. It does not choose wire types, generate conversion
/// code, inspect a declared struct's fields, or validate all nested types.
///
/// # Recursing over a type
///
/// For `Option<Vec<Payload>>`, the first call returns [`Shape::Option`] with
/// a reference to `Vec<Payload>`. Classifying that child returns [`Shape::Seq`]
/// with `kind: SequenceKind::Vec`, `access: Access::Owned`, and the
/// element `Payload`. Only a call on
/// that element consults `lookup("Payload")`.
///
/// The adapter matches each returned shape, chooses what that layer means
/// for its boundary, and recursively handles the children it needs. For
/// example, a sequence may become a pointer and length in C, or a JVM array
/// in JNI. Successfully classifying the outer layer does not imply the
/// adapter supports the whole type.
///
/// # Looking up declarations
///
/// `lookup` maps a named type's flat name to the adapter's own setting `S`.
/// It can return a reference, for example `|name| settings.get(name)`;
/// settings do not need to be cloned. The name alone is supplied, without
/// generic arguments or borrow information. Keep `ty` if the adapter needs
/// those details beyond what the returned shape carries.
///
/// For a named `T`, `&T`, or `&mut T`, a setting produces
/// [`Shape::Declared`] with [`Access::Owned`], [`Access::Shared`], or
/// [`Access::Exclusive`], respectively. Without a setting the result is
/// [`Shape::Undeclared`], not an error; that variant retains only the name.
/// The adapter decides whether to reject an undeclared type.
///
/// `String` is also offered to `lookup`, including when borrowed. A setting
/// takes precedence over its built-in text shape. Other built-in containers
/// do not consult `lookup` at their outer layer.
///
/// # Container kind and access
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
            None => Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            },
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
            let access = if *mutable {
                Access::Exclusive
            } else {
                Access::Shared
            };
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
                TypeKind::Named { id, .. } => named(&id.name, access),
                TypeKind::String => match lookup("String") {
                    Some(setting) => Shape::Declared {
                        name: "String",
                        setting,
                        access,
                    },
                    None => Shape::Str {
                        kind: TextKind::String,
                        access,
                    },
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
