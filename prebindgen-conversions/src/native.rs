//! Native views derived from the authoritative value tree and adapter metadata.
use prebindgen_flat::{
    Conversion, Function, OutcomeKind, Result, Wire, WireKind, WireLayout, WireNode,
};
use proc_macro2::TokenStream;

/// One node's explicit native encoding. This describes additional physical
/// values only; payload structure remains exclusively in the WireNode tree.
pub enum NativeEncoding<'a, W: WireKind> {
    /// Additional controls precede the node's payload in the current scope.
    Inline(&'a [Wire<W>]),
    /// One physical value carries the payload, whose fields have another scope.
    Carrier(&'a Wire<W>),
}
impl<'a, W: WireKind> NativeEncoding<'a, W> {
    /// Carrier owned by this metadata, if this node uses one.
    pub fn carrier(self) -> Option<&'a Wire<W>> {
        match self {
            NativeEncoding::Carrier(w) => Some(w),
            _ => None,
        }
    }
    /// Inline controls owned by this metadata, or None for a carrier.
    pub fn controls(self) -> Option<&'a [Wire<W>]> {
        match self {
            NativeEncoding::Inline(ws) => Some(ws),
            _ => None,
        }
    }
}
/// Mutable access to the additional values owned by one node's metadata.
pub enum NativeEncodingMut<'a, W: WireKind> {
    /// Controls in the current scope.
    Inline(&'a mut [Wire<W>]),
    /// Carrier in the current scope.
    Carrier(&'a mut Wire<W>),
}
/// Adapter metadata interpreted by the optional staged native reader. Common
/// Conversion does not require this trait or prescribe any native encoding.
pub trait NativeMetadata<W: WireKind> {
    /// Explicit encoding at this node; field/record metadata has no controls.
    fn encoding(&self) -> NativeEncoding<'_, W>;
    /// Edit those same physical values without copying their descriptions.
    fn encoding_mut(&mut self) -> NativeEncodingMut<'_, W>;
}
impl<W: WireKind> NativeMetadata<W> for () {
    fn encoding(&self) -> NativeEncoding<'_, W> {
        NativeEncoding::Inline(&[])
    }
    fn encoding_mut(&mut self) -> NativeEncodingMut<'_, W> {
        NativeEncodingMut::Inline(&mut [])
    }
}
impl<W: WireKind> NativeMetadata<W> for Vec<Wire<W>> {
    fn encoding(&self) -> NativeEncoding<'_, W> {
        NativeEncoding::Inline(self)
    }
    fn encoding_mut(&mut self) -> NativeEncodingMut<'_, W> {
        NativeEncodingMut::Inline(self)
    }
}

