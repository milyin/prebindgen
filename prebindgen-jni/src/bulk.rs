//! C-representation packing of existing expansion trees.
use super::*;
use crate::kotlin::{JniNode, KotlinBinding, KotlinContext, KotlinInput, KotlinOutput};
use prebindgen_declarations::{Condition, SequenceInput, SequenceOutput};
use prebindgen_flat::{Wire, WireNode};
use std::{fmt::Write, rc::Rc};

/// Physical record shared by the Rust encoder and the initialized Kotlin layout table.
#[derive(Clone, Debug)]
pub struct JniBulkRecord {
    pub(crate) name: syn::Ident,
    pub(crate) definitions: TokenStream,
    pub(crate) offsets: Vec<TokenStream>,
    pub(crate) key: String,
    widths: Vec<usize>,
    active: Condition,
}
pub(crate) fn field_type(w: &JniWire) -> Result<syn::Type> {
    Ok(match w {
        JniWire::NativeBuffer{..}|JniWire::NativeRows{..}=>syn::parse_quote!(::prebindgen_jni_runtime::bulk::Range),
        JniWire::Native128{..}=>syn::parse_quote!([u8;16]),
        JniWire::Scalar{primitive:Primitive::I128|Primitive::U128,..}|JniWire::Text{..}|JniWire::Bytes{..}|JniWire::Rows{..}|JniWire::BulkBuffer{..}|JniWire::Callback{..}=>return Err("bulk fields require native scalar/buffer/row encodings; callback objects cannot be packed".into()),
        _=>w.rust_type(),
    })
}
pub(crate) fn record(node: &JniNode) -> Result<Rc<JniBulkRecord>> {
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
    let (definitions, offsets) = physical_record(node, &name, fields)?;
    Ok(Rc::new(JniBulkRecord {
        name,
        definitions,
        offsets,
        key,
        widths,
        active: Condition::default(),
    }))
}
fn parameter(name: &str) -> TokenStream {
    prebindgen_conversions::template_parameter(name)
}
fn resolve(code: &TokenStream, key: &str, value: &TokenStream) -> Result<TokenStream> {
    prebindgen_conversions::render_template(
        code,
        &|name| {
            Ok(if name == key {
                value.clone()
            } else {
                parameter(name)
            })
        },
        &|count| Ok(prebindgen_conversions::record_input_transfer(count.clone())),
    )
}
pub(crate) fn native_read(
    w: &Wire<JniWire>,
    record: &JniBulkRecord,
    i: usize,
    base: TokenStream,
    reader: TokenStream,
) -> Result<TokenStream> {
    let field_offset = record.offsets[i].clone();
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
    record: &JniBulkRecord,
    i: usize,
    base: TokenStream,
    value: TokenStream,
) -> Result<TokenStream> {
    let writer = parameter("bulk_writer");
    let field_offset = record.offsets[i].clone();
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
    mut child: JniInputPlan<JniWire>,
    active: &Condition,
) -> Result<JniInputPlan<JniWire>> {
    let mut layout = (*record(&child.node)?).clone();
    layout.active = active.clone();
    let layout = Rc::new(layout);
    let ty = &layout.name;
    let length = format_ident!("{wire}_length");
    let bytes = format_ident!("{wire}_bytes");
    let reader = format_ident!("{wire}_reader");
    let mut bind = TokenStream::new();
    let wires = crate::group::row_fields(&child.node).collect::<Vec<_>>();
    let selection = crate::selected::Selection::new(&child.node)?;
    for i in selection.order()? {
        let w = wires[i];
        let name = &w.name;
        let read = native_read(w, &layout, i, quote!(0usize), parameter("bulk_reader"))?;
        let absent = crate::optional::absent(&w.kind, None, true)?;
        let selected = selection.rust(i, |j| {
            let n = &wires[j].name;
            quote!(#n)
        });
        let condition = if selection.tests[i].is_empty() {
            active.rust()
        } else {
            active.and(selected)
        };
        bind.extend(quote!(let #name=if #condition {#read}else{#absent};));
    }
    let bind = resolve(&bind, "bulk_reader", &quote!(#reader))?;
    let conversion = resolve(&child.code, "bulk_reader", &quote!(#reader))?;
    child.storage = resolve(&child.storage, "bulk_reader", &quote!(#reader))?;
    let backing = std::mem::take(&mut child.storage);
    // The carrier is initialized only on the selected conversion path. Rust's
    // definite-initialization/drop tracking preserves that conditional lifetime;
    // an Option adds an unnecessary presence check and can spill the row-loop
    // pointers even though the carrier is always present when the reader is used.
    child.storage = quote!(#[allow(clippy::needless_late_init)] let #bytes; #backing);
    let read = parameter(&format!("bulk_read_{wire}"));
    let env = parameter("environment");
    // Keep the wire operands explicit so enclosing row composition can rebind
    // them to the successfully decoded fields rather than their Result locals.
    let read = quote!((#read)(&mut #env, &#wire, #length));
    let truncated = active.and(quote!(#bytes.len()<::core::mem::size_of::<#ty>()));
    child.code = quote! {{
        #[allow(clippy::redundant_closure_call)]
        let bulk_value=#read?;
        #bytes=bulk_value;
        let #bytes=&#bytes;
        let #reader=&::prebindgen_jni_runtime::bulk::Reader::new(#bytes);
        if #truncated{return Err("truncated bulk root record".into());}
        #bind #conversion
    }};
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
    mut child: JniOutputPlan<JniWire>,
) -> Result<JniOutputPlan<JniWire>> {
    let layout = record(&child.outcomes.success)?;
    let ty = &layout.name;
    let writer = format_ident!("{wire}_writer");
    let prepare = resolve(&child.statements, "bulk_writer", &quote!(#writer))?;
    let values = crate::group::row_values(&child.outcomes.success, &child.values)?;
    let mut store = TokenStream::new();
    let selection = crate::selected::Selection::new(&child.outcomes.success)?;
    for (i, (w, value)) in values.iter().enumerate() {
        let condition = selection.rust(i, |j| values[j].1.clone());
        let write = native_store(w, &layout, i, quote!(0usize), (*value).clone())?;
        if selection.tests[i].is_empty() {
            store.extend(write);
        } else {
            store.extend(quote!(if #condition {#write}));
        }
    }
    let store = resolve(&store, "bulk_writer", &quote!(#writer))?;
    let env = parameter("environment");
    let commit = &child.commit;
    child.commit = quote!(#commit #writer.commit(););
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
    let wires = crate::group::row_fields(element).collect::<Vec<_>>();
    let selection = crate::selected::Selection::new(element)?;
    for i in selection.order()? {
        let w = wires[i];
        let name = &w.name;
        let read = native_read(w, &layout, i, base.clone(), parameter("bulk_reader"))?;
        let condition = selection.rust(i, |j| {
            let n = &wires[j].name;
            quote!(#n)
        });
        let absent = crate::optional::absent(&w.kind, None, true)?;
        if selection.tests[i].is_empty() {
            bind.extend(quote!(let #name=#read;));
        } else {
            bind.extend(quote!(let #name=if #condition {#read}else{#absent};));
        }
    }
    for w in wires {
        let name = &w.name;
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
        validation_values: values,
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
    let owner = format_ident!("{wire}_owners");
    let release = crate::cleanup::helper(element, crate::cleanup::Carrier::Native)?;
    let owners = if let Some(release) = &release {
        quote!(let #owner = unsafe { #writer.own_rows(#wire, ::core::mem::size_of::<#row>(), #release) };)
    } else {
        quote!()
    };
    let mut store = TokenStream::new();
    let values = crate::group::row_values(element, values)?;
    let selection = crate::selected::Selection::new(element)?;
    for (i, (w, value)) in values.iter().enumerate() {
        let condition = selection.rust(i, |j| values[j].1.clone());
        let write = native_store(
            w,
            &layout,
            i,
            quote!(#wire.offset as usize+#index*::core::mem::size_of::<#row>()),
            (*value).clone(),
        )?;
        if selection.tests[i].is_empty() {
            store.extend(write);
        } else {
            store.extend(quote!(if #condition {#write}));
        }
    }
    let commit_element = parameter("commit_element");
    store.extend(quote!(#commit_element));
    if release.is_some() {
        store.extend(quote!(unsafe { #writer.commit_rows(#owner, #index + 1); }));
    }
    Ok(SequenceOutput {
        controls: vec![Wire {
            name: wire.clone(),
            kind: JniWire::NativeRows {
                condition: "true".into(),
            },
        }],
        metadata: JniChain::Sequence(JniSequence::Bulk),
        commit_each: true,
        fallible: true,
        begin: quote!(let #wire=#writer.rows(#length,::core::mem::size_of::<#row>(),::core::mem::align_of::<#row>())?; #owners),
        enter: quote!(),
        store,
        finish: quote!(let #wire=Some(#wire);),
        values: vec![(wire.clone(), quote!(#wire))],
        commit: quote!(),
    })
}

fn decode_buffer(text: bool, wire: &syn::Ident) -> TokenStream {
    let reader = parameter("bulk_reader");
    if text {
        quote!(::core::str::from_utf8(#reader.buffer_range(#wire)?.ok_or("null bulk text")?).map_err(|_|"invalid UTF-8 in bulk text")?)
    } else {
        quote!(#reader.buffer_range(#wire)?.ok_or("null bulk bytes")?)
    }
}

pub(crate) fn input_leaf(
    flat: &JniBindings,
    ty: &syn::Type,
    w: syn::Ident,
    v: syn::Ident,
    active: &Condition,
) -> Result<JniInputPlan<JniWire>> {
    match Value::of(ty) {
        Some(Value::Text) | Some(Value::Bytes) => {
            let text = Value::of(ty) == Some(Value::Text);
            let decode = decode_buffer(text, &w);
            prebindgen_conversions::value::input(ty, w, v, decode, JniWire::NativeBuffer { text })
        }
        Some(Value::Scalar(p @ (Primitive::I128 | Primitive::U128))) => {
            let native = p.rust_type();
            let decode = quote!(#native::from_be_bytes(#w));
            prebindgen_conversions::value::input(
                ty,
                w,
                v,
                decode,
                JniWire::Native128 {
                    primitive: p,
                    condition: kotlin_condition(active),
                },
            )
        }
        _ => JniCodec::raw().input_leaf(prebindgen_declarations::InputLeaf::new(
            flat,
            ty,
            &(),
            w,
            v,
            active,
        )),
    }
}
pub(crate) fn output_leaf(
    flat: &JniBindings,
    ty: &syn::Type,
    w: syn::Ident,
    value: TokenStream,
    borrowed: bool,
) -> Result<JniOutputPlan<JniWire>> {
    if let Some(kind @ (Value::Text | Value::Bytes)) = Value::of(ty) {
        let owner = format_ident!("{w}_source");
        let text = kind == Value::Text;
        let bytes = if text {
            quote!(#owner.as_bytes())
        } else {
            quote!(#owner.as_ref())
        };
        let writer = parameter("bulk_writer");
        return Ok(JniOutputPlan {
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
    JniCodec::raw().output_leaf(prebindgen_declarations::OutputLeaf::new(
        flat,
        ty,
        &(),
        w,
        value,
        if borrowed {
            prebindgen_declarations::ValueAccess::Shared
        } else {
            prebindgen_declarations::ValueAccess::Owned
        },
    ))
}
fn kt_layout(record: &JniBulkRecord, context: &KotlinContext<'_>) -> String {
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

fn input_writer(layout: &JniBulkRecord) -> String {
    format!(
        "Input{}",
        layout.name.to_string().trim_start_matches("__PbBulk")
    )
}

/// Recompute handle positions from measured record offsets and packed row ranges.
/// Only handle-bearing ranges are visited; no source objects are read again.
fn patch_handles(
    node: &JniNode,
    package: &str,
    base: &str,
    next: &mut usize,
    kinds: &[BoxedWire],
) -> Result<String> {
    let layout = record(node)?;
    let layout = format!(
        "{package}.BulkLayouts.r{:016x}",
        prebindgen_reprc::schema_id(&layout.key)
    );
    let selection = crate::selected::Selection::new(node)?;
    let wires = group::row_fields(node).collect::<Vec<_>>();
    let mut code = String::new();
    for (field, wire) in crate::group::row_fields(node).enumerate() {
        if !matches!(
            wire.kind,
            JniWire::Handle { .. } | JniWire::NativeRows { .. }
        ) {
            continue;
        }
        let at = format!("({base}) + {layout}.offsets[{field}]");
        let predicates = selection.tests[field]
            .iter()
            .map(|test| {
                let i = test.index();
                let at = format!("({base}) + {layout}.offsets[{i}]");
                let kind = &wires[i].kind;
                let accessor = match kind {
                    JniWire::NativeBuffer { .. } | JniWire::NativeRows { .. } => "long",
                    _ => kind.bulk_scalar_access()?.0,
                };
                let value = format!("{accessor}({at})");
                if matches!(
                    test,
                    crate::selected::Test::Present(_, Some(JniNiche::Null))
                ) {
                    Ok(
                        if matches!(
                            kind,
                            JniWire::NativeBuffer { .. } | JniWire::NativeRows { .. }
                        ) {
                            format!("{value} >= 0L")
                        } else {
                            format!("{value} != 0L")
                        },
                    )
                } else {
                    Ok(test.kotlin(&value))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let condition = if predicates.is_empty() {
            "true".into()
        } else {
            predicates.join(" && ")
        };
        let conditional = condition != "true";
        if conditional {
            code.push_str(&format!("if ({condition}) {{\n"));
        }
        match &wire.kind {
            JniWire::Handle { handle, .. } => {
                let group = kinds
                    .iter()
                    .position(|kind| crate::handles::key(kind) == crate::handles::key(handle))
                    .ok_or("missing bulk handle group")?;
                let index = format!("handleIndex{group}");
                code.push_str(&format!("{index} = patchHandle({at}, {index}, {group})\n"));
            }
            JniWire::NativeRows { .. } => {
                let child =
                    sequence_child(node, &wire.name).ok_or("missing bulk collection element")?;
                if !crate::kotlin::input_handles(child, false) {
                    if conditional {
                        code.push_str("}\n");
                    }
                    continue;
                }
                let child_layout = record(child)?;
                let child_layout = format!(
                    "{package}.BulkLayouts.r{:016x}",
                    prebindgen_reprc::schema_id(&child_layout.key)
                );
                let id = *next;
                *next += 1;
                let start = format!("rows{id}Start");
                let count = format!("rows{id}Count");
                let index = format!("rows{id}Index");
                code.push_str(&format!("val {start} = long({at}).toInt()\nif ({start} >= 0) {{\nval {count} = long(({at}) + 8).toInt()\nvar {index} = 0\nwhile ({index} < {count}) {{\n"));
                code.push_str(&patch_handles(
                    child,
                    package,
                    &format!("{start} + {index} * {child_layout}.size"),
                    next,
                    kinds,
                )?);
                code.push_str(&format!("{index}++\n}}\n}}\n"));
            }
            _ => {}
        }
        if conditional {
            code.push_str("}\n");
        }
    }
    Ok(code)
}

fn input_writers(
    node: &JniNode,
    input: bool,
    package: &str,
    out: &mut BTreeMap<String, String>,
) -> Result<()> {
    match node {
        WireNode::Chain {
            meta: JniChain::Bulk(layout),
            child,
            ..
        } => {
            if input && crate::kotlin::input_handles(child, false) {
                let name = input_writer(layout);
                let kinds = crate::handles::kinds(child);
                let count = kinds.len();
                let patch = patch_handles(child, package, "0", &mut 0, &kinds)?;
                let mut indices = String::new();
                for i in 0..count {
                    writeln!(indices, "var handleIndex{i} = 0").unwrap();
                }
                let class = format!("internal class {name} : BulkHandleWriter(BulkLayouts.r{:016x}.size, {count}) {{\noverride fun patchHandles() {{\n{indices}{patch}}}\n}}\n",prebindgen_reprc::schema_id(&layout.key));
                if let Some(previous) = out.get(&name) {
                    if previous != &class {
                        return Err(
                            "bulk input schema has incompatible handle patching layouts".into()
                        );
                    }
                } else {
                    out.insert(name, class);
                }
            }
            input_writers(child, input, package, out)?;
        }
        WireNode::Chain { child, .. } => input_writers(child, input, package, out)?,
        WireNode::Branch { children, .. } => {
            for child in children {
                input_writers(child, input, package, out)?;
            }
        }
        WireNode::Leaf(wires) => {
            for wire in wires {
                if let JniWire::Callback { callback } = &wire.kind {
                    for (input, node) in callback.bulk_nodes() {
                        input_writers(node, input, package, out)?;
                    }
                }
            }
        }
    }
    Ok(())
}
fn pack(
    node: &JniNode,
    args: &[String],
    writer: &str,
    base: &str,
    context: &KotlinContext<'_>,
    kinds: &[BoxedWire],
) -> Result<String> {
    let layout = record(node)?;
    let layout = kt_layout(&layout, context);
    let wires = crate::group::row_fields(node).collect::<Vec<_>>();
    if wires.len() != args.len() {
        return Err("bulk Kotlin input fields differ from the record schema".into());
    }
    let selection = crate::selected::Selection::new(node)?;
    let mut code = String::new();
    for (i, (w, arg)) in wires.iter().zip(args).enumerate() {
        let condition = selection.kotlin(i, |j| args[j].clone());
        let conditional = condition != "true";
        if conditional {
            code.push_str(&format!("if ({condition}) {{\n"));
        }
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
                        kinds,
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
            JniWire::Handle { handle, .. } => {
                let group = kinds
                    .iter()
                    .position(|kind| crate::handles::key(kind) == crate::handles::key(handle))
                    .ok_or("missing packed handle group")?;
                code.push_str(&format!(
                    "{writer}.handle({at}, {arg}, {}, {group})\n",
                    handle.access == Access::Move
                ));
            }
            _ => return Err("unsupported field in bulk Kotlin writer".into()),
        }
        if conditional {
            code.push_str("}\n");
        }
    }
    Ok(code)
}
pub(crate) fn input_list(
    carrier: &Wire<JniWire>,
    element: &JniNode,
    context: &KotlinContext<'_>,
    active: &str,
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
        handle_inputs: if crate::kotlin::input_handles(element, false) {
            None
        } else {
            Some(Vec::new())
        },
        parameters: vec![format!(
            "{name}: kotlin.collections.List<{ty}>{}",
            if active == "true" { "" } else { "?" }
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
    layout: &JniBulkRecord,
    controls: &[Wire<JniWire>],
    child: &JniNode,
    context: &KotlinContext<'_>,
    active: &str,
) -> Result<KotlinInput> {
    let mut input = crate::kotlin::input_kotlin_when(child, context, active)?;
    let writer = format!("{}_writer", controls[0].name);
    let schema = kt_layout(layout, context);
    input.prepare_arguments(crate::kotlin::input_handles(child, false))?;
    let body = pack(
        child,
        &input.prepared_arguments,
        &writer,
        "0",
        context,
        &crate::handles::kinds(child),
    )?;
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
            expression:format!("if ({condition}) run {{ val {writer} = {}.BulkInputs.{}()\n{body}\n{writer} }} else null",context.package,input_writer(layout)),
            absent:"null".into(),
        });
        input.arguments = vec![
            format!("{state}?.encode()"),
            format!("{state}?.length ?: 0"),
        ];
        input.prepared_arguments = vec![state.clone(), format!("{state}?.length ?: 0")];
        input.locks.clear();
        input.handle_inputs = Some(crate::handles::carrier_inputs(child, &state));
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
    _layout: &JniBulkRecord,
    child: &JniNode,
    context: &KotlinContext<'_>,
    owners: Option<&str>,
) -> Result<KotlinOutput> {
    let token = format!("{}bulkRow", context.output_prefix);
    let reader_state = format!("{}bulkReader", context.output_prefix);
    let reader = format!("requireNotNull({reader_state})");
    let ctx = KotlinContext {
        result: &token,
        ..*context
    };
    let mut output = crate::kotlin::output_kotlin_scoped(child, &ctx, owners)?;
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
pub(crate) fn output_list(
    element: &JniNode,
    context: &KotlinContext<'_>,
    owners: Option<&str>,
) -> Result<KotlinOutput> {
    if owners.is_none() && crate::kotlin::input_handles(element, false) {
        return Err("handle collection output requires an ownership scope".into());
    }
    let prefix = context.output_prefix;
    let row = format!("{prefix}bulkRow");
    let base = format!("{prefix}bulkBase");
    let index = format!("{prefix}bulkIndex");
    let range = format!("{prefix}bulkRange");
    let list = format!("{prefix}bulkList");
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
    let mut child = row_output(element, &child_context, owners)?;
    if child.delivered.len() != 1 {
        return Err(
            "bulk collection outputs must expose one Kotlin value; declare a data class".into(),
        );
    }
    remap(&mut child, element, &row, reader, &base, context)?;
    let ty = &child.result_types[0];
    let value = &child.delivered[0];
    let raw = format!("({}[0] as {}.BulkRange?)", context.result, context.package);
    let locals=format!("var {range}: {}.BulkRange? = null\nvar {list}: kotlin.collections.MutableList<{ty}>? = null\n",context.package);
    let materialize = format!("{range} = {reader}.rows({raw}, {schema}.size)\n{list} = kotlin.collections.ArrayList({range}!!.length)\nfor ({index} in 0 until {range}!!.length) {{\nval {base} = {range}!!.offset + {index} * {schema}.size\n{}\n{}\n{list}!!.add({value})\n}}\n", child.output_locals, child.materialize);
    let cleanup_error = format!("{prefix}bulkCleanupError");
    let record_row_failure = crate::kotlin::record_cleanup_failure(&child_context, &cleanup_error);
    let record_outer_failure = crate::kotlin::record_cleanup_failure(context, &cleanup_error);
    let cleanup = if !child.cleanup.is_empty() {
        let initial = context.cleanup_failure.unwrap_or(context.failure);
        format!("try {{ if ({range} == null) {range} = {reader}.rows({raw}, {schema}.size)\nfor ({index} in 0 until {range}!!.length) {{\nval {base} = {range}!!.offset + {index} * {schema}.size\nvar {failure}: Throwable? = {initial}\ntry {{ {}\n{} }} catch ({cleanup_error}: Throwable) {{ {record_row_failure} }}\n{}\n}} }} catch ({cleanup_error}: Throwable) {{ {record_outer_failure} }}\n",child.output_locals,child.cleanup,context.cleanup_failure.map(|f|format!("{f} = {failure}")).unwrap_or_default())
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

fn register(
    record: Rc<JniBulkRecord>,
    out: &mut BTreeMap<String, Rc<JniBulkRecord>>,
) -> Result<()> {
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
fn records(node: &JniNode, out: &mut BTreeMap<String, Rc<JniBulkRecord>>) -> Result<()> {
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
pub(crate) fn emit<I: KotlinInputConversion, O: KotlinOutputConversion>(
    flat: &Flat<JniSchema<I, O>>,
    rust: &mut TokenStream,
    native: &mut String,
    files: &mut BTreeMap<String, String>,
) -> Result<()> {
    let mut catalogue = BTreeMap::new();
    let prefix = &flat.metadata.package;
    let mut writers = BTreeMap::new();
    for (input, node) in crate::metadata::nodes(flat) {
        records(node, &mut catalogue)?;
        input_writers(node, input, prefix, &mut writers)?;
    }
    if catalogue.is_empty() {
        return Ok(());
    }
    if !writers.is_empty() {
        files.insert(format!("{}/BulkInputs.kt",prefix.replace('.',"/")),format!("@file:Suppress(\"ASSIGNED_BUT_NEVER_ACCESSED_VARIABLE\", \"UNUSED_VALUE\")\npackage {prefix}\n\ninternal object BulkInputs {{\n{}}}\n",writers.values().cloned().collect::<String>()));
    }
    let schema = catalogue
        .values()
        .map(|r| r.key.as_str())
        .collect::<Vec<_>>()
        .join("|");
    let hash = prebindgen_reprc::schema_id(&schema) as i64;
    let mut measurements = quote!(let mut data=vec![1i64,#hash,if cfg!(target_endian="big"){1}else{0},::core::mem::size_of::<usize>() as i64];);
    let mut kotlin=format!("package {prefix}\n\ninternal object BulkLayouts {{\nprivate val data = JNINative.bulkLayouts()\ninit {{ require(data.size == {} && data[0] == 1L && data[1] == {hash}L && data[2] in 0L..1L && (data[3] == 4L || data[3] == 8L)) {{ \"incompatible bulk layout catalogue\" }} }}\nval byteOrder = if(data[2] == 0L) java.nio.ByteOrder.LITTLE_ENDIAN else java.nio.ByteOrder.BIG_ENDIAN\n",4+catalogue.values().map(|r|r.offsets.len()+2).sum::<usize>());
    let mut offset = 4;
    for r in catalogue.values() {
        let definition = &r.definitions;
        rust.extend(quote!(#[allow(non_camel_case_types,dead_code)] #definition));
        let name = &r.name;
        let offsets = &r.offsets;
        let layout = quote!([::core::mem::size_of::<#name>(),::core::mem::align_of::<#name>(),#(#offsets),*]);
        measurements.extend(quote!(data.extend((#layout).into_iter().map(|v|v as i64));));
        let widths = r
            .widths
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let count = r.offsets.len();
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
fn row_output(
    node: &JniNode,
    context: &KotlinContext<'_>,
    owners: Option<&str>,
) -> Result<KotlinOutput> {
    let mut output = crate::kotlin::output_kotlin_scoped(node, context, owners)?;
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
            meta: JniBranch::Group(JniGroup::Argument(_)),
            children,
            controls,
        } if controls.is_empty() && children.len() == 1 => schema(&children[0])?,
        WireNode::Branch {
            meta,
            children,
            controls,
        } => {
            let kind = match meta {
                JniBranch::Alternatives => "alternatives".into(),
                JniBranch::Enum(e) => format!("enum:{}:{}", type_key(&e.ty), e.variants.join(";")),
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
// Only bulk roots in this native scope can be cached before decoding. An
// ordinary sequence introduces per-row bindings, including its bulk buffers.
fn callback_roots(node: &JniNode) -> Vec<&JniNode> {
    match node {
        WireNode::Chain {
            meta: JniChain::Bulk(_),
            ..
        } => vec![node],
        WireNode::Chain {
            meta: JniChain::Sequence(_),
            ..
        } => vec![],
        WireNode::Chain { child, .. } => callback_roots(child),
        WireNode::Branch { children, .. } => children.iter().flat_map(callback_roots).collect(),
        _ => vec![],
    }
}
pub(crate) fn callback_cache(node: &JniNode, context: &JniContext) -> Result<TokenStream> {
    let mut code = TokenStream::new();
    for root in callback_roots(node) {
        if let WireNode::Chain {
            meta: JniChain::Bulk(layout),
            controls,
            ..
        } = root
        {
            let wire = &controls[0].name;
            let length = &controls[1].name;
            let cache = format_ident!("{wire}_cached_bytes");
            let active = layout.active.rust();
            let env = &context.environment;
            code.extend(quote!(let #cache=if #active {::prebindgen_jni_runtime::bulk::input(&mut #env,&#wire,#length)?}else{Vec::new()};));
        }
    }
    Ok(code)
}
pub(crate) fn cached_reads(code: &TokenStream, node: &JniNode) -> Result<TokenStream> {
    let roots = callback_roots(node)
        .into_iter()
        .filter_map(|root| {
            if let WireNode::Chain { controls, .. } = root {
                Some(controls[0].name.to_string())
            } else {
                None
            }
        })
        .collect::<std::collections::BTreeSet<_>>();
    prebindgen_conversions::render_template(
        code,
        &|name| {
            Ok(
                if let Some(wire) = name
                    .strip_prefix("bulk_read_")
                    .filter(|wire| roots.contains(*wire))
                {
                    let cache = format_ident!("{wire}_cached_bytes");
                    quote!(|_, _, _| Ok::<_,String>(#cache.as_slice()))
                } else {
                    parameter(name)
                },
            )
        },
        &|count| Ok(prebindgen_conversions::record_input_transfer(count.clone())),
    )
}
// Read independent owned fields separately on failure: a malformed nested
// range must not prevent releasing valid sibling handles. All reads use cached
// native bytes; no JNI calls or source constructors belong in this scan.
fn cleanup_attempt(code: TokenStream) -> TokenStream {
    quote! {
        if let Err(error) = (|| -> ::prebindgen_runtime::Result<()> { #code Ok(()) })() {
            ::prebindgen_runtime::report_callback_cleanup_error(&error);
        }
    }
}

pub(crate) fn cleanup_cached(
    layout: &JniBulkRecord,
    controls: &[Wire<JniWire>],
    child: &JniNode,
) -> Result<TokenStream> {
    // Like the eager scan, reclaim readable owner fields even in an incomplete
    // root. Reader bounds-checks each field; missing siblings cannot hide them.
    let scan = cleanup_native(child, child, layout, quote!(0usize))?;
    if scan.is_empty() {
        return Ok(scan);
    }
    let cache = format_ident!("{}_cached_bytes", controls[0].name);
    let reader = format_ident!("{}_cleanup_reader", controls[0].name);
    let active = layout.active.rust();
    let code = quote!(if #active {
        let #reader=::prebindgen_jni_runtime::bulk::Reader::new(&#cache);
        #scan
    });
    resolve(&code, "bulk_reader", &quote!(#reader))
}

// The ordinary-row cleanup walker has read this occurrence's JVM buffer after
// decoding failed. Reuse the native scanner rather than introducing another
// ownership traversal for packed children.
pub(crate) fn cleanup_buffer(
    layout: &JniBulkRecord,
    child: &JniNode,
    bytes: &syn::Ident,
) -> Result<TokenStream> {
    let scan = cleanup_native(child, child, layout, quote!(0usize))?;
    let reader = format_ident!("{bytes}_reader");
    resolve(
        &quote! {
            let #reader=::prebindgen_jni_runtime::bulk::Reader::new(&#bytes);
            #scan
        },
        "bulk_reader",
        &quote!(#reader),
    )
}

fn cleanup_native(
    node: &JniNode,
    row: &JniNode,
    layout: &JniBulkRecord,
    base: TokenStream,
) -> Result<TokenStream> {
    if !crate::kotlin::input_handles(node, true) {
        return Ok(quote!());
    }
    let read = |name: &syn::Ident| {
        let (i, wire) = crate::group::row_fields(row)
            .enumerate()
            .find(|(_, wire)| wire.name == *name)
            .ok_or_else(|| format!("missing bulk cleanup field `{name}`"))?;
        native_read(wire, layout, i, base.clone(), parameter("bulk_reader"))
    };
    Ok(match node {
        WireNode::Leaf(fields) => {
            let mut code = TokenStream::new();
            for field in fields {
                if let JniWire::Handle { handle, .. } = &field.kind {
                    if handle.access == Access::Move {
                        let value = read(&field.name)?;
                        let ty = &handle.ty;
                        code.extend(cleanup_attempt(quote!(
                            unsafe { callback_return_owners.receive(
                                (#value) as *mut ::prebindgen_jni_runtime::Allocation<#ty>
                            ); }
                        )));
                    }
                }
            }
            code
        }
        WireNode::Chain {
            meta: JniChain::Optional(optional),
            child,
            ..
        } => {
            let name = optional.discriminator();
            let value = read(name)?;
            let present = if let Some(niche) = optional.niche() {
                let field = crate::group::row_fields(row)
                    .find(|field| field.name == *name)
                    .ok_or("missing bulk cleanup discriminator")?;
                niche.present(&field.kind, quote!(#name))
            } else {
                quote!(#name == 1)
            };
            let scan = cleanup_native(child, row, layout, base)?;
            cleanup_attempt(quote!(let #name=#value; if #present { #scan }))
        }
        WireNode::Chain {
            meta: JniChain::Sequence(JniSequence::Bulk),
            controls,
            child,
        } => {
            let value = read(&controls[0].name)?;
            let element = record(child)?;
            let ty = &element.name;
            let start = format_ident!("{}_cleanup_start", controls[0].name);
            let count = format_ident!("{}_cleanup_count", controls[0].name);
            let index = format_ident!("{}_cleanup_index", controls[0].name);
            let reader = parameter("bulk_reader");
            let scan = cleanup_native(
                child,
                child,
                &element,
                quote!(#start + #index * ::core::mem::size_of::<#ty>()),
            )?;
            cleanup_attempt(quote!(
                let (#start,#count)=#reader.rows(#value,::core::mem::size_of::<#ty>())?;
                for #index in 0..#count { #scan }
            ))
        }
        WireNode::Chain { child, .. } => cleanup_native(child, row, layout, base)?,
        WireNode::Branch {
            meta: JniBranch::Alternatives | JniBranch::Enum(_),
            controls,
            children,
        } => {
            let name = &controls[0].name;
            let value = read(name)?;
            let mut scans = TokenStream::new();
            for (index, child) in children.iter().enumerate() {
                let index = index as i32;
                let scan = cleanup_native(child, row, layout, base.clone())?;
                scans.extend(quote!(if #name==#index { #scan }));
            }
            cleanup_attempt(quote!(let #name=#value; #scans))
        }
        WireNode::Branch { children, .. } => children
            .iter()
            .map(|child| cleanup_native(child, row, layout, base.clone()))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect(),
    })
}

// Preserve ordinary flat records while grouping enum alternatives into target-
// compiled unions. Offsets stay in logical wire order for both language sides.
fn physical_record(
    node: &JniNode,
    name: &syn::Ident,
    fields: Vec<(syn::Ident, syn::Type)>,
) -> Result<(TokenStream, Vec<TokenStream>)> {
    enum Piece {
        Wire(usize),
        Enum(usize, Vec<Vec<Piece>>),
    }
    fn pieces(node: &JniNode, next: &mut usize) -> Vec<Piece> {
        let mut take = |n: usize| {
            let start = *next;
            *next += n;
            (start..start + n).map(Piece::Wire).collect::<Vec<_>>()
        };
        match node {
            WireNode::Leaf(w) => take(w.len()),
            WireNode::Chain {
                controls,
                meta: JniChain::Sequence(_) | JniChain::Bulk(_),
                ..
            } => take(controls.len()),
            WireNode::Chain {
                controls, child, ..
            } => {
                let mut out = take(controls.len());
                out.extend(pieces(child, next));
                out
            }
            WireNode::Branch {
                controls,
                children,
                meta: JniBranch::Enum(_),
            } => {
                debug_assert_eq!(controls.len(), 1);
                let tag = *next;
                *next += 1;
                vec![Piece::Enum(
                    tag,
                    children.iter().map(|c| pieces(c, next)).collect(),
                )]
            }
            WireNode::Branch {
                controls, children, ..
            } => {
                let mut out = take(controls.len());
                for child in children {
                    out.extend(pieces(child, next));
                }
                out
            }
        }
    }
    fn compile(
        name: syn::Ident,
        pieces: &[Piece],
        fields: &[(syn::Ident, syn::Type)],
        definitions: &mut TokenStream,
        offsets: &mut [TokenStream],
    ) -> Result<prebindgen_reprc::Record> {
        let mut members = vec![];
        for piece in pieces {
            match piece {
                Piece::Wire(i) => {
                    let (field, ty) = &fields[*i];
                    members.push((field.clone(), ty.clone()));
                    offsets[*i] = quote!(::core::mem::offset_of!(#name,#field));
                }
                Piece::Enum(tag, arms) => {
                    let carrier = format_ident!("{name}_enum{tag}");
                    let member = format_ident!("e{tag}");
                    let prefix = quote!(::core::mem::offset_of!(#name,#member));
                    let mut records = vec![];
                    let mut indices = vec![];
                    for (i, arm) in arms.iter().enumerate() {
                        let mut local = vec![TokenStream::new(); fields.len()];
                        let record = if arm.is_empty() {
                            None
                        } else {
                            Some(compile(
                                format_ident!("{carrier}_a{i}"),
                                arm,
                                fields,
                                definitions,
                                &mut local,
                            )?)
                        };
                        indices.push(local);
                        records.push(record);
                    }
                    let union = prebindgen_reprc::TaggedUnion::new(carrier.clone(), records)?;
                    offsets[*tag] = quote!((#prefix)+::core::mem::offset_of!(#carrier,tag));
                    for arm in indices {
                        for (index, local) in arm.into_iter().enumerate() {
                            if !local.is_empty() {
                                offsets[index] = quote!((#prefix)+::core::mem::offset_of!(#carrier,payload)+(#local));
                            }
                        }
                    }
                    // compile() emitted nested dependencies, not these arm records.
                    definitions.extend(union.definition());
                    members.push((member, union.ty()));
                }
            }
        }
        if members.is_empty() {
            members.push((syn::parse_quote!(unused), syn::parse_quote!(u8)));
        }
        prebindgen_reprc::Record::new(name, members)
    }
    let mut next = 0;
    let parts = pieces(node, &mut next);
    let mut offsets = vec![TokenStream::new(); fields.len()];
    let mut definitions = TokenStream::new();
    let record = compile(
        name.clone(),
        &parts,
        &fields,
        &mut definitions,
        &mut offsets,
    )?;
    let definition = record.definition();
    definitions.extend(quote!(#definition));
    if next == 0 {
        offsets[0] = quote!(0usize);
    }
    Ok((definitions, offsets))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handle_patch_predicate_uses_the_64_bit_niche_donor() {
        let handle = BoxedWire {
            ty: syn::parse_quote!(Resource),
            access: Access::Move,
        };
        let donor: syn::Ident = syn::parse_quote!(number);
        let node = WireNode::Chain {
            controls: vec![],
            meta: JniChain::Optional(JniOptional::Niche {
                presence: syn::parse_quote!(present),
                reservation: prebindgen_declarations::NicheReservation {
                    discriminator: donor.clone(),
                    niche: JniNiche::Integer(i64::MIN),
                },
            }),
            child: Box::new(WireNode::Leaf(vec![
                Wire {
                    name: donor,
                    kind: JniWire::scalar(Primitive::U32),
                },
                Wire {
                    name: syn::parse_quote!(resource),
                    kind: JniWire::handle(handle.clone(), "true"),
                },
            ])),
        };
        // u32 uses a JVM long so excluded values can encode absence. Truncating
        // MIN to 32 bits changes it to zero and wrongly patches an absent owner.
        let code = patch_handles(&node, "test", "0", &mut 0, &[handle]).unwrap();
        assert!(code.contains("long((0)"), "{code}");
        assert!(code.contains("!= kotlin.Long.MIN_VALUE"), "{code}");
        assert!(!code.contains("int((0)"), "{code}");
    }
}
