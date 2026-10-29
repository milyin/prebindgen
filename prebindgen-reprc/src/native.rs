//! Layout queries for existing native records; no replacement struct is emitted.
use crate::{MeasuredLayout, Record, Result};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};
/// A named-field native type whose actual target layout is measured in generated code.
/// The adapter verifies representation and field-carrier compatibility.
#[derive(Clone, Debug)]
pub struct NativeRecord {
    ty: Type,
    fields: Vec<(Ident, Type)>,
}
impl NativeRecord {
    /// Describe an existing type and its real fields, preserving declaration order.
    /// No ownership, bit-validity or representation guarantee is inferred.
    pub fn new(ty: Type, fields: impl IntoIterator<Item = (Ident, Type)>) -> Result<Self> {
        let fields = fields.into_iter().collect::<Vec<_>>();
        Record::new(syn::parse_quote!(__NativeRecordValidation), fields.clone())?;
        Ok(Self { ty, fields })
    }
    /// Actual Rust source type, qualified for the generated crate.
    pub fn rust_type(&self) -> &Type {
        &self.ty
    }
    /// Real source field names and types, in schema order.
    pub fn fields(&self) -> impl ExactSizeIterator<Item = (&Ident, &Type)> {
        self.fields.iter().map(|(n, t)| (n, t))
    }
    /// Generate target-evaluated size, alignment and field offsets.
    pub fn layout(&self) -> TokenStream {
        let ty = &self.ty;
        let offsets = self
            .fields
            .iter()
            .map(|(f, _)| quote!(::core::mem::offset_of!(#ty,#f)));
        quote!([::core::mem::size_of::<#ty>(),::core::mem::align_of::<#ty>(),#(#offsets),*])
    }
    /// Require a complete external record measurement and generate target assertions.
    pub fn assertions(&self, measured: &MeasuredLayout) -> Result<TokenStream> {
        measured.validate()?;
        if measured.fields.len() != self.fields.len() {
            return Err("external record layout must include every field exactly once".into());
        }
        let ty = &self.ty;
        let size = measured.size;
        let alignment = measured.alignment;
        let mut checks = TokenStream::new();
        for (field, _) in &self.fields {
            let key = field.to_string();
            let key = key.trim_start_matches("r#");
            let offset = measured
                .fields
                .get(key)
                .ok_or_else(|| format!("missing external record field `{key}`"))?;
            checks.extend(quote!(assert!(::core::mem::offset_of!(#ty,#field)==#offset);));
        }
        Ok(
            quote!(const _:()={assert!(::core::mem::size_of::<#ty>()==#size);assert!(::core::mem::align_of::<#ty>()==#alignment);#checks};),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_fields_must_match_the_source_exactly() {
        let native = NativeRecord::new(
            syn::parse_quote!(source::Row),
            [
                (syn::parse_quote!(r#type), syn::parse_quote!(u64)),
                (syn::parse_quote!(tag), syn::parse_quote!(u8)),
            ],
        )
        .unwrap();
        let mut layout = MeasuredLayout {
            size: 16,
            alignment: 8,
            fields: [("type".into(), 0), ("tag".into(), 8)].into(),
        };
        let checks = native.assertions(&layout).unwrap();
        assert!(checks.to_string().contains("offset_of"));
        let measured = native.layout().to_string();
        assert!(measured.contains("source :: Row"));
        layout.fields.remove("tag");
        assert!(native.assertions(&layout).is_err());
        layout.fields.insert("typo".into(), 8);
        assert!(native.assertions(&layout).unwrap_err().contains("tag"));
    }
}
