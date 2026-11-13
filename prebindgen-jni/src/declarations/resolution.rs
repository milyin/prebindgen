use super::*;
#[derive(Clone)]
/// This environment is discarded after prepared conversions have been installed.
pub struct DeclJniDefaults {
    values: prebindgen_declarations::DeclDefaults<
        JniInput<JniInputRequest, JniOutputRequest>,
        JniOutput<JniInputRequest, JniOutputRequest>,
    >,
}

fn input_body(
    input: JniInput<JniInputRequest, JniOutputRequest>,
) -> JniInput<JniInputRequest, JniOutputRequest> {
    match input {
        JniInput::Branch {
            operation: JniInputBranch::Selector,
            children: mut arms,
        } if arms.len() == 1 => arms.remove(0),
        JniInput::Unary {
            operation: JniInputUnary::Bulk,
            child,
        } => JniInput::Unary {
            operation: JniInputUnary::Bulk,
            child: Box::new(input_body(*child)),
        },
        JniInput::Unary {
            operation: JniInputUnary::Temporary,
            child,
        } => JniInput::Unary {
            operation: JniInputUnary::Temporary,
            child: Box::new(input_body(*child)),
        },
        other => other,
    }
}
fn output_body(
    output: JniOutput<JniInputRequest, JniOutputRequest>,
) -> JniOutput<JniInputRequest, JniOutputRequest> {
    match output {
        JniOutput::Branch {
            operation: JniOutputBranch::Combine,
            children: mut parts,
        } if parts.len() == 1 => parts.remove(0),
        JniOutput::Unary {
            operation: JniOutputUnary::Bulk,
            child,
        } => JniOutput::Unary {
            operation: JniOutputUnary::Bulk,
            child: Box::new(output_body(*child)),
        },
        other => other,
    }
}
impl DeclJniDefaults {
    /// Register typed Input/Output defaults and explicit error-message functions.
    /// Typed trees use Input::for_type / Output::for_type. Production declaration
    /// builders also convert directly into these trees with `.into()`.
    pub fn new<M: prebindgen_flat::Schema>(
        flat: &Flat<M>,
        inputs: Vec<JniInput<JniInputRequest, JniOutputRequest>>,
        outputs: Vec<JniOutput<JniInputRequest, JniOutputRequest>>,
        errors: Vec<(syn::Type, String)>,
    ) -> Result<Self> {
        let inputs = inputs
            .into_iter()
            .map(|tree| match tree {
                JniInput::Request(JniInputRequest::ForType { ty, input }) => {
                    Ok((*ty, input_body(*input)))
                }
                _ => Err("input defaults require a source type".into()),
            })
            .collect::<Result<Vec<_>>>()?;
        let outputs = outputs
            .into_iter()
            .map(|tree| match tree {
                JniOutput::Request(JniOutputRequest::ForType { ty, output }) => {
                    Ok((*ty, output_body(*output)))
                }
                _ => Err("output defaults require a source type".into()),
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            values: prebindgen_declarations::DeclDefaults::new(flat, inputs, outputs, errors)?,
        })
    }

    /// Clone a source binding for a second explicit sequence delivery policy.
    /// The adapter selects and validates the sequence while resolving the returned declaration.
    pub fn callback_variant(
        &self,
        flat: &mut Flat<JniSchema>,
        _codec: &JniCodec,
        decl: &DeclJniFunction,
        name: &str,
        _extra: &(),
    ) -> Result<DeclJniFunction>
    where
        <JniSchema as Schema>::Input: Clone,
        <JniSchema as Schema>::Output: Clone,
        <JniSchema as prebindgen_flat::Schema>::Function: Clone,
    {
        prebindgen_declarations::callback_variant(flat, decl, name, |output, structural| {
            JniOutput::Request(JniOutputRequest::SequenceCallback {
                output: Box::new(output.cloned().unwrap_or_else(JniOutput::auto)),
                structural,
            })
        })
    }

    /// Resolve source-dependent operations, compile them, and atomically install one binding.
    /// Existing converters are preserved unless explicitly replaced.
    pub fn fill(
        &self,
        flat: &mut Flat<JniSchema>,
        codec: &JniCodec,
        decl: &DeclJniFunction,
        extra: &(),
    ) -> Result<()> {
        let recipe = self
            .resolve_function(flat, codec, decl, extra)
            .map_err(|e| format!("{}: {e}", decl.name))?;
        codec.compile_declarations(flat, extra)?;
        prebindgen_declarations::compile_function(flat, codec, &decl.name, recipe, extra)
            .map_err(|e| format!("{}: {e}", decl.name))
    }
    pub(crate) fn resolve_function(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        decl: &DeclJniFunction,
        extra: &(),
    ) -> Result<FunctionRecipe<JniInput, JniOutput>> {
        prebindgen_declarations::resolve_function(
            flat,
            decl,
            |ty, explicit| {
                self.resolve_input(
                    flat,
                    codec,
                    ty,
                    explicit.unwrap_or(&JniInput::auto()),
                    extra,
                )
            },
            |ty| self.function_output(flat, codec, ty, decl, extra),
        )
    }

    fn function_output(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &syn::Type,
        decl: &DeclJniFunction,
        extra: &(),
    ) -> Result<JniOutput> {
        let tree = prebindgen_declarations::function_output(
            ty,
            decl,
            &self.values.errors,
            JniOutput::auto(),
            |error, child| JniOutput::Unary {
                operation: JniOutputUnary::Result { error },
                child: Box::new(child),
            },
        );
        self.resolve_output(flat, codec, ty, &tree, extra)
    }

