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

/// Describe one outer layer of a source type for a language adapter.
///
/// Pass the [`TypeRef`] being converted and a closure that answers which
/// named types this adapter has declared. The closure receives a type name
/// such as `"Payload"` and returns the adapter's setting for it, or `None`.
/// The adapter supplies this `lookup` closure from its declarations. It can
/// pass `|name| settings.get(name)` to borrow a setting from a map.
///
/// # Walking into child types
///
/// An adapter can get `ty` from a function parameter in the flat model, then
/// call `shape(ty, lookup)` and match the returned [`Shape`]. Some variants
/// hold a child [`TypeRef`]; call `shape` again with that child and the same
/// closure. This example shows each call for `Option<Vec<Payload>>`:
///
/// ```
/// use std::collections::HashMap;
/// use prebindgen::SourceLocation;
/// use prebindgen_flat::Flat;
/// use prebindgen_tools::{shape, Access, Holding, Shape};
///
/// let source = syn::parse_file(
///     "pub struct Payload; pub fn send(value: Option<Vec<Payload>>) {}"
/// ).unwrap();
/// let flat = Flat::builder()
///     .items(source.items.into_iter().map(|item| (item, SourceLocation::default())))
///     .build().unwrap();
/// let ty = &flat.function("send").unwrap().params[0].ty;
/// let settings = HashMap::from([("Payload".to_owned(), "opaque handle")]);
/// let lookup = |name: &str| settings.get(name).copied();
///
/// let Shape::Option(inner) = shape(ty, lookup).unwrap() else { panic!() };
/// let Shape::Seq { elem, holding: Holding::Owned } = shape(inner, lookup).unwrap()
///     else { panic!() };
/// assert!(matches!(
///     shape(elem, lookup).unwrap(),
///     Shape::Declared {
///         name: "Payload", setting: "opaque handle", access: Access::Owned
///     }
/// ));
/// ```
///
/// The first two calls inspect the `Option` and `Vec` layers; neither calls
/// `lookup`. The third reaches `Payload`, calls `lookup("Payload")`, and
/// returns [`Shape::Declared`] with that setting. The adapter decides how
/// to represent each layer on its boundary and which children to visit.
/// A successful call describes only the current layer; it does not establish
/// that the adapter can convert the complete type.
///
/// # Named types
///
/// `lookup` receives only the flat name, without generic arguments or
/// borrow information. The original `ty` remains available if the adapter
/// needs those details beyond what the returned shape carries.
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
