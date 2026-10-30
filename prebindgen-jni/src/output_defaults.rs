//! Measured collection-output defaults, expressed as ordinary declaration trees.
use super::*;
use prebindgen_flat::WireNode;
use syn::visit_mut::VisitMut;

/// Discover source collection occurrences, including callback signatures and getter results.
fn collections(flat: &Bindings) -> BTreeMap<String, syn::Type> {
    struct Collections(BTreeMap<String, syn::Type>);
    impl VisitMut for Collections {
        fn visit_type_mut(&mut self, ty: &mut syn::Type) {
            if prebindgen_flat::sequence_parts(ty).is_some() {
                self.0.insert(type_key(ty), ty.clone());
            }
            syn::visit_mut::visit_type_mut(self, ty);
        }
    }
    let mut found = Collections(BTreeMap::new());
    for function in flat.functions.values() {
        found.visit_type_mut(&mut function.ret.ty.clone());
        for parameter in &function.params {
            found.visit_type_mut(&mut parameter.ty.ty.clone());
        }
    }
    for record in flat.records.values() {
        for field in &record.fields {
            found.visit_type_mut(&mut field.ty.clone());
        }
    }
    found.0
}

/// Only fixed-width scalar rows, matching the measured transport shape, get a new default.
/// Strings, byte payloads, handles and custom protocols retain their existing policy.
fn scalar_rows(node: &kotlin::JniNode) -> bool {
    fn scalar(wire: &prebindgen_flat::Wire<JniWire>) -> bool {
        matches!(wire.kind, JniWire::Scalar { .. } | JniWire::Selector { .. })
            && wire.kind.bulk_scalar_access().is_ok()
    }
    match node {
        WireNode::Leaf(fields) => !fields.is_empty() && fields.iter().all(scalar),
        WireNode::Branch {
            controls, children, ..
        } => {
            !children.is_empty() && controls.iter().all(scalar) && children.iter().all(scalar_rows)
        }
        WireNode::Chain {
            controls,
            child,
            meta: JniChain::Optional(_),
        } => controls.iter().all(scalar) && scalar_rows(child),
        _ => false,
    }
}

pub(crate) fn infer(
    flat: &Bindings,
    explicit: &prebindgen_declarations::Declarations,
    configured: &BTreeSet<String>,
) -> Vec<Output<OutputRequest>> {
    let codec = data::DataCodec::default();
    collections(flat)
        .into_iter()
        .filter_map(|(key, ty)| {
            if configured.contains(&key) {
                return None;
            }
            let (element, view) = prebindgen_flat::sequence_parts(&ty)?;
            if view == prebindgen_flat::SequenceView::Mutable || type_key(element) == "u8" {
                return None;
            }
            // Inspect the actual expanded boundary, not just the source record fields:
            // user getters/conversions can change a numeric record into text or handles.
            let recipe = explicit
                .resolve_output(flat, &codec, &ty, &Output::auto(), &())
                .ok()?;
            let plan =
                prebindgen_declarations::compile_output_value(flat, &codec, &ty, &recipe, &())
                    .ok()?;
            let WireNode::Chain {
                meta: JniChain::Sequence(JniSequence::Buffered),
                child,
                ..
            } = &plan.output_outcomes().success
            else {
                return None;
            };
            if !scalar_rows(child) {
                return None;
            }
            Some(Output::for_type(
                ty,
                Output::Bulk(Box::new(Output::Sequence {
                    element: Box::new(Output::auto()),
                    mode: None,
                    delivery: prebindgen_declarations::SequenceDelivery::Value,
                })),
            ))
        })
        .collect()
}
