//! Preparation and emission of retained source callbacks.
use super::*;
use prebindgen_conversions::{OutputFragment, TemplateContext};
use prebindgen_declarations::{Declarations, OutputRequest};
use prebindgen_flat::{callback_signature, Wire, WireNode};
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
}
#[derive(Clone, Debug)]
struct Argument {
    source_type: syn::Type,
    value: TokenStream,
    output: JniOutput,
    scoped: bool,
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
/// Preserve direct callback slots while constructors compile their own occurrences.
pub(crate) fn prepare(
    flat: &mut Bindings,
    defaults: &Declarations,
    functions: &[FunctionDecl],
    explicit_outputs: &BTreeSet<String>,
) -> Result<()> {
    for declaration in functions {
        let params = flat.function(&declaration.name)?.params.clone();
        for (index, parameter) in params.into_iter().enumerate() {
            if callback_signature(&parameter.ty.ty)?.is_none() {
                continue;
            }
            if declaration.params.contains_key(&parameter.name.to_string()) {
                return Err("source callback parameters use their argument output expansions, not expand_param".into());
            }
            if parameter.ty.slot.is_some() {
                continue;
            }
            let plan = input(
                flat,
                defaults,
                explicit_outputs,
                &parameter.ty.ty,
                format_ident!("p{index}_callback"),
                format_ident!("p{index}_callback_value"),
                &prebindgen_declarations::Condition::default(),
            )?;
            flat.function_mut(&declaration.name)?.params[index].ty.slot = Some(plan.into());
        }
    }
    Ok(())
}

/// Compile one callback occurrence, including a source constructor's argument.
pub(crate) fn input(
    flat: &Bindings,
    defaults: &Declarations,
    explicit_outputs: &BTreeSet<String>,
    ty: &syn::Type,
    wire: syn::Ident,
    var: syn::Ident,
    active: &prebindgen_declarations::Condition,
) -> Result<InputPlan<JniWire>> {
    let signature = callback_signature(ty)?.ok_or("expected source callback")?;
    let name = if signature.arguments.is_empty() {
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
    validate_name(&name)?;
    let package = signature
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
        .unwrap_or_else(|| flat.metadata.package.clone());
    if foreign_name_exists(flat, &format!("{package}.{name}")) {
        return Err("duplicate foreign callback interface name".into());
    }
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
        let recipe = defaults.output_recipe(flat, &data::DataCodec::default(), &ty, &())?;
        let scoped = !explicit_outputs.contains(&type_key(&ty))
            && !explicit_outputs.contains(&type_key(subject_type(&ty)));
        let output = prebindgen_declarations::compile_callback_argument(
            flat,
            &data::DataCodec::default(),
            &ty,
            &recipe,
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
    });
    callback.input(wire, var, active)
}
impl JniCallback {
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
        descriptor.push_str(")V");
        if slots > 254 {
            return Err("source callback exceeds JVM parameter limit".into());
        }
        let capacity = i32::try_from(values.len() + 16).map_err(|_| "too many callback fields")?;
        let mut invoke = quote!({
            #encode
            let arguments = [#(#values),*];
            #commits
            unsafe { #retained.invoke(#environment, &arguments) }
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
        let (registration, activate) = if active.0.is_empty() {
            (
                quote!(let #retained = ::prebindgen_jni_runtime::Callback::new(&mut #env, &#wire, #descriptor)?;),
                quote!(),
            )
        } else {
            let condition = active.rust();
            (
                quote!(let #retained = if #condition {Some(::prebindgen_jni_runtime::Callback::new(&mut #env, &#wire, #descriptor)?)} else {None};),
                // Optional/selector construction executes only on its validated active path.
                quote!(let #retained = #retained.expect("validated active callback");),
            )
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
            statements: quote!(#activate #[allow(unused_mut)] let #var = move |#(#params),*| {
                #retained.with_env(#capacity, |#environment| { let mut #environment = #environment; #source_bindings #invoke });
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
        let call = format!("{parameter}.run({})", delivered.join(", "));
        let body = if scoped_cleanup.is_empty() {
            format!("{reconstruction}\n        {call}")
        } else {
            format!("var callbackFailure: Throwable? = null\n        var callbackReady = false\n        try {{\n            {reconstruction}\n            callbackReady = true\n            {call}\n        }} catch (failure: Throwable) {{\n            callbackFailure = failure\n            throw failure\n        }} finally {{\n            if (callbackReady) {{\n{scoped_cleanup}                callbackCleanupFailure?.let {{ cleanup -> if (callbackFailure == null) throw cleanup else if (callbackFailure !== cleanup) callbackFailure.addSuppressed(cleanup) }}\n            }}\n        }}")
        };
        Ok(format!("if ({parameter} == null) null else object {{\n    @Suppress(\"UNUSED_PARAMETER\", \"UNUSED_VARIABLE\", \"UNCHECKED_CAST\", \"UNNECESSARY_NOT_NULL_ASSERTION\", \"USELESS_CAST\", \"VARIABLE_WITH_REDUNDANT_INITIALIZER\")\n    fun run({}) {{\n{locals}        {body}\n    }}\n}}", raw.join(", ")))
    }
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
    for function in flat.functions.values().filter(|f| f.metadata.is_some()) {
        for wire in function.input_wire_definitions()? {
            let JniWire::Callback { callback } = &wire.kind else {
                continue;
            };
            let context = KotlinContext {
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
            let text = format!("package {}\n\n/** Invoked by Rust, possibly from a native worker thread. */\npublic fun interface {} {{\n    public fun run({})\n}}\n", callback.package, callback.name, params.join(", "));
            let path = format!(
                "{}/{}.kt",
                callback.package.replace('.', "/"),
                callback.name
            );
            if let Some(existing) = files.get(&path) {
                if existing != &text {
                    return Err("callback interface signature collision".into());
                }
            } else {
                files.insert(path, text);
            }
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
        match ty.strip_prefix("kotlin.").unwrap_or(ty) {
            "Boolean" => Some("false"),
            "Byte" => Some("0.toByte()"),
            "Short" => Some("0.toShort()"),
            "Int" => Some("0"),
            "Long" => Some("0L"),
            "UByte" => Some("0.toUByte()"),
            "UShort" => Some("0.toUShort()"),
            "UInt" => Some("0U"),
            "ULong" => Some("0UL"),
            "Float" => Some("0.0f"),
            "Double" => Some("0.0"),
            "Char" => Some("'\\u0000'"),
            _ => None,
        }
    };
    match default {
        Some(default) => format!("        var {name}: {ty} = {default}\n"),
        None => format!("        lateinit var {name}: {ty}\n"),
    }
}
