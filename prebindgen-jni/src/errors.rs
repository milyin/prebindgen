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

/// Prepare each error once on its type and attach each function's selected binding.
pub(crate) fn prepare(
    flat: &mut Bindings,
    codec: &data::DataCodec,
    declarations: &prebindgen_declarations::Declarations,
    functions: &[FunctionDecl],
) -> Result<()> {
    let mut result_count = 0usize;
    for function in functions {
        let ret = flat.function(&function.name)?.ret.ty.clone();
        let Some((_, error)) = prebindgen_flat::result_parts(&ret) else {
            continue;
        };
        flat.type_metadata_mut(error)?;
        let mapped = declarations.error_mapper_for(function, error).is_some();
        let mut helper_index = result_count;
        result_count += 1;
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
                error.clone(),
                declarations.output_recipe(flat, codec, error, &())?,
            )
        };
        let helper = loop {
            let candidate = format_ident!("__pb_jni_error_{}", helper_index);
            if !flat
                .types
                .values()
                .any(|t| t.metadata.errors.values().any(|e| e.helper == candidate))
            {
                break candidate;
            }
            helper_index += 1;
        };
        let output = prebindgen_declarations::compile_output_value(flat, codec, &ty, &recipe, &())
            .map_err(|e| {
                format!(
                    "JNI error `{}` in `{}`: {e}",
                    type_key(error),
                    function.name
                )
            })?;
        let short = if let syn::Type::Path(path) = error {
            path.path.segments.last().unwrap().ident.to_string()
        } else {
            let mut name = camel(&function.name);
            if let Some(first) = name.get_mut(..1) {
                first.make_ascii_uppercase();
            }
            format!("{name}Error")
        };
        let pkg = flat
            .type_info(error)?
            .metadata
            .class
            .as_ref()
            .map_or(flat.metadata.package.as_str(), |c| c.pkg.as_str());
        let suffix = if mapped { "MessageHandler" } else { "Handler" };
        let handler = format!("{pkg}.{short}{suffix}");
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
            is_domain_error: true,
            output_committed: quote!(*reported = Some(result.as_raw());),
        };
        let commit = error.output.code_fragment_slot(
            OutputFragment::Commit {
                context: &context,
                source: &source,
            },
            &syn::parse_quote!(()),
        )?;
        let body = error.output.code_fragment_slot(
            OutputFragment::Deliver {
                context: &context,
                source: &source,
                next: quote!({let _: () = #commit; Ok(())}),
            },
            &syn::parse_quote!(::prebindgen_runtime::Result<()>),
        )?;
        let body = error.output.code_fragment_slot(
            OutputFragment::Prepare {
                context: &context,
                source: &source,
                next: body,
            },
            &syn::parse_quote!(::prebindgen_runtime::Result<()>),
        )?;
        code.extend(quote! {
            #[allow(clippy::needless_borrow, clippy::let_unit_value, clippy::unnecessary_cast, clippy::useless_conversion)]
            fn #helper(mut env: &mut ::prebindgen_jni_runtime::jni::JNIEnv<'_>, source: #ty, reported: &mut Option<::prebindgen_jni_runtime::jni::sys::jobjectArray>) -> ::prebindgen_runtime::Result<()> { #body }
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
        return Ok(format!("{}\n{call}", output.materialize));
    }
    let cleanup = output.cleanup;
    Ok(format!("{}\nvar cleanupFailure: Throwable? = null\nvar callbackFailed = false\ntry {{\n{}\n{call}\n}} catch (failure: Throwable) {{ callbackFailed = true; cleanupFailure = failure; throw failure }} finally {{\n{cleanup}\nif (!callbackFailed) cleanupFailure?.let {{ throw it }}\n}}", output.output_locals, output.materialize))
}
