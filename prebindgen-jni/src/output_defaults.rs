//! Measured collection-output defaults, expressed as ordinary declaration trees.
use super::*;
use prebindgen_flat::WireNode;
use syn::visit_mut::VisitMut;

/// Discover source collection occurrences, including callback signatures and getter results.
fn collections(flat: &Bindings) -> BTreeMap<String, syn::Type> {
    struct Collections<'a>(&'a Bindings, BTreeMap<String, syn::Type>);
    impl VisitMut for Collections<'_> {
        fn visit_type_mut(&mut self, ty: &mut syn::Type) {
            if let Ok(resolved) = self.0.resolve_type(ty) {
                if prebindgen_flat::sequence_parts(&resolved).is_some() {
                    self.1.insert(type_key(&resolved), resolved);
                }
            }
            syn::visit_mut::visit_type_mut(self, ty);
        }
    }
    let mut found = Collections(flat, BTreeMap::new());
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
    found.1
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
            let bulk = Output::Bulk(Box::new(Output::Sequence {
                element: Box::new(Output::auto()),
                mode: None,
                delivery: prebindgen_declarations::SequenceDelivery::Value,
            }));
            // A default must never force a conversion that only the ordinary path
            // supports. Validate the native plan and the Kotlin element contract.
            let resolved = explicit
                .resolve_output(flat, &codec, &ty, &bulk, &())
                .ok()?;
            let plan =
                prebindgen_declarations::compile_output_value(flat, &codec, &ty, &resolved, &())
                    .ok()?;
            plan.kotlin(&KotlinContext {
                package: &flat.metadata.package,
                classes: &BTreeMap::new(),
                result: "result",
                failure: "failure",
                cleanup_failure: None,
                output_prefix: "probe",
            })
            .ok()?;
            Some(Output::for_type(ty, bulk))
        })
        .collect()
}

/// Resolve user-selected output trees before adding inferred defaults. In particular,
/// an implicit inner Vec default must not repack an explicitly ordinary outer Vec.
pub(crate) fn preserve_explicit(
    flat: &Bindings,
    explicit: &prebindgen_declarations::Declarations,
    outputs: &mut [Output<OutputRequest>],
    inputs: &mut [Input<InputRequest>],
    functions: &mut [FunctionDecl],
) -> Result<bool> {
    fn resolve(
        flat: &Bindings,
        explicit: &prebindgen_declarations::Declarations,
        ty: &syn::Type,
        tree: &Output<OutputRequest>,
    ) -> Result<Option<Output<OutputRequest>>> {
        if matches!(tree, Output::Request(OutputRequest::Auto)) {
            return Ok(None);
        }
        let ty = flat.resolve_type(ty)?;
        explicit
            .resolve_output(flat, &data::DataCodec::default(), &ty, tree, &())
            .map(|tree| Some(tree.into_preparation()))
    }
    fn callbacks(
        flat: &Bindings,
        explicit: &prebindgen_declarations::Declarations,
        tree: &mut Input<InputRequest>,
    ) -> Result<()> {
        match tree {
            Input::Request(InputRequest::Callback(declaration)) => {
                let signature = prebindgen_flat::callback_signature(&declaration.ty)?
                    .ok_or("callback declaration requires impl Fn(...) -> R")?;
                for (index, output) in &mut declaration.params {
                    if let Some(ty) = signature.arguments.get(*index) {
                        let ty = match ty {
                            syn::Type::Reference(r)
                                if r.mutability.is_none()
                                    && !matches!(r.elem.as_ref(), syn::Type::Slice(_))
                                    && !matches!(r.elem.as_ref(), syn::Type::Path(p) if p.path.is_ident("str")) =>
                            {
                                &r.elem
                            }
                            ty => ty,
                        };
                        if let Some(resolved) = resolve(flat, explicit, ty, output)? {
                            *output = resolved;
                        }
                    }
                }
                if let Some(returned) = &mut declaration.ret {
                    callbacks(flat, explicit, returned)?;
                }
            }
            Input::Request(InputRequest::ForType { input, .. })
            | Input::Bulk(input)
            | Input::Optional(input)
            | Input::Sequence(input) => callbacks(flat, explicit, input)?,
            Input::Record(children)
            | Input::Fields(children)
            | Input::Selector(children)
            | Input::Constructor { args: children, .. } => {
                for child in children {
                    callbacks(flat, explicit, child)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut can_infer = true;
    for tree in outputs {
        if let Output::Request(OutputRequest::ForType { ty, .. }) = tree {
            let ty = (**ty).clone();
            match resolve(flat, explicit, &ty, tree) {
                Ok(Some(resolved)) => *tree = Output::for_type(ty, resolved),
                Err(_) => can_infer = false,
                _ => {}
            }
        }
    }
    for tree in inputs {
        let mut prepared = tree.clone();
        if callbacks(flat, explicit, &mut prepared).is_ok() {
            *tree = prepared;
        } else {
            can_infer = false;
        }
    }
    for function in functions {
        if let Some(tree) = &mut function.ret {
            if let Ok(source) = flat.function(&function.name) {
                let ty = prebindgen_flat::result_parts(&source.ret.ty)
                    .map_or(&source.ret.ty, |(ok, _)| ok);
                if let Some(resolved) = resolve(flat, explicit, ty, tree)
                    .map_err(|e| format!("{}: {e}", function.name))?
                {
                    *tree = resolved;
                }
            }
        }
        for tree in function.params.values_mut() {
            callbacks(flat, explicit, tree).map_err(|e| format!("{}: {e}", function.name))?;
        }
    }
    // Invalid unused global declarations retain their delayed diagnostics, but
    // disable inference so their unresolved Auto nodes cannot acquire new defaults.
    Ok(can_infer)
}
