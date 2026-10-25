//! JNI error channels reuse ordinary output conversions. User callbacks run in Kotlin.
use super::*;
use prebindgen_conversions::OutputFragment;
use prebindgen_flat::OutputConversion;
use prebindgen_flat::{Conversion, Function, TypeUse};

#[derive(Clone, Debug)]
pub(crate) struct ErrorBinding {
    pub(crate) helper: syn::Ident,
    ty: syn::Type,
    pub output: JniOutput,
    pub handler: String,
    exception: Option<String>,
}
impl ErrorBinding {
    pub fn throws(&self) -> String {
        self.exception.as_ref().map_or_else(String::new, |class| {
            format!("@kotlin.jvm.Throws({}::class)\n", class.replace('$', "."))
        })
    }
    pub fn parameter(&self, ret: &str) -> String {
        let default = self.exception.as_ref().map_or_else(String::new, |class| {
            format!(
                " = {} {{ message -> throw {}(message) }}",
                self.handler,
                class.replace('$', ".")
            )
        });
        format!("onError: {}<{ret}>{default}", self.handler)
    }
}

pub(crate) fn prepare(
    flat: &mut Bindings,
    codec: &mut data::DataCodec,
    declarations: &prebindgen_declarations::Declarations,
    functions: &[FunctionDecl],
    classes: &BTreeMap<String, Class>,
    prefix: &str,
    seen: &mut BTreeSet<String>,
) -> Result<BTreeMap<String, std::rc::Rc<ErrorBinding>>> {
    let mut out = BTreeMap::new();
    let mut cache: BTreeMap<(String, bool), std::rc::Rc<ErrorBinding>> = BTreeMap::new();
    let mut handlers: BTreeMap<(String, bool), String> = BTreeMap::new();
    for function in functions {
        let ret = flat.function(&function.name)?.ret.ty.clone();
        let Some((_, error)) = prebindgen_flat::result_parts(&ret) else {
            continue;
        };
        let mapped = declarations.error_mapper_for(function, error).is_some();
        let key = (type_key(error), mapped);
        if let Some(binding) = cache.get(&key) {
            out.insert(function.name.clone(), binding.clone());
            continue;
        }
        let exception = codec.error_classes.get(&key.0).cloned();
        if exception.is_some() && !mapped {
            return Err(
                "error_class requires an explicit message mapper for its default Kotlin handler"
                    .into(),
            );
        }
        let (ty, recipe) = if mapped {
            (
                syn::parse_quote!(String),
                prebindgen_declarations::Output::Leaf,
            )
        } else {
            (
                error.clone(),
                declarations
                    .output_recipe(flat, codec, error, &())?
                    .into_preparation(),
            )
        };
        let helper = format_ident!("__pb_jni_error_{}", out.len());
        if flat.functions.contains_key(&helper.to_string()) {
            return Err("JNI error helper name collision".into());
        }
        flat.functions.insert(
            helper.to_string(),
            Function {
                metadata: Default::default(),
                name: helper.clone(),
                path: syn::parse_quote!(#helper),
                params: vec![],
                ret: TypeUse {
                    ty: ty.clone(),
                    slot: None,
                },
                location: Default::default(),
            },
        );
        let result =
            prebindgen_declarations::bind_output(flat, codec, &helper.to_string(), &recipe, &());
        let output = flat.functions.remove(&helper.to_string()).unwrap().ret.slot;
        result.map_err(|e| format!("JNI error `{}` in `{}`: {e}", key.0, function.name))?;
        let output = output.unwrap();
        let handler = if let Some(handler) = handlers.get(&key) {
            handler.clone()
        } else {
            let short = if let syn::Type::Path(path) = error {
                path.path.segments.last().unwrap().ident.to_string()
            } else {
                let mut name = camel(&function.name);
                if let Some(first) = name.get_mut(..1) {
                    first.make_ascii_uppercase();
                }
                format!("{name}Error")
            };
            let pkg = classes
                .get(&key.0)
                .map_or(prefix, |class| class.pkg.as_str());
            let suffix = if mapped { "MessageHandler" } else { "Handler" };
            let handler = format!("{pkg}.{short}{suffix}");
            if !seen.insert(handler.clone()) {
                return Err(format!("duplicate foreign name `{handler}`"));
            }
            handlers.insert(key.clone(), handler.clone());
            handler
        };
        // Each function can have its own mapper; the helper converts the already mapped String.
        let binding = ErrorBinding {
            helper,
            ty,
            output,
            handler,
            exception,
        };
        let binding = std::rc::Rc::new(binding);
        flat.type_metadata_mut(error)?
            .errors
            .insert(mapped, binding.clone());
        codec.errors.insert(key.clone(), binding.clone());
        cache.insert(key, binding.clone());
        out.insert(function.name.clone(), binding);
    }
    Ok(out)
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
