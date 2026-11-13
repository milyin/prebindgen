//! Source-aware resolution helpers. Each operation accepts only its own dependencies.
use crate::{DeclFunction, FunctionRecipe, InputNode, InputOps, OutputOps};
use prebindgen_flat::{result_parts, same_type, type_key, Flat, Result, Schema};
use std::collections::BTreeMap;
use syn::Type;

/// Type-wide defaults containing the caller's own input and output representations.
#[derive(Clone)]
pub struct DeclDefaults<InTree, OutTree> {
    /// Input defaults indexed by canonical source type or callback signature.
    pub inputs: BTreeMap<String, InTree>,
    /// Output defaults indexed by canonical source type.
    pub outputs: BTreeMap<String, OutTree>,
    /// Explicit source-error message helpers indexed by canonical error type.
    pub errors: BTreeMap<String, String>,
}
impl<I, O> Default for DeclDefaults<I, O> {
    fn default() -> Self {
        Self {
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            errors: BTreeMap::new(),
        }
    }
}

/// Resolve function occurrences, preserving explicitly prepared slots unless overridden.
/// The caller supplies input defaults and its policy for the complete function result.
pub fn resolve_function<M: Schema, I, O, RI, RO>(
    flat: &Flat<M>,
    decl: &DeclFunction<I, O>,
    mut input: impl FnMut(&Type, Option<&I>) -> Result<RI>,
    output: impl FnOnce(&Type) -> Result<RO>,
) -> Result<FunctionRecipe<RI, RO>> {
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
    let inputs = source
        .params
        .iter()
        .map(|p| {
            let explicit = decl.params.get(&p.name.to_string());
            if explicit.is_some() || p.ty.slot.is_none() {
                input(&p.ty.ty, explicit).map(Some)
            } else {
                Ok(None)
            }
        })
        .collect::<Result<Vec<_>>>()?;
    if decl.result_value && (decl.error.is_some() || result_parts(&source.ret.ty).is_none()) {
        return Err(
            "result_as_value requires Result<T, E> and cannot combine with on_error".into(),
        );
    }
    if decl.error.is_some() && result_parts(&source.ret.ty).is_none() {
        return Err("on_error requires a top-level Result<T, E> return".into());
    }
    let output = if decl.ret.is_some() || decl.error.is_some() || source.ret.slot.is_none() {
        Some(output(&source.ret.ty)?)
    } else {
        None
    };
    Ok(FunctionRecipe { inputs, output })
}

/// Check a source constructor and resolve its arguments into the caller's input vocabulary.
pub fn resolve_constructor<I: InputOps, O: OutputOps, IR, OR>(
    flat: &Flat<impl Schema>,
    expected: &Type,
    function: &str,
    arguments: &[InputNode<I, O, IR, OR>],
    mut resolve: impl FnMut(&Type, &InputNode<I, O, IR, OR>) -> Result<InputNode<I, O>>,
    operation: impl FnOnce(String) -> I::Branch,
) -> Result<InputNode<I, O>> {
    let f = flat.function(function)?;
    if !same_type(&f.ret.ty, expected) {
        return Err(format!(
            "constructor `{function}` returns `{}`, expected `{}`",
            type_key(&f.ret.ty),
            type_key(expected)
        ));
    }
    if arguments.len() != f.params.len() {
        return Err(format!("constructor `{function}`: wrong argument count"));
    }
    Ok(InputNode::Branch {
        operation: operation(function.to_owned()),
        children: f
            .params
            .iter()
            .zip(arguments)
            .map(|(p, child)| resolve(&p.ty.ty, child))
            .collect::<Result<_>>()?,
    })
}

/// Resolve one recipe per source record field, checking the declared field count.
pub fn resolve_record_fields<M: Schema, T, R>(
    flat: &Flat<M>,
    ty: &Type,
    recipes: &[T],
    mut resolve: impl FnMut(&Type, &T) -> Result<R>,
) -> Result<Vec<R>> {
    let record = flat.record(ty)?;
    if record.fields.len() != recipes.len() {
        return Err(format!(
            "`{}`: expected {} field recipes, got {}",
            type_key(ty),
            record.fields.len(),
            recipes.len()
        ));
    }
    record
        .fields
        .iter()
        .zip(recipes)
        .map(|(f, r)| resolve(&f.ty, r))
        .collect()
}

