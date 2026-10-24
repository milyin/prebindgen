//! Array row transport and Kotlin list conversion; schemas live in sequence nodes.
use super::*;
use crate::kotlin::{
    input_kotlin, output_kotlin, KotlinBinding, KotlinContext, KotlinInput, KotlinOutput,
};
use prebindgen_declarations::{Condition, SequenceInput, SequenceOutput};
use prebindgen_flat::Wire;

pub(crate) fn input(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &[prebindgen_flat::WireNode<JniWire, JniGroup>],
    active: &Condition,
) -> Result<SequenceInput<JniGroup>> {
    let env = prebindgen_conversions::template_parameter("environment");
    let row = format_ident!("{wire}_row");
    let count = crate::group::row_fields(element).len();
    let capacity = i32::try_from(count + 8).map_err(|_| "too many collection element fields")?;
    let frame = format_ident!("{wire}_frame");
    let mut bind = quote! {let #frame=unsafe {::prebindgen_jni_runtime::LocalFrame::new(&#env,#capacity)}?;let #row=::prebindgen_jni_runtime::row(&mut #env,&#wire,#index,#count)?;};
    let mut validation_values = Vec::new();
    let mut prepared_values = Vec::new();
    for (position, field) in crate::group::row_fields(element).enumerate() {
        let name = &field.name;
        let read = field.kind.read_array_element(&quote!(#row), position);
        bind.extend(quote! {
            let #name=#read;
            if #name.is_err() && #env.exception_check().map_err(|e|e.to_string())? {return Err("JNI collection field decoding failed".into());}
        });
        let borrow = quote!(*);
        validation_values.push((
            field.name.clone(),
            quote!(#borrow #name.as_ref().map_err(|e|e.clone())?),
        ));
        prepared_values.push((
            field.name.clone(),
            if field.kind.is_reference() {
                None
            } else {
                Some(quote!(#borrow #name.as_ref().expect("validated collection field")))
            },
        ));
    }
    Ok(SequenceInput {
        metadata: JniGroup::Sequence(Wire {
            name: wire.clone(),
            kind: JniWire::Rows {
                condition: kotlin_condition(active),
                null_niche_available: true,
            },
        }),
        open: quote! {if #wire.is_null(){return Err("null collection".into());}let #length=#env.get_array_length(&#wire).map_err(|e|e.to_string())? as usize;},
        bind,
        validation_values,
        prepared_values,
        release: quote!(drop(#frame);),
    })
}
pub(crate) fn output(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &[prebindgen_flat::WireNode<JniWire, JniGroup>],
    values: &[(syn::Ident, TokenStream)],
) -> Result<SequenceOutput<JniGroup>> {
    let env = prebindgen_conversions::template_parameter("environment");
    let row = format_ident!("{wire}_row");
    let count = crate::group::row_fields(element).len();
    let capacity = i32::try_from(count + 8).map_err(|_| "too many collection fields")?;
    let encode = encode_array(crate::group::row_values(element, values)?, &row, 0);
    let frame = format_ident!("{wire}_frame");
    Ok(SequenceOutput {
        commit_each: false,
        fallible: true,
        metadata: JniGroup::Sequence(Wire {
            name: wire.clone(),
            kind: JniWire::Rows {
                condition: "true".into(),
                null_niche_available: true,
            },
        }),
        begin: quote!(let #wire=::prebindgen_jni_runtime::array(&mut #env,#length)?;),
        enter: quote!(let #frame=unsafe {::prebindgen_jni_runtime::LocalFrame::new(&#env,#capacity)}?;),
        store: quote! {
            let #row=::prebindgen_jni_runtime::array(&mut #env,#count)?;#encode
            #env.set_object_array_element(&#wire,#index as i32,&#row).map_err(|e|e.to_string())?;
            drop(#frame);
        },
        finish: quote!(let #wire=Some(#wire);),
        values: vec![(wire.clone(), quote!(#wire))],
        commit: quote!(),
    })
}

pub(crate) fn input_list(
    carrier: &Wire<JniWire>,
    element: &[prebindgen_flat::WireNode<JniWire, JniGroup>],
    context: &KotlinContext<'_>,
) -> Result<KotlinInput> {
    let child = input_kotlin(element, context)?;
    if child.parameters.len() != 1 {
        return Err("Kotlin collection elements must expose exactly one value; declare a data class for expanded fields".into());
    }
    let (parameter, ty) = child.parameters[0]
        .split_once(": ")
        .ok_or("invalid element parameter")?;
    let name = carrier.name.to_string();
    let state = format!("{name}_elements");
    let JniWire::Rows { condition, .. } = &carrier.kind else {
        return Err("sequence node requires an array carrier".into());
    };
    let mut code = String::new();
    for binding in &child.setup {
        code.push_str(&format!("val {} = {}\n", binding.name, binding.expression));
    }
    let mut locks = if child.locks.is_empty() {
        "kotlin.collections.emptyList<Nothing>()".into()
    } else {
        format!(
            "kotlin.collections.listOfNotNull({})",
            child.locks.join(", ")
        )
    };
    for group in &child.lock_groups {
        locks = format!("({locks} + ({group}))");
    }
    let expression=format!("if ({condition}) {name}?.toList()?.map {{ {parameter} ->\n{code}kotlin.Triple({{ kotlin.arrayOf<kotlin.Any?>({}) }}, {{ {locks} }}, {{ {}; kotlin.Unit }})\n}} else null",child.arguments.join(", "),child.consumed.join("; "));
    Ok(KotlinInput {
        parameters: vec![format!(
            "{name}: kotlin.collections.List<{ty}>{}",
            if condition == "true" { "" } else { "?" }
        )],
        setup: vec![KotlinBinding {
            name: state.clone(),
            expression,
            absent: "null".into(),
        }],
        arguments: vec![format!(
            "{state}?.map {{ it.first() }}?.toTypedArray<kotlin.Any?>()"
        )],
        locks: Vec::new(),
        lock_groups: if child.locks.is_empty() && child.lock_groups.is_empty() {
            vec![]
        } else {
            vec![format!("{state}.orEmpty().flatMap {{ it.second() }}")]
        },
        consumed: if child.consumed.is_empty() {
            vec![]
        } else {
            vec![format!("{state}.orEmpty().forEach {{ it.third() }}")]
        },
    })
}

pub(crate) fn output_list(
    element: &[prebindgen_flat::WireNode<JniWire, JniGroup>],
    context: &KotlinContext<'_>,
) -> Result<KotlinOutput> {
    let row = format!("{}_row", context.output_prefix);
    let prefix = format!("{}item_", context.output_prefix);
    let failure = format!("{}failure", context.output_prefix);
    let ctx = KotlinContext {
        failure: &failure,
        cleanup_failure: None,
        result: &row,
        output_prefix: &prefix,
        ..*context
    };
    let child = output_kotlin(element, &ctx)?;
    if child.delivered.len() != 1 {
        return Err("Kotlin collection elements must deliver exactly one value; declare a data class for expanded outputs".into());
    }
    let ty = &child.result_types[0];
    let value = &child.delivered[0];
    let raw = format!("({}[0] as kotlin.Array<*>)", context.result);
    let output = format!("{}list", context.output_prefix);
    let actions = format!("{}cleanups", context.output_prefix);
    let (failure_value, finish_cleanup) = match context.cleanup_failure {
        Some(target) => (
            format!("{target} ?: kotlin.RuntimeException(\"collection cleanup failed\")"),
            format!("if ({target} == null && {failure}.suppressed.isNotEmpty()) {{ {target} = {failure}.suppressed.first(); {failure}.suppressed.drop(1).forEach {{ {target}!!.addSuppressed(it) }} }}"),
        ),
        None => (context.failure.to_owned(), String::new()),
    };
    Ok(KotlinOutput {
        delivered:vec![format!("requireNotNull({output})")],result_types:vec![format!("kotlin.collections.List<{ty}>")],
        output_locals:format!("var {output}: kotlin.collections.MutableList<{ty}>? = null\nvar {actions}: kotlin.collections.MutableList<(kotlin.Throwable) -> kotlin.Unit>? = null\n"),
        materialize:format!("{actions} = kotlin.collections.mutableListOf()\n{output} = kotlin.collections.mutableListOf()\nfor (rawRow in {raw}) {{\nval {row} = rawRow as kotlin.Array<*>\n{}\n{actions}.add {{ {failure} -> {} }}\n{}\n{output}.add({value})\n}}\n",child.output_locals,child.cleanup,child.materialize),
        cleanup:format!("val {failure} = {failure_value}\n{actions}?.forEach {{ it({failure}) }}\nfor (index in ({actions}?.size ?: 0) until {raw}.size) {{\nval {row} = {raw}[index] as kotlin.Array<*>\n{}\n{}\n}}\n{finish_cleanup}\n",child.output_locals,child.cleanup),
    })
}

/// The adapter selects its native scope before looking for a sequence.
#[allow(clippy::type_complexity)]
pub(crate) fn return_sequence(
    layout: &[prebindgen_flat::WireNode<JniWire, JniGroup>],
) -> Option<(
    &Wire<JniWire>,
    &[prebindgen_flat::WireNode<JniWire, JniGroup>],
)> {
    let success = match layout {
        [prebindgen_flat::WireNode::Outcomes { success, .. }] => success.as_slice(),
        nodes => nodes,
    };
    prebindgen_declarations::outer_sequences(success)
        .into_iter()
        .find_map(|c| {
            c.metadata
                .as_ref()
                .and_then(JniGroup::sequence_wire)
                .map(|w| (w, c.element.as_slice()))
        })
}
pub(crate) fn callback_target(
    layout: &[prebindgen_flat::WireNode<JniWire, JniGroup>],
) -> Result<syn::Ident> {
    let success = match layout {
        [prebindgen_flat::WireNode::Outcomes { success, .. }] => success.as_slice(),
        nodes => nodes,
    };
    if prebindgen_declarations::outer_sequences(success).len() != 1
        || crate::group::result_packet_fields(layout).len() > 2
    {
        return Err(
            "callback_return permits only one sequence and its optional presence flag".into(),
        );
    }
    return_sequence(layout)
        .map(|(wire, _)| wire.name.clone())
        .ok_or_else(|| "callback_return requires a sequence boundary".into())
}
