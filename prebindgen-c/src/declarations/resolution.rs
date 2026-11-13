use super::*;
#[derive(Clone)]
/// This environment is discarded after prepared conversions have been installed.
pub struct DeclCDefaults {
    input_role: std::cell::Cell<InputRole>,

    values: prebindgen_declarations::DeclDefaults<
        CInput<CInputRequest, COutputRequest>,
        COutput<CInputRequest, COutputRequest>,
    >,
}

fn input_body(
    input: CInput<CInputRequest, COutputRequest>,
) -> CInput<CInputRequest, COutputRequest> {
    match input {
        CInput::Branch {
            operation: prebindgen_declarations::Operation::Resolved(CInputBranch::Selector),
            children: mut arms,
        } if arms.len() == 1 => arms.remove(0),
        CInput::Unary {
            operation: prebindgen_declarations::Operation::Resolved(CInputUnary::Temporary),
            child,
        } => CInput::Unary {
            operation: (CInputUnary::Temporary).into(),
            child: Box::new(input_body(*child)),
        },
        other => other,
    }
}
fn output_body(
    output: COutput<CInputRequest, COutputRequest>,
) -> COutput<CInputRequest, COutputRequest> {
    match output {
        COutput::Branch {
            operation: prebindgen_declarations::Operation::Resolved(COutputBranch::Combine),
            children: mut parts,
        } if parts.len() == 1 => parts.remove(0),
        other => other,
    }
}
impl DeclCDefaults {
    /// Register typed Input/Output defaults and explicit error-message functions.
    /// Typed trees use Input::for_type / Output::for_type. Production declaration
    /// builders also convert directly into these trees with `.into()`.
    pub fn new<M: prebindgen_flat::Schema>(
        flat: &Flat<M>,
        inputs: Vec<CInput<CInputRequest, COutputRequest>>,
        outputs: Vec<COutput<CInputRequest, COutputRequest>>,
        errors: Vec<(syn::Type, String)>,
    ) -> Result<Self> {
        let inputs = inputs
            .into_iter()
            .map(|tree| match tree {
                CInput::Unary {
                    operation:
                        prebindgen_declarations::Operation::Request(CInputRequest::ForType { ty }),
                    child: input,
                } => Ok((*ty, input_body(*input))),

                _ => Err("input defaults require a source type".into()),
            })
            .collect::<Result<Vec<_>>>()?;
        let outputs = outputs
            .into_iter()
            .map(|tree| match tree {
                COutput::Unary {
                    operation:
                        prebindgen_declarations::Operation::Request(COutputRequest::ForType { ty }),
                    child: output,
                } => Ok((*ty, output_body(*output))),

                _ => Err("output defaults require a source type".into()),
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            values: prebindgen_declarations::DeclDefaults::new(flat, inputs, outputs, errors)?,
            input_role: std::cell::Cell::new(InputRole::Parameter),
        })
    }

    /// Clone a source binding for a second explicit sequence delivery policy.
    /// The adapter selects and validates the sequence while resolving the returned declaration.
    pub fn callback_variant(
        &self,
        flat: &mut Flat<CSchema>,
        _codec: &CCodec,
        decl: &DeclCFunction,
        name: &str,
        _extra: &(),
    ) -> Result<DeclCFunction>
    where
        <CSchema as Schema>::Input: Clone,
        <CSchema as Schema>::Output: Clone,
        <CSchema as prebindgen_flat::Schema>::Function: Clone,
    {
        prebindgen_declarations::callback_variant(flat, decl, name, |output, structural| {
            COutput::Unary {
                operation: prebindgen_declarations::Operation::Request(
                    COutputRequest::SequenceCallback { structural },
                ),
                child: Box::new(output.cloned().unwrap_or_else(COutput::auto)),
            }
        })
    }

    /// Resolve source-dependent operations, compile them, and atomically install one binding.
    /// Existing converters are preserved unless explicitly replaced.
    pub fn fill(
        &self,
        flat: &mut Flat<CSchema>,
        codec: &CCodec,
        decl: &DeclCFunction,
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
        flat: &Flat<CSchema>,
        codec: &CCodec,
        decl: &DeclCFunction,
        extra: &(),
    ) -> Result<FunctionRecipe<CInput, COutput>> {
        prebindgen_declarations::resolve_function(
            flat,
            decl,
            |ty, explicit| {
                self.resolve_input(flat, codec, ty, explicit.unwrap_or(&CInput::auto()), extra)
            },
            |ty| self.function_output(flat, codec, ty, decl, extra),
        )
    }

    fn function_output(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &syn::Type,
        decl: &DeclCFunction,
        extra: &(),
    ) -> Result<COutput> {
        let tree = prebindgen_declarations::function_output(
            ty,
            decl,
            &self.values.errors,
            COutput::auto(),
            |error, child| COutput::Unary {
                operation: (COutputUnary::Result { error }).into(),
                child: Box::new(child),
            },
        );
        self.resolve_output(flat, codec, ty, &tree, extra)
    }

