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
    for wire in layout.iter() {
        wire.kind.validate()?;
    }
    fn visit(
        layout: &WireLayout<JniWire, JniGroup>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
        reserved: &mut BTreeMap<String, Vec<JniNiche>>,
    ) -> Result<()> {
        for node in layout.nodes() {
            if let WireNode::Group { metadata, children } = node {
                if let JniGroup::Optional(o) = metadata {
                    if !bindings.insert(o.presence.to_string()) {
                        return Err(format!("duplicate Kotlin group binding `{}`", o.presence));
                    }
                    let wire = children
                        .iter()
                        .find(|w| w.name == o.discriminator)
                        .ok_or("optional discriminator outside group")?;
                    if let Some(niche) = &o.niche {
                        wire.kind.validate_niche(niche)?;
                        if wire.kind.available_niche().as_ref() == Some(niche) {
                            return Err(
                                "available JNI niche is already reserved by an optional group"
                                    .into(),
                            );
                        }
                        let niches = reserved.entry(o.discriminator.to_string()).or_default();
                        if niches.contains(niche) {
                            return Err("duplicate JNI niche reservation on one wire".into());
                        }
                        niches.push(niche.clone());
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
                visit(
                    children,
                    bindings,
                    matches!(metadata, JniGroup::Fields(_)),
                    reserved,
                )?;
            }
        }
        Ok(())
    }
    visit(
        layout,
        &mut std::collections::BTreeSet::new(),
        false,
        &mut BTreeMap::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn leaf(name: &str) -> prebindgen_flat::Wire<JniWire> {
        prebindgen_flat::Wire {
            name: syn::parse_str(name).unwrap(),
            kind: JniWire::scalar(Primitive::Char),
        }
    }
    fn optional(presence: &str, discriminator: &str, niche: Option<i64>) -> JniGroup {
        JniGroup::Optional(JniOptional {
            presence: syn::parse_str(presence).unwrap(),
            discriminator: syn::parse_str(discriminator).unwrap(),
            niche: niche.map(JniNiche::Integer),
        })
    }
    #[test]
    fn nested_optional_references_and_presence_bindings_are_validated() {
        let mut tree = WireLayout::from(vec![leaf("value")]);
        tree[0].kind.set_niche(Some(JniNiche::Integer(-3))).unwrap();
        tree.enclose(optional("inner", "value", Some(-1)));
        tree.enclose(optional("outer", "value", Some(-2)));
        validate_layout(&tree).unwrap();
        assert_eq!(tree.len(), 1);
        let mut bad = tree.clone();
        bad.enclose(optional("missing", "elsewhere", Some(-3)));
        assert!(validate_layout(&bad).unwrap_err().contains("outside group"));
        tree.enclose(optional("inner", "value", Some(-1)));
        assert!(validate_layout(&tree).unwrap_err().contains("duplicate"));
        let mut flag = WireLayout::from(vec![leaf("flag")]);
        flag.enclose(optional("flag", "flag", None));
        validate_layout(&flag).unwrap();
        flag.enclose(optional("outer", "flag", None));
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
    #[test]
    fn reserved_niches_are_checked_independently_of_remaining_availability() {
        let mut wire = JniWire::scalar(Primitive::Char);
        wire.clear_niche();
        let mut tree = WireLayout::from(vec![prebindgen_flat::Wire {
            name: syn::parse_quote!(value),
            kind: wire,
        }]);
        tree.enclose(optional("present", "value", Some(-1)));
        validate_layout(&tree).unwrap();
        tree[0].kind = JniWire::scalar(Primitive::I64);
        assert!(validate_layout(&tree).unwrap_err().contains("incompatible"));
        tree[0].kind = JniWire::Scalar {
            primitive: Primitive::Char,
            next_integer_niche: Some(0),
        };
        assert!(validate_layout(&tree).unwrap_err().contains("incompatible"));
    }
    #[test]
    fn incompatible_direct_wire_state_is_rejected_at_finalization() {
        let mut input = JniCodec
            .input_leaf(
                &syn::parse_quote!(i64),
                &(),
                syn::parse_quote!(wire),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            )
            .unwrap();
        input.layout[0].kind = JniWire::Scalar {
            primitive: Primitive::I64,
            next_integer_niche: Some(-1),
        };
        assert!(JniCodec
            .finish_input(&mut input, &())
            .unwrap_err()
            .contains("incompatible"));
        let mut output = JniCodec
            .output_leaf(
                &syn::parse_quote!(i64),
                &(),
                syn::parse_quote!(wire),
                quote!(value),
                false,
            )
            .unwrap();
        output.layout[0].kind = input.layout[0].kind.clone();
        assert!(JniCodec
            .finish_output(&mut output, &())
            .unwrap_err()
            .contains("incompatible"));
    }
    #[test]
    fn optional_reservations_cannot_overlap_each_other_or_available_state() {
        let mut tree = WireLayout::from(vec![leaf("value")]);
        tree.enclose(optional("inner", "value", Some(-1)));
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("already reserved"));
        tree[0].kind.set_niche(Some(JniNiche::Integer(-2))).unwrap();
        validate_layout(&tree).unwrap();
        tree.enclose(optional("outer", "value", Some(-1)));
        assert!(validate_layout(&tree)
            .unwrap_err()
            .contains("duplicate JNI niche"));
    }
}
