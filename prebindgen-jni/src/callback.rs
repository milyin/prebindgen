//! Preparation and emission of retained source callbacks.
use super::*;
use prebindgen_conversions::{InputFragment, OutputFragment, TemplateContext};
use prebindgen_declarations::OutputRequest;
use prebindgen_flat::{callback_signature, InputConversion, Wire, WireNode};
use std::rc::Rc;

/// Prepared callback signature and the independent output boundary of its arguments.
///
/// A callback crosses into Rust as one JVM object. Its Rust-to-Kotlin argument
/// converters are retained here for later invocations, which can outlive the
/// function accepting that callback.
#[derive(Clone, Debug)]
pub struct JniCallback {
    package: String,
    name: String,
    arguments: Vec<Argument>,
    return_value: Option<CallbackReturn>,
}
#[derive(Clone, Debug)]
struct CallbackReturn {
    package: String,
    source_type: syn::Type,
    input: JniInput,
}
#[derive(Clone, Debug)]
struct Argument {
    source_type: syn::Type,
    value: TokenStream,
    output: JniOutput,
    scoped: bool,
}
/// Retain source callback occurrences before filtering to exported functions/helpers.
pub(crate) fn source_parameters(
    items: &[(syn::Item, prebindgen::SourceLocation)],
) -> Vec<syn::Type> {
    items
        .iter()
        .filter_map(|(item, _)| match item {
            // Generic parameters introduce a different type scope; they are not
            // concrete callback occurrences in the supported flat source model.
            syn::Item::Fn(function) if function.sig.generics.params.is_empty() => Some(function),
            _ => None,
        })
        .flat_map(|function| &function.sig.inputs)
        .filter_map(|parameter| match parameter {
            syn::FnArg::Typed(parameter)
                if matches!(parameter.ty.as_ref(), syn::Type::ImplTrait(_)) =>
            {
                Some((*parameter.ty).clone())
            }
            _ => None,
        })
        .collect()
}

/// A declaration configures a source occurrence, rather than inventing a signature.
pub(crate) fn validate_source_occurrence(
    flat: &Bindings,
    declaration: &prebindgen_declarations::CallbackInput,
    sources: &[syn::Type],
) -> Result<()> {
    let expected = callback_signature(&declaration.ty)?
        .ok_or("callback declaration requires impl Fn(...) -> R")?;
    let matches = sources.iter().any(|source| {
        let Ok(Some(actual)) = callback_signature(source) else {
            return false;
        };
        actual.arguments.len() == expected.arguments.len()
            && actual
                .arguments
                .iter()
                .zip(&expected.arguments)
                .all(|(actual, expected)| {
                    flat.resolve_type(actual)
                        .is_ok_and(|actual| prebindgen_flat::same_type(&actual, expected))
                })
            && flat
                .resolve_type(&actual.return_type)
                .is_ok_and(|actual| prebindgen_flat::same_type(&actual, &expected.return_type))
    });
    if !matches {
        let name = declaration.foreign.as_deref().unwrap_or("<unnamed>");
        return Err(format!(
            "callback declaration `{name}` for `{}` does not match any source function parameter; check the callback argument and return types",
            type_key(&declaration.ty)
        ));
    }
    Ok(())
}

