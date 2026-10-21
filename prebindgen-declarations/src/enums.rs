//! Checked integer representations shared by the C and JNI enum bindings.
use super::*;

/// An explicit mapping from fieldless source variants to signed 32-bit values.
///
/// Both adapters currently use this representation. Source discriminants must
/// be integer literals (optionally negative or parenthesized) or implicit
/// successors, and must fit `i32`. Expressions requiring Rust name resolution
/// are rejected rather than guessed. The source enum's memory layout is unused.
#[derive(Clone, Debug)]
pub struct EnumMapping {
    /// Qualified Rust enum type.
    pub ty: Type,
    /// Source names and their integer representations, in source order.
    pub variants: Vec<(syn::Ident, i32)>,
}
impl EnumMapping {
    /// Preserve the source discriminants in a checked `i32` mapping.
    pub fn new(source: &prebindgen_flat::FieldlessEnum) -> Result<Self> {
        fn integer(e: &syn::Expr) -> Result<i128> {
            match e {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(n),
                    ..
                }) => n.base10_parse().map_err(|e| e.to_string()),
                syn::Expr::Unary(syn::ExprUnary {
                    op: syn::UnOp::Neg(_),
                    expr,
                    ..
                }) => integer(expr)?
                    .checked_neg()
                    .ok_or_else(|| "enum discriminant overflow".into()),
                syn::Expr::Paren(e) => integer(&e.expr),
                _ => {
                    Err("enum discriminants must be integer literals or implicit successors".into())
                }
            }
        }
        let mut next = Some(0i32);
        let mut used = std::collections::BTreeSet::new();
        let mut variants = Vec::new();
        for variant in &source.variants {
            let value = match &variant.discriminant {
                Some(e) => i32::try_from(integer(e)?)
                    .map_err(|_| "enum discriminant does not fit the i32 binding representation")?,
                None => next.ok_or("implicit enum discriminant overflows i32")?,
            };
            if !used.insert(value) {
                return Err("duplicate enum discriminant".into());
            }
            next = value.checked_add(1);
            variants.push((variant.name.clone(), value));
        }
        if variants.is_empty() {
            return Err("enum mapping requires at least one variant".into());
        }
        let mapping = Self {
            ty: source.ty.clone(),
            variants,
        };
        mapping.validate()?;
        Ok(mapping)
    }
    /// Check public mapping data for empty or duplicate variants/discriminants.
    pub fn validate(&self) -> Result<()> {
        if self.variants.is_empty() {
            return Err("enum mapping requires at least one variant".into());
        }
        let mut names = std::collections::BTreeSet::new();
        let mut values = std::collections::BTreeSet::new();
        for (name, value) in &self.variants {
            if !names.insert(name.to_string().trim_start_matches("r#").to_string())
                || !values.insert(*value)
            {
                return Err("duplicate enum variant or discriminant".into());
            }
        }
        Ok(())
    }
    /// Whether the integer is outside the set of valid enum values.
    pub fn is_niche(&self, value: i64) -> bool {
        i32::try_from(value).is_ok_and(|n| !self.variants.iter().any(|(_, v)| *v == n))
    }
    /// First unused integer at or below `start`, stopping at `i32::MIN`.
    pub fn unused_at_or_below(&self, mut start: i32) -> Option<i32> {
        loop {
            if self.is_niche(start.into()) {
                return Some(start);
            }
            start = start.checked_sub(1)?;
        }
    }
    /// Next available negative sentinel, skipping valid negative discriminants.
    pub fn initial_niche(&self) -> Option<i32> {
        self.unused_at_or_below(-1)
    }
    /// Rust assertions ensuring that captured discriminants agree with the source.
    pub fn guards(&self) -> Vec<syn::ItemConst> {
        let ty = &self.ty;
        self.variants.iter().map(|(name, value)| syn::parse_quote! {
            const _: () = assert!(#ty::#name as i128 == #value as i128, "enum discriminant differs from captured source");
        }).collect()
    }
    /// Generate a checked integer-to-enum expression; never transmute an integer.
    pub fn decode(&self, value: TokenStream) -> TokenStream {
        let ty = &self.ty;
        let arms = self
            .variants
            .iter()
            .map(|(name, tag)| quote!(#tag => #ty::#name));
        quote!(match #value { #(#arms,)* _ => return Err("invalid enum discriminant".into()) })
    }
    /// Generate an exhaustive match, accepting owned or borrowed enum values.
    pub fn encode(&self, value: TokenStream) -> TokenStream {
        let ty = &self.ty;
        let arms = self
            .variants
            .iter()
            .map(|(name, tag)| quote!(#ty::#name => #tag));
        quote!(match #value { #(#arms,)* })
    }
}

/// Prepare checked enum decoding with the supplied boundary carrier.
/// References borrow the decoded local; mutable references are unsupported.
pub fn enum_input<W: prebindgen_flat::WireKind, C, G>(
    mapping: &EnumMapping,
    ty: &Type,
    wire: syn::Ident,
    var: syn::Ident,
    active: &Condition,
    kind: W,
) -> Result<InputPlan<W, C, G>> {
    let borrowed = match ty {
        Type::Reference(r) if r.mutability.is_some() => {
            return Err("mutable enum inputs are unsupported".into())
        }
        Type::Reference(_) => true,
        _ => false,
    };
    let decode = mapping.decode(quote!(#wire));
    let active = active.rust();
    Ok(InputPlan {
        context: Default::default(),
        acknowledgement: None,
        layout: vec![Wire { name: wire, kind }].into(),
        validation: quote!(let #var = if #active { Some(#decode) } else { None };),
        acquire: quote!(),
        resources: vec![],
        statements: if borrowed {
            quote!()
        } else {
            quote!(let #var = #var.expect("active enum");)
        },
        argument: if borrowed {
            quote!(#var.as_ref().expect("active enum"))
        } else {
            quote!(#var)
        },
    })
}
/// Prepare enum output by matching a borrow, without requiring `Copy` or `Clone`.
pub fn enum_output<W: prebindgen_flat::WireKind, C, G>(
    mapping: &EnumMapping,
    wire: syn::Ident,
    value: TokenStream,
    kind: W,
) -> OutputPlan<W, C, G> {
    let encoded = mapping.encode(quote!(&(#value)));
    OutputPlan {
        context: Default::default(),
        delivery: Default::default(),
        layout: vec![Wire {
            name: wire.clone(),
            kind,
        }]
        .into(),
        statements: quote!(let #wire: i32 = #encoded;),
        values: vec![quote!(#wire)],
        commit: quote!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mapping(source: &str) -> Result<EnumMapping> {
        let flat = Flat::<(), ()>::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )?;
        EnumMapping::new(flat.enums.values().next().unwrap())
    }
    #[test]
    fn discriminants_preserve_negative_hex_parenthesized_and_implicit_values() {
        let m =
            mapping("pub enum E { A = -2, B, C = (0x10), D, Min = -2147483648, Max = 2147483647 }")
                .unwrap();
        assert_eq!(
            m.variants.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
            [-2, -1, 16, 17, i32::MIN, i32::MAX]
        );
        assert_eq!(m.initial_niche(), Some(-3));
        assert_eq!(m.unused_at_or_below(17), Some(15));
        assert!(!m.is_niche(-1));
        assert!(!m.is_niche(i64::MAX));
        assert!(m.is_niche(0));
        assert_eq!(m.unused_at_or_below(i32::MIN), None);
    }
    #[test]
    fn invalid_and_unresolved_discriminants_fail_instead_of_guessing() {
        for (source, message) in [
            ("pub enum E { A = 2147483648 }", "does not fit"),
            ("pub enum E { A = -2147483649 }", "does not fit"),
            ("pub enum E { A = 2147483647, B }", "overflows"),
            ("pub enum E { A = 1, B = 1 }", "duplicate"),
            ("pub enum E { A = SOME_CONST }", "integer literals"),
            ("pub enum E { A = 1 << 3 }", "integer literals"),
        ] {
            assert!(mapping(source).unwrap_err().contains(message), "{source}");
        }
        let m = mapping("pub enum E { A = 2147483647, B = 0 }").unwrap();
        assert_eq!(m.variants[1].1, 0);
    }
}
