//! Adapter-supplied views of native boundary scopes.
use prebindgen_flat::{Conversion, Function, Result, Wire, WireKind, WireLayout, WireNode};
use proc_macro2::TokenStream;

/// Reads one native boundary scope from the authoritative value tree.
///
/// The adapter chooses which leaves and metadata values participate, their order,
/// and which children belong to another scope. Shared conversion code consumes
/// the resulting borrowed lists without prescribing packing or delivery rules.
/// Readers may expose several values for one node, including pointer/length pairs.
pub trait NativeReader<W: WireKind, G> {
    /// Physical values in the requested scope, borrowed from these nodes.
    /// Readers decide how to select output outcomes and interpret metadata.
    fn wires(nodes: &[WireNode<W, G>]) -> std::vec::IntoIter<&Wire<W>>;
    /// Edit those same values in the same order. Other scopes remain untouched.
    fn wires_mut(nodes: &mut [WireNode<W, G>]) -> std::vec::IntoIter<&mut Wire<W>>;
    /// Validate the schema and the adapter's boundary rules before installation.
    fn validate(layout: &WireLayout<W, G>) -> Result<()>;
}

/// Check one adapter-selected namespace. The caller decides which values share
/// a namespace; this helper knows nothing about tree shape or ABI placement.
pub fn validate_wire_names<'a, W: WireKind + 'a>(
    wires: impl IntoIterator<Item = &'a Wire<W>>,
) -> Result<()> {
    let mut names = std::collections::BTreeSet::new();
    for wire in wires {
        if !names.insert(wire.name.to_string()) {
            return Err(format!("duplicate wire identifier `{}`", wire.name));
        }
    }
    Ok(())
}

// The default plan reader is for direct Rust values without adapter metadata.
// It has no collection transport. Adapters select their own reader explicitly.
impl<W: WireKind> NativeReader<W, ()> for () {
    fn wires(nodes: &[WireNode<W, ()>]) -> std::vec::IntoIter<&Wire<W>> {
        fn visit<'a, W: WireKind>(nodes: &'a [WireNode<W, ()>], out: &mut Vec<&'a Wire<W>>) {
            for node in nodes {
                match node {
                    WireNode::Wire(w) => out.push(w),
                    WireNode::Group { children, .. } => visit(children.nodes(), out),
                    WireNode::Outcomes { alternatives } => {
                        if let Some(o) = alternatives
                            .iter()
                            .find(|o| o.kind == prebindgen_flat::OutcomeKind::Success)
                        {
                            visit(o.layout.nodes(), out);
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut out = vec![];
        visit(nodes, &mut out);
        out.into_iter()
    }
    fn wires_mut(nodes: &mut [WireNode<W, ()>]) -> std::vec::IntoIter<&mut Wire<W>> {
        fn visit<'a, W: WireKind>(
            nodes: &'a mut [WireNode<W, ()>],
            out: &mut Vec<&'a mut Wire<W>>,
        ) {
            for node in nodes {
                match node {
                    WireNode::Wire(w) => out.push(w),
                    WireNode::Group { children, .. } => visit(children.nodes_mut(), out),
                    WireNode::Outcomes { alternatives } => {
                        if let Some(o) = alternatives
                            .iter_mut()
                            .find(|o| o.kind == prebindgen_flat::OutcomeKind::Success)
                        {
                            visit(o.layout.nodes_mut(), out);
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut out = vec![];
        visit(nodes, &mut out);
        out.into_iter()
    }
    fn validate(layout: &WireLayout<W, ()>) -> Result<()> {
        layout.validate()?;
        if let [WireNode::Outcomes { alternatives }] = layout.nodes() {
            for outcome in alternatives {
                Self::validate(&outcome.layout)?;
            }
            return Ok(());
        }
        fn visit<W: WireKind>(nodes: &[WireNode<W, ()>]) -> Result<()> {
            for node in nodes {
                match node {
                    WireNode::Wire(_) => {}
                    WireNode::Group { children, .. } => visit(children.nodes())?,
                    WireNode::Outcomes { alternatives } => {
                        for o in alternatives {
                            visit(o.layout.nodes())?;
                        }
                    }
                    _ => return Err("the direct Rust reader requires a value or group".into()),
                }
            }
            Ok(())
        }
        visit(layout.nodes())?;
        validate_wire_names(Self::wires(layout.nodes()))
    }
}

/// Physical-value expression requests for the adapter-selected native reader.
pub trait NativeConversion: Conversion {
    /// Reader used by this conversion; metadata has no required ABI interface.
    type Reader: NativeReader<Self::Wire, Self::Group>;
    /// Reader-specific requests concerning a physical value.
    type WireFragment<'a>;
    /// Emit an expression at this reader's boundary position.
    fn code_fragment_wire(
        &self,
        pos: usize,
        mode: Self::WireFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream>;
}
/// Native input enumeration for a function's selected readers.
pub trait NativeFunction {
    /// Adapter's value description.
    type Wire: WireKind;
    /// Source-slot values; the adapter adds any invocation infrastructure.
    fn native_inputs(&self) -> Result<std::vec::IntoIter<&Wire<Self::Wire>>>;
}
impl<I: NativeConversion, O: Conversion> NativeFunction for Function<I, O> {
    type Wire = I::Wire;
    fn native_inputs(&self) -> Result<std::vec::IntoIter<&Wire<I::Wire>>> {
        Ok(self
            .input_conversions()?
            .into_iter()
            .flat_map(|i| I::Reader::wires(i.layout().nodes()))
            .collect::<Vec<_>>()
            .into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direct_reader_validates_every_outcome_namespace() {
        let wire = || Wire {
            name: syn::parse_quote!(value),
            kind: syn::parse_quote!(i32),
        };
        let mut layout: WireLayout<syn::Type> = vec![wire()].into();
        layout.set_outcome(
            prebindgen_flat::OutcomeKind::SourceError,
            vec![wire()].into(),
        );
        layout.set_outcome(
            prebindgen_flat::OutcomeKind::BindingFailure,
            vec![wire(), wire()].into(),
        );
        assert!(<()>::validate(&layout)
            .unwrap_err()
            .contains("duplicate wire"));
        layout.set_outcome(
            prebindgen_flat::OutcomeKind::BindingFailure,
            vec![wire()].into(),
        );
        <()>::validate(&layout).unwrap();
    }
}
