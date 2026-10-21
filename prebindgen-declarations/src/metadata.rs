//! Shared descriptions attached to parent nodes in a slot's wire tree.
use super::*;
use prebindgen_flat::WireNode;

/// How the children of a tree node form one value.
///
/// The node owns its children in `WireLayout`; this description contains only
/// the information needed to interpret them. For example, an optional integer
/// has a flag leaf and an integer leaf, and its group names the flag to inspect.
#[derive(Clone, Debug)]
pub struct CompositionGroup<N, G> {
    /// Generated name for this value, unique within the slot.
    pub name: String,
    /// An optional value or a value described by the language adapter.
    pub kind: GroupKind<N, G>,
}
/// Interpretation of a group's child values.
#[derive(Clone, Debug)]
pub enum GroupKind<N, G> {
    /// Describes how to distinguish `None` from `Some`.
    Optional(OptionalGroup<N>),
    /// Adapter information, such as the Kotlin class constructed from fields.
    Adapter(G),
}
/// How an optional node decides whether its child value is present.
#[derive(Clone, Debug)]
pub struct OptionalGroup<N> {
    /// Name of a descendant wire to inspect: a flag or a payload wire with a niche.
    pub discriminator: syn::Ident,
    /// Reserved representation for `None`, or `None` when a separate 0/1 flag is used.
    /// Remaining available niches belong to the leaf's wire kind instead.
    pub niche: Option<N>,
}
/// Check shared group references as well as the tree's unique physical wire names.
/// Empty adapter groups are allowed; an optional group needs a discriminator wire.
pub fn validate_layout<W: prebindgen_flat::WireKind, N, G>(
    layout: &WireLayout<W, CompositionGroup<N, G>>,
) -> Result<()> {
    layout.validate()?;
    fn visit<W: prebindgen_flat::WireKind, N, G>(
        layout: &WireLayout<W, CompositionGroup<N, G>>,
        names: &mut std::collections::BTreeSet<String>,
    ) -> Result<()> {
        for node in layout.nodes() {
            if let WireNode::Group { metadata, children } = node {
                if !names.insert(metadata.name.clone()) {
                    return Err(format!("duplicate group name `{}`", metadata.name));
                }
                if let GroupKind::Optional(o) = &metadata.kind {
                    if !children.iter().any(|w| w.name == o.discriminator) {
                        return Err("optional discriminator outside group".into());
                    }
                    if o.niche.is_none()
                        && !matches!(children.nodes().first(),Some(WireNode::Wire(w)) if w.name==o.discriminator)
                    {
                        return Err("optional flag must be the first child wire".into());
                    }
                }
                visit(children, names)?;
            }
        }
        Ok(())
    }
    visit(layout, &mut std::collections::BTreeSet::new())
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
        CompositionGroup {
            name: name.into(),
            kind: GroupKind::Optional(OptionalGroup {
                discriminator: syn::parse_str(discriminator).unwrap(),
                niche,
            }),
        }
    }
    #[test]
    fn nested_options_share_one_leaf_but_keep_distinct_group_rules() {
        let mut layout: WireLayout<_, _> = vec![wire("value")].into();
        layout.enclose(optional("inner", "value", Some(())));
        layout.enclose(optional("outer", "value", Some(())));
        validate_layout(&layout).unwrap();
        assert_eq!(layout.len(), 1);
        layout.enclose(optional("wrong", "missing", Some(())));
        assert!(validate_layout(&layout)
            .unwrap_err()
            .contains("outside group"));
    }
    #[test]
    fn flags_are_direct_children_and_group_names_are_unique() {
        let mut layout: WireLayout<_, _> = vec![wire("flag")].into();
        layout.enclose(CompositionGroup {
            name: "record".into(),
            kind: GroupKind::Adapter(()),
        });
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
            .contains("duplicate group"));
        let mut empty: WireLayout<syn::Type, CompositionGroup<(), ()>> = WireLayout::default();
        empty.enclose(CompositionGroup {
            name: "unit".into(),
            kind: GroupKind::Adapter(()),
        });
        validate_layout(&empty).unwrap();
        empty.enclose(optional("bad", "missing", Some(())));
        assert!(validate_layout(&empty).is_err());
    }
}
