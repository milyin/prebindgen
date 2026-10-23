//! Array rows retain their element schema in TransportNode::Sequence, not in CWire.
use super::*;
use prebindgen_declarations::{SequenceInput, SequenceOutput};

use prebindgen_conversions::{TransportLayout, TransportNode};

pub(crate) fn input(
    wire: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    element: &TransportLayout<CWire, CGroup>,
) -> Result<SequenceInput<CWire>> {
    let row = prebindgen_conversions::template_parameter(&format!("row_{wire}"));
    let mut bind = TokenStream::new();
    let mut values = Vec::new();
    for (position, field) in element.iter().enumerate() {
        let name = &field.name;
        let member = format_ident!("v{position}");
        bind.extend(
            quote!(let #name=unsafe {(*((#wire.data as *const #row).add(#index))).#member};),
        );
        values.push(quote!(#name));
    }
    Ok(SequenceInput {
        carrier: CWire::Rows,
        open: quote!(let #length=#wire.len;::prebindgen_runtime::validate_slice(#wire.data as *const #row,#length)?;),
        bind,
        validation_values: values.clone(),
        prepared_values: values.into_iter().map(Some).collect(),
        release: quote!(),
    })
}
pub(crate) fn output(
    wire: syn::Ident,
    length: syn::Ident,
    element: &TransportLayout<CWire, CGroup>,
    values: &[TokenStream],
) -> Result<SequenceOutput<CWire>> {
    let row = prebindgen_conversions::template_parameter(&format!("row_{wire}"));
    if element.len() != values.len() {
        return Err("collection row/value count differs".into());
    }
    let fields = values.iter().enumerate().map(|(i, value)| {
        let field = format_ident!("v{i}");
        quote!(#field:#value)
    });
    Ok(SequenceOutput {
        commit_each: false,
        fallible: false,
        carrier: CWire::Rows,
        begin: quote!(let mut #wire:Vec<#row>=Vec::with_capacity(#length);),
        enter: quote!(),
        store: quote!(#wire.push(#row {#(#fields),*});),
        finish: quote!(let #wire=#wire.into_boxed_slice();),
        value: quote!(__PbRows {data:if #wire.is_empty(){::core::ptr::null()}else{#wire.as_ptr().cast()},len:#wire.len()}),
        commit: quote!(let _=Box::into_raw(#wire);),
    })
}
pub(crate) fn schemas<'a>(
    layout: &'a TransportLayout<CWire, CGroup>,
    out: &mut Vec<(&'a Wire, &'a TransportLayout<CWire, CGroup>)>,
) {
    for node in layout.nodes() {
        match node {
            TransportNode::Wire(_) => {}
            TransportNode::Alternatives { alternatives, .. } => {
                for arm in alternatives {
                    schemas(arm, out);
                }
            }
            TransportNode::Group { children, .. } => schemas(children, out),
            TransportNode::Optional(c) => schemas(&c.element, out),
            TransportNode::Sequence(c) => {
                schemas(&c.element, out);
                if let Some(carrier) = c.carrier() {
                    out.push((carrier, &c.element));
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_schemas(
    schemas: Vec<(&Wire, &TransportLayout<CWire, CGroup>)>,
    output_carriers: &std::collections::BTreeSet<String>,
    context: &CContext,
    prefix: &str,
    pointers: &[(String, syn::Type)],
    symbols: &mut std::collections::BTreeSet<String>,
    tokens: &mut TokenStream,
    header: &mut String,
) -> Result<()> {
    for (carrier, element) in schemas {
        if !matches!(carrier.kind, CWire::Rows) {
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
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let name = format_ident!("v{i}");
                let ty = w.rust_type();
                quote!(pub #name:#ty)
            })
            .collect::<Vec<_>>();
        tokens.extend(quote!(#[repr(C)] #[derive(Clone,Copy)] pub struct #row {#(#fields),*}));
        header.push_str("typedef struct {\n");
        for (i, w) in element.iter().enumerate() {
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
    layout: &TransportLayout<CWire, CGroup>,
    values: &[TokenStream],
    context: &CContext,
    catch_panics: bool,
) -> TokenStream {
    let mut schemas = Vec::new();
    self::schemas(layout, &mut schemas);
    let mut code = TokenStream::new();
    for (wire, value) in layout.iter().zip(values) {
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
                let element=schemas.iter().find(|(carrier,_)|carrier.name==wire.name).unwrap().1;
                let fields=element.iter().enumerate().map(|(i,_)|{let field=format_ident!("v{i}");quote!(row.#field)}).collect::<Vec<_>>();
                let child=release(element,&fields,context,catch_panics);
                code.extend(quote! {if (#value).len != 0 {let rows=Box::from_raw(std::ptr::slice_from_raw_parts_mut((#value).data as *mut #row,(#value).len));for row in rows.iter(){#child}}});
            }
            _=>{}
        }
    }
    code
}

pub(crate) fn callback_output(values: &[TokenStream], length: syn::Ident) -> SequenceOutput<CWire> {
    let names = (0..values.len())
        .map(|i| format_ident!("__pb_callback_value_{i}"))
        .collect::<Vec<_>>();
    let prepare = names
        .iter()
        .zip(values)
        .map(|(name, value)| quote!(let #name=#value;));
    let commit = prebindgen_conversions::template_parameter("commit_element");
    SequenceOutput {
        commit_each: true,
        fallible: true, // A callback pointer must be provided; checked before source invocation.
        carrier: CWire::scalar(Primitive::Usize),
        begin: quote!(),
        enter: quote!(),
        store: quote!(#(#prepare)* #commit unsafe { __pb_callback.expect("validated callback")(__pb_callback_context, #(#names),*); }),
        finish: quote!(),
        value: quote!(#length),
        commit: quote!(),
    }
}

/// A visitor may replace exactly one collection, optionally with its presence flag.
pub(crate) fn callback_shape(layout: &TransportLayout<CWire, CGroup>) -> bool {
    fn one(nodes: &[TransportNode<CWire, CGroup>]) -> bool {
        match nodes {
            [TransportNode::Sequence(c)] => c.carrier().is_some_and(|carrier| {
                matches!(
                    carrier.kind,
                    CWire::Scalar {
                        primitive: Primitive::Usize,
                        ..
                    }
                )
            }),
            [TransportNode::Optional(c)] => match (&c.representation, &c.metadata) {
                (
                    prebindgen_conversions::TransportRepresentation::Inline { controls },
                    Some(CGroup::Optional(o)),
                ) if o.niche.is_none() => {
                    matches!(controls.as_slice(),[flag] if flag.name==o.discriminator)
                        && one(c.element.nodes())
                }
                _ => false,
            },
            [TransportNode::Group { children, .. }] => one(children.nodes()),
            _ => false,
        }
    }
    one(layout.nodes())
}
