//! Direct Input/Output descriptions, defaults and function export choices.
use crate::{
    option_inner, result_parts, sequence_parts, DeclarationCodec, Input, InputRequest, Output,
    OutputRequest, ProjectionMode, Schema, SequenceDelivery,
};
use crate::{InputBranch, InputLeafOp, InputUnary, OutputBranch, OutputLeafOp, OutputUnary};
use crate::{InputNode, OutputNode};
use prebindgen_flat::{same_type, type_key, Flat, Result};
use std::collections::BTreeMap;

#[macro_export]
/// Start a typed input declaration using the production builder API.
macro_rules! expand_param {($($ty:tt)+)=>{$crate::ExpandParamDecl::new($crate::declarations::parse_type(stringify!($($ty)+)))};}
#[macro_export]
/// Start a typed output declaration using the production builder API.
macro_rules! expand_return {($($ty:tt)+)=>{$crate::ExpandReturnDecl::new($crate::declarations::parse_type(stringify!($($ty)+)))};}
#[macro_export]
/// Decompose a source getter's returned record into named field views.
macro_rules! fields {
    ($name:ident) => {
        $crate::FieldsDecl::new($crate::ident(stringify!($name)).expect("function identifier"))
    };
}
#[macro_export]
/// Reference a source function and configure its exported binding.
macro_rules! fun {
    ($name:ident) => {
        $crate::FunctionDecl::new($crate::ident(stringify!($name)).expect("function identifier"))
    };
}