    /// Select a function-specific or type-wide explicit error message mapper.
    pub fn error_mapper_for<'a>(
        &'a self,
        decl: &'a DeclJniFunction,
        ty: &syn::Type,
    ) -> Option<&'a str> {
        decl.error
            .as_deref()
            .or_else(|| self.values.errors.get(&type_key(ty)).map(String::as_str))
    }
    /// Prepare owned source-field decomposition, retaining each field's conversion default.
    pub fn owned_record_output(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &syn::Type,
        extra: &(),
    ) -> Result<JniOutput> {
        self.resolve_output(
            flat,
            codec,
            ty,
            &JniOutput::Request(JniOutputRequest::NamedFields {
                overrides: BTreeMap::new(),
                mode: ProjectionMode::Owned,
            }),
            extra,
        )
    }
    /// Resolve a type's default output tree, for example for a typed error channel.
    pub fn output_recipe(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &syn::Type,
        extra: &(),
    ) -> Result<JniOutput> {
        self.resolve_output(flat, codec, ty, &JniOutput::auto(), extra)
    }
    /// Resolve every input operation against canonical source declarations before compilation.
    pub fn resolve_input(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &syn::Type,
        tree: &JniInput<JniInputRequest, JniOutputRequest>,
        extra: &(),
    ) -> Result<JniInput> {
        self.input(flat, codec, ty, tree, extra, &mut vec![])
            .map(operations::lift_input_bulk)
    }
    pub(crate) fn input(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &syn::Type,
        tree: &JniInput<JniInputRequest, JniOutputRequest>,
        extra: &(),
        stack: &mut Vec<String>,
    ) -> Result<JniInput> {
        let resolve = |ty: &syn::Type,
                       tree: &JniInput<JniInputRequest, JniOutputRequest>,
                       stack: &mut Vec<String>| {
            self.input(flat, codec, ty, tree, extra, stack)
        };
        match tree {
            JniInput::None => {
                if type_key(ty) == "()" {
                    Ok(InputNode::None)
                } else {
                    Err("None input requires unit".into())
                }
            }
            JniInput::Branch {
                operation: JniInputBranch::Variant,
                ..
            } => Err("variant branch requires an enum parent".into()),
            JniInput::Request(JniInputRequest::EnumFields(entries)) => {
                self.enum_input_recipe(flat, codec, ty, extra, entries, false, stack)
            }
            JniInput::Branch {
                operation: JniInputBranch::Enum { value },
                children: variants,
            } => {
                let variants = prebindgen_declarations::resolve_enum_variants(
                    flat,
                    ty,
                    variants,
                    |tree| tree.variant_fields(),
                    |ty, recipe| resolve(ty, recipe, stack),
                )?;
                Ok(JniInput::enum_variants(variants, *value))
            }
            JniInput::Request(JniInputRequest::Value) => {
                self.resolve_input_value(flat, codec, ty, extra, stack)
            }
            JniInput::Request(JniInputRequest::Conversion(decl)) => {
                decl.validate()?;
                let source = decl
                    .input
                    .as_ref()
                    .ok_or("local input conversion has no input source")?;
                let conversion = source.resolve(flat, ty, true)?;
                self.resolve_input_conversion(
                    flat,
                    codec,
                    ty,
                    conversion,
                    &JniInput::value(),
                    extra,
                    stack,
                )
            }
            JniInput::Unary {
                operation: JniInputUnary::Convert { conversion },
                child: input,
            } => self.resolve_input_conversion(
                flat,
                codec,
                ty,
                (**conversion).clone(),
                input,
                extra,
                stack,
            ),
            JniInput::Unary {
                operation: JniInputUnary::Temporary,
                child,
            } => Ok(InputNode::Unary {
                operation: JniInputUnary::Temporary,
                child: Box::new(resolve(
                    &prebindgen_declarations::temporary_input_type(ty)?,
                    &input_body((**child).clone()),
                    stack,
                )?),
            }),
            JniInput::Unary {
                operation: JniInputUnary::Bulk,
                child,
            } => Ok(InputNode::Unary {
                operation: JniInputUnary::Bulk,
                child: Box::new(resolve(ty, &input_body((**child).clone()), stack)?),
            }),
            JniInput::Request(JniInputRequest::Auto) => {
                let key = prebindgen_declarations::callback::input_key(flat, ty)?;
                if let Some(tree) = prebindgen_declarations::resolve_type_default(
                    &key,
                    &self.values.inputs,
                    stack,
                    prebindgen_declarations::Direction::Input,
                    |tree, stack| resolve(ty, tree, stack),
                )? {
                    return Ok(tree);
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(InputNode::Unary {
                        operation: JniInputUnary::Optional,
                        child: Box::new(resolve(inner, &JniInput::auto(), stack)?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(InputNode::Unary {
                        operation: JniInputUnary::Sequence,
                        child: Box::new(resolve(element, &JniInput::auto(), stack)?),
                    });
                }
                if type_key(ty) == "()" {
                    return Ok(InputNode::None);
                }
                resolve(ty, &JniInput::Leaf(JniInputLeafOp::Value), stack)
            }
            JniInput::Request(JniInputRequest::ForType { ty: target, input }) => {
                let target = prebindgen_declarations::callback::input_type(flat, target)?;
                let matched =
                    prebindgen_declarations::callback::input_key(flat, ty)? == type_key(&target);
                prebindgen_declarations::resolve_typed_expansion(
                    ty,
                    matched,
                    &input_body((**input).clone()),
                    tree,
                    stack,
                    |ty, tree, stack| resolve(ty, tree, stack),
                    |child| JniInput::Unary {
                        operation: JniInputUnary::Optional,
                        child: Box::new(child),
                    },
                    |child| JniInput::Unary {
                        operation: JniInputUnary::Sequence,
                        child: Box::new(child),
                    },
                    prebindgen_declarations::Direction::Input,
                )
            }
            JniInput::Leaf(JniInputLeafOp::Value) => {
                if flat
                    .type_info(ty)
                    .ok()
                    .is_some_and(|i| i.conversion.input.is_none() && i.conversion.output.is_some())
                {
                    return Err(format!(
                        "`{}` has no canonical input conversion",
                        type_key(ty)
                    ));
                }
                if let Some(conversion) = flat
                    .type_info(ty)
                    .ok()
                    .and_then(|i| i.conversion.input.clone())
                {
                    return self.resolve_input_conversion(
                        flat,
                        codec,
                        ty,
                        conversion,
                        &JniInput::value(),
                        extra,
                        stack,
                    );
                }

                if prebindgen_flat::callback_signature(ty)?.is_some() {
                    return resolve(
                        ty,
                        &JniInput::Request(JniInputRequest::Callback(Box::new(
                            crate::DeclJniCallback::new(ty.clone()),
                        ))),
                        stack,
                    );
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(InputNode::Unary {
                        operation: JniInputUnary::Optional,
                        child: Box::new(resolve(
                            inner,
                            &JniInput::Leaf(JniInputLeafOp::Value),
                            stack,
                        )?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(InputNode::Unary {
                        operation: JniInputUnary::Sequence,
                        child: Box::new(resolve(element, &JniInput::auto(), stack)?),
                    });
                }
                self.resolve_input_representation(flat, codec, ty, tree, stack)
            }
            JniInput::Arrow {
                operation: callback,
                domain,
                codomain,
            } => {
                callback.validate_for(flat, ty)?;
                let signature = prebindgen_flat::callback_signature(ty)?
                    .ok_or("arrow requires a callable source")?;
                let arguments = domain.argument_nodes()?;
                if arguments.len() != signature.arguments.len() {
                    return Err("wrong arrow domain arity".into());
                }
                Ok(InputNode::Arrow {
                    operation: callback.clone(),
                    domain: Box::new(JniOutput::arguments(
                        arguments
                            .iter()
                            .zip(&signature.arguments)
                            .map(|(tree, ty)| {
                                self.output(flat, codec, ty, tree, extra, &mut vec![], true)
                            })
                            .collect::<Result<Vec<_>>>()?,
                    )),
                    codomain: Box::new(resolve(&signature.return_type, codomain, stack)?),
                })
            }
            JniInput::Request(JniInputRequest::Callback(declaration)) => {
                let canonical = prebindgen_declarations::callback::input_key(flat, ty)?;
                let default = match self.values.inputs.get(&canonical) {
                    Some(JniInput::Request(JniInputRequest::Callback(default))) => {
                        Some(default.as_ref())
                    }
                    _ => None,
                };
                let (operation, arguments, ret) = prebindgen_declarations::resolve_callback(
                    flat,
                    ty,
                    declaration,
                    default,
                    &self.values.outputs,
                    &JniInput::auto(),
                    &JniOutput::auto(),
                    stack,
                    |ty, tree, stack| resolve(ty, tree, stack),
                    |ty, tree, ownership| {
                        Ok(self
                            .output(flat, codec, ty, tree, extra, &mut vec![], true)?
                            .callback_argument(ownership))
                    },
                )?;
                Ok(InputNode::Arrow {
                    operation: Box::new(operation),
                    domain: Box::new(JniOutput::arguments(arguments)),
                    codomain: Box::new(ret),
                })
            }
            JniInput::Request(JniInputRequest::Call(function)) => {
                let f = flat.function(function)?;
                resolve(
                    ty,
                    &JniInput::Branch {
                        operation: JniInputBranch::Constructor {
                            function: function.clone(),
                        },
                        children: vec![JniInput::auto(); f.params.len()],
                    },
                    stack,
                )
            }
            JniInput::Branch {
                operation: JniInputBranch::Constructor { function },
                children: args,
            } => prebindgen_declarations::resolve_constructor(
                flat,
                ty,
                function,
                args,
                |ty, child| resolve(ty, child, stack),
                |function| JniInputBranch::Constructor { function },
            ),
            JniInput::Unary {
                operation: JniInputUnary::Optional,
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(InputNode::Unary {
                    operation: JniInputUnary::Optional,
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            JniInput::Unary {
                operation: JniInputUnary::Sequence,
                child,
            } => {
                let (element, _) =
                    sequence_parts(ty).ok_or("sequence input requires Vec<T>, &[T] or &Vec<T>")?;
                Ok(InputNode::Unary {
                    operation: JniInputUnary::Sequence,
                    child: Box::new(resolve(element, child, stack)?),
                })
            }
            JniInput::Branch {
                operation: JniInputBranch::Fields,
                children,
            }
            | JniInput::Branch {
                operation: JniInputBranch::Record,
                children,
            } => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(InputNode::None);
                }
                let children = prebindgen_declarations::resolve_record_fields(
                    flat,
                    ty,
                    children,
                    |ty, child| resolve(ty, child, stack),
                )?;
                Ok(
                    if matches!(
                        tree,
                        JniInput::Branch {
                            operation: JniInputBranch::Record,
                            children: _
                        }
                    ) {
                        InputNode::Branch {
                            operation: JniInputBranch::Record,
                            children,
                        }
                    } else {
                        InputNode::Branch {
                            operation: JniInputBranch::Fields,
                            children,
                        }
                    },
                )
            }
            JniInput::Branch {
                operation: JniInputBranch::Selector,
                children: arms,
            } => {
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                Ok(InputNode::Branch {
                    operation: JniInputBranch::Selector,
                    children: arms
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
        }
    }
    /// Resolve every output operation, source reference and ownership choice before compilation.
    pub fn resolve_output(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &syn::Type,
        tree: &JniOutput<JniInputRequest, JniOutputRequest>,
        extra: &(),
    ) -> Result<JniOutput> {
        self.output(flat, codec, ty, tree, extra, &mut vec![], false)
            .map(operations::lift_output_bulk)
    }
    #[allow(clippy::too_many_arguments)] // Getter defaults differ from the existing plain-reference ABI.
    pub(crate) fn output(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &syn::Type,
        tree: &JniOutput<JniInputRequest, JniOutputRequest>,
        extra: &(),
        stack: &mut Vec<String>,
        getter_result: bool,
    ) -> Result<JniOutput> {
        let resolve = |ty: &syn::Type,
                       tree: &JniOutput<JniInputRequest, JniOutputRequest>,
                       stack: &mut Vec<String>| {
            self.output(flat, codec, ty, tree, extra, stack, getter_result)
        };
        if let Some(referent) = prebindgen_declarations::output_referent(ty) {
            if (getter_result || !matches!(tree, JniOutput::Leaf(JniOutputLeafOp::Value)))
                && !matches!(
                    tree,
                    JniOutput::Request(
                        JniOutputRequest::Auto
                            | JniOutputRequest::ForType { .. }
                            | JniOutputRequest::SequenceCallback { .. }
                    )
                )
            {
                return resolve(referent, tree, stack);
            }
        }
        match tree {
            JniOutput::Request(JniOutputRequest::SequenceCallback { output, structural }) => {
                let output = resolve(ty, output, stack)?;
                operations::sequence_callback(
                    &mut crate::declarations::JniResolve {
                        declarations: self,
                        flat,
                        codec,
                        extra,
                        stack,
                        getter: getter_result,
                    },
                    ty,
                    output,
                    *structural,
                )
            }

            JniOutput::Unary {
                operation: JniOutputUnary::CallbackArgument { ownership },
                child,
            } => Ok((resolve(ty, child, stack)?).callback_argument(*ownership)),
            JniOutput::None => {
                if type_key(ty) == "()" {
                    Ok(OutputNode::None)
                } else {
                    Err("None output requires unit".into())
                }
            }
            JniOutput::Arrow { operation, .. } => match *operation {},
            JniOutput::Branch {
                operation: JniOutputBranch::Variant | JniOutputBranch::Arguments,
                ..
            } => Err("variant/argument branch requires its parent operation".into()),
            JniOutput::Request(JniOutputRequest::EnumFields(entries)) => self.enum_output_recipe(
                flat,
                codec,
                prebindgen_declarations::output_referent(ty).unwrap_or(ty),
                extra,
                entries,
                false,
                stack,
                getter_result,
            ),
            JniOutput::Branch {
                operation: JniOutputBranch::Enum { value, mode },
                children: variants,
            } => {
                let variants = prebindgen_declarations::resolve_enum_variants(
                    flat,
                    ty,
                    variants,
                    |tree| tree.variant_fields(),
                    |ty, recipe| resolve(ty, recipe, stack),
                )?;
                Ok(JniOutput::enum_variants(variants, *value, *mode))
            }
            JniOutput::Request(JniOutputRequest::Value { projection }) => self
                .resolve_output_value(
                    flat,
                    codec,
                    ty,
                    extra,
                    stack,
                    getter_result,
                    projection.as_ref(),
                ),
            JniOutput::Request(JniOutputRequest::Conversion(decl)) => {
                decl.validate()?;
                let source = decl
                    .output
                    .as_ref()
                    .ok_or("local output conversion has no output source")?;
                let conversion = source.resolve(flat, ty, false)?;
                self.resolve_output_conversion(
                    flat,
                    codec,
                    ty,
                    conversion,
                    &JniOutput::value(),
                    extra,
                    stack,
                )
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Convert { conversion },
                child: output,
            } => self.resolve_output_conversion(
                flat,
                codec,
                ty,
                (**conversion).clone(),
                output,
                extra,
                stack,
            ),
            JniOutput::Unary {
                operation: JniOutputUnary::Bulk,
                child,
            } => Ok(OutputNode::Unary {
                operation: JniOutputUnary::Bulk,
                child: Box::new(resolve(ty, &output_body((**child).clone()), stack)?),
            }),
            JniOutput::Leaf(JniOutputLeafOp::Discard) => {
                Ok(OutputNode::Leaf(JniOutputLeafOp::Discard))
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Named { name },
                child: output,
            } => Ok(OutputNode::Unary {
                operation: JniOutputUnary::Named { name: name.clone() },
                child: Box::new(resolve(ty, output, stack)?),
            }),
            JniOutput::Request(JniOutputRequest::Auto) => {
                let key = type_key(ty);
                if let Some(tree) = prebindgen_declarations::resolve_type_default(
                    &key,
                    &self.values.outputs,
                    stack,
                    prebindgen_declarations::Direction::Output,
                    |tree, stack| resolve(ty, tree, stack),
                )? {
                    return Ok(tree);
                }
                if getter_result {
                    if let Some(referent) = prebindgen_declarations::output_referent(ty) {
                        return resolve(referent, tree, stack);
                    }
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(OutputNode::Unary {
                        operation: JniOutputUnary::Optional,
                        child: Box::new(resolve(inner, &JniOutput::auto(), stack)?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(OutputNode::Unary {
                        operation: JniOutputUnary::Sequence {
                            delivery: SequenceDelivery::Auto,
                            mode: None,
                        },
                        child: Box::new(resolve(element, &JniOutput::auto(), stack)?),
                    });
                }
                if type_key(ty) == "()" {
                    return Ok(OutputNode::None);
                }
                resolve(ty, &JniOutput::Leaf(JniOutputLeafOp::Value), stack)
            }
            JniOutput::Request(JniOutputRequest::ForType { ty: target, output }) => {
                let target = flat.resolve_type(target)?;
                let matched = same_type(ty, &target);
                prebindgen_declarations::resolve_typed_expansion(
                    ty,
                    matched,
                    &output_body((**output).clone()),
                    tree,
                    stack,
                    |ty, tree, stack| resolve(ty, tree, stack),
                    |child| {
                        if matches!(&child, JniOutput::Leaf(JniOutputLeafOp::Discard)) {
                            child
                        } else {
                            JniOutput::Unary {
                                operation: JniOutputUnary::Optional,
                                child: Box::new(child),
                            }
                        }
                    },
                    |child| {
                        if matches!(&child, JniOutput::Leaf(JniOutputLeafOp::Discard)) {
                            child
                        } else {
                            JniOutput::Unary {
                                operation: JniOutputUnary::Sequence {
                                    delivery: SequenceDelivery::Auto,
                                    mode: None,
                                },
                                child: Box::new(child),
                            }
                        }
                    },
                    prebindgen_declarations::Direction::Output,
                )
            }
            JniOutput::Leaf(JniOutputLeafOp::Value)
            | JniOutput::Leaf(JniOutputLeafOp::Projected(_)) => {
                let conversion_type = prebindgen_declarations::output_referent(ty).unwrap_or(ty);
                if flat
                    .type_info(conversion_type)
                    .ok()
                    .is_some_and(|i| i.conversion.output.is_none() && i.conversion.input.is_some())
                {
                    return Err(format!(
                        "`{}` has no canonical output conversion",
                        type_key(ty)
                    ));
                }
                let converted = flat
                    .type_info(ty)
                    .ok()
                    .and_then(|i| i.conversion.output.clone())
                    .or_else(|| {
                        prebindgen_declarations::output_referent(ty)
                            .and_then(|t| flat.type_info(t).ok())
                            .and_then(|i| i.conversion.output.clone())
                    });
                if let Some(conversion) = converted {
                    if let JniOutput::Leaf(JniOutputLeafOp::Projected(policy)) = tree {
                        return self.resolve_output_value(
                            flat,
                            codec,
                            ty,
                            extra,
                            stack,
                            getter_result,
                            Some(policy),
                        );
                    }
                    return self.resolve_output_conversion(
                        flat,
                        codec,
                        ty,
                        conversion,
                        &JniOutput::value(),
                        extra,
                        stack,
                    );
                }

                if let Some(inner) = option_inner(ty) {
                    return Ok(OutputNode::Unary {
                        operation: JniOutputUnary::Optional,
                        child: Box::new(resolve(inner, tree, stack)?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    let mode = if let JniOutput::Leaf(JniOutputLeafOp::Projected(p)) = tree {
                        if !p.fields.is_empty() {
                            return Err(
                                "collection field projections belong on the element type".into()
                            );
                        }
                        Some(p.mode)
                    } else {
                        None
                    };
                    return Ok(OutputNode::Unary {
                        operation: JniOutputUnary::Sequence {
                            delivery: SequenceDelivery::Value,
                            mode,
                        },
                        child: Box::new(resolve(element, &JniOutput::auto(), stack)?),
                    });
                }
                self.resolve_output_representation(flat, codec, ty, tree, stack, getter_result)
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Optional,
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(OutputNode::Unary {
                    operation: JniOutputUnary::Optional,
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Sequence { delivery, mode },
                child: element,
            } => {
                let (ty, _) =
                    sequence_parts(ty).ok_or("sequence output requires a collection source")?;
                Ok(OutputNode::Unary {
                    operation: JniOutputUnary::Sequence {
                        delivery: *delivery,
                        mode: *mode,
                    },
                    child: Box::new(resolve(ty, element, stack)?),
                })
            }
            JniOutput::Request(JniOutputRequest::NamedFields { overrides, mode }) => {
                let children = prebindgen_declarations::resolve_named_fields(
                    flat,
                    ty,
                    overrides,
                    &JniOutput::auto(),
                    |ty, tree| resolve(ty, tree, stack),
                )?;
                Ok(if *mode == ProjectionMode::Owned {
                    OutputNode::Branch {
                        operation: JniOutputBranch::OwnedFields,
                        children,
                    }
                } else {
                    OutputNode::Branch {
                        operation: JniOutputBranch::Fields,
                        children,
                    }
                })
            }
            JniOutput::Branch {
                operation: JniOutputBranch::Fields,
                children,
            }
            | JniOutput::Branch {
                operation: JniOutputBranch::OwnedFields,
                children,
            }
            | JniOutput::Branch {
                operation: JniOutputBranch::Record { .. },
                children,
                ..
            } => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(OutputNode::None);
                }
                let children = prebindgen_declarations::resolve_record_fields(
                    flat,
                    ty,
                    children,
                    |ty, child| resolve(ty, child, stack),
                )?;
                Ok(match tree {
                    JniOutput::Branch {
                        operation: JniOutputBranch::Fields,
                        children: _,
                    } => OutputNode::Branch {
                        operation: JniOutputBranch::Fields,
                        children,
                    },
                    JniOutput::Branch {
                        operation: JniOutputBranch::OwnedFields,
                        children: _,
                    } => OutputNode::Branch {
                        operation: JniOutputBranch::OwnedFields,
                        children,
                    },
                    JniOutput::Branch {
                        operation: JniOutputBranch::Record { mode, .. },
                        ..
                    } => OutputNode::Branch {
                        operation: JniOutputBranch::Record { mode: *mode },
                        children,
                    },
                    _ => unreachable!(),
                })
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Getter { function },
                child: output,
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Into { function },
                child: output,
            } => {
                let output = Box::new(prebindgen_declarations::resolve_getter(
                    flat,
                    ty,
                    function,
                    matches!(
                        tree,
                        JniOutput::Unary {
                            operation: JniOutputUnary::Into { .. },
                            ..
                        }
                    ),
                    output,
                    |ty, output| self.output(flat, codec, ty, output, extra, stack, true),
                )?);
                Ok(
                    if matches!(
                        tree,
                        JniOutput::Unary {
                            operation: JniOutputUnary::Into { .. },
                            ..
                        }
                    ) {
                        OutputNode::Unary {
                            operation: JniOutputUnary::Into {
                                function: function.clone(),
                            },
                            child: output,
                        }
                    } else {
                        OutputNode::Unary {
                            operation: JniOutputUnary::Getter {
                                function: function.clone(),
                            },
                            child: output,
                        }
                    },
                )
            }
            JniOutput::Branch {
                operation: JniOutputBranch::Combine,
                children: parts,
            } => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                Ok(OutputNode::Branch {
                    operation: JniOutputBranch::Combine,
                    children: parts
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Result { error },
                child: ok,
            } => {
                let (success, failure) =
                    result_parts(ty).ok_or("Result recipe requires Result<T, E>")?;
                if let Some(name) = error {
                    prebindgen_declarations::validate_error_mapper(flat, failure, name)?;
                }
                Ok(OutputNode::Unary {
                    operation: JniOutputUnary::Result {
                        error: error.clone(),
                    },
                    child: Box::new(resolve(success, ok, stack)?),
                })
            }
        }
    }
}

impl DeclJniDefaults {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_input_conversion(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &Type,
        conversion: ValueConversion,
        child: &JniInput<JniInputRequest, JniOutputRequest>,
        extra: &(),
        stack: &mut Vec<String>,
    ) -> Result<JniInput> {
        let child = prebindgen_declarations::resolve_conversion(
            ty,
            &conversion,
            child,
            stack,
            prebindgen_declarations::Direction::Input,
            |ty, child, stack| self.input(flat, codec, ty, child, extra, stack),
        )?;
        Ok(JniInput::Unary {
            operation: JniInputUnary::Convert {
                conversion: Box::new(conversion),
            },
            child: Box::new(child),
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_output_conversion(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &Type,
        conversion: ValueConversion,
        child: &JniOutput<JniInputRequest, JniOutputRequest>,
        extra: &(),
        stack: &mut Vec<String>,
    ) -> Result<JniOutput> {
        let child = prebindgen_declarations::resolve_conversion(
            ty,
            &conversion,
            child,
            stack,
            prebindgen_declarations::Direction::Output,
            |ty, child, stack| self.output(flat, codec, ty, child, extra, stack, true),
        )?;
        Ok(JniOutput::Unary {
            operation: JniOutputUnary::Convert {
                conversion: Box::new(conversion),
            },
            child: Box::new(child),
        })
    }
    pub(crate) fn resolve_input_value(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &Type,
        extra: &(),
        stack: &mut Vec<String>,
    ) -> Result<JniInput> {
        if type_key(ty) == "()" {
            return Ok(InputNode::Branch {
                operation: JniInputBranch::Fields,
                children: vec![],
            });
        }
        if let Some(c) = flat
            .type_info(ty)
            .ok()
            .and_then(|i| i.conversion.input.clone())
        {
            return self.resolve_input_conversion(
                flat,
                codec,
                ty,
                c,
                &JniInput::value(),
                extra,
                stack,
            );
        }
        if let Some(inner) = prebindgen_flat::option_inner(ty) {
            return Ok(InputNode::Unary {
                operation: JniInputUnary::Optional,
                child: Box::new(self.resolve_input_value(flat, codec, inner, extra, stack)?),
            });
        }
        if let Some((inner, _)) = prebindgen_flat::sequence_parts(ty) {
            return Ok(InputNode::Unary {
                operation: JniInputUnary::Sequence,
                child: Box::new(self.resolve_input_value(flat, codec, inner, extra, stack)?),
            });
        }
        self.input(
            flat,
            codec,
            ty,
            &JniInput::Leaf(JniInputLeafOp::Value),
            extra,
            stack,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_output_value(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &Type,
        extra: &(),
        stack: &mut Vec<String>,
        getter: bool,
        policy: Option<&prebindgen_declarations::Projection>,
    ) -> Result<JniOutput> {
        if type_key(ty) == "()" {
            return Ok(OutputNode::Branch {
                operation: JniOutputBranch::Fields,
                children: vec![],
            });
        }
        if let Some(c) = prebindgen_declarations::output_value_conversion(flat, ty, policy)? {
            return self.resolve_output_conversion(
                flat,
                codec,
                ty,
                c,
                &JniOutput::value(),
                extra,
                stack,
            );
        }
        if let Some(inner) = prebindgen_flat::option_inner(ty) {
            return Ok(OutputNode::Unary {
                operation: JniOutputUnary::Optional,
                child: Box::new(
                    self.resolve_output_value(flat, codec, inner, extra, stack, getter, policy)?,
                ),
            });
        }
        if let Some((inner, _)) = prebindgen_flat::sequence_parts(ty) {
            return Ok(OutputNode::Unary {
                operation: JniOutputUnary::Sequence {
                    delivery: prebindgen_declarations::SequenceDelivery::Value,
                    mode: policy.map(|p| p.mode),
                },
                child: Box::new(
                    self.resolve_output_value(flat, codec, inner, extra, stack, getter, policy)?,
                ),
            });
        }
        self.output(
            flat,
            codec,
            ty,
            &policy.map_or(JniOutput::Leaf(JniOutputLeafOp::Value), |p| {
                JniOutput::Leaf(JniOutputLeafOp::Projected(p.clone()))
            }),
            extra,
            stack,
            getter,
        )
    }
}

impl DeclJniDefaults {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_input_recipe(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &Type,
        extra: &(),
        entries: &BTreeMap<String, BTreeMap<String, JniInput<JniInputRequest, JniOutputRequest>>>,
        value: bool,
        stack: &mut Vec<String>,
    ) -> Result<JniInput> {
        let default = if value {
            JniInput::value()
        } else {
            JniInput::auto()
        };
        let variants = prebindgen_declarations::resolve_enum_fields(
            flat,
            ty,
            entries,
            &default,
            |ty, recipe| self.input(flat, codec, ty, recipe, extra, stack),
        )?;
        Ok(JniInput::enum_variants(variants, value))
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_output_recipe(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &Type,
        extra: &(),
        entries: &BTreeMap<String, BTreeMap<String, JniOutput<JniInputRequest, JniOutputRequest>>>,
        value: bool,
        stack: &mut Vec<String>,
        getter: bool,
    ) -> Result<JniOutput> {
        let default = if value {
            JniOutput::value()
        } else {
            JniOutput::auto()
        };
        let variants = prebindgen_declarations::resolve_enum_fields(
            flat,
            ty,
            entries,
            &default,
            |ty, recipe| self.output(flat, codec, ty, recipe, extra, stack, getter),
        )?;
        Ok(JniOutput::enum_variants(variants, value, None))
    }
}

/// Source-aware context for choosing an adapter representation.
/// Nested resolution uses the same defaults, local policies and recursion stack.
pub struct JniResolve<'a> {
    pub(crate) declarations: &'a DeclJniDefaults,
    /// Source declarations and adapter metadata.
    pub flat: &'a Flat<JniSchema>,
    pub(crate) codec: &'a JniCodec,
    /// Occurrence settings supplied to nested conversions.
    pub extra: &'a (),
    pub(crate) stack: &'a mut Vec<String>,
    pub(crate) getter: bool,
}
impl JniResolve<'_> {
    /// Resolve a nested parameter or callback result.
    pub fn input(
        &mut self,
        ty: &Type,
        tree: &JniInput<JniInputRequest, JniOutputRequest>,
    ) -> Result<ResolvedInput<JniCodec>> {
        self.declarations
            .input(self.flat, self.codec, ty, tree, self.extra, self.stack)
    }
    /// Resolve a nested result, getter or callback argument.
    pub fn output(
        &mut self,
        ty: &Type,
        tree: &JniOutput<JniInputRequest, JniOutputRequest>,
    ) -> Result<ResolvedOutput<JniCodec>> {
        self.declarations.output(
            self.flat,
            self.codec,
            ty,
            tree,
            self.extra,
            self.stack,
            self.getter,
        )
    }
}

impl DeclJniDefaults {
    /// Resolve function occurrences into an independently compilable Flat.
    ///
    /// Defaults and local overrides are consumed here. The returned model owns
    /// its compiler policy and occurrence settings; drop this DeclJniDefaults value
    /// before calling [`prebindgen_declarations::compile_resolved`] if desired.
    pub fn resolve(
        &self,
        flat: Flat<JniSchema>,
        codec: JniCodec,
        declarations: &[DeclJniFunction],
        extra: (),
    ) -> Result<Flat<ResolvedSchema<JniCodec>>> {
        prebindgen_declarations::resolve_bindings(
            flat,
            codec,
            declarations,
            |flat, codec, decl, extra| self.resolve_function(flat, codec, decl, extra),
            extra,
        )
    }
}

impl DeclJniDefaults {
    fn resolve_input_representation(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &Type,
        tree: &JniInput<JniInputRequest, JniOutputRequest>,
        stack: &mut Vec<String>,
    ) -> Result<JniInput> {
        let replacement = (|| {
            if !codec.declared {
                return Ok(None);
            }

            if !matches!(tree, JniInput::Leaf(JniInputLeafOp::Value)) {
                return Ok(None);
            }
            if flat.payload_enums.contains_key(&type_key(ty))
                && flat
                    .type_info(ty)
                    .ok()
                    .and_then(|t| t.metadata.class.as_ref())
                    .is_none()
            {
                return Err(format!(
                "ordinary payload enum `{}` requires sealed_class! or an explicit field expansion",
                type_key(ty)
            ));
            }
            if flat
                .type_info(ty)
                .ok()
                .and_then(|t| t.metadata.class.as_ref())
                .is_some_and(|c| c.kind == ClassKind::Sealed)
            {
                return Ok(Some(JniInput::enum_variants(
                    flat.payload_enum(ty)?
                        .variants
                        .iter()
                        .map(|v| vec![JniInput::value(); v.fields.len()])
                        .collect(),
                    true,
                )));
            }
            let Some(record) = JniCodec::source_record(flat, ty)? else {
                return Ok(None);
            };
            if record.has_handles {
                return Err(format!(
                    "data class `{}` contains handles and cannot be passed into Rust as a record",
                    record.fqn()
                ));
            }
            Ok(Some(JniInput::Branch {
                operation: JniInputBranch::Record,
                children: vec![JniInput::value(); flat.record(ty)?.fields.len()],
            }))
        })();

        match replacement? {
            Some(tree) => self.input(flat, codec, ty, &tree, &(), stack),
            None => Ok(JniInput::Leaf(JniInputLeafOp::Value)),
        }
    }
}

impl DeclJniDefaults {
    fn resolve_output_representation(
        &self,
        flat: &Flat<JniSchema>,
        codec: &JniCodec,
        ty: &Type,
        tree: &JniOutput<JniInputRequest, JniOutputRequest>,
        stack: &mut Vec<String>,
        getter: bool,
    ) -> Result<JniOutput> {
        let replacement = (|| {
            if !codec.declared {
                return Ok(None);
            }

            let policy = match tree {
                JniOutput::Leaf(JniOutputLeafOp::Value) => None,
                JniOutput::Leaf(JniOutputLeafOp::Projected(p)) => Some(p.clone()),
                _ => return Ok(None),
            };
            if flat.payload_enums.contains_key(&type_key(ty))
                && flat
                    .type_info(ty)
                    .ok()
                    .and_then(|t| t.metadata.class.as_ref())
                    .is_none()
            {
                return Err(format!(
                "ordinary payload enum `{}` requires sealed_class! or an explicit field expansion",
                type_key(ty)
            ));
            }
            if flat
                .type_info(ty)
                .ok()
                .and_then(|t| t.metadata.class.as_ref())
                .is_some_and(|c| c.kind == ClassKind::Sealed)
            {
                return Ok(Some(JniOutput::enum_variants(
                    flat.payload_enum(ty)?
                        .variants
                        .iter()
                        .map(|v| vec![JniOutput::value(); v.fields.len()])
                        .collect(),
                    true,
                    policy.as_ref().map(|p| p.mode),
                )));
            }
            let Some(record) = JniCodec::source_record(flat, ty)? else {
                return Ok(None);
            };
            let fields = &flat.record(ty)?.fields;
            for name in policy.iter().flat_map(|p| p.fields.keys()) {
                if !fields
                    .iter()
                    .any(|f| f.name.to_string().trim_start_matches("r#") == name)
                {
                    return Err(format!(
                        "data class `{}`: unknown projection field `{name}`",
                        record.fqn()
                    ));
                }
            }
            let inherited = policy.as_ref().map(Projection::inherited);
            let children = fields.iter().map(|field| {
            let name = field.name.to_string();
            let Some(policy) = &policy else { return Ok(JniOutput::value()); };
            let child = policy.fields.get(name.trim_start_matches("r#")).unwrap_or(inherited.as_ref().unwrap());
            if policy.mode == ProjectionMode::Borrowed && child.mode == ProjectionMode::Owned {
                return Err(format!("data class `{}` field `{name}`: owned projection cannot consume a shared-reference occurrence", record.fqn()));
            }
            Ok(JniOutput::Request(JniOutputRequest::Value {projection:Some(child.clone())}))
        }).collect::<Result<Vec<_>>>()?;
            Ok(Some(JniOutput::Branch {
                operation: JniOutputBranch::Record {
                    mode: policy.map(|p| p.mode),
                },
                children,
            }))
        })();

        match replacement? {
            Some(tree) => self.output(flat, codec, ty, &tree, &(), stack, getter),
            None => Ok(match tree {
                JniOutput::Leaf(JniOutputLeafOp::Value) => JniOutput::Leaf(JniOutputLeafOp::Value),
                JniOutput::Leaf(JniOutputLeafOp::Projected(p)) => {
                    JniOutput::Leaf(JniOutputLeafOp::Projected(p.clone()))
                }
                _ => unreachable!(),
            }),
        }
    }
}
