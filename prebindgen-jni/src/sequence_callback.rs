//! Per-element native delivery and Kotlin folds, without invoking user code under JNI locks.
use super::*;
use prebindgen_declarations::SequenceOutput;
use prebindgen_flat::WireLayout;

pub(crate) fn transport(
    wire: syn::Ident,
    element: &WireLayout<JniWire, JniGroup>,
    values: &[TokenStream],
) -> Result<SequenceOutput<JniWire>> {
    let env = prebindgen_conversions::template_parameter("environment");
    let method = format_ident!("{wire}_callback_method");
    let class = format_ident!("{wire}_callback_class");
    let frame = format_ident!("{wire}_callback_frame");
    let mut descriptor = String::from("(");
    let mut encode = TokenStream::new();
    let mut args = Vec::new();
    let slots: usize = element.iter().map(|w| w.kind.parameter_slots()).sum();
    if slots > 254 {
        return Err("sequence callback exceeds JVM parameter limit".into());
    }
    for (i, (field, value)) in element.iter().zip(values).enumerate() {
        let primitive = match field.kind.kotlin_type() {
            "Boolean" => Some(("Z", "z")),
            "Byte" => Some(("B", "b")),
            "Short" => Some(("S", "s")),
            "Int" => Some(("I", "i")),
            "Long" => Some(("J", "j")),
            "Float" => Some(("F", "f")),
            "Double" => Some(("D", "d")),
            _ => None,
        };
        if let Some((signature, member)) = primitive {
            descriptor.push_str(signature);
            let member = format_ident!("{member}");
            let value = if matches!(field.kind, JniWire::Handle { .. }) {
                quote!((#value) as i64)
            } else {
                value.clone()
            };
            args.push(quote!(::prebindgen_jni_runtime::jni::sys::jvalue {#member:#value}));
        } else {
            let object = format_ident!("{wire}_callback_object_{i}");
            let (signature, expression) = match &field.kind {
                JniWire::Text { .. } => (
                    "Ljava/lang/String;",
                    quote!(match (#value).as_ref(){Some(value)=>::prebindgen_jni_runtime::text_object(&mut #env,value)?,None=>::prebindgen_jni_runtime::jni::objects::JObject::null()}),
                ),
                JniWire::Bytes { .. } => (
                    "[B",
                    quote!(match (#value).as_ref(){Some(value)=>::prebindgen_jni_runtime::bytes_object(&mut #env,value)?,None=>::prebindgen_jni_runtime::jni::objects::JObject::null()}),
                ),
                JniWire::Rows { .. } => (
                    "[Ljava/lang/Object;",
                    quote!(match (#value).as_ref(){Some(value)=>#env.new_local_ref(value).map_err(|e|e.to_string())?,None=>::prebindgen_jni_runtime::jni::objects::JObject::null()}),
                ),
                JniWire::Scalar {
                    primitive: Primitive::I128 | Primitive::U128,
                    ..
                } => (
                    "[B",
                    quote!(::prebindgen_jni_runtime::bytes_object(&mut #env, &#value)?),
                ),
                _ => return Err("unsupported sequence callback wire".into()),
            };
            descriptor.push_str(signature);
            encode.extend(quote!(let #object=#expression;));
            args.push(quote!(::prebindgen_jni_runtime::jni::sys::jvalue{l:#object.as_raw()}));
        }
    }
    descriptor.push_str(")V");
    let capacity = i32::try_from(element.len() + 8).map_err(|_| "too many callback fields")?;
    Ok(SequenceOutput {
        commit_each: false,
        fallible: true,
        carrier: JniWire::Rows {
            condition: "true".into(),
            null_niche_available: true,
        },
        begin: quote! {
            let #class=#env.get_object_class(&__pb_callback).map_err(|e|e.to_string())?;
            let #method=#env.get_method_id(&#class,"run",#descriptor).map_err(|e|e.to_string())?;
            #env.delete_local_ref(#class).map_err(|e|e.to_string())?;
        },
        enter: quote!(let #frame=unsafe{::prebindgen_jni_runtime::LocalFrame::new(&#env,#capacity)}?;),
        store: quote! {
            #encode
            // The method was resolved on this callback object with exactly these
            // JVM wire types. The generated bridge only retains raw fields.
            unsafe {#env.call_method_unchecked(&__pb_callback,#method,::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(::prebindgen_jni_runtime::jni::signature::Primitive::Void),&[#(#args),*])}.map_err(|e|e.to_string())?;
            drop(#frame);
        },
        // Only a presence marker crosses back; row payloads crossed through run.
        finish: quote!(let #wire=Some(::prebindgen_jni_runtime::array(&mut #env,0)?);),
        value: quote!(#wire),
        commit: quote!(),
    })
}

pub(crate) struct Fold {
    pub output: KotlinOutput,
    pub parameter: String,
    pub discard: String,
}
pub(crate) fn fold(
    layout: &WireLayout<JniWire, JniGroup>,
    context: &KotlinContext<'_>,
) -> Result<Fold> {
    fn optional(nodes: &[prebindgen_flat::WireNode<JniWire, JniGroup>]) -> bool {
        nodes.iter().any(|node| match node {
            prebindgen_flat::WireNode::Group {
                metadata: JniGroup::Optional(_),
                ..
            } => true,
            prebindgen_flat::WireNode::Group { children, .. } => optional(children.nodes()),
            _ => false,
        })
    }
    let optional = optional(layout.nodes());
    let (carrier, element) = prebindgen_declarations::sequence_return(layout)
        .ok_or("missing sequence callback layout")?;
    let position = layout.iter().position(|w| w.name == carrier.name).unwrap();
    let row = "__pb_fold_row";
    let ctx = KotlinContext {
        result: row,
        output_prefix: "foldElement",
        cleanup_failure: None,
        ..*context
    };
    let child = kotlin::output_kotlin(element.nodes(), &ctx)?;
    let present = format!("{}[{position}] != null", context.result);
    let raw = "__pb_rows";
    let types = child.result_types.join(", ");
    let args = child.delivered.join(", ");
    let failure = context.failure;
    Ok(Fold {
        parameter:format!("fold: (A, {types}) -> A"),
        discard:format!("fun __pb_discard({failure}: Throwable) {{ for (rawRow in __pb_rows) {{ val {row} = rawRow as kotlin.Array<*>\n{}\n{}\n}} }}\n",child.output_locals,child.cleanup),
        output:KotlinOutput {
            delivered:vec![if optional{"__pb_accumulator".into()}else{"__pb_accumulator as A".into()}],
            result_types:vec![if optional{"A?".into()}else{"A".into()}],
            output_locals:"var __pb_accumulator: A? = null\nvar __pb_processed = 0\nvar __pb_pending: ((Throwable) -> Unit)? = null\n".into(),
            materialize:format!("if ({present}) {{\nvar current = initial\nfor (rawRow in {raw}) {{\nval {row} = rawRow as kotlin.Array<*>\n{}\n__pb_pending = {{ {failure} -> {} }}\n{}\ncurrent = fold(current, {args})\n__pb_pending = null\n__pb_processed++\n}}\n__pb_accumulator = current\n}}\n",child.output_locals,child.cleanup,child.materialize),
            cleanup:format!("__pb_pending?.invoke({failure})\nif ({present}) for (index in (__pb_processed + if (__pb_pending != null) 1 else 0) until {raw}.size) {{\nval {row} = {raw}[index] as kotlin.Array<*>\n{}\n{}\n}}\n",child.output_locals,child.cleanup),
        },
    })
}
