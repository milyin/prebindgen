//! `Shape<N>` — a leaf wrapped in an ordered stack of structural layers
//! (`Option`, `Vec`), with a bottom-up [`fold_shape`] combinator.
//!
//! One shared algebra replaces the three former per-module copies of the same
//! "leaf + wrapper layers" idea:
//!   * jnigen's `FoldStrategy` (`Direct` / `Nullable { kind, inner }` /
//!     `Iterable`) → `Shape<NullableKind>` (the `Optional` layer carries the
//!     null-representation choice),
//!   * expand's `FoldShape` (`Construct` / `Optional` / `Iterable`) → `Shape`
//!     (= `Shape<()>`),
//!   * unfold's `UnfoldShape` (`Decompose` / `Optional` / `Iterable`) →
//!     `Shape`.
//!
//! `N` is the per-`Optional`-layer payload: `()` for the language-agnostic
//! engines, an adapter type (e.g. jnigen's `NullableKind`) where the layer
//! needs to remember how null is represented over the wire.

/// A base leaf wrapped in zero or more `Optional` / `Iterable` layers, from the
/// inside out. `N` is the payload each `Optional` layer carries.
#[derive(Clone, Debug)]
pub enum Shape<N = ()> {
    /// The leaf — no wrapping layers.
    Base,
    /// `Option<…>` layer over `inner`. `meta` is this layer's payload.
    Optional(N, Box<Shape<N>>),
    /// `Vec<…>` / `List<…>` layer over `inner`.
    Iterable(Box<Shape<N>>),
}

impl<N> Shape<N> {
    /// `Optional(meta, inner)` without the explicit `Box`.
    pub fn optional(meta: N, inner: Shape<N>) -> Self {
        Shape::Optional(meta, Box::new(inner))
    }

    /// `Iterable(inner)` without the explicit `Box`.
    pub fn iterable(inner: Shape<N>) -> Self {
        Shape::Iterable(Box::new(inner))
    }

    /// True when any layer of the stack is `Iterable`. This is the
    /// fold-delivery discriminator: a fold surface (accumulator + per-element
    /// callback) is selected whether or not `Optional` layers wrap the
    /// iterable, and an iterable-shaped value has no single return.
    pub fn has_iterable_layer(&self) -> bool {
        match self {
            Shape::Base => false,
            Shape::Optional(_, inner) => inner.has_iterable_layer(),
            Shape::Iterable(_) => true,
        }
    }
}

/// Bottom-up fold over the layer stack: compute the leaf value with `on_base`,
/// then apply `on_optional` / `on_iterable` for each wrapping layer from the
/// inside out. `on_optional` also receives the layer's payload `&N` and the
/// `Shape` it wraps, so callers can special-case e.g. a layer sitting directly
/// over the leaf.
///
/// This is the generalization of jnigen's former `fold_strategy`.
pub fn fold_shape<N, T>(
    s: &Shape<N>,
    on_base: &dyn Fn() -> T,
    on_optional: &dyn Fn(T, &N, &Shape<N>) -> T,
    on_iterable: &dyn Fn(T) -> T,
) -> T {
    match s {
        Shape::Base => on_base(),
        Shape::Optional(meta, inner) => {
            let inner_val = fold_shape(inner, on_base, on_optional, on_iterable);
            on_optional(inner_val, meta, inner)
        }
        Shape::Iterable(inner) => {
            let inner_val = fold_shape(inner, on_base, on_optional, on_iterable);
            on_iterable(inner_val)
        }
    }
}
