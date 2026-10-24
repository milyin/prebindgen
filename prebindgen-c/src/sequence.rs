//! Array rows retain their element schema in a sequence chain, not in CWire.
use super::*;
use prebindgen_declarations::{SequenceInput, SequenceOutput};

use prebindgen_flat::WireNode;

/// Selected delivery of a logical sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSequence {
    /// All rows travel in one collection carrier.
    Buffered,
    /// Each row is delivered through the function's sequence callback.
    Callback,
}

pub(crate) fn input(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
) -> Result<SequenceInput<CWire, crate::CChain>> {
    let row = prebindgen_conversions::template_parameter(&format!("row_{wire}"));
    let mut bind = TokenStream::new();
    let mut values = Vec::new();
    for (position, field) in crate::group::row_fields(element).enumerate() {
        let name = &field.name;
        let member = format_ident!("v{position}");
        bind.extend(
            quote!(let #name=unsafe {(*((#wire.data as *const #row).add(#index))).#member};),
        );
        values.push((field.name.clone(), quote!(#name)));
    }
    Ok(SequenceInput {
        controls: vec![prebindgen_flat::Wire {
            name: wire.clone(),
            kind: CWire::Rows,
        }],
        metadata: CChain::Sequence(CSequence::Buffered),
        open: quote!(let #length=#wire.len;::prebindgen_runtime::validate_slice(#wire.data as *const #row,#length)?;),
        bind,
        validation_values: values.clone(),
        prepared_values: values.into_iter().map(|(n, v)| (n, Some(v))).collect(),
        release: quote!(),
    })
}
pub(crate) fn output(
    wire: syn::Ident,
    length: syn::Ident,
    element: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
    values: &[(syn::Ident, TokenStream)],
) -> Result<SequenceOutput<CWire, crate::CChain>> {
    let row = prebindgen_conversions::template_parameter(&format!("row_{wire}"));
    let fields = crate::group::row_values(element, values)?
        .into_iter()
        .enumerate()
        .map(|(i, (_, value))| {
            let field = format_ident!("v{i}");
            quote!(#field:#value)
        });
    Ok(SequenceOutput {
        controls: vec![prebindgen_flat::Wire {
            name: wire.clone(),
            kind: CWire::Rows,
        }],
        commit_each: false,
        fallible: false,
        metadata: CChain::Sequence(CSequence::Buffered),
        begin: quote!(let mut #wire:Vec<#row>=Vec::with_capacity(#length);),
        enter: quote!(),
        store: quote!(#wire.push(#row {#(#fields),*});),
        finish: quote!(let #wire=#wire.into_boxed_slice();),
        values: vec![(
            wire.clone(),
            quote!(__PbRows {data:if #wire.is_empty(){::core::ptr::null()}else{#wire.as_ptr().cast()},len:#wire.len()}),
        )],
        commit: quote!(let _=Box::into_raw(#wire);),
    })
}
#[allow(clippy::type_complexity)] // Pair each sequence carrier with its element tree.
/// Visit every sequence, including callback elements that need C declaration checks.
/// Nested schemas precede their parents so generated row types can refer to them.
pub(crate) fn schemas<'a>(
    node: &'a prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
    out: &mut Vec<(
        &'a prebindgen_flat::Wire<CWire>,
        &'a prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
        &'a CSequence,
    )>,
) {
    match node {
        WireNode::Leaf(_) => {}
        WireNode::Branch {
            meta: CBranch::Group(_),
            children: elements,
            ..
        }
        | WireNode::Branch {
            meta: CBranch::Alternatives,
            children: elements,
            ..
        } => {
            for child in elements {
                schemas(child, out);
            }
        }
        WireNode::Chain {
            meta: CChain::Optional(_),
            child: element,
            ..
        } => schemas(element, out),
        WireNode::Chain {
            controls,
            child: element,
            meta: CChain::Sequence(metadata),
            ..
        } => {
            schemas(element, out);
            if let [carrier] = controls.as_slice() {
                out.push((carrier, element, metadata));
            }
        }
    }
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn emit_schemas(
    schemas: Vec<(
        &prebindgen_flat::Wire<CWire>,
        &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
        &CSequence,
    )>,
    output_carriers: &std::collections::BTreeSet<String>,
    context: &CContext,
    prefix: &str,
    pointers: &[(String, syn::Type)],
    symbols: &mut std::collections::BTreeSet<String>,
    tokens: &mut TokenStream,
    header: &mut String,
) -> Result<()> {
    for (carrier, element, metadata) in schemas {
        if *metadata != CSequence::Buffered {
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
        let fields = crate::group::row_fields(element)
            .enumerate()
            .map(|(i, w)| {
                let name = format_ident!("v{i}");
                let ty = w.rust_type();
                quote!(pub #name:#ty)
            })
            .collect::<Vec<_>>();
        tokens.extend(quote!(#[repr(C)] #[derive(Clone,Copy)] pub struct #row {#(#fields),*}));
        header.push_str("typedef struct {\n");
        for (i, w) in crate::group::row_fields(element).enumerate() {
            header.push_str(&format!(
                "    {} v{i};\n",
                c_ty(w, prefix, pointers, context)
            ));
        }
        header.push_str(&format!(
            "}} {c_row};\ntypedef struct {{ {c_row} *data; size_t len; }} {c_rows};\n"
        ));
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
pub(crate) fn release(
    layout: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
    fields: &[(&Wire, TokenStream)],
    context: &CContext,
    catch_panics: bool,
) -> TokenStream {
    let mut schemas = Vec::new();
    self::schemas(layout, &mut schemas);
    let mut code = TokenStream::new();
    for (wire, value) in fields {
        match &wire.kind {
            CWire::Buffer => code.extend(quote! {if !(#value).data.is_null(){drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut((#value).data as *mut u8,(#value).len)));}}),
            CWire::Pointer { pointer, .. } => {
                let ty = &pointer.ty;
                let drop = quote!(drop(Box::from_raw((#value) as *mut #ty)););
                let drop = if catch_panics {
                    quote!(let _ = ::prebindgen_runtime::boundary(|| { #drop Ok(()) });)
                } else { drop };
                code.extend(quote! { if !(#value).is_null() { #drop } });
            }
            CWire::Rows => {
                let row=&context.row_types[&wire.name.to_string()];
                let element=schemas.iter().find(|(carrier,_,_)|carrier.name==wire.name).unwrap().1;
                let fields=crate::group::row_fields(element).enumerate().map(|(i,wire)|{let field=format_ident!("v{i}");(wire,quote!(row.#field))}).collect::<Vec<_>>();
                let child=release(element,&fields,context,catch_panics);
                code.extend(quote! {if (#value).len != 0 {let rows=Box::from_raw(std::ptr::slice_from_raw_parts_mut((#value).data as *mut #row,(#value).len));for row in rows.iter(){#child}}});
            }
            _=>{}
        }
    }
    code
}

pub(crate) fn callback_output(
    wire: syn::Ident,
    values: &[(syn::Ident, TokenStream)],
    element: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
    length: syn::Ident,
) -> Result<SequenceOutput<CWire, crate::CChain>> {
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
        controls: vec![prebindgen_flat::Wire {
            name: wire.clone(),
            kind: CWire::scalar(Primitive::Usize),
        }],
        commit_each: true,
        fallible: true, // A callback pointer must be provided; checked before source invocation.
        metadata: CChain::Sequence(CSequence::Callback),
        begin: quote!(),
        enter: quote!(),
        store: quote!(#(#prepare)* #commit unsafe { __pb_callback.expect("validated callback")(__pb_callback_context, #(#names),*); }),
        finish: quote!(),
        values: vec![(wire.clone(), quote!(#length))],
        commit: quote!(),
    })
}

/// A visitor may replace exactly one collection, optionally with its presence flag.
pub(crate) fn callback_shape(
    node: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
) -> bool {
    match node {
        WireNode::Chain {
            meta: CChain::Sequence(metadata),
            ..
        } => *metadata == CSequence::Callback,
        WireNode::Chain {
            child: element,
            meta: CChain::Optional(metadata),
            ..
        } => matches!(metadata, COptional::Flag(_)) && callback_shape(element),
        WireNode::Branch {
            meta: CBranch::Group(_),
            children: elements,
            ..
        } => {
            matches!(elements.as_slice(),[only] if callback_shape(only))
        }
        _ => false,
    }
}

/// The adapter selects its native scope before looking for a sequence.
#[allow(clippy::type_complexity)]
pub(crate) fn return_sequence(
    layout: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
) -> Option<(
    &prebindgen_flat::Wire<CWire>,
    &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
    &CSequence,
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
    layout: &prebindgen_flat::WireNode<CWire, crate::CChain, crate::CBranch>,
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
    node: &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
) -> Result<bool> {
    let sequences = outer_sequences(node);
    if !sequences
        .iter()
        .any(|(_, _, metadata)| **metadata == CSequence::Callback)
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
pub(crate) fn validate_buffered(
    node: &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
) -> Result<()> {
    match node {
        prebindgen_flat::WireNode::Leaf(_) => {}
        prebindgen_flat::WireNode::Chain {
            child: element,
            meta: CChain::Sequence(metadata),
            ..
        } => {
            if *metadata == CSequence::Callback {
                return Err(
                    "callback sequence is only supported as the function's successful return"
                        .into(),
                );
            }
            validate_buffered(element)?;
        }
        prebindgen_flat::WireNode::Chain {
            meta: CChain::Optional(_),
            child: element,
            ..
        } => validate_buffered(element)?,
        prebindgen_flat::WireNode::Branch {
            meta: CBranch::Group(_),
            children: elements,
            ..
        }
        | prebindgen_flat::WireNode::Branch {
            meta: CBranch::Alternatives,
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
    node: &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
) -> Vec<(
    &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
    &[prebindgen_flat::Wire<CWire>],
    &CSequence,
)> {
    fn visit<'a>(
        node: &'a prebindgen_flat::WireNode<CWire, CChain, CBranch>,
        out: &mut Vec<(
            &'a prebindgen_flat::WireNode<CWire, CChain, CBranch>,
            &'a [prebindgen_flat::Wire<CWire>],
            &'a CSequence,
        )>,
    ) {
        use prebindgen_flat::WireNode;
        match node {
            WireNode::Chain {
                meta: CChain::Sequence(meta),
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
    fn leaf() -> prebindgen_flat::WireNode<CWire, CChain, CBranch> {
        WireNode::Leaf(vec![prebindgen_flat::Wire {
            name: syn::parse_quote!(element),
            kind: CWire::scalar(Primitive::I32),
        }])
    }
    fn sequence(
        delivery: CSequence,
        name: &str,
        element: prebindgen_flat::WireNode<CWire, CChain, CBranch>,
    ) -> prebindgen_flat::WireNode<CWire, CChain, CBranch> {
        let kind = match delivery {
            CSequence::Buffered => CWire::Rows,
            CSequence::Callback => CWire::scalar(Primitive::Usize),
        };
        WireNode::Chain {
            controls: vec![prebindgen_flat::Wire {
                name: syn::parse_str(name).unwrap(),
                kind,
            }],
            child: Box::new(element),
            meta: CChain::Sequence(delivery),
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
            meta: CBranch::Group(CGroup::Values),
        };
        assert!(callback_delivery(&extra)
            .unwrap_err()
            .contains("additional return"));
    }
}
