//! Direct Input/Output descriptions, defaults and function export choices.
use crate::{
    option_inner, result_parts, sequence_parts, Codec, Compiler, Condition, Input, InputRequest,
    Output, OutputRequest, ProjectionMode, Schema, SequenceDelivery,
};
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
    inputs: BTreeMap<String, Input<InputRequest>>,
    outputs: BTreeMap<String, Output<OutputRequest>>,
    errors: BTreeMap<String, String>,
}
fn input_body(input: Input<InputRequest>) -> Input<InputRequest> {
    match input {
        Input::Selector(mut arms) if arms.len() == 1 => arms.remove(0),
        other => other,
    }
}
fn output_body(output: Output<OutputRequest>) -> Output<OutputRequest> {
    match output {
        Output::Combine(mut parts) if parts.len() == 1 => parts.remove(0),
        other => other,
    }
}
impl Declarations {
    /// Register typed Input/Output defaults and explicit error-message functions.
    /// Typed trees use Input::for_type / Output::for_type. Production declaration
    /// builders also convert directly into these trees with `.into()`.
    pub fn new<M: prebindgen_flat::Schema>(
        flat: &Flat<M>,
        inputs: Vec<Input<InputRequest>>,
        outputs: Vec<Output<OutputRequest>>,
        errors: Vec<(syn::Type, String)>,
    ) -> Result<Self> {
        let mut out = Self {
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            errors: BTreeMap::new(),
        };
        for input in inputs {
            let Input::Request(InputRequest::ForType { ty, input }) = input else {
                return Err("input defaults require a source type".into());
            };
            if crate::contains_result(&ty) {
                return Err("Result expansions are unsupported; configure success and error mapping separately".into());
            }
            let key = type_key(&crate::callback::input_type(flat, &ty)?);
            if out.inputs.insert(key, input_body(*input)).is_some() {
                return Err("duplicate input expansion".into());
            }
        }
        for output in outputs {
            let Output::Request(OutputRequest::ForType { ty, output }) = output else {
                return Err("output defaults require a source type".into());
            };
            if crate::contains_result(&ty) {
                return Err("Result expansions are unsupported; configure success and error mapping separately".into());
            }
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
    pub fn callback_variant<C: Codec>(
        &self,
        flat: &mut Flat<C::Schema>,
        codec: &C,
        decl: &FunctionDecl,
        name: &str,
        extra: &C::Extra,
    ) -> Result<FunctionDecl>
    where
        <C::Schema as Schema>::Input: Clone,
        <C::Schema as Schema>::Output: Clone,
        <C::Schema as prebindgen_flat::Schema>::Function: Clone,
    {
        let source = flat.function(&decl.name)?;
        let ty = result_parts(&source.ret.ty).map_or(&source.ret.ty, |(ok, _)| ok);
        let mut output = self.resolve_output(
            flat,
            codec,
            ty,
            decl.ret.as_ref().unwrap_or(&Output::auto()),
            extra,
        )?;
        fn select(output: &mut Output) -> usize {
            match output {
                Output::Sequence { delivery, .. } => {
                    *delivery = SequenceDelivery::Callback;
                    1
                }
                Output::Optional(child)
                | Output::Named { output: child, .. }
                | Output::Getter { output: child, .. }
                | Output::Into { output: child, .. } => select(child),
                Output::Combine(children)
                | Output::Fields(children)
                | Output::OwnedFields(children)
                | Output::Record {
                    fields: children, ..
                } => children.iter_mut().map(select).sum(),
                _ => 0,
            }
        }
        if select(&mut output) != 1 {
            return Err("callback_return requires exactly one outer sequence".into());
        }
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
        variant.ret = Some(output.into_preparation());
        Ok(variant)
    }
    /// Resolve source-dependent operations, compile them, and atomically install one binding.
    /// Existing converters are preserved unless explicitly replaced.
    pub fn fill<C: Codec>(
        &self,
        flat: &mut Flat<C::Schema>,
        codec: &C,
        decl: &FunctionDecl,
        extra: &C::Extra,
    ) -> Result<()> {
        self.fill_function(flat, codec, decl, extra)
            .map_err(|error| format!("{}: {error}", decl.name))
    }
    fn fill_function<C: Codec>(
        &self,
        flat: &mut Flat<C::Schema>,
        codec: &C,
        decl: &FunctionDecl,
        extra: &C::Extra,
    ) -> Result<()> {
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
        for (index, p) in source.params.iter().enumerate() {
            let explicit = decl.params.get(&p.name.to_string());
            inputs.push(if explicit.is_some() || p.ty.slot.is_none() {
                let tree = self.resolve_input(
                    flat,
                    codec,
                    &p.ty.ty,
                    explicit.unwrap_or(&Input::auto()),
                    extra,
                )?;
                let compiler = Compiler::new(flat, codec, format!("p{index}"));
                // Keep direct callback parameter names stable while using the
                // same conversion resolution as constructor occurrences.
                let callback_wire = quote::format_ident!("p{index}_callback");
                let callback_value = quote::format_ident!("p{index}_callback_value");
                let names = matches!(&tree, Input::Callback(_))
                    .then_some((&callback_wire, &callback_value));
                Some(compiler.input_named(&p.ty.ty, &tree, extra, &Condition::default(), names)?)
            } else {
                None
            });
        }
        if decl.error.is_some() && result_parts(&source.ret.ty).is_none() {
            return Err("on_error requires a top-level Result<T, E> return".into());
        }
        let mut output = if decl.ret.is_some() || decl.error.is_some() || source.ret.slot.is_none()
        {
            let tree = self.function_output(flat, codec, &source.ret.ty, decl, extra)?;
            Some(Compiler::new(flat, codec, "ret".into()).output_root(
                &source.ret.ty,
                &tree,
                extra,
                prebindgen_conversions::source_result_expr(),
                false,
                true,
            )?)
        } else {
            None
        };
        for input in inputs.iter_mut().flatten() {
            codec.finish_input(input, extra)?;
            codec.validate_input(input)?;
        }
        if let Some(output) = &mut output {
            crate::finalize_output_schema(codec, output, extra)?;
            codec.finish_output(output, extra)?;
            codec.validate_output(output)?;
        }
        let dest = flat.function_mut(&decl.name)?;
        for (p, input) in dest.params.iter_mut().zip(inputs) {
            if let Some(input) = input {
                p.ty.slot = Some(input.into());
            }
        }
        if let Some(output) = output {
            dest.ret.slot = Some(output.into());
        }
        Ok(())
    }
    fn function_output<C: Codec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        decl: &FunctionDecl,
        extra: &C::Extra,
    ) -> Result<Output> {
        let tree = if let Some((_, error)) = result_parts(ty) {
            Output::Result {
                ok: Box::new(decl.ret.clone().unwrap_or_else(Output::auto)),
                error: self.error_mapper_for(decl, error).map(str::to_owned),
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
    pub fn owned_record_output<C: Codec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        extra: &C::Extra,
    ) -> Result<Output> {
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
    pub fn output_recipe<C: Codec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        extra: &C::Extra,
    ) -> Result<Output> {
        self.resolve_output(flat, codec, ty, &Output::auto(), extra)
    }
    /// Resolve every input operation against canonical source declarations before compilation.
    pub fn resolve_input<C: Codec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &Input<InputRequest>,
        extra: &C::Extra,
    ) -> Result<Input> {
        self.input(flat, codec, ty, tree, extra, &mut vec![])
    }
    fn input<C: Codec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &Input<InputRequest>,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<Input> {
        let resolve = |ty: &syn::Type, tree: &Input<InputRequest>, stack: &mut Vec<String>| {
            self.input(flat, codec, ty, tree, extra, stack)
        };
        match tree {
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
                    return Ok(Input::Optional(Box::new(resolve(
                        inner,
                        &Input::auto(),
                        stack,
                    )?)));
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(Input::Sequence(Box::new(resolve(
                        element,
                        &Input::auto(),
                        stack,
                    )?)));
                }
                if type_key(ty) == "()" {
                    return Ok(Input::Fields(vec![]));
                }
                resolve(ty, &Input::Leaf, stack)
            }
            Input::Request(InputRequest::ForType { ty: target, input }) => {
                let target = crate::callback::input_type(flat, target)?;
                if crate::callback::input_key(flat, ty)? == type_key(&target) {
                    return resolve(ty, &input_body((**input).clone()), stack);
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(Input::Optional(Box::new(resolve(inner, tree, stack)?)));
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(Input::Sequence(Box::new(resolve(element, tree, stack)?)));
                }
                Err("input expansion type does not match occurrence".into())
            }
            Input::Leaf => {
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
                    return Ok(Input::Optional(Box::new(resolve(
                        inner,
                        &Input::Leaf,
                        stack,
                    )?)));
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(Input::Sequence(Box::new(resolve(
                        element,
                        &Input::auto(),
                        stack,
                    )?)));
                }
                if let Some(replacement) = codec.expand_input(flat, ty, tree, extra)? {
                    return resolve(ty, &replacement, stack);
                }
                Ok(Input::Leaf)
            }
            Input::Callback(callback) => {
                callback.validate_for(flat, ty)?;
                Ok(Input::Callback(callback.clone()))
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
                        // References expose the referred value, except the borrowed
                        // string/slice views handled by ordinary output conversions.
                        let ty = match source {
                            syn::Type::Reference(reference) if reference.mutability.is_none() => {
                                match reference.elem.as_ref() {
                                    syn::Type::Slice(_) => source,
                                    syn::Type::Path(path) if path.path.is_ident("str") => source,
                                    payload => payload,
                                }
                            }
                            _ => source,
                        };
                        arguments.push(self.resolve_output(
                            flat,
                            codec,
                            ty,
                            selected.params.get(&index).unwrap_or(&Output::auto()),
                            extra,
                        )?);
                    }
                    let ret = resolve(
                        &signature.return_type,
                        selected.ret.as_ref().unwrap_or(&Input::auto()),
                        stack,
                    )?;
                    Ok(Input::Callback(Box::new(crate::CallbackInput {
                        ty: canonical,
                        foreign: selected.foreign,
                        namespace: selected.namespace,
                        arguments,
                        return_value: Box::new(ret),
                        explicit_arguments: selected.params.keys().copied().collect(),
                        explicit_return: selected.ret.is_some(),
                    })))
                })();
                stack.pop();
                result
            }
            Input::Request(InputRequest::Call(function)) => {
                let f = flat.function(function)?;
                resolve(
                    ty,
                    &Input::Constructor {
                        function: function.clone(),
                        args: vec![Input::auto(); f.params.len()],
                    },
                    stack,
                )
            }
            Input::Constructor { function, args } => {
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
                Ok(Input::Constructor {
                    function: function.clone(),
                    args: f
                        .params
                        .iter()
                        .zip(args)
                        .map(|(p, child)| resolve(&p.ty.ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
            Input::Optional(child) => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(Input::Optional(Box::new(resolve(inner, child, stack)?)))
            }
            Input::Sequence(child) => {
                let (element, _) =
                    sequence_parts(ty).ok_or("sequence input requires Vec<T>, &[T] or &Vec<T>")?;
                Ok(Input::Sequence(Box::new(resolve(element, child, stack)?)))
            }
            Input::Fields(children) | Input::Record(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(Input::Fields(vec![]));
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
                Ok(if matches!(tree, Input::Record(_)) {
                    Input::Record(children)
                } else {
                    Input::Fields(children)
                })
            }
            Input::Selector(arms) => {
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                Ok(Input::Selector(
                    arms.iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                ))
            }
        }
    }
    /// Resolve every output operation, source reference and ownership choice before compilation.
    pub fn resolve_output<C: Codec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &Output<OutputRequest>,
        extra: &C::Extra,
    ) -> Result<Output> {
        self.output(flat, codec, ty, tree, extra, &mut vec![], false)
    }
    #[allow(clippy::too_many_arguments)] // Getter defaults differ from the existing plain-reference ABI.
    fn output<C: Codec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &Output<OutputRequest>,
        extra: &C::Extra,
        stack: &mut Vec<String>,
        getter_result: bool,
    ) -> Result<Output> {
        let resolve = |ty: &syn::Type, tree: &Output<OutputRequest>, stack: &mut Vec<String>| {
            self.output(flat, codec, ty, tree, extra, stack, getter_result)
        };
        if let Some(referent) = crate::output_referent(ty) {
            if (getter_result || !matches!(tree, Output::Leaf))
                && !matches!(
                    tree,
                    Output::Request(OutputRequest::Auto | OutputRequest::ForType { .. })
                )
            {
                return resolve(referent, tree, stack);
            }
        }
        match tree {
            Output::Discard => Ok(Output::Discard),
            Output::Named { name, output } => Ok(Output::Named {
                name: name.clone(),
                output: Box::new(resolve(ty, output, stack)?),
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
                    return Ok(Output::Optional(Box::new(resolve(
                        inner,
                        &Output::auto(),
                        stack,
                    )?)));
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(Output::Sequence {
                        element: Box::new(resolve(element, &Output::auto(), stack)?),
                        mode: None,
                        delivery: SequenceDelivery::Value,
                    });
                }
                if type_key(ty) == "()" {
                    return Ok(Output::Fields(vec![]));
                }
                resolve(ty, &Output::Leaf, stack)
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
                    return Ok(if matches!(child, Output::Discard) {
                        Output::Discard
                    } else {
                        Output::Optional(Box::new(child))
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    let child = resolve(element, tree, stack)?;
                    return Ok(if matches!(child, Output::Discard) {
                        Output::Discard
                    } else {
                        Output::Sequence {
                            element: Box::new(child),
                            mode: None,
                            delivery: SequenceDelivery::Value,
                        }
                    });
                }
                Err("output expansion type does not match occurrence".into())
            }
            Output::Leaf | Output::Projected(_) => {
                if let Some(inner) = option_inner(ty) {
                    return Ok(Output::Optional(Box::new(resolve(inner, tree, stack)?)));
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    let mode = if let Output::Projected(p) = tree {
                        if !p.fields.is_empty() {
                            return Err(
                                "collection field projections belong on the element type".into()
                            );
                        }
                        Some(p.mode)
                    } else {
                        None
                    };
                    return Ok(Output::Sequence {
                        element: Box::new(resolve(element, &Output::auto(), stack)?),
                        mode,
                        delivery: SequenceDelivery::Value,
                    });
                }
                if let Some(replacement) = codec.expand_output(flat, ty, tree, extra)? {
                    return resolve(ty, &replacement, stack);
                }
                Ok(match tree {
                    Output::Leaf => Output::Leaf,
                    Output::Projected(policy) => Output::Projected(policy.clone()),
                    _ => unreachable!(),
                })
            }
            Output::Optional(child) => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(Output::Optional(Box::new(resolve(inner, child, stack)?)))
            }
            Output::Sequence {
                element,
                mode,
                delivery,
            } => {
                let (ty, _) =
                    sequence_parts(ty).ok_or("sequence output requires a collection source")?;
                Ok(Output::Sequence {
                    element: Box::new(resolve(ty, element, stack)?),
                    mode: *mode,
                    delivery: *delivery,
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
                    Output::OwnedFields(children)
                } else {
                    Output::Fields(children)
                })
            }
            Output::Fields(children)
            | Output::OwnedFields(children)
            | Output::Record {
                fields: children, ..
            } => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(match tree {
                        Output::Fields(_) => Output::Fields(vec![]),
                        Output::OwnedFields(_) => Output::OwnedFields(vec![]),
                        Output::Record { mode, .. } => Output::Record {
                            fields: vec![],
                            mode: *mode,
                        },
                        _ => unreachable!(),
                    });
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
                    Output::Fields(_) => Output::Fields(children),
                    Output::OwnedFields(_) => Output::OwnedFields(children),
                    Output::Record { mode, .. } => Output::Record {
                        fields: children,
                        mode: *mode,
                    },
                    _ => unreachable!(),
                })
            }
            Output::Getter { function, output } | Output::Into { function, output } => {
                let f = flat.function(function)?;
                let expected = if matches!(tree, Output::Into { .. }) {
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
                Ok(if matches!(tree, Output::Into { .. }) {
                    Output::Into {
                        function: function.clone(),
                        output,
                    }
                } else {
                    Output::Getter {
                        function: function.clone(),
                        output,
                    }
                })
            }
            Output::Combine(parts) => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                Ok(Output::Combine(
                    parts
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                ))
            }
            Output::Result { ok, error } => {
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
                Ok(Output::Result {
                    ok: Box::new(resolve(success, ok, stack)?),
                    error: error.clone(),
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
    /// Also emit an explicit callback-delivery variant for a sequence return.
    pub callback_return: Option<CallbackReturn>,
    /// Unqualified source function name in the flat namespace.
    pub name: String,
    /// Explicit destination name; supported by JNI, currently rejected by C.
    pub foreign: Option<String>,
    /// Input expansion overrides keyed by source parameter name.
    pub params: BTreeMap<String, Input<InputRequest>>,
    /// Result expansion override, replacing the type default.
    pub ret: Option<Output<OutputRequest>>,
    /// Source mapper for a top-level Result error; it takes E or &E and returns String.
    pub error: Option<String>,
}
impl FunctionDecl {
    /// Reference a source function without overriding names or conversions.
    ///
    /// `name` is the function's unqualified source name.
    pub fn new(name: syn::Ident) -> Self {
        Self {
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
    pub fn expand_param(self, name: impl AsRef<str>, decl: impl Into<Input<InputRequest>>) -> Self {
        self.input(name, decl)
    }
    /// Replace this return's default conversion using the production API.
    pub fn expand_return(self, decl: crate::ExpandReturnDecl) -> Self {
        self.output(decl)
    }
    /// Add callback delivery alongside the ordinary sequence return.
    ///
    /// C emits `<function>_each`, whose visitor receives each element's converted
    /// fields. Kotlin emits `<function>Fold(initial, ..., fold)`, whose folder
    /// receives an accumulator and each element's converted Kotlin values.
    /// Supports one projected sequence, optionally inside Option and an outer Result.
    /// Existing buffer/list returns are preserved. Adapter builders validate the
    /// return shape and generated-name collisions.
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
    pub fn input(mut self, name: impl AsRef<str>, decl: impl Into<Input<InputRequest>>) -> Self {
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
    /// Raw structural nodes infer the preparation payload automatically. An explicitly
    /// resolved tree must use [`Output::into_preparation`] before registration.
    /// ```
    /// use prebindgen_declarations::{fun,fields,expand_return,Input,Output};
    /// let _ = fun!(run).input("x", Input::fields([])).output(Output::Leaf);
    /// let _ = fields!(parts).field_override("hidden", Output::Discard);
    /// let _ = expand_return!(Value).view(Output::getter("length", Output::Leaf));
    /// let resolved: Output = Output::getter("length", Output::Leaf);
    /// let _ = fun!(run).output(resolved.into_preparation());
    /// ```
    pub fn output(mut self, decl: impl Into<Output<OutputRequest>>) -> Self {
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

/// Collect referenced source helper functions directly from the conversion trees.
pub fn source_functions<'a>(
    inputs: &[Input<InputRequest>],
    outputs: &[Output<OutputRequest>],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a FunctionDecl>,
) -> Vec<String> {
    fn input(tree: &Input<InputRequest>, names: &mut Vec<String>) {
        match tree {
            Input::Callback(callback) => {
                for tree in &callback.arguments {
                    output(&tree.clone().into_preparation(), names);
                }
                input(&(*callback.return_value).clone().into_preparation(), names);
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
            Input::Constructor { function, args } => {
                names.push(function.clone());
                for child in args {
                    input(child, names);
                }
            }
            Input::Request(InputRequest::ForType { input: child, .. })
            | Input::Optional(child)
            | Input::Sequence(child) => input(child, names),
            Input::Fields(children) | Input::Record(children) | Input::Selector(children) => {
                for child in children {
                    input(child, names)
                }
            }
            Input::Leaf | Input::Request(InputRequest::Auto) => {}
        }
    }
    fn output(tree: &Output<OutputRequest>, names: &mut Vec<String>) {
        match tree {
            Output::Named { output: child, .. } => output(child, names),
            Output::Discard => {}
            Output::Getter {
                function,
                output: child,
            }
            | Output::Into {
                function,
                output: child,
            } => {
                names.push(function.clone());
                output(child, names);
            }
            Output::Request(OutputRequest::ForType { output: child, .. })
            | Output::Optional(child)
            | Output::Sequence { element: child, .. } => output(child, names),
            Output::Result { ok, error } => {
                output(ok, names);
                if let Some(name) = error {
                    names.push(name.clone());
                }
            }
            Output::Fields(children)
            | Output::OwnedFields(children)
            | Output::Record {
                fields: children, ..
            }
            | Output::Combine(children) => {
                for child in children {
                    output(child, names)
                }
            }
            Output::Request(OutputRequest::NamedFields { overrides, .. }) => {
                for child in overrides.values() {
                    output(child, names)
                }
            }
            Output::Request(OutputRequest::Auto) | Output::Leaf | Output::Projected(_) => {}
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
