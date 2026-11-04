//! Per-element native delivery and Kotlin folds, without invoking user code under JNI locks.
use super::*;
use prebindgen_declarations::SequenceOutput;

pub(crate) fn transport(
    wire: syn::Ident,
    length: syn::Ident,
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
            // Zero means present, even when no rows arrived. The enclosing
            // option reserves -1; both values use cached boxes in packet mode.
            kind: JniWire::selector(),
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
            // An extra upcall costs more than small-list growth. Keep short
            // captures on the existing JNI path and reserve only larger lists,
            // once before delivery and never inside the row loop.
            if #length >= 64 {
                let capacity=<i32 as ::core::convert::TryFrom<_>>::try_from(#length).map_err(|_| "collection exceeds JVM capacity".to_string())?;
                let (_, reserve)=__PB_SEQUENCE_CAPACITY.get().ok_or_else(|| "collection capacity hint is not initialized".to_string())?;
                unsafe {#env.call_method_unchecked(&__pb_callback,*reserve,::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(::prebindgen_jni_runtime::jni::signature::Primitive::Void),&[::prebindgen_jni_runtime::jni::sys::jvalue{i:capacity}])}.map_err(|e|e.to_string())?;
            }
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
        // Row payloads crossed through run; only scalar presence crosses back.
        finish: quote!(let #wire=0i32;),
        values: vec![(wire.clone(), quote!(#wire))],
        commit: quote!(),
    })
}

/// Retain only raw wire fields while JNI and handle locks are active. A small
/// immutable record avoids primitive boxes and generic row arrays without
/// copying wide rows when the collector grows. Logical projections can share
/// the same physical JVM record type.
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
    let kotlin_types = fields
        .iter()
        .map(|w| w.kind.kotlin_type())
        .collect::<Vec<_>>()
        .join(",");
    let name = names.name("SequenceRow", &label, &[&signature, &kotlin_types])?;
    let fields = fields
        .iter()
        .enumerate()
        .map(|(i, w)| format!("@JvmField val c{i}: {}", w.kind.kotlin_type()))
        .collect::<Vec<_>>()
        .join(", ");
    Ok((
        name.clone(),
        format!("    internal class {name}({fields})\n"),
    ))
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
    let optional = crate::sequence::callback_optional(layout)?;
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
    let fields_read = (0..fields.len())
        .map(|i| format!("__pb_fold_row.c{i}"))
        .collect::<Vec<_>>();
    let mut unread = kotlin::output_kotlin(element, &ctx)?;
    returns::remap(&mut unread, row, &fields, 0, &fields_read);
    let shared = nested_owners(element);
    let owners = "__pb_foldOwners";
    let child = if shared {
        let mut child = kotlin::output_kotlin_scoped(element, &ctx, Some(owners))?;
        returns::remap(&mut child, row, &fields, 0, &fields_read);
        child
    } else {
        unread.clone()
    };
    let present = format!("{}[{position}] as Int == 0", context.result);
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
        discard = format!("fun __pb_discard({failure}: Throwable, start: Int = 0) {{\nfor (__pb_rowIndex in start until {raw}.size) {{\ntry {{\nval __pb_fold_row = {raw}[__pb_rowIndex]\n{}\n{}\n}} catch (cleanup: Throwable) {{ if ({failure} !== cleanup) {failure}.addSuppressed(cleanup) }}\n}}\n}}\n", unread.output_locals, unread.cleanup);
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
            materialize: format!("if ({present}) {{\n{initialize}var current = initial\nfor (__pb_rowIndex in 0 until {raw}.size) {{\nval __pb_fold_row = {raw}[__pb_rowIndex]\n{}\n{row}}}\n__pb_accumulator = current\n}}\n", child.output_locals),
            cleanup,
        },
    })
}

/// One reservation method shared by every generated collector. Register the
/// actual interface class once, avoiding application-class lookups on workers
/// and method-name/signature allocations on each large return.
pub(crate) fn emit_capacity(
    prefix: &str,
    rust: &mut TokenStream,
    native: &mut String,
    names: &mut names::Names,
) -> Result<()> {
    let method = names.name("init", "SequenceCapacity", &["collection capacity hint"])?;
    let symbol = symbol(prefix, &method);
    rust.extend(quote! {
        static __PB_SEQUENCE_CAPACITY: ::std::sync::OnceLock<(
            ::prebindgen_jni_runtime::jni::objects::GlobalRef,
            ::prebindgen_jni_runtime::jni::objects::JMethodID,
        )> = ::std::sync::OnceLock::new();
        #[no_mangle]
        pub extern "system" fn #symbol(mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>, _class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>, capacity: ::prebindgen_jni_runtime::jni::objects::JClass<'_>) {
            if let Err(error)=::prebindgen_runtime::boundary(|| {
                if __PB_SEQUENCE_CAPACITY.get().is_none() {
                    let reserve=env.get_method_id(&capacity,"reserve","(I)V").map_err(|e|e.to_string())?;
                    let class=env.new_global_ref(&capacity).map_err(|e|e.to_string())?;
                    let _=__PB_SEQUENCE_CAPACITY.set((class,reserve));
                }
                let (class,_)=__PB_SEQUENCE_CAPACITY.get().expect("capacity method initialized");
                if !env.is_same_object(class.as_obj(),&capacity).map_err(|e|e.to_string())? {
                    return Err("collection capacity interface registered with a different class".into());
                }
                Ok(())
            }) { ::prebindgen_jni_runtime::fail(&mut env,&error); }
        }
    });
    native.push_str(&format!("    internal interface SequenceCapacity {{ fun reserve(capacity: Int) }}\n    @JvmStatic private external fun {method}(capacity: Class<*>)\n    init {{ {method}(SequenceCapacity::class.java) }}\n"));
    Ok(())
}