/// Conversion defaults indexed by canonical Rust type during preparation.
/// Values are the same Input/Output trees accepted by per-function overrides.
/// This environment is discarded after prepared conversions have been installed.
pub struct Declarations {
    pub(crate) input_role: std::cell::Cell<crate::InputRole>,
    inputs: BTreeMap<String, Input<InputRequest, OutputRequest>>,
    outputs: BTreeMap<String, Output<OutputRequest, InputRequest>>,
    errors: BTreeMap<String, String>,
}
fn input_body(input: Input<InputRequest, OutputRequest>) -> Input<InputRequest, OutputRequest> {
    match input {
        Input::Branch {
            operation: InputBranch::Selector,
            children: mut arms,
        } if arms.len() == 1 => arms.remove(0),
        Input::Unary {
            operation: InputUnary::Bulk,
            child,
        } => Input::Unary {
            operation: InputUnary::Bulk,
            child: Box::new(input_body(*child)),
        },
        Input::Unary {
            operation: InputUnary::Temporary,
            child,
        } => Input::Unary {
            operation: InputUnary::Temporary,
            child: Box::new(input_body(*child)),
        },
        other => other,
    }
}
fn output_body(output: Output<OutputRequest, InputRequest>) -> Output<OutputRequest, InputRequest> {
    match output {
        Output::Branch {
            operation: OutputBranch::Combine,
            children: mut parts,
        } if parts.len() == 1 => parts.remove(0),
        Output::Unary {
            operation: OutputUnary::Bulk,
            child,
        } => Output::Unary {
            operation: OutputUnary::Bulk,
            child: Box::new(output_body(*child)),
        },
        other => other,
    }
}
impl Declarations {
    /// Register typed Input/Output defaults and explicit error-message functions.
    /// Typed trees use Input::for_type / Output::for_type. Production declaration
    /// builders also convert directly into these trees with `.into()`.
    pub fn new<M: prebindgen_flat::Schema>(
        flat: &Flat<M>,
        inputs: Vec<Input<InputRequest, OutputRequest>>,
        outputs: Vec<Output<OutputRequest, InputRequest>>,
        errors: Vec<(syn::Type, String)>,
    ) -> Result<Self> {
        let mut out = Self {
            input_role: std::cell::Cell::new(crate::InputRole::Parameter),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            errors: BTreeMap::new(),
        };
        for input in inputs {
            let Input::Request(InputRequest::ForType { ty, input }) = input else {
                return Err("input defaults require a source type".into());
            };
            let key = type_key(&crate::callback::input_type(flat, &ty)?);
            if out.inputs.insert(key, input_body(*input)).is_some() {
                return Err("duplicate input expansion".into());
            }
        }
        for output in outputs {
            let Output::Request(OutputRequest::ForType { ty, output }) = output else {
                return Err("output defaults require a source type".into());
            };
            let key = type_key(&flat.resolve_type(&ty)?);
            if out.outputs.insert(key, output_body(*output)).is_some() {
                return Err("duplicate output expansion".into());
            }
        }
        for (ty, function) in errors {
            let key = type_key(&flat.resolve_type(&ty)?);
            if out.errors.insert(key, function).is_some() {
                return Err("duplicate error mapper for source type".into());
            }
        }
        Ok(out)
    }
    /// Clone a source binding for a second explicit sequence delivery policy.
    /// The adapter selects and validates the sequence while resolving the returned declaration.
    pub fn callback_variant<C: DeclarationCodec>(
        &self,
        flat: &mut Flat<C::Schema>,
        _codec: &C,
        decl: &FunctionDecl,
        name: &str,
        _extra: &C::Extra,
    ) -> Result<FunctionDecl>
    where
        <C::Schema as Schema>::Input: Clone,
        <C::Schema as Schema>::Output: Clone,
        <C::Schema as prebindgen_flat::Schema>::Function: Clone,
    {
        let mut function = flat.function(&decl.name)?.clone();
        function.name = crate::ident(name)?;
        function.ret.slot = None;
        if flat.functions.contains_key(name) {
            return Err("callback return function name collision".into());
        }
        flat.functions.insert(name.into(), function);
        let mut variant = decl.clone();
        variant.name = name.into();
        variant.foreign = None;
        variant.callback_return = None;
        variant.ret = Some(Output::Request(OutputRequest::SequenceCallback {
            output: Box::new(decl.ret.clone().unwrap_or_else(Output::auto)),
            structural: decl.ret.is_none(),
        }));
        Ok(variant)
    }
    /// Resolve source-dependent operations, compile them, and atomically install one binding.
    /// Existing converters are preserved unless explicitly replaced.
    pub fn fill<C: DeclarationCodec>(
        &self,
        flat: &mut Flat<C::Schema>,
        codec: &C,
        decl: &FunctionDecl,
        extra: &C::Extra,
    ) -> Result<()> {
        let recipe = self
            .resolve_function(flat, codec, decl, extra)
            .map_err(|e| format!("{}: {e}", decl.name))?;
        codec.compile_declarations(flat, extra)?;
        crate::resolved::compile_function(flat, codec, &decl.name, recipe, extra)
            .map_err(|e| format!("{}: {e}", decl.name))
    }
    pub(crate) fn resolve_function<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        decl: &FunctionDecl,
        extra: &C::Extra,
    ) -> Result<crate::resolved::FunctionRecipe<C>> {
        let source = flat.function(&decl.name)?;
        if decl.callback_return.is_some()
            && source.ret.slot.is_some()
            && decl.ret.is_none()
            && decl.error.is_none()
        {
            return Err("callback_return cannot reconstruct a prebound output slot; supply an explicit output override".into());
        }
        for name in decl.params.keys() {
            if !source.params.iter().any(|p| p.name == name.as_str()) {
                return Err(format!("unknown parameter `{name}`"));
            }
        }
        let mut inputs = vec![];
        for p in &source.params {
            let explicit = decl.params.get(&p.name.to_string());
            inputs.push(if explicit.is_some() || p.ty.slot.is_none() {
                let tree = self.resolve_input(
                    flat,
                    codec,
                    &p.ty.ty,
                    explicit.unwrap_or(&Input::auto()),
                    extra,
                )?;
                Some(tree)
            } else {
                None
            });
        }
        if decl.result_value && (decl.error.is_some() || result_parts(&source.ret.ty).is_none()) {
            return Err(
                "result_as_value requires Result<T, E> and cannot combine with on_error".into(),
            );
        }
        if decl.error.is_some() && result_parts(&source.ret.ty).is_none() {
            return Err("on_error requires a top-level Result<T, E> return".into());
        }
        let output = if decl.ret.is_some() || decl.error.is_some() || source.ret.slot.is_none() {
            let tree = self.function_output(flat, codec, &source.ret.ty, decl, extra)?;
            Some(tree)
        } else {
            None
        };
        Ok(crate::resolved::FunctionRecipe { inputs, output })
    }
    fn function_output<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        decl: &FunctionDecl,
        extra: &C::Extra,
    ) -> Result<crate::ResolvedOutput<C>> {
        let tree = if let Some((_, error)) = result_parts(ty).filter(|_| !decl.result_value) {
            Output::Unary {
                operation: OutputUnary::Result {
                    error: self.error_mapper_for(decl, error).map(str::to_owned),
                },
                child: Box::new(decl.ret.clone().unwrap_or_else(Output::auto)),
            }
        } else {
            decl.ret.clone().unwrap_or_else(Output::auto)
        };
        self.resolve_output(flat, codec, ty, &tree, extra)
    }
    /// Select a function-specific or type-wide explicit error message mapper.
    pub fn error_mapper_for<'a>(
        &'a self,
        decl: &'a FunctionDecl,
        ty: &syn::Type,
    ) -> Option<&'a str> {
        decl.error
            .as_deref()
            .or_else(|| self.errors.get(&type_key(ty)).map(String::as_str))
    }
    /// Prepare owned source-field decomposition, retaining each field's conversion default.
    pub fn owned_record_output<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        extra: &C::Extra,
    ) -> Result<crate::ResolvedOutput<C>> {
        self.resolve_output(
            flat,
            codec,
            ty,
            &Output::Request(OutputRequest::NamedFields {
                overrides: BTreeMap::new(),
                mode: ProjectionMode::Owned,
            }),
            extra,
        )
    }
    /// Resolve a type's default output tree, for example for a typed error channel.
    pub fn output_recipe<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        extra: &C::Extra,
    ) -> Result<crate::ResolvedOutput<C>> {
        self.resolve_output(flat, codec, ty, &Output::auto(), extra)
    }
    /// Resolve every input operation against canonical source declarations before compilation.
    pub fn resolve_input<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &Input<InputRequest, OutputRequest>,
        extra: &C::Extra,
    ) -> Result<crate::ResolvedInput<C>> {
        self.input(flat, codec, ty, tree, extra, &mut vec![])
            .and_then(|tree| codec.finish_input_tree(tree))
    }
    pub(crate) fn input<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &Input<InputRequest, OutputRequest>,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<crate::ResolvedInput<C>> {
        let resolve =
            |ty: &syn::Type, tree: &Input<InputRequest, OutputRequest>, stack: &mut Vec<String>| {
                self.input(flat, codec, ty, tree, extra, stack)
            };
        match tree {
            Input::None => {
                if type_key(ty) == "()" {
                    Ok(InputNode::None)
                } else {
                    Err("None input requires unit".into())
                }
            }
            Input::Branch {
                operation: InputBranch::Variant,
                ..
            } => Err("variant branch requires an enum parent".into()),
            Input::Request(InputRequest::EnumFields(entries)) => {
                self.enum_input_recipe(flat, codec, ty, extra, entries, false, stack)
            }
            Input::Branch {
                operation: InputBranch::Enum { value },
                children: variants,
            } => {
                let source = flat.payload_enum(ty)?;
                if variants.len() != source.variants.len() {
                    return Err("wrong enum variant count".into());
                }
                let variants = source
                    .variants
                    .iter()
                    .zip(variants)
                    .map(|(arm, fields)| {
                        let fields = fields.variant_fields()?;
                        if fields.len() != arm.fields.len() {
                            return Err("wrong enum field count".into());
                        }
                        arm.fields
                            .iter()
                            .zip(fields)
                            .map(|(f, r)| {
                                let previous = self.input_role.get();
                                if *value {
                                    self.input_role.set(crate::InputRole::EnumField);
                                }
                                let result = resolve(&f.ty, r, stack);
                                self.input_role.set(previous);
                                result
                            })
                            .collect::<Result<Vec<_>>>()
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(crate::resolution::resolve_input_enum(
                    codec, variants, *value,
                )?)
            }
            Input::Request(InputRequest::Value) => {
                self.resolve_input_value(flat, codec, ty, extra, stack)
            }
            Input::Request(InputRequest::Conversion(decl)) => {
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
                    &Input::value(),
                    extra,
                    stack,
                )
            }
            Input::Unary {
                operation: InputUnary::Convert { conversion },
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
            Input::Unary {
                operation: InputUnary::Temporary,
                child,
            } => Ok(InputNode::Unary {
                operation: codec.resolve_input_unary(InputUnary::Temporary)?,
                child: Box::new(resolve(
                    &crate::temporary_input_type(ty)?,
                    &input_body((**child).clone()),
                    stack,
                )?),
            }),
            Input::Unary {
                operation: InputUnary::Bulk,
                child,
            } => Ok(InputNode::Unary {
                operation: codec.resolve_input_unary(InputUnary::Bulk)?,
                child: Box::new(resolve(ty, &input_body((**child).clone()), stack)?),
            }),
            Input::Request(InputRequest::Auto) => {
                let key = crate::callback::input_key(flat, ty)?;
                if let Some(tree) = self.inputs.get(&key) {
                    if stack.contains(&key) {
                        return Err(format!("recursive input expansion `{key}`"));
                    }
                    stack.push(key);
                    let result = resolve(ty, tree, stack);
                    stack.pop();
                    return result;
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(InputNode::Unary {
                        operation: codec.resolve_input_unary(InputUnary::Optional)?,
                        child: Box::new(resolve(inner, &Input::auto(), stack)?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty).filter(|_| {
                    !codec.input_collection_is_leaf(flat, ty, self.input_role.get(), extra)
                }) {
                    return Ok(InputNode::Unary {
                        operation: codec.resolve_input_unary(InputUnary::Sequence)?,
                        child: Box::new(resolve(element, &Input::auto(), stack)?),
                    });
                }
                if type_key(ty) == "()" {
                    return Ok(InputNode::None);
                }
                resolve(ty, &Input::Leaf(InputLeafOp::Value), stack)
            }
            Input::Request(InputRequest::ForType { ty: target, input }) => {
                let target = crate::callback::input_type(flat, target)?;
                if crate::callback::input_key(flat, ty)? == type_key(&target) {
                    return resolve(ty, &input_body((**input).clone()), stack);
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(InputNode::Unary {
                        operation: codec.resolve_input_unary(InputUnary::Optional)?,
                        child: Box::new(resolve(inner, tree, stack)?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(InputNode::Unary {
                        operation: codec.resolve_input_unary(InputUnary::Sequence)?,
                        child: Box::new(resolve(element, tree, stack)?),
                    });
                }
                Err("input expansion type does not match occurrence".into())
            }
            Input::Leaf(InputLeafOp::Value) => {
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
                        &Input::value(),
                        extra,
                        stack,
                    );
                }

                if prebindgen_flat::callback_signature(ty)?.is_some() {
                    return resolve(
                        ty,
                        &Input::Request(InputRequest::Callback(Box::new(
                            crate::CallbackDecl::new(ty.clone()),
                        ))),
                        stack,
                    );
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(InputNode::Unary {
                        operation: codec.resolve_input_unary(InputUnary::Optional)?,
                        child: Box::new(resolve(inner, &Input::Leaf(InputLeafOp::Value), stack)?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty).filter(|_| {
                    !codec.input_collection_is_leaf(flat, ty, self.input_role.get(), extra)
                }) {
                    return Ok(InputNode::Unary {
                        operation: codec.resolve_input_unary(InputUnary::Sequence)?,
                        child: Box::new(resolve(element, &Input::auto(), stack)?),
                    });
                }
                if let Some(replacement) = codec.expand_input(
                    &mut crate::resolution::Resolve {
                        declarations: self,
                        flat,
                        codec,
                        extra,
                        stack,
                        getter: false,
                    },
                    ty,
                    tree,
                )? {
                    return Ok(replacement);
                }
                Ok(InputNode::Leaf(
                    codec.resolve_input_leaf(InputLeafOp::Value)?,
                ))
            }
            Input::Arrow {
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
                    operation: codec.resolve_input_arrow(callback.clone())?,
                    domain: Box::new(crate::resolution::resolve_output_arguments(
                        codec,
                        arguments
                            .iter()
                            .zip(&signature.arguments)
                            .map(|(tree, ty)| {
                                self.output(flat, codec, ty, tree, extra, &mut vec![], true)
                            })
                            .collect::<Result<Vec<_>>>()?,
                    )?),
                    codomain: Box::new(resolve(&signature.return_type, codomain, stack)?),
                })
            }
            Input::Request(InputRequest::Callback(declaration)) => {
                let canonical = crate::callback::input_type(flat, ty)?;
                if !same_type(
                    &canonical,
                    &crate::callback::input_type(flat, &declaration.ty)?,
                ) {
                    return Err("callback declaration type does not match occurrence".into());
                }
                let recursion_key = format!("callback:{}", type_key(&canonical));
                if stack.contains(&recursion_key) {
                    return Err("recursive callback conversion".into());
                }
                stack.push(recursion_key);
                let result = (|| {
                    let signature = prebindgen_flat::callback_signature(&canonical)?
                        .ok_or("callback declaration requires impl Fn(...) -> R")?;
                    let mut selected = match self.inputs.get(&type_key(&canonical)) {
                        Some(Input::Request(InputRequest::Callback(default))) => {
                            (**default).clone()
                        }
                        _ => crate::CallbackDecl::new(canonical.clone()),
                    };
                    if declaration.foreign.is_some() {
                        selected.foreign = declaration.foreign.clone();
                    }
                    if declaration.namespace.is_some() {
                        selected.namespace = declaration.namespace.clone();
                    }
                    selected.params.extend(declaration.params.clone());
                    if declaration.ret.is_some() {
                        selected.ret = declaration.ret.clone();
                    }
                    if selected
                        .params
                        .keys()
                        .any(|&i| i >= signature.arguments.len())
                    {
                        return Err("callback parameter override index is out of range".into());
                    }
                    let mut arguments = Vec::with_capacity(signature.arguments.len());
                    for (index, source) in signature.arguments.iter().enumerate() {
                        // Retain the source reference for exact type overrides.
                        // As with getters, referent defaults may expand a borrow;
                        // compilation preserves its lifetime and ownership.
                        fn subject(ty: &syn::Type) -> &syn::Type {
                            match ty {
                                syn::Type::Reference(r) => subject(&r.elem),
                                _ => option_inner(ty).map(subject).unwrap_or(ty),
                            }
                        }
                        let exposed = crate::output_referent(source).unwrap_or(source);
                        let scoped = !selected.params.contains_key(&index)
                            && !self.outputs.contains_key(&type_key(exposed))
                            && !self.outputs.contains_key(&type_key(subject(exposed)));
                        let ownership = if scoped {
                            crate::CallbackArgumentOwnership::Scoped
                        } else {
                            crate::CallbackArgumentOwnership::Transferred
                        };
                        arguments.push(crate::resolution::resolve_callback_argument(
                            codec,
                            self.output(
                                flat,
                                codec,
                                source,
                                selected.params.get(&index).unwrap_or(&Output::auto()),
                                extra,
                                &mut vec![],
                                true,
                            )?,
                            ownership,
                        )?);
                    }
                    let ret = resolve(
                        &signature.return_type,
                        selected.ret.as_ref().unwrap_or(&Input::auto()),
                        stack,
                    )?;
                    Ok(InputNode::Arrow {
                        domain: Box::new(crate::resolution::resolve_output_arguments(
                            codec, arguments,
                        )?),
                        codomain: Box::new(ret),
                        operation: codec.resolve_input_arrow(Box::new(
                            crate::CallbackOperation {
                                ty: canonical,
                                foreign: selected.foreign,
                                namespace: selected.namespace,
                                explicit_return: selected.ret.is_some(),
                            },
                        ))?,
                    })
                })();
                stack.pop();
                result
            }
            Input::Request(InputRequest::Call(function)) => {
                let f = flat.function(function)?;
                resolve(
                    ty,
                    &Input::Branch {
                        operation: InputBranch::Constructor {
                            function: function.clone(),
                        },
                        children: vec![Input::auto(); f.params.len()],
                    },
                    stack,
                )
            }
            Input::Branch {
                operation: InputBranch::Constructor { function },
                children: args,
            } => {
                let f = flat.function(function)?;
                if !same_type(&f.ret.ty, ty) {
                    return Err(format!(
                        "constructor `{function}` returns `{}`, expected `{}`",
                        type_key(&f.ret.ty),
                        type_key(ty)
                    ));
                }
                if args.len() != f.params.len() {
                    return Err(format!("constructor `{function}`: wrong argument count"));
                }
                Ok(InputNode::Branch {
                    operation: codec.resolve_input_branch(InputBranch::Constructor {
                        function: function.clone(),
                    })?,
                    children: f
                        .params
                        .iter()
                        .zip(args)
                        .map(|(p, child)| resolve(&p.ty.ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
            Input::Unary {
                operation: InputUnary::Optional,
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(InputNode::Unary {
                    operation: codec.resolve_input_unary(InputUnary::Optional)?,
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            Input::Unary {
                operation: InputUnary::Sequence,
                child,
            } => {
                let (element, _) =
                    sequence_parts(ty).ok_or("sequence input requires Vec<T>, &[T] or &Vec<T>")?;
                Ok(InputNode::Unary {
                    operation: codec.resolve_input_unary(InputUnary::Sequence)?,
                    child: Box::new(resolve(element, child, stack)?),
                })
            }
            Input::Branch {
                operation: InputBranch::Fields,
                children,
            }
            | Input::Branch {
                operation: InputBranch::Record,
                children,
            } => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(InputNode::None);
                }
                let fields = &flat.record(ty)?.fields;
                if fields.len() != children.len() {
                    return Err(format!(
                        "`{}`: expected {} field converters, got {}",
                        type_key(ty),
                        fields.len(),
                        children.len()
                    ));
                }
                let children = fields
                    .iter()
                    .zip(children)
                    .map(|(f, c)| resolve(&f.ty, c, stack))
                    .collect::<Result<_>>()?;
                Ok(
                    if matches!(
                        tree,
                        Input::Branch {
                            operation: InputBranch::Record,
                            children: _
                        }
                    ) {
                        InputNode::Branch {
                            operation: codec.resolve_input_branch(InputBranch::Record)?,
                            children,
                        }
                    } else {
                        InputNode::Branch {
                            operation: codec.resolve_input_branch(InputBranch::Fields)?,
                            children,
                        }
                    },
                )
            }
            Input::Branch {
                operation: InputBranch::Selector,
                children: arms,
            } => {
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                Ok(InputNode::Branch {
                    operation: codec.resolve_input_branch(InputBranch::Selector)?,
                    children: arms
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
        }
    }
    /// Resolve every output operation, source reference and ownership choice before compilation.
    pub fn resolve_output<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &Output<OutputRequest, InputRequest>,
        extra: &C::Extra,
    ) -> Result<crate::ResolvedOutput<C>> {
        self.output(flat, codec, ty, tree, extra, &mut vec![], false)
            .and_then(|tree| codec.finish_output_tree(tree))
    }
    #[allow(clippy::too_many_arguments)] // Getter defaults differ from the existing plain-reference ABI.
    pub(crate) fn output<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &Output<OutputRequest, InputRequest>,
        extra: &C::Extra,
        stack: &mut Vec<String>,
        getter_result: bool,
    ) -> Result<crate::ResolvedOutput<C>> {
        let resolve = |ty: &syn::Type,
                       tree: &Output<OutputRequest, InputRequest>,
                       stack: &mut Vec<String>| {
            self.output(flat, codec, ty, tree, extra, stack, getter_result)
        };
        if let Some(referent) = crate::output_referent(ty) {
            if (getter_result || !matches!(tree, Output::Leaf(OutputLeafOp::Value)))
                && !matches!(
                    tree,
                    Output::Request(
                        OutputRequest::Auto
                            | OutputRequest::ForType { .. }
                            | OutputRequest::SequenceCallback { .. }
                    )
                )
            {
                return resolve(referent, tree, stack);
            }
        }
        match tree {
            Output::Request(OutputRequest::SequenceCallback { output, structural }) => {
                let output = resolve(ty, output, stack)?;
                codec.sequence_callback(
                    &mut crate::resolution::Resolve {
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

            Output::Unary {
                operation: OutputUnary::CallbackArgument { ownership },
                child,
            } => Ok(crate::resolution::resolve_callback_argument(
                codec,
                resolve(ty, child, stack)?,
                *ownership,
            )?),
            Output::None => {
                if type_key(ty) == "()" {
                    Ok(OutputNode::None)
                } else {
                    Err("None output requires unit".into())
                }
            }
            Output::Arrow { operation, .. } => match *operation {},
            Output::Branch {
                operation: OutputBranch::Variant | OutputBranch::Arguments,
                ..
            } => Err("variant/argument branch requires its parent operation".into()),
            Output::Request(OutputRequest::EnumFields(entries)) => self.enum_output_recipe(
                flat,
                codec,
                crate::output_referent(ty).unwrap_or(ty),
                extra,
                entries,
                false,
                stack,
                getter_result,
            ),
            Output::Branch {
                operation: OutputBranch::Enum { value, mode },
                children: variants,
            } => {
                let source = flat.payload_enum(crate::output_referent(ty).unwrap_or(ty))?;
                if variants.len() != source.variants.len() {
                    return Err("wrong enum variant count".into());
                }
                let variants = source
                    .variants
                    .iter()
                    .zip(variants)
                    .map(|(arm, fields)| {
                        let fields = fields.variant_fields()?;
                        if fields.len() != arm.fields.len() {
                            return Err("wrong enum field count".into());
                        }
                        arm.fields
                            .iter()
                            .zip(fields)
                            .map(|(f, r)| resolve(&f.ty, r, stack))
                            .collect::<Result<Vec<_>>>()
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(crate::resolution::resolve_output_enum(
                    codec, variants, *value, *mode,
                )?)
            }
            Output::Request(OutputRequest::Value { projection }) => self.resolve_output_value(
                flat,
                codec,
                ty,
                extra,
                stack,
                getter_result,
                projection.as_ref(),
            ),
            Output::Request(OutputRequest::Conversion(decl)) => {
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
                    &Output::value(),
                    extra,
                    stack,
                )
            }
            Output::Unary {
                operation: OutputUnary::Convert { conversion },
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
            Output::Unary {
                operation: OutputUnary::Bulk,
                child,
            } => Ok(OutputNode::Unary {
                operation: codec.resolve_output_unary(OutputUnary::Bulk)?,
                child: Box::new(resolve(ty, &output_body((**child).clone()), stack)?),
            }),
            Output::Leaf(OutputLeafOp::Discard) => Ok(OutputNode::Leaf(
                codec.resolve_output_leaf(OutputLeafOp::Discard)?,
            )),
            Output::Unary {
                operation: OutputUnary::Named { name },
                child: output,
            } => Ok(OutputNode::Unary {
                operation: codec.resolve_output_unary(OutputUnary::Named { name: name.clone() })?,
                child: Box::new(resolve(ty, output, stack)?),
            }),
            Output::Request(OutputRequest::Auto) => {
                let key = type_key(ty);
                if let Some(tree) = self.outputs.get(&key) {
                    if stack.contains(&key) {
                        return Err(format!("recursive output expansion `{key}`"));
                    }
                    stack.push(key);
                    let result = resolve(ty, tree, stack);
                    stack.pop();
                    return result;
                }
                if getter_result {
                    if let Some(referent) = crate::output_referent(ty) {
                        return resolve(referent, tree, stack);
                    }
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(OutputNode::Unary {
                        operation: codec.resolve_output_unary(OutputUnary::Optional)?,
                        child: Box::new(resolve(inner, &Output::auto(), stack)?),
                    });
                }
                if let Some((element, _)) =
                    sequence_parts(ty).filter(|_| !codec.output_collection_is_leaf(flat, ty, extra))
                {
                    return Ok(OutputNode::Unary {
                        operation: codec.resolve_output_unary(OutputUnary::Sequence {
                            delivery: SequenceDelivery::Auto,
                            mode: None,
                        })?,
                        child: Box::new(resolve(element, &Output::auto(), stack)?),
                    });
                }
                if type_key(ty) == "()" {
                    return Ok(OutputNode::None);
                }
                resolve(ty, &Output::Leaf(OutputLeafOp::Value), stack)
            }
            Output::Request(OutputRequest::ForType { ty: target, output }) => {
                let target = flat.resolve_type(target)?;
                if same_type(ty, &target) {
                    return resolve(ty, &output_body((**output).clone()), stack);
                }
                if let Some(referent) = crate::output_referent(ty) {
                    return resolve(referent, tree, stack);
                }
                if let Some(inner) = option_inner(ty) {
                    let child = resolve(inner, tree, stack)?;
                    return Ok(if codec.is_discarded_output(&child) {
                        OutputNode::Leaf(codec.resolve_output_leaf(OutputLeafOp::Discard)?)
                    } else {
                        OutputNode::Unary {
                            operation: codec.resolve_output_unary(OutputUnary::Optional)?,
                            child: Box::new(child),
                        }
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    let child = resolve(element, tree, stack)?;
                    return Ok(if codec.is_discarded_output(&child) {
                        OutputNode::Leaf(codec.resolve_output_leaf(OutputLeafOp::Discard)?)
                    } else {
                        OutputNode::Unary {
                            operation: codec.resolve_output_unary(OutputUnary::Sequence {
                                delivery: SequenceDelivery::Auto,
                                mode: None,
                            })?,
                            child: Box::new(child),
                        }
                    });
                }
                Err("output expansion type does not match occurrence".into())
            }
            Output::Leaf(OutputLeafOp::Value) | Output::Leaf(OutputLeafOp::Projected(_)) => {
                let conversion_type = crate::output_referent(ty).unwrap_or(ty);
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
                        crate::output_referent(ty)
                            .and_then(|t| flat.type_info(t).ok())
                            .and_then(|i| i.conversion.output.clone())
                    });
                if let Some(conversion) = converted {
                    if let Output::Leaf(OutputLeafOp::Projected(policy)) = tree {
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
                        &Output::value(),
                        extra,
                        stack,
                    );
                }

                if let Some(inner) = option_inner(ty) {
                    return Ok(OutputNode::Unary {
                        operation: codec.resolve_output_unary(OutputUnary::Optional)?,
                        child: Box::new(resolve(inner, tree, stack)?),
                    });
                }
                if let Some((element, _)) =
                    sequence_parts(ty).filter(|_| !codec.output_collection_is_leaf(flat, ty, extra))
                {
                    let mode = if let Output::Leaf(OutputLeafOp::Projected(p)) = tree {
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
                        operation: codec.resolve_output_unary(OutputUnary::Sequence {
                            delivery: SequenceDelivery::Value,
                            mode,
                        })?,
                        child: Box::new(resolve(element, &Output::auto(), stack)?),
                    });
                }
                if let Some(replacement) = codec.expand_output(
                    &mut crate::resolution::Resolve {
                        declarations: self,
                        flat,
                        codec,
                        extra,
                        stack,
                        getter: getter_result,
                    },
                    ty,
                    tree,
                )? {
                    return Ok(replacement);
                }
                Ok(match tree {
                    Output::Leaf(OutputLeafOp::Value) => {
                        OutputNode::Leaf(codec.resolve_output_leaf(OutputLeafOp::Value)?)
                    }
                    Output::Leaf(OutputLeafOp::Projected(policy)) => OutputNode::Leaf(
                        codec.resolve_output_leaf(OutputLeafOp::Projected(policy.clone()))?,
                    ),
                    _ => unreachable!(),
                })
            }
            Output::Unary {
                operation: OutputUnary::Optional,
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(OutputNode::Unary {
                    operation: codec.resolve_output_unary(OutputUnary::Optional)?,
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            Output::Unary {
                operation: OutputUnary::Sequence { delivery, mode },
                child: element,
            } => {
                let (ty, _) =
                    sequence_parts(ty).ok_or("sequence output requires a collection source")?;
                Ok(OutputNode::Unary {
                    operation: codec.resolve_output_unary(OutputUnary::Sequence {
                        delivery: *delivery,
                        mode: *mode,
                    })?,
                    child: Box::new(resolve(ty, element, stack)?),
                })
            }
            Output::Request(OutputRequest::NamedFields { overrides, mode }) => {
                let fields = &flat.record(ty)?.fields;
                for name in overrides.keys() {
                    if !fields.iter().any(|f| f.name == name.as_str()) {
                        return Err(format!("unknown output field `{name}`"));
                    }
                }
                let children = fields
                    .iter()
                    .map(|f| {
                        resolve(
                            &f.ty,
                            overrides
                                .get(&f.name.to_string())
                                .unwrap_or(&Output::auto()),
                            stack,
                        )
                    })
                    .collect::<Result<_>>()?;
                Ok(if *mode == ProjectionMode::Owned {
                    OutputNode::Branch {
                        operation: codec.resolve_output_branch(OutputBranch::OwnedFields)?,
                        children,
                    }
                } else {
                    OutputNode::Branch {
                        operation: codec.resolve_output_branch(OutputBranch::Fields)?,
                        children,
                    }
                })
            }
            Output::Branch {
                operation: OutputBranch::Fields,
                children,
            }
            | Output::Branch {
                operation: OutputBranch::OwnedFields,
                children,
            }
            | Output::Branch {
                operation: OutputBranch::Record { .. },
                children,
                ..
            } => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(OutputNode::None);
                }
                let fields = &flat.record(ty)?.fields;
                if fields.len() != children.len() {
                    return Err("wrong output field count".into());
                }
                let children = fields
                    .iter()
                    .zip(children)
                    .map(|(f, c)| resolve(&f.ty, c, stack))
                    .collect::<Result<_>>()?;
                Ok(match tree {
                    Output::Branch {
                        operation: OutputBranch::Fields,
                        children: _,
                    } => OutputNode::Branch {
                        operation: codec.resolve_output_branch(OutputBranch::Fields)?,
                        children,
                    },
                    Output::Branch {
                        operation: OutputBranch::OwnedFields,
                        children: _,
                    } => OutputNode::Branch {
                        operation: codec.resolve_output_branch(OutputBranch::OwnedFields)?,
                        children,
                    },
                    Output::Branch {
                        operation: OutputBranch::Record { mode, .. },
                        ..
                    } => OutputNode::Branch {
                        operation: codec
                            .resolve_output_branch(OutputBranch::Record { mode: *mode })?,
                        children,
                    },
                    _ => unreachable!(),
                })
            }
            Output::Unary {
                operation: OutputUnary::Getter { function },
                child: output,
            }
            | Output::Unary {
                operation: OutputUnary::Into { function },
                child: output,
            } => {
                let f = flat.function(function)?;
                let expected = if matches!(
                    tree,
                    Output::Unary {
                        operation: OutputUnary::Into { .. },
                        ..
                    }
                ) {
                    ty.clone()
                } else if matches!(f.params.first().map(|p| &p.ty.ty), Some(syn::Type::Reference(r)) if r.mutability.is_some())
                {
                    syn::parse_quote!(&mut #ty)
                } else {
                    syn::parse_quote!(&#ty)
                };
                if f.params.len() != 1 || !same_type(&f.params[0].ty.ty, &expected) {
                    return Err(format!(
                        "getter `{function}` must take exactly `{}`",
                        type_key(&expected)
                    ));
                }
                // Mutable getters require a retained owner. Compilation decides
                // this once all sibling output views and the codec are known.
                let output =
                    Box::new(self.output(flat, codec, &f.ret.ty, output, extra, stack, true)?);
                Ok(
                    if matches!(
                        tree,
                        Output::Unary {
                            operation: OutputUnary::Into { .. },
                            ..
                        }
                    ) {
                        OutputNode::Unary {
                            operation: codec.resolve_output_unary(OutputUnary::Into {
                                function: function.clone(),
                            })?,
                            child: output,
                        }
                    } else {
                        OutputNode::Unary {
                            operation: codec.resolve_output_unary(OutputUnary::Getter {
                                function: function.clone(),
                            })?,
                            child: output,
                        }
                    },
                )
            }
            Output::Branch {
                operation: OutputBranch::Combine,
                children: parts,
            } => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                Ok(OutputNode::Branch {
                    operation: codec.resolve_output_branch(OutputBranch::Combine)?,
                    children: parts
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
            Output::Unary {
                operation: OutputUnary::Result { error },
                child: ok,
            } => {
                let (success, failure) =
                    result_parts(ty).ok_or("Result recipe requires Result<T, E>")?;
                if let Some(name) = error {
                    let f = flat.function(name)?;
                    let valid = f.params.len() == 1
                        && (same_type(&f.params[0].ty.ty, failure)
                            || matches!(&f.params[0].ty.ty,syn::Type::Reference(r) if r.mutability.is_none()&&same_type(&r.elem,failure)));
                    if !valid || type_key(&f.ret.ty) != "String" {
                        return Err(
                            "Result error mapper must take exactly E or &E and return String"
                                .into(),
                        );
                    }
                }
                Ok(OutputNode::Unary {
                    operation: codec.resolve_output_unary(OutputUnary::Result {
                        error: error.clone(),
                    })?,
                    child: Box::new(resolve(success, ok, stack)?),
                })
            }
        }
    }
}

/// Which sequence return variants an adapter should expose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackReturn {
    /// Keep the ordinary return and add a callback variant.
    Additional,
    /// Expose only the callback variant, including multi-value Kotlin elements
    /// that have no ordinary List representation.
    Only,
}

/// A source callable to export, with optional destination name and occurrence overrides.
#[derive(Clone, Debug)]
pub struct FunctionDecl {
    /// Treat an outer standard Result as ordinary variant data.
    pub result_value: bool,
    /// Also emit an explicit callback-delivery variant for a sequence return.
    pub callback_return: Option<CallbackReturn>,
    /// Unqualified source function name in the flat namespace.
    pub name: String,
    /// Explicit destination name; supported by JNI, currently rejected by C.
    pub foreign: Option<String>,
    /// Input expansion overrides keyed by source parameter name.
    pub params: BTreeMap<String, Input<InputRequest, OutputRequest>>,
    /// Result expansion override, replacing the type default.
    pub ret: Option<Output<OutputRequest, InputRequest>>,
    /// Source mapper for a top-level Result error; it takes E or &E and returns String.
    pub error: Option<String>,
}
impl FunctionDecl {
    /// Reference a source function without overriding names or conversions.
    ///
    /// `name` is the function's unqualified source name.
    pub fn new(name: syn::Ident) -> Self {
        Self {
            result_value: false,
            callback_return: None,
            name: name.to_string(),
            foreign: None,
            params: BTreeMap::new(),
            ret: None,
            error: None,
        }
    }
    /// Extract an undecorated helper's source name, rejecting binding-only settings.
    pub fn source_name(self) -> String {
        self.reference_name(false)
    }
    /// Source reference used by constructor, getter and error-mapper declarations.
    pub(crate) fn reference_name(self, allow_name: bool) -> String {
        assert!(
            self.params.is_empty()
                && self.ret.is_none()
                && self.error.is_none()
                && !self.result_value
                && self.callback_return.is_none(),
            "conversion helpers only support their source name and getter name override"
        );
        assert!(
            allow_name || self.foreign.is_none(),
            "this helper reference does not support renaming"
        );
        self.name
    }
    /// Replace one parameter's default conversion using the production API.
    pub fn expand_param(
        self,
        name: impl AsRef<str>,
        decl: impl Into<Input<InputRequest, OutputRequest>>,
    ) -> Self {
        self.input(name, decl)
    }
    /// Replace this return's default conversion using the production API.
    pub fn expand_return(self, decl: impl Into<Output<OutputRequest, InputRequest>>) -> Self {
        self.output(decl)
    }
    /// Deliver the whole source Result as data instead of routing its Err arm.
    /// Return overrides then address the complete Result. Incompatible with on_error.
    pub fn result_as_value(mut self) -> Self {
        self.result_value = true;
        self
    }
    /// Add callback delivery alongside the ordinary sequence return.
    ///
    /// C emits `<function>_each`, whose visitor receives each element's converted
    /// fields. Kotlin emits `<function>Fold(initial, ..., fold)`, whose folder
    /// receives an accumulator and each element's converted Kotlin values.
    /// Supports one projected sequence, optionally inside Option and an outer Result.
    /// Existing buffer/list returns are preserved. Adapter builders validate the
    /// return shape and generated-name collisions.
    ///
    /// On adapters supporting bulk, expansions stay within each callback element.
    /// The ordinary variant
    /// and callback variant are prepared separately from the same declaration.
    /// For a source returning `Vec<Row>`, this selects a frame per row for the
    /// callback variant; its ordinary return keeps a frame for the collection:
    /// ```
    /// use prebindgen_declarations::{fun, expand_return};
    /// let _ = fun!(rows)
    ///     .expand_return(expand_return!(Row).field_self_into().bulk())
    ///     .callback_return();
    /// ```
    /// For a source returning `Vec<Vec<Row>>`, target the inner collection to
    /// select one bulk frame per batch:
    /// ```
    /// use prebindgen_declarations::{fun, expand_return};
    /// let _ = fun!(batches)
    ///     .callback_return_only()
    ///     .expand_return(expand_return!(Vec<Row>).field_self_into().bulk());
    /// ```
    /// An explicit bulk wrapper around the entire callback sequence is invalid;
    /// apply bulk to its element expansion instead. This also applies to a bulk
    /// wrapper supplied by a whole-collection type default. Nested automatic
    /// sequences stay ordinary inside a callback element; explicit bulk opts in
    /// to packing them. Chain order does not affect these policies, and
    /// per-function expansions override type-wide defaults.
    pub fn callback_return(mut self) -> Self {
        self.callback_return = Some(CallbackReturn::Additional);
        self
    }
    /// Expose only callback delivery, without generating an ordinary return.
    ///
    /// This explicitly permits decomposed multi-value Kotlin elements without
    /// requiring a Kotlin data class. Generated names are the same as for
    /// `callback_return`: C `_each` and Kotlin `Fold`.
    pub fn callback_return_only(mut self) -> Self {
        self.callback_return = Some(CallbackReturn::Only);
        self
    }
    /// Map the error of a top-level `Result<T, E>` return with a source function.
    ///
    /// The helper must take exactly `E` or `&E` and return `String`. Only its
    /// source name is accepted; it is selected for capture but is not exported.
    /// The success value uses normal return defaults and `expand_return` overrides.
    /// Replaces any previously selected mapper. Non-Result returns reject this option.
    pub fn on_error(mut self, mapper: impl Into<FunctionDecl>) -> Self {
        self.error = Some(mapper.into().reference_name(false));
        self
    }
    /// Set a literal destination name, subject to adapter validation.
    ///
    /// The JNI adapter uses it instead of its default camel-case name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.foreign = Some(name.into());
        self
    }
    /// Replace the expansion for one source parameter.
    ///
    /// # Arguments
    ///
    /// * `name` — the parameter's name, as written in the source.
    /// * `decl` — how to build that argument instead of its type's default.
    ///
    /// # Panics
    /// Panics if that parameter already has an override. Resolution checks that the
    /// parameter exists and the expansion type matches.
    pub fn input(
        mut self,
        name: impl AsRef<str>,
        decl: impl Into<Input<InputRequest, OutputRequest>>,
    ) -> Self {
        assert!(
            self.params
                .insert(name.as_ref().into(), decl.into())
                .is_none(),
            "duplicate parameter override"
        );
        self
    }
    /// Replace the result expansion, including any previously assigned override.
    ///
    /// Raw declaration syntax infers the request types automatically. Syntax with
    /// uninhabited request types uses [`crate::DeclarationOutput::into_preparation`] before
    /// registration. Adapter-resolved trees instead go directly to [`crate::compile_output`].
    /// ```
    /// use prebindgen_declarations::{fun,fields,expand_return,DeclarationInput as Input,DeclarationOutput as Output};
    /// let _ = fun!(run).input("x", Input::fields([])).output(Output::Leaf(prebindgen_declarations::frontend::OutputLeafOp::Value));
    /// let _ = fields!(parts).field_override("hidden", Output::Leaf(prebindgen_declarations::frontend::OutputLeafOp::Discard));
    /// let _ = expand_return!(Value).view(Output::getter("length", Output::Leaf(prebindgen_declarations::frontend::OutputLeafOp::Value)));
    /// let resolved: Output = Output::getter("length", Output::Leaf(prebindgen_declarations::frontend::OutputLeafOp::Value));
    /// let _ = fun!(run).output(resolved.into_preparation());
    /// ```
    pub fn output(mut self, decl: impl Into<Output<OutputRequest, InputRequest>>) -> Self {
        self.ret = Some(decl.into());
        self
    }
}

impl From<&str> for FunctionDecl {
    fn from(name: &str) -> Self {
        Self::new(crate::ident(name).expect("source function identifier"))
    }
}
impl From<String> for FunctionDecl {
    fn from(name: String) -> Self {
        Self::from(name.as_str())
    }
}

/// Collect source helper and representation-type names needed by `Flat::parse_selected`.
pub fn source_items<'a>(
    inputs: &[Input<InputRequest, OutputRequest>],
    outputs: &[Output<OutputRequest, InputRequest>],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a FunctionDecl>,
) -> Vec<String> {
    fn representation_types(decl: &crate::ConvertDecl, names: &mut Vec<String>) {
        struct References<'a>(&'a mut Vec<String>);
        impl syn::visit_mut::VisitMut for References<'_> {
            fn visit_type_path_mut(&mut self, path: &mut syn::TypePath) {
                if path.qself.is_none() {
                    if let Some(segment) = path.path.segments.last() {
                        self.0.push(segment.ident.to_string());
                    }
                }
                syn::visit_mut::visit_type_path_mut(self, path);
            }
        }
        for mut ty in decl.representation_types() {
            syn::visit_mut::VisitMut::visit_type_mut(&mut References(names), &mut ty);
        }
    }
    fn input(tree: &Input<InputRequest, OutputRequest>, names: &mut Vec<String>) {
        match tree {
            Input::None => {}
            Input::Branch {
                operation: InputBranch::Variant,
                children,
            } => {
                for child in children {
                    input(child, names);
                }
            }
            Input::Branch {
                operation: InputBranch::Enum { .. },
                children: variants,
                ..
            } => {
                for tree in variants.iter() {
                    input(tree, names);
                }
            }
            Input::Request(InputRequest::EnumFields(variants)) => {
                for tree in variants.values().flat_map(|v| v.values()) {
                    input(tree, names);
                }
            }
            Input::Request(InputRequest::Conversion(d)) => {
                names.extend(d.functions());
                representation_types(d, names);
            }
            Input::Unary {
                operation: InputUnary::Convert { conversion },
                child,
            } => {
                if let prebindgen_flat::ValueSource::Function(f) = &conversion.source {
                    names.push(f.clone());
                }
                input(child, names);
            }
            Input::Arrow {
                operation: callback,
                domain,
                codomain,
            } => {
                let _ = callback;
                output(domain, names);
                input(codomain, names);
            }
            Input::Request(InputRequest::Callback(declaration)) => {
                for tree in declaration.params.values() {
                    output(tree, names);
                }
                if let Some(tree) = &declaration.ret {
                    input(tree, names);
                }
            }
            Input::Request(InputRequest::Call(name)) => names.push(name.clone()),
            Input::Branch {
                operation: InputBranch::Constructor { function },
                children: args,
            } => {
                names.push(function.clone());
                for child in args {
                    input(child, names);
                }
            }
            Input::Request(InputRequest::ForType { input: child, .. })
            | Input::Unary {
                operation: InputUnary::Optional,
                child,
            }
            | Input::Unary {
                operation: InputUnary::Sequence,
                child,
            }
            | Input::Unary {
                operation: InputUnary::Bulk,
                child,
            }
            | Input::Unary {
                operation: InputUnary::Temporary,
                child,
            } => input(child, names),
            Input::Branch {
                operation: InputBranch::Fields,
                children,
            }
            | Input::Branch {
                operation: InputBranch::Record,
                children,
            }
            | Input::Branch {
                operation: InputBranch::Selector,
                children,
            } => {
                for child in children {
                    input(child, names)
                }
            }
            Input::Leaf(InputLeafOp::Value)
            | Input::Request(InputRequest::Auto)
            | Input::Request(InputRequest::Value) => {}
        }
    }
    fn output(tree: &Output<OutputRequest, InputRequest>, names: &mut Vec<String>) {
        match tree {
            Output::Request(OutputRequest::SequenceCallback { output: child, .. }) => {
                output(child, names)
            }
            Output::Unary {
                operation: OutputUnary::CallbackArgument { .. },
                child,
            } => output(child, names),
            Output::None => {}
            Output::Arrow { operation, .. } => match *operation {},
            Output::Branch {
                operation: OutputBranch::Variant | OutputBranch::Arguments,
                children,
            } => {
                for child in children {
                    output(child, names);
                }
            }
            Output::Branch {
                operation: OutputBranch::Enum { .. },
                children: variants,
                ..
            } => {
                for tree in variants.iter() {
                    output(tree, names);
                }
            }
            Output::Request(OutputRequest::EnumFields(variants)) => {
                for tree in variants.values().flat_map(|v| v.values()) {
                    output(tree, names);
                }
            }
            Output::Request(OutputRequest::Conversion(d)) => {
                names.extend(d.functions());
                representation_types(d, names);
            }
            Output::Unary {
                operation: OutputUnary::Convert { conversion },
                child,
            } => {
                if let prebindgen_flat::ValueSource::Function(f) = &conversion.source {
                    names.push(f.clone());
                }
                output(child, names);
            }
            Output::Unary {
                operation: OutputUnary::Named { .. },
                child,
                ..
            } => output(child, names),
            Output::Leaf(OutputLeafOp::Discard) => {}
            Output::Unary {
                operation: OutputUnary::Getter { function },
                child,
            }
            | Output::Unary {
                operation: OutputUnary::Into { function },
                child,
            } => {
                names.push(function.clone());
                output(child, names);
            }
            Output::Request(OutputRequest::ForType { output: child, .. })
            | Output::Unary {
                operation: OutputUnary::Optional,
                child,
            }
            | Output::Unary {
                operation: OutputUnary::Sequence { .. },
                child,
                ..
            }
            | Output::Unary {
                operation: OutputUnary::Bulk,
                child,
            } => output(child, names),
            Output::Unary {
                operation: OutputUnary::Result { error },
                child: ok,
            } => {
                output(ok, names);
                if let Some(name) = error {
                    names.push(name.clone());
                }
            }
            Output::Branch {
                operation: OutputBranch::Fields,
                children,
            }
            | Output::Branch {
                operation: OutputBranch::OwnedFields,
                children,
            }
            | Output::Branch {
                operation: OutputBranch::Record { .. },
                children,
                ..
            }
            | Output::Branch {
                operation: OutputBranch::Combine,
                children,
            } => {
                for child in children {
                    output(child, names)
                }
            }
            Output::Request(OutputRequest::NamedFields { overrides, .. }) => {
                for child in overrides.values() {
                    output(child, names)
                }
            }
            Output::Request(OutputRequest::Auto)
            | Output::Request(OutputRequest::Value { .. })
            | Output::Leaf(OutputLeafOp::Value)
            | Output::Leaf(OutputLeafOp::Projected(_)) => {}
        }
    }
    let mut names = vec![];
    for tree in inputs {
        input(tree, &mut names)
    }
    for tree in outputs {
        output(tree, &mut names)
    }
    names.extend(errors.iter().map(|(_, name)| name.clone()));
    for f in functions {
        names.push(f.name.clone());
        for tree in f.params.values() {
            input(tree, &mut names)
        }
        if let Some(tree) = &f.ret {
            output(tree, &mut names)
        }
        if let Some(name) = &f.error {
            names.push(name.clone())
        }
    }
    names
}
/// Parse source type syntax without requiring it to be in scope in build.rs.
pub fn parse_type(text: &str) -> syn::Type {
    syn::parse_str(text).expect("source type syntax")
}
