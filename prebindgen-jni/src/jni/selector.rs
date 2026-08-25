//! Structural converter-selection policy for [`Declarations`].

use prebindgen_registry::Conversions;

use super::*;

impl Declarations {
    /// `Result<T, E>` **output**: succeeds as `T`, routes `E` to the error sink.
    ///
    /// Read off the model, which calls this shape `TypeKind::Fallible`.
    pub(crate) fn result_shape(
        &self,
        ty: &prebindgen_registry::flat::TypeRef,
        registry: &impl Conversions,
        emit: &prebindgen_registry::Emit,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let (ok, err) = fallible_parts(ty, emit)?;
        self.result_peel(ty, &ok, &err, registry, emit)
    }
}

/// The `Ok`/`Err` of a `Result`, spelled.
///
/// The model classifies a `Result` as [`TypeKind::Fallible`], so a reading
/// answers this directly — no lookup, and no syntactic fallback.
///
/// The fallback there used to be (`types_util::result_parts` over the node)
/// existed because the caller held only a **spelling**: a `Result` the adapter
/// composed itself would have no entry in `flat`, so the reading lookup could
/// miss. A caller holding a `TypeRef` cannot be in that position — #280 sealed
/// minting to the model, so every reading reaching here was classified, and
/// `kind` is what says whether it is a `Result`. The fallback was measured
/// never to fire in-tree; it is now unreachable by construction.
fn fallible_parts(
    ty: &prebindgen_registry::flat::TypeRef,
    emit: &prebindgen_registry::Emit,
) -> Option<(syn::Type, syn::Type)> {
    let (ok, err) = ty.fallible_parts()?;
    let (ok, err) = (emit.spell(ok), emit.spell(err));
    Some((syn::parse_quote!(#ok), syn::parse_quote!(#err)))
}
