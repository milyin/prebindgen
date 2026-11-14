//! Source structure of data-carrying enum variants.
use super::*;
use quote::quote;

/// A Rust enum represented by a selected variant and recursively converted fields.
#[derive(Clone, Debug)]
pub struct PayloadEnum<V = (), F = ()> {
    /// Qualified source type, including the arguments of a standard Result.
    pub ty: Type,
    /// Source variants in wire-tag order, starting at zero.
    pub variants: Vec<PayloadVariant<V, F>>,
}

/// Source spelling used to construct or match an enum alternative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantStyle {
    /// `Variant`.
    Unit,
    /// `Variant(...)`, including an empty tuple constructor.
    Tuple,
    /// `Variant { ... }`, including an empty named constructor.
    Named,
}

/// One alternative of a data-carrying enum.
#[derive(Clone, Debug)]
pub struct PayloadVariant<V = (), F = ()> {
    /// Generator declaration metadata.
    pub metadata: V,
    /// Source variant name, independent of a foreign display name.
    pub name: syn::Ident,
    /// Source constructor delimiters.
    pub style: VariantStyle,
    /// Payload fields in source order.
    pub fields: Vec<PayloadField<F>>,
}

/// One named or positional enum payload member.
#[derive(Clone, Debug)]
pub struct PayloadField<F = ()> {
    /// Generator field metadata, as on record fields.
    pub metadata: F,
    /// Source member: a named field or tuple position.
    pub member: syn::Member,
    /// Qualified source type.
    pub ty: Type,
}
impl<F> PayloadField<F> {
    /// Source field key used by occurrence overrides.
    pub fn key(&self) -> String {
        match &self.member {
            syn::Member::Named(name) => name.to_string().trim_start_matches("r#").into(),
            syn::Member::Unnamed(index) => index.index.to_string(),
        }
    }
    /// Public default property name, using `v0`, `v1`, ... for tuple fields.
    pub fn property(&self) -> String {
        match &self.member {
            syn::Member::Named(_) => self.key(),
            syn::Member::Unnamed(index) => format!("v{}", index.index),
        }
    }
}
impl<V, F> PayloadEnum<V, F> {
    /// Spell a source constructor or pattern with the supplied field expressions.
    /// A standard Result uses its generic constructor path with inferred arguments.
    pub fn spell(
        &self,
        variant: usize,
        values: &[proc_macro2::TokenStream],
    ) -> Result<proc_macro2::TokenStream> {
        let arm = self
            .variants
            .get(variant)
            .ok_or("unknown payload variant")?;
        if arm.fields.len() != values.len() {
            return Err("wrong enum payload field count".into());
        }
        let Type::Path(source) = &self.ty else {
            return Err("enum source must be a path".into());
        };
        let mut path = source.path.clone();
        if result_parts(&self.ty).is_some() {
            path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;
        }
        let name = &arm.name;
        Ok(match arm.style {
            VariantStyle::Unit => quote!(#path::#name),
            VariantStyle::Tuple => quote!(#path::#name(#(#values),*)),
            VariantStyle::Named => {
                let members = arm.fields.iter().map(|f| &f.member);
                quote!(#path::#name { #(#members: #values),* })
            }
        })
    }
}

impl<M: Schema> Flat<M> {
    pub(crate) fn parse_payload(
        &self,
        ty: Type,
        source: syn::ItemEnum,
    ) -> Result<PayloadEnum<M::Variant, M::Field>> {
        if source.variants.is_empty() || i32::try_from(source.variants.len()).is_err() {
            return Err("payload enum needs 1..=i32::MAX variants".into());
        }
        let mut names = std::collections::BTreeSet::new();
        let variants = source
            .variants
            .into_iter()
            .map(|arm| {
                if !names.insert(arm.ident.to_string().trim_start_matches("r#").to_owned()) {
                    return Err("duplicate enum variant".into());
                }
                if arm.attrs.iter().any(|a| {
                    a.path().is_ident("cfg")
                        || a.path().is_ident("cfg_attr")
                        || a.path().is_ident("non_exhaustive")
                }) {
                    return Err(
                        "conditional or non-exhaustive enum variants are unsupported".into(),
                    );
                }
                let style = match &arm.fields {
                    syn::Fields::Unit => VariantStyle::Unit,
                    syn::Fields::Unnamed(_) => VariantStyle::Tuple,
                    syn::Fields::Named(_) => VariantStyle::Named,
                };
                let fields = arm
                    .fields
                    .into_iter()
                    .enumerate()
                    .map(|(index, field)| {
                        if field
                            .attrs
                            .iter()
                            .any(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
                        {
                            return Err("conditional enum payload fields are unsupported".into());
                        }
                        validate_input_lifetimes(&field.ty)?;
                        Ok(PayloadField {
                            metadata: Default::default(),
                            member: field
                                .ident
                                .map(syn::Member::Named)
                                .unwrap_or_else(|| syn::Member::Unnamed(syn::Index::from(index))),
                            ty: self.resolve_type(&field.ty)?,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(PayloadVariant {
                    metadata: Default::default(),
                    name: arm.ident,
                    style,
                    fields,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(PayloadEnum { ty, variants })
    }

    /// Look up a payload enum, including a concrete standard `Result<T, E>`.
    pub fn payload_enum(&self, ty: &Type) -> Result<&PayloadEnum<M::Variant, M::Field>> {
        self.payload_enums
            .get(&type_key(ty))
            .ok_or_else(|| format!("`{}` is not a payload enum", type_key(ty)))
    }

    pub(crate) fn register_result(&mut self, ty: &Type) {
        let Some((ok, error)) = result_parts(ty) else {
            return;
        };
        let key = type_key(ty);
        if self.payload_enums.contains_key(&key) {
            return;
        }
        self.types.entry(key.clone()).or_insert_with(|| TypeInfo {
            ty: ty.clone(),
            metadata: Default::default(),
            conversion: Default::default(),
        });
        let variants = [("Ok", ok), ("Err", error)]
            .into_iter()
            .map(|(name, child)| PayloadVariant {
                metadata: Default::default(),
                name: syn::Ident::new(name, proc_macro2::Span::call_site()),
                style: VariantStyle::Tuple,
                fields: vec![PayloadField {
                    metadata: Default::default(),
                    member: syn::Member::Unnamed(syn::Index::from(0)),
                    ty: child.clone(),
                }],
            })
            .collect();
        self.payload_enums.insert(
            key,
            PayloadEnum {
                ty: ty.clone(),
                variants,
            },
        );
    }

    pub(crate) fn register_results(&mut self) {
        struct Results(Vec<Type>);
        impl VisitMut for Results {
            fn visit_type_mut(&mut self, ty: &mut Type) {
                if result_parts(ty).is_some() {
                    self.0.push(ty.clone());
                }
                syn::visit_mut::visit_type_mut(self, ty);
            }
        }
        let mut roots = self
            .functions
            .values()
            .flat_map(|f| {
                f.params
                    .iter()
                    .map(|p| p.ty.ty.clone())
                    .chain(std::iter::once(f.ret.ty.clone()))
            })
            .collect::<Vec<_>>();
        roots.extend(
            self.records
                .values()
                .flat_map(|r| r.fields.iter().map(|f| f.ty.clone())),
        );
        roots.extend(self.payload_enums.values().flat_map(|e| {
            e.variants
                .iter()
                .flat_map(|v| v.fields.iter().map(|f| f.ty.clone()))
        }));
        let mut results = Results(vec![]);
        for mut ty in roots {
            results.visit_type_mut(&mut ty);
        }
        for ty in results.0 {
            self.register_result(&ty);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(source: &str) -> Result<Flat<()>> {
        Flat::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
    }
    #[test]
    fn payload_syntax_preserves_empty_constructor_shapes() {
        let flat = parse("pub enum Empty { Unit, Tuple(), Named{} }").unwrap();
        let source = flat.payload_enums.values().next().unwrap();
        assert_eq!(
            source.variants.iter().map(|v| v.style).collect::<Vec<_>>(),
            [VariantStyle::Unit, VariantStyle::Tuple, VariantStyle::Named]
        );
        assert_eq!(
            source.spell(1, &[]).unwrap().to_string(),
            "crate :: Empty :: Tuple ()"
        );
    }
    #[test]
    fn conditional_fields_and_generic_enums_are_diagnostics() {
        for source in [
            "pub enum E{ A{#[cfg(any())] value:i64} }",
            "pub enum E<T>{A(T)}",
            "#[non_exhaustive]pub enum E{A(i64)}",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
    }
    #[test]
    fn result_source_variants_do_not_change_function_error_routing() {
        let flat =
            parse("pub fn f(value:Option<Result<i64,String>>)->Result<(),String>{}").unwrap();
        let result = flat
            .resolve_type(&syn::parse_quote!(Result<i64,String>))
            .unwrap();
        assert!(flat.payload_enums.contains_key(&type_key(&result)));
        let f = flat.function("f").unwrap();
        assert!(result_parts(&f.ret.ty).is_some());
    }
}
