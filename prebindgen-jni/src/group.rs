//! JNI adapter-owned descriptions of the wire tree.
use super::*;
use prebindgen_flat::{WireLayout, WireNode};
use syn::ext::IdentExt;

/// Metadata interpreted by the JNI binding writer.
#[derive(Clone, Debug)]
pub enum JniGroup {
    /// Presence rule for the optional payload subtree.
    Optional(JniOptional),
    /// An ordered collection of field subtrees.
    /// `Some` constructs a Kotlin data class; `None` exposes expanded values.
    Fields(Option<DataGroup>),
    /// A named source field, including empty and multi-wire fields.
    Field(syn::Ident),
}
/// Optional-value information retained by the JNI adapter.
#[derive(Clone, Debug)]
pub struct JniOptional {
    /// Normalized presence binding used by payload conditions.
    pub presence: syn::Ident,
    /// Descendant wire inspected for presence.
    pub discriminator: syn::Ident,
    /// Reserved absent representation; `None` means a separate 0/1 flag.
    pub niche: Option<JniNiche>,
}

pub(crate) fn validate_layout(layout: &WireLayout<JniWire, JniGroup>) -> Result<()> {
    layout.validate()?;
    fn visit(
        layout: &WireLayout<JniWire, JniGroup>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
    ) -> Result<()> {
        for node in layout.nodes() {
            if let WireNode::Group { metadata, children } = node {
                if let JniGroup::Optional(o) = metadata {
                    if !bindings.insert(o.presence.to_string()) {
                        return Err(format!("duplicate Kotlin group binding `{}`", o.presence));
                    }
                    if !children.iter().any(|w| w.name == o.discriminator) {
                        return Err("optional discriminator outside group".into());
                    }
                    if o.niche.is_none()
                        && !matches!(children.nodes().first(),Some(WireNode::Wire(w)) if w.name==o.discriminator)
                    {
                        return Err("optional flag must be the first child wire".into());
                    }
                }
                if let JniGroup::Fields(Some(record)) = metadata {
                    if !bindings.insert(record.root.clone()) {
                        return Err(format!("duplicate Kotlin group binding `{}`", record.root));
                    }
                }
                match metadata {
                    JniGroup::Field(_) if !in_fields => {
                        return Err(
                            "named field must be a direct child of a field collection".into()
                        );
                    }
                    JniGroup::Fields(_) => {
                        let mut fields = std::collections::BTreeSet::new();
                        for child in children.nodes() {
                            let WireNode::Group { metadata, .. } = child else {
                                return Err("field collection requires named field children".into());
                            };
                            let JniGroup::Field(name) = metadata else {
                                return Err("field collection requires named field children".into());
                            };
                            if !fields.insert(name.unraw().to_string()) {
                                return Err(format!("duplicate field `{name}`"));
                            }
                        }
                    }
                    _ => {}
                }
                visit(children, bindings, matches!(metadata, JniGroup::Fields(_)))?;
            }
        }
        Ok(())
    }
    visit(layout, &mut std::collections::BTreeSet::new(), false)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn leaf(name: &str) -> prebindgen_flat::Wire<JniWire> {
        prebindgen_flat::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: JniWire::from(JniKind::Scalar(Primitive::Char)),
        }
    }
    fn optional(presence: &str, discriminator: &str, niche: bool) -> JniGroup {
        JniGroup::Optional(JniOptional {
            presence: syn::parse_str(presence).unwrap(),
            discriminator: syn::parse_str(discriminator).unwrap(),
            niche: niche.then_some(JniNiche::Integer(-1)),
        })
    }
    #[test]
    fn nested_optional_references_and_presence_bindings_are_validated() {
        let mut tree = WireLayout::from(vec![leaf("value")]);
        tree.enclose(optional("inner", "value", true));
        tree.enclose(optional("outer", "value", true));
        validate_layout(&tree).unwrap();
        assert_eq!(tree.len(), 1);
        let mut bad = tree.clone();
        bad.enclose(optional("missing", "elsewhere", true));
        assert!(validate_layout(&bad).unwrap_err().contains("outside group"));
        tree.enclose(optional("inner", "value", true));
        assert!(validate_layout(&tree).unwrap_err().contains("duplicate"));
        let mut flag = WireLayout::from(vec![leaf("flag")]);
        flag.enclose(optional("flag", "flag", false));
        validate_layout(&flag).unwrap();
        flag.enclose(optional("outer", "flag", false));
        assert!(validate_layout(&flag).unwrap_err().contains("first child"));
    }
    #[test]
    fn field_names_placement_and_empty_records_are_validated() {
        let mut field: WireLayout<JniWire, JniGroup> = WireLayout::default();
        field.enclose(JniGroup::Field(syn::parse_quote!(x)));
        assert!(validate_layout(&field)
            .unwrap_err()
            .contains("direct child"));
        let mut duplicates = field.clone();
        duplicates.extend(field.clone());
        duplicates.enclose(JniGroup::Fields(None));
        assert!(validate_layout(&duplicates)
            .unwrap_err()
            .contains("duplicate field"));
        field.enclose(JniGroup::Fields(None));
        validate_layout(&field).unwrap();
        let mut malformed = WireLayout::from(vec![leaf("value")]);
        malformed.enclose(JniGroup::Fields(None));
        assert!(validate_layout(&malformed)
            .unwrap_err()
            .contains("named field children"));
        let mut empty: WireLayout<JniWire, JniGroup> = WireLayout::default();
        empty.enclose(JniGroup::Fields(None));
        validate_layout(&empty).unwrap();
    }
    #[test]
    fn malformed_metadata_is_rejected_before_final_slots_are_installed() {
        let mut input = JniCodec
            .input_leaf(
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(wire),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            )
            .unwrap();
        input
            .layout
            .enclose(JniGroup::Field(syn::parse_quote!(orphan)));
        assert!(JniCodec
            .finish_input(&mut input, &())
            .unwrap_err()
            .contains("direct child"));
        let mut output = JniCodec
            .output_leaf(
                &syn::parse_quote!(i32),
                &(),
                syn::parse_quote!(wire),
                quote!(value),
                false,
            )
            .unwrap();
        output
            .layout
            .enclose(JniGroup::Field(syn::parse_quote!(orphan)));
        assert!(JniCodec
            .finish_output(&mut output, &())
            .unwrap_err()
            .contains("direct child"));
    }
}
