//! Decode and construct collection elements directly into their final storage.
use super::*;

use prebindgen_flat::SequenceView;
use prebindgen_flat::WireNode;
use quote::ToTokens;

/// Adapter-supplied array input transport. Element conversion remains shared.
pub struct SequenceInput<W: prebindgen_flat::WireKind, G> {
    /// Adapter policy for interpreting `controls`; must not own wire descriptors.
    pub metadata: G,
    /// Supporting boundary values, stored on the resulting sequence node.
    pub controls: Vec<Wire<W>>,
    /// Validate the outer input values and make the supplied length identifier
    /// available, either as a boundary parameter or a local binding.
    pub open: TokenStream,
    /// Per-row setup, including any locals used by `validation_values`.
    pub bind: TokenStream,
    /// Raw element expressions keyed by binding name, used during validation.
    pub validation_values: Vec<(syn::Ident, TokenStream)>,
    /// Per-row cleanup after construction. Generated code runs this before
    /// propagating an ordinary conversion error from the row.
    pub release: TokenStream,
}
/// Adapter-supplied array output transport. Owners are committed only after delivery.
pub struct SequenceOutput<W: prebindgen_flat::WireKind, G> {
    /// Transfer each element inside `store`, after capturing its prepared wire
    /// values, into a rollback-capable carrier or to a foreign visitor. A carrier
    /// must reclaim transferred owners if outer output delivery fails.
    /// Every fallible row store must precede the marker. Element commit code is
    /// evaluated in an infallible, unit-returning closure.
    /// `store` must contain exactly one
    /// `template_parameter("commit_element")` marker when true. When false, all
    /// element owners remain in Rust until the outer slot commits.
    pub commit_each: bool,
    /// Whether preparing or delivering the array transport can return a binding
    /// error, in addition to failures from converting its elements.
    pub fallible: bool,
    /// Adapter delivery policy; must not own wire descriptors or child trees.
    pub metadata: G,
    /// Supporting boundary values, stored on the resulting sequence node.
    pub controls: Vec<Wire<W>>,
    /// Initialize delivery state, with the supplied element count available.
    pub begin: TokenStream,
    /// Per-row setup before element conversion.
    pub enter: TokenStream,
    /// Store or deliver one prepared element's wire values.
    pub store: TokenStream,
    /// Finish preparing the result after every element has been delivered.
    pub finish: TokenStream,
    /// Named expressions for every outer value. The adapter determines ABI order.
    pub values: Vec<(syn::Ident, TokenStream)>,
    /// Complete ownership transfer of the outer carrier, such as a row allocation.
    pub commit: TokenStream,
}

