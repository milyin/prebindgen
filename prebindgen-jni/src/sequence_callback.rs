//! Per-element native delivery and Kotlin folds, without invoking user code under JNI locks.
use super::*;
use prebindgen_declarations::SequenceOutput;

pub(crate) fn transport(
    wire: syn::Ident,
    element: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    values: &[(syn::Ident, TokenStream)],
) -> Result<SequenceOutput<JniWire, crate::JniChain>> {
    let env = prebindgen_conversions::template_parameter("environment");
    let method = format_ident!("{wire}_callback_method");
    let class = format_ident!("{wire}_callback_class");
    let frame = format_ident!("{wire}_callback_frame");
    let mut descriptor = String::from("(");
    let mut encode = TokenStream::new();
    let mut args = Vec::new();
    let mut needs_frame = false;
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
        // This is the same classification used by output_argument to allocate
        // a JVM object. Primitive wire values, including handles, create none.
        needs_frame |= field.kind.is_reference();
        encode.extend(argument.statements);
        args.push(argument.value);
    }
    descriptor.push_str(")V");
    let (enter, leave) = if needs_frame {
        let capacity = i32::try_from(crate::group::callback_parameters(element).len() + 8)
            .map_err(|_| "too many callback fields")?;
        (
            quote!(let #frame=unsafe{::prebindgen_jni_runtime::LocalFrame::new(&#env,#capacity)}?;),
            quote!(drop(#frame);),
        )
    } else {
        (TokenStream::new(), TokenStream::new())
    };
    let commit_element = prebindgen_conversions::template_parameter("commit_element");
    Ok(SequenceOutput {
        controls: vec![prebindgen_flat::Wire {
            name: wire.clone(),
            kind: JniWire::Rows {
                condition: ("true").into(),
                null_niche_available: true,
            },
        }],
        commit_each: true,
        fallible: true,
        metadata: JniChain::Sequence(JniSequence::Callback),
        begin: quote! {
            // Every accepted row belongs to the Kotlin collector, including
            // when a later row fails. Publish this before any row can arrive.
            #env.set_boolean_array_region(&__pb_output_accepted,0,&[1]).map_err(|e|e.to_string())?;
            let #class=#env.get_object_class(&__pb_callback).map_err(|e|e.to_string())?;
            let #method=#env.get_method_id(&#class,"run",#descriptor).map_err(|e|e.to_string())?;
            #env.delete_local_ref(#class).map_err(|e|e.to_string())?;
        },
        enter,
        store: quote! {
            #encode
            // The method was resolved on this callback object with exactly these
            // JVM wire types. The generated bridge only retains raw fields.
            unsafe {#env.call_method_unchecked(&__pb_callback,#method,::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(::prebindgen_jni_runtime::jni::signature::Primitive::Void),&[#(#args),*])}.map_err(|e|e.to_string())?;
            #commit_element
            #leave
        },
        // Only a presence marker crosses back; row payloads crossed through run.
        finish: quote!(let #wire=Some(::prebindgen_jni_runtime::array(&mut #env,0)?);),
        values: vec![(wire.clone(), quote!(#wire))],
        commit: quote!(),
    })
}

/// Capture only raw wire values while JNI and handle locks are active. Columns
/// use the physical JVM types, so neither primitive boxes nor row arrays are
/// needed. The same storage shape can serve different logical projections.
pub(crate) fn capture(
    element: &kotlin::JniNode,
    names: &mut names::Names,
) -> Result<(String, String)> {
    let fields = crate::group::callback_parameters(element).collect::<Vec<_>>();
    let signature = fields
        .iter()
        .map(|w| w.kind.jvm_descriptor())
        .collect::<String>();
    let label = fields
        .iter()
        .take(3)
        .map(|w| w.kind.kotlin_type().trim_end_matches('?'))
        .collect::<Vec<_>>()
        .join("_");
    let label = if fields.is_empty() {
        "Empty".into()
    } else {
        format!("{label}_{}", fields.len())
    };
    let name = names.name("SequenceRows", &label, &[&signature])?;
    let mut columns = String::new();
    let mut allocate = String::new();
    let mut store = String::new();
    let mut empty = BTreeMap::new();
    let mut parameters = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        let ty = field.kind.kotlin_type();
        let (array, initializer) = field.kind.kotlin_column();
        let empty_name = format!("empty{}", empty.len());
        let empty_name = empty
            .entry(array.clone())
            .or_insert((empty_name, initializer));
        columns.push_str(&format!(
            "    var c{i}: {array} = {}\n        private set\n",
            empty_name.0
        ));
        parameters.push(format!("v{i}: {ty}"));
        allocate.push_str(&format!("        c{i} = c{i}.copyOf(next)\n"));
        store.push_str(&format!("        c{i}[size] = v{i}\n"));
    }
    let empty = if empty.is_empty() {
        String::new()
    } else {
        format!(
            "    companion object {{\n{}    }}\n",
            empty
                .values()
                .map(|(name, initializer)| format!("        private val {name} = {initializer}\n"))
                .collect::<String>()
        )
    };
    // Replacing a column preserves its accepted prefix, even if a later
    // allocation fails. Publish capacity only after every column has grown,
    // and size only after all fields are stored. Rust owns any rejected row.
    let declaration = format!("    internal class {name} {{\n    var size: Int = 0\n        private set\n    private var capacity: Int = 0\n{columns}    fun add({}) {{\n        if (size == capacity) grow()\n{store}        size++\n    }}\n    private fun grow() {{\n        if (size == Int.MAX_VALUE) throw OutOfMemoryError(\"too many callback rows\")\n        val next = if (capacity == 0) 1 else minOf(capacity.toLong() * 2, Int.MAX_VALUE.toLong()).toInt()\n{allocate}        capacity = next\n    }}\n{empty}}}\n", parameters.join(", "));
    Ok((name, declaration))
}

pub(crate) struct Fold {
    pub output: KotlinOutput,
    pub parameter: String,
    pub discard: String,
}
pub(crate) fn fold(
    layout: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    context: &KotlinContext<'_>,
) -> Result<Fold> {
    fn optional(
        node: &prebindgen_flat::WireNode<JniWire, crate::JniChain, crate::JniBranch>,
    ) -> bool {
        match node {
            prebindgen_flat::WireNode::Chain {
                meta: JniChain::Optional(_),
                ..
            } => true,
            prebindgen_flat::WireNode::Branch {
                meta: JniBranch::Group(_),
                children: elements,
                ..
            } => elements.iter().any(optional),
            _ => false,
        }
    }
    let optional = optional(layout);
    let (carrier, element, _) =
        crate::sequence::return_sequence(layout).ok_or("missing sequence callback layout")?;
    let position = crate::group::result_packet_fields(layout)
        .position(|w| w.name == carrier.name)
        .unwrap();
    let row = "__pb_fold_row";
    let ctx = KotlinContext {
        result: row,
        output_prefix: "foldElement",
        cleanup_failure: None,
        ..*context
    };
    // Lexical row cleanup keeps partially constructed wrappers in scope. Only
    // dynamic nested collections need the shared owner journal; fixed fields
    // can use their existing locals without allocating a journal or a lambda.
    fn nested_owners(node: &kotlin::JniNode) -> bool {
        match node {
            prebindgen_flat::WireNode::Chain {
                meta: JniChain::Sequence(_),
                child,
                ..
            } => kotlin::input_handles(child, false),
            prebindgen_flat::WireNode::Chain { child, .. } => nested_owners(child),
            prebindgen_flat::WireNode::Branch { children, .. } => {
                children.iter().any(nested_owners)
            }
            _ => false,
        }
    }
    let fields = crate::group::callback_parameters(element).collect::<Vec<_>>();
    let columns = (0..fields.len())
        .map(|i| format!("__pb_rows.c{i}[__pb_rowIndex]"))
        .collect::<Vec<_>>();
    let mut unread = kotlin::output_kotlin(element, &ctx)?;
    returns::remap(&mut unread, row, &fields, 0, &columns);
    let shared = nested_owners(element);
    let owners = "__pb_foldOwners";
    let child = if shared {
        let mut child = kotlin::output_kotlin_scoped(element, &ctx, Some(owners))?;
        returns::remap(&mut child, row, &fields, 0, &columns);
        child
    } else {
        unread.clone()
    };
    let present = format!("{}[{position}] != null", context.result);
    let raw = "__pb_rows";
    let types = child.callback_type_parameters()?;
    let args = child.delivered.join(", ");
    let failure = context.failure;
    let resources = !unread.cleanup.is_empty();
    let mut locals = String::from("var __pb_accumulator: A? = null\n");
    let mut initialize = String::new();
    let mut row = format!("{}\ncurrent = fold(current, {args})\n", child.materialize);
    let mut discard = String::new();
    let mut cleanup = String::new();
    if resources {
        locals.push_str("var __pb_processed = 0\n");
        let release = if shared {
            locals.push_str(&format!(
                "var {owners}: {}.CollectionOwners? = null\n",
                context.package
            ));
            initialize = format!(
                "if ({raw}.size != 0) {owners} = {}.CollectionOwners()\n",
                context.package
            );
            row = format!(
                "{}\n{owners}!!.complete = true\ncurrent = fold(current, {args})\n",
                child.materialize
            );
            format!(
                "{owners}?.close({failure})\nif ({owners}?.complete != true) {{\n{}\n}}",
                child.cleanup
            )
        } else {
            child.cleanup.clone()
        };
        let delivered = if shared {
            format!("{owners}!!.release()\n")
        } else {
            String::new()
        };
        // Account for the current row before cleanup, so a secondary cleanup
        // failure cannot make the outer failure scan destroy it a second time.
        row = format!("try {{\n{row}}} catch ({failure}: Throwable) {{\n__pb_processed++\ntry {{\n{release}\n}} catch (cleanup: Throwable) {{ if ({failure} !== cleanup) {failure}.addSuppressed(cleanup) }}\nthrow {failure}\n}}\n__pb_processed++\n{delivered}");
        discard = format!("fun __pb_discard({failure}: Throwable, start: Int = 0) {{\nfor (__pb_rowIndex in start until {raw}.size) {{\ntry {{\n{}\n{}\n}} catch (cleanup: Throwable) {{ if ({failure} !== cleanup) {failure}.addSuppressed(cleanup) }}\n}}\n}}\n", unread.output_locals, unread.cleanup);
        cleanup = format!("if ({present}) __pb_discard({failure}, __pb_processed)\n");
    }
    Ok(Fold {
        parameter: format!("fold: (A, {types}) -> A"),
        discard,
        output: KotlinOutput {
            result_names: vec![None],
            delivered: vec![if optional { "__pb_accumulator".into() } else { "__pb_accumulator as A".into() }],
            result_types: vec![if optional { "A?".into() } else { "A".into() }],
            output_locals: locals,
            materialize: format!("if ({present}) {{\n{initialize}var current = initial\nfor (__pb_rowIndex in 0 until {raw}.size) {{\n{}\n{row}}}\n__pb_accumulator = current\n}}\n", child.output_locals),
            cleanup,
        },
    })
}