    /// Select a function-specific or type-wide explicit error message mapper.
    pub fn error_mapper_for<'a>(
        &'a self,
        decl: &'a DeclCFunction,
        ty: &syn::Type,
    ) -> Option<&'a str> {
        decl.error
            .as_deref()
            .or_else(|| self.values.errors.get(&type_key(ty)).map(String::as_str))
    }
    /// Prepare owned source-field decomposition, retaining each field's conversion default.
    pub fn owned_record_output(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &syn::Type,
        extra: &(),
    ) -> Result<COutput> {
        self.resolve_output(
            flat,
            codec,
            ty,
            &COutput::Branch {
                operation: prebindgen_declarations::Operation::Request(
                    COutputRequest::NamedFields {
                        fields: vec![],
                        mode: ProjectionMode::Owned,
                    },
                ),
                children: vec![],
            },
            extra,
        )
    }
    /// Resolve a type's default output tree, for example for a typed error channel.
    pub fn output_recipe(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &syn::Type,
        extra: &(),
    ) -> Result<COutput> {
        self.resolve_output(flat, codec, ty, &COutput::auto(), extra)
    }
    /// Resolve every input operation against canonical source declarations before compilation.
    pub fn resolve_input(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &syn::Type,
        tree: &CInput<CInputRequest, COutputRequest>,
        extra: &(),
    ) -> Result<CInput> {
        self.input(flat, codec, ty, tree, extra, &mut vec![])
    }
    pub(crate) fn input(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &syn::Type,
        tree: &CInput<CInputRequest, COutputRequest>,
        extra: &(),
        stack: &mut Vec<String>,
    ) -> Result<CInput> {
        let resolve = |ty: &syn::Type,
                       tree: &CInput<CInputRequest, COutputRequest>,
                       stack: &mut Vec<String>| {
            self.input(flat, codec, ty, tree, extra, stack)
        };
        match tree {
            CInput::None => {
                if type_key(ty) == "()" {
                    Ok(InputNode::None)
                } else {
                    Err("None input requires unit".into())
                }
            }
            CInput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(CInputBranch::Variant),
                ..
            } => Err("variant branch requires an enum parent".into()),
            CInput::Branch {
                operation: prebindgen_declarations::Operation::Request(CInputRequest::EnumFields),
                children,
            } => {
                let entries =
                    prebindgen_declarations::enum_children(children, |tree| match tree {
                        CInput::Branch {
                            operation:
                                prebindgen_declarations::Operation::Request(
                                    CInputRequest::VariantFields { variant, fields },
                                ),
                            children,
                        } => Ok((variant.as_str(), fields.as_slice(), children.as_slice())),
                        _ => Err("named enum fields require variant branches".into()),
                    })?;
                self.enum_input_recipe(flat, codec, ty, extra, &entries, false, stack)
            }
            CInput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(CInputBranch::Enum { value }),
                children: variants,
            } => {
                let variants = prebindgen_declarations::resolve_enum_variants(
                    flat,
                    ty,
                    variants,
                    |tree| tree.variant_fields(),
                    |ty, recipe| {
                        let previous = self.input_role.get();
                        if *value {
                            self.input_role.set(InputRole::EnumField);
                        }
                        let result = resolve(ty, recipe, stack);
                        self.input_role.set(previous);
                        result
                    },
                )?;
                Ok(CInput::enum_variants(variants, *value))
            }
            CInput::Request(CInputRequest::Value) => {
                self.resolve_input_value(flat, codec, ty, extra, stack)
            }
            CInput::Request(CInputRequest::Conversion(decl)) => {
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
                    &CInput::value(),
                    extra,
                    stack,
                )
            }
            CInput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(CInputUnary::Convert { conversion }),
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
            CInput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(CInputUnary::Temporary),
                child,
            } => Ok(InputNode::Unary {
                operation: (CInputUnary::Temporary).into(),
                child: Box::new(resolve(
                    &prebindgen_declarations::temporary_input_type(ty)?,
                    &input_body((**child).clone()),
                    stack,
                )?),
            }),
            CInput::Request(CInputRequest::Auto) => {
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
                if let Some(tree) = prebindgen_declarations::resolve_container(
                    ty,
                    !(owned::native_input(flat, ty)
                        || (self.input_role.get() == InputRole::EnumField
                            && owned::declaration(flat, ty).is_some())),
                    |inner| resolve(inner, &CInput::auto(), stack),
                    |child| CInput::Unary {
                        operation: (CInputUnary::Optional).into(),
                        child: Box::new(child),
                    },
                    |child| CInput::Unary {
                        operation: (CInputUnary::Sequence).into(),
                        child: Box::new(child),
                    },
                )? {
                    return Ok(tree);
                }
                if type_key(ty) == "()" {
                    return Ok(InputNode::None);
                }
                resolve(ty, &CInput::Leaf(CInputLeafOp::Value), stack)
            }
            CInput::Unary {
                operation:
                    prebindgen_declarations::Operation::Request(CInputRequest::ForType { ty: target }),
                child: input,
            } => {
                let target = prebindgen_declarations::callback::input_type(flat, target)?;
                let matched =
                    prebindgen_declarations::callback::input_key(flat, ty)? == type_key(&target);
                prebindgen_declarations::resolve_typed_expansion(
                    ty,
                    matched,
                    &input_body((**input).clone()),
                    tree,
                    stack,
                    resolve,
                    |child| CInput::Unary {
                        operation: (CInputUnary::Optional).into(),
                        child: Box::new(child),
                    },
                    |child| CInput::Unary {
                        operation: (CInputUnary::Sequence).into(),
                        child: Box::new(child),
                    },
                    prebindgen_declarations::Direction::Input,
                )
            }
            CInput::Leaf(CInputLeafOp::Value) => {
                if let Some(conversion) = prebindgen_declarations::input_value_conversion(flat, ty)?
                {
                    return self.resolve_input_conversion(
                        flat,
                        codec,
                        ty,
                        conversion,
                        &CInput::value(),
                        extra,
                        stack,
                    );
                }
                if prebindgen_flat::callback_signature(ty)?.is_some() {
                    return resolve(ty, &CInput::from(DeclCCallback::new(ty.clone())), stack);
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(InputNode::Unary {
                        operation: (CInputUnary::Optional).into(),
                        child: Box::new(resolve(inner, &CInput::Leaf(CInputLeafOp::Value), stack)?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty).filter(|_| {
                    !(owned::native_input(flat, ty)
                        || (self.input_role.get() == InputRole::EnumField
                            && owned::declaration(flat, ty).is_some()))
                }) {
                    return Ok(InputNode::Unary {
                        operation: (CInputUnary::Sequence).into(),
                        child: Box::new(resolve(element, &CInput::auto(), stack)?),
                    });
                }
                self.resolve_input_representation(flat, codec, ty, tree, stack)
            }
            CInput::Arrow {
                operation: prebindgen_declarations::Operation::Resolved(callback),
                domain,
                codomain,
            } => {
                let (arguments, result) = prebindgen_declarations::resolve_callback_arrow(
                    flat,
                    ty,
                    callback,
                    domain.argument_nodes()?,
                    codomain,
                    |ty, tree| self.output(flat, codec, ty, tree, extra, &mut vec![], true),
                    |ty, tree| resolve(ty, tree, stack),
                )?;
                Ok(InputNode::Arrow {
                    operation: (callback.clone()).into(),
                    domain: Box::new(COutput::arguments(arguments)),
                    codomain: Box::new(result),
                })
            }
            CInput::Arrow {
                operation:
                    prebindgen_declarations::Operation::Request(CInputRequest::Callback(declaration)),
                domain,
                codomain,
            } => {
                let canonical = prebindgen_declarations::callback::input_key(flat, ty)?;
                let default = match self.values.inputs.get(&canonical) {
                    Some(CInput::Arrow {
                        operation:
                            prebindgen_declarations::Operation::Request(CInputRequest::Callback(
                                request,
                            )),
                        domain,
                        codomain,
                    }) => Some(prebindgen_declarations::CallbackTree {
                        request,
                        arguments: domain.argument_nodes()?,
                        result: codomain.as_ref(),
                    }),
                    _ => None,
                };
                let (operation, arguments, ret) = prebindgen_declarations::resolve_callback_tree(
                    flat,
                    ty,
                    prebindgen_declarations::CallbackTree {
                        request: declaration,
                        arguments: domain.argument_nodes()?,
                        result: codomain.as_ref(),
                    },
                    default,
                    &self.values.outputs,
                    &CInput::auto(),
                    &COutput::auto(),
                    |tree| matches!(tree, CInput::Request(CInputRequest::Auto)),
                    stack,
                    resolve,
                    |ty, tree, ownership| {
                        Ok(self
                            .output(flat, codec, ty, tree, extra, &mut vec![], true)?
                            .callback_argument(ownership))
                    },
                )?;
                Ok(InputNode::Arrow {
                    operation: Box::new(operation).into(),
                    domain: Box::new(COutput::arguments(arguments)),
                    codomain: Box::new(ret),
                })
            }

            CInput::Request(CInputRequest::Call(function)) => {
                let f = flat.function(function)?;
                resolve(
                    ty,
                    &CInput::Branch {
                        operation: (CInputBranch::Constructor {
                            function: function.clone(),
                        })
                        .into(),
                        children: vec![CInput::auto(); f.params.len()],
                    },
                    stack,
                )
            }
            CInput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(CInputBranch::Constructor { function }),
                children: args,
            } => prebindgen_declarations::resolve_constructor(
                flat,
                ty,
                function,
                args,
                |ty, child| resolve(ty, child, stack),
                |function| CInputBranch::Constructor { function },
            ),
            CInput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(CInputUnary::Optional),
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(InputNode::Unary {
                    operation: (CInputUnary::Optional).into(),
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            CInput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(CInputUnary::Sequence),
                child,
            } => {
                let (element, _) =
                    sequence_parts(ty).ok_or("sequence input requires Vec<T>, &[T] or &Vec<T>")?;
                Ok(InputNode::Unary {
                    operation: (CInputUnary::Sequence).into(),
                    child: Box::new(resolve(element, child, stack)?),
                })
            }
            CInput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(CInputBranch::Fields),
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
                Ok({
                    InputNode::Branch {
                        operation: (CInputBranch::Fields).into(),
                        children,
                    }
                })
            }
            CInput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(CInputBranch::Selector),
                children: arms,
            } => {
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                Ok(InputNode::Branch {
                    operation: (CInputBranch::Selector).into(),
                    children: arms
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
            CInput::Request(_)
            | CInput::Unary {
                operation: prebindgen_declarations::Operation::Request(_),
                ..
            }
            | CInput::Branch {
                operation: prebindgen_declarations::Operation::Request(_),
                ..
            }
            | CInput::Arrow {
                operation: prebindgen_declarations::Operation::Request(_),
                ..
            } => Err("pending operation has the wrong structural node shape".into()),
        }
    }
    /// Resolve every output operation, source reference and ownership choice before compilation.
    pub fn resolve_output(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &syn::Type,
        tree: &COutput<CInputRequest, COutputRequest>,
        extra: &(),
    ) -> Result<COutput> {
        self.output(flat, codec, ty, tree, extra, &mut vec![], false)
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn output(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &syn::Type,
        tree: &COutput<CInputRequest, COutputRequest>,
        extra: &(),
        stack: &mut Vec<String>,
        getter_result: bool,
    ) -> Result<COutput> {
        let resolve = |ty: &syn::Type,
                       tree: &COutput<CInputRequest, COutputRequest>,
                       stack: &mut Vec<String>| {
            self.output(flat, codec, ty, tree, extra, stack, getter_result)
        };
        if let Some(referent) = prebindgen_declarations::output_referent(ty) {
            if (getter_result || !matches!(tree, COutput::Leaf(COutputLeafOp::Value)))
                && !matches!(
                    tree.pending_request(),
                    Some(
                        COutputRequest::Auto
                            | COutputRequest::ForType { .. }
                            | COutputRequest::SequenceCallback { .. }
                    )
                )
            {
                return resolve(referent, tree, stack);
            }
        }
        match tree {
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Request(COutputRequest::SequenceCallback {
                        structural,
                    }),
                child: output,
            } => {
                let output = resolve(ty, output, stack)?;
                operations::sequence_callback(
                    &mut crate::declarations::CResolve {
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
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::CallbackArgument {
                        ownership,
                    }),
                child,
            } => Ok((resolve(ty, child, stack)?).callback_argument(*ownership)),
            COutput::None => {
                if type_key(ty) == "()" {
                    Ok(OutputNode::None)
                } else {
                    Err("None output requires unit".into())
                }
            }
            COutput::Arrow {
                operation: prebindgen_declarations::Operation::Resolved(never),
                ..
            } => match *never {},
            COutput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(
                        COutputBranch::Variant | COutputBranch::Arguments,
                    ),
                ..
            } => Err("variant/argument branch requires its parent operation".into()),
            COutput::Branch {
                operation: prebindgen_declarations::Operation::Request(COutputRequest::EnumFields),
                children,
            } => {
                let entries =
                    prebindgen_declarations::enum_children(children, |tree| match tree {
                        COutput::Branch {
                            operation:
                                prebindgen_declarations::Operation::Request(
                                    COutputRequest::VariantFields { variant, fields },
                                ),
                            children,
                        } => Ok((variant.as_str(), fields.as_slice(), children.as_slice())),
                        _ => Err("named enum fields require variant branches".into()),
                    })?;
                self.enum_output_recipe(
                    flat,
                    codec,
                    prebindgen_declarations::output_referent(ty).unwrap_or(ty),
                    extra,
                    &entries,
                    false,
                    stack,
                    getter_result,
                )
            }

            COutput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputBranch::Enum { value, mode }),
                children: variants,
            } => {
                let variants = prebindgen_declarations::resolve_enum_variants(
                    flat,
                    ty,
                    variants,
                    |tree| tree.variant_fields(),
                    |ty, recipe| resolve(ty, recipe, stack),
                )?;
                Ok(COutput::enum_variants(variants, *value, *mode))
            }
            COutput::Request(COutputRequest::Value { projection }) => self.resolve_output_value(
                flat,
                codec,
                ty,
                extra,
                stack,
                getter_result,
                projection.as_ref(),
            ),
            COutput::Request(COutputRequest::Conversion(decl)) => {
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
                    &COutput::value(),
                    extra,
                    stack,
                )
            }
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Convert { conversion }),
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
            COutput::Leaf(COutputLeafOp::Discard) => Ok(OutputNode::Leaf(COutputLeafOp::Discard)),
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Named { name }),
                child: output,
            } => Ok(OutputNode::Unary {
                operation: (COutputUnary::Named { name: name.clone() }).into(),
                child: Box::new(resolve(ty, output, stack)?),
            }),
            COutput::Request(COutputRequest::Auto) => {
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
                if let Some(tree) = prebindgen_declarations::resolve_container(
                    ty,
                    !owned::native_output(flat, ty),
                    |inner| resolve(inner, &COutput::auto(), stack),
                    |child| COutput::Unary {
                        operation: (COutputUnary::Optional).into(),
                        child: Box::new(child),
                    },
                    |child| COutput::Unary {
                        operation: (COutputUnary::Sequence {
                            delivery: SequenceDelivery::Auto,
                            mode: None,
                        })
                        .into(),
                        child: Box::new(child),
                    },
                )? {
                    return Ok(tree);
                }
                if type_key(ty) == "()" {
                    return Ok(OutputNode::None);
                }
                resolve(ty, &COutput::Leaf(COutputLeafOp::Value), stack)
            }
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Request(COutputRequest::ForType { ty: target }),
                child: output,
            } => {
                let target = flat.resolve_type(target)?;
                let matched = same_type(ty, &target);
                prebindgen_declarations::resolve_typed_expansion(
                    ty,
                    matched,
                    &output_body((**output).clone()),
                    tree,
                    stack,
                    resolve,
                    |child| {
                        if matches!(&child, COutput::Leaf(COutputLeafOp::Discard)) {
                            child
                        } else {
                            COutput::Unary {
                                operation: (COutputUnary::Optional).into(),
                                child: Box::new(child),
                            }
                        }
                    },
                    |child| {
                        if matches!(&child, COutput::Leaf(COutputLeafOp::Discard)) {
                            child
                        } else {
                            COutput::Unary {
                                operation: (COutputUnary::Sequence {
                                    delivery: SequenceDelivery::Auto,
                                    mode: None,
                                })
                                .into(),
                                child: Box::new(child),
                            }
                        }
                    },
                    prebindgen_declarations::Direction::Output,
                )
            }
            COutput::Leaf(COutputLeafOp::Value) | COutput::Leaf(COutputLeafOp::Projected(_)) => {
                if let Some(conversion) =
                    prebindgen_declarations::output_value_conversion(flat, ty, None)?
                {
                    if let COutput::Leaf(COutputLeafOp::Projected(policy)) = tree {
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
                        &COutput::value(),
                        extra,
                        stack,
                    );
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(OutputNode::Unary {
                        operation: (COutputUnary::Optional).into(),
                        child: Box::new(resolve(inner, tree, stack)?),
                    });
                }
                if let Some((element, _)) =
                    sequence_parts(ty).filter(|_| !owned::native_output(flat, ty))
                {
                    let mode = if let COutput::Leaf(COutputLeafOp::Projected(p)) = tree {
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
                        operation: (COutputUnary::Sequence {
                            delivery: SequenceDelivery::Value,
                            mode,
                        })
                        .into(),
                        child: Box::new(resolve(element, &COutput::auto(), stack)?),
                    });
                }
                self.resolve_output_representation(flat, codec, ty, tree, stack, getter_result)
            }
            COutput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(COutputUnary::Optional),
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(OutputNode::Unary {
                    operation: (COutputUnary::Optional).into(),
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Sequence {
                        delivery,
                        mode,
                    }),
                child: element,
            } => {
                let (ty, _) =
                    sequence_parts(ty).ok_or("sequence output requires a collection source")?;
                Ok(OutputNode::Unary {
                    operation: (COutputUnary::Sequence {
                        delivery: *delivery,
                        mode: *mode,
                    })
                    .into(),
                    child: Box::new(resolve(ty, element, stack)?),
                })
            }
            COutput::Branch {
                operation:
                    prebindgen_declarations::Operation::Request(COutputRequest::NamedFields {
                        fields,
                        mode,
                    }),
                children: recipes,
            } => {
                let overrides = prebindgen_declarations::named_children(fields, recipes)?;
                let children = prebindgen_declarations::resolve_named_fields(
                    flat,
                    ty,
                    &overrides,
                    &COutput::auto(),
                    |ty, tree| resolve(ty, tree, stack),
                )?;
                Ok(if *mode == ProjectionMode::Owned {
                    OutputNode::Branch {
                        operation: (COutputBranch::OwnedFields).into(),
                        children,
                    }
                } else {
                    OutputNode::Branch {
                        operation: (COutputBranch::Fields).into(),
                        children,
                    }
                })
            }
            COutput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(COutputBranch::Fields),
                children,
            }
            | COutput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(COutputBranch::OwnedFields),
                children,
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
                    COutput::Branch {
                        operation:
                            prebindgen_declarations::Operation::Resolved(COutputBranch::Fields),
                        children: _,
                    } => OutputNode::Branch {
                        operation: (COutputBranch::Fields).into(),
                        children,
                    },
                    COutput::Branch {
                        operation:
                            prebindgen_declarations::Operation::Resolved(COutputBranch::OwnedFields),
                        children: _,
                    } => OutputNode::Branch {
                        operation: (COutputBranch::OwnedFields).into(),
                        children,
                    },
                    _ => unreachable!(),
                })
            }
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Getter { function }),
                child: output,
            }
            | COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Into { function }),
                child: output,
            } => {
                let output = Box::new(prebindgen_declarations::resolve_getter(
                    flat,
                    ty,
                    function,
                    matches!(
                        tree,
                        COutput::Unary {
                            operation: prebindgen_declarations::Operation::Resolved(
                                COutputUnary::Into { .. }
                            ),
                            ..
                        }
                    ),
                    output,
                    |ty, output| self.output(flat, codec, ty, output, extra, stack, true),
                )?);
                Ok(
                    if matches!(
                        tree,
                        COutput::Unary {
                            operation: prebindgen_declarations::Operation::Resolved(
                                COutputUnary::Into { .. }
                            ),
                            ..
                        }
                    ) {
                        OutputNode::Unary {
                            operation: (COutputUnary::Into {
                                function: function.clone(),
                            })
                            .into(),
                            child: output,
                        }
                    } else {
                        OutputNode::Unary {
                            operation: (COutputUnary::Getter {
                                function: function.clone(),
                            })
                            .into(),
                            child: output,
                        }
                    },
                )
            }
            COutput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(COutputBranch::Combine),
                children: parts,
            } => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                Ok(OutputNode::Branch {
                    operation: (COutputBranch::Combine).into(),
                    children: parts
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Result { error }),
                child: ok,
            } => {
                let (success, failure) =
                    result_parts(ty).ok_or("Result recipe requires Result<T, E>")?;
                if let Some(name) = error {
                    prebindgen_declarations::validate_error_mapper(flat, failure, name)?;
                }
                Ok(OutputNode::Unary {
                    operation: (COutputUnary::Result {
                        error: error.clone(),
                    })
                    .into(),
                    child: Box::new(resolve(success, ok, stack)?),
                })
            }
            COutput::Request(_)
            | COutput::Unary {
                operation: prebindgen_declarations::Operation::Request(_),
                ..
            }
            | COutput::Branch {
                operation: prebindgen_declarations::Operation::Request(_),
                ..
            }
            | COutput::Arrow {
                operation: prebindgen_declarations::Operation::Request(_),
                ..
            } => Err("pending operation has the wrong structural node shape".into()),
        }
    }
}
impl DeclCDefaults {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_input_conversion(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &Type,
        conversion: ValueConversion,
        child: &CInput<CInputRequest, COutputRequest>,
        extra: &(),
        stack: &mut Vec<String>,
    ) -> Result<CInput> {
        let child = prebindgen_declarations::resolve_conversion(
            ty,
            &conversion,
            child,
            stack,
            prebindgen_declarations::Direction::Input,
            |ty, child, stack| self.input(flat, codec, ty, child, extra, stack),
        )?;
        Ok(CInput::Unary {
            operation: (CInputUnary::Convert {
                conversion: Box::new(conversion),
            })
            .into(),
            child: Box::new(child),
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_output_conversion(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &Type,
        conversion: ValueConversion,
        child: &COutput<CInputRequest, COutputRequest>,
        extra: &(),
        stack: &mut Vec<String>,
    ) -> Result<COutput> {
        let child = prebindgen_declarations::resolve_conversion(
            ty,
            &conversion,
            child,
            stack,
            prebindgen_declarations::Direction::Output,
            |ty, child, stack| self.output(flat, codec, ty, child, extra, stack, true),
        )?;
        Ok(COutput::Unary {
            operation: (COutputUnary::Convert {
                conversion: Box::new(conversion),
            })
            .into(),
            child: Box::new(child),
        })
    }
    pub(crate) fn resolve_input_value(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &Type,
        extra: &(),
        stack: &mut Vec<String>,
    ) -> Result<CInput> {
        if type_key(ty) == "()" {
            return Ok(InputNode::Branch {
                operation: (CInputBranch::Fields).into(),
                children: vec![],
            });
        }
        if let Some(c) = prebindgen_declarations::input_value_conversion(flat, ty)? {
            return self.resolve_input_conversion(
                flat,
                codec,
                ty,
                c,
                &CInput::value(),
                extra,
                stack,
            );
        }
        if let Some(tree) = prebindgen_declarations::resolve_container(
            ty,
            !(owned::native_input(flat, ty)
                || (self.input_role.get() == InputRole::EnumField
                    && owned::declaration(flat, ty).is_some())),
            |inner| self.resolve_input_value(flat, codec, inner, extra, stack),
            |child| CInput::Unary {
                operation: (CInputUnary::Optional).into(),
                child: Box::new(child),
            },
            |child| CInput::Unary {
                operation: (CInputUnary::Sequence).into(),
                child: Box::new(child),
            },
        )? {
            return Ok(tree);
        }
        self.input(
            flat,
            codec,
            ty,
            &CInput::Leaf(CInputLeafOp::Value),
            extra,
            stack,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_output_value(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &Type,
        extra: &(),
        stack: &mut Vec<String>,
        getter: bool,
        policy: Option<&prebindgen_declarations::Projection>,
    ) -> Result<COutput> {
        if type_key(ty) == "()" {
            return Ok(OutputNode::Branch {
                operation: (COutputBranch::Fields).into(),
                children: vec![],
            });
        }
        if let Some(c) = prebindgen_declarations::output_value_conversion(flat, ty, policy)? {
            return self.resolve_output_conversion(
                flat,
                codec,
                ty,
                c,
                &COutput::value(),
                extra,
                stack,
            );
        }
        if let Some(tree) = prebindgen_declarations::resolve_container(
            ty,
            !owned::native_output(flat, ty),
            |inner| self.resolve_output_value(flat, codec, inner, extra, stack, getter, policy),
            |child| COutput::Unary {
                operation: (COutputUnary::Optional).into(),
                child: Box::new(child),
            },
            |child| COutput::Unary {
                operation: (COutputUnary::Sequence {
                    delivery: prebindgen_declarations::SequenceDelivery::Value,
                    mode: policy.map(|p| p.mode),
                })
                .into(),
                child: Box::new(child),
            },
        )? {
            return Ok(tree);
        }
        self.output(
            flat,
            codec,
            ty,
            &policy.map_or(COutput::Leaf(COutputLeafOp::Value), |p| {
                COutput::Leaf(COutputLeafOp::Projected(p.clone()))
            }),
            extra,
            stack,
            getter,
        )
    }
}
impl DeclCDefaults {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_input_recipe(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &Type,
        extra: &(),
        entries: &BTreeMap<String, BTreeMap<String, &CInput<CInputRequest, COutputRequest>>>,
        value: bool,
        stack: &mut Vec<String>,
    ) -> Result<CInput> {
        let default = if value {
            CInput::value()
        } else {
            CInput::auto()
        };
        let variants = prebindgen_declarations::resolve_enum_fields(
            flat,
            ty,
            entries,
            &default,
            |ty, recipe| {
                let previous = self.input_role.get();
                if value {
                    self.input_role.set(InputRole::EnumField);
                }
                let result = self.input(flat, codec, ty, recipe, extra, stack);
                self.input_role.set(previous);
                result
            },
        )?;
        Ok(CInput::enum_variants(variants, value))
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_output_recipe(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &Type,
        extra: &(),
        entries: &BTreeMap<String, BTreeMap<String, &COutput<CInputRequest, COutputRequest>>>,
        value: bool,
        stack: &mut Vec<String>,
        getter: bool,
    ) -> Result<COutput> {
        let default = if value {
            COutput::value()
        } else {
            COutput::auto()
        };
        let variants = prebindgen_declarations::resolve_enum_fields(
            flat,
            ty,
            entries,
            &default,
            |ty, recipe| self.output(flat, codec, ty, recipe, extra, stack, getter),
        )?;
        Ok(COutput::enum_variants(variants, value, None))
    }
}
/// Source-aware context for choosing an adapter representation.
/// Nested resolution uses the same defaults, local policies and recursion stack.
pub struct CResolve<'a> {
    pub(crate) declarations: &'a DeclCDefaults,
    /// Source declarations and adapter metadata.
    pub flat: &'a Flat<CSchema>,
    pub(crate) codec: &'a CCodec,
    /// Occurrence settings supplied to nested conversions.
    pub extra: &'a (),
    pub(crate) stack: &'a mut Vec<String>,
    pub(crate) getter: bool,
}
impl CResolve<'_> {
    /// Resolve a nested parameter or callback result.
    pub fn input(
        &mut self,
        ty: &Type,
        tree: &CInput<CInputRequest, COutputRequest>,
    ) -> Result<ResolvedInput<CCodec>> {
        self.declarations
            .input(self.flat, self.codec, ty, tree, self.extra, self.stack)
    }
    /// Resolve a nested result, getter or callback argument.
    pub fn output(
        &mut self,
        ty: &Type,
        tree: &COutput<CInputRequest, COutputRequest>,
    ) -> Result<ResolvedOutput<CCodec>> {
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

impl DeclCDefaults {
    /// Resolve function occurrences into an independently compilable Flat.
    ///
    /// Defaults and local overrides are consumed here. The returned model owns
    /// its compiler policy and occurrence settings; drop this DeclCDefaults value
    /// before calling [`prebindgen_declarations::compile_resolved`] if desired.
    pub fn resolve(
        &self,
        flat: Flat<CSchema>,
        codec: CCodec,
        declarations: &[DeclCFunction],
        extra: (),
    ) -> Result<Flat<ResolvedSchema<CCodec>>> {
        prebindgen_declarations::resolve_bindings(
            flat,
            codec,
            declarations,
            |flat, codec, decl, extra| self.resolve_function(flat, codec, decl, extra),
            extra,
        )
    }
}
impl DeclCDefaults {
    fn resolve_input_representation(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &Type,
        tree: &CInput<CInputRequest, COutputRequest>,
        stack: &mut Vec<String>,
    ) -> Result<CInput> {
        let replacement = (|| {
            if !codec.declared || !matches!(tree, crate::CInput::Leaf(CInputLeafOp::Value)) {
                return Ok(None);
            }
            if flat
                .payload_enums
                .contains_key(&prebindgen_flat::type_key(ty))
                && !flat.type_info(ty).ok().is_some_and(|t| {
                    t.metadata.tagged.is_some()
                        || t.metadata.pointer.is_some()
                        || t.metadata.owned.is_some()
                })
            {
                return Err(format!(
                "ordinary payload enum `{}` requires tagged_union or an explicit field expansion",
                prebindgen_flat::type_key(ty)
            ));
            }
            if flat
                .type_info(ty)
                .ok()
                .is_some_and(|t| t.metadata.tagged.is_some())
            {
                let e = flat.payload_enum(ty)?;
                return Ok(Some(crate::CInput::enum_variants(
                    e.variants
                        .iter()
                        .map(|v| v.fields.iter().map(|_| crate::CInput::value()).collect())
                        .collect(),
                    true,
                )));
            }
            Ok(None)
        })();

        match replacement? {
            Some(tree) => self.input(flat, codec, ty, &tree, &(), stack),
            None => Ok(CInput::Leaf(CInputLeafOp::Value)),
        }
    }
}

impl DeclCDefaults {
    fn resolve_output_representation(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &Type,
        tree: &COutput<CInputRequest, COutputRequest>,
        stack: &mut Vec<String>,
        getter: bool,
    ) -> Result<COutput> {
        let replacement = (|| {
            if !codec.declared || !matches!(tree, crate::COutput::Leaf(COutputLeafOp::Value)) {
                return Ok(None);
            }
            if flat
                .payload_enums
                .contains_key(&prebindgen_flat::type_key(ty))
                && !flat.type_info(ty).ok().is_some_and(|t| {
                    t.metadata.tagged.is_some()
                        || t.metadata.pointer.is_some()
                        || t.metadata.owned.is_some()
                })
            {
                return Err(format!(
                "ordinary payload enum `{}` requires tagged_union or an explicit field expansion",
                prebindgen_flat::type_key(ty)
            ));
            }
            if flat
                .type_info(ty)
                .ok()
                .is_some_and(|t| t.metadata.tagged.is_some())
            {
                let e = flat.payload_enum(ty)?;
                return Ok(Some(crate::COutput::enum_variants(
                    e.variants
                        .iter()
                        .map(|v| v.fields.iter().map(|_| crate::COutput::value()).collect())
                        .collect(),
                    true,
                    None,
                )));
            }
            Ok(None)
        })();

        match replacement? {
            Some(tree) => self.output(flat, codec, ty, &tree, &(), stack, getter),
            None => Ok(match tree {
                COutput::Leaf(COutputLeafOp::Value) => COutput::Leaf(COutputLeafOp::Value),
                COutput::Leaf(COutputLeafOp::Projected(p)) => {
                    COutput::Leaf(COutputLeafOp::Projected(p.clone()))
                }
                _ => unreachable!(),
            }),
        }
    }
}
