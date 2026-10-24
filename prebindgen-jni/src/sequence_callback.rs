//! Per-element native delivery and Kotlin folds, without invoking user code under JNI locks.
use super::*;
use prebindgen_declarations::SequenceOutput;

pub(crate) fn transport(
    wire: syn::Ident,
    element: &prebindgen_flat::WireNode<
        JniWire,
        crate::JniOptional,
        crate::JniSequence,
        JniGroup,
        crate::JniSelector,
    >,
    values: &[(syn::Ident, TokenStream)],
) -> Result<SequenceOutput<crate::JniSequence>> {
    let env = prebindgen_conversions::template_parameter("environment");
    let method = format_ident!("{wire}_callback_method");
    let class = format_ident!("{wire}_callback_class");
    let frame = format_ident!("{wire}_callback_frame");
    let mut descriptor = String::from("(");
    let mut encode = TokenStream::new();
    let mut args = Vec::new();
    let slots: usize = crate::group::callback_parameters(element)
        .map(|w| w.kind.parameter_slots())
        .sum();
    if slots > 254 {
        return Err("sequence callback exceeds JVM parameter limit".into());
    }
    for (i, (field, value)) in prebindgen_conversions::match_output_fields(
        crate::group::callback_parameters(element),
        values,
    )?
    .into_iter()
    .enumerate()
    {
        descriptor.push_str(field.kind.jvm_descriptor());
        let local = format_ident!("{wire}_callback_object_{i}");
        let argument = field.kind.output_argument(value, &local);
        encode.extend(argument.statements);
        args.push(argument.value);
    }
    descriptor.push_str(")V");
    let capacity = i32::try_from(crate::group::callback_parameters(element).len() + 8)
        .map_err(|_| "too many callback fields")?;
    Ok(SequenceOutput {
        commit_each: false,
        fallible: true,
        metadata: JniSequence::callback(wire.clone()),
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
        values: vec![(wire.clone(), quote!(#wire))],
        commit: quote!(),
    })
}

pub(crate) struct Fold {
    pub output: KotlinOutput,
    pub parameter: String,
    pub discard: String,
}
pub(crate) fn fold(
    layout: &prebindgen_flat::WireNode<
        JniWire,
        crate::JniOptional,
        crate::JniSequence,
        JniGroup,
        crate::JniSelector,
    >,
    context: &KotlinContext<'_>,
) -> Result<Fold> {
    fn optional(
        node: &prebindgen_flat::WireNode<
            JniWire,
            JniOptional,
            crate::JniSequence,
            JniGroup,
            crate::JniSelector,
        >,
    ) -> bool {
        match node {
            prebindgen_flat::WireNode::Optional { .. } => true,
            prebindgen_flat::WireNode::Group { elements, .. } => elements.iter().any(optional),
            _ => false,
        }
    }
    let optional = optional(layout);
    let (sequence, element) =
        crate::sequence::return_sequence(layout).ok_or("missing sequence callback layout")?;
    let position = crate::group::result_packet_fields(layout)
        .position(|w| w.name == sequence.carrier().name)
        .unwrap();
    let row = "__pb_fold_row";
    let ctx = KotlinContext {
        result: row,
        output_prefix: "foldElement",
        cleanup_failure: None,
        ..*context
    };
    let child = kotlin::output_kotlin(element, &ctx)?;
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
