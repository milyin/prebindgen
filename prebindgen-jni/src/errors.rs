//! Typed error handlers use the Err child of the function's ordinary output converter.
use super::*;

/// Replace the ordinary Err payload converter of a prepared function. Explicit
/// message mappers stay in front of this converter; other exports remain unchanged.
pub fn bind_error_output<I, O: KotlinOutputConversion>(
    flat: &mut Flat<JniSchema<I, O>>,
    function: &str,
    output: impl KotlinOutputConversion + 'static,
) -> Result<()> {
    let source = flat.function(function)?;
    if source.metadata.len() != 1 {
        return Err("select a binding explicitly when replacing its error output".into());
    }
    let binding = source.metadata[0].name.to_string();
    bind_export_error_output(flat, function, &binding, output)
}
/// Replace the Err payload converter of one explicitly selected prepared export.
pub fn bind_export_error_output<I, O: KotlinOutputConversion>(
    flat: &mut Flat<JniSchema<I, O>>,
    source: &str,
    binding: &str,
    output: impl KotlinOutputConversion + 'static,
) -> Result<()> {
    let function = flat.function_mut(source)?;
    let export = function
        .metadata
        .iter_mut()
        .find(|b| b.name == binding)
        .ok_or("unknown binding")?;
    let identity = format!("{}::{}", names::path_identity(&function.path), export.name);
    let hash = prebindgen_reprc::schema_id(&identity);
    let label = export.name.to_string();
    let label = label.trim_start_matches("r#");
    let prebindgen_declarations::BindingState::Prepared(call) = &mut export.state else {
        return Err("function has no prepared domain-error converter".into());
    };
    let mut error = call
        .output
        .domain_error()
        .ok_or("function has no prepared domain-error converter")?
        .clone();
    let package = error
        .handler
        .rsplit_once('.')
        .ok_or("domain-error handler has no package")?
        .0;
    error.handler = format!("{package}.LocalErrorHandler_{label}_{hash:016x}");
    error.replace_output(JniOutputConverter::new(output))?;
    call.output = call.output.with_domain_error(error)?;
    Ok(())
}

pub(crate) fn handler_name(flat: &JniBindings, ty: &syn::Type, mapped: bool) -> Result<String> {
    let info = flat.type_info(ty).ok();
    if info.is_some_and(|info| info.metadata.error_class.is_some()) && !mapped {
        return Err(
            "error_class requires an explicit message mapper for its default Kotlin handler".into(),
        );
    }
    let ty = info.map_or(ty, |info| &info.ty);
    let pkg = info
        .and_then(|info| info.metadata.class.as_ref())
        .map_or(flat.metadata.package.as_str(), |c| c.pkg.as_str());
    let suffix = if mapped { "MessageHandler" } else { "Handler" };
    let short = if let syn::Type::Path(path) = ty {
        if path.qself.is_none()
            && path
                .path
                .segments
                .iter()
                .all(|s| matches!(s.arguments, syn::PathArguments::None))
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
    let short = match short {
        Some(short) => short,
        None => {
            let mode = if mapped { "mapped" } else { "typed" };
            names::Names::default().name(
                "ErrorHandler",
                &format!("{}_{mode}", names::type_label(ty)),
                &[&names::type_identity(ty), mode],
            )?
        }
    };
    let handler = format!("{pkg}.{short}");
    if foreign_name_exists(flat, &handler) {
        return Err(format!("duplicate foreign name `{handler}`"));
    }
    Ok(handler)
}

pub(crate) fn interfaces(prefix: &str, files: &mut BTreeMap<String, String>) -> Result<()> {
    let binding_name = format!("{}/JniErrorHandler.kt", prefix.replace('.', "/"));
    if files.insert(binding_name, format!("package {prefix}\n\n/** Binding failure callback, invoked after the native call and handle locks finish. */\npublic fun interface JniErrorHandler<out R> {{ public fun run(message: String?): R }}\n")).is_some() {
        return Err("Kotlin error interface collision".into());
    }
    Ok(())
}

pub(crate) fn handler_files(
    errors: &[&JniDomainError],
    context: &KotlinContext<'_>,
    files: &mut BTreeMap<String, String>,
) -> Result<()> {
    let mut seen = BTreeMap::new();
    for error in errors {
        let (pkg, name) = error.handler.rsplit_once('.').unwrap();
        let output = error.output().kotlin(context)?;
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
        let path = format!("{}/{}.kt", pkg.replace('.', "/"), name);
        if let Some(previous) = seen.insert(path.clone(), text.clone()) {
            if previous != text {
                return Err(format!(
                    "incompatible Kotlin error handler `{}`",
                    error.handler
                ));
            }
            continue;
        }
        if files.insert(path, text).is_some() {
            return Err("Kotlin error interface collision".into());
        }
    }
    Ok(())
}

pub(crate) fn delivery(error: &JniDomainError, context: &KotlinContext<'_>) -> Result<String> {
    let context = KotlinContext {
        cleanup_failure: Some("cleanupFailure"),
        ..*context
    };
    let mut output = error.output().kotlin(&context)?;
    kotlin::remap_output(
        &mut output,
        context.result,
        context.result,
        1,
        crate::group::result_packet_fields(&error.output().output_outcomes().success).len(),
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
