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
    /// `String`, `&str`, `Cow<str>`.
    Str(Holding),
    /// `Vec<T>`, `&[T]`, `&Vec<T>`, `Cow<[T]>`.
    Seq {
        /// The sequence element to lower recursively.
        elem: &'t TypeRef,
        /// Whether the sequence is owned, borrowed, or held in a `Cow`.
        holding: Holding,
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
    /// `Cow<T>` over anything but `str` or a slice.
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
/// with `holding: Holding::Owned` and the element `Payload`. Only a call on
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
/// `String` is also offered to `lookup`, allowing an explicit declaration
/// to take precedence over built-in text handling. Without that declaration,
/// `String` is [`Shape::Str`] with [`Holding::Owned`], whereas `&String` and
/// `&mut String` are [`Shape::Ref`]. Other built-in containers do not consult
/// `lookup` at their outer layer.
///
/// # Borrows and containers
///
/// Several common combinations are returned as a single shape:
///
/// | Source type | Shape |
/// |---|---|
/// | `&str` | `Str(Borrowed)` |
/// | `Vec<T>` | `Seq { elem: T, holding: Owned }` |
/// | `&[T]` | `Seq { elem: T, holding: Borrowed }` |
/// | `&Vec<T>` | `Seq { elem: T, holding: BorrowedVec }` |
/// | `Cow<str>` / `Cow<[T]>` | `Str(Cow)` / `Seq { elem: T, holding: Cow }` |
/// | `[T; N]` | `Array { elem: T, len: N }`, using the resolved length |
/// | `&mut MaybeUninit<T>` | `Out(T)`, a slot the callee fills |
///
/// Other borrows, including `&mut str`, `&mut [T]`, and `&mut Vec<T>`,
/// remain [`Shape::Ref`] with their inner type and access. `Option`, `Box`,
/// other `Cow` types, `Result`, and callbacks expose their children without
/// visiting them. Lifetimes are not represented in `Shape`; the original
/// [`TypeRef`] remains available to the caller.
///
/// # Errors
///
/// A bare `str`, bare slice `[T]`, or bare `MaybeUninit<T>` returns an error
/// explaining the supported form. The `&mut MaybeUninit<T>` case above is
/// recognized before its inner type is visited. A different borrow such as
/// `&MaybeUninit<T>` initially returns `Ref`; classifying its child then
/// encounters the bare-`MaybeUninit` error.
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