/// Resolve named enum field overrides in source order and diagnose unknown names.
pub fn resolve_enum_fields<M: Schema, T, R>(
    flat: &Flat<M>,
    ty: &Type,
    entries: &BTreeMap<String, BTreeMap<String, T>>,
    default: &T,
    mut resolve: impl FnMut(&Type, &T) -> Result<R>,
) -> Result<Vec<Vec<R>>> {
    let source = flat.payload_enum(ty)?;
    crate::validate_enum_overrides(source, entries)?;
    source
        .variants
        .iter()
        .map(|arm| {
            let fields = entries.get(arm.name.to_string().trim_start_matches("r#"));
            arm.fields
                .iter()
                .map(|field| {
                    let recipe = fields.and_then(|f| f.get(&field.key())).unwrap_or(default);
                    resolve(&field.ty, recipe)
                        .map_err(|e| format!("{}::{}.{}: {e}", type_key(ty), arm.name, field.key()))
                })
                .collect()
        })
        .collect()
}

/// Resolve a registered type default with a direction-specific recursion guard.
pub fn resolve_type_default<T, R>(
    key: &str,
    defaults: &BTreeMap<String, T>,
    stack: &mut Vec<String>,
    direction: crate::Direction,
    resolve: impl FnOnce(&T, &mut Vec<String>) -> Result<R>,
) -> Result<Option<R>> {
    let Some(tree) = defaults.get(key) else {
        return Ok(None);
    };
    if stack.iter().any(|k| k == key) {
        let name = match direction {
            crate::Direction::Input => "input",
            crate::Direction::Output => "output",
        };
        return Err(format!("recursive {name} expansion `{key}`"));
    }
    stack.push(key.into());
    let result = resolve(tree, stack);
    stack.pop();
    result.map(Some)
}

/// Apply a typed expansion at its matching source or descend through Option/collection payloads.
// Keep each construction callback explicit instead of requiring an aggregate operations trait.
#[allow(clippy::too_many_arguments)]
pub fn resolve_typed_expansion<T, R>(
    ty: &Type,
    matched: bool,
    body: &T,
    enclosing: &T,
    stack: &mut Vec<String>,
    mut resolve: impl FnMut(&Type, &T, &mut Vec<String>) -> Result<R>,
    optional: impl FnOnce(R) -> R,
    sequence: impl FnOnce(R) -> R,
    direction: crate::Direction,
) -> Result<R> {
    if matched {
        return resolve(ty, body, stack);
    }
    if direction == crate::Direction::Output {
        if let Some(referent) = crate::output_referent(ty) {
            return resolve(referent, enclosing, stack);
        }
    }
    if let Some(inner) = crate::option_inner(ty) {
        return resolve(inner, enclosing, stack).map(optional);
    }
    if let Some((inner, _)) = crate::sequence_parts(ty) {
        return resolve(inner, enclosing, stack).map(sequence);
    }
    let name = match direction {
        crate::Direction::Input => "input",
        crate::Direction::Output => "output",
    };
    Err(format!("{name} expansion type does not match occurrence"))
}

/// Resolve a getter or consuming accessor after checking its exact receiver contract.
pub fn resolve_getter<M: Schema, T, R>(
    flat: &Flat<M>,
    ty: &Type,
    function: &str,
    consuming: bool,
    child: &T,
    resolve: impl FnOnce(&Type, &T) -> Result<R>,
) -> Result<R> {
    let f = flat.function(function)?;
    let expected = if consuming {
        ty.clone()
    } else if matches!(f.params.first().map(|p|&p.ty.ty),Some(Type::Reference(r)) if r.mutability.is_some())
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
    resolve(&f.ret.ty, child)
}

