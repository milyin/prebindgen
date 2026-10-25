//! Direct Input/Output descriptions, defaults and function export choices.
use crate::{
    option_inner, result_parts, sequence_parts, Codec, Compiler, Condition, Input, Output,
    ProjectionMode, SequenceDelivery,
};
use prebindgen_flat::{same_type, type_key, Flat, Result};
use std::collections::BTreeMap;

#[macro_export]
/// Start a typed input conversion tree; add direct or constructor alternatives.
macro_rules! expand_param {($($ty:tt)+)=>{$crate::Input::for_type($crate::declarations::parse_type(stringify!($($ty)+)),$crate::Input::Selector(Vec::new()))};}
#[macro_export]
/// Start a typed output conversion tree; append the requested views.
macro_rules! expand_return {($($ty:tt)+)=>{$crate::Output::for_type($crate::declarations::parse_type(stringify!($($ty)+)),$crate::Output::Combine(Vec::new()))};}
#[macro_export]
/// Decompose the named fields returned by a source getter, using field defaults.
macro_rules! fields {
    ($name:ident) => {
        $crate::Output::getter(
            stringify!($name),
            $crate::Output::NamedFields {
                overrides: std::collections::BTreeMap::new(),
                mode: $crate::ProjectionMode::Borrowed,
            },
        )
    };
}
#[macro_export]
/// Reference a source function to export and configure its conversions/placement.
macro_rules! fun {
    ($name:ident) => {
        $crate::FunctionDecl::new($crate::ident(stringify!($name)).expect("function identifier"))
    };
}

