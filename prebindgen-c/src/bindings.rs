//! Configure, resolve and compile exports owned by their original source function.
use super::*;
use prebindgen_conversions::BoundFunction;
use prebindgen_declarations::{BindingState, ConfiguredCall, OutputNode};

pub(crate) fn views<
    I,
    O,
    S: prebindgen_flat::Schema<
        Parameter = (),
        Return = (),
        Module = CModule,
        Function = CFunctionData<I, O>,
        Type = CType,
        Field = (),
        Variant = (),
    >,
>(
    flat: &Flat<S>,
) -> Result<Vec<BoundFunction<'_, I, O, CFunction>>> {
    flat.functions
        .values()
        .flat_map(|source| {
            source
                .metadata
                .bindings
                .iter()
                .map(move |binding| binding.prepared(source))
        })
        .collect()
}

pub(crate) fn install(
    flat: &mut CBindings,
    mut decl: DeclCFunction,
    metadata: Option<CFunction>,
) -> Result<()> {
    let source = decl.name.clone();
    let mode = decl.callback_return.take();
    let base = decl.foreign.clone().unwrap_or_else(|| source.clone());
    let mut metadata = metadata.unwrap_or_default();
    let inherited = flat
        .function(&source)?
        .metadata
        .bindings
        .iter()
        .find(|b| b.name == base)
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
        flat.function_mut(&source)?
            .metadata
            .bindings
            .retain(|b| b.name != base);
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

        let structural = decl.ret.is_none();
        decl.ret = Some(OutputNode::Unary {
            operation: COutputUnary::ConfigSequenceCallback { structural },
            child: Box::new(decl.ret.take().unwrap_or_else(COutputConfig::auto)),
        });
        install_one(
            flat,
            &source,
            format!("{base}_each"),
            metadata,
            decl,
            inherited_inputs,
        )?;
    }
    Ok(())
}
fn install_one(
    flat: &mut CBindings,
    source: &str,
    name: String,
    mut metadata: CFunction,
    decl: DeclCFunction,
    inherited_inputs: Option<
        Vec<prebindgen_declarations::ConfiguredSlot<CInputConfig, CInputConverter>>,
    >,
) -> Result<()> {
    let name = prebindgen_declarations::ident(&name)?;
    let previous = flat
        .function_mut(source)?
        .metadata
        .bindings
        .iter()
        .position(|binding| binding.name == name)
        .map(|index| {
            flat.function_mut(source)
                .unwrap()
                .metadata
                .bindings
                .remove(index)
        });
    let previous = match previous {
        None => None,
        Some(binding) => {
            metadata.abort_on_failure |= binding.metadata.abort_on_failure;
            match binding.state {
                BindingState::Prepared(call) => Some(call.into()),
                BindingState::Configured(call) => Some(call),
                _ => return Err(format!("duplicate binding `{name}`")),
            }
        }
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
    flat.function_mut(source)?.metadata.bindings.push(CBinding {
        name,
        metadata,
        state: BindingState::Configured(call),
    });
    Ok(())
}

pub(crate) fn resolve_with(
    flat: &mut CBindings,
    resolver: &CSourceResolver,
    codec: &CCodec,
    extra: &(),
) -> Result<()> {
    let names = flat.functions.keys().cloned().collect::<Vec<_>>();
    for source in names {
        let bindings = std::mem::take(&mut flat.function_mut(&source)?.metadata.bindings);
        let mut done = Vec::with_capacity(bindings.len());
        for mut binding in bindings {
            binding.state = match binding.state {
                BindingState::Configured(call) => BindingState::Resolved(
                    resolver
                        .resolve_call(flat, codec, &source, call, extra)
                        .map_err(|e| format!("{}: {e}", binding.name))?,
                ),
                other => other,
            };
            done.push(binding);
        }
        flat.function_mut(&source)?.metadata.bindings = done;
    }
    Ok(())
}
pub(crate) fn compile_with(flat: &mut CBindings, codec: &CCodec, extra: &()) -> Result<()> {
    let names = flat.functions.keys().cloned().collect::<Vec<_>>();
    for source in names {
        let bindings = std::mem::take(&mut flat.function_mut(&source)?.metadata.bindings);
        let mut done = Vec::with_capacity(bindings.len());
        for mut binding in bindings {
            binding.state = match binding.state {
                BindingState::Resolved(call) => BindingState::Prepared(
                    prebindgen_declarations::compile_call(flat, codec, &source, call, extra)
                        .map_err(|e| format!("{}: {e}", binding.name))?,
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
                sequence::callback_target(&binding.call()?.output.outcomes.success)?;
            }
            done.push(binding);
        }
        flat.function_mut(&source)?.metadata.bindings = done;
    }
    Ok(())
}

fn selected<'a>(flat: &'a mut CBindings, source: &str) -> Result<&'a mut CBinding> {
    if flat.function(source)?.metadata.bindings.is_empty() {
        install(
            flat,
            DeclCFunction::new(prebindgen_declarations::ident(source)?),
            None,
        )?;
    }
    let bindings = &mut flat.function_mut(source)?.metadata.bindings;
    if bindings.len() != 1 {
        return Err("multiple exports: select a binding explicitly".into());
    }
    Ok(&mut bindings[0])
}
pub(crate) fn set_input(
    flat: &mut CBindings,
    source: &str,
    parameter: &str,
    input: CInputConverter,
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
    flat: &mut CBindings,
    source: &str,
    output: COutputConverter,
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

#[cfg(test)]
mod diagnostic_tests {
    use super::*;

    #[test]
    fn occurrence_diagnostics_retain_c_export_and_explicit_children() {
        let mut flat: CBindings = Flat::parse(
            syn::parse_file("pub fn read()->i64{}")
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap();
        let resolver = CSourceResolver::new(&mut flat, vec![], vec![], vec![]).unwrap();
        for (ty, operation, label) in [
            (
                syn::parse_quote!(Option<i64>),
                CInputUnary::Optional,
                "optional payload",
            ),
            (
                syn::parse_quote!(Vec<i64>),
                CInputUnary::Sequence,
                "element",
            ),
        ] {
            let tree = CInputConfig::Unary {
                operation,
                child: Box::new(CInputConfig::None),
            };
            let error = resolver
                .resolve_input(&flat, &CCodec::raw(), &ty, &tree, &())
                .unwrap_err();
            println!("{error}");
            assert_eq!(error, format!("{label}: None input requires unit"));
        }
        let tree = CInputConfig::Branch {
            operation: CInputBranch::Selector,
            children: vec![CInputConfig::None],
        };
        let error = resolver
            .resolve_input(&flat, &CCodec::raw(), &syn::parse_quote!(i64), &tree, &())
            .unwrap_err();
        println!("{error}");
        assert_eq!(error, "alternative 0: None input requires unit");
        for (ty, operation, label) in [
            (
                syn::parse_quote!(Option<i64>),
                COutputUnary::Optional,
                "optional payload",
            ),
            (
                syn::parse_quote!(Vec<i64>),
                COutputUnary::Sequence {
                    delivery: prebindgen_declarations::SequenceDelivery::Value,
                    mode: None,
                },
                "element",
            ),
            (
                syn::parse_quote!(Result<i64,i64>),
                COutputUnary::Result { error: None },
                "Result Ok",
            ),
        ] {
            let tree = COutputConfig::Unary {
                operation,
                child: Box::new(COutputConfig::None),
            };
            let error = resolver
                .resolve_output(&flat, &CCodec::raw(), &ty, &tree, &())
                .unwrap_err();
            println!("{error}");
            assert_eq!(error, format!("{label}: None output requires unit"));
        }
        let tree = COutput::Unary {
            operation: COutputUnary::Optional,
            child: Box::new(COutput::None),
        };
        let error = prebindgen_declarations::compile_output_value(
            &flat,
            &CCodec::raw(),
            &syn::parse_quote!(Option<i64>),
            &tree,
            &(),
        )
        .err()
        .unwrap();
        println!("{error}");
        assert_eq!(error, "optional payload: None output requires unit");
        let error = Cbindgen::new(flat)
            .function(
                crate::fun!(read)
                    .name("read_alias")
                    .expand_return(COutputConfig::None),
            )
            .build()
            .err()
            .unwrap();
        println!("{error}");
        assert_eq!(
            error,
            "read_alias: function `read` result: None output requires unit"
        );
    }
}