/// Resolve record fields by source name, inheriting defaults for unspecified members.
pub fn resolve_named_fields<M: Schema, T, R>(
    flat: &Flat<M>,
    ty: &Type,
    overrides: &BTreeMap<String, T>,
    default: &T,
    mut resolve: impl FnMut(&Type, &T) -> Result<R>,
) -> Result<Vec<R>> {
    let fields = &flat.record(ty)?.fields;
    for name in overrides.keys() {
        if !fields
            .iter()
            .any(|f| f.name.to_string().trim_start_matches("r#") == name)
        {
            return Err(format!("unknown output field `{name}`"));
        }
    }
    fields
        .iter()
        .map(|f| {
            resolve(
                &f.ty,
                overrides
                    .get(f.name.to_string().trim_start_matches("r#"))
                    .unwrap_or(default),
            )
        })
        .collect()
}

/// Resolve explicit enum branches in source order, checking variant and member counts.
pub fn resolve_enum_variants<M: Schema, T, R>(
    flat: &Flat<M>,
    ty: &Type,
    variants: &[T],
    fields: impl Fn(&T) -> Result<&[T]>,
    mut resolve: impl FnMut(&Type, &T) -> Result<R>,
) -> Result<Vec<Vec<R>>> {
    let source = flat.payload_enum(ty)?;
    if variants.len() != source.variants.len() {
        return Err("wrong enum variant count".into());
    }
    source
        .variants
        .iter()
        .zip(variants)
        .map(|(arm, tree)| {
            let children = fields(tree)?;
            if children.len() != arm.fields.len() {
                return Err("wrong enum field count".into());
            }
            arm.fields
                .iter()
                .zip(children)
                .map(|(field, child)| {
                    resolve(&field.ty, child)
                        .map_err(|e| format!("{}::{}.{}: {e}", type_key(ty), arm.name, field.key()))
                })
                .collect()
        })
        .collect()
}

/// Resolve one canonical value conversion while detecting recursive representation chains.
pub fn resolve_conversion<T, R>(
    ty: &Type,
    conversion: &prebindgen_flat::ValueConversion,
    child: &T,
    stack: &mut Vec<String>,
    direction: crate::Direction,
    resolve: impl FnOnce(&Type, &T, &mut Vec<String>) -> Result<R>,
) -> Result<R> {
    let direction = match direction {
        crate::Direction::Input => "input",
        crate::Direction::Output => "output",
    };
    let key = format!("value-{direction}:{}", type_key(ty));
    if stack.contains(&key) {
        return Err(format!("recursive canonical conversion `{}`", type_key(ty)));
    }
    stack.push(key);
    let result = resolve(&conversion.representation, child, stack);
    stack.pop();
    result
}

/// Resolve an Option or sequence payload, leaving non-container types to the caller.
/// Adapters can keep native collection carriers as leaves by disabling sequence expansion.
/// Child resolution and node construction are independent of the adapter vocabulary.
pub fn resolve_container<T>(
    ty: &Type,
    expand_sequence: bool,
    resolve: impl FnOnce(&Type) -> Result<T>,
    optional: impl FnOnce(T) -> T,
    sequence: impl FnOnce(T) -> T,
) -> Result<Option<T>> {
    if let Some(inner) = prebindgen_flat::option_inner(ty) {
        return resolve(inner).map(optional).map(Some);
    }
    if let Some((element, _)) = prebindgen_flat::sequence_parts(ty).filter(|_| expand_sequence) {
        return resolve(element).map(sequence).map(Some);
    }
    Ok(None)
}

/// Find a canonical input representation, rejecting a type configured only for output.
pub fn input_value_conversion<M: Schema>(
    flat: &Flat<M>,
    ty: &Type,
) -> Result<Option<prebindgen_flat::ValueConversion>> {
    let Some(info) = flat.type_info(ty).ok() else {
        return Ok(None);
    };
    if info.conversion.input.is_none() && info.conversion.output.is_some() {
        return Err(format!(
            "`{}` has no canonical input conversion",
            type_key(ty)
        ));
    }
    Ok(info.conversion.input.clone())
}

/// Override a named field through accessor and typed-expansion wrappers.
/// The adapter identifies the field map and transparent wrapper child; no tree copy is needed.
pub fn override_output_field<T>(
    mut tree: T,
    name: String,
    output: T,
    fields: impl Fn(&mut T) -> Option<&mut BTreeMap<String, T>>,
    child: impl Fn(&mut T) -> Option<&mut T>,
) -> T {
    let mut current = &mut tree;
    loop {
        // Keep the mutable borrows separate: either edit this map or descend.
        if let Some(overrides) = fields(current) {
            assert!(
                overrides.insert(name, output).is_none(),
                "duplicate field override"
            );
            return tree;
        }
        current = child(current).expect("field override requires named field decomposition");
    }
}