/// Conversion defaults indexed by canonical Rust type during preparation.
/// Values are the same Input/Output trees accepted by per-function overrides.
/// This environment is discarded after prepared conversions have been installed.
pub struct Declarations {
    inputs: BTreeMap<String, Input>,
    outputs: BTreeMap<String, Output>,
    errors: BTreeMap<String, String>,
}
fn input_body(input: Input) -> Input {
    match input {
        Input::Selector(mut arms) if arms.len() == 1 => arms.remove(0),
        other => other,
    }
}
fn output_body(output: Output) -> Output {
    match output {
        Output::Combine(mut parts) if parts.len() == 1 => parts.remove(0),
        other => other,
    }
}
impl Declarations {
    /// Register typed Input/Output defaults and explicit error-message functions.
    /// Typed trees are created by Input::for_type / Output::for_type, or the
    /// expand_param! / expand_return! convenience macros; no parallel declaration AST exists.
    pub fn new<I, O, M: prebindgen_flat::Metadata>(
        flat: &Flat<I, O, M>,
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        errors: Vec<(syn::Type, String)>,
    ) -> Result<Self> {
        let mut out = Self {
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            errors: BTreeMap::new(),
        };
        for input in inputs {
            let Input::ForType { ty, input } = input else {
                return Err("input defaults require a source type".into());
            };
            if crate::contains_result(&ty) {
                return Err("Result expansions are unsupported; configure success and error mapping separately".into());
            }
            let key = type_key(&flat.resolve_type(&ty)?);
            if out.inputs.insert(key, input_body(*input)).is_some() {
                return Err("duplicate input expansion".into());
            }
        }
        for output in outputs {
            let Output::ForType { ty, output } = output else {
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
    /// Resolve a source-only Flat into explicit Input/Output occurrence slots.
    /// Source declarations and all metadata are moved into the returned model.
    /// The descriptions contain resolved operations and refer to this model's
    /// canonical declarations; no plans or wire encodings are generated here.
    pub fn resolve<C: Codec>(
        &self,
        flat: Flat<(), (), C::Metadata>,
        codec: &C,
        functions: &[FunctionDecl],
        extra: &C::Extra,
    ) -> Result<Flat<Input, Output, C::Metadata>> {
        let mut flat = flat.map_slots(|_, _| Ok(None), |_, _| Ok(None))?;
        for decl in functions {
            let source = flat.function(&decl.name)?;
            for name in decl.params.keys() {
                if !source.params.iter().any(|p| p.name == name.as_str()) {
                    return Err(format!("`{}`: unknown parameter `{name}`", decl.name));
                }
            }
            if decl.error.is_some() && result_parts(&source.ret.ty).is_none() {
                return Err("on_error requires a top-level Result<T, E> return".into());
            }
            let inputs = source
                .params
                .iter()
                .map(|p| {
                    self.resolve_input(
                        &flat,
                        codec,
                        &p.ty.ty,
                        decl.params.get(&p.name.to_string()).unwrap_or(&Input::Auto),
                        extra,
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            let output = self.function_output(&flat, codec, &source.ret.ty, decl, extra)?;
            let target = flat.function_mut(&decl.name)?;
            for (param, input) in target.params.iter_mut().zip(inputs) {
                param.ty.slot = Some(input);
            }
            target.ret.slot = Some(output);
        }
        Ok(flat)
    }
    /// Clone a source binding for a second explicit sequence delivery policy.
    pub fn callback_variant<C: Codec>(
        &self,
        flat: &mut Flat<C::Input, C::Output, C::Metadata>,
        codec: &C,
        decl: &FunctionDecl,
        name: &str,
        extra: &C::Extra,
    ) -> Result<FunctionDecl>
    where
        C::Input: Clone,
        C::Output: Clone,
        <C::Metadata as prebindgen_flat::Metadata>::Function: Clone,
    {
        let source = flat.function(&decl.name)?;
        let ty = result_parts(&source.ret.ty).map_or(&source.ret.ty, |(ok, _)| ok);
        let mut output = self.resolve_output(
            flat,
            codec,
            ty,
            decl.ret.as_ref().unwrap_or(&Output::Auto),
            extra,
        )?;
        fn select(output: &mut Output) -> usize {
            match output {
                Output::Sequence { delivery, .. } => {
                    *delivery = SequenceDelivery::Callback;
                    1
                }
                Output::Optional(child)
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
        variant.ret = Some(output);
        Ok(variant)
    }
    /// Resolve source-dependent operations, compile them, and atomically install one binding.
    /// Existing converters are preserved unless explicitly replaced.
    pub fn fill<C: Codec>(
        &self,
        flat: &mut Flat<C::Input, C::Output, C::Metadata>,
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
                return Err(format!("`{}`: unknown parameter `{name}`", decl.name));
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
                    explicit.unwrap_or(&Input::Auto),
                    extra,
                )?;
                Some(Compiler::new(flat, codec, format!("p{index}")).input(
                    &p.ty.ty,
                    &tree,
                    extra,
                    &Condition::default(),
                )?)
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
    fn function_output<C: Codec, I, O>(
        &self,
        flat: &Flat<I, O, C::Metadata>,
        codec: &C,
        ty: &syn::Type,
        decl: &FunctionDecl,
        extra: &C::Extra,
    ) -> Result<Output> {
        let tree = if let Some((ok, error)) = result_parts(ty) {
            Output::Result {
                ok: Box::new(self.resolve_output(
                    flat,
                    codec,
                    ok,
                    decl.ret.as_ref().unwrap_or(&Output::Auto),
                    extra,
                )?),
                error: self.error_mapper_for(decl, error).map(str::to_owned),
            }
        } else {
            decl.ret.clone().unwrap_or(Output::Auto)
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
    pub fn owned_record_output<C: Codec, I, O>(
        &self,
        flat: &Flat<I, O, C::Metadata>,
        codec: &C,
        ty: &syn::Type,
        extra: &C::Extra,
    ) -> Result<Output> {
        self.resolve_output(
            flat,
            codec,
            ty,
            &Output::NamedFields {
                overrides: BTreeMap::new(),
                mode: ProjectionMode::Owned,
            },
            extra,
        )
    }
    /// Resolve a type's default output tree, for example for a typed error channel.
    pub fn output_recipe<C: Codec, I, O>(
        &self,
        flat: &Flat<I, O, C::Metadata>,
        codec: &C,
        ty: &syn::Type,
        extra: &C::Extra,
    ) -> Result<Output> {
        self.resolve_output(flat, codec, ty, &Output::Auto, extra)
    }
    /// Resolve every input operation against canonical source declarations before compilation.
    pub fn resolve_input<C: Codec, I, O>(
        &self,
        flat: &Flat<I, O, C::Metadata>,
        codec: &C,
        ty: &syn::Type,
        tree: &Input,
        extra: &C::Extra,
    ) -> Result<Input> {
        self.input(flat, codec, ty, tree, extra, &mut vec![])
    }
    fn input<C: Codec, I, O>(
        &self,
        flat: &Flat<I, O, C::Metadata>,
        codec: &C,
        ty: &syn::Type,
        tree: &Input,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<Input> {
        let resolve = |ty: &syn::Type, tree: &Input, stack: &mut Vec<String>| {
            self.input(flat, codec, ty, tree, extra, stack)
        };
        match tree {
            Input::Auto => {
                let key = type_key(ty);
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
                        &Input::Auto,
                        stack,
                    )?)));
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(Input::Sequence(Box::new(resolve(
                        element,
                        &Input::Auto,
                        stack,
                    )?)));
                }
                if type_key(ty) == "()" {
                    return Ok(Input::Fields(vec![]));
                }
                resolve(ty, &Input::Leaf, stack)
            }
            Input::ForType { ty: target, input } => {
                let target = flat.resolve_type(target)?;
                if same_type(ty, &target) {
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
                if let Some(inner) = option_inner(ty) {
                    return Ok(Input::Optional(Box::new(resolve(
                        inner,
                        &Input::Leaf,
                        stack,
                    )?)));
                }
                if let Some(replacement) = codec.expand_input(flat, ty, tree, extra)? {
                    return resolve(ty, &replacement, stack);
                }
                Ok(Input::Leaf)
            }
            Input::Call(function) => {
                let f = flat.function(function)?;
                resolve(
                    ty,
                    &Input::Constructor {
                        function: function.clone(),
                        args: vec![Input::Auto; f.params.len()],
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
    pub fn resolve_output<C: Codec, I, O>(
        &self,
        flat: &Flat<I, O, C::Metadata>,
        codec: &C,
        ty: &syn::Type,
        tree: &Output,
        extra: &C::Extra,
    ) -> Result<Output> {
        self.output(flat, codec, ty, tree, extra, &mut vec![])
    }
    fn output<C: Codec, I, O>(
        &self,
        flat: &Flat<I, O, C::Metadata>,
        codec: &C,
        ty: &syn::Type,
        tree: &Output,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<Output> {
        let resolve = |ty: &syn::Type, tree: &Output, stack: &mut Vec<String>| {
            self.output(flat, codec, ty, tree, extra, stack)
        };
        match tree {
            Output::Auto => {
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
                if let Some(inner) = option_inner(ty) {
                    return Ok(Output::Optional(Box::new(resolve(
                        inner,
                        &Output::Auto,
                        stack,
                    )?)));
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(Output::Sequence {
                        element: Box::new(resolve(element, &Output::Auto, stack)?),
                        mode: None,
                        delivery: SequenceDelivery::Value,
                    });
                }
                if type_key(ty) == "()" {
                    return Ok(Output::Fields(vec![]));
                }
                resolve(ty, &Output::Leaf, stack)
            }
            Output::ForType { ty: target, output } => {
                let target = flat.resolve_type(target)?;
                if same_type(ty, &target) {
                    return resolve(ty, &output_body((**output).clone()), stack);
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(Output::Optional(Box::new(resolve(inner, tree, stack)?)));
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(Output::Sequence {
                        element: Box::new(resolve(element, tree, stack)?),
                        mode: None,
                        delivery: SequenceDelivery::Value,
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
                        element: Box::new(resolve(element, &Output::Auto, stack)?),
                        mode,
                        delivery: SequenceDelivery::Value,
                    });
                }
                if let Some(replacement) = codec.expand_output(flat, ty, tree, extra)? {
                    return resolve(ty, &replacement, stack);
                }
                Ok(tree.clone())
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
            Output::NamedFields { overrides, mode } => {
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
                            overrides.get(&f.name.to_string()).unwrap_or(&Output::Auto),
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
                    return Ok(tree.clone());
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
                } else {
                    syn::parse_quote!(&#ty)
                };
                if f.params.len() != 1 || !same_type(&f.params[0].ty.ty, &expected) {
                    return Err(format!(
                        "getter `{function}` must take exactly `{}`",
                        type_key(&expected)
                    ));
                }
                if matches!(f.ret.ty, syn::Type::Reference(_)) {
                    return Err("borrowed getter results are outside this subset".into());
                }
                let output = Box::new(resolve(&f.ret.ty, output, stack)?);
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
#[derive(Clone)]
pub struct FunctionDecl {
    /// Also emit an explicit callback-delivery variant for a sequence return.
    pub callback_return: Option<CallbackReturn>,
    /// Unqualified source function name in the flat namespace.
    pub name: String,
    /// Explicit destination name; supported by JNI, currently rejected by C.
    pub foreign: Option<String>,
    /// Input expansion overrides keyed by source parameter name.
    pub params: BTreeMap<String, Input>,
    /// Result expansion override, replacing the type default.
    pub ret: Option<Output>,
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
    pub fn on_error(mut self, mapper: impl Into<String>) -> Self {
        self.error = Some(mapper.into());
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
    pub fn input(mut self, name: impl AsRef<str>, decl: Input) -> Self {
        assert!(
            self.params.insert(name.as_ref().into(), decl).is_none(),
            "duplicate parameter override"
        );
        self
    }
    /// Replace the result expansion, including any previously assigned override.
    pub fn output(mut self, decl: Output) -> Self {
        self.ret = Some(decl);
        self
    }
}

/// Collect referenced source helper functions directly from the conversion trees.
pub fn source_functions<'a>(
    inputs: &[Input],
    outputs: &[Output],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a FunctionDecl>,
) -> Vec<String> {
    fn input(tree: &Input, names: &mut Vec<String>) {
        match tree {
            Input::Call(name) => names.push(name.clone()),
            Input::Constructor { function, args } => {
                names.push(function.clone());
                for child in args {
                    input(child, names);
                }
            }
            Input::ForType { input: child, .. }
            | Input::Optional(child)
            | Input::Sequence(child) => input(child, names),
            Input::Fields(children) | Input::Record(children) | Input::Selector(children) => {
                for child in children {
                    input(child, names)
                }
            }
            Input::Leaf | Input::Auto => {}
        }
    }
    fn output(tree: &Output, names: &mut Vec<String>) {
        match tree {
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
            Output::ForType { output: child, .. }
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
            Output::NamedFields { overrides, .. } => {
                for child in overrides.values() {
                    output(child, names)
                }
            }
            Output::Auto | Output::Leaf | Output::Projected(_) => {}
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