/// Borrowed native parameter views. No second hierarchy or payload storage is
/// constructed: each returned value belongs to a leaf or to adapter metadata.
pub trait NativeLayoutExt {
    /// Adapter's individual value description.
    type Wire: WireKind;
    /// Current scope's physical values in the reader's chosen order. At an
    /// Outcomes root this selects Success; error payloads are read explicitly.
    fn native_wires(&self) -> std::vec::IntoIter<&Wire<Self::Wire>>;
    /// Mutable view of those same values, excluding carried element scopes.
    fn native_wires_mut(&mut self) -> std::vec::IntoIter<&mut Wire<Self::Wire>>;
    /// Number of physical values in this scope, derived from the tree.
    fn native_len(&self) -> usize {
        self.native_wires().len()
    }
    /// Whether the current scope has no physical values.
    fn native_is_empty(&self) -> bool {
        self.native_len() == 0
    }
    /// One physical value by position in this reader's derived view.
    fn native_get(&self, pos: usize) -> Option<&Wire<Self::Wire>> {
        self.native_wires().nth(pos)
    }
    /// Edit one physical value by its derived position.
    fn native_get_mut(&mut self, pos: usize) -> Option<&mut Wire<Self::Wire>> {
        self.native_wires_mut().nth(pos)
    }
    /// Check physical names, including carried elements, separately for each
    /// outcome. Alternative outcomes may reuse names because they never coexist.
    fn validate_native_names(&self) -> Result<()>;
}
impl<W: WireKind, G: NativeMetadata<W>> NativeLayoutExt for WireLayout<W, G> {
    type Wire = W;
    fn native_wires(&self) -> std::vec::IntoIter<&Wire<W>> {
        native_node_wires(self.nodes())
    }
    fn native_wires_mut(&mut self) -> std::vec::IntoIter<&mut Wire<W>> {
        fn walk<'a, W: WireKind, G: NativeMetadata<W>>(
            layout: &'a mut WireLayout<W, G>,
            out: &mut Vec<&'a mut Wire<W>>,
        ) {
            for node in layout.nodes_mut() {
                match node {
                    WireNode::Wire(w) => out.push(w),
                    WireNode::Group { children, .. } => walk(children, out),
                    WireNode::Optional(c) | WireNode::Sequence(c) => {
                        match c.metadata.as_mut().map(NativeMetadata::encoding_mut) {
                            Some(NativeEncodingMut::Carrier(w)) => out.push(w),
                            encoding => {
                                if let Some(NativeEncodingMut::Inline(controls)) = encoding {
                                    out.extend(controls);
                                }
                                walk(&mut c.element, out);
                            }
                        }
                    }
                    WireNode::Alternatives {
                        metadata,
                        alternatives,
                    } => {
                        if let Some(metadata) = metadata {
                            match metadata.encoding_mut() {
                                NativeEncodingMut::Inline(controls) => out.extend(controls),
                                NativeEncodingMut::Carrier(w) => {
                                    out.push(w);
                                    continue;
                                }
                            }
                        }
                        for arm in alternatives {
                            walk(arm, out);
                        }
                    }
                    WireNode::Outcomes { alternatives } => {
                        if let Some(success) = alternatives
                            .iter_mut()
                            .find(|o| o.kind == OutcomeKind::Success)
                        {
                            walk(&mut success.layout, out);
                        }
                    }
                }
            }
        }
        let mut out = vec![];
        walk(self, &mut out);
        out.into_iter()
    }
    fn validate_native_names(&self) -> Result<()> {
        fn insert<W: WireKind>(
            w: &Wire<W>,
            names: &mut std::collections::BTreeSet<String>,
        ) -> Result<()> {
            if !names.insert(w.name.to_string()) {
                return Err(format!("duplicate wire identifier `{}`", w.name));
            }
            Ok(())
        }
        fn walk<W: WireKind, G: NativeMetadata<W>>(
            layout: &WireLayout<W, G>,
            names: &mut std::collections::BTreeSet<String>,
        ) -> Result<()> {
            for node in layout.nodes() {
                match node {
                    WireNode::Wire(w) => insert(w, names)?,
                    WireNode::Group { metadata, children } => {
                        if !matches!(metadata.encoding(), NativeEncoding::Inline([])) {
                            return Err(
                                "group metadata cannot carry native controls or a carrier".into()
                            );
                        }
                        walk(children, names)?;
                    }
                    WireNode::Optional(c) | WireNode::Sequence(c) => {
                        if let Some(m) = &c.metadata {
                            match m.encoding() {
                                NativeEncoding::Carrier(w) => insert(w, names)?,
                                NativeEncoding::Inline(ws) => {
                                    for w in ws {
                                        insert(w, names)?;
                                    }
                                }
                            }
                        }
                        walk(&c.element, names)?;
                    }
                    WireNode::Alternatives {
                        metadata,
                        alternatives,
                    } => {
                        if let Some(m) = metadata {
                            match m.encoding() {
                                NativeEncoding::Carrier(w) => insert(w, names)?,
                                NativeEncoding::Inline(ws) => {
                                    for w in ws {
                                        insert(w, names)?;
                                    }
                                }
                            }
                        }
                        for arm in alternatives {
                            walk(arm, names)?;
                        }
                    }
                    WireNode::Outcomes { alternatives } => {
                        for arm in alternatives {
                            walk(&arm.layout, &mut Default::default())?;
                        }
                    }
                }
            }
            Ok(())
        }
        self.validate()?;
        walk(self, &mut Default::default())
    }
}

/// Physical values borrowed from logical child nodes and their encoding metadata.
pub fn native_node_wires<W: WireKind, G: NativeMetadata<W>>(
    nodes: &[WireNode<W, G>],
) -> std::vec::IntoIter<&Wire<W>> {
    fn walk<'a, W: WireKind, G: NativeMetadata<W>>(
        nodes: &'a [WireNode<W, G>],
        out: &mut Vec<&'a Wire<W>>,
    ) {
        for node in nodes {
            match node {
                WireNode::Wire(w) => out.push(w),
                WireNode::Group { children, .. } => walk(children.nodes(), out),
                WireNode::Optional(c) | WireNode::Sequence(c) => {
                    match c.metadata.as_ref().map(NativeMetadata::encoding) {
                        Some(NativeEncoding::Carrier(w)) => out.push(w),
                        encoding => {
                            if let Some(NativeEncoding::Inline(controls)) = encoding {
                                out.extend(controls);
                            }
                            walk(c.element.nodes(), out);
                        }
                    }
                }
                WireNode::Alternatives {
                    metadata,
                    alternatives,
                } => {
                    if let Some(metadata) = metadata {
                        match metadata.encoding() {
                            NativeEncoding::Inline(controls) => out.extend(controls),
                            NativeEncoding::Carrier(w) => {
                                out.push(w);
                                continue;
                            }
                        }
                    }
                    for arm in alternatives {
                        walk(arm.nodes(), out);
                    }
                }
                WireNode::Outcomes { alternatives } => {
                    if let Some(success) =
                        alternatives.iter().find(|o| o.kind == OutcomeKind::Success)
                    {
                        walk(success.layout.nodes(), out);
                    }
                }
            }
        }
    }
    let mut out = vec![];
    walk(nodes, &mut out);
    out.into_iter()
}