/// Find a canonical output representation and validate an explicit projection of it.
pub fn output_value_conversion<M: Schema>(
    flat: &Flat<M>,
    ty: &Type,
    policy: Option<&crate::Projection>,
) -> Result<Option<prebindgen_flat::ValueConversion>> {
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
    let conversion = flat
        .type_info(ty)
        .ok()
        .and_then(|i| i.conversion.output.clone())
        .or_else(|| {
            crate::output_referent(ty)
                .and_then(|t| flat.type_info(t).ok())
                .and_then(|i| i.conversion.output.clone())
        });
    if let Some(c) = &conversion {
        if policy.is_some_and(|p| !p.fields.is_empty()) {
            return Err(
                "field projections on a canonical value require an explicit output tree".into(),
            );
        }
        if policy.is_some_and(|p| p.mode == crate::ProjectionMode::Borrowed)
            && !crate::conversion_borrows(flat, c)?
        {
            return Err(
                "a borrowed projection requires a shared-reference conversion helper".into(),
            );
        }
    }
    Ok(conversion)
}

/// Merge callback defaults and local overrides, then resolve arguments and the result in opposite directions.
#[allow(clippy::too_many_arguments)]
pub fn resolve_callback<M: Schema, I: Clone, O: Clone, RI, RO>(
    flat: &Flat<M>,
    ty: &Type,
    declaration: &crate::DeclCallback<I, O>,
    default: Option<&crate::DeclCallback<I, O>>,
    output_defaults: &BTreeMap<String, O>,
    auto_input: &I,
    auto_output: &O,
    stack: &mut Vec<String>,
    mut input: impl FnMut(&Type, &I, &mut Vec<String>) -> Result<RI>,
    mut output: impl FnMut(&Type, &O, crate::CallbackArgumentOwnership) -> Result<RO>,
) -> Result<(crate::CallbackOperation, Vec<RO>, RI)> {
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
        let mut selected = default
            .cloned()
            .unwrap_or_else(|| crate::DeclCallback::new(canonical.clone()));
        if declaration.foreign.is_some() {
            selected.foreign = declaration.foreign.clone()
        }
        if declaration.namespace.is_some() {
            selected.namespace = declaration.namespace.clone()
        }
        selected.params.extend(declaration.params.clone());
        if declaration.ret.is_some() {
            selected.ret = declaration.ret.clone()
        }
        if selected
            .params
            .keys()
            .any(|&i| i >= signature.arguments.len())
        {
            return Err("callback parameter override index is out of range".into());
        }
        fn subject(ty: &Type) -> &Type {
            match ty {
                Type::Reference(r) => subject(&r.elem),
                _ => crate::option_inner(ty).map(subject).unwrap_or(ty),
            }
        }
        let arguments = signature
            .arguments
            .iter()
            .enumerate()
            .map(|(index, source)| {
                let exposed = crate::output_referent(source).unwrap_or(source);
                let scoped = !selected.params.contains_key(&index)
                    && !output_defaults.contains_key(&type_key(exposed))
                    && !output_defaults.contains_key(&type_key(subject(exposed)));
                let ownership = if scoped {
                    crate::CallbackArgumentOwnership::Scoped
                } else {
                    crate::CallbackArgumentOwnership::Transferred
                };
                output(
                    source,
                    selected.params.get(&index).unwrap_or(auto_output),
                    ownership,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let ret = input(
            &signature.return_type,
            selected.ret.as_ref().unwrap_or(auto_input),
            stack,
        )?;
        Ok((
            crate::CallbackOperation {
                ty: canonical,
                foreign: selected.foreign,
                namespace: selected.namespace,
                explicit_return: selected.ret.is_some(),
            },
            arguments,
            ret,
        ))
    })();
    stack.pop();
    result
}

/// Walk the input-direction children of one node, including an arrow's opposite domain.
pub fn input_children<I: InputOps, O: OutputOps, IR, OR, S>(
    tree: &InputNode<I, O, IR, OR>,
    state: &mut S,
    input: &mut impl FnMut(&InputNode<I, O, IR, OR>, &mut S),
    output: &mut impl FnMut(&crate::OutputNode<I, O, IR, OR>, &mut S),
) {
    match tree {
        InputNode::None | InputNode::Request(_) | InputNode::Leaf(_) => {}
        InputNode::Unary { child, .. } => input(child, state),
        InputNode::Branch { children, .. } => {
            for child in children {
                input(child, state)
            }
        }
        InputNode::Arrow {
            domain, codomain, ..
        } => {
            output(domain, state);
            input(codomain, state)
        }
    }
}
/// Walk the output-direction children of one node, including an arrow's opposite domain.
pub fn output_children<I: InputOps, O: OutputOps, IR, OR, S>(
    tree: &crate::OutputNode<I, O, IR, OR>,
    state: &mut S,
    input: &mut impl FnMut(&InputNode<I, O, IR, OR>, &mut S),
    output: &mut impl FnMut(&crate::OutputNode<I, O, IR, OR>, &mut S),
) {
    match tree {
        crate::OutputNode::None | crate::OutputNode::Request(_) | crate::OutputNode::Leaf(_) => {}
        crate::OutputNode::Unary { child, .. } => output(child, state),
        crate::OutputNode::Branch { children, .. } => {
            for child in children {
                output(child, state)
            }
        }
        crate::OutputNode::Arrow {
            domain, codomain, ..
        } => {
            input(domain, state);
            output(codomain, state)
        }
    }
}
/// Collect helper-function and representation-type references from a value conversion declaration.
pub fn conversion_source_items(decl: &crate::DeclConvert, names: &mut Vec<String>) {
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
    names.extend(decl.functions());
    for mut ty in decl.representation_types() {
        syn::visit_mut::VisitMut::visit_type_mut(&mut References(names), &mut ty);
    }
}
/// Collect selected source references using the caller's tree walkers.
pub fn collect_source_items<'a, I: 'a, O: 'a>(
    inputs: &[I],
    outputs: &[O],
    errors: &[(Type, String)],
    functions: impl IntoIterator<Item = &'a DeclFunction<I, O>>,
    mut input: impl FnMut(&I, &mut Vec<String>),
    mut output: impl FnMut(&O, &mut Vec<String>),
) -> Vec<String> {
    let mut names = Vec::new();
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

impl<I, O> DeclDefaults<I, O> {
    /// Register typed defaults using canonical source identities and reject duplicate declarations.
    pub fn new<M: Schema>(
        flat: &Flat<M>,
        inputs: impl IntoIterator<Item = (Type, I)>,
        outputs: impl IntoIterator<Item = (Type, O)>,
        errors: impl IntoIterator<Item = (Type, String)>,
    ) -> Result<Self> {
        let mut values = Self::default();
        for (ty, input) in inputs {
            let key = type_key(&crate::callback::input_type(flat, &ty)?);
            if values.inputs.insert(key, input).is_some() {
                return Err("duplicate input expansion".into());
            }
        }
        for (ty, output) in outputs {
            let key = type_key(&flat.resolve_type(&ty)?);
            if values.outputs.insert(key, output).is_some() {
                return Err("duplicate output expansion".into());
            }
        }
        for (ty, mapper) in errors {
            let key = type_key(&flat.resolve_type(&ty)?);
            if values.errors.insert(key, mapper).is_some() {
                return Err("duplicate error mapper for source type".into());
            }
        }
        Ok(values)
    }
}
/// Choose a function's result recipe, including its explicit or type-wide source-error mapper.
pub fn function_output<I, O: Clone>(
    ty: &Type,
    decl: &DeclFunction<I, O>,
    errors: &BTreeMap<String, String>,
    auto: O,
    result: impl FnOnce(Option<String>, O) -> O,
) -> O {
    let child = decl.ret.clone().unwrap_or(auto);
    if let Some((_, error)) = crate::result_parts(ty).filter(|_| !decl.result_value) {
        result(
            decl.error
                .clone()
                .or_else(|| errors.get(&type_key(error)).cloned()),
            child,
        )
    } else {
        child
    }
}
/// Check a source error mapper's owned/shared error parameter and String result.
pub fn validate_error_mapper<M: Schema>(flat: &Flat<M>, error: &Type, name: &str) -> Result<()> {
    let f = flat.function(name)?;
    let valid = f.params.len() == 1
        && (same_type(&f.params[0].ty.ty, error)
            || matches!(&f.params[0].ty.ty,Type::Reference(r) if r.mutability.is_none()&&same_type(&r.elem,error)));
    if !valid || type_key(&f.ret.ty) != "String" {
        return Err("Result error mapper must take exactly E or &E and return String".into());
    }
    Ok(())
}
/// Clone a source binding for another sequence delivery policy, retaining existing input slots.
pub fn callback_variant<M: Schema, I: Clone, O: Clone>(
    flat: &mut Flat<M>,
    decl: &DeclFunction<I, O>,
    name: &str,
    output: impl FnOnce(Option<&O>, bool) -> O,
) -> Result<DeclFunction<I, O>>
where
    M::Input: Clone,
    M::Output: Clone,
    M::Function: Clone,
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
    variant.ret = Some(output(decl.ret.as_ref(), decl.ret.is_none()));
    Ok(variant)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;

    #[derive(Clone, Debug)]
    struct ConstructorInputs;
    impl InputOps for ConstructorInputs {
        type Leaf = i64;
        type Unary = Infallible;
        type Branch = String;
        type Arrow = Infallible;
    }
    #[derive(Clone, Debug)]
    struct NoOutputs;
    impl OutputOps for NoOutputs {
        type Leaf = Infallible;
        type Unary = Infallible;
        type Branch = Infallible;
        type Arrow = Infallible;
    }
    type ConstructorInput = InputNode<ConstructorInputs, NoOutputs>;

    fn model(source: &str) -> Flat {
        Flat::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap()
    }

    #[test]
    fn container_fallback_respects_native_collection_leaves_and_child_errors() {
        let resolve = |ty: &Type| Ok(type_key(ty));
        assert_eq!(
            resolve_container(
                &syn::parse_quote!(Option<i64>),
                false,
                resolve,
                |child| format!("optional({child})"),
                |child| format!("sequence({child})")
            )
            .unwrap(),
            Some("optional(i64)".into())
        );
        assert_eq!(
            resolve_container(
                &syn::parse_quote!(Vec<i64>),
                true,
                resolve,
                |child| format!("optional({child})"),
                |child| format!("sequence({child})")
            )
            .unwrap(),
            Some("sequence(i64)".into())
        );
        let native: Option<String> = resolve_container(
            &syn::parse_quote!(Vec<i64>),
            false,
            |_| panic!("native collection must remain a leaf"),
            |_| unreachable!(),
            |_| unreachable!(),
        )
        .unwrap();
        assert!(native.is_none());
        let error = resolve_container(
            &syn::parse_quote!(Option<i64>),
            true,
            |_| Err::<String, _>("child conversion failed".into()),
            |child| child,
            |child| child,
        )
        .unwrap_err();
        assert_eq!(error, "child conversion failed");
    }

    #[test]
    fn named_field_edits_need_no_cloning_or_placeholder_operation() {
        enum View {
            Fields(BTreeMap<String, Self>),
            Wrapper(Box<Self>),
            Value,
        }
        let tree = View::Wrapper(Box::new(View::Wrapper(Box::new(View::Fields(
            BTreeMap::new(),
        )))));
        let result = override_output_field(
            tree,
            "field".into(),
            View::Value,
            |tree| {
                if let View::Fields(fields) = tree {
                    Some(fields)
                } else {
                    None
                }
            },
            |tree| {
                if let View::Wrapper(child) = tree {
                    Some(child)
                } else {
                    None
                }
            },
        );
        let View::Wrapper(outer) = result else {
            panic!("lost outer wrapper")
        };
        let View::Wrapper(inner) = *outer else {
            panic!("lost inner wrapper")
        };
        let View::Fields(fields) = *inner else {
            panic!("lost fields")
        };
        assert!(matches!(fields.get("field"), Some(View::Value)));
    }

    #[test]
    fn constructors_need_no_codec_or_other_operation_capabilities() {
        let flat = model("pub struct Packet {} pub fn make(value:i64)->Packet{}");
        let arguments = [ConstructorInput::Leaf(42)];
        let resolved = resolve_constructor(
            &flat,
            &flat.resolve_type(&syn::parse_quote!(Packet)).unwrap(),
            "make",
            &arguments,
            |ty, tree| {
                assert!(same_type(ty, &syn::parse_quote!(i64)));
                Ok(tree.clone())
            },
            |name| name,
        )
        .unwrap();
        assert!(
            matches!(resolved, ConstructorInput::Branch {operation, children} if operation == "make" && matches!(children.as_slice(),[ConstructorInput::Leaf(42)]))
        );
        assert!(resolve_constructor(
            &flat,
            &syn::parse_quote!(i64),
            "make",
            &arguments,
            |_, tree| Ok(tree.clone()),
            |name| name
        )
        .unwrap_err()
        .contains("expected `i64`"));
        assert!(
            resolve_constructor::<ConstructorInputs, NoOutputs, Infallible, Infallible>(
                &flat,
                &flat.resolve_type(&syn::parse_quote!(Packet)).unwrap(),
                "make",
                &[],
                |_, tree| Ok(tree.clone()),
                |name| name
            )
            .unwrap_err()
            .contains("wrong argument count")
        );
    }

    #[test]
    fn callback_resolution_merges_local_choices_and_preserves_argument_ownership() {
        let flat = model("pub fn register(handler:impl Fn(i64,i64)->i64){}");
        let ty: Type = syn::parse_quote!(impl Fn(i64, i64) -> i64);
        let mut default = crate::DeclCallback::new(ty.clone());
        default.params.insert(0, 10);
        default.ret = Some(30);
        let mut local = crate::DeclCallback::new(ty.clone());
        local.params.insert(1, 20);
        local.ret = Some(40);
        let mut stack = vec![];
        let (operation, arguments, result) = resolve_callback(
            &flat,
            &ty,
            &local,
            Some(&default),
            &BTreeMap::new(),
            &0,
            &0,
            &mut stack,
            |_, value, _| Ok(*value),
            |_, value, ownership| Ok((*value, ownership)),
        )
        .unwrap();
        assert_eq!(
            arguments,
            vec![
                (10, crate::CallbackArgumentOwnership::Transferred),
                (20, crate::CallbackArgumentOwnership::Transferred)
            ]
        );
        assert_eq!(result, 40);
        assert!(operation.explicit_return);
        assert!(stack.is_empty());
        let (_, arguments, _) = resolve_callback(
            &flat,
            &ty,
            &crate::DeclCallback::new(ty.clone()),
            None,
            &BTreeMap::new(),
            &0,
            &0,
            &mut stack,
            |_, value, _| Ok(*value),
            |_, value, ownership| Ok((*value, ownership)),
        )
        .unwrap();
        assert!(arguments
            .iter()
            .all(|(_, ownership)| *ownership == crate::CallbackArgumentOwnership::Scoped));
    }

    #[test]
    fn callback_resolution_cleans_its_recursion_scope_after_error() {
        let flat = model("pub fn register(handler:impl Fn(i64)->i64){}");
        let ty: Type = syn::parse_quote!(impl Fn(i64) -> i64);
        let mut stack = vec![];
        let failure = resolve_callback(
            &flat,
            &ty,
            &crate::DeclCallback::new(ty.clone()),
            None,
            &BTreeMap::new(),
            &0,
            &0,
            &mut stack,
            |_, _, _| Ok(0),
            |_, _, _| Err::<(), _>("rejected argument".into()),
        );
        assert!(failure.unwrap_err().contains("rejected argument"));
        assert!(stack.is_empty());
    }

    #[test]
    fn named_fields_accept_source_names_without_raw_identifier_prefixes() {
        let flat = model("pub struct Row {pub r#type:i64,pub value:i64}");
        let fields = resolve_named_fields(
            &flat,
            &flat.resolve_type(&syn::parse_quote!(Row)).unwrap(),
            &BTreeMap::from([("type".into(), 7)]),
            &0,
            |_, value| Ok(*value),
        )
        .unwrap();
        assert_eq!(fields, vec![7, 0]);
    }
}
