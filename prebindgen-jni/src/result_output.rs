//! Function completion composes ordinary converters for both Result payloads.
use super::*;
use prebindgen_conversions::{Delivery, DeliveryPolicy, Outcomes, OutputFragment};
use prebindgen_declarations::{Compiler, OutputNode};

/// Prepared typed domain-error payload and its Kotlin handler. This is owned by
/// a function's Result output converter, never by a parallel type registry.
#[derive(Clone, Debug)]
pub struct JniDomainError {
    pub(crate) handler: String,
    output: JniOutputConverter,
    // An explicit on_error mapper precedes the replaceable handler payload.
    // Its prefix is prepared by the same compiler as ordinary Convert nodes.
    mapping: Option<(syn::Ident, TokenStream)>,
}
impl JniDomainError {
    /// Ordinary converter producing the arguments of this error's Kotlin handler.
    pub fn output(&self) -> &JniOutputConverter {
        &self.output
    }
    /// Fully qualified Kotlin error-handler interface.
    pub fn handler(&self) -> &str {
        &self.handler
    }
    pub(crate) fn replace_output(&mut self, output: JniOutputConverter) -> Result<()> {
        if output.output_outcomes().source_error.is_some() {
            return Err("a domain-error payload cannot open another domain-error channel".into());
        }
        if group::error_wire_shape(&self.output.output_outcomes().success)?
            != group::error_wire_shape(&output.output_outcomes().success)?
        {
            return Err("domain-error converter differs from the declared error wires".into());
        }
        sequence::validate_buffered(&output.output_outcomes().success)?;
        self.output = output;
        Ok(())
    }
    pub(crate) fn throws(&self, exception: Option<&str>) -> String {
        exception.map_or_else(String::new, |class| {
            format!("@kotlin.jvm.Throws({}::class)\n", class.replace('$', "."))
        })
    }
    pub(crate) fn parameter(&self, ret: &str, exception: Option<&str>) -> String {
        let default = exception.map_or_else(String::new, |class| {
            format!(
                " = {} {{ message -> throw {}(message) }}",
                self.handler,
                class.replace('$', ".")
            )
        });
        format!("onError: {}<{ret}>{default}", self.handler)
    }
    fn prepare(&self, context: &JniContext, source: &syn::Ident) -> Result<TokenStream> {
        let mut context = context.clone();
        context.is_domain_error = true;
        context.return_delivery = if context.return_sink.is_some() {
            JniReturnDelivery::Capture
        } else {
            JniReturnDelivery::Packet
        };
        let reported = &context.domain_output;
        context.output_committed = if context.return_sink.is_some() {
            quote!(#reported = Some(std::ptr::null_mut());)
        } else {
            quote!(#reported = Some(result.as_raw());)
        };
        returns::prepare(&self.output, &mut context)?;
        let (payload, prefix) = if let Some((value, preparation)) = &self.mapping {
            (
                value,
                prebindgen_conversions::render_output(&context, preparation, source)?,
            )
        } else {
            (source, TokenStream::new())
        };
        let expected = syn::parse_quote!(::prebindgen_runtime::Result<()>);
        let commit = self.output.code_fragment_slot(
            OutputFragment::Commit {
                context: &context,
                source: payload,
            },
            &syn::parse_quote!(()),
        )?;
        let deliver = self.output.code_fragment_slot(
            OutputFragment::Deliver {
                context: &context,
                source: payload,
                next: quote!({let _:() = #commit; Ok(())}),
            },
            &expected,
        )?;
        let prepare = self.output.code_fragment_slot(
            OutputFragment::Prepare {
                context: &context,
                source: payload,
                next: deliver,
            },
            &expected,
        )?;
        Ok(quote!((|| -> ::prebindgen_runtime::Result<()> {#prefix #prepare})()))
    }
}

#[derive(Clone, Debug)]
struct ResultOutput {
    success: JniOutputConverter,
    error: JniDomainError,
    success_source: syn::Ident,
    failure_source: syn::Ident,
    outcomes: Outcomes<JniOutputWireNode>,
}
impl ResultOutput {
    fn new(
        success: JniOutputConverter,
        error: JniDomainError,
        success_source: syn::Ident,
        failure_source: syn::Ident,
    ) -> Result<Self> {
        if success.output_outcomes().source_error.is_some()
            || error.output.output_outcomes().source_error.is_some()
        {
            return Err("nested Result completion channels require result_as_value".into());
        }
        let mut outcomes = success.output_outcomes().clone();
        outcomes.source_error = Some(error.output.output_outcomes().success.clone());
        Ok(Self {
            success,
            error,
            success_source,
            failure_source,
            outcomes,
        })
    }
}
impl Conversion for ResultOutput {
    type Wire = JniWire;
    type Chain = JniOutputWireChain;
    type Branch = JniOutputWireBranch;
    type Annotation = JniOutputWireAnnotation;
    type Container = JniOutputWireContainer;
    type SlotFragment<'a> = OutputFragment<'a, JniContext>;
    fn code_fragment_slot(
        &self,
        request: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        let payload = &self.success_source;
        Ok(match request {
            OutputFragment::Prepare {
                context,
                source,
                next,
            } => {
                let failure = &self.failure_source;
                let error = self.error.prepare(context, failure)?;
                let prepared = self.success.code_fragment_slot(
                    OutputFragment::Prepare {
                        context,
                        source: payload,
                        next,
                    },
                    expected,
                )?;
                quote!({let #payload = match #source {::core::result::Result::Ok(value)=>value,::core::result::Result::Err(#failure)=>{let _:()=(#error)?;return Err(String::new());}};#prepared})
            }
            OutputFragment::Deliver { context, next, .. } => self.success.code_fragment_slot(
                OutputFragment::Deliver {
                    context,
                    source: payload,
                    next,
                },
                expected,
            )?,
            OutputFragment::Commit { context, .. } => self.success.code_fragment_slot(
                OutputFragment::Commit {
                    context,
                    source: payload,
                },
                expected,
            )?,
            OutputFragment::Result { context, .. } => self.success.code_fragment_slot(
                OutputFragment::Result {
                    context,
                    source: payload,
                },
                expected,
            )?,
        })
    }
}
impl OutputConversion for ResultOutput {
    fn output_outcomes(&self) -> &Outcomes<JniOutputWireNode> {
        &self.outcomes
    }
}
impl KotlinOutputConversion for ResultOutput {
    fn domain_error(&self) -> Option<&JniDomainError> {
        Some(&self.error)
    }
    fn with_domain_error(&self, error: JniDomainError) -> Result<Self> {
        Self::new(
            self.success.clone(),
            error,
            self.success_source.clone(),
            self.failure_source.clone(),
        )
    }
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
        self.success.kotlin(context)
    }
    fn delivery_policy(&self) -> DeliveryPolicy {
        self.success.delivery_policy()
    }
    fn prepare_return_delivery(&self, context: &JniContext) -> Result<Option<Delivery>> {
        self.success.prepare_return_delivery(context)
    }
    fn kotlin_return(&self, context: &KotlinReturnContext<'_>) -> Result<KotlinOutput> {
        self.success.kotlin_return(context)
    }
    fn kotlin_sequence_element(
        &self,
        context: &KotlinContext<'_>,
        owners: Option<&str>,
    ) -> Result<KotlinOutput> {
        self.success.kotlin_sequence_element(context, owners)
    }
}

pub(crate) fn compile(
    cx: &Compiler<'_, JniCodec>,
    ty: &syn::Type,
    tree: &JniOutput,
    value: TokenStream,
    borrowed: bool,
    reference_return: bool,
) -> Result<Option<JniOutputConverter>> {
    let OutputNode::Branch {
        operation:
            JniOutputBranch::Result {
                handler,
                preserve_message_mapper,
            },
        children,
    } = tree
    else {
        return Ok(None);
    };
    if borrowed {
        return Err("Result error routing requires an owned Result occurrence".into());
    }
    let (ok, err) =
        prebindgen_flat::result_parts(ty).ok_or("Result recipe requires Result<T, E>")?;
    let [success, error] = children.as_slice() else {
        return Err("Result completion needs Ok and Err conversions".into());
    };
    let success = cx.output_root(ok, success, &(), value.clone(), false, reference_return)?;
    let (error, mapping) = if *preserve_message_mapper {
        let OutputNode::Unary {
            operation: JniOutputUnary::Convert { conversion },
            child,
        } = error
        else {
            return Err("mapped Result error requires its resolved value conversion".into());
        };
        if prebindgen_flat::type_key(&conversion.representation) != "String" {
            return Err("a preserved Result message mapper must produce String".into());
        }
        let (local, preparation, _) =
            cx.prepare_output_conversion(err, conversion, value, false)?;
        (
            cx.output_root(
                &conversion.representation,
                child,
                &(),
                prebindgen_conversions::source_result_expr(),
                false,
                true,
            )?,
            Some((local, preparation)),
        )
    } else {
        (cx.output_root(err, error, &(), value, false, true)?, None)
    };
    let error = JniDomainError {
        handler: handler.clone(),
        output: error,
        mapping,
    };
    Ok(Some(JniOutputConverter::new(ResultOutput::new(
        success,
        error,
        cx.name(),
        cx.name(),
    )?)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_error_mappers_preserve_e_and_shared_e_arguments() {
        let calls = regex::Regex::new(r"crate::message\((&?)[a-zA-Z0-9_]+\)").unwrap();
        for error in ["&str", "&i64", "&mut i64"] {
            for shared in [false, true] {
                for default in [false, true] {
                    for success in ["i64", "Vec<i64>"] {
                        let parameter = if shared {
                            format!("&{error}")
                        } else {
                            error.into()
                        };
                        let source = syn::parse_file(&format!(
                            "pub fn read(lifetime: &str)->Result<{success},{error}>{{}} \
                         pub fn message(error: {parameter})->String{{}}"
                        ))
                        .unwrap();
                        let mut builder = JniGen::builder()
                            .items(
                                source
                                    .items
                                    .into_iter()
                                    .map(|item| (item, Default::default())),
                            )
                            .set_package_prefix("test");
                        let mut function = fun!(read);
                        if success == "Vec<i64>" {
                            function = function.callback_return();
                        }
                        if default {
                            builder =
                                builder.error_mapper(syn::parse_str(error).unwrap(), fun!(message));
                        } else {
                            function = function.on_error(fun!(message));
                        }
                        let generated = builder.package(package!().fun(function)).build().unwrap();
                        let arguments: Vec<_> = calls.captures_iter(&generated.rust).collect();
                        assert_eq!(arguments.len(), if success == "i64" { 1 } else { 2 });
                        for argument in arguments {
                            assert_eq!(&argument[1], if shared { "&" } else { "" });
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn same_handler_name_requires_the_complete_kotlin_signature_to_match() {
        let source = syn::parse_file(
            "pub fn first()->Result<i64,i64>{} pub fn second()->Result<i64,String>{}",
        )
        .unwrap();
        let mut flat = JniGen::builder()
            .items(
                source
                    .items
                    .into_iter()
                    .map(|item| (item, Default::default())),
            )
            .set_package_prefix("test")
            .package(package!().fun(fun!(first)).fun(fun!(second)))
            .prepare()
            .unwrap();
        let name = flat.function("first").unwrap().metadata[0]
            .call()
            .unwrap()
            .output
            .domain_error()
            .unwrap()
            .handler
            .clone();
        let output = &mut flat.function_mut("second").unwrap().metadata[0]
            .call_mut()
            .unwrap()
            .output;
        let mut error = output.domain_error().unwrap().clone();
        error.handler = name;
        *output = output.with_domain_error(error).unwrap();
        let error = generate(&flat).err().unwrap();
        assert!(
            error.contains("incompatible Kotlin error handler"),
            "{error}"
        );
    }
}
