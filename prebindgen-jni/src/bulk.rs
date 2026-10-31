//! C-representation packing of existing expansion trees.
use super::*;
use crate::kotlin::{JniNode, KotlinBinding, KotlinContext, KotlinInput, KotlinOutput};
use prebindgen_declarations::{Condition, SequenceInput, SequenceOutput};
use prebindgen_flat::{InputConversion, Wire, WireNode};
use std::rc::Rc;

/// Physical record shared by the Rust encoder and the initialized Kotlin layout table.
#[derive(Clone, Debug)]
pub struct BulkRecord {
    pub(crate) name: syn::Ident,
    pub(crate) definition: prebindgen_reprc::Record,
    pub(crate) key: String,
    widths: Vec<usize>,
    active: Condition,
}
fn field_type(w: &JniWire) -> Result<syn::Type> {
    Ok(match w {
        JniWire::NativeBuffer{..}|JniWire::NativeRows{..}=>syn::parse_quote!(::prebindgen_jni_runtime::bulk::Range),
        JniWire::Native128{..}=>syn::parse_quote!([u8;16]),
        JniWire::Scalar{primitive:Primitive::I128|Primitive::U128,..}|JniWire::Text{..}|JniWire::Bytes{..}|JniWire::Rows{..}|JniWire::BulkBuffer{..}|JniWire::Callback{..}=>return Err("bulk fields require native scalar/buffer/row encodings; callback objects cannot be packed".into()),
        _=>w.rust_type(),
    })
}
fn record(node: &JniNode) -> Result<Rc<BulkRecord>> {
    let fields = crate::group::row_fields(node)
        .enumerate()
        .map(|(i, w)| Ok((format_ident!("v{i}"), field_type(&w.kind)?)))
        .collect::<Result<Vec<_>>>()?;
    let mut widths = crate::group::row_fields(node)
        .map(|wire| match wire.kind {
            JniWire::NativeBuffer { .. }
            | JniWire::NativeRows { .. }
            | JniWire::Native128 { .. } => Ok(16),
            _ => wire.kind.bulk_scalar_access().map(|(_, width)| width),
        })
        .collect::<Result<Vec<_>>>()?;
    let empty = fields.is_empty();
    if empty {
        widths.push(1);
    }
    let fields = if empty {
        vec![(syn::parse_quote!(unused), syn::parse_quote!(u8))]
    } else {
        fields
    };
    let key = if empty { "empty".into() } else { schema(node)? };
    let label = fields
        .iter()
        .take(2)
        .map(|(_, ty)| crate::names::type_label(ty))
        .collect::<Vec<_>>()
        .join("_");
    let label = label.trim_start_matches("r#");
    let name = format_ident!(
        "__PbBulk_{}_{:016x}",
        label,
        prebindgen_reprc::schema_id(&key)
    );
    let definition = prebindgen_reprc::Record::new(name.clone(), fields)?;
    Ok(Rc::new(BulkRecord {
        name,
        definition,
        key,
        widths,
        active: Condition::default(),
    }))
}
fn parameter(name: &str) -> TokenStream {
    prebindgen_conversions::template_parameter(name)
}
fn resolve(code: &TokenStream, key: &str, value: &TokenStream) -> Result<TokenStream> {
    prebindgen_conversions::render_template(code, &|name| {
        Ok(if name == key {
            value.clone()
        } else {
            parameter(name)
        })
    })
}
fn native_read(
    w: &Wire<JniWire>,
    record: &BulkRecord,
    i: usize,
    base: TokenStream,
) -> Result<TokenStream> {
    let reader = parameter("bulk_reader");
    let row = &record.name;
    let field = format_ident!("v{i}");
    let field_offset = quote!(::core::mem::offset_of!(#row,#field));
    let offset = if base.to_string() == "0usize" {
        field_offset
    } else {
        quote!((#base)+#field_offset)
    };
    Ok(match w.kind {
        JniWire::NativeBuffer { .. } => quote!(#reader.range(#offset)?),
        JniWire::NativeRows { .. } => quote!(#reader.range(#offset)?),
        _ => {
            let ty = field_type(&w.kind)?;
            quote!(#reader.read::<#ty>(#offset)?)
        }
    })
}
fn native_store(
    w: &Wire<JniWire>,
    record: &BulkRecord,
    i: usize,
    base: TokenStream,
    value: TokenStream,
) -> Result<TokenStream> {
    let writer = parameter("bulk_writer");
    let row = &record.name;
    let field = format_ident!("v{i}");
    let field_offset = quote!(::core::mem::offset_of!(#row,#field));
    let offset = if base.to_string() == "0usize" {
        field_offset
    } else {
        quote!((#base)+#field_offset)
    };
    Ok(match w.kind {
        JniWire::NativeBuffer { .. } | JniWire::NativeRows { .. } => {
            quote!(#writer.range(#offset,#value)?;)
        }
        _ => {
            let value = w.kind.output_scalar(&value);
            quote!(#writer.write(#offset,#value)?;)
        }
    })
}
pub(crate) fn input(
    wire: syn::Ident,
    mut child: InputPlan<JniWire>,
    active: &Condition,
) -> Result<InputPlan<JniWire>> {
    let mut layout = (*record(&child.node)?).clone();
    layout.active = active.clone();
    let layout = Rc::new(layout);
    let ty = &layout.name;
    let length = format_ident!("{wire}_length");
    let bytes = format_ident!("{wire}_bytes");
    let reader = format_ident!("{wire}_reader");
    let mut bind = TokenStream::new();
    for (i, w) in crate::group::row_fields(&child.node).enumerate() {
        let name = &w.name;
        let read = native_read(w, &layout, i, quote!(0usize))?;
        let absent = crate::optional::absent(&w.kind, None, true)?;
        let condition = active.rust();
        bind.extend(quote!(let #name=if #condition {#read}else{#absent};));
    }
    let bind = resolve(&bind, "bulk_reader", &quote!(#reader))?;
    let validation = resolve(&child.validation, "bulk_reader", &quote!(#reader))?;
    child.acquire = resolve(&child.acquire, "bulk_reader", &quote!(#reader))?;
    child.statements = resolve(&child.statements, "bulk_reader", &quote!(#reader))?;
    let read = parameter(&format!("bulk_read_{wire}"));
    let condition = active.rust();
    let truncated = active.and(quote!(#bytes.len()<::core::mem::size_of::<#ty>()));
    child.validation = quote! {
        let #bytes=if #condition {#read?}else{Vec::new()};
        let #reader=::prebindgen_jni_runtime::bulk::Reader::new(&#bytes);
        if #truncated{return Err("truncated bulk root record".into());}
        #bind #validation
    };
    child.node = WireNode::Chain {
        meta: JniChain::Bulk(layout),
        child: Box::new(child.node),
        controls: vec![
            Wire {
                name: wire,
                kind: JniWire::BulkBuffer {
                    condition: kotlin_condition(active),
                    null_niche_available: false,
                },
            },
            Wire {
                name: length,
                kind: JniWire::scalar(Primitive::I32),
            },
        ],
    };
    Ok(child)
}
pub(crate) fn output(
    wire: syn::Ident,
    mut child: OutputPlan<JniWire>,
) -> Result<OutputPlan<JniWire>> {
    let layout = record(&child.outcomes.success)?;
    let ty = &layout.name;
    let writer = format_ident!("{wire}_writer");
    let prepare = resolve(&child.statements, "bulk_writer", &quote!(#writer))?;
    let values = crate::group::row_values(&child.outcomes.success, &child.values)?;
    let mut store = TokenStream::new();
    for (i, (w, value)) in values.into_iter().enumerate() {
        store.extend(native_store(w, &layout, i, quote!(0usize), value.clone())?);
    }
    let store = resolve(&store, "bulk_writer", &quote!(#writer))?;
    let env = parameter("environment");
    child.statements = quote! {
        let mut #writer=::prebindgen_jni_runtime::bulk::Writer::new(::core::mem::size_of::<#ty>());
        #prepare #store
        let #wire=::prebindgen_jni_runtime::jni::objects::JObject::from(#env.byte_array_from_slice(#writer.bytes()).map_err(|e|e.to_string())?);
    };
    child.outcomes.success = WireNode::Chain {
        meta: JniChain::Bulk(layout),
        child: Box::new(child.outcomes.success),
        controls: vec![Wire {
            name: wire.clone(),
            kind: JniWire::BulkBuffer {
                condition: "true".into(),
                null_niche_available: true,
            },
        }],
    };
    child.values = vec![(wire.clone(), quote!(#wire))];
    child.fallible = true;
    Ok(child)
}
pub(crate) fn sequence_input(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &JniNode,
    active: &Condition,
) -> Result<SequenceInput<JniWire, JniChain>> {
    let layout = record(element)?;
    let row = &layout.name;
    let reader = parameter("bulk_reader");
    let start = format_ident!("{wire}_start");
    let base = quote!(#start+#index*::core::mem::size_of::<#row>());
    let mut bind = TokenStream::new();
    let mut values = vec![];
    for (i, w) in crate::group::row_fields(element).enumerate() {
        let name = &w.name;
        let read = native_read(w, &layout, i, base.clone())?;
        bind.extend(quote!(let #name=#read;));
        values.push((name.clone(), quote!(#name)));
    }
    Ok(SequenceInput {
        controls: vec![Wire {
            name: wire.clone(),
            kind: JniWire::NativeRows {
                condition: kotlin_condition(active),
            },
        }],
        metadata: JniChain::Sequence(JniSequence::Bulk),
        open: quote!(let(#start,#length)=#reader.rows(#wire,::core::mem::size_of::<#row>())?;),
        bind,
        validation_values: values.clone(),
        prepared_values: values.into_iter().map(|(n, v)| (n, Some(v))).collect(),
        release: quote!(),
    })
}
pub(crate) fn sequence_output(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &JniNode,
    values: &[(syn::Ident, TokenStream)],
) -> Result<SequenceOutput<JniWire, JniChain>> {
    let layout = record(element)?;
    let row = &layout.name;
    let writer = parameter("bulk_writer");
    let mut store = TokenStream::new();
    for (i, (w, value)) in crate::group::row_values(element, values)?
        .into_iter()
        .enumerate()
    {
        store.extend(native_store(
            w,
            &layout,
            i,
            quote!(#wire.offset as usize+#index*::core::mem::size_of::<#row>()),
            value.clone(),
        )?);
    }
    Ok(SequenceOutput {
        controls: vec![Wire {
            name: wire.clone(),
            kind: JniWire::NativeRows {
                condition: "true".into(),
            },
        }],
        metadata: JniChain::Sequence(JniSequence::Bulk),
        commit_each: false,
        fallible: true,
        begin: quote!(let #wire=#writer.rows(#length,::core::mem::size_of::<#row>(),::core::mem::align_of::<#row>())?;),
        enter: quote!(),
        store,
        finish: quote!(let #wire=Some(#wire);),
        values: vec![(wire.clone(), quote!(#wire))],
        commit: quote!(),
    })
}

pub(crate) fn input_leaf(
    flat: &Bindings,
    ty: &syn::Type,
    w: syn::Ident,
    v: syn::Ident,
    active: &Condition,
) -> Result<InputPlan<JniWire>> {
    match Value::of(ty) {
        Some(Value::Text) | Some(Value::Bytes) => {
            let text = Value::of(ty) == Some(Value::Text);
            let reader = parameter("bulk_reader");
            let decode = if text {
                quote!(::core::str::from_utf8(#reader.buffer_range(#w)?.ok_or("null bulk text")?).map_err(|_|"invalid UTF-8 in bulk text")?)
            } else {
                quote!(#reader.buffer_range(#w)?.ok_or("null bulk bytes")?)
            };
            prebindgen_conversions::value::input(
                ty,
                w,
                v,
                &active.rust(),
                decode,
                JniWire::NativeBuffer { text },
            )
        }
        Some(Value::Scalar(p @ (Primitive::I128 | Primitive::U128))) => {
            let native = p.rust_type();
            let decode = quote!(#native::from_be_bytes(#w));
            prebindgen_conversions::value::input(
                ty,
                w,
                v,
                &active.rust(),
                decode,
                JniWire::Native128 {
                    primitive: p,
                    condition: kotlin_condition(active),
                },
            )
        }
        _ => JniCodec.input_leaf(flat, ty, &(), w, v, active),
    }
}
pub(crate) fn output_leaf(
    flat: &Bindings,
    ty: &syn::Type,
    w: syn::Ident,
    value: TokenStream,
    borrowed: bool,
) -> Result<OutputPlan<JniWire>> {
    if let Some(kind @ (Value::Text | Value::Bytes)) = Value::of(ty) {
        let owner = format_ident!("{w}_source");
        let text = kind == Value::Text;
        let bytes = if text {
            quote!(#owner.as_bytes())
        } else {
            quote!(#owner.as_ref())
        };
        let writer = parameter("bulk_writer");
        return Ok(OutputPlan {
            fallible: true,
            context: Default::default(),
            delivery: Default::default(),
            outcomes: prebindgen_flat::Outcomes {
                success: WireNode::Leaf(vec![Wire {
                    name: w.clone(),
                    kind: JniWire::NativeBuffer { text },
                }]),
                source_error: None,
                binding_failure: None,
            },
            statements: quote!(let #owner=#value;let #w=Some(#writer.append_buffer(#bytes)?);),
            values: vec![(w.clone(), quote!(#w))],
            commit: quote!(),
        });
    }
    if let Some(Value::Scalar(p @ (Primitive::I128 | Primitive::U128))) = Value::of(ty) {
        let source = match ty {
            syn::Type::Reference(r) => &*r.elem,
            _ => ty,
        };
        return prebindgen_conversions::value::output(
            source,
            w,
            value,
            borrowed,
            |v| quote!((#v).to_be_bytes()),
            JniWire::Native128 {
                primitive: p,
                condition: "true".into(),
            },
        );
    }
    JniCodec.output_leaf(flat, ty, &(), w, value, borrowed)
}
fn kt_layout(record: &BulkRecord, context: &KotlinContext<'_>) -> String {
    format!(
        "{}.BulkLayouts.r{:016x}",
        context.package,
        prebindgen_reprc::schema_id(&record.key)
    )
}
fn sequence_child<'a>(node: &'a JniNode, name: &syn::Ident) -> Option<&'a JniNode> {
    match node {
        WireNode::Chain {
            meta: JniChain::Sequence(_),
            controls,
            child,
        } if controls.iter().any(|w| &w.name == name) => Some(child),
        WireNode::Chain { child, .. } => sequence_child(child, name),
        WireNode::Branch { children, .. } => children.iter().find_map(|c| sequence_child(c, name)),
        _ => None,
    }
}
fn pack(
    node: &JniNode,
    args: &[String],
    writer: &str,
    base: &str,
    context: &KotlinContext<'_>,
) -> Result<String> {
    let layout = record(node)?;
    let layout = kt_layout(&layout, context);
    let wires = crate::group::row_fields(node).collect::<Vec<_>>();
    if wires.len() != args.len() {
        return Err("bulk Kotlin input fields differ from the record schema".into());
    }
    let mut code = String::new();
    for (i, (w, arg)) in wires.iter().zip(args).enumerate() {
        let arg = format!("{}.InputValidation.value {{ {arg} }}", context.package);
        let at = format!("({base}) + {layout}.offsets[{i}]");
        match &w.kind {
            JniWire::NativeBuffer { text } => code.push_str(&format!(
                "{writer}.{}({at}, {arg})\n",
                if *text { "text" } else { "bytes" }
            )),
            JniWire::Native128 { primitive, .. } => code.push_str(&format!(
                "{writer}.wide({at}, {arg}, {})\n",
                *primitive == Primitive::I128
            )),
            JniWire::NativeRows { .. } => {
                let child =
                    sequence_child(node, &w.name).ok_or("missing bulk collection element")?;
                let child_layout = record(child)?;
                let child_layout = kt_layout(&child_layout, context);
                let input = row_input(child, context)?;
                if input.parameters.len() != 1 {
                    return Err("bulk collection elements must expose one Kotlin value; declare a data class".into());
                }
                let variable = input.parameters[0].split_once(": ").unwrap().0;
                let list = format!("{}_bulkList", w.name);
                let start = format!("{}_bulkStart", w.name);
                let index = format!("{}_bulkIndex", w.name);
                code.push_str(&format!("val {list} = {arg}\nif ({list} == null) {{ {writer}.absent({at}) }} else {{\nval {start} = {writer}.rows({list}.size, {child_layout})\n{writer}.range({at}, {start}, {list}.size)\n"));
                {
                    code.push_str(&format!(
                        "{list}.forEachIndexed {{ {index}, {variable} ->\n"
                    ));
                    for setup in &input.setup {
                        code.push_str(&format!("val {} = {}\n", setup.name, setup.expression));
                    }
                    code.push_str(&pack(
                        child,
                        &input.prepared_arguments,
                        writer,
                        &format!("{start} + {index} * {child_layout}.size"),
                        context,
                    )?);
                    code.push_str("}\n");
                }
                code.push_str("}\n");
            }
            JniWire::Scalar { primitive, .. } => {
                let (method, _) = w.kind.bulk_scalar_access()?;
                let arg = if *primitive == Primitive::Bool {
                    format!("(if ({arg}) 1 else 0).toByte()")
                } else {
                    arg.clone()
                };
                code.push_str(&format!("{writer}.{method}({at}, {arg})\n"));
            }
            JniWire::Enum { .. } | JniWire::Selector { .. } => {
                code.push_str(&format!("{writer}.int({at}, {arg})\n"))
            }
            JniWire::Handle { handle, .. } => code.push_str(&format!(
                "{writer}.handle({at}, {arg}, {})\n",
                handle.access == Access::Move
            )),
            _ => return Err("unsupported field in bulk Kotlin writer".into()),
        }
    }
    Ok(code)
}
pub(crate) fn input_list(
    carrier: &Wire<JniWire>,
    element: &JniNode,
    context: &KotlinContext<'_>,
) -> Result<KotlinInput> {
    let input = row_input(element, context)?;
    if input.parameters.len() != 1 {
        return Err(
            "bulk collection elements must expose one Kotlin value; declare a data class".into(),
        );
    }
    let (_, ty) = input.parameters[0].split_once(": ").unwrap();
    let name = carrier.name.to_string();
    let state = format!("{name}_elements");
    let JniWire::NativeRows { condition } = &carrier.kind else {
        return Err("bulk sequence requires native rows".into());
    };
    Ok(KotlinInput {
        parameters: vec![format!(
            "{name}: kotlin.collections.List<{ty}>{}",
            if condition == "true" { "" } else { "?" }
        )],
        setup: vec![KotlinBinding {
            name: state.clone(),
            expression: format!("if ({condition}) {name}?.toList() else null"),
            absent: "null".into(),
        }],
        arguments: vec![state.clone()],
        prepared_arguments: vec![state],
        ..Default::default()
    })
}

pub(crate) fn input_kotlin(
    layout: &BulkRecord,
    controls: &[Wire<JniWire>],
    child: &JniNode,
    context: &KotlinContext<'_>,
) -> Result<KotlinInput> {
    let mut input = crate::kotlin::input_kotlin(child, context)?;
    let writer = format!("{}_writer", controls[0].name);
    let schema = kt_layout(layout, context);
    input.prepare_arguments(crate::kotlin::input_handles(child, false))?;
    let body = pack(child, &input.prepared_arguments, &writer, "0", context)?;
    let handles = crate::kotlin::input_handles(child, false);
    let owned = crate::kotlin::input_handles(child, true);
    let condition = match &controls[0].kind {
        JniWire::BulkBuffer { condition, .. } => condition.as_str(),
        _ => return Err("missing bulk input carrier".into()),
    };
    if handles {
        let state = format!("{writer}_state");
        input.setup.push(KotlinBinding {
            name:state.clone(),
            expression:format!("if ({condition}) run {{ val {writer} = {}.BulkHandleWriter({schema}.size)\n{body}\n{writer} }} else null",context.package),
            absent:"null".into(),
        });
        input.arguments = vec![
            format!("{state}?.encode()"),
            format!("{state}?.length ?: 0"),
        ];
        input.prepared_arguments = vec![state.clone(), format!("{state}?.length ?: 0")];
        input.locks.clear();
        input.lock_groups = vec![format!("{state}?.handles.orEmpty()")];
        input.consumed = if owned {
            vec![format!("{state}?.consume(__pb_consume)")]
        } else {
            vec![]
        };
        return Ok(input);
    }
    let state = format!("{writer}_state");
    input.setup.push(KotlinBinding {
        name:state.clone(),
        expression:format!("if ({condition}) run {{ val {writer} = {}.BulkWriter({schema}.size)\n{body}\n{writer} }} else null",context.package),
        absent:"null".into(),
    });
    input.arguments = vec![format!("{state}?.array"), format!("{state}?.length ?: 0")];
    input.prepared_arguments = input.arguments.clone();
    Ok(input)
}
fn unpack_expression(w: &Wire<JniWire>, reader: &str, at: &str) -> Result<String> {
    Ok(match &w.kind {
        JniWire::NativeBuffer { text } => {
            format!("{reader}.{}({at})", if *text { "text" } else { "bytes" })
        }
        JniWire::Native128 { .. } => format!("{reader}.wide({at})"),
        JniWire::NativeRows { .. } => format!("{reader}.range({at})"),
        JniWire::Handle { .. } => format!("{reader}.long({at})"),
        JniWire::Enum { .. } | JniWire::Selector { .. } => format!("{reader}.int({at})"),
        JniWire::Scalar { primitive, .. } => {
            let (method, _) = w.kind.bulk_scalar_access()?;
            let value = format!("{reader}.{method}({at})");
            if *primitive == Primitive::Bool {
                format!("({value}.toInt() != 0)")
            } else {
                value
            }
        }
        _ => return Err("unsupported bulk Kotlin reader field".into()),
    })
}
fn remap(
    output: &mut KotlinOutput,
    node: &JniNode,
    token: &str,
    reader: &str,
    base: &str,
    context: &KotlinContext<'_>,
) -> Result<()> {
    let layout = record(node)?;
    let schema = kt_layout(&layout, context);
    for (i, w) in crate::group::row_fields(node).enumerate() {
        let from = format!("{token}[{i}]");
        let expression =
            unpack_expression(w, reader, &format!("({base}) + {schema}.offsets[{i}]"))?;
        for text in output.delivered.iter_mut().chain([
            &mut output.output_locals,
            &mut output.materialize,
            &mut output.cleanup,
        ]) {
            *text = text.replace(&from, &format!("({expression})"));
        }
    }
    Ok(())
}
pub(crate) fn output_kotlin(
    _layout: &BulkRecord,
    child: &JniNode,
    context: &KotlinContext<'_>,
) -> Result<KotlinOutput> {
    let token = format!("{}bulkRow", context.output_prefix);
    let reader_state = format!("{}bulkReader", context.output_prefix);
    let reader = format!("requireNotNull({reader_state})");
    let ctx = KotlinContext {
        result: &token,
        ..*context
    };
    let mut output = crate::kotlin::output_kotlin(child, &ctx)?;
    remap(&mut output, child, &token, &reader, "0", context)?;
    for text in output.delivered.iter_mut().chain([
        &mut output.output_locals,
        &mut output.materialize,
        &mut output.cleanup,
    ]) {
        *text = text.replace("__PB_BULK_READER__", &reader);
    }
    let initialize = format!(
        "{reader_state} = {}.BulkReader({}[0] as ByteArray)\n",
        context.package, context.result
    );
    output.output_locals = format!(
        "var {reader_state}: {}.BulkReader? = null\n{}",
        context.package, output.output_locals
    );
    output.materialize = format!("{initialize}{}", output.materialize);
    if !output.cleanup.is_empty() {
        output.cleanup = format!(
            "if ({reader_state} == null) {{ {initialize} }}\n{}",
            output.cleanup
        );
    }

    Ok(output)
}
fn owned_fields(node: &JniNode) -> bool {
    prebindgen_flat::wire_definitions(std::iter::once(node))
        .any(|w| matches!(w.kind, JniWire::Handle { .. }))
}
pub(crate) fn output_list(element: &JniNode, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
    let prefix = context.output_prefix;
    let row = format!("{prefix}bulkRow");
    let base = format!("{prefix}bulkBase");
    let index = format!("{prefix}bulkIndex");
    let range = format!("{prefix}bulkRange");
    let list = format!("{prefix}bulkList");
    let actions = format!("{prefix}bulkActions");
    let failure = format!("{prefix}bulkFailure");
    let reader = "__PB_BULK_READER__";
    let layout = record(element)?;
    let schema = kt_layout(&layout, context);
    let child_context = KotlinContext {
        result: &row,
        output_prefix: &format!("{prefix}bulkItem_"),
        failure: &failure,
        cleanup_failure: Some(&failure),
        ..*context
    };
    let mut child = row_output(element, &child_context)?;
    if child.delivered.len() != 1 {
        return Err(
            "bulk collection outputs must expose one Kotlin value; declare a data class".into(),
        );
    }
    remap(&mut child, element, &row, reader, &base, context)?;
    let ty = &child.result_types[0];
    let value = &child.delivered[0];
    let resources = owned_fields(element);
    let raw = format!("({}[0] as {}.BulkRange?)", context.result, context.package);
    let mut locals=format!("var {range}: {}.BulkRange? = null\nvar {list}: kotlin.collections.MutableList<{ty}>? = null\n",context.package);
    if resources {
        locals.push_str(&format!(
            "var {actions}: kotlin.collections.MutableList<(Throwable?) -> Throwable?>? = null\n"
        ));
    }
    let mut materialize=format!("{range} = {reader}.rows({raw}, {schema}.size)\n{list} = kotlin.collections.ArrayList({range}!!.length)\n");
    if resources {
        materialize.push_str(&format!("{actions} = kotlin.collections.mutableListOf()\n"));
    }
    materialize.push_str(&format!("for ({index} in 0 until {range}!!.length) {{\nval {base} = {range}!!.offset + {index} * {schema}.size\n{}\n",child.output_locals));
    let cleanup_error = format!("{prefix}bulkCleanupError");
    let record_row_failure = crate::kotlin::record_cleanup_failure(&child_context, &cleanup_error);
    let record_outer_failure = crate::kotlin::record_cleanup_failure(context, &cleanup_error);
    if resources {
        materialize.push_str(&format!(
            "{actions}!!.add {{ initial -> var {failure} = initial; try {{ {} }} catch ({cleanup_error}: Throwable) {{ {record_row_failure} }}; {failure} }}\n",
            child.cleanup
        ));
    }
    materialize.push_str(&format!(
        "{}\n{list}!!.add({value})\n}}\n",
        child.materialize
    ));
    let cleanup = if resources {
        let initial = context.cleanup_failure.unwrap_or(context.failure);
        let assignment = context
            .cleanup_failure
            .map(|f| format!("{f} = "))
            .unwrap_or_default();
        format!("try {{ if ({range} == null) {range} = {reader}.rows({raw}, {schema}.size)\nfor (action in {actions}.orEmpty()) {{ try {{ {assignment}action({initial}) }} catch ({cleanup_error}: Throwable) {{ {record_outer_failure} }} }}\nfor ({index} in ({actions}?.size ?: 0) until {range}!!.length) {{\nval {base} = {range}!!.offset + {index} * {schema}.size\nvar {failure}: Throwable? = {initial}\ntry {{ {}\n{} }} catch ({cleanup_error}: Throwable) {{ {record_row_failure} }}\n{}\n}} }} catch ({cleanup_error}: Throwable) {{ {record_outer_failure} }}\n",child.output_locals,child.cleanup,context.cleanup_failure.map(|f|format!("{f} = {failure}")).unwrap_or_default())
    } else {
        String::new()
    };
    Ok(KotlinOutput {
        result_names: vec![None],
        result_types: vec![format!("kotlin.collections.List<{ty}>")],
        delivered: vec![format!("requireNotNull({list})")],
        output_locals: locals,
        materialize,
        cleanup,
    })
}
fn register(record: Rc<BulkRecord>, out: &mut BTreeMap<String, Rc<BulkRecord>>) -> Result<()> {
    let id = format!("{:016x}", prebindgen_reprc::schema_id(&record.key));
    if let Some(previous) = out.get(&id) {
        if previous.key != record.key {
            return Err("bulk schema hash collision".into());
        }
    } else {
        out.insert(id, record);
    }
    Ok(())
}
fn records(node: &JniNode, out: &mut BTreeMap<String, Rc<BulkRecord>>) -> Result<()> {
    match node {
        WireNode::Chain {
            meta: JniChain::Bulk(r),
            child,
            ..
        } => {
            register(r.clone(), out)?;
            records(child, out)?;
        }
        WireNode::Chain {
            meta: JniChain::Sequence(JniSequence::Bulk),
            child,
            ..
        } => {
            let r = record(child)?;
            register(r, out)?;
            records(child, out)?;
        }
        WireNode::Chain { child, .. } => records(child, out)?,
        WireNode::Branch { children, .. } => {
            for child in children {
                records(child, out)?;
            }
        }
        WireNode::Leaf(wires) => {
            for wire in wires {
                if let JniWire::Callback { callback } = &wire.kind {
                    for (_, node) in callback.bulk_nodes() {
                        records(node, out)?;
                    }
                }
            }
        }
    }
    Ok(())
}
/// Emit all target measurements in one bootstrap query, before any Kotlin packing.
pub(crate) fn emit(
    flat: &Bindings,
    rust: &mut TokenStream,
    native: &mut String,
    files: &mut BTreeMap<String, String>,
) -> Result<()> {
    let mut catalogue = BTreeMap::new();
    let mut nodes = Vec::new();
    for f in flat.functions.values() {
        for p in &f.params {
            if let Some(plan) = &p.ty.slot {
                nodes.push(plan.input_node());
            }
        }
        if let Some(plan) = &f.ret.slot {
            let outcomes = plan.output_outcomes();
            nodes.extend(
                std::iter::once(&outcomes.success)
                    .chain(outcomes.source_error.iter())
                    .chain(outcomes.binding_failure.iter()),
            );
        }
        if let Some(error) = f.metadata.as_ref().and_then(|m| m.error.as_deref()) {
            nodes.push(&error.output.output_outcomes().success);
        }
    }
    for input in &flat.metadata.callbacks {
        nodes.push(input.input_node());
    }
    for node in nodes {
        records(node, &mut catalogue)?;
    }
    if catalogue.is_empty() {
        return Ok(());
    }
    let prefix = &flat.metadata.package;
    let schema = catalogue
        .values()
        .map(|r| r.key.as_str())
        .collect::<Vec<_>>()
        .join("|");
    let hash = prebindgen_reprc::schema_id(&schema) as i64;
    let mut measurements = quote!(let mut data=vec![1i64,#hash,if cfg!(target_endian="big"){1}else{0},::core::mem::size_of::<usize>() as i64];);
    let mut kotlin=format!("package {prefix}\n\ninternal object BulkLayouts {{\nprivate val data = JNINative.bulkLayouts()\ninit {{ require(data.size == {} && data[0] == 1L && data[1] == {hash}L && data[2] in 0L..1L && (data[3] == 4L || data[3] == 8L)) {{ \"incompatible bulk layout catalogue\" }} }}\nval byteOrder = if(data[2] == 0L) java.nio.ByteOrder.LITTLE_ENDIAN else java.nio.ByteOrder.BIG_ENDIAN\n",4+catalogue.values().map(|r|r.definition.fields().len()+2).sum::<usize>());
    let mut offset = 4;
    for r in catalogue.values() {
        let definition = r.definition.definition();
        rust.extend(quote!(#[allow(non_camel_case_types,dead_code)] #definition));
        let layout = r.definition.layout();
        measurements.extend(quote!(data.extend((#layout).into_iter().map(|v|v as i64));));
        let widths = r
            .widths
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let count = r.definition.fields().len();
        let id = prebindgen_reprc::schema_id(&r.key);
        let end = offset + 2 + count;
        kotlin.push_str(&format!(
            "val r{id:016x} = read({offset}, intArrayOf({widths}))\n"
        ));
        offset = end;
    }
    kotlin.push_str("private fun read(at: Int, widths: IntArray): BulkLayout {\nval size = data[at]; val alignment = data[at + 1]\nrequire(size > 0 && size <= Int.MAX_VALUE && alignment > 0 && alignment <= Int.MAX_VALUE && (alignment and (alignment - 1)) == 0L && size % alignment == 0L) { \"invalid bulk record layout\" }\nval offsets = IntArray(widths.size) { i -> val value = data[at + 2 + i]; require(value >= 0 && value <= size - widths[i]) { \"invalid bulk field offset\" }; value.toInt() }\nreturn BulkLayout(size.toInt(), alignment.toInt(), offsets)\n}\n}\n");
    let sym = crate::symbol(prefix, "bulkLayouts");
    rust.extend(quote!{
        #[no_mangle] pub extern "system" fn #sym(mut env: ::prebindgen_jni_runtime::jni::JNIEnv,_class: ::prebindgen_jni_runtime::jni::objects::JClass)->::prebindgen_jni_runtime::jni::sys::jlongArray{
            let result=(||->Result<_,String>{#measurements let array=env.new_long_array(data.len() as i32).map_err(|e|e.to_string())?;env.set_long_array_region(&array,0,&data).map_err(|e|e.to_string())?;Ok(array.into_raw())})();
            match result{Ok(array)=>array,Err(message)=>{let _=env.throw_new("java/lang/IllegalStateException",message);std::ptr::null_mut()}}
        }
    });
    native.push_str("    @JvmStatic external fun bulkLayouts(): LongArray\n");
    files.insert(
        format!("{}/BulkLayouts.kt", prefix.replace('.', "/")),
        kotlin,
    );
    files.insert(
        format!("{}/BulkRuntime.kt", prefix.replace('.', "/")),
        include_str!("bulk.kt").replace("__PACKAGE__", prefix),
    );
    Ok(())
}

fn row_input(node: &JniNode, context: &KotlinContext<'_>) -> Result<KotlinInput> {
    let mut input = crate::kotlin::input_kotlin(node, context)?;
    input.prepare_arguments(crate::kotlin::input_handles(node, false))?;
    if input.parameters.is_empty() {
        input.parameters.push("bulkUnit: kotlin.Unit".into());
    }
    Ok(input)
}
fn row_output(node: &JniNode, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
    let mut output = crate::kotlin::output_kotlin(node, context)?;
    if output.delivered.is_empty() {
        output.delivered.push("kotlin.Unit".into());
        output.result_types.push("kotlin.Unit".into());
        output.result_names.push(None);
    }
    Ok(output)
}

fn schema(node: &JniNode) -> Result<String> {
    let field = |w: &Wire<JniWire>| -> Result<String> {
        Ok(match &w.kind {
            JniWire::Scalar { primitive, .. } => format!(
                "scalar:{}",
                prebindgen_flat::type_key(&primitive.rust_type())
            ),
            JniWire::Native128 { primitive, .. } => {
                format!("wide:{}", prebindgen_flat::type_key(&primitive.rust_type()))
            }
            JniWire::NativeBuffer { text } => if *text { "text" } else { "bytes" }.into(),
            JniWire::NativeRows { .. } => "rows".into(),
            JniWire::Selector { .. } => "selector".into(),
            JniWire::Enum { enum_type, .. } => format!(
                "enum:{}:{}",
                enum_type.class_name,
                enum_type
                    .mapping
                    .variants
                    .iter()
                    .map(|(name, value)| format!("{name}={value}"))
                    .collect::<Vec<_>>()
                    .join(";")
            ),
            JniWire::Handle { handle, .. } => format!(
                "handle:{}:{}",
                crate::names::type_identity(&handle.ty),
                match handle.access {
                    Access::Move => "move",
                    Access::Shared => "share",
                    Access::Exclusive => "exclusive",
                }
            ),
            _ => return Err("unsupported bulk schema field".into()),
        })
    };
    Ok(match node {
        WireNode::Leaf(wires) => format!(
            "leaf({})",
            wires
                .iter()
                .map(field)
                .collect::<Result<Vec<_>>>()?
                .join(",")
        ),
        WireNode::Chain {
            meta: JniChain::Sequence(_),
            child,
            ..
        } => format!("sequence({})", schema(child)?),
        WireNode::Chain {
            meta: JniChain::Optional(optional),
            child,
            ..
        } => {
            let kind = match optional {
                crate::JniOptional::Flag(_) => "flag".into(),
                crate::JniOptional::Niche { reservation, .. } => match reservation.niche {
                    JniNiche::Null => "null".into(),
                    JniNiche::Integer(n) => format!("niche:{n}"),
                },
            };
            format!("optional:{kind}({})", schema(child)?)
        }
        WireNode::Chain {
            meta: JniChain::Bulk(_),
            child,
            ..
        } => schema(child)?,
        WireNode::Branch {
            meta,
            children,
            controls,
        } => {
            let kind = match meta {
                JniBranch::Alternatives => "alternatives".into(),
                JniBranch::Group(JniGroup::Field(name)) => format!("field:{name}"),
                JniBranch::Group(JniGroup::Named(name)) => format!("view:{name}"),
                JniBranch::Group(JniGroup::Fields(_)) => "fields".into(),
                _ => "group".into(),
            };
            format!(
                "{kind}[{}]({})",
                controls
                    .iter()
                    .map(field)
                    .collect::<Result<Vec<_>>>()?
                    .join(","),
                children
                    .iter()
                    .map(schema)
                    .collect::<Result<Vec<_>>>()?
                    .join(",")
            )
        }
    })
}
pub(crate) fn callback_cache(node: &JniNode, context: &JniContext) -> Result<TokenStream> {
    let mut code = TokenStream::new();
    match node {
        WireNode::Chain {
            meta: JniChain::Bulk(layout),
            controls,
            ..
        } => {
            let wire = &controls[0].name;
            let length = &controls[1].name;
            let cache = format_ident!("{wire}_cached_bytes");
            let active = layout.active.rust();
            let env = &context.environment;
            code.extend(quote!(let #cache=if #active {::prebindgen_jni_runtime::bulk::input(&mut #env,&#wire,#length)?}else{Vec::new()};));
        }
        WireNode::Chain { child, .. } => code.extend(callback_cache(child, context)?),
        WireNode::Branch { children, .. } => {
            for child in children {
                code.extend(callback_cache(child, context)?);
            }
        }
        _ => {}
    }
    Ok(code)
}
pub(crate) fn cached_reads(code: &TokenStream) -> Result<TokenStream> {
    prebindgen_conversions::render_template(code, &|name| {
        Ok(if let Some(wire) = name.strip_prefix("bulk_read_") {
            let cache = format_ident!("{wire}_cached_bytes");
            quote!(Ok::<_,String>(#cache))
        } else {
            parameter(name)
        })
    })
}
pub(crate) fn scan_bulk(
    layout: &BulkRecord,
    controls: &[Wire<JniWire>],
    child: &JniNode,
    context: &JniContext,
) -> Result<TokenStream> {
    let scan = crate::callback::receive_return_owners(child, context)?;
    if scan.is_empty() {
        return Ok(scan);
    }
    let wire = &controls[0].name;
    let cache = format_ident!("{wire}_cached_bytes");
    let reader = format_ident!("{wire}_owner_reader");
    let active = layout.active.rust();
    let mut reads = TokenStream::new();
    for (i, w) in crate::group::row_fields(child).enumerate() {
        let name = &w.name;
        if matches!(w.kind, JniWire::NativeBuffer { .. }) {
            continue;
        }
        let read = native_read(w, layout, i, quote!(0usize))?;
        reads.extend(quote!(let #name=#read;));
    }
    let code = quote!(if #active {let #reader=::prebindgen_jni_runtime::bulk::Reader::new(&#cache);#reads #scan});
    resolve(&code, "bulk_reader", &quote!(#reader))
}
pub(crate) fn scan_rows(
    controls: &[Wire<JniWire>],
    child: &JniNode,
    context: &JniContext,
) -> Result<TokenStream> {
    let scan = crate::callback::receive_return_owners(child, context)?;
    if scan.is_empty() {
        return Ok(scan);
    }
    let layout = record(child)?;
    let ty = &layout.name;
    let reader = parameter("bulk_reader");
    let wire = &controls[0].name;
    let base = format_ident!("{wire}_ownership_base");
    let mut reads = TokenStream::new();
    for (i, w) in crate::group::row_fields(child).enumerate() {
        if matches!(w.kind, JniWire::NativeBuffer { .. }) {
            continue;
        }
        let name = &w.name;
        let read = native_read(w, &layout, i, quote!(#base))?;
        reads.extend(quote!(let #name=#read;));
    }
    Ok(
        quote!(if let Some(range)=#wire {let (start,count)=#reader.rows(Some(range),::core::mem::size_of::<#ty>())?;for index in 0..count {let #base=start+index*::core::mem::size_of::<#ty>();#reads #scan}}),
    )
}
