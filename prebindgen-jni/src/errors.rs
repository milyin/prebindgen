//! JNI error channels reuse ordinary output conversions. User callbacks run in Kotlin.
use super::*;
use prebindgen_conversions::OutputFragment;
use prebindgen_flat::Conversion;
use prebindgen_flat::OutputConversion;

#[derive(Clone, Debug)]
pub(crate) struct ErrorBinding {
    pub(crate) helper: syn::Ident,
    ty: syn::Type,
    pub output: JniOutput,
    pub handler: String,
}
impl ErrorBinding {
    pub fn throws(&self, exception: Option<&str>) -> String {
        exception.map_or_else(String::new, |class| {
            format!("@kotlin.jvm.Throws({}::class)\n", class.replace('$', "."))
        })
    }
    pub fn parameter(&self, ret: &str, exception: Option<&str>) -> String {
        let default = exception.map_or_else(String::new, |class| {
            format!(
                " = {} {{ message -> throw {}(message) }}",
                self.handler,
                class.replace('$', ".")
            )
        });
        format!("onError: {}<{ret}>{default}", self.handler)
    }
}

fn helper_name(names: &mut names::Names, ty: &syn::Type, mapped: bool) -> Result<syn::Ident> {
    let mode = if mapped { "mapped" } else { "typed" };
    Ok(format_ident!(
        "{}",
        names.name(
            "__pb_jni_error",
            &format!("{}_{mode}", names::type_label(ty)),
            &[&names::type_identity(ty), mode]
        )?
    ))
}

/// Prepare each error once on its type and attach each function's selected binding.
pub(crate) fn prepare(
    flat: &mut Bindings,
    codec: &data::DataCodec<'_>,
    declarations: &prebindgen_declarations::Declarations,
    functions: &[FunctionDecl],
) -> Result<()> {
    let mut names = names::Names::default();
    for ty in flat.types.values() {
        for (mapped, binding) in &ty.metadata.errors {
            names.reserve(
                &binding.helper.to_string(),
                "__pb_jni_error",
                &[
                    &names::type_identity(&ty.ty),
                    if *mapped { "mapped" } else { "typed" },
                ],
            )?;
        }
    }
    for function in functions {
        let ret = flat.function(&function.name)?.ret.ty.clone();
        let Some((_, error)) = prebindgen_flat::result_parts(&ret) else {
            continue;
        };
        flat.type_metadata_mut(error)?;
        let canonical_error = flat.type_info(error)?.ty.clone();
        let mapped = declarations.error_mapper_for(function, error).is_some();
        let exception = flat.type_info(error)?.metadata.error_class.clone();
        if exception.is_some() && !mapped {
            return Err(
                "error_class requires an explicit message mapper for its default Kotlin handler"
                    .into(),
            );
        }
        if let Some(binding) = flat.type_info(error)?.metadata.errors.get(&mapped).cloned() {
            flat.function_mut(&function.name)?
                .metadata
                .as_mut()
                .ok_or_else(|| format!("JNI function `{}` has no export metadata", function.name))?
                .error = Some(binding);
            continue;
        }
        let (ty, recipe) = if mapped {
            (
                syn::parse_quote!(String),
                prebindgen_declarations::Output::Leaf,
            )
        } else {
            (
                canonical_error.clone(),
                declarations.output_recipe(flat, codec, &canonical_error, &())?,
            )
        };
        let helper = helper_name(&mut names, &canonical_error, mapped)?;
        let output = prebindgen_declarations::compile_output_value(flat, codec, &ty, &recipe, &())
            .map_err(|e| {
                format!(
                    "JNI error `{}` in `{}`: {e}",
                    type_key(error),
                    function.name
                )
            })?;
        let pkg = flat
            .type_info(error)?
            .metadata
            .class
            .as_ref()
            .map_or(flat.metadata.package.as_str(), |c| c.pkg.as_str());
        let suffix = if mapped { "MessageHandler" } else { "Handler" };
        let short = if let syn::Type::Path(path) = &canonical_error {
            if path.qself.is_none()
                && path
                    .path
                    .segments
                    .iter()
                    .all(|segment| matches!(segment.arguments, syn::PathArguments::None))
            {
                Some(format!(
                    "{}{suffix}",
                    path.path.segments.last().unwrap().ident
                ))
            } else {
                None
            }
        } else {
            None
        };
        let short = if let Some(short) = short {
            short
        } else {
            let mode = if mapped { "mapped" } else { "typed" };
            names.name(
                "ErrorHandler",
                &format!("{}_{mode}", names::type_label(&canonical_error)),
                &[&names::type_identity(&canonical_error), mode],
            )?
        };
        let handler = format!("{pkg}.{short}");
        if foreign_name_exists(flat, &handler) {
            return Err(format!("duplicate foreign name `{handler}`"));
        }
        let binding = std::rc::Rc::new(ErrorBinding {
            helper,
            ty,
            output,
            handler,
        });
        flat.type_metadata_mut(error)?
            .errors
            .insert(mapped, binding.clone());
        flat.function_mut(&function.name)?
            .metadata
            .as_mut()
            .ok_or_else(|| format!("JNI function `{}` has no export metadata", function.name))?
            .error = Some(binding);
    }
    Ok(())
}

