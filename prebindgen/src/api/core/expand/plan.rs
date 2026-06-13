//! Resolved constructor-expansion plans.

/// Outer shape wrapping the core construct. The value-side analog of how the
/// `Option<_>` / `Vec<_>` wrapper converters compose at the wire.
///
/// The unified [`Shape`](crate::api::core::shape::Shape) layer stack: `Base`
/// builds the target directly from the decoded leaves (a single constructor of
/// any arity or a combined-selector dispatch); `Optional((), inner)` lifts that
/// over `Option<T>`/`Option<&T>` (`Some` ⇒ run `inner` on the unwrapped value
/// and re-wrap, `None` ⇒ `None`; inner is always `Base` today);
/// `Iterable(inner)` maps `inner` over each element of a `Vec<T>` (emit-ready
/// but not yet produced by `apply`). The `()` payload is unused here — only the
/// JNI adapter's `Shape<NullableKind>` carries per-layer data.
pub use crate::api::core::shape::Shape as FoldShape;

/// A resolved expansion for one `(function, parameter)`.
#[derive(Clone)]
pub struct FoldPlan {
    /// Owned type the core construct produces — what the underlying call needs
    /// (before any [`Self::shape`] wrapping).
    pub target: syn::Type,
    /// True when the original parameter was `&T` / `Option<&T>`: the call
    /// receives `&folded` (or `folded.as_ref()` when also optional). A
    /// call-site concern (the resolver's `&_` handler shares the inner
    /// converter the same way), not part of the fold.
    pub by_ref: bool,
    /// Outer shape over the core construct (`Construct` for a plain `T`/`&T`
    /// param; `Optional(Construct)` for `Option<T>`/`Option<&T>`).
    pub shape: FoldShape,
    /// Flattened wire leaves, in foreign-signature order.
    pub leaves: Vec<FoldLeaf>,
    /// Index into [`Self::leaves`] of the selector leaf; `None` for a single
    /// constructor (the sole variant is applied unconditionally).
    pub selector: Option<usize>,
    /// Index into [`Self::leaves`] of the explicit presence-flag (`bool`) leaf
    /// for a **multi-argument** `Optional` shape (`Option<T>` built from a
    /// constructor taking ≥2 args): the flag decides `Some`/`None`, the arg
    /// leaves are plain (non-`Option`). `None` for a non-optional fold or the
    /// legacy single-arg `Optional` (where presence rides the sole leaf's own
    /// `Option`-ness). A separate flag avoids boxing a nullable primitive arg
    /// (e.g. `Option<i32>` → `Integer?`) on the wire.
    pub present: Option<usize>,
    /// Dispatch arms — one for a single constructor, selector order for a
    /// combined one.
    pub variants: Vec<FoldVariant>,
}

impl FoldPlan {
    /// True when the fold produces an `Option<_>` (outermost shape layer is
    /// `Optional`) — drives the by-ref call-site form (`folded.as_ref()`).
    pub fn produces_option(&self) -> bool {
        matches!(self.shape, FoldShape::Optional((), _))
    }
}

/// One flattened wire leaf of an expanded parameter.
#[derive(Clone)]
pub struct FoldLeaf {
    /// Foreign-side parameter name.
    pub name: syn::Ident,
    /// Rust type whose resolved **input** converter decodes this leaf. For a
    /// single constructor these are the raw constructor parameter types; for a
    /// combined one the selector (`i32`) and `Option`-wrapped variant inputs.
    pub ty: syn::Type,
}

/// One dispatch arm of a [`FoldPlan`].
#[derive(Clone)]
pub struct FoldVariant {
    /// `None` => identity (pass the decoded target value through). `Some` =>
    /// call this constructor function.
    pub ctor: Option<syn::Ident>,
    /// Whether the constructor returns `Result` (its `Err` is routed through
    /// the adapter's error channel). Always `false` for identity.
    pub fallible: bool,
    /// `true` for a borrowed identity arm (`&T` parameter): the input leaf is
    /// `Option<&T>` and the fold clones it (`T: Clone`) so the caller's handle
    /// is preserved rather than consumed. `false` otherwise.
    pub clone: bool,
    /// This variant's constructor inputs, in parameter order. Each is either a
    /// flat wire leaf or a recursively-built sub-value (a parameter that is
    /// itself a type with a canonical constructor — recursive input).
    pub inputs: Vec<FoldArg>,
}

/// One constructor-parameter input of a [`FoldVariant`].
#[derive(Clone)]
pub enum FoldArg {
    /// Decode the flat wire leaf at this index into [`FoldPlan::leaves`].
    Leaf(usize),
    /// Build this parameter by recursively folding its own canonical
    /// constructor (the parameter's type is itself a ptr_class with a canonical
    /// input). Its leaves live in the shared flat [`FoldPlan::leaves`].
    Build(Box<FoldBuild>),
}

/// A recursively-nested construction for one [`FoldArg::Build`] parameter — the
/// same dispatch shape as a top-level [`FoldPlan`]'s core, minus the outer
/// `Option`/`Vec` wrapping (a nested param is built by value).
#[derive(Clone)]
pub struct FoldBuild {
    /// Owned type this nested build produces (the constructor parameter type).
    pub target: syn::Type,
    /// `true` when the consuming parameter is `&T` (the built value is borrowed
    /// at the call site).
    pub by_ref: bool,
    /// Selector leaf index for a combined nested build; `None` for a single one.
    pub selector: Option<usize>,
    /// Dispatch arms (recursive).
    pub variants: Vec<FoldVariant>,
}
