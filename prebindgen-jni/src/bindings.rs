//! Configure, resolve and compile exports owned by their original source function.
use super::*;
use prebindgen_conversions::BoundFunction;
use prebindgen_declarations::{BindingState, ConfiguredCall, OutputNode};

pub(crate) fn views<I, O>(
    flat: &Flat<JniSchema<I, O>>,
) -> Result<Vec<BoundFunction<'_, I, O, JniFunction>>> {
    flat.functions
        .values()
        .flat_map(|source| {
            source
                .metadata
                .iter()
                .map(move |binding| binding.prepared(source))
        })
        .collect()
}

pub(crate) fn install(
    flat: &mut JniBindings,
    mut decl: DeclJniFunction,
    metadata: Option<JniFunction>,
) -> Result<()> {
    let source = decl.name.clone();
    let mode = decl.callback_return.take();
    let base = source.clone();
    let mut metadata = metadata.unwrap_or_else(|| JniFunction {
        package: flat.metadata.package.clone(),
        name: decl.foreign.clone().unwrap_or_else(|| camel(&source)),
        order: 0,
        ..Default::default()
    });
    let inherited = flat
        .function(&source)?
        .metadata
        .iter()
        .find(|b| {
            (b.metadata.package == metadata.package && b.metadata.name == metadata.name)
                || b.metadata.package.is_empty()
        })
        .map(|b| b.state.clone());
    if mode.is_some()
        && decl.ret.is_none()
        && inherited.as_ref().is_some_and(|s| match s {
            BindingState::Prepared(_) => true,
            BindingState::Configured(c) => matches!(
                c.output,
                prebindgen_declarations::ConfiguredSlot::Prepared(_)
            ),
            _ => false,
        })
    {
        return Err("callback_return cannot reconstruct a prebound output; supply an explicit output override".into());
    }
    let inherited_inputs = inherited.map(|s| match s {
        BindingState::Prepared(c) => c
            .inputs
            .into_iter()
            .map(prebindgen_declarations::ConfiguredSlot::Prepared)
            .collect(),
        BindingState::Configured(c) => c.inputs,
        BindingState::Resolved(_) => Vec::new(),
    });
    if mode == Some(prebindgen_declarations::CallbackReturn::Only) {
        flat.function_mut(&source)?.metadata.retain(|b| {
            !((b.metadata.package == metadata.package && b.metadata.name == metadata.name)
                || b.metadata.package.is_empty())
        });
    }
    if mode != Some(prebindgen_declarations::CallbackReturn::Only) {
        metadata.callback_return = false;
        install_one(
            flat,
            &source,
            base.clone(),
            metadata.clone(),
            decl.clone(),
            None,
        )?;
    }
    if mode.is_some() {
        metadata.callback_return = true;
        metadata.name.push_str("Fold");
        let structural = decl.ret.is_none();
        decl.ret = Some(OutputNode::Unary {
            operation: JniOutputUnary::ConfigSequenceCallback { structural },
            child: Box::new(decl.ret.take().unwrap_or_else(JniOutputConfig::auto)),
        });
        install_one(
            flat,
            &source,
            format!("{base}_fold"),
            metadata,
            decl,
            inherited_inputs,
        )?;
    }
    Ok(())
}
fn install_one(
    flat: &mut JniBindings,
    source: &str,
    name: String,
    metadata: JniFunction,
    decl: DeclJniFunction,
    inherited_inputs: Option<
        Vec<prebindgen_declarations::ConfiguredSlot<JniInputConfig, JniInputConverter>>,
    >,
) -> Result<()> {
    let name = prebindgen_declarations::ident(&name)?;
    let previous_index = flat.function(source)?.metadata.iter().position(|binding| {
        (binding.metadata.package == metadata.package && binding.metadata.name == metadata.name)
            || binding.metadata.package.is_empty()
    });
    let previous =
        previous_index.map(|index| flat.function_mut(source).unwrap().metadata.remove(index));
    let previous = match previous {
        None => None,
        Some(binding) => match binding.state {
            BindingState::Prepared(call) => Some(call.into()),
            BindingState::Configured(call) => Some(call),
            _ => return Err(format!("duplicate binding `{name}`")),
        },
    };
    let mut call = ConfiguredCall::from_declaration(flat.function(source)?, decl, previous)?;
    if let Some(inputs) = inherited_inputs {
        for (current, inherited) in call.inputs.iter_mut().zip(inputs) {
            if matches!(current, prebindgen_declarations::ConfiguredSlot::Default)
                && matches!(
                    inherited,
                    prebindgen_declarations::ConfiguredSlot::Prepared(_)
                )
            {
                *current = inherited;
            }
        }
    }
    flat.function_mut(source)?.metadata.push(JniBinding {
        name,
        metadata,
        state: BindingState::Configured(call),
    });
    Ok(())
}