/// Opt-in requests for physical values derived from the conversion's value tree.
pub trait NativeConversion: Conversion<Group: NativeMetadata<Self::Wire>> {
    /// Reader-specific requests concerning a physical value.
    type WireFragment<'a>;
    /// Emit a physical-value expression in the derived native index domain.
    fn code_fragment_wire(
        &self,
        pos: usize,
        mode: Self::WireFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream>;
}
/// Native input enumeration for the optional staged reader.
pub trait NativeFunction {
    /// Adapter's value description.
    type Wire: WireKind;
    /// Source slots lowered to native parameters. Reader infrastructure may add
    /// callbacks and acknowledgements separately.
    fn native_inputs(&self) -> Result<std::vec::IntoIter<&Wire<Self::Wire>>>;
}
impl<I: NativeConversion, O: Conversion> NativeFunction for Function<I, O> {
    type Wire = I::Wire;
    fn native_inputs(&self) -> Result<std::vec::IntoIter<&Wire<I::Wire>>> {
        Ok(self
            .input_conversions()?
            .into_iter()
            .flat_map(|i| i.layout().native_wires())
            .collect::<Vec<_>>()
            .into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Neither wire nor metadata needs Clone: native views borrow the only tree.
    struct Kind(syn::Type);
    impl WireKind for Kind {
        fn rust_type(&self) -> syn::Type {
            self.0.clone()
        }
    }
    enum Encoding {
        Inline(Vec<Wire<Kind>>),
        Carrier(Box<Wire<Kind>>),
        Record,
    }
    impl NativeMetadata<Kind> for Encoding {
        fn encoding(&self) -> NativeEncoding<'_, Kind> {
            match self {
                Self::Inline(ws) => NativeEncoding::Inline(ws),
                Self::Carrier(w) => NativeEncoding::Carrier(w),
                Self::Record => NativeEncoding::Inline(&[]),
            }
        }
        fn encoding_mut(&mut self) -> NativeEncodingMut<'_, Kind> {
            match self {
                Self::Inline(ws) => NativeEncodingMut::Inline(ws),
                Self::Carrier(w) => NativeEncodingMut::Carrier(w),
                Self::Record => NativeEncodingMut::Inline(&mut []),
            }
        }
    }
    type Layout = WireLayout<Kind, Encoding>;
    fn leaf(name: &str) -> Wire<Kind> {
        Wire {
            name: syn::parse_str(name).unwrap(),
            kind: Kind(syn::parse_quote!(i64)),
        }
    }
    fn names(layout: &Layout) -> Vec<String> {
        layout.native_wires().map(|w| w.name.to_string()).collect()
    }

    #[test]
    fn native_views_borrow_payloads_and_metadata_from_one_tree() {
        let mut element: Layout = vec![leaf("x"), leaf("y")].into();
        element.enclose_optional(Some(Encoding::Inline(vec![leaf("item_present")])));
        element.enclose_sequence(Some(Encoding::Carrier(leaf("rows").into())));
        element.enclose_optional(Some(Encoding::Inline(vec![leaf("list_present")])));
        element.enclose(Encoding::Record);
        let mut layout: Layout = vec![leaf("before")].into();
        layout.extend(element);
        layout.push(leaf("after"));
        layout.validate_native_names().unwrap();
        assert_eq!(names(&layout), ["before", "list_present", "rows", "after"]);
        assert_eq!(
            layout
                .iter()
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["before", "x", "y", "after"]
        );
        assert!(std::ptr::eq(
            layout.get(0).unwrap(),
            layout.native_get(0).unwrap()
        ));
        assert!(std::ptr::eq(
            layout.get(3).unwrap(),
            layout.native_get(3).unwrap()
        ));
        for (pos, wire) in layout.native_wires().enumerate() {
            assert!(std::ptr::eq(wire, layout.native_get(pos).unwrap()));
        }
        assert!(layout.native_get(usize::MAX).is_none());
        for wire in layout.native_wires_mut() {
            wire.kind.0 = syn::parse_quote!(u64);
        }
        assert_eq!(prebindgen_flat::type_key(&layout[0].kind.0), "u64");
        // Carried fields are another scope, so mutating outer values excludes them.
        assert_eq!(prebindgen_flat::type_key(&layout[1].kind.0), "i64");
        assert_eq!(
            prebindgen_flat::type_key(&layout.native_get(2).unwrap().kind.0),
            "u64"
        );
    }