fn replace(code: &TokenStream, values: &[(syn::Ident, TokenStream)]) -> Result<TokenStream> {
    prebindgen_conversions::validate_value_bindings(values)?;
    let replacements = values
        .iter()
        .map(|(name, value)| {
            Ok((
                name.to_string(),
                syn::parse2::<syn::Expr>(quote!((#value))).map_err(|e| e.to_string())?,
            ))
        })
        .collect::<Result<std::collections::BTreeMap<_, _>>>()?;
    struct Replace<'a> {
        values: &'a std::collections::BTreeMap<String, syn::Expr>,
    }
    impl syn::visit_mut::VisitMut for Replace<'_> {
        fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
            if let syn::Expr::Path(p) = expr {
                if p.qself.is_none() && p.path.segments.len() == 1 {
                    if let Some(value) = self.values.get(&p.path.segments[0].ident.to_string()) {
                        *expr = value.clone();
                        return;
                    }
                }
            }
            syn::visit_mut::visit_expr_mut(self, expr);
            // An argument already has its own expression boundary. Keep grouping
            // elsewhere (notably when replacing the operand of `&wire`).
            if let syn::Expr::Match(value) = expr {
                if let syn::Expr::Paren(paren) = &*value.expr {
                    value.expr = paren.expr.clone();
                }
            }
            let args = match expr {
                syn::Expr::Call(call) => Some(&mut call.args),
                syn::Expr::MethodCall(call) => Some(&mut call.args),
                _ => None,
            };
            if let Some(args) = args {
                for arg in args {
                    if let syn::Expr::Paren(paren) = arg {
                        *arg = *paren.expr.clone();
                    }
                }
            }
        }
    }
    let mut expr: syn::Expr = syn::parse2(quote!({ #code })).map_err(|e| e.to_string())?;
    let mut visitor = Replace {
        values: &replacements,
    };
    syn::visit_mut::VisitMut::visit_expr_mut(&mut visitor, &mut expr);
    let syn::Expr::Block(block) = expr else {
        unreachable!()
    };
    Ok(block
        .block
        .stmts
        .iter()
        .map(ToTokens::to_token_stream)
        .collect())
}

#[allow(clippy::too_many_arguments)]
/// Assemble one sequence input from a prepared row conversion.
pub fn input<C: Codec>(
    codec: &C,
    ty: &Type,
    extra: &C::Extra,
    wire: syn::Ident,
    var: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    active: &Condition,
    child: CodecInput<C>,
    bulk: bool,
) -> Result<CodecInput<C>> {
    let (element, view) = prebindgen_flat::sequence_parts(ty)
        .ok_or("sequence recipe requires Vec<T>, &[T] or &Vec<T>")?;
    if view == SequenceView::Mutable {
        return Err("mutable collections require explicit write-back support".into());
    }
    if matches!(element, Type::Reference(_)) {
        return Err("collection reference elements are unsupported; use owned elements".into());
    }
    codec.validate_input(&child)?;
    if bulk {
        require_bulk(codec, Direction::Input)?;
    }
    let transport = codec.sequence_input(SequenceInputRequest {
        extra,
        view,
        wire: wire.clone(),
        length: length.clone(),
        index: index.clone(),
        element: &child.node,
        active,
        representation: if bulk {
            Representation::Bulk
        } else {
            Representation::Ordinary
        },
    })?;
    if transport.validation_values.is_empty() && !bulk {
        return Err("collection elements must have at least one input binding".into());
    }
    prebindgen_conversions::validate_value_bindings(&transport.validation_values)?;
    for (name, _) in &transport.validation_values {
        if !prebindgen_flat::wire_definitions(std::iter::once(&child.node))
            .any(|wire| wire.name == *name)
        {
            return Err(format!("unknown collection validation binding `{name}`"));
        }
    }
    let target = format_ident!("{wire}_row_value");
    let mut child = child;
    let child_storage = std::mem::take(&mut child.storage);
    let conversion = child.bind(&target, quote!(Ok(#target)))?;
    let conversion = replace(&conversion, &transport.validation_values)?;
    let child_storage = replace(&child_storage, &transport.validation_values)?;
    let fallible = codec.validates_inputs() || child.fallible;
    let open = transport.open;
    let bind = transport.bind;
    let release = transport.release;
    let condition = active.rust();
    let array = match ty {
        syn::Type::Reference(r) => r.elem.as_ref(),
        _ => ty,
    };
    let length_check = if let syn::Type::Array(array) = array {
        let expected = &array.len;
        if codec.validates_inputs() {
            quote!(if #length != #expected { return Err("wrong fixed array length".into()); })
        } else {
            TokenStream::new()
        }
    } else {
        TokenStream::new()
    };
    let to_array = if view == SequenceView::Array {
        if codec.validates_inputs() {
            quote!(let #var: #ty = #var.try_into().map_err(|_|"wrong fixed array length".to_string())?;)
        } else {
            quote!(let #var:#ty=unsafe {#var.try_into().unwrap_unchecked()};)
        }
    } else {
        TokenStream::new()
    };
    let value = match view {
        SequenceView::Owned | SequenceView::Array => quote!(#var),
        SequenceView::ArrayReference => {
            if codec.validates_inputs() {
                quote!(<#ty>::try_from(#var.as_slice()).map_err(|_|"wrong fixed array length".to_string())?)
            } else {
                quote!(unsafe {&*(#var.as_ptr() as *const _)})
            }
        }
        SequenceView::Vector => quote!(&#var),
        SequenceView::Slice => quote!(#var.as_slice()),
        SequenceView::Mutable => unreachable!(),
    };
    let element_value = format_ident!("{wire}_element_built");
    let borrowed = matches!(
        view,
        SequenceView::ArrayReference | SequenceView::Vector | SequenceView::Slice
    );
    let declaration = quote!(let mut #var:Vec<#element>=Vec::new(););
    let declare = if borrowed {
        quote!()
    } else {
        declaration.clone()
    };
    let direct = quote! {
            #declare
            if #condition { #open #length_check
                #var.reserve_exact(#length);
                for #index in 0..#length {
                    #bind
                    #[allow(clippy::redundant_closure_call)]
                    let #element_value=(|| -> ::core::result::Result<#element,String> {
                        #child_storage #conversion
                    })();
                    #release
                    let element = #element_value?;
                    // reserve_exact(length) covers the whole loop; update len after
                    // each initialized element so ordinary Rust cleanup is sufficient.
                    // Pointer addition is also valid (and a no-op) for ZSTs.
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = #var.len();
                        #var.as_mut_ptr().add(initialized).write(element);
                        #var.set_len(initialized + 1);
                    }
                }
            }
    };
    let mut plan = InputPlan::new(
        WireNode::Chain {
            controls: transport.controls,
            child: Box::new(child.node),
            meta: transport.metadata,
        },
        quote!(#direct #to_array),
        value,
    );
    plan.fallible = fallible;
    if borrowed {
        plan.storage = declaration;
    }
    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
/// Assemble one sequence output with the selected transport and delivery.
pub fn output<C: Codec>(
    codec: &C,
    ty: &Type,
    extra: &C::Extra,
    wire: syn::Ident,
    element: syn::Ident,
    length: syn::Ident,
    index: syn::Ident,
    value: TokenStream,
    borrowed: bool,
    mode: ProjectionMode,
    child: CodecOutput<C>,
    delivery: crate::SequenceDelivery,
    bulk: bool,
) -> Result<CodecOutput<C>> {
    let (element_type, view) =
        prebindgen_flat::sequence_parts(ty).ok_or("sequence recipe requires a collection")?;
    if matches!(element_type, Type::Reference(_)) {
        return Err("collection reference elements are unsupported; use owned elements".into());
    }
    if view == SequenceView::Mutable {
        return Err("mutable collection outputs are unsupported".into());
    }
    if mode == ProjectionMode::Owned && (borrowed || !view.is_owned()) {
        return Err("owned collection projection cannot consume a borrowed collection".into());
    }
    validate_nested_output(&child)?;
    codec.validate_output(&child)?;
    if child.values.is_empty() && !bulk {
        return Err("collection elements must have at least one wire".into());
    }
    if !child.delivery.statements.is_empty() {
        return Err("collection element delivery belongs to the outer slot".into());
    }
    if bulk {
        require_bulk(codec, Direction::Output)?;
    }
    let transport = codec.sequence_output(SequenceOutputRequest {
        extra,
        wire: wire.clone(),
        length: length.clone(),
        index: index.clone(),
        element: &child.outcomes.success,
        values: &child.values,
        delivery,
        representation: if bulk {
            Representation::Bulk
        } else {
            Representation::Ordinary
        },
    })?;
    let owner = format_ident!("{wire}_source");
    let commits = format_ident!("{wire}_commits");
    let vector = matches!(view, SequenceView::Owned | SequenceView::Vector);
    // Use standard operations explicitly: extension traits in the including
    // module must not replace length/iteration and violate native row extents.
    let (binding, iterator, extent) = if mode == ProjectionMode::Owned {
        let extent = if vector {
            quote!(::std::vec::Vec::len(&#owner))
        } else {
            quote!(<[#element_type]>::len(&#owner))
        };
        (
            value,
            quote!(::core::iter::IntoIterator::into_iter(#owner)),
            extent,
        )
    } else {
        let slice = quote!(::core::convert::AsRef::<[#element_type]>::as_ref(#owner));
        (
            quote!(&(#value)),
            quote!(<[#element_type]>::iter(#slice)),
            quote!(<[#element_type]>::len(#slice)),
        )
    };
    // Fix the binding's type, rather than relying on a custom Vec name or
    // Deref target to give len and iteration the standard collection semantics.
    let binding = if matches!(view, SequenceView::Owned | SequenceView::Vector) {
        let vector = quote!(::std::vec::Vec<#element_type>);
        let vector = if mode == ProjectionMode::Owned {
            vector
        } else {
            quote!(&#vector)
        };
        quote!(let #owner: #vector = #binding;)
    } else {
        quote!(let #owner = #binding;)
    };
    let prepare = child.statements;
    let child_commit = child.commit;
    let SequenceOutput {
        commit_each,
        fallible,
        metadata,
        controls,
        begin,
        enter,
        store,
        finish,
        values,
        commit,
    } = transport;
    let store = if commit_each {
        let count = std::cell::Cell::new(0);
        let store = prebindgen_conversions::render_template(
            &store,
            &|name| {
                if name == "commit_element" {
                    count.set(count.get() + 1);
                    Ok(quote!(
                        #[allow(clippy::redundant_closure_call, clippy::let_unit_value, clippy::unused_unit)]
                        let _: () = (|| -> () { #child_commit })();
                    ))
                } else {
                    Ok(prebindgen_conversions::template_parameter(name))
                }
            },
            &|count| Ok(prebindgen_conversions::record_input_transfer(count.clone())),
        )?;
        if count.get() != 1 {
            return Err("per-element delivery requires exactly one commit_element marker".into());
        }
        store
    } else {
        quote!(#store #commits.push(move || {#child_commit});)
    };
    let declarations = if commit_each {
        quote!()
    } else {
        quote!(let mut #commits=Vec::with_capacity(#length);)
    };
    let commit_elements = if commit_each {
        quote!()
    } else {
        quote!(for commit in #commits {commit();})
    };
    let out: CodecOutput<C> = OutputPlan {
        fallible: child.fallible || fallible,
        context: Default::default(),
        delivery: Default::default(),
        outcomes: prebindgen_flat::Outcomes {
            success: WireNode::Chain {
                controls,
                child: Box::new(child.outcomes.success),
                meta: metadata,
            },
            source_error: None,
            binding_failure: None,
        },
        statements: quote! {
            #binding let #length=#extent; #declarations
            #begin
            for (#index,#element) in ::core::iter::Iterator::enumerate(#iterator){ #enter #prepare #store }
            #finish
        },
        values,
        commit: quote!(#commit #commit_elements),
    };
    codec.validate_output(&out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn row_read_replacement_preserves_raw_pointer_reborrows() {
        let code = replace(
            &quote!(let value = unsafe { &(*pointer) }; consume(raw, value);),
            &[(syn::parse_quote!(raw), quote!(cached))],
        )
        .unwrap();
        assert!(code.to_string().contains(&quote!(&(*pointer)).to_string()));
        assert!(code.to_string().contains("consume (cached , value)"));
    }
}
