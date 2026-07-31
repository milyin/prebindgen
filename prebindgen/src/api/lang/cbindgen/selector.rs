//! Structural converter-selection policy for [`CbindgenBuilder`].

use super::*;
use crate::api::core::registry::Conversions;

impl CbindgenBuilder {
    /// Select the input converter for `ty`: terminal categories, then built-in
    /// C structural wrappers.
    pub(crate) fn select_input_type(
        &self,
        ty: &syn::Type,
        registry: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        self.in_custom(ty, registry)
            .or_else(|| self.in_opaque_handle(ty))
            .or_else(|| self.in_data_struct(ty, registry))
            .or_else(|| self.in_value_opaque(ty, registry))
            .or_else(|| self.in_enum(ty, registry))
            .or_else(|| self.in_tagged_union(ty, registry))
            .or_else(|| self.in_string(ty))
            .or_else(|| self.in_str(ty))
            .or_else(|| self.in_bool(ty))
            .or_else(|| self.in_scalar(ty))
            .or_else(|| self.in_wrappers(ty, registry))
    }

    /// Select the output converter for `ty`: terminal categories, then built-in
    /// C structural wrappers.
    pub(crate) fn select_output_type(
        &self,
        ty: &syn::Type,
        registry: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        self.out_custom(ty, registry)
            .or_else(|| self.out_terminal(ty, registry))
            .or_else(|| self.out_wrappers(ty, registry))
    }
}