    #[test]
    fn changing_node_metadata_changes_the_native_view_without_refreshing() {
        let mut layout: Layout = vec![leaf("x"), leaf("y")].into();
        layout.enclose_sequence(Some(Encoding::Carrier(leaf("rows").into())));
        let addresses = layout
            .entries()
            .map(|entry| (entry.path, entry.wire as *const _ as usize))
            .collect::<Vec<_>>();
        assert_eq!(names(&layout), ["rows"]);
        let WireNode::Sequence(c) = &mut layout.nodes_mut()[0] else {
            panic!()
        };
        c.metadata = Some(Encoding::Inline(vec![leaf("count")]));
        assert_eq!(names(&layout), ["count", "x", "y"]);
        for (path, address) in addresses {
            assert_eq!(
                layout.get_path(&path).unwrap() as *const _ as usize,
                address
            );
        }
        layout.native_get_mut(0).unwrap().name = syn::parse_quote!(length);
        assert_eq!(names(&layout), ["length", "x", "y"]);
        assert_eq!(layout.len(), 2);
    }

    #[test]
    fn selector_controls_are_owned_by_alternative_metadata() {
        let mut layout = Layout::from_nodes(vec![WireNode::Alternatives {
            metadata: Some(Encoding::Inline(vec![leaf("tag")])),
            alternatives: vec![Layout::default(), vec![leaf("x"), leaf("y")].into()],
        }]);
        layout.validate_native_names().unwrap();
        assert_eq!(names(&layout), ["tag", "x", "y"]);
        assert_eq!(layout.len(), 2);
        assert_eq!(
            layout.entries().next().unwrap().path,
            prebindgen_flat::WirePath(vec![0, 1, 0])
        );
        for wire in layout.native_wires_mut() {
            wire.kind.0 = syn::parse_quote!(u64);
        }
        assert!(layout
            .iter()
            .all(|w| prebindgen_flat::type_key(&w.kind.0) == "u64"));
    }

    #[test]
    fn outcomes_have_explicit_separate_native_views_and_name_scopes() {
        let mut layout: Layout = vec![leaf("value")].into();
        let mut source_error: Layout = vec![leaf("value")].into();
        source_error.enclose_optional(Some(Encoding::Inline(vec![leaf("present")])));
        layout.set_outcome(OutcomeKind::SourceError, source_error);
        layout.set_outcome(OutcomeKind::BindingFailure, vec![leaf("message")].into());
        layout.validate_native_names().unwrap();
        assert_eq!(names(&layout), ["value"]);
        assert_eq!(
            names(layout.outcome(OutcomeKind::SourceError).unwrap()),
            ["present", "value"]
        );
        assert_eq!(layout.len(), 3);
        layout.native_get_mut(0).unwrap().name = syn::parse_quote!(success);
        assert_eq!(
            names(layout.outcome(OutcomeKind::SourceError).unwrap()),
            ["present", "value"]
        );
        layout
            .outcome(OutcomeKind::Success)
            .unwrap()
            .validate_native_names()
            .unwrap();
    }

    #[test]
    fn duplicate_names_are_checked_in_carried_payloads_and_controls() {
        let mut layout: Layout = vec![leaf("duplicate")].into();
        layout.enclose_sequence(Some(Encoding::Carrier(leaf("duplicate").into())));
        assert!(layout
            .validate_native_names()
            .unwrap_err()
            .contains("duplicate wire"));
        let mut choice = Layout::from_nodes(vec![WireNode::Alternatives {
            metadata: Some(Encoding::Inline(vec![leaf("tag")])),
            alternatives: vec![vec![leaf("tag")].into()],
        }]);
        assert!(choice.validate_native_names().is_err());
        choice.native_get_mut(1).unwrap().name = syn::parse_quote!(payload);
        choice.validate_native_names().unwrap();
    }
    #[test]
    fn group_metadata_cannot_hide_unread_native_values() {
        for metadata in [
            Encoding::Inline(vec![leaf("hidden")]),
            Encoding::Carrier(leaf("hidden").into()),
        ] {
            let mut layout: Layout = vec![leaf("payload")].into();
            layout.enclose(metadata);
            assert!(layout
                .validate_native_names()
                .unwrap_err()
                .contains("group metadata cannot carry"));
        }
        let mut layout: Layout = vec![leaf("payload")].into();
        layout.enclose(Encoding::Inline(vec![]));
        layout.validate_native_names().unwrap();
    }
}
