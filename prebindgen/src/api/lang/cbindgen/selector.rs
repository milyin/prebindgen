//! Structural converter-selection policy for [`Cbindgen`].

use super::*;

impl Cbindgen {
    /// Select the input converter for `ty`: terminal categories, then built-in
    /// C structural wrappers.
    pub(crate) fn select_input_type(
        &self,
        ty: &syn::Type,
        registry: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        self.in_opaque_handle(ty)
            .or_else(|| self.in_data_struct(ty, registry))
            .or_else(|| self.in_value_opaque(ty, registry))
            .or_else(|| self.in_enum(ty, registry))
            .or_else(|| self.in_string(ty))
            .or_else(|| self.in_str(ty))
            .or_else(|| self.in_scalar(ty))
            .or_else(|| self.in_wrappers(ty, registry))
    }

    /// Select the output converter for `ty`: terminal categories, then built-in
    /// C structural wrappers.
    pub(crate) fn select_output_type(
        &self,
        ty: &syn::Type,
        registry: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        self.out_terminal(ty, registry)
            .or_else(|| self.out_wrappers(ty, registry))
    }
}
