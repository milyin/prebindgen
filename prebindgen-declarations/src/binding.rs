//! Binding lifecycle data, attached by adapters to their source function owner.
use crate::{DeclFunction, ResolvedSlot};
use prebindgen_conversions::PreparedCall;
use prebindgen_flat::{Function, Result};

/// Configuration for one argument/result, preserving independently supplied converters.
#[derive(Clone, Debug)]
pub enum ConfiguredSlot<C, P> {
    /// Resolve this source position using type defaults.
    Default,
    /// Resolve an explicit local configuration.
    Recipe(C),
    /// Use this converter without reconstructing its configuration.
    Prepared(P),
}
/// Configuration of one exported call; contains no copied source signature.
#[derive(Clone, Debug)]
pub struct ConfiguredCall<IC, OC, I, O> {
    /// One input configuration per source argument.
    pub inputs: Vec<ConfiguredSlot<IC, I>>,
    /// This export's result configuration.
    pub output: ConfiguredSlot<OC, O>,
    /// Deliver the source Result as enum data.
    pub result_value: bool,
    /// Explicit source error-message helper.
    pub error: Option<String>,
}
/// Explicit operations for one call, ready for converter compilation.
#[derive(Clone, Debug)]
pub struct ResolvedCall<IR, OR, I, O> {
    /// Source-ordered input operations or independent converters.
    pub inputs: Vec<ResolvedSlot<IR, I>>,
    /// Result operations or independent converter.
    pub output: ResolvedSlot<OR, O>,
}
/// One exported call with declaration-specific configuration and resolved operations.
pub type DeclBinding<IC, OC, IR, OR, I, O, M> = prebindgen_conversions::FunctionBinding<
    M,
    ConfiguredCall<IC, OC, I, O>,
    ResolvedCall<IR, OR, I, O>,
    PreparedCall<I, O>,
>;
pub use prebindgen_conversions::BindingState;
impl<IC, OC, I, O> ConfiguredCall<IC, OC, I, O> {
    /// Consume a frontend declaration and attach its overrides by source argument order.
    /// A previous prepared call is moved through unchanged unless explicitly overridden.
    pub fn from_declaration<P, R, F, E>(
        source: &Function<P, R, F>,
        mut decl: DeclFunction<IC, OC, E>,
        previous: Option<Self>,
    ) -> Result<Self> {
        if decl.callback_return.is_some() {
            return Err("normalize callback delivery before configuring a binding".into());
        }
        if decl.result_value
            && (decl.error.is_some() || prebindgen_flat::result_parts(&source.ret.ty).is_none())
        {
            return Err(
                "result_as_value requires Result<T, E> and cannot combine with on_error".into(),
            );
        }
        if decl.error.is_some() && prebindgen_flat::result_parts(&source.ret.ty).is_none() {
            return Err("on_error requires a top-level Result<T, E> return".into());
        }
        let (old_inputs, old_output) = match previous {
            Some(call) => {
                if call.inputs.len() != source.params.len() {
                    return Err("prepared input count does not match source parameters".into());
                }
                let native = |slot: &ConfiguredSlot<_, _>| {
                    matches!(slot, ConfiguredSlot::Default | ConfiguredSlot::Prepared(_))
                };
                if call.result_value
                    || call.error.is_some()
                    || !call.inputs.iter().all(native)
                    || !matches!(
                        call.output,
                        ConfiguredSlot::Default | ConfiguredSlot::Prepared(_)
                    )
                    || !(call
                        .inputs
                        .iter()
                        .any(|s| matches!(s, ConfiguredSlot::Prepared(_)))
                        || matches!(call.output, ConfiguredSlot::Prepared(_)))
                {
                    return Err("duplicate binding or unresolved previous configuration".into());
                }
                (call.inputs, call.output)
            }
            None => (
                (0..source.params.len())
                    .map(|_| ConfiguredSlot::Default)
                    .collect(),
                ConfiguredSlot::Default,
            ),
        };
        let inputs = source
            .params
            .iter()
            .zip(old_inputs)
            .map(
                |(p, previous)| match decl.params.remove(&p.name.to_string()) {
                    Some(tree) => ConfiguredSlot::Recipe(tree),
                    None => previous,
                },
            )
            .collect();
        if let Some(name) = decl.params.keys().next() {
            return Err(format!("unknown parameter `{name}`"));
        }
        let output = match decl.ret {
            Some(tree) => ConfiguredSlot::Recipe(tree),
            None => {
                if matches!(old_output, ConfiguredSlot::Prepared(_))
                    && (decl.error.is_some() || decl.result_value)
                {
                    return Err("result policy cannot reconstruct a prebound output; supply an explicit output override".into());
                }
                old_output
            }
        };
        Ok(Self {
            inputs,
            output,
            result_value: decl.result_value,
            error: decl.error,
        })
    }
}

