//! Function success/error delivery through typed, reusable capture callbacks.
use super::*;
use prebindgen_conversions::Delivery;
use prebindgen_flat::Wire;

#[derive(Clone, Debug)]
pub(crate) enum NativeReturn {
    Unit,
    Direct(JniWire),
    Callback,
}
impl NativeReturn {
    pub(crate) fn rust_type(&self) -> syn::Type {
        match self {
            Self::Direct(wire) => wire.native_return_type(),
            _ => syn::parse_quote!(()),
        }
    }
    pub(crate) fn kotlin_type(&self) -> &str {
        match self {
            Self::Direct(wire) => wire.kotlin_type(),
            _ => "Unit",
        }
    }
}
pub(crate) fn packet_delivery(plan: &OutputPlan<JniWire>) -> Result<Delivery> {
    let env = prebindgen_conversions::template_parameter("environment");
    let count = crate::group::result_packet_fields(&plan.outcomes.success).len() + 1;
    let encode = encode_array(
        crate::group::result_packet(&plan.outcomes.success, &plan.values)?,
        &syn::parse_quote!(result),
        1,
    );
    let mark = prebindgen_conversions::template_parameter("mark_output");
    Ok(Delivery {
        statements: quote!(let result = ::prebindgen_jni_runtime::array(&mut #env, #count)?; #encode #mark),
        result: quote!(result.into_raw()),
    })
}
fn acknowledge_rows(plan: &OutputPlan<JniWire>) -> Result<TokenStream> {
    if sequence::callback_delivery(&plan.outcomes.success)? {
        let env = prebindgen_conversions::template_parameter("environment");
        Ok(
            quote!(#env.set_boolean_array_region(&__pb_output_accepted,0,&[1]).map_err(|e|e.to_string())?;),
        )
    } else {
        Ok(TokenStream::new())
    }
}

pub(crate) fn select(plan: &OutputPlan<JniWire>) -> Option<NativeReturn> {
    let mut packet = packet_delivery(plan).ok()?;
    packet.statements.extend(acknowledge_rows(plan).ok()?);
    if plan.delivery.statements.to_string() != packet.statements.to_string()
        || plan.delivery.result.to_string() != packet.result.to_string()
    {
        return None;
    }
    let fields = crate::group::result_packet_fields(&plan.outcomes.success).collect::<Vec<_>>();
    Some(match fields.as_slice() {
        [] => NativeReturn::Unit,
        [wire] => NativeReturn::Direct(wire.kind.clone()),
        _ => NativeReturn::Callback,
    })
}
fn chunks(wires: &[&Wire<JniWire>]) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut slots = 0;
    for (index, wire) in wires.iter().enumerate() {
        let width = wire.kind.parameter_slots();
        if slots + width > 254 {
            ranges.push(start..index);
            start = index;
            slots = 0;
        }
        slots += width;
    }
    ranges.push(start..wires.len());
    ranges
}

pub(crate) fn callback_delivery(
    plan: &OutputPlan<JniWire>,
    sink: &syn::Ident,
    domain: bool,
) -> Result<Delivery> {
    let env = prebindgen_conversions::template_parameter("environment");
    let fields = crate::group::result_packet(&plan.outcomes.success, &plan.values)?;
    let wires = fields.iter().map(|(wire, _)| *wire).collect::<Vec<_>>();
    let mut prepare = TokenStream::new();
    let mut arguments = Vec::new();
    for (i, (wire, value)) in fields.iter().enumerate() {
        let argument = wire
            .kind
            .output_argument(value, &format_ident!("__pb_return_argument_{i}"));
        prepare.extend(argument.statements);
        arguments.push(argument.value);
    }
    let mut invoke = TokenStream::new();
    for (index, range) in chunks(&wires).into_iter().enumerate() {
        let descriptor = format!(
            "({})V",
            wires[range.clone()]
                .iter()
                .map(|w| w.kind.jvm_descriptor())
                .collect::<String>()
        );
        let arguments = &arguments[range];
        let method = format!("{}{index}", if domain { "domainError" } else { "success" });
        if domain {
            invoke.extend(quote!(unsafe { ::prebindgen_jni_runtime::invoke_error_callback(&mut #env, &#sink, #method, #descriptor, &[#(#arguments),*]) }?;));
        } else {
            let cache = format_ident!("__PB_RETURN_METHOD_{index}");
            invoke.extend(quote!(static #cache: ::prebindgen_jni_runtime::ReturnCallback = ::prebindgen_jni_runtime::ReturnCallback::new();
                unsafe { #cache.invoke(&mut #env, &#sink, #method, #descriptor, &[#(#arguments),*]) }?;));
        }
    }
    Ok(Delivery {
        statements: quote!(#prepare #invoke),
        result: quote!(()),
    })
}
pub(crate) fn delivery(
    plan: &OutputPlan<JniWire>,
    sink: &syn::Ident,
    domain: bool,
) -> Result<Delivery> {
    if domain {
        return callback_delivery(plan, sink, true);
    }
    let mut delivery = match select(plan)
        .ok_or("custom output protocol cannot use a return capture")?
    {
        NativeReturn::Unit => Delivery::default(),
        NativeReturn::Direct(wire) => {
            let fields = crate::group::result_packet(&plan.outcomes.success, &plan.values)?;
            let argument = wire.output_argument(fields[0].1, &format_ident!("__pb_direct_object"));
            let prepare = argument.statements;
            let argument = argument.value;
            let member = wire.value_member();
            Delivery {
                statements: quote!(#prepare let __pb_direct_result = unsafe { (#argument).#member };),
                result: quote!(__pb_direct_result),
            }
        }
        NativeReturn::Callback => callback_delivery(plan, sink, false)?,
    };
    delivery.statements.extend(acknowledge_rows(plan)?);
    Ok(delivery)
}

/// Replace top-level packet reads with typed native locals/capture fields.
/// Nested collection rows keep their independent row arrays and distinct local
/// names; only the caller-supplied top-level `result[index]` references are mapped.
pub(crate) fn remap(
    output: &mut KotlinOutput,
    result: &str,
    fields: &[&Wire<JniWire>],
    offset: usize,
    values: &[String],
) {
    for text in output.delivered.iter_mut().chain([
        &mut output.output_locals,
        &mut output.materialize,
        &mut output.cleanup,
    ]) {
        for (i, (wire, value)) in fields.iter().zip(values).enumerate().rev() {
            let old = format!("{result}[{}]", offset + i);
            if !wire.kind.is_reference() {
                *text = text.replace(&format!("{old} as {}", wire.kind.kotlin_type()), value);
            }
            *text = text.replace(&old, value);
        }
    }
}
pub(crate) fn remap_text(
    text: &str,
    result: &str,
    fields: &[&Wire<JniWire>],
    offset: usize,
    values: &[String],
) -> String {
    let mut output = KotlinOutput {
        materialize: text.into(),
        ..Default::default()
    };
    remap(&mut output, result, fields, offset, values);
    output.materialize
}

pub(crate) fn capture(name: &str, success: &[&Wire<JniWire>], domain: &[&Wire<JniWire>]) -> String {
    let mut fields = String::new();
    let mut reset = String::new();
    let mut methods = String::new();
    for (prefix, method, wires) in [("s", "success", success), ("d", "domainError", domain)] {
        for (i, wire) in wires.iter().enumerate() {
            let ty = wire.kind.kotlin_type();
            let default = wire.kind.kotlin_default();
            fields.push_str(&format!("    var {prefix}{i}: {ty} = {default}\n"));
            if wire.kind.is_reference() {
                reset.push_str(&format!("        {prefix}{i} = {default}\n"));
            }
        }
        let ranges = chunks(wires);
        for (chunk, range) in ranges.iter().enumerate() {
            let params = range
                .clone()
                .map(|i| format!("v{i}: {}", wires[i].kind.kotlin_type()))
                .collect::<Vec<_>>();
            let mut body = String::new();
            for i in range.clone() {
                use std::fmt::Write as _;
                write!(body, "this.{prefix}{i} = v{i}; ").expect("writing to a String");
            }
            if chunk + 1 == ranges.len() {
                body.push_str(if method == "success" {
                    "status = 1"
                } else {
                    "status = 3"
                });
            }
            methods.push_str(&format!(
                "    fun {method}{chunk}({}) {{ {body} }}\n",
                params.join(", ")
            ));
        }
    }
    format!("internal class {name} private constructor(private val pool: Pool) {{\n    var status: Int = 0\n    var message: String? = null\n    var discardSuccess: Boolean = false\n{fields}{methods}    fun bindingError(message: String?, committed: Boolean) {{ this.message = message; discardSuccess = committed; status = 2 }}\n    fun release() {{\n{reset}        status = 0; message = null; discardSuccess = false\n        pool.depth--\n    }}\n    private class Pool {{\n        val frames = java.util.ArrayList<{name}>()\n        var depth = 0\n        fun acquire(): {name} {{\n            val index = depth\n            if (index == frames.size) frames.add({name}(this))\n            depth = index + 1\n            return frames[index]\n        }}\n    }}\n    companion object {{\n        private val local = ThreadLocal.withInitial {{ Pool() }}\n        fun acquire(): {name} = local.get().acquire()\n    }}\n}}\n")
}

pub(crate) fn binding_failure(
    env: &syn::Ident,
    sink: &syn::Ident,
    kind: &NativeReturn,
) -> TokenStream {
    let result = if matches!(kind, NativeReturn::Direct(_)) {
        quote!(direct_output)
    } else {
        TokenStream::new()
    };
    quote! {
        if domain_output.is_none() && matches!(#env.exception_check(), Ok(false)) {
            let reported = ::prebindgen_runtime::boundary(|| {
                let message = ::prebindgen_jni_runtime::text_object(&mut #env, &e)?;
                unsafe { ::prebindgen_jni_runtime::invoke_error_callback(&mut #env, &#sink, "bindingError", "(Ljava/lang/String;Z)V", &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue { l: message.as_raw() },
                    ::prebindgen_jni_runtime::jni::sys::jvalue { z: u8::from(success_transferred) },
                ]) }
            });
            if let Err(message) = reported { ::prebindgen_jni_runtime::fail(&mut #env, &message); }
        }
        #result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fun, package, ptr_class};
    fn prepared() -> Bindings {
        let source = syn::parse_file("pub type Handle=String; pub type Failure=String; pub fn value(x:i64)->i64{} pub fn unit(){} pub fn owned()->Handle{} pub fn failed()->Result<i64,Failure>{} pub fn message(x:Failure)->String{}").unwrap();
        JniGen::builder()
            .items(
                source
                    .items
                    .into_iter()
                    .map(|item| (item, Default::default())),
            )
            .set_package_prefix("test")
            .error_mapper(syn::parse_quote!(Failure), fun!(message))
            .package(
                package!()
                    .class(ptr_class!(Handle))
                    .fun(fun!(value))
                    .fun(fun!(unit))
                    .fun(fun!(owned))
                    .fun(fun!(failed)),
            )
            .prepare()
            .unwrap()
    }
    fn externs(generation: &JniGen) -> BTreeMap<String, syn::ItemFn> {
        syn::parse_file(&generation.rust)
            .unwrap()
            .items
            .into_iter()
            .filter_map(|item| {
                if let syn::Item::Fn(f) = item {
                    if f.sig.abi.is_some() {
                        return Some((f.sig.ident.to_string(), f));
                    }
                }
                None
            })
            .collect()
    }
    #[test]
    fn native_results_do_not_box_or_allocate_success_packets() {
        let generation = generate(&prepared()).unwrap();
        let functions = externs(&generation);
        for name in ["value", "unit", "owned", "failed"] {
            let function = functions
                .iter()
                .find(|(n, _)| n.contains(&format!("call_1{name}_1")))
                .unwrap()
                .1;
            let code = quote!(#function).to_string();
            assert!(
                !code.contains("prebindgen_jni_runtime :: array"),
                "{name}: {code}"
            );
            assert!(!code.contains("valueOf"), "{name}: {code}");
            assert!(code.contains("bindingError"));
            assert!(!quote!(#function).to_string().contains("jobjectArray {"));
        }
        let kotlin = &generation.kotlin["test/Functions.kt"];
        assert!(!kotlin.contains("when (result[0])"));
        assert!(kotlin.contains("returnCapture.release()"));
        assert!(kotlin.contains("if (returnCapture.discardSuccess)"));
        // Scalar and handle functions with the same error payload share one pool.
        let captures = generation
            .kotlin
            .keys()
            .filter(|k| k.contains("/Return_"))
            .count();
        assert_eq!(captures, 2);
    }
    #[test]
    fn a_custom_error_delivery_keeps_the_whole_call_on_its_packet_protocol() {
        let mut flat = prepared();
        let error = flat
            .function_mut("failed")
            .unwrap()
            .metadata
            .as_mut()
            .unwrap()
            .error
            .as_mut()
            .unwrap();
        std::rc::Rc::make_mut(error)
            .output
            .rust
            .delivery
            .statements
            .extend(quote!(let custom_error_protocol = ();));
        let generation = generate(&flat).unwrap();
        let functions = externs(&generation);
        let function = functions
            .iter()
            .find(|(n, _)| n.contains("call_1failed_1"))
            .unwrap()
            .1;
        let code = quote!(#function).to_string();
        assert!(code.contains("jobjectArray"));
        assert!(!code.contains("__pb_return :"));
        assert!(code.contains("binding_packet"));
    }
}