pub(crate) fn resolve_with(
    flat: &mut JniBindings,
    resolver: &JniSourceResolver,
    codec: &JniCodec,
    extra: &(),
) -> Result<()> {
    let names = flat.functions.keys().cloned().collect::<Vec<_>>();
    for source in names {
        let bindings = std::mem::take(&mut flat.function_mut(&source)?.metadata);
        let mut done = Vec::with_capacity(bindings.len());
        for mut binding in bindings {
            binding.state = match binding.state {
                BindingState::Configured(call) => BindingState::Resolved(
                    resolver
                        .resolve_call(flat, codec, &source, call, extra)
                        .map_err(|e| {
                            if binding.metadata.name.is_empty()
                                || (binding.metadata.package.is_empty()
                                    && binding.name == binding.metadata.name)
                            {
                                format!("{}: {e}", binding.name)
                            } else {
                                let export = if binding.metadata.package.is_empty() {
                                    binding.metadata.name.clone()
                                } else {
                                    format!(
                                        "{}.{}",
                                        binding.metadata.package, binding.metadata.name
                                    )
                                };
                                format!("{} (export `{export}`): {e}", binding.name)
                            }
                        })?,
                ),
                other => other,
            };
            done.push(binding);
        }
        flat.function_mut(&source)?.metadata = done;
    }
    Ok(())
}
pub(crate) fn compile_with(flat: &mut JniBindings, codec: &JniCodec, extra: &()) -> Result<()> {
    let names = flat.functions.keys().cloned().collect::<Vec<_>>();
    for source in names {
        let bindings = std::mem::take(&mut flat.function_mut(&source)?.metadata);
        let mut done = Vec::with_capacity(bindings.len());
        for mut binding in bindings {
            binding.state = match binding.state {
                BindingState::Resolved(call) => BindingState::Prepared(
                    prebindgen_declarations::compile_call(flat, codec, &source, call, extra)
                        .map_err(|e| {
                            if binding.metadata.name.is_empty()
                                || (binding.metadata.package.is_empty()
                                    && binding.name == binding.metadata.name)
                            {
                                format!("{}: {e}", binding.name)
                            } else {
                                let export = if binding.metadata.package.is_empty() {
                                    binding.metadata.name.clone()
                                } else {
                                    format!(
                                        "{}.{}",
                                        binding.metadata.package, binding.metadata.name
                                    )
                                };
                                format!("{} (export `{export}`): {e}", binding.name)
                            }
                        })?,
                ),
                BindingState::Prepared(call) => BindingState::Prepared(call),
                BindingState::Configured(_) => {
                    return Err(format!(
                        "binding `{}` has unresolved configuration",
                        binding.name
                    ))
                }
            };
            if binding.metadata.callback_return {
                sequence::callback_target(&binding.call()?.output.output_outcomes().success)?;
            }
            done.push(binding);
        }
        flat.function_mut(&source)?.metadata = done;
    }
    Ok(())
}

pub(crate) fn canonicalize_names(flat: &mut JniBindings) -> Result<()> {
    let mut names = crate::names::Names::default();
    for source in flat.functions.values_mut() {
        let family = |m: &JniFunction| {
            format!(
                "{}.{}",
                m.package,
                if m.callback_return {
                    m.name.strip_suffix("Fold").unwrap_or(&m.name)
                } else {
                    &m.name
                }
            )
        };
        let families = source
            .metadata
            .iter()
            .map(|b| family(&b.metadata))
            .collect::<BTreeSet<_>>();
        for binding in &mut source.metadata {
            let base = if families.len() > 1 {
                names.name(
                    "binding",
                    &source.name.to_string(),
                    &[
                        &crate::names::path_identity(&source.path),
                        &family(&binding.metadata),
                    ],
                )?
            } else {
                source.name.to_string()
            };
            binding.name = prebindgen_declarations::ident(&if binding.metadata.callback_return {
                format!("{base}_fold")
            } else {
                base
            })?;
        }
    }
    // Different source functions can share a callback-derived native label.
    // Foreign names are validated separately; only colliding internal labels need a hash.
    let mut counts = BTreeMap::new();
    for binding in flat.functions.values().flat_map(|f| &f.metadata) {
        *counts.entry(binding.name.to_string()).or_insert(0usize) += 1;
    }
    for source in flat.functions.values_mut() {
        for binding in &mut source.metadata {
            if counts[&binding.name.to_string()] > 1 {
                binding.name = prebindgen_declarations::ident(&names.name(
                    "binding",
                    &binding.name.to_string(),
                    &[
                        &crate::names::path_identity(&source.path),
                        &binding.metadata.package,
                        &binding.metadata.name,
                    ],
                )?)?;
            }
        }
    }
    Ok(())
}

fn selected<'a>(flat: &'a mut JniBindings, source: &str) -> Result<&'a mut JniBinding> {
    if flat.function(source)?.metadata.is_empty() {
        install(
            flat,
            DeclJniFunction::new(prebindgen_declarations::ident(source)?),
            None,
        )?;
    }
    let bindings = &mut flat.function_mut(source)?.metadata;
    if bindings.len() != 1 {
        return Err("multiple exports: select a binding explicitly".into());
    }
    Ok(&mut bindings[0])
}
pub(crate) fn set_input(
    flat: &mut JniBindings,
    source: &str,
    parameter: &str,
    input: JniInputConverter,
) -> Result<()> {
    let index = flat
        .function(source)?
        .params
        .iter()
        .position(|p| p.name == parameter)
        .ok_or("unknown parameter")?;
    match &mut selected(flat, source)?.state {
        BindingState::Configured(call) => {
            call.inputs[index] = prebindgen_declarations::ConfiguredSlot::Prepared(input)
        }
        BindingState::Resolved(call) => {
            call.inputs[index] = prebindgen_declarations::ResolvedSlot::Prepared(input)
        }
        BindingState::Prepared(call) => call.inputs[index] = input,
    }
    Ok(())
}
pub(crate) fn set_output(
    flat: &mut JniBindings,
    source: &str,
    output: JniOutputConverter,
) -> Result<()> {
    match &mut selected(flat, source)?.state {
        BindingState::Configured(call) => {
            call.output = prebindgen_declarations::ConfiguredSlot::Prepared(output)
        }
        BindingState::Resolved(call) => {
            call.output = prebindgen_declarations::ResolvedSlot::Prepared(output)
        }
        BindingState::Prepared(call) => call.output = output,
    }
    Ok(())
}