pub(crate) fn declared_outputs(
    flat: &Bindings,
    outputs: &[Output<OutputRequest>],
) -> Result<BTreeSet<String>> {
    outputs
        .iter()
        .filter_map(|tree| match tree {
            Output::Request(OutputRequest::ForType { ty, .. }) => {
                Some(flat.resolve_type(ty).map(|ty| type_key(&ty)))
            }
            _ => None,
        })
        .collect()
}
/// Compile one callback occurrence, including a source constructor's argument.
pub(crate) fn input(
    flat: &Bindings,
    configuration: &prebindgen_declarations::CallbackInput,
    explicit_outputs: &BTreeSet<String>,
    ty: &syn::Type,
    wire: syn::Ident,
    var: syn::Ident,
    active: &prebindgen_declarations::Condition,
) -> Result<InputPlan<JniWire>> {
    let signature = callback_signature(ty)?.ok_or("expected source callback")?;
    let returns_unit = matches!(&signature.return_type, syn::Type::Tuple(t) if t.elems.is_empty());
    if returns_unit
        && !matches!(configuration.return_value.as_ref(), Input::Fields(fields) if fields.is_empty())
    {
        return Err("unit callback returns require an empty unit input representation".into());
    }
    let mut name = if signature.arguments.is_empty() {
        "VoidCallback".into()
    } else {
        format!(
            "{}Callback",
            signature
                .arguments
                .iter()
                .map(subject_name)
                .collect::<Vec<_>>()
                .join("")
        )
    };
    if !returns_unit {
        name = format!(
            "{}To{}Callback",
            name.strip_suffix("Callback").unwrap(),
            subject_name(&signature.return_type)
        );
    }
    if let Some(selected) = &configuration.foreign {
        name = selected.clone();
    }
    validate_class_name(&name)?;
    let package = configuration.namespace.clone().unwrap_or_else(|| {
        signature
            .arguments
            .first()
            .and_then(|ty| {
                flat.type_info(subject_type(ty))
                    .ok()?
                    .metadata
                    .class
                    .as_ref()
                    .map(|c| c.pkg.clone())
            })
            .unwrap_or_else(|| flat.metadata.package.clone())
    });
    validate_package(&package)?;
    if configuration.arguments.len() != signature.arguments.len() {
        return Err("prepared callback has the wrong argument count".into());
    }
    if foreign_name_exists(flat, &format!("{package}.{name}")) {
        return Err("duplicate foreign callback interface name".into());
    }
    let return_value = if returns_unit {
        None
    } else {
        if has_reference(&signature.return_type) {
            return Err(
                "callback return values must be owned; borrowed returns are unsupported".into(),
            );
        }
        let codec = data::DataCodec::default();
        let input = prebindgen_declarations::compile_callback_return(
            flat,
            &codec,
            &signature.return_type,
            &configuration.return_value,
            &(),
        )?;
        Some(CallbackReturn {
            package: flat.metadata.package.clone(),
            source_type: signature.return_type,
            input,
        })
    };
    let mut arguments = vec![];
    for (i, source_type) in signature.arguments.into_iter().enumerate() {
        let binding = format_ident!("callback_source_{i}");
        let (ty, borrowed) = match &source_type {
            syn::Type::Reference(r) if r.mutability.is_none() => match &*r.elem {
                syn::Type::Slice(_) => (source_type.clone(), false),
                syn::Type::Path(p) if p.path.is_ident("str") => (source_type.clone(), false),
                ty => (ty.clone(), true),
            },
            syn::Type::Reference(_) => {
                return Err("mutable callback arguments are unsupported".into())
            }
            _ => (source_type.clone(), false),
        };
        let value = quote!(#binding);
        if contains_result(&ty) {
            return Err(
                "Result-valued callback arguments require an explicit value representation".into(),
            );
        }
        let recipe = &configuration.arguments[i];
        let scoped = !configuration.explicit_arguments.contains(&i)
            && !explicit_outputs.contains(&type_key(&ty))
            && !explicit_outputs.contains(&type_key(subject_type(&ty)));
        let output = prebindgen_declarations::compile_callback_argument(
            flat,
            &data::DataCodec::default(),
            &ty,
            recipe,
            i,
            borrowed,
            &(),
        )?;
        sequence::validate_buffered(&output.output_outcomes().success)?;
        arguments.push(Argument {
            source_type,
            value,
            output,
            scoped,
        });
    }
    let callback = Rc::new(JniCallback {
        package,
        name,
        arguments,
        return_value,
    });
    callback.input(wire, var, active)
}
impl JniCallback {
    pub(crate) fn bulk_nodes(&self) -> Vec<(bool, &kotlin::JniNode)> {
        let mut nodes = self
            .arguments
            .iter()
            .map(|a| (false, &a.output.output_outcomes().success))
            .collect::<Vec<_>>();
        if let Some(value) = &self.return_value {
            nodes.push((true, value.input.input_node()));
        }
        nodes
    }

    pub(crate) fn fqn(&self) -> String {
        format!("{}.{}", self.package, self.name)
    }

    fn input(
        self: &Rc<Self>,
        wire: syn::Ident,
        var: syn::Ident,
        active: &prebindgen_declarations::Condition,
    ) -> Result<InputPlan<JniWire>> {
        prebindgen_conversions::validate_wire_names(self.arguments.iter().flat_map(|arg| {
            prebindgen_flat::wire_definitions(std::iter::once(
                &arg.output.output_outcomes().success,
            ))
        }))?;
        let env = prebindgen_conversions::template_parameter("environment");
        let retained = format_ident!("{wire}_retained");
        let environment = format_ident!("callback_environment");
        let context = JniContext {
            environment: environment.clone(),
            acknowledgement: format_ident!("unused_ack"),
            domain_output: format_ident!("unused_domain"),
            return_sink: None,
            is_domain_error: false,
            output_committed: quote!(),
        };
        let mut descriptor = String::from("(");
        let mut encode = TokenStream::new();
        let mut commits = TokenStream::new();
        let mut values = vec![];
        let mut source_bindings = TokenStream::new();
        let mut prepares = vec![];
        let mut params = vec![];
        let mut slots = 0;
        for (index, argument) in self.arguments.iter().enumerate() {
            let source = format_ident!("callback_value_{index}");
            let param = format_ident!("callback_source_{index}");
            let ty = &argument.source_type;
            params.push(quote!(#param: #ty));
            let value = &argument.value;
            source_bindings.extend(quote!(let #source = #value;));
            let fields = group::callback_parameters(&argument.output.output_outcomes().success);
            let assignments =
                prebindgen_conversions::match_output_fields(fields, &argument.output.rust.values)?;
            for (position, (field, value)) in assignments.into_iter().enumerate() {
                descriptor.push_str(field.kind.jvm_descriptor());
                slots += field.kind.parameter_slots();
                let local = format_ident!("callback_java_{index}_{position}");
                let code = field.kind.output_argument(value, &local);
                encode.extend(render(&context, &code.statements, &source)?);
                values.push(render(&context, &code.value, &source)?);
            }
            commits.extend(argument.output.code_fragment_slot(
                OutputFragment::Commit {
                    context: &context,
                    source: &source,
                },
                &syn::parse_quote!(()),
            )?);
            prepares.push((argument, source));
        }
        descriptor.push(')');
        descriptor.push_str(
            &self
                .return_value
                .as_ref()
                .map(|r| r.descriptor())
                .unwrap_or_else(|| "V".into()),
        );
        if slots > 254 {
            return Err("source callback exceeds JVM parameter limit".into());
        }
        let return_references =
            self.return_value
                .as_ref()
                .map_or(0, |result| match result.fields().as_slice() {
                    [] => 0,
                    [field] => usize::from(field.kind.is_reference()),
                    fields => fields.len() + 1,
                });
        let capacity = i32::try_from(values.len() + return_references + 16)
            .map_err(|_| "too many callback fields")?;
        let call = if let Some(result) = &self.return_value {
            result.rust_invoke(&context, &retained, &environment)?
        } else {
            quote!(unsafe { #retained.invoke(#environment, &arguments) })
        };
        let mut invoke = quote!({
            #encode
            let arguments = [#(#values),*];
            #commits
            #call
        });
        for (argument, source) in prepares.into_iter().rev() {
            invoke = argument.output.code_fragment_slot(
                OutputFragment::Prepare {
                    context: &context,
                    source: &source,
                    next: invoke,
                },
                &syn::parse_quote!(::prebindgen_runtime::Result<()>),
            )?;
        }
        let callback_name = self.fqn();
        let (registration, activate) = if active.0.is_empty() {
            (
                quote!(let #retained = ::prebindgen_jni_runtime::Callback::new_named(&mut #env, &#wire, #descriptor, #callback_name)?;),
                quote!(),
            )
        } else {
            let condition = active.rust();
            (
                quote!(let #retained = if #condition {Some(::prebindgen_jni_runtime::Callback::new_named(&mut #env, &#wire, #descriptor, #callback_name)?)} else {None};),
                // Optional/selector construction executes only on its validated active path.
                quote!(let #retained = #retained.expect("validated active callback");),
            )
        };
        let invocation = if self.return_value.is_some() {
            quote!(#retained.with_env_result(#capacity, |#environment| { let mut #environment = #environment; #source_bindings #invoke }).unwrap_or_else(|error| panic!("callback failed: {}", error)))
        } else {
            quote!(#retained.with_env(#capacity, |#environment| { let mut #environment = #environment; #source_bindings #invoke });)
        };
        Ok(InputPlan {
            fallible: true,
            context: Default::default(),
            node: WireNode::Leaf(vec![Wire {
                name: wire.clone(),
                kind: JniWire::Callback {
                    callback: self.clone(),
                },
            }]),
            acknowledgement: None,
            acquire: quote!(),
            resources: vec![],
            validation: registration,
            statements: quote!(#activate # [allow(unused_mut, clippy::collapsible_if)] let #var = move |#(#params),*| {
                #invocation
            };),
            argument: quote!(#var),
        })
    }

    pub(crate) fn adapter(&self, parameter: &str, context: &KotlinContext<'_>) -> Result<String> {
        let mut raw = vec![];
        let mut outputs = vec![];
        let mut offset = 0;
        for (index, argument) in self.arguments.iter().enumerate() {
            let prefix = format!("callbackOutput{index}_");
            let ctx = KotlinContext {
                result: "parts",
                output_prefix: &prefix,
                cleanup_failure: Some("callbackCleanupFailure"),
                ..*context
            };
            let mut output = argument.output.kotlin(&ctx)?;
            let fields = group::callback_parameters(&argument.output.output_outcomes().success)
                .collect::<Vec<_>>();
            for (i, field) in fields.iter().enumerate() {
                let name = format!("raw{}", offset + i);
                raw.push(format!("{name}: {}", field.kind.kotlin_type()));
            }
            for text in output.delivered.iter_mut().chain([
                &mut output.output_locals,
                &mut output.materialize,
                &mut output.cleanup,
            ]) {
                for i in (0..fields.len()).rev() {
                    *text = text.replace(&format!("parts[{i}]"), &format!("raw{}", offset + i));
                }
            }
            offset += fields.len();
            outputs.push((argument.scoped, output));
        }
        let mut locals = String::from("        var callbackCleanupFailure: Throwable? = null\n");
        let mut materialize = String::new();
        let mut cleanup = String::new();
        let mut scoped_cleanup = String::new();
        let mut delivered = vec![];
        for (argument, (scoped, output)) in outputs.iter().enumerate() {
            locals.push_str(&output.output_locals);
            materialize.push_str(&output.materialize);
            cleanup.push_str(&output.cleanup);
            if *scoped {
                scoped_cleanup.push_str(&output.cleanup);
            }
            for (value, (ty, expression)) in output
                .result_types
                .iter()
                .zip(&output.delivered)
                .enumerate()
            {
                let name = format!("callbackArgument{argument}_{value}");
                locals.push_str(&argument_local(&name, ty));
                materialize.push_str(&format!("            {name} = {expression}\n"));
                delivered.push(name);
            }
        }
        let reconstruction = format!("try {{\n{materialize}        }} catch (failure: Throwable) {{\n{cleanup}            callbackCleanupFailure?.let {{ if (failure !== it) failure.addSuppressed(it) }}\n            throw failure\n        }}");
        let mut call = format!("{parameter}.run({})", delivered.join(", "));
        let (return_type, finish_return) = if let Some(result) = &self.return_value {
            locals.push_str(&argument_local(
                "callbackReturned",
                &result.kotlin_type(context)?,
            ));
            call = format!("callbackReturned = {call}");
            if !scoped_cleanup.is_empty() && !result.input.kotlin(context)?.consumed.is_empty() {
                locals.push_str("var callbackReturnReady = false\n");
                call.push_str("; callbackReturnReady = true");
            }
            (
                format!(": {}", result.raw_kotlin_type()),
                result.kotlin_encode("callbackReturned", context)?,
            )
        } else {
            (String::new(), String::new())
        };
        let body = if scoped_cleanup.is_empty() {
            format!("{reconstruction}\n        {call}")
        } else {
            format!("var callbackFailure: Throwable? = null\n        var callbackReady = false\n        try {{\n            {reconstruction}\n            callbackReady = true\n            {call}\n        }} catch (failure: Throwable) {{\n            callbackFailure = failure\n            throw failure\n        }} finally {{\n            if (callbackReady) {{\n{scoped_cleanup}                callbackCleanupFailure?.let {{ cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }}\n            }}\n        }}")
        };
        let body = if let Some(result) = &self.return_value {
            if !scoped_cleanup.is_empty() {
                if let Some(cleanup) = result.kotlin_cleanup("callbackReturned", context)? {
                    format!("try {{ {body} }} catch (failure: Throwable) {{ if (callbackReturnReady) {{ {cleanup} }}; throw failure }}")
                } else {
                    body
                }
            } else {
                body
            }
        } else {
            body
        };
        let body = format!("{body}\n{finish_return}");
        Ok(format!("if ({parameter} == null) null else object {{\n    @Suppress(\"UNUSED_PARAMETER\", \"UNUSED_VARIABLE\", \"UNCHECKED_CAST\", \"UNNECESSARY_NOT_NULL_ASSERTION\", \"USELESS_CAST\", \"VARIABLE_WITH_REDUNDANT_INITIALIZER\")\n    fun run({}){return_type} {{\n{locals}        {body}\n    }}\n}}", raw.join(", ")))
    }
}
fn has_reference(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(_))
        || prebindgen_flat::option_inner(ty).is_some_and(has_reference)
        || prebindgen_flat::sequence_parts(ty).is_some_and(|(t, _)| has_reference(t))
}
impl CallbackReturn {
    fn fields(&self) -> Vec<&Wire<JniWire>> {
        group::argument_parameters(self.input.input_node()).collect()
    }
    fn descriptor(&self) -> String {
        match self.fields().as_slice() {
            [field] => field.kind.jvm_descriptor().into(),
            _ => "[Ljava/lang/Object;".into(),
        }
    }
    fn raw_kotlin_type(&self) -> String {
        match self.fields().as_slice() {
            [field] => field.kind.kotlin_type().into(),
            _ => "Array<Any?>".into(),
        }
    }
    fn kotlin_type(&self, context: &KotlinContext<'_>) -> Result<String> {
        let input = self.input.kotlin(context)?;
        if input.parameters.len() != 1 {
            return Err("callback return conversion must expose exactly one Kotlin value".into());
        }
        Ok(input.parameters[0]
            .split_once(": ")
            .ok_or("invalid callback return parameter")?
            .1
            .into())
    }
    fn kotlin_encode(&self, call: &str, context: &KotlinContext<'_>) -> Result<String> {
        self.kotlin_type(context)?;
        let input = self.input.kotlin(context)?;
        let name = input.parameters[0].split_once(": ").unwrap().0;
        let binding = format!("val {name} = {call}\n");
        let mut setup = String::new();
        for binding in input.setup {
            setup.push_str(&format!("val {} = {}\n", binding.name, binding.expression));
        }
        let raw = match input.arguments.as_slice() {
            [value] => value.clone(),
            values => format!("kotlin.arrayOf<kotlin.Any?>({})", values.join(", ")),
        };
        let mut body = format!(
            "val callbackReturnWire = {raw}\n{}\ncallbackReturnWire",
            input.consumed.join("; ")
        );
        if !input.locks.is_empty() || !input.lock_groups.is_empty() {
            let prefix = &self.package;
            if input.lock_groups.is_empty() && input.locks.len() <= 3 {
                // Kotlin evaluates all handle arguments before acquiring monitors.
                // Reuse the ordinary-call overloads without allocating a lock list.
                body = format!(
                    "{prefix}.withHandleLocks({}) {{ {body} }}",
                    input.locks.join(", ")
                );
            } else {
                let mut handles = if input.locks.is_empty() {
                    "kotlin.collections.emptyList<Nothing>()".into()
                } else {
                    format!(
                        "kotlin.collections.listOfNotNull({})",
                        input.locks.join(", ")
                    )
                };
                for group in input.lock_groups {
                    handles = format!("({handles} + ({group}))");
                }
                body = format!("{prefix}.withHandleLocks({handles}) {{ {body} }}");
            }
        }
        body = format!("{setup} {body}");
        if let Some(cleanup) = self.kotlin_cleanup(name, context)? {
            body = format!(
                "try {{ {body} }} catch (failure: Throwable) {{ {cleanup}; throw failure }}"
            );
        }
        Ok(format!("return kotlin.run {{ {binding} {body} }}"))
    }
    fn kotlin_cleanup(&self, value: &str, context: &KotlinContext<'_>) -> Result<Option<String>> {
        if self.input.kotlin(context)?.consumed.is_empty() {
            return Ok(None);
        }
        let prefix = &self.package;
        Ok(Some(format!("fun releaseReturned(value: Any?) {{ when (value) {{ is {prefix}.NativeHandle -> try {{ value.close() }} catch (cleanup: Throwable) {{ if (failure !== cleanup) failure.addSuppressed(cleanup) }}; is Iterable<*> -> value.forEach {{ releaseReturned(it) }} }} }}; releaseReturned({value})")))
    }
    fn ownership_guard(&self, context: &JniContext) -> Result<(TokenStream, TokenStream)> {
        let fields = self.fields();
        if let [field] = fields.as_slice() {
            if let JniWire::Handle { handle, .. } = &field.kind {
                if handle.access == Access::Move {
                    let name = &field.name;
                    let ty = &handle.ty;
                    return Ok((
                        quote!(let callback_return_owner = if #name == 0 {None} else {
                        let pointer=#name as *mut ::prebindgen_jni_runtime::Allocation<#ty>;
                        ::prebindgen_runtime::validate_pointer(pointer)?;
                        Some(unsafe {Box::from_raw(pointer)})
                    };),
                        quote!(let _=callback_return_owner.map(Box::into_raw);),
                    ));
                }
            }
        }
        let scan = receive_return_owners(self.input.input_node(), context)?;
        if scan.is_empty() {
            return Ok((quote!(), quote!()));
        }
        Ok((
            quote!(let mut callback_return_owners=::prebindgen_runtime::CallbackReturnAllocations::default(); #scan callback_return_owners.check()?;),
            quote!(callback_return_owners.commit();),
        ))
    }
    fn rust_invoke(
        &self,
        context: &JniContext,
        retained: &syn::Ident,
        env: &syn::Ident,
    ) -> Result<TokenStream> {
        let mut input = self.input.clone();
        let cache = crate::bulk::callback_cache(input.input_node(), context)?;
        input.rust.validation = crate::bulk::cached_reads(&input.rust.validation)?;
        let fields = self.fields();
        let result = format_ident!("callback_result");
        let (return_type, decode) = match fields.as_slice() {
            [field] => {
                let name = &field.name;
                let read = field.kind.read_callback_return(&result);
                (
                    field.kind.callback_return_type(),
                    quote!(let #name = #read;),
                )
            }
            _ => {
                let reads = fields.iter().enumerate().map(|(i, field)| {
                    let name = &field.name;
                    let read = field
                        .kind
                        .read_array_element(&quote!(callback_result_rows), i);
                    quote!(let #name = #read?;)
                });
                (
                    quote!(::prebindgen_jni_runtime::jni::signature::ReturnType::Object),
                    quote!(
                        let callback_result_rows = ::prebindgen_jni_runtime::jni::objects::JObjectArray::from(#result.l().map_err(|e|e.to_string())?);
                        #(#reads)*
                    ),
                )
            }
        };
        let target = format_ident!("callback_return_value");
        let ty = &self.source_type;
        let expected = syn::parse_quote!(::prebindgen_runtime::Result<#ty>);
        let (owners, commit_owners) = self.ownership_guard(context)?;
        let mut body = input.code_fragment_slot(
            InputFragment::Construct {
                context,
                target: &target,
                next: quote!(Ok(#target)),
            },
            &expected,
        )?;
        body = input.code_fragment_slot(
            InputFragment::Acquire {
                context,
                next: body,
            },
            &expected,
        )?;
        body = quote!({ #commit_owners #body });
        let checks = input.code_fragment_slot(
            InputFragment::CheckInternalAliases { context },
            &syn::parse_quote!(()),
        )?;
        if checks.to_string() != quote!(()).to_string() {
            body = quote!({let _: () = #checks; #body});
        }
        body = input.code_fragment_slot(
            InputFragment::Validate {
                context,
                next: body,
            },
            &expected,
        )?;
        context.render(&quote!({
            let #result = unsafe { #retained.invoke_return(#env, #return_type, &arguments) }?;
            #decode
            #cache
            #owners
            #body
        }))
    }
}
pub(crate) fn receive_return_owners(
    node: &kotlin::JniNode,
    context: &JniContext,
) -> Result<TokenStream> {
    Ok(match node {
        WireNode::Chain {meta:JniChain::Bulk(layout),controls,child} => crate::bulk::scan_bulk(layout,controls,child,context)?,
        WireNode::Chain {meta:JniChain::Sequence(JniSequence::Bulk),controls,child} => crate::bulk::scan_rows(controls,child,context)?,
        WireNode::Leaf(fields)=>fields.iter().filter_map(|field| {
            if let JniWire::Handle {handle,..}=&field.kind {
                if handle.access==Access::Move {let name=&field.name;let ty=&handle.ty;return Some(quote!(unsafe {callback_return_owners.receive(#name as *mut ::prebindgen_jni_runtime::Allocation<#ty>);}));}
            }
            None
        }).collect(),
        WireNode::Chain { meta:JniChain::Optional(optional),child,..}=> {
            let scan=receive_return_owners(child,context)?;
            if scan.is_empty(){return Ok(scan);}
            let name=optional.discriminator();
            let present=if let Some(niche)=optional.niche(){let field=group::argument_parameters(child).find(|w|w.name==*name).ok_or("missing callback return optional discriminator")?; niche.present(&field.kind,quote!(#name))} else {quote!(#name==1)};
            quote!(if #present {#scan})
        }
        WireNode::Chain {meta:JniChain::Sequence(_),controls,child}=> {
            let scan=receive_return_owners(child,context)?;
            if scan.is_empty(){return Ok(scan);}
            let carrier=&controls[0].name;let row=format_ident!("{carrier}_ownership_row");
            let fields=group::argument_parameters(child).collect::<Vec<_>>();let count=fields.len();let capacity=i32::try_from(count+8).map_err(|_|"too many returned fields")?;
            let reads=fields.into_iter().enumerate().map(|(index,field)|{let name=&field.name;let read=field.kind.read_array_element(&quote!(#row),index);quote!(let #name=#read?;)});
            let env=&context.environment;
            let code=quote!(if !#carrier.is_null() {let length=#env.get_array_length(&#carrier).map_err(|e|e.to_string())? as usize;for index in 0..length {
                let frame=unsafe {::prebindgen_jni_runtime::LocalFrame::new(&#env,#capacity)}?;
                let #row=::prebindgen_jni_runtime::row(&mut #env,&#carrier,index,#count)?;
                #(#reads)* #scan drop(frame);
            }});
            context.render(&code)?
        }
        WireNode::Branch {meta:JniBranch::Alternatives,controls,children}=> {
            let selector=&controls[0].name;let scans=children.iter().enumerate().map(|(i,child)|{let scan=receive_return_owners(child,context)?;let i=i as i32;Ok(quote!(if #selector==#i {#scan}))}).collect::<Result<Vec<_>>>()?;quote!(#(#scans)*)
        }
        WireNode::Branch {children,..}=>children.iter().map(|child|receive_return_owners(child,context)).collect::<Result<Vec<_>>>()?.into_iter().collect(),
    })
}
fn render(context: &JniContext, code: &TokenStream, source: &syn::Ident) -> Result<TokenStream> {
    let code = prebindgen_conversions::render_template(code, &|name| {
        Ok(if name == "source_result" {
            quote!(#source)
        } else {
            prebindgen_conversions::template_parameter(name)
        })
    })?;
    context.render(&code)
}
pub(crate) fn files(
    flat: &Bindings,
    classes: &BTreeMap<String, KotlinClass>,
    files: &mut BTreeMap<String, String>,
) -> Result<()> {
    let mut ownership_by_name = BTreeMap::new();
    let mut wires = Vec::new();
    for input in &flat.metadata.callbacks {
        wires.extend(prebindgen_flat::wire_definitions(std::iter::once(
            input.input_node(),
        )));
    }
    for function in flat.functions.values().filter(|f| f.metadata.is_some()) {
        wires.extend(function.input_wire_definitions()?);
    }
    for wire in wires {
        let JniWire::Callback { callback } = &wire.kind else {
            continue;
        };
        let context = KotlinContext {
            package: &flat.metadata.package,
            classes,
            result: "parts",
            failure: "failure",
            cleanup_failure: None,
            output_prefix: "output",
        };
        let mut params = vec![];
        let mut names = BTreeSet::new();
        for (i, arg) in callback.arguments.iter().enumerate() {
            let output = arg.output.kotlin(&context)?;
            output.callback_parameter_declarations()?;
            for (j, ty) in output.result_types.iter().enumerate() {
                let mut name = output
                    .result_names
                    .get(j)
                    .and_then(|n| n.clone())
                    .unwrap_or_else(|| format!("arg{i}_{j}"));
                while !names.insert(name.clone()) {
                    name.push('_');
                }
                params.push(format!("`{name}`: {ty}"));
            }
        }
        let return_type = callback
            .return_value
            .as_ref()
            .map(|r| r.kotlin_type(&context).map(|ty| format!(": {ty}")))
            .transpose()?
            .unwrap_or_default();
        let contract = if let Some(result) = &callback.return_value {
            let ownership = if result.input.kotlin(&context)?.consumed.is_empty() {
                ""
            } else {
                " Returned handles transfer ownership to Rust."
            };
            format!(" Failure to invoke or convert the returned value raises a Rust panic on the invoking thread; an uncaught worker panic terminates that worker.{ownership}")
        } else {
            String::new()
        };
        let ownership = callback
            .arguments
            .iter()
            .map(|arg| {
                let handles = prebindgen_flat::wire_definitions(std::iter::once(
                    &arg.output.output_outcomes().success,
                ))
                .any(|wire| matches!(wire.kind, JniWire::Handle { .. }));
                handles && !arg.scoped
            })
            .collect::<Vec<_>>();
        if let Some(previous) = ownership_by_name.insert(callback.fqn(), ownership.clone()) {
            if previous != ownership {
                return Err("callback interface ownership collision; use callback!(...).name(...) for a distinct interface".into());
            }
        }
        let text = format!("package {}\n\n/** Invoked by Rust, possibly from a native worker thread.{contract} */\npublic fun interface {} {{\n    public fun run({}){return_type}\n}}\n", callback.package, callback.name, params.join(", "));
        let path = format!(
            "{}/{}.kt",
            callback.package.replace('.', "/"),
            callback.name
        );
        if let Some(existing) = files.get(&path) {
            if existing != &text {
                return Err("callback interface signature collision; use callback!(...).name(...) for a distinct interface".into());
            }
        } else {
            files.insert(path, text);
        }
    }
    Ok(())
}

fn subject_type(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Reference(r) => subject_type(&r.elem),
        _ => prebindgen_flat::option_inner(ty)
            .map(subject_type)
            .unwrap_or(ty),
    }
}
fn subject_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Reference(r) => subject_name(&r.elem),
        syn::Type::Slice(s) => format!("{}List", subject_name(&s.elem)),
        _ => {
            if let Some(inner) = prebindgen_flat::option_inner(ty) {
                return format!("Optional{}", subject_name(inner));
            }
            if let Some(inner) = prebindgen_flat::vec_inner(ty) {
                return format!("{}Vec", subject_name(inner));
            }
            match ty {
                syn::Type::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
                _ => "Unit".into(),
            }
        }
    }
}

fn contains_result(ty: &syn::Type) -> bool {
    prebindgen_flat::result_parts(ty).is_some()
        || prebindgen_flat::option_inner(ty).is_some_and(contains_result)
        || prebindgen_flat::sequence_parts(ty).is_some_and(|(ty, _)| contains_result(ty))
        || matches!(ty,syn::Type::Reference(r) if contains_result(&r.elem))
}
fn argument_local(name: &str, ty: &str) -> String {
    let default = if ty.ends_with('?') {
        Some("null")
    } else {
        let kotlin_type = ty.strip_prefix("kotlin.").unwrap_or(ty);
        crate::wire::jvm::primitive_local_default(kotlin_type).or(match kotlin_type {
            "UByte" => Some("0.toUByte()"),
            "UShort" => Some("0.toUShort()"),
            "UInt" => Some("0U"),
            "ULong" => Some("0UL"),
            "Char" => Some("'\\u0000'"),
            _ => None,
        })
    };
    match default {
        Some(default) => format!("        var {name}: {ty} = {default}\n"),
        None => format!("        lateinit var {name}: {ty}\n"),
    }
}
