//! Failure-only ownership cleanup for flat rows returned by Kotlin callbacks.
use super::*;
use crate::kotlin::JniNode;
use prebindgen_flat::{Wire, WireNode};

/// Generated Kotlin creates private row arrays, retained by the JNI local frame.
/// Nested collections and bulk carriers retain their existing cleanup paths.
pub(crate) fn cleanup(node: &JniNode, context: &JniContext) -> Result<Option<TokenStream>> {
    fn flat(node: &JniNode, row: bool) -> bool {
        match node {
            WireNode::Leaf(_) => true,
            WireNode::Chain {
                meta: JniChain::Sequence(JniSequence::Buffered),
                child,
                ..
            } => !row && flat(child, true),
            WireNode::Chain {
                meta: JniChain::Sequence(_) | JniChain::Bulk(_),
                ..
            } => false,
            WireNode::Chain { child, .. } => flat(child, row),
            WireNode::Branch { children, .. } => children.iter().all(|child| flat(child, row)),
        }
    }
    let fields = group::argument_parameters(node).collect::<Vec<_>>();
    if !matches!(fields.as_slice(), [field] if matches!(field.kind, JniWire::Rows { .. }))
        || !kotlin::input_handles(node, true)
        || !flat(node, false)
    {
        return Ok(None);
    }
    receive(node, context, None, &[]).map(Some)
}

// Each fallible JNI read has its own rollback boundary, so a missing or invalid
// field cannot prevent cleanup of other owners in the current or later rows.
fn read(
    context: &JniContext,
    name: &syn::Ident,
    expression: TokenStream,
    next: TokenStream,
) -> TokenStream {
    let env = &context.environment;
    quote! {
        ::prebindgen_jni_runtime::reclaim_output(&mut #env, |#env| {
            let mut #env = #env;
            match #expression {
                Ok(#name) => { #next }
                Err(error) => {
                    // Format errors permit sibling cleanup. A JVM failure that
                    // prevents reading trusted storage uses reclaim_output's
                    // existing fatal policy instead of leaking native owners.
                    if #env.exception_check().map_err(|e| e.to_string())? {
                        return Err(error);
                    }
                    ::prebindgen_runtime::report_callback_cleanup_error(&error);
                }
            }
            Ok(())
        });
    }
}
fn field(
    context: &JniContext,
    row: Option<&syn::Ident>,
    fields: &[&Wire<JniWire>],
    name: &syn::Ident,
    next: TokenStream,
) -> Result<TokenStream> {
    let Some(row) = row else { return Ok(next) };
    let (index, wire) = fields
        .iter()
        .enumerate()
        .find(|(_, wire)| wire.name == *name)
        .ok_or("missing callback cleanup field")?;
    Ok(read(
        context,
        name,
        {
            let read = wire.kind.read_array_element(&quote!(#row), index);
            // A malformed short row can still contain valid preceding owners.
            // Skip absent columns without raising a second JVM exception.
            let env = &context.environment;
            quote!(if #index < #env.get_array_length(&#row).map_err(|e| e.to_string())? as usize {
                #read
            } else {
                Err("callback cleanup field is absent".to_owned())
            })
        },
        next,
    ))
}
fn receive(
    node: &JniNode,
    context: &JniContext,
    row: Option<&syn::Ident>,
    fields: &[&Wire<JniWire>],
) -> Result<TokenStream> {
    Ok(match node {
        WireNode::Leaf(wires) => {
            let mut code = TokenStream::new();
            for wire in wires {
                if let JniWire::Handle { handle, .. } = &wire.kind {
                    if handle.access == Access::Move {
                        let name = &wire.name;
                        let ty = &handle.ty;
                        code.extend(field(context, row, fields, name, quote!(unsafe {
                            callback_return_owners.receive(#name as *mut ::prebindgen_jni_runtime::Allocation<#ty>);
                        }))?);
                    }
                }
            }
            code
        }
        WireNode::Chain {
            meta: JniChain::Sequence(JniSequence::Buffered),
            controls,
            child,
        } => {
            let carrier = &controls[0].name;
            let env = &context.environment;
            let row = format_ident!("{carrier}_cleanup_row");
            let fields = group::row_fields(child).collect::<Vec<_>>();
            let capacity =
                i32::try_from(fields.len() + 8).map_err(|_| "too many callback row fields")?;
            let scan = receive(child, context, Some(&row), &fields)?;
            let scan = read(
                context,
                &row,
                quote!(::prebindgen_jni_runtime::row_rows(&mut #env, &#carrier, index)),
                quote!(if !#row.is_null() { #scan }),
            );
            quote!(if !#carrier.is_null() {
                let length = #env.get_array_length(&#carrier).map_err(|e| e.to_string())? as usize;
                for index in 0..length {
                    let frame = unsafe { ::prebindgen_jni_runtime::LocalFrame::new(&#env, #capacity) }?;
                    #scan
                    drop(frame);
                }
            })
        }
        WireNode::Chain {
            meta: JniChain::Optional(optional),
            child,
            ..
        } => {
            let scan = receive(child, context, row, fields)?;
            if scan.is_empty() {
                return Ok(scan);
            }
            let name = optional.discriminator();
            let present = if let Some(niche) = optional.niche() {
                let wire = group::argument_parameters(child)
                    .find(|w| w.name == *name)
                    .ok_or("missing callback cleanup discriminator")?;
                niche.present(&wire.kind, quote!(#name))
            } else {
                quote!(#name == 1)
            };
            field(context, row, fields, name, quote!(if #present { #scan }))?
        }
        WireNode::Branch {
            meta: JniBranch::Alternatives,
            controls,
            children,
        } => {
            let selector = &controls[0].name;
            let mut code = TokenStream::new();
            for (index, child) in children.iter().enumerate() {
                let scan = receive(child, context, row, fields)?;
                let index = index as i32;
                code.extend(quote!(if #selector == #index { #scan }));
            }
            field(context, row, fields, selector, code)?
        }
        WireNode::Branch { children, .. } => {
            let mut code = TokenStream::new();
            for child in children {
                code.extend(receive(child, context, row, fields)?);
            }
            code
        }
        _ => return Err("unsupported callback collection cleanup shape".into()),
    })
}
