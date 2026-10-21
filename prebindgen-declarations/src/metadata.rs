//! Shared descriptions attached to parent nodes in a slot's wire tree.
use super::*;
use prebindgen_flat::WireNode;

/// Shared structure and optional destination-specific information for a parent node.
///
/// The structure describes how the children form a value. An annotation can add
/// destination information, such as the Kotlin class constructed from fields,
/// without replacing that structure. Physical wires still belong only to leaves.
#[derive(Clone, Debug)]
pub struct CompositionGroup<N, A> {
    /// Meaning of the ordered children, independent of the destination language.
    pub structure: GroupStructure<N>,
    /// Destination-specific information attached to this same structural node.
    pub annotation: Option<A>,
}
impl<N, A> CompositionGroup<N, A> {
    /// Describe a shared structure without destination-specific information.
    pub fn new(structure: GroupStructure<N>) -> Self {
        Self {
            structure,
            annotation: None,
        }
    }
    /// Attach destination-specific information while preserving the structure.
    pub fn annotated(mut self, annotation: A) -> Self {
        self.annotation = Some(annotation);
        self
    }
}
/// Meaning of a parent's children, produced by the shared declaration compiler.
#[derive(Clone, Debug)]
pub enum GroupStructure<N> {
    /// Optional payload and its presence rule.
    Optional(OptionalGroup<N>),
    /// Ordered named fields. Each immediate child must be a `Field` group.
    Fields,
    /// One source field; its children may contain zero, one or several wires.
    Field(syn::Ident),
    /// Adapter-defined structure. Requires an annotation describing its meaning.
    Custom,
}
/// How an optional node decides whether its child value is present.
#[derive(Clone, Debug)]
pub struct OptionalGroup<N> {
    /// Normalized 0/1 presence binding used by the prepared payload conditions.
    /// Unique within the slot. With a separate flag it equals `discriminator`;
    /// with a niche it may be a computed local rather than a physical wire.
    pub presence: syn::Ident,
    /// Name of a descendant wire to inspect: a flag or a payload wire with a niche.
    pub discriminator: syn::Ident,
    /// Reserved representation for `None`, or `None` when a separate 0/1 flag is used.
    /// Remaining available niches belong to the leaf's wire kind instead.
    pub niche: Option<N>,
}
/// Check physical wire names, optional bindings and references, and field structure.
/// Field collections contain uniquely named field children, including empty fields.
/// Custom structures require an adapter annotation describing their meaning.
pub fn validate_layout<W: prebindgen_flat::WireKind, N, G>(
    layout: &WireLayout<W, CompositionGroup<N, G>>,
) -> Result<()> {
    layout.validate()?;
    fn visit<W: prebindgen_flat::WireKind, N, G>(
        layout: &WireLayout<W, CompositionGroup<N, G>>,
        bindings: &mut std::collections::BTreeSet<String>,
        in_fields: bool,
    ) -> Result<()> {
        for node in layout.nodes() {
            if let WireNode::Group { metadata, children } = node {
                if let GroupStructure::Optional(o) = &metadata.structure {
                    if !bindings.insert(o.presence.to_string()) {
                        return Err(format!(
                            "duplicate optional presence binding `{}`",
                            o.presence
                        ));
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
                match &metadata.structure {
                    GroupStructure::Field(_) if !in_fields => {
                        return Err(
                            "named field must be a direct child of a field collection".into()
                        );
                    }
                    GroupStructure::Fields => {
                        let mut fields = std::collections::BTreeSet::new();
                        for child in children.nodes() {
                            let WireNode::Group { metadata, .. } = child else {
                                return Err("field collection requires named field children".into());
                            };
                            let GroupStructure::Field(name) = &metadata.structure else {
                                return Err("field collection requires named field children".into());
                            };
                            if !fields.insert(name.to_string()) {
                                return Err(format!("duplicate field `{name}`"));
                            }
                        }
                    }
                    GroupStructure::Custom if metadata.annotation.is_none() => {
                        return Err("custom structure requires an adapter annotation".into());
                    }
                    _ => {}
                }
                visit(
                    children,
                    bindings,
                    matches!(metadata.structure, GroupStructure::Fields),
                )?;
            }
        }
        Ok(())
    }
    visit(layout, &mut std::collections::BTreeSet::new(), false)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wire(name: &str) -> Wire<syn::Type> {
        Wire {
            name: syn::parse_str(name).unwrap(),
            kind: syn::parse_quote!(u32),
        }
    }
    fn optional(name: &str, discriminator: &str, niche: Option<()>) -> CompositionGroup<(), ()> {
        CompositionGroup::new(GroupStructure::Optional(OptionalGroup {
            presence: syn::parse_str(name).unwrap(),
            discriminator: syn::parse_str(discriminator).unwrap(),
            niche,
        }))
    }
    #[test]
    fn nested_options_share_one_leaf_but_keep_distinct_group_rules() {
        let mut layout: WireLayout<_, _> = vec![wire("value")].into();
        layout.enclose(optional("inner", "value", Some(())));
        layout.enclose(optional("outer", "value", Some(())));
        validate_layout(&layout).unwrap();
        assert_eq!(layout.len(), 1);
        assert!(layout
            .iter()
            .all(|w| w.name != "inner" && w.name != "outer"));
        layout.enclose(optional("wrong", "missing", Some(())));
        assert!(validate_layout(&layout)
            .unwrap_err()
            .contains("outside group"));
    }
    #[test]
    fn flags_are_direct_children_and_adapter_groups_need_no_identity() {
        let mut layout: WireLayout<_, _> = vec![wire("flag")].into();
        layout.enclose(CompositionGroup::new(GroupStructure::Custom).annotated(()));
        layout.enclose(optional("option", "flag", None));
        assert!(validate_layout(&layout)
            .unwrap_err()
            .contains("first child"));
        let mut layout: WireLayout<_, _> = vec![wire("flag")].into();
        layout.enclose(optional("option", "flag", None));
        validate_layout(&layout).unwrap(); // Option<()> has only its flag.
        layout.enclose(optional("option", "flag", Some(())));
        assert!(validate_layout(&layout)
            .unwrap_err()
            .contains("duplicate optional presence"));
        let mut empty: WireLayout<syn::Type, CompositionGroup<(), ()>> = WireLayout::default();
        empty.enclose(CompositionGroup::new(GroupStructure::Custom).annotated(()));
        empty.enclose(CompositionGroup::new(GroupStructure::Custom).annotated(()));
        validate_layout(&empty).unwrap();
        empty.enclose(optional("bad", "missing", Some(())));
        assert!(validate_layout(&empty).is_err());
    }
    #[test]
    fn shared_structures_accept_annotations_and_validate_field_children() {
        let mut field: WireLayout<_, CompositionGroup<(), &str>> = vec![wire("value")].into();
        field.enclose(CompositionGroup::new(GroupStructure::Field(
            syn::parse_quote!(x),
        )));
        field.enclose(CompositionGroup::new(GroupStructure::Fields).annotated("KotlinPoint"));
        validate_layout(&field).unwrap();
        field.enclose(
            CompositionGroup::new(GroupStructure::Optional(OptionalGroup {
                presence: syn::parse_quote!(present),
                discriminator: syn::parse_quote!(value),
                niche: Some(()),
            }))
            .annotated("nullable point"),
        );
        validate_layout(&field).unwrap();
        let mut invalid: WireLayout<_, CompositionGroup<(), ()>> = vec![wire("value")].into();
        invalid.enclose(CompositionGroup::new(GroupStructure::Fields));
        assert!(validate_layout(&invalid)
            .unwrap_err()
            .contains("named field children"));
        let mut a: WireLayout<syn::Type, CompositionGroup<(), ()>> = WireLayout::default();
        a.enclose(CompositionGroup::new(GroupStructure::Field(
            syn::parse_quote!(x),
        )));
        a.extend(a.clone());
        a.enclose(CompositionGroup::new(GroupStructure::Fields));
        assert!(validate_layout(&a).unwrap_err().contains("duplicate field"));
        let mut orphan: WireLayout<syn::Type, CompositionGroup<(), ()>> = WireLayout::default();
        orphan.enclose(CompositionGroup::new(GroupStructure::Field(
            syn::parse_quote!(orphan),
        )));
        assert!(validate_layout(&orphan)
            .unwrap_err()
            .contains("direct child"));
        let mut custom: WireLayout<syn::Type, CompositionGroup<(), ()>> = WireLayout::default();
        custom.enclose(CompositionGroup::new(GroupStructure::Custom));
        assert!(validate_layout(&custom)
            .unwrap_err()
            .contains("requires an adapter annotation"));
    }
}
