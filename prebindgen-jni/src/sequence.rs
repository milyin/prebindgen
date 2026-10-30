//! Array row transport and Kotlin list conversion; schemas live in sequence nodes.
use super::*;
use crate::kotlin::{
    input_kotlin, output_kotlin, KotlinBinding, KotlinContext, KotlinInput, KotlinOutput,
};
use prebindgen_declarations::{Condition, SequenceInput, SequenceOutput};

/// Selected delivery of a logical sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JniSequence {
    /// All rows travel in one collection carrier.
    Buffered,
    /// Native row ranges inside an enclosing bulk frame.
    Bulk,
    /// Each row is delivered through the function's sequence callback.
    Callback,
}

pub(crate) fn input(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    active: &Condition,
) -> Result<SequenceInput<JniWire, crate::JniChain>> {
    let env = prebindgen_conversions::template_parameter("environment");
    let row = format_ident!("{wire}_row");
    let count = crate::group::row_fields(element).len();
    let capacity = i32::try_from(count + 8).map_err(|_| "too many collection element fields")?;
    let frame = format_ident!("{wire}_frame");
    let mut bind = quote! {let #frame=unsafe {::prebindgen_jni_runtime::LocalFrame::new(&#env,#capacity)}?;let #row=::prebindgen_jni_runtime::row(&mut #env,&#wire,#index,#count)?;};
    let mut validation_values = Vec::new();
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
    }
    Ok(SequenceInput {
        controls: vec![prebindgen_flat::Wire {
            name: wire.clone(),
            kind: JniWire::Rows {
                condition: kotlin_condition(active),
                null_niche_available: true,
            },
        }],
        metadata: JniChain::Sequence(JniSequence::Buffered),
        open: quote! {if #wire.is_null(){return Err("null collection".into());}let #length=#env.get_array_length(&#wire).map_err(|e|e.to_string())? as usize;},
        bind,
        validation_values,
        release: quote!(drop(#frame);),
    })
}
pub(crate) fn output(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    values: &[(syn::Ident, TokenStream)],
) -> Result<SequenceOutput<JniWire, crate::JniChain>> {
    let env = prebindgen_conversions::template_parameter("environment");
    let row = format_ident!("{wire}_row");
    let count = crate::group::row_fields(element).len();
    let capacity = i32::try_from(count + 8).map_err(|_| "too many collection fields")?;
    let encode = encode_array(crate::group::row_values(element, values)?, &row, 0);
    let frame = format_ident!("{wire}_frame");
    Ok(SequenceOutput {
        controls: vec![prebindgen_flat::Wire {
            name: wire.clone(),
            kind: JniWire::Rows {
                condition: ("true").into(),
                null_niche_available: true,
            },
        }],
        commit_each: false,
        fallible: true,
        metadata: JniChain::Sequence(JniSequence::Buffered),
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
    carrier: &prebindgen_flat::Wire<JniWire>,
    sequence: &JniSequence,
    element: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    context: &KotlinContext<'_>,
) -> Result<KotlinInput> {
    if *sequence == JniSequence::Bulk {
        return crate::bulk::input_list(carrier, element, context);
    }
    if *sequence != JniSequence::Buffered {
        return Err(
            "callback sequence is only supported as the function's successful return".into(),
        );
    }
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
        return Err("sequence requires an object-array control".into());
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
    let expression=format!("if ({condition}) {name}?.toList()?.map {{ {parameter} ->\n{code}kotlin.Triple({{ kotlin.arrayOf<kotlin.Any?>({}) }}, {{ {locks} }}, {{ __pb_consume: ({}.NativeHandle) -> Unit -> {}; kotlin.Unit }})\n}} else null",child.arguments.join(", "),context.package,child.consumed.join("; "));
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
            vec![format!(
                "{state}.orEmpty().forEach {{ it.third(__pb_consume) }}"
            )]
        },
    })
}

pub(crate) fn output_list(
    element: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    context: &KotlinContext<'_>,
) -> Result<KotlinOutput> {
    let row = format!("{}_row", context.output_prefix);
    let prefix = format!("{}item_", context.output_prefix);
    let failure = format!("{}failure", context.output_prefix);
    let ctx = KotlinContext {
        failure: &failure,
        cleanup_failure: Some(&failure),
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
    let initial = format!("{}initialFailure", context.output_prefix);
    let action = format!("{}cleanupAction", context.output_prefix);
    let raw_row = format!("{}rawRow", context.output_prefix);
    let index = format!("{}index", context.output_prefix);
    let cleanup = format!("{}cleanupError", context.output_prefix);
    let record_row_failure = kotlin::record_cleanup_failure(&ctx, &cleanup);
    let record_outer_failure = kotlin::record_cleanup_failure(context, &cleanup);
    let current_failure = context.cleanup_failure.unwrap_or(context.failure);
    let run_action = if let Some(target) = context.cleanup_failure {
        format!("{target} = {action}({target})")
    } else {
        format!("{action}({current_failure})")
    };
    let finish_row = context
        .cleanup_failure
        .map(|target| format!("{target} = {failure}"))
        .unwrap_or_default();
    Ok(KotlinOutput {
        result_names: vec![None],
        delivered: vec![format!("requireNotNull({output})")],
        result_types: vec![format!("kotlin.collections.List<{ty}>")],
        output_locals: format!("var {output}: kotlin.collections.MutableList<{ty}>? = null\nvar {actions}: kotlin.collections.MutableList<(kotlin.Throwable?) -> kotlin.Throwable?>? = null\n"),
        materialize: format!(
            "{actions} = kotlin.collections.mutableListOf()\n{output} = kotlin.collections.mutableListOf()\nfor ({raw_row} in {raw}) {{\nval {row} = {raw_row} as kotlin.Array<*>\n{}\n{actions}.add {{ {initial} -> var {failure} = {initial}; try {{ {} }} catch ({cleanup}: Throwable) {{ {record_row_failure} }}; {failure} }}\n{}\n{output}.add({value})\n}}\n",
            child.output_locals, child.cleanup, child.materialize
        ),
        cleanup: format!(
            "try {{\nfor ({action} in {actions}.orEmpty()) {{ try {{ {run_action} }} catch ({cleanup}: Throwable) {{ {record_outer_failure} }} }}\nfor ({index} in ({actions}?.size ?: 0) until {raw}.size) {{\nvar {failure}: kotlin.Throwable? = {current_failure}\ntry {{ val {row} = {raw}[{index}] as kotlin.Array<*>\n{}\n{}\n}} catch ({cleanup}: Throwable) {{ {record_row_failure} }}\n{finish_row}\n}}\n}} catch ({cleanup}: Throwable) {{ {record_outer_failure} }}\n",
            child.output_locals, child.cleanup
        ),
    })
}

/// The adapter selects its native scope before looking for a sequence.
#[allow(clippy::type_complexity)]
pub(crate) fn return_sequence(
    layout: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
) -> Option<(
    &prebindgen_flat::Wire<JniWire>,
    &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    &JniSequence,
)> {
    outer_sequences(layout)
        .into_iter()
        .next()
        .and_then(|(element, controls, metadata)| match controls {
            [carrier] => Some((carrier, element, metadata)),
            _ => None,
        })
}
pub(crate) fn callback_target(
    layout: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
) -> Result<syn::Ident> {
    if outer_sequences(layout).len() != 1 || crate::group::result_packet_fields(layout).len() > 2 {
        return Err(
            "callback_return permits only one sequence and its optional presence flag; cannot discard additional return values".into(),
        );
    }
    return_sequence(layout)
        .map(|(carrier, _, _)| carrier.name.clone())
        .ok_or_else(|| "callback_return requires a sequence boundary".into())
}

/// Whether the successful return uses the function's sequence callback.
pub(crate) fn callback_delivery(
    node: &prebindgen_flat::WireNode<JniWire, JniChain, JniBranch>,
) -> Result<bool> {
    let sequences = outer_sequences(node);
    if !sequences
        .iter()
        .any(|(_, _, metadata)| **metadata == JniSequence::Callback)
    {
        return Ok(false);
    }
    callback_target(node)?;
    if crate::group::result_packet_fields(node).len() != 1 {
        return Err("callback_return cannot discard additional return values".into());
    }
    Ok(true)
}

/// Function sequence callbacks are not available in parameters, error channels or rows.
pub(crate) fn validate_buffered(
    node: &prebindgen_flat::WireNode<JniWire, JniChain, JniBranch>,
) -> Result<()> {
    match node {
        prebindgen_flat::WireNode::Leaf(_) => {}
        prebindgen_flat::WireNode::Chain {
            child: element,
            meta: JniChain::Sequence(metadata),
            ..
        } => {
            if *metadata == JniSequence::Callback {
                return Err(
                    "callback sequence is only supported as the function's successful return"
                        .into(),
                );
            }
            validate_buffered(element)?;
        }
        prebindgen_flat::WireNode::Chain {
            meta: JniChain::Optional(_) | JniChain::Bulk(_),
            child: element,
            ..
        } => validate_buffered(element)?,
        prebindgen_flat::WireNode::Branch {
            meta: JniBranch::Group(_),
            children: elements,
            ..
        }
        | prebindgen_flat::WireNode::Branch {
            meta: JniBranch::Alternatives,
            children: elements,
            ..
        } => {
            for element in elements {
                validate_buffered(element)?;
            }
        }
    }
    Ok(())
}

// A sequence owns a separate row scope: find the outer occurrences only.
#[allow(clippy::type_complexity)] // Borrow the element schema, controls and delivery policy together.
fn outer_sequences(
    node: &prebindgen_flat::WireNode<JniWire, JniChain, JniBranch>,
) -> Vec<(
    &prebindgen_flat::WireNode<JniWire, JniChain, JniBranch>,
    &[prebindgen_flat::Wire<JniWire>],
    &JniSequence,
)> {
    fn visit<'a>(
        node: &'a prebindgen_flat::WireNode<JniWire, JniChain, JniBranch>,
        out: &mut Vec<(
            &'a prebindgen_flat::WireNode<JniWire, JniChain, JniBranch>,
            &'a [prebindgen_flat::Wire<JniWire>],
            &'a JniSequence,
        )>,
    ) {
        use prebindgen_flat::WireNode;
        match node {
            WireNode::Chain {
                meta: JniChain::Sequence(meta),
                child,
                controls,
            } => out.push((child, controls, meta)),
            WireNode::Chain { child, .. } => visit(child, out),
            WireNode::Branch { children, .. } => {
                for child in children {
                    visit(child, out)
                }
            }
            WireNode::Leaf(_) => {}
        }
    }
    let mut out = vec![];
    visit(node, &mut out);
    out
}

#[cfg(test)]
mod delivery_tests {
    use super::*;
    use prebindgen_flat::WireNode;
    fn leaf() -> prebindgen_flat::WireNode<JniWire, JniChain, JniBranch> {
        WireNode::Leaf(vec![prebindgen_flat::Wire {
            name: syn::parse_quote!(element),
            kind: JniWire::scalar(Primitive::I32),
        }])
    }
    fn sequence(
        delivery: JniSequence,
        name: &str,
        element: prebindgen_flat::WireNode<JniWire, JniChain, JniBranch>,
    ) -> prebindgen_flat::WireNode<JniWire, JniChain, JniBranch> {
        let kind = match delivery {
            JniSequence::Bulk => JniWire::NativeRows {
                condition: "true".into(),
            },
            JniSequence::Buffered | JniSequence::Callback => JniWire::Rows {
                condition: "true".into(),
                null_niche_available: true,
            },
        };
        WireNode::Chain {
            controls: vec![prebindgen_flat::Wire {
                name: syn::parse_str(name).unwrap(),
                kind,
            }],
            child: Box::new(element),
            meta: JniChain::Sequence(delivery),
        }
    }
    #[test]
    fn delivery_is_explicit_and_callback_scope_is_checked() {
        let visitor = sequence(JniSequence::Callback, "visitor", leaf());
        assert!(callback_delivery(&visitor).unwrap());
        assert!(validate_buffered(&visitor)
            .unwrap_err()
            .contains("successful return"));
        let packed = sequence(JniSequence::Buffered, "rows", leaf());
        assert!(!callback_delivery(&packed).unwrap());
        validate_buffered(&packed).unwrap();
        let nested = sequence(JniSequence::Buffered, "rows", visitor.clone());
        assert!(crate::group::validate_layout(&nested)
            .unwrap_err()
            .contains("successful return"));
        let extra = WireNode::Branch {
            controls: vec![],
            children: vec![visitor, leaf()],
            meta: JniBranch::Group(JniGroup::Values),
        };
        assert!(callback_delivery(&extra)
            .unwrap_err()
            .contains("additional return"));
    }
}
