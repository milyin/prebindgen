//! Array rows retain their element schema in a sequence chain, not in CWire.
use super::*;
use prebindgen_declarations::{SequenceInput, SequenceOutput};

use prebindgen_conversions::WireNode;

/// Selected delivery of a logical sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSequence {
    /// All rows travel in one collection carrier.
    Buffered,
    /// Each row is delivered through the function's sequence callback.
    Callback,
}

pub(crate) fn decode_buffer(value: Value, wire: &syn::Ident) -> TokenStream {
    let read = prebindgen_conversions::template_parameter("read_buffer");
    match value {
        Value::Text => {
            quote!(unsafe {std::str::from_utf8_unchecked(#read(#wire))})
        }
        Value::Bytes => quote!(unsafe {#read(#wire)}),
        Value::Scalar(_) => unreachable!("a buffer has text or byte contents"),
    }
}

pub(crate) fn input(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &CInputWireNode,
) -> Result<SequenceInput<CWire, crate::CInputWireChain>> {
    let row = prebindgen_conversions::template_parameter(&format!("row_{wire}"));
    let data = format_ident!("{wire}_data");
    let mut bind = TokenStream::new();
    let mut values = Vec::new();
    for (position, field) in crate::group::row_fields(element).enumerate() {
        let name = &field.name;
        let member = format_ident!("v{position}");
        bind.extend(quote!(let #name=unsafe {(*#data.add(#index)).#member};));
        values.push((field.name.clone(), quote!(#name)));
    }
    Ok(SequenceInput {
        controls: vec![prebindgen_conversions::Wire {
            name: wire.clone(),
            kind: CWire::Rows,
        }],
        metadata: CInputWireChain::Sequence,
        open: quote!(let #length=#wire.len;let #data=#wire.data as *const #row;),
        bind,
        validation_values: values,
        release: quote!(),
    })
}
pub(crate) fn output(
    wire: syn::Ident,
    length: syn::Ident,
    element: &COutputWireNode,
    values: &[(syn::Ident, TokenStream)],
) -> Result<SequenceOutput<CWire, crate::COutputWireChain>> {
    let row = prebindgen_conversions::template_parameter(&format!("row_{wire}"));
    let release_name = format_ident!("{wire}_release_row");
    let owned_fields = crate::group::row_fields(element)
        .enumerate()
        .map(|(i, wire)| {
            let field = format_ident!("v{i}");
            (wire, quote!(row.#field))
        })
        .collect::<Vec<_>>();
    let cleanup = release_fields(
        element,
        &owned_fields,
        &owned_fields,
        &|wire| prebindgen_conversions::template_parameter(&format!("row_{}", wire.name)),
        true,
    );
    let fields = crate::group::row_values(element, values)?
        .into_iter()
        .enumerate()
        .map(|(i, (_, value))| {
            let field = format_ident!("v{i}");
            quote!(#field:#value)
        });
    Ok(SequenceOutput {
        controls: vec![prebindgen_conversions::Wire {
            name: wire.clone(),
            kind: CWire::Rows,
        }],
        commit_each: true,
        fallible: false,
        metadata: COutputWireChain::Sequence(CSequence::Buffered),
        begin: quote! {
            unsafe fn #release_name(row: &#row) { #cleanup }
            let mut #wire = unsafe { ::prebindgen_runtime::OutputRows::<#row>::new(#length, #release_name) };
        },
        enter: quote!(),
        store: {
            let commit = prebindgen_conversions::template_parameter("commit_element");
            quote! {
                unsafe { #wire.write_next(#row {#(#fields),*}); }
                #commit
                unsafe { #wire.commit_row(); }
            }
        },
        finish: quote!(),
        values: vec![(
            wire.clone(),
            quote!(__PbRows {data:if #wire.is_empty(){::core::ptr::null()}else{#wire.as_ptr().cast()},len:#wire.len()}),
        )],
        commit: quote!(#wire.commit();),
    })
}
#[allow(clippy::type_complexity)] // Pair each sequence carrier with its element tree.
/// Visit every sequence, including callback elements that need C declaration checks.
/// Nested schemas precede their parents so generated row types can refer to them.
pub(crate) fn schemas<'a, Chain: group::ChainMetadata, Branch>(
    node: &'a CWireNode<Chain, Branch>,
    out: &mut Vec<(
        &'a prebindgen_conversions::Wire<CWire>,
        &'a CWireNode<Chain, Branch>,
        CSequence,
    )>,
) {
    match node {
        WireNode::Leaf(_) => {}
        WireNode::Branch {
            children: elements, ..
        } => {
            for child in elements {
                schemas(child, out);
            }
        }
        WireNode::Chain {
            meta,
            child,
            controls,
        } => {
            schemas(child, out);
            if let Some(delivery) = meta.sequence() {
                if let [carrier] = controls.as_slice() {
                    out.push((carrier, child, delivery));
                }
            }
        }
    }
}
/// An emitted C row can originate in an input or output tree.
#[derive(Clone, Copy)]
pub(crate) enum Element<'a> {
    Input(&'a CInputWireNode),
    Output(&'a COutputWireNode),
}
impl<'a> From<&'a CInputWireNode> for Element<'a> {
    fn from(node: &'a CInputWireNode) -> Self {
        Self::Input(node)
    }
}
impl<'a> From<&'a COutputWireNode> for Element<'a> {
    fn from(node: &'a COutputWireNode) -> Self {
        Self::Output(node)
    }
}
impl<'a> Element<'a> {
    pub(crate) fn fields(self) -> std::vec::IntoIter<&'a Wire> {
        match self {
            Self::Input(node) => group::row_fields(node),
            Self::Output(node) => group::row_fields(node),
        }
    }
}
#[allow(clippy::type_complexity)]
pub(crate) fn boundary_schemas<'a>(
    node: Element<'a>,
    out: &mut Vec<(&'a Wire, Element<'a>, CSequence)>,
) {
    match node {
        Element::Input(node) => {
            let mut rows = Vec::new();
            schemas(node, &mut rows);
            out.extend(
                rows.into_iter()
                    .map(|(wire, element, delivery)| (wire, Element::Input(element), delivery)),
            );
        }
        Element::Output(node) => {
            let mut rows = Vec::new();
            schemas(node, &mut rows);
            out.extend(
                rows.into_iter()
                    .map(|(wire, element, delivery)| (wire, Element::Output(element), delivery)),
            );
        }
    }
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn emit_schemas<R, E, P>(
    schemas: Vec<(&prebindgen_conversions::Wire<CWire>, Element<'_>, CSequence)>,
    output_carriers: &std::collections::BTreeSet<String>,
    context: &CContext,
    prefix: &str,
    pointers: &std::collections::BTreeMap<
        String,
        prebindgen_flat::TypeInfo<CType, (), (), R, E, P>,
    >,
    symbols: &mut std::collections::BTreeSet<String>,
    tokens: &mut TokenStream,
    header: &mut String,
) -> Result<()> {
    for (carrier, element, metadata) in schemas {
        if metadata != CSequence::Buffered {
            continue;
        }
        let row = &context.row_types[&carrier.name.to_string()];
        let base = row.to_string();
        let base = base.trim_start_matches("__PbRow_");
        let c_row = format!("{prefix}_{base}_row");
        let c_rows = format!("{prefix}_{base}_rows");
        let free_rows = format_ident!("{prefix}_{base}_rows_free");
        let is_output = output_carriers.contains(&carrier.name.to_string());
        let mut row_symbols = vec![c_row.clone(), c_rows.clone()];
        if is_output {
            row_symbols.push(free_rows.to_string());
        }
        for symbol in row_symbols {
            if !symbols.insert(symbol) {
                return Err("duplicate C collection symbol".into());
            }
        }
        let fields = element
            .fields()
            .enumerate()
            .map(|(i, w)| (format_ident!("v{i}"), w.rust_type()));
        let record = prebindgen_reprc::Record::new(row.clone(), fields)?;
        let mut definition = record.definition();
        definition
            .attrs
            .push(syn::parse_quote!(#[derive(Clone,Copy)]));
        tokens.extend(quote!(#definition));
        header.push_str("typedef struct {\n");
        for (i, w) in element.fields().enumerate() {
            header.push_str(&format!(
                "    {} v{i};\n",
                c_ty(w, prefix, pointers, context)
            ));
        }
        header.push_str(&format!(
            "}} {c_row};\ntypedef struct {{ {c_row} *data; size_t len; }} {c_rows};\n"
        ));
        if !is_output {
            header.push_str("/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */\n");
        }
        if is_output {
            header.push_str(&format!("/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */\nvoid {free_rows}({c_rows} rows);\n"));
            tokens.extend(quote! {
                    #[doc="# Safety\nThe rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields."]
                    #[no_mangle]pub unsafe extern "C" fn #free_rows(rows:__PbRows){if rows.len!=0{drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(rows.data as *mut #row,rows.len)));}}
                });
        }
    }
    Ok(())
}

/// Reclaim a committed output that will not reach its caller, including nested rows.
pub(crate) fn release<Chain: group::ChainMetadata, Branch: group::BranchMetadata>(
    layout: &CWireNode<Chain, Branch>,
    fields: &[(&Wire, TokenStream)],
    context: &CContext,
    catch_panics: bool,
) -> TokenStream {
    release_fields(
        layout,
        fields,
        fields,
        &|wire| {
            let row = &context.row_types[&wire.name.to_string()];
            quote!(#row)
        },
        catch_panics,
    )
}

// The same schema destruction is used before delivery (row-type placeholders)
// and after delivery (resolved wrapper context), including nested collections.
pub(crate) fn release_fields<Chain: group::ChainMetadata, Branch: group::BranchMetadata>(
    layout: &CWireNode<Chain, Branch>,
    fields: &[(&Wire, TokenStream)],
    conditions: &[(&Wire, TokenStream)],
    row_type: &impl Fn(&Wire) -> TokenStream,
    catch_panics: bool,
) -> TokenStream {
    let mut schemas = Vec::new();
    self::schemas(layout, &mut schemas);
    let mut code = TokenStream::new();
    for (wire, value) in fields {
        let mut release = TokenStream::new();
        match &wire.kind {
            CWire::Buffer => release.extend(quote! {if !(#value).data.is_null(){drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut((#value).data as *mut u8,(#value).len)));}}),
            CWire::Pointer { pointer, .. } if pointer.access == Access::Move => {
                let ty = &pointer.ty;
                let drop = quote!(drop(Box::from_raw((#value) as *mut #ty)););
                let drop = if catch_panics {
                    quote!(let _ = ::prebindgen_runtime::boundary(|| { #drop Ok(()) });)
                } else { drop };
                release.extend(quote! { if !(#value).is_null() { #drop } });
            }
            CWire::Owned {access:Access::Move,by_value:true,..} => {
                let drop=quote!(drop((#value).into_rust()););
                let drop=if catch_panics {quote!(let _=::prebindgen_runtime::boundary(||{#drop Ok(())});)} else {drop};
                release.extend(drop);
            }
            CWire::Tagged{value:carrier,by_value:true} if carrier.owned=>{
                let drop=&carrier.destructor;release.extend(quote!({let mut carrier=#value;#drop(::core::ptr::addr_of_mut!(carrier));}));
            }
            CWire::Rows => {
                let row=row_type(wire);
                let element=schemas.iter().find(|(carrier,_,_)|carrier.name==wire.name).unwrap().1;
                let fields=crate::group::row_fields(element).enumerate().map(|(i,wire)|{let field=format_ident!("v{i}");(wire,quote!(row.#field))}).collect::<Vec<_>>();
                let child=release_fields(element,&fields,&fields,row_type,catch_panics);
                release.extend(quote! {if (#value).len != 0 {let rows=Box::from_raw(std::ptr::slice_from_raw_parts_mut((#value).data as *mut #row,(#value).len));for row in rows.iter(){#child}}});
            }
            _=>{}
        }
        if release.is_empty() {
            continue;
        }
        let condition =
            release_condition(layout, &wire.name, conditions).unwrap_or_else(|| quote!(true));
        if condition.to_string() == "true" {
            code.extend(release);
        } else {
            // The outer selector and the leaf's resource guard are composed
            // independently; preserving that structure keeps inactive reads out.
            code.extend(quote!(#[allow(clippy::collapsible_if)] if #condition {#release}));
        }
    }
    code
}

pub(crate) fn callback_output(
    wire: syn::Ident,
    values: &[(syn::Ident, TokenStream)],
    element: &COutputWireNode,
    length: syn::Ident,
) -> Result<SequenceOutput<CWire, crate::COutputWireChain>> {
    let values = prebindgen_conversions::match_output_fields(
        crate::group::callback_parameters(element),
        values,
    )?;
    let names = (0..values.len())
        .map(|i| format_ident!("__pb_callback_value_{i}"))
        .collect::<Vec<_>>();
    let prepare = names
        .iter()
        .zip(values)
        .map(|(name, (_, value))| quote!(let #name=#value;));
    let commit = prebindgen_conversions::template_parameter("commit_element");
    Ok(SequenceOutput {
        controls: vec![prebindgen_conversions::Wire {
            name: wire.clone(),
            kind: CWire::scalar(Primitive::Usize),
        }],
        commit_each: true,
        fallible: true, // A callback pointer must be provided; checked before source invocation.
        metadata: COutputWireChain::Sequence(CSequence::Callback),
        begin: quote!(),
        enter: quote!(),
        store: quote!(#(#prepare)* #commit unsafe { __pb_callback.unwrap_unchecked()(__pb_callback_context, #(#names),*); }),
        finish: quote!(),
        values: vec![(wire.clone(), quote!(#length))],
        commit: quote!(),
    })
}

/// A visitor may replace exactly one collection, optionally with its presence flag.
pub(crate) fn callback_shape(node: &COutputWireNode) -> bool {
    match node {
        WireNode::Chain {
            meta: COutputWireChain::Sequence(metadata),
            ..
        } => *metadata == CSequence::Callback,
        WireNode::Chain {
            child: element,
            meta: COutputWireChain::Optional(metadata),
            ..
        } => matches!(metadata, COptional::Flag(_)) && callback_shape(element),
        WireNode::Branch {
            meta: COutputWireBranch::Enum(_),
            ..
        } => false,
        WireNode::Branch {
            children: elements, ..
        } => {
            matches!(elements.as_slice(),[only] if callback_shape(only))
        }
        _ => false,
    }
}

/// The adapter selects its native scope before looking for a sequence.
#[allow(clippy::type_complexity)]
pub(crate) fn return_sequence(
    layout: &COutputWireNode,
) -> Option<(
    &prebindgen_conversions::Wire<CWire>,
    &COutputWireNode,
    CSequence,
)> {
    outer_sequences(layout)
        .into_iter()
        .next()
        .and_then(|(element, controls, metadata)| match controls {
            [carrier] => Some((carrier, element, metadata)),
            _ => None,
        })
}
pub(crate) fn callback_target(layout: &COutputWireNode) -> Result<syn::Ident> {
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
pub(crate) fn callback_delivery(node: &COutputWireNode) -> Result<bool> {
    let sequences = outer_sequences(node);
    if !sequences
        .iter()
        .any(|(_, _, metadata)| *metadata == CSequence::Callback)
    {
        return Ok(false);
    }
    callback_target(node)?;
    if !callback_shape(node) {
        return Err("callback sequence cannot be combined with additional return values".into());
    }
    Ok(true)
}

/// Function sequence callbacks are not available in parameters, error channels or rows.
pub(crate) fn validate_buffered<Chain: group::ChainMetadata, Branch>(
    node: &CWireNode<Chain, Branch>,
) -> Result<()> {
    match node {
        prebindgen_conversions::WireNode::Leaf(_) => {}
        prebindgen_conversions::WireNode::Chain { meta, child, .. } => {
            if meta.sequence() == Some(CSequence::Callback) {
                return Err(
                    "callback sequence is only supported as the function's successful return"
                        .into(),
                );
            }
            validate_buffered(child)?;
        }
        prebindgen_conversions::WireNode::Branch { children, .. } => {
            for child in children {
                validate_buffered(child)?;
            }
        }
    }
    Ok(())
}

// A sequence owns a separate row scope: find the outer occurrences only.
#[allow(clippy::type_complexity)] // Borrow the element schema, controls and delivery policy together.
fn outer_sequences(
    node: &COutputWireNode,
) -> Vec<(
    &COutputWireNode,
    &[prebindgen_conversions::Wire<CWire>],
    CSequence,
)> {
    fn visit<'a>(
        node: &'a COutputWireNode,
        out: &mut Vec<(
            &'a COutputWireNode,
            &'a [prebindgen_conversions::Wire<CWire>],
            CSequence,
        )>,
    ) {
        use prebindgen_conversions::WireNode;
        match node {
            WireNode::Chain {
                meta: COutputWireChain::Sequence(meta),
                child,
                controls,
            } => out.push((child, controls, *meta)),
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

// A carrier is a scope boundary. Its own destructor interprets its children.
fn release_condition<Chain: group::ChainMetadata, Branch: group::BranchMetadata>(
    node: &CWireNode<Chain, Branch>,
    target: &syn::Ident,
    fields: &[(&Wire, TokenStream)],
) -> Option<TokenStream> {
    fn enclose(predicate: TokenStream, child: TokenStream) -> TokenStream {
        if child.to_string() == "true" {
            predicate
        } else {
            quote!((#predicate) && (#child))
        }
    }
    let value = |name: &syn::Ident| {
        fields
            .iter()
            .find(|(w, _)| w.name == *name)
            .map(|(_, v)| v.clone())
    };
    let controls = match node {
        WireNode::Leaf(w) => w,
        WireNode::Chain { controls, .. } | WireNode::Branch { controls, .. } => controls,
    };
    if controls.iter().any(|w| w.name == *target) {
        return Some(quote!(true));
    }
    match node {
        WireNode::Leaf(_) => None,
        WireNode::Chain { meta: encoding, .. }
            if (encoding.sequence().is_some() || encoding.tagged().is_some()) =>
        {
            None
        }
        WireNode::Chain {
            meta: encoding,
            child,
            ..
        } if (encoding.optional().is_some()) => {
            let optional = encoding.optional().expect("matched metadata");

            let child = release_condition(child, target, fields)?;
            let discriminator = value(optional.discriminator())?;
            let present = optional
                .niche()
                .map(|n| n.present(discriminator.clone()))
                .unwrap_or_else(|| quote!(#discriminator==1));
            Some(enclose(present, child))
        }
        WireNode::Branch {
            meta: encoding,
            controls,
            children,
        } if (encoding.enumeration().is_some() || encoding.is_alternatives()) => {
            let tag = value(&controls[0].name)?;
            children.iter().enumerate().find_map(|(i, child)| {
                let child = release_condition(child, target, fields)?;
                let i = i as i32;
                Some(enclose(quote!((#tag)==#i), child))
            })
        }
        WireNode::Chain { .. } => panic!("unsupported C cleanup chain metadata"),
        WireNode::Branch { children, .. } => children
            .iter()
            .find_map(|child| release_condition(child, target, fields)),
    }
}

#[cfg(test)]
mod delivery_tests {
    use super::*;
    fn leaf() -> COutputWireNode {
        WireNode::Leaf(vec![prebindgen_conversions::Wire {
            name: syn::parse_quote!(element),
            kind: CWire::scalar(Primitive::I32),
        }])
    }
    fn sequence(delivery: CSequence, name: &str, element: COutputWireNode) -> COutputWireNode {
        let kind = match delivery {
            CSequence::Buffered => CWire::Rows,
            CSequence::Callback => CWire::scalar(Primitive::Usize),
        };
        WireNode::Chain {
            controls: vec![prebindgen_conversions::Wire {
                name: syn::parse_str(name).unwrap(),
                kind,
            }],
            child: Box::new(element),
            meta: COutputWireChain::Sequence(delivery),
        }
    }
    #[test]
    fn delivery_is_explicit_and_callback_scope_is_checked() {
        let visitor = sequence(CSequence::Callback, "visitor", leaf());
        assert!(callback_delivery(&visitor).unwrap());
        assert!(validate_buffered(&visitor)
            .unwrap_err()
            .contains("successful return"));
        let packed = sequence(CSequence::Buffered, "rows", leaf());
        assert!(!callback_delivery(&packed).unwrap());
        validate_buffered(&packed).unwrap();
        let nested = sequence(CSequence::Buffered, "rows", visitor.clone());
        assert!(crate::group::validate_layout(&nested)
            .unwrap_err()
            .contains("successful return"));
        let extra = WireNode::Branch {
            controls: vec![],
            children: vec![visitor, leaf()],
            meta: COutputWireBranch::Values,
        };
        assert!(callback_delivery(&extra)
            .unwrap_err()
            .contains("additional return"));
    }
}
