use super::*;
#[derive(Clone)]
/// This environment is discarded after prepared conversions have been installed.
pub struct DeclCDefaults {
    input_role: std::cell::Cell<InputRole>,

    values: prebindgen_declarations::DeclDefaults<CInputConfig, COutputConfig>,
}

fn input_body(input: CInputConfig) -> CInputConfig {
    match input {
        InputNode::Branch {
            operation: CInputBranch::Selector,
            children: mut arms,
        } if arms.len() == 1 => arms.remove(0),
        InputNode::Unary {
            operation: CInputUnary::Temporary,
            child,
        } => InputNode::Unary {
            operation: CInputUnary::Temporary,
            child: Box::new(input_body(*child)),
        },
        other => other,
    }
}
fn output_body(output: COutputConfig) -> COutputConfig {
    match output {
        OutputNode::Branch {
            operation: COutputBranch::Combine,
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
        inputs: Vec<CInputConfig>,
        outputs: Vec<COutputConfig>,
        errors: Vec<(syn::Type, String)>,
    ) -> Result<Self> {
        let inputs = inputs
            .into_iter()
            .map(|tree| match tree {
                InputNode::Unary {
                    operation: CInputUnary::ConfigForType { ty },
                    child: input,
                } => Ok((*ty, input_body(*input))),

                _ => Err("input defaults require a source type".into()),
            })
            .collect::<Result<Vec<_>>>()?;
        let outputs = outputs
            .into_iter()
            .map(|tree| match tree {
                OutputNode::Unary {
                    operation: COutputUnary::ConfigForType { ty },
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
            OutputNode::Unary {
                operation: COutputUnary::ConfigSequenceCallback { structural },
                child: Box::new(output.cloned().unwrap_or_else(COutputConfig::auto)),
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
                self.resolve_input(
                    flat,
                    codec,
                    ty,
                    explicit.unwrap_or(&CInputConfig::auto()),
                    extra,
                )
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
            COutputConfig::auto(),
            |error, child| OutputNode::Unary {
                operation: COutputUnary::Result { error },
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
            &OutputNode::Branch {
                operation: COutputBranch::ConfigNamedFields(CNamedFields {
                    fields: vec![],
                    mode: ProjectionMode::Owned,
                }),
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
        self.resolve_output(flat, codec, ty, &COutputConfig::auto(), extra)
    }
    /// Resolve every input operation against canonical source declarations before compilation.
    pub fn resolve_input(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &syn::Type,
        tree: &CInputConfig,
        extra: &(),
    ) -> Result<CInput> {
        self.input(flat, codec, ty, tree, extra, &mut vec![])
    }
    pub(crate) fn input(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &syn::Type,
        tree: &CInputConfig,
        extra: &(),
        stack: &mut Vec<String>,
    ) -> Result<CInput> {
        let resolve = |ty: &syn::Type, tree: &CInputConfig, stack: &mut Vec<String>| {
            self.input(flat, codec, ty, tree, extra, stack)
        };
        match tree {
            InputNode::None => {
                if type_key(ty) == "()" {
                    Ok(InputNode::None)
                } else {
                    Err("None input requires unit".into())
                }
            }
            InputNode::Branch {
                operation:
                    CInputBranch::Variant | CInputBranch::ConfigVariantFields(CVariantFields { .. }),
                ..
            } => Err("variant branch requires an enum parent".into()),
            InputNode::Branch {
                operation: CInputBranch::ConfigEnumFields(_),
                children,
            } => {
                let entries = prebindgen_declarations::enum_children(
                    children,
                    |tree| match tree {
                        InputNode::Branch {
                            operation:
                                CInputBranch::ConfigVariantFields(CVariantFields { variant, fields }),
                            children,
                        } => Ok((variant.as_str(), fields.as_slice(), children.as_slice())),
                        _ => Err("named enum fields require variant branches".into()),
                    },
                )?;
                self.enum_input_recipe(flat, codec, ty, extra, &entries, false, stack)
            }
            InputNode::Branch {
                operation: CInputBranch::Enum { value },
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
            CInputConfig::Leaf(CInputLeafOp::ConfigValue(_)) => {
                self.resolve_input_value(flat, codec, ty, extra, stack)
            }
            CInputConfig::Leaf(CInputLeafOp::ConfigConversion(decl)) => {
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
                    &CInputConfig::value(),
                    extra,
                    stack,
                )
            }
            InputNode::Unary {
                operation: CInputUnary::Convert { conversion },
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
            InputNode::Unary {
                operation: CInputUnary::Temporary,
                child,
            } => Ok(InputNode::Unary {
                operation: CInputUnary::Temporary,
                child: Box::new(resolve(
                    &prebindgen_declarations::temporary_input_type(ty)?,
                    &input_body((**child).clone()),
                    stack,
                )?),
            }),
            CInputConfig::Leaf(CInputLeafOp::ConfigDefault(_)) => {
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
                    |inner| resolve(inner, &CInputConfig::auto(), stack),
                    |child| InputNode::Unary {
                        operation: CInputUnary::Optional,
                        child: Box::new(child),
                    },
                    |child| InputNode::Unary {
                        operation: CInputUnary::Sequence,
                        child: Box::new(child),
                    },
                )? {
                    return Ok(tree);
                }
                if type_key(ty) == "()" {
                    return Ok(InputNode::None);
                }
                resolve(ty, &InputNode::Leaf(CInputLeafOp::Value), stack)
            }
            InputNode::Unary {
                operation: CInputUnary::ConfigForType { ty: target },
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
                    |child| InputNode::Unary {
                        operation: CInputUnary::Optional,
                        child: Box::new(child),
                    },
                    |child| InputNode::Unary {
                        operation: CInputUnary::Sequence,
                        child: Box::new(child),
                    },
                    prebindgen_declarations::Direction::Input,
                )
            }
            InputNode::Leaf(CInputLeafOp::Value) => {
                if let Some(conversion) = prebindgen_declarations::input_value_conversion(flat, ty)?
                {
                    return self.resolve_input_conversion(
                        flat,
                        codec,
                        ty,
                        conversion,
                        &CInputConfig::value(),
                        extra,
                        stack,
                    );
                }
                if prebindgen_flat::callback_signature(ty)?.is_some() {
                    return resolve(
                        ty,
                        &CInputConfig::from(DeclCCallback::new(ty.clone())),
                        stack,
                    );
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(InputNode::Unary {
                        operation: CInputUnary::Optional,
                        child: Box::new(resolve(
                            inner,
                            &InputNode::Leaf(CInputLeafOp::Value),
                            stack,
                        )?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty).filter(|_| {
                    !(owned::native_input(flat, ty)
                        || (self.input_role.get() == InputRole::EnumField
                            && owned::declaration(flat, ty).is_some()))
                }) {
                    return Ok(InputNode::Unary {
                        operation: CInputUnary::Sequence,
                        child: Box::new(resolve(element, &CInputConfig::auto(), stack)?),
                    });
                }
                self.resolve_input_representation(flat, codec, ty, tree, stack)
            }
            InputNode::Arrow {
                operation: CInputArrow::BoundCallback(callback),
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
                    operation: CInputArrow::BoundCallback(callback.clone()),
                    domain: Box::new(COutput::arguments(arguments)),
                    codomain: Box::new(result),
                })
            }
            InputNode::Arrow {
                operation: CInputArrow::ConfigCallback(declaration),
                domain,
                codomain,
            } => {
                let canonical = prebindgen_declarations::callback::input_key(flat, ty)?;
                let default = match self.values.inputs.get(&canonical) {
                    Some(InputNode::Arrow {
                        operation: CInputArrow::ConfigCallback(config),
                        domain,
                        codomain,
                    }) => Some(prebindgen_declarations::CallbackTree {
                        config,
                        arguments: domain.argument_nodes()?,
                        result: codomain.as_ref(),
                    }),
                    _ => None,
                };
                let (operation, arguments, ret) = prebindgen_declarations::resolve_callback_tree(
                    flat,
                    ty,
                    prebindgen_declarations::CallbackTree {
                        config: declaration,
                        arguments: domain.argument_nodes()?,
                        result: codomain.as_ref(),
                    },
                    default,
                    &self.values.outputs,
                    &CInputConfig::auto(),
                    &COutputConfig::auto(),
                    |tree| matches!(tree, CInputConfig::Leaf(CInputLeafOp::ConfigDefault(_))),
                    stack,
                    resolve,
                    |ty, tree, ownership| {
                        Ok(self
                            .output(flat, codec, ty, tree, extra, &mut vec![], true)?
                            .callback_argument(ownership))
                    },
                )?;
                Ok(InputNode::Arrow {
                    operation: CInputArrow::BoundCallback(Box::new(operation)),
                    domain: Box::new(COutput::arguments(arguments)),
                    codomain: Box::new(ret),
                })
            }

            CInputConfig::Leaf(CInputLeafOp::ConfigConstructor(function)) => {
                let f = flat.function(function)?;
                resolve(
                    ty,
                    &InputNode::Branch {
                        operation: CInputBranch::Constructor {
                            function: function.clone(),
                        },
                        children: vec![CInputConfig::auto(); f.params.len()],
                    },
                    stack,
                )
            }
            InputNode::Branch {
                operation:
                    CInputBranch::ConfigConstructorArguments(CConstructorArguments {
                        function,
                        parameters,
                    }),
                children,
            } => prebindgen_declarations::resolve_constructor_arguments(
                flat,
                ty,
                function,
                parameters,
                children,
                &CInputConfig::auto(),
                |ty, child| resolve(ty, child, stack),
                |function| CInputBranch::Constructor { function },
            ),
            InputNode::Branch {
                operation: CInputBranch::ConfigNamedFields(fields),
                children,
            } => {
                let overrides = prebindgen_declarations::named_children(&fields.fields, children)?;
                let children = prebindgen_declarations::resolve_named_fields(
                    flat,
                    ty,
                    &overrides,
                    &CInputConfig::auto(),
                    |ty, child| resolve(ty, child, stack),
                )?;
                Ok(InputNode::Branch {
                    operation: CInputBranch::Fields,
                    children,
                })
            }
            InputNode::Branch {
                operation: CInputBranch::Constructor { function },
                children: args,
            } => prebindgen_declarations::resolve_constructor(
                flat,
                ty,
                function,
                args,
                |ty, child| resolve(ty, child, stack),
                |function| CInputBranch::Constructor { function },
            ),
            InputNode::Unary {
                operation: CInputUnary::Optional,
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(InputNode::Unary {
                    operation: CInputUnary::Optional,
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            InputNode::Unary {
                operation: CInputUnary::Sequence,
                child,
            } => {
                let (element, _) =
                    sequence_parts(ty).ok_or("sequence input requires Vec<T>, &[T] or &Vec<T>")?;
                Ok(InputNode::Unary {
                    operation: CInputUnary::Sequence,
                    child: Box::new(resolve(element, child, stack)?),
                })
            }
            InputNode::Branch {
                operation: CInputBranch::Fields,
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
                        operation: CInputBranch::Fields,
                        children,
                    }
                })
            }
            InputNode::Branch {
                operation: CInputBranch::Selector,
                children: arms,
            } => {
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                Ok(InputNode::Branch {
                    operation: CInputBranch::Selector,
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
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &syn::Type,
        tree: &COutputConfig,
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
        tree: &COutputConfig,
        extra: &(),
        stack: &mut Vec<String>,
        getter_result: bool,
    ) -> Result<COutput> {
        let resolve = |ty: &syn::Type, tree: &COutputConfig, stack: &mut Vec<String>| {
            self.output(flat, codec, ty, tree, extra, stack, getter_result)
        };
        if let Some(referent) = prebindgen_declarations::output_referent(ty) {
            if (getter_result || !matches!(tree, OutputNode::Leaf(COutputLeafOp::Value)))
                && !matches!(
                    tree,
                    COutputConfig::Leaf(COutputLeafOp::ConfigDefault(_))
                        | OutputNode::Unary {
                            operation: COutputUnary::ConfigForType { .. }
                                | COutputUnary::ConfigSequenceCallback { .. },
                            ..
                        }
                )
            {
                return resolve(referent, tree, stack);
            }
        }
        match tree {
            OutputNode::Unary {
                operation: COutputUnary::ConfigSequenceCallback { structural },
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
            OutputNode::Unary {
                operation: COutputUnary::CallbackArgument { ownership },
                child,
            } => Ok((resolve(ty, child, stack)?).callback_argument(*ownership)),
            OutputNode::None => {
                if type_key(ty) == "()" {
                    Ok(OutputNode::None)
                } else {
                    Err("None output requires unit".into())
                }
            }
            OutputNode::Arrow {
                operation: never, ..
            } => match *never {},
            OutputNode::Branch {
                operation:
                    COutputBranch::Variant
                    | COutputBranch::ConfigVariantFields(CVariantFields { .. })
                    | COutputBranch::Arguments,
                ..
            } => Err("variant/argument branch requires its parent operation".into()),
            OutputNode::Branch {
                operation: COutputBranch::ConfigEnumFields(_),
                children,
            } => {
                let entries = prebindgen_declarations::enum_children(
                    children,
                    |tree| match tree {
                        OutputNode::Branch {
                            operation:
                                COutputBranch::ConfigVariantFields(CVariantFields { variant, fields }),
                            children,
                        } => Ok((variant.as_str(), fields.as_slice(), children.as_slice())),
                        _ => Err("named enum fields require variant branches".into()),
                    },
                )?;
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

            OutputNode::Branch {
                operation: COutputBranch::Enum { value, mode },
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
            COutputConfig::Leaf(COutputLeafOp::ConfigValue(projection)) => self
                .resolve_output_value(
                    flat,
                    codec,
                    ty,
                    extra,
                    stack,
                    getter_result,
                    projection.as_ref(),
                ),
            COutputConfig::Leaf(COutputLeafOp::ConfigConversion(decl)) => {
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
                    &COutputConfig::value(),
                    extra,
                    stack,
                )
            }
            OutputNode::Unary {
                operation: COutputUnary::Convert { conversion },
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
            OutputNode::Leaf(COutputLeafOp::Discard) => {
                Ok(OutputNode::Leaf(COutputLeafOp::Discard))
            }
            OutputNode::Unary {
                operation: COutputUnary::Named { name },
                child: output,
            } => Ok(OutputNode::Unary {
                operation: COutputUnary::Named { name: name.clone() },
                child: Box::new(resolve(ty, output, stack)?),
            }),
            COutputConfig::Leaf(COutputLeafOp::ConfigDefault(_)) => {
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
                    |inner| resolve(inner, &COutputConfig::auto(), stack),
                    |child| OutputNode::Unary {
                        operation: COutputUnary::Optional,
                        child: Box::new(child),
                    },
                    |child| OutputNode::Unary {
                        operation: COutputUnary::Sequence {
                            delivery: SequenceDelivery::Auto,
                            mode: None,
                        },
                        child: Box::new(child),
                    },
                )? {
                    return Ok(tree);
                }
                if type_key(ty) == "()" {
                    return Ok(OutputNode::None);
                }
                resolve(ty, &OutputNode::Leaf(COutputLeafOp::Value), stack)
            }
            OutputNode::Unary {
                operation: COutputUnary::ConfigForType { ty: target },
                child: output,
            } => {
                let target = flat.resolve_type(target)?;
                // Getter results describe a borrowed observation of the logical value.
                // Match local typed overrides at that same logical type, before composition.
                let ty = if getter_result {
                    prebindgen_declarations::output_referent(ty).unwrap_or(ty)
                } else {
                    ty
                };
                let target = if getter_result {
                    prebindgen_declarations::output_referent(&target).unwrap_or(&target)
                } else {
                    &target
                };
                let matched = same_type(ty, target);
                prebindgen_declarations::resolve_typed_expansion(
                    ty,
                    matched,
                    &output_body((**output).clone()),
                    tree,
                    stack,
                    resolve,
                    |child| {
                        if matches!(&child, OutputNode::Leaf(COutputLeafOp::Discard)) {
                            child
                        } else {
                            OutputNode::Unary {
                                operation: COutputUnary::Optional,
                                child: Box::new(child),
                            }
                        }
                    },
                    |child| {
                        if matches!(&child, OutputNode::Leaf(COutputLeafOp::Discard)) {
                            child
                        } else {
                            OutputNode::Unary {
                                operation: COutputUnary::Sequence {
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
            OutputNode::Leaf(COutputLeafOp::Value)
            | OutputNode::Leaf(COutputLeafOp::Projected(_)) => {
                if let Some(conversion) =
                    prebindgen_declarations::output_value_conversion(flat, ty, None)?
                {
                    if let OutputNode::Leaf(COutputLeafOp::Projected(policy)) = tree {
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
                        &COutputConfig::value(),
                        extra,
                        stack,
                    );
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(OutputNode::Unary {
                        operation: COutputUnary::Optional,
                        child: Box::new(resolve(inner, tree, stack)?),
                    });
                }
                if let Some((element, _)) =
                    sequence_parts(ty).filter(|_| !owned::native_output(flat, ty))
                {
                    let mode = if let OutputNode::Leaf(COutputLeafOp::Projected(p)) = tree {
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
                        operation: COutputUnary::Sequence {
                            delivery: SequenceDelivery::Value,
                            mode,
                        },
                        child: Box::new(resolve(element, &COutputConfig::auto(), stack)?),
                    });
                }
                self.resolve_output_representation(flat, codec, ty, tree, stack, getter_result)
            }
            OutputNode::Unary {
                operation: COutputUnary::Optional,
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(OutputNode::Unary {
                    operation: COutputUnary::Optional,
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            OutputNode::Unary {
                operation: COutputUnary::Sequence { delivery, mode },
                child: element,
            } => {
                let (ty, _) =
                    sequence_parts(ty).ok_or("sequence output requires a collection source")?;
                Ok(OutputNode::Unary {
                    operation: COutputUnary::Sequence {
                        delivery: *delivery,
                        mode: *mode,
                    },
                    child: Box::new(resolve(ty, element, stack)?),
                })
            }
            OutputNode::Branch {
                operation: COutputBranch::ConfigNamedFields(CNamedFields { fields, mode }),
                children: recipes,
            } => {
                let overrides = prebindgen_declarations::named_children(fields, recipes)?;
                let children = prebindgen_declarations::resolve_named_fields(
                    flat,
                    ty,
                    &overrides,
                    &COutputConfig::auto(),
                    |ty, tree| resolve(ty, tree, stack),
                )?;
                Ok(if *mode == ProjectionMode::Owned {
                    OutputNode::Branch {
                        operation: COutputBranch::OwnedFields,
                        children,
                    }
                } else {
                    OutputNode::Branch {
                        operation: COutputBranch::Fields,
                        children,
                    }
                })
            }
            OutputNode::Branch {
                operation: COutputBranch::Fields,
                children,
            }
            | OutputNode::Branch {
                operation: COutputBranch::OwnedFields,
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
                    OutputNode::Branch {
                        operation: COutputBranch::Fields,
                        children: _,
                    } => OutputNode::Branch {
                        operation: COutputBranch::Fields,
                        children,
                    },
                    OutputNode::Branch {
                        operation: COutputBranch::OwnedFields,
                        children: _,
                    } => OutputNode::Branch {
                        operation: COutputBranch::OwnedFields,
                        children,
                    },
                    _ => unreachable!(),
                })
            }
            OutputNode::Unary {
                operation: COutputUnary::Getter { function },
                child: output,
            }
            | OutputNode::Unary {
                operation: COutputUnary::Into { function },
                child: output,
            } => {
                let output = Box::new(prebindgen_declarations::resolve_getter(
                    flat,
                    ty,
                    function,
                    matches!(
                        tree,
                        OutputNode::Unary {
                            operation: COutputUnary::Into { .. },
                            ..
                        }
                    ),
                    output,
                    |ty, output| self.output(flat, codec, ty, output, extra, stack, true),
                )?);
                Ok(
                    if matches!(
                        tree,
                        OutputNode::Unary {
                            operation: COutputUnary::Into { .. },
                            ..
                        }
                    ) {
                        OutputNode::Unary {
                            operation: COutputUnary::Into {
                                function: function.clone(),
                            },
                            child: output,
                        }
                    } else {
                        OutputNode::Unary {
                            operation: COutputUnary::Getter {
                                function: function.clone(),
                            },
                            child: output,
                        }
                    },
                )
            }
            OutputNode::Branch {
                operation: COutputBranch::Combine,
                children: parts,
            } => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                Ok(OutputNode::Branch {
                    operation: COutputBranch::Combine,
                    children: parts
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
            OutputNode::Unary {
                operation: COutputUnary::Result { error },
                child: ok,
            } => {
                let (success, failure) =
                    result_parts(ty).ok_or("Result recipe requires Result<T, E>")?;
                if let Some(name) = error {
                    prebindgen_declarations::validate_error_mapper(flat, failure, name)?;
                }
                Ok(OutputNode::Unary {
                    operation: COutputUnary::Result {
                        error: error.clone(),
                    },
                    child: Box::new(resolve(success, ok, stack)?),
                })
            }
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
        child: &CInputConfig,
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
        Ok(InputNode::Unary {
            operation: CInputUnary::Convert {
                conversion: Box::new(conversion),
            },
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
        child: &COutputConfig,
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
        Ok(OutputNode::Unary {
            operation: COutputUnary::Convert {
                conversion: Box::new(conversion),
            },
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
                operation: CInputBranch::Fields,
                children: vec![],
            });
        }
        if let Some(c) = prebindgen_declarations::input_value_conversion(flat, ty)? {
            return self.resolve_input_conversion(
                flat,
                codec,
                ty,
                c,
                &CInputConfig::value(),
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
            |child| InputNode::Unary {
                operation: CInputUnary::Optional,
                child: Box::new(child),
            },
            |child| InputNode::Unary {
                operation: CInputUnary::Sequence,
                child: Box::new(child),
            },
        )? {
            return Ok(tree);
        }
        self.input(
            flat,
            codec,
            ty,
            &InputNode::Leaf(CInputLeafOp::Value),
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
                operation: COutputBranch::Fields,
                children: vec![],
            });
        }
        if let Some(c) = prebindgen_declarations::output_value_conversion(flat, ty, policy)? {
            return self.resolve_output_conversion(
                flat,
                codec,
                ty,
                c,
                &COutputConfig::value(),
                extra,
                stack,
            );
        }
        if let Some(tree) = prebindgen_declarations::resolve_container(
            ty,
            !owned::native_output(flat, ty),
            |inner| self.resolve_output_value(flat, codec, inner, extra, stack, getter, policy),
            |child| OutputNode::Unary {
                operation: COutputUnary::Optional,
                child: Box::new(child),
            },
            |child| OutputNode::Unary {
                operation: COutputUnary::Sequence {
                    delivery: prebindgen_declarations::SequenceDelivery::Value,
                    mode: policy.map(|p| p.mode),
                },
                child: Box::new(child),
            },
        )? {
            return Ok(tree);
        }
        self.output(
            flat,
            codec,
            ty,
            &policy.map_or(OutputNode::Leaf(COutputLeafOp::Value), |p| {
                OutputNode::Leaf(COutputLeafOp::Projected(p.clone()))
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
        entries: &BTreeMap<String, BTreeMap<String, &CInputConfig>>,
        value: bool,
        stack: &mut Vec<String>,
    ) -> Result<CInput> {
        let default = if value {
            CInputConfig::value()
        } else {
            CInputConfig::auto()
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
        entries: &BTreeMap<String, BTreeMap<String, &COutputConfig>>,
        value: bool,
        stack: &mut Vec<String>,
        getter: bool,
    ) -> Result<COutput> {
        let default = if value {
            COutputConfig::value()
        } else {
            COutputConfig::auto()
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
    pub fn input(&mut self, ty: &Type, tree: &CInputConfig) -> Result<ResolvedInput<CCodec>> {
        self.declarations
            .input(self.flat, self.codec, ty, tree, self.extra, self.stack)
    }
    /// Resolve a nested result, getter or callback argument.
    pub fn output(&mut self, ty: &Type, tree: &COutputConfig) -> Result<ResolvedOutput<CCodec>> {
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
        tree: &CInputConfig,
        stack: &mut Vec<String>,
    ) -> Result<CInput> {
        let replacement = (|| {
            if !codec.declared || !matches!(tree, InputNode::Leaf(CInputLeafOp::Value)) {
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
                return Ok(Some(crate::CInputConfig::enum_variants(
                    e.variants
                        .iter()
                        .map(|v| {
                            v.fields
                                .iter()
                                .map(|_| crate::CInputConfig::value())
                                .collect()
                        })
                        .collect(),
                    true,
                )));
            }
            Ok(None)
        })();

        match replacement? {
            Some(tree) => self.input(flat, codec, ty, &tree, &(), stack),
            None => Ok(InputNode::Leaf(CInputLeafOp::Value)),
        }
    }
}

impl DeclCDefaults {
    fn resolve_output_representation(
        &self,
        flat: &Flat<CSchema>,
        codec: &CCodec,
        ty: &Type,
        tree: &COutputConfig,
        stack: &mut Vec<String>,
        getter: bool,
    ) -> Result<COutput> {
        let replacement = (|| {
            if !codec.declared || !matches!(tree, OutputNode::Leaf(COutputLeafOp::Value)) {
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
                return Ok(Some(crate::COutputConfig::enum_variants(
                    e.variants
                        .iter()
                        .map(|v| {
                            v.fields
                                .iter()
                                .map(|_| crate::COutputConfig::value())
                                .collect()
                        })
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
                OutputNode::Leaf(COutputLeafOp::Value) => OutputNode::Leaf(COutputLeafOp::Value),
                OutputNode::Leaf(COutputLeafOp::Projected(p)) => {
                    OutputNode::Leaf(COutputLeafOp::Projected(p.clone()))
                }
                _ => unreachable!(),
            }),
        }
    }
}
