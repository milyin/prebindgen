//! Failure-only ownership cleanup for ordinary Kotlin callback result trees.
use super::*;
use crate::kotlin::JniNode;
use prebindgen_flat::{Wire, WireNode};

/// Generated Kotlin creates private row arrays, retained by the JNI local frame.
/// Internal native callers must keep those wire values stable through decoding
/// and rollback, as required by the generated preparation/ownership protocol.
/// Bulk carriers retain their existing cleanup paths.
pub(crate) fn cleanup(
    node: &JniNode,
    context: &JniContext,
    root: &syn::Ident,
) -> Result<Option<TokenStream>> {
    // Find ordinary collections transferring ownership, rejecting any packed
    // subtree. Fixed handles alone already have native-only cleanup guards.
    fn rows(node: &JniNode) -> Option<bool> {
        match node {
            WireNode::Leaf(_) => Some(false),
            WireNode::Chain {
                meta: JniChain::Sequence(JniSequence::Buffered),
                child,
                ..
            } => {
                let nested = rows(child)?;
                Some(nested || kotlin::input_handles(child, true))
            }
            WireNode::Chain {
                meta: JniChain::Sequence(_) | JniChain::Bulk(_),
                ..
            } => None,
            WireNode::Chain { child, .. } => rows(child),
            WireNode::Branch { children, .. } => children
                .iter()
                .try_fold(false, |found, child| Some(rows(child)? || found)),
        }
    }
    let fields = group::argument_parameters(node).collect::<Vec<_>>();
    if rows(node) != Some(true) {
        return Ok(None);
    }
    // A single field is already a decoded native local. Composite results stay
    // in their private root array, just like rows at any deeper nesting level.
    let composite = fields.len() != 1;
    let scan = receive(node, context, composite.then_some(root), &fields)?;
    Ok(Some(if composite {
        quote!(if !#root.is_null() { #scan })
    } else {
        scan
    }))
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
            // Keep sibling reference columns bounded even in wide result trees.
            let cleanup_read_frame = unsafe { ::prebindgen_jni_runtime::LocalFrame::new(&#env, 8) }?;
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
            drop(cleanup_read_frame);
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
    if !kotlin::input_handles(node, true) {
        return Ok(quote!());
    }
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
            let child_row = format_ident!("{carrier}_cleanup_row");
            let index = format_ident!("{carrier}_cleanup_index");
            let child_fields = group::row_fields(child).collect::<Vec<_>>();
            let capacity = i32::try_from(child_fields.len() + 8)
                .map_err(|_| "too many callback row fields")?;
            let scan = receive(child, context, Some(&child_row), &child_fields)?;
            let scan = read(
                context,
                &child_row,
                quote!(::prebindgen_jni_runtime::row_rows(&mut #env, &#carrier, #index)),
                quote!(if !#child_row.is_null() { #scan }),
            );
            let scan = quote!(if !#carrier.is_null() {
                let length = #env.get_array_length(&#carrier).map_err(|e| e.to_string())? as usize;
                for #index in 0..length {
                    let frame = unsafe { ::prebindgen_jni_runtime::LocalFrame::new(&#env, #capacity) }?;
                    #scan
                    drop(frame);
                }
            });
            field(context, row, fields, carrier, scan)?
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