impl<IC, OC, I, O> From<PreparedCall<I, O>> for ConfiguredCall<IC, OC, I, O> {
    fn from(call: PreparedCall<I, O>) -> Self {
        Self {
            inputs: call
                .inputs
                .into_iter()
                .map(ConfiguredSlot::Prepared)
                .collect(),
            output: ConfiguredSlot::Prepared(call.output),
            result_value: false,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prebindgen_flat::{Flat, Schema};
    struct Token(String); // no Clone implementation
    type Export = DeclBinding<String, String, String, String, Token, Token, String>;
    struct Source;
    impl Schema for Source {
        type Module = ();
        type Type = ();
        type Field = ();
        type Variant = ();
        type Parameter = ();
        type Return = ();
        type Function = Vec<Export>;
    }
    fn source() -> Flat<Source> {
        Flat::parse(
            syn::parse_file("pub fn send(packet:i64)->i64{}")
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap()
    }
    #[test]
    fn two_exports_borrow_one_source_signature_and_keep_independent_conversions() {
        let mut flat = source();
        for (name, value) in [("send", 1), ("send_fold", 2)] {
            flat.function_mut("send").unwrap().metadata.push(Export {
                name: syn::parse_str(name).unwrap(),
                metadata: name.into(),
                state: BindingState::Prepared(PreparedCall {
                    inputs: vec![Token(format!("input{value}"))],
                    output: Token(format!("output{value}")),
                }),
            });
        }
        assert_eq!(flat.functions.len(), 1);
        let f = flat.function("send").unwrap();
        let a = f.metadata[0].prepared(f).unwrap();
        let b = f.metadata[1].prepared(f).unwrap();
        assert_eq!(a.params[0].ty.slot.0, "input1");
        assert_eq!(b.params[0].ty.slot.0, "input2");
        assert_eq!(b.ret.slot.0, "output2");
        assert!(std::ptr::eq(a.path, b.path));
        assert!(std::ptr::eq(a.params[0].ty.ty, &f.params[0].ty.ty));
        assert!(std::ptr::eq(b.ret.ty, &f.ret.ty));
    }
    #[test]
    fn unfinished_and_wrong_arity_bindings_are_rejected() {
        let flat = source();
        let f = flat.function("send").unwrap();
        let mut binding = Export {
            name: syn::parse_quote!(export),
            metadata: "name".into(),
            state: BindingState::Configured(ConfiguredCall {
                inputs: vec![ConfiguredSlot::Default],
                output: ConfiguredSlot::Default,
                result_value: false,
                error: None,
            }),
        };
        assert!(binding.prepared(f).err().unwrap().contains("not prepared"));
        binding.state = BindingState::Prepared(PreparedCall {
            inputs: vec![],
            output: Token("out".into()),
        });
        assert!(binding.prepared(f).err().unwrap().contains("input count"));
    }
    #[test]
    fn result_policy_cannot_reinterpret_a_move_only_prepared_output() {
        let flat = Flat::<Source>::parse(
            syn::parse_file("pub fn read()->Result<i64,String>{}")
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap();
        let f = flat.function("read").unwrap();
        let old = || PreparedCall {
            inputs: Vec::<Token>::new(),
            output: Token("domain errors".into()),
        };
        let decl =
            || DeclFunction::<String, String>::new(syn::parse_quote!(read)).result_as_value();
        assert!(
            ConfiguredCall::from_declaration(f, decl(), Some(old().into()))
                .err()
                .unwrap()
                .contains("prebound output")
        );
        let call =
            ConfiguredCall::from_declaration(f, decl().output("enum data"), Some(old().into()))
                .unwrap();
        assert!(matches!(call.output,ConfiguredSlot::Recipe(s) if s=="enum data"));
    }
    #[test]
    fn local_configuration_replaces_only_the_selected_prepared_conversion() {
        let flat = source();
        let f = flat.function("send").unwrap();
        let old = PreparedCall {
            inputs: vec![Token("original input".into())],
            output: Token("original output".into()),
        };
        let decl = DeclFunction::<String, String>::new(syn::parse_quote!(send))
            .input("packet", "local input");
        let call = ConfiguredCall::from_declaration(f, decl, Some(old.into())).unwrap();
        assert!(matches!(&call.inputs[0],ConfiguredSlot::Recipe(s) if s=="local input"));
        assert!(matches!(&call.output,ConfiguredSlot::Prepared(Token(s)) if s=="original output"));
    }
}

/// Compile one resolved call against the original source signature and type attachments.
/// No source occurrence slot is used to represent export existence or conversion state.
pub fn compile_call<C: crate::Codec>(
    flat: &prebindgen_flat::Flat<C::Schema>,
    codec: &C,
    source_name: &str,
    call: ResolvedCall<crate::ResolvedInput<C>, crate::ResolvedOutput<C>, C::Input, C::Output>,
    extra: &C::Extra,
) -> Result<PreparedCall<C::Input, C::Output>> {
    let source = flat.function(source_name)?;
    if call.inputs.len() != source.params.len() {
        return Err(format!(
            "`{source_name}`: input recipe count does not match source parameters"
        ));
    }
    let inputs = source
        .params
        .iter()
        .zip(call.inputs)
        .map(|(p, slot)| match slot {
            ResolvedSlot::Prepared(plan) => Ok(plan),
            ResolvedSlot::Tree(tree) => {
                crate::compile_input(flat, codec, source_name, &p.name.to_string(), &tree, extra)
            }
        })
        .collect::<Result<_>>()?;
    let output = match call.output {
        ResolvedSlot::Prepared(plan) => plan,
        ResolvedSlot::Tree(tree) => crate::compile_output(flat, codec, source_name, &tree, extra)?,
    };
    Ok(PreparedCall { inputs, output })
}

/// Resolve one binding's configuration using its source positions and adapter operations.
/// Defaults are read by the supplied operations from the same Flat's type attachments.
pub fn resolve_call<P, R, F, IC, OC, IR, OR, I, O>(
    source: &Function<P, R, F>,
    call: ConfiguredCall<IC, OC, I, O>,
    mut input: impl FnMut(&syn::Type, Option<IC>) -> Result<IR>,
    output: impl FnOnce(&syn::Type, Option<OC>, bool, Option<String>) -> Result<OR>,
) -> Result<ResolvedCall<IR, OR, I, O>> {
    if call.inputs.len() != source.params.len() {
        return Err("input recipe count does not match source parameters".into());
    }
    let inputs = source
        .params
        .iter()
        .zip(call.inputs)
        .map(|(p, slot)| {
            Ok(match slot {
                ConfiguredSlot::Default => ResolvedSlot::Tree(input(&p.ty.ty, None)?),
                ConfiguredSlot::Recipe(tree) => ResolvedSlot::Tree(input(&p.ty.ty, Some(tree))?),
                ConfiguredSlot::Prepared(plan) => ResolvedSlot::Prepared(plan),
            })
        })
        .collect::<Result<_>>()?;
    let output = match call.output {
        ConfiguredSlot::Default => {
            ResolvedSlot::Tree(output(&source.ret.ty, None, call.result_value, call.error)?)
        }
        ConfiguredSlot::Recipe(tree) => ResolvedSlot::Tree(output(
            &source.ret.ty,
            Some(tree),
            call.result_value,
            call.error,
        )?),
        ConfiguredSlot::Prepared(plan) => ResolvedSlot::Prepared(plan),
    };
    Ok(ResolvedCall { inputs, output })
}

/// Replace one export's parameter converter at any preparation stage.
/// Argument indices are those of the original source signature.
pub fn bind_prepared_input<IC, OC, IR, OR, I, O, M>(
    binding: &mut DeclBinding<IC, OC, IR, OR, I, O, M>,
    index: usize,
    input: I,
) -> Result<()> {
    match &mut binding.state {
        BindingState::Configured(call) => {
            *call
                .inputs
                .get_mut(index)
                .ok_or("unknown parameter index")? = ConfiguredSlot::Prepared(input)
        }
        BindingState::Resolved(call) => {
            *call
                .inputs
                .get_mut(index)
                .ok_or("unknown parameter index")? = ResolvedSlot::Prepared(input)
        }
        BindingState::Prepared(call) => {
            *call
                .inputs
                .get_mut(index)
                .ok_or("unknown parameter index")? = input
        }
    }
    Ok(())
}
/// Replace one export's complete result converter at any preparation stage.
/// Adapter metadata must continue to describe the selected converter's channels.
pub fn bind_prepared_output<IC, OC, IR, OR, I, O, M>(
    binding: &mut DeclBinding<IC, OC, IR, OR, I, O, M>,
    output: O,
) {
    match &mut binding.state {
        BindingState::Configured(call) => call.output = ConfiguredSlot::Prepared(output),
        BindingState::Resolved(call) => call.output = ResolvedSlot::Prepared(output),
        BindingState::Prepared(call) => call.output = output,
    }
}
