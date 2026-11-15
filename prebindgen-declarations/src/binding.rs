//! Binding lifecycle data, attached by adapters to their source function owner.
use crate::{DeclFunction, ResolvedSlot};
use prebindgen_conversions::{BoundFunction, PreparedCall};
use prebindgen_flat::{Function, Result};

/// Configuration for one argument/result, preserving independently supplied converters.
pub enum ConfiguredSlot<C, P> {
    /// Resolve this source position using type defaults.
    Default,
    /// Resolve an explicit local configuration.
    Recipe(C),
    /// Use this converter without reconstructing its configuration.
    Prepared(P),
}
/// Configuration of one exported call; contains no copied source signature.
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
pub struct ResolvedCall<IR, OR, I, O> {
    /// Source-ordered input operations or independent converters.
    pub inputs: Vec<ResolvedSlot<IR, I>>,
    /// Result operations or independent converter.
    pub output: ResolvedSlot<OR, O>,
}
/// Preparation stage of a binding, interpreted entirely above Flat.
pub enum BindingState<C, R, P> {
    /// All user configuration is attached to this binding and its type owners.
    Configured(C),
    /// Source-dependent choices are explicit.
    Resolved(R),
    /// Ready for foreign wrapper emission.
    Prepared(P),
}
/// One exported use of a source function. Adapters store these in function attachments.
pub struct DeclBinding<IC, OC, IR, OR, I, O, M> {
    /// Generated wrapper identity, independent of the original Rust callable path.
    pub name: syn::Ident,
    /// Adapter-owned placement and error associations, available at every stage.
    pub metadata: M,
    /// Configuration, operations or compiled conversions for this particular export.
    pub state:
        BindingState<ConfiguredCall<IC, OC, I, O>, ResolvedCall<IR, OR, I, O>, PreparedCall<I, O>>,
}
impl<IC, OC, IR, OR, I, O, M> DeclBinding<IC, OC, IR, OR, I, O, M> {
    /// Borrow one complete export without inventing another source function.
    pub fn prepared<'a, P, R, F>(
        &'a self,
        source: &'a Function<P, R, F>,
    ) -> Result<BoundFunction<'a, I, O, M>> {
        match &self.state {
            BindingState::Prepared(call) => call.view(source, &self.name, &self.metadata),
            _ => Err(format!("binding `{}` is not prepared", self.name)),
        }
    }
}
impl<IC, OC, I, O> ConfiguredCall<IC, OC, I, O> {
    /// Consume a frontend declaration and attach its overrides by source argument order.
    /// A previous prepared call is moved through unchanged unless explicitly overridden.
    pub fn from_declaration<P, R, F, E>(
        source: &Function<P, R, F>,
        mut decl: DeclFunction<IC, OC, E>,
        previous: Option<PreparedCall<I, O>>,
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
                (
                    call.inputs.into_iter().map(Some).collect::<Vec<_>>(),
                    Some(call.output),
                )
            }
            None => ((0..source.params.len()).map(|_| None).collect(), None),
        };
        let inputs = source
            .params
            .iter()
            .zip(old_inputs)
            .map(
                |(p, previous)| match decl.params.remove(&p.name.to_string()) {
                    Some(tree) => ConfiguredSlot::Recipe(tree),
                    None => previous
                        .map(ConfiguredSlot::Prepared)
                        .unwrap_or(ConfiguredSlot::Default),
                },
            )
            .collect();
        if let Some(name) = decl.params.keys().next() {
            return Err(format!("unknown parameter `{name}`"));
        }
        let output = match decl.ret {
            Some(tree) => ConfiguredSlot::Recipe(tree),
            None => {
                if old_output.is_some() && (decl.error.is_some() || decl.result_value) {
                    return Err("result policy cannot reconstruct a prebound output; supply an explicit output override".into());
                }
                old_output
                    .map(ConfiguredSlot::Prepared)
                    .unwrap_or(ConfiguredSlot::Default)
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
        assert!(ConfiguredCall::from_declaration(f, decl(), Some(old()))
            .err()
            .unwrap()
            .contains("prebound output"));
        let call =
            ConfiguredCall::from_declaration(f, decl().output("enum data"), Some(old())).unwrap();
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
        let call = ConfiguredCall::from_declaration(f, decl, Some(old)).unwrap();
        assert!(matches!(&call.inputs[0],ConfiguredSlot::Recipe(s) if s=="local input"));
        assert!(matches!(&call.output,ConfiguredSlot::Prepared(Token(s)) if s=="original output"));
    }
}