pub(crate) fn emit(errors: &BTreeMap<String, &ErrorBinding>) -> Result<TokenStream> {
    let mut code = TokenStream::new();
    let mut emitted = BTreeSet::new();
    for error in errors.values() {
        if !emitted.insert(error.helper.to_string()) {
            continue;
        }
        let helper = &error.helper;
        let ty = &error.ty;
        let source = format_ident!("source");
        let context = JniContext {
            environment: format_ident!("env"),
            acknowledgement: format_ident!("unused_ack"),
            domain_output: format_ident!("reported"),
            return_sink: None,
            is_domain_error: true,
            output_committed: quote!(*reported = Some(result.as_raw());),
        };
        let body_for = |context: &JniContext| -> Result<TokenStream> {
            let commit = error.output.code_fragment_slot(
                OutputFragment::Commit {
                    context,
                    source: &source,
                },
                &syn::parse_quote!(()),
            )?;
            let body = error.output.code_fragment_slot(
                OutputFragment::Deliver {
                    context,
                    source: &source,
                    next: quote!({let _: () = #commit; Ok(())}),
                },
                &syn::parse_quote!(::prebindgen_runtime::Result<()>),
            )?;
            error.output.code_fragment_slot(
                OutputFragment::Prepare {
                    context,
                    source: &source,
                    next: body,
                },
                &syn::parse_quote!(::prebindgen_runtime::Result<()>),
            )
        };
        let body = body_for(&context)?;
        let callback_context = JniContext {
            return_sink: Some(format_ident!("__pb_return")),
            output_committed: quote!(*reported = Some(std::ptr::null_mut());),
            ..context
        };
        let callback_body = if error.output.native_return().is_some() {
            body_for(&callback_context)?
        } else {
            quote!(Err("custom error output requires packet delivery".into()))
        };
        code.extend(quote! {
            #[allow(non_snake_case, clippy::needless_borrow, clippy::let_unit_value, clippy::unnecessary_cast, clippy::useless_conversion)]
            fn #helper(mut env: &mut ::prebindgen_jni_runtime::jni::JNIEnv<'_>, source: #ty, reported: &mut Option<::prebindgen_jni_runtime::jni::sys::jobjectArray>, return_sink: Option<&::prebindgen_jni_runtime::jni::objects::JObject<'_>>) -> ::prebindgen_runtime::Result<()> {
                if let Some(__pb_return) = return_sink { #callback_body } else { #body }
            }
        });
    }
    Ok(code)
}

pub(crate) fn interfaces(prefix: &str, files: &mut BTreeMap<String, String>) -> Result<()> {
    let binding_name = format!("{}/JniErrorHandler.kt", prefix.replace('.', "/"));
    if files.insert(binding_name, format!("package {prefix}\n\n/** Binding failure callback, invoked after the native call and handle locks finish. */\npublic fun interface JniErrorHandler<out R> {{ public fun run(message: String?): R }}\n")).is_some() {
        return Err("Kotlin error interface collision".into());
    }
    Ok(())
}

pub(crate) fn handler_files(
    errors: &BTreeMap<String, &ErrorBinding>,
    context: &KotlinContext<'_>,
    files: &mut BTreeMap<String, String>,
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for error in errors.values() {
        if !seen.insert(error.handler.clone()) {
            continue;
        }
        let (pkg, name) = error.handler.rsplit_once('.').unwrap();
        let output = error.output.kotlin(context)?;
        let slots: usize = output
            .result_types
            .iter()
            .map(|ty| {
                if matches!(ty.as_str(), "Long" | "Double" | "ULong") {
                    2
                } else {
                    1
                }
            })
            .sum();
        if slots > 254 {
            return Err(format!(
                "domain handler `{}` exceeds the JVM parameter limit",
                error.handler
            ));
        }
        let params = output.callback_parameter_declarations()?;
        let text = format!("package {pkg}\n\n/** Domain-error callback. Called after JNI and handle locks finish. Handle arguments are closed after run; call take() to retain an owner. */\npublic fun interface {name}<out R> {{ public fun run({params}): R }}\n");
        if files
            .insert(format!("{}/{}.kt", pkg.replace('.', "/"), name), text)
            .is_some()
        {
            return Err("Kotlin error interface collision".into());
        }
    }
    Ok(())
}

pub(crate) fn delivery(error: &ErrorBinding, context: &KotlinContext<'_>) -> Result<String> {
    let context = KotlinContext {
        cleanup_failure: Some("cleanupFailure"),
        ..*context
    };
    let mut output = error.output.kotlin(&context)?;
    kotlin::remap_output(
        &mut output,
        context.result,
        context.result,
        1,
        crate::group::result_packet_fields(&error.output.output_outcomes().success).len(),
    );
    let call = format!("onError.run({})", output.delivered.join(", "));
    if output.cleanup.is_empty() {
        return Ok(format!(
            "{}{}\n{call}",
            output.output_locals, output.materialize
        ));
    }
    let cleanup = output.cleanup;
    Ok(format!("{}\nvar cleanupFailure: Throwable? = null\nvar callbackFailed = false\ntry {{\n{}\n{call}\n}} catch (failure: Throwable) {{ callbackFailed = true; cleanupFailure = failure; throw failure }} finally {{\n{cleanup}\nif (!callbackFailed) cleanupFailure?.let {{ throw it }}\n}}", output.output_locals, output.materialize))
}
