//! Shared optional conversion composition and wire niches.
use super::*;

/// Optional layout chosen from the inner conversion's available wire niches.
#[derive(Clone, Debug)]
struct OptionalLayout<N> {
    /// Inner wire index and its reserved absent representation; otherwise a flag is added.
    pub niche: Option<(usize, N)>,
}
fn layout<C: Codec>(codec: &C, wires: &WireLayout<C::Wire, C::Group>) -> OptionalLayout<C::Niche> {
    OptionalLayout {
        niche: wires
            .iter()
            .enumerate()
            .find_map(|(i, w)| codec.niche(&w.kind).map(|n| (i, n))),
    }
}
/// Wrap an input plan whose activation condition includes `tag == 1`.
/// The payload's construction is guarded as well as its validation/acquisition.
/// Choose `tag` and `var` fresh within the complete slot.
pub fn optional_input<C: Codec>(
    codec: &C,
    extra: &C::Extra,
    tag: syn::Ident,
    var: syn::Ident,
    active: &Condition,
    mut child: CodecInput<C>,
) -> Result<CodecInput<C>> {
    let layout = layout(codec, &child.layout);
    let mut validation = TokenStream::new();
    if let Some((i, niche)) = &layout.niche {
        let w = &child.layout[*i].name;
        let present = codec.is_present(&child.layout[*i].kind, niche, quote!(#w))?;
        validation.extend(quote!(let #tag: i32 = i32::from(#present);));
    } else {
        let bad = active.and(quote!(#tag != 0 && #tag != 1));
        validation.extend(quote!(if #bad {return Err("invalid option presence".into());}));
    }
    validation.extend(child.validation);
    child.validation = validation;
    let body = child.statements;
    let arg = child.argument;
    let selected = active.arm(tag.clone(), 1).rust();
    child.statements = quote!(let #var = if #selected {#body Some(#arg)} else {None};);
    child.argument = quote!(#var);
    if layout.niche.is_none() {
        child.layout.prepend(Wire {
            name: tag.clone(),
            kind: codec.selector(extra),
        });
    }
    mark(codec, &mut child.layout, tag, layout);
    Ok(child)
}
/// Prepare only a present output, retaining its owners in a deferred commit closure.
/// `child` must be unfinalized and refer to the supplied payload binding.
/// Choose `tag` and `payload` fresh within the complete slot.
/// Values must own their data or point into owners retained by `child.commit`.
/// The commit must be infallible and must not borrow an emission context that
/// delivery still needs: it is captured in a move closure until delivery succeeds.
pub fn optional_output<C: Codec>(
    codec: &C,
    extra: &C::Extra,
    tag: syn::Ident,
    payload: syn::Ident,
    value: TokenStream,
    borrowed: bool,
    mut child: CodecOutput<C>,
) -> Result<CodecOutput<C>> {
    if !child.delivery.statements.is_empty()
        || child.delivery.result.to_string() != quote!(()).to_string()
    {
        return Err("optional child has delivery code".into());
    }
    if child.layout.len() != child.values.len() {
        return Err("optional output wire/value count mismatch".into());
    }
    let layout = layout(codec, &child.layout);
    let mut defaults = Vec::new();
    for (i, w) in child.layout.iter().enumerate() {
        defaults.push(
            codec.absent_value(
                &w.kind,
                layout
                    .niche
                    .as_ref()
                    .filter(|(j, _)| *j == i)
                    .map(|(_, n)| n),
                false,
            )?,
        );
    }
    let names = (0..child.layout.len())
        .map(|i| format_ident!("{tag}_value{i}"))
        .collect::<Vec<_>>();
    let presence_binding = if layout.niche.is_some() {
        quote!(_)
    } else {
        quote!(#tag)
    };
    let commit_var = format_ident!("{tag}_commit");
    let values_var = format_ident!("{tag}_values");
    let action_var = format_ident!("{tag}_action");
    let body = child.statements;
    let values = child.values;
    let commit = child.commit;
    let source = if borrowed {
        quote!((#value).as_ref())
    } else {
        value
    };
    // Compute wire values before moving the owners into the closure. Dropping an
    // uncalled closure drops those owners on any subsequent conversion/delivery failure.
    child.statements = quote! {
        let (#presence_binding, (#(#names,)*), #commit_var) = if let Some(#payload) = #source {
            #body
            let #values_var = (#(#values,)*);
            (1i32, #values_var, Some(move || {#commit}))
        } else { (0i32, (#(#defaults,)*), None) };
    };
    child.commit = quote!(if let Some(#action_var) = #commit_var {#action_var();});
    child.values = names.iter().map(|n| quote!(#n)).collect();
    if layout.niche.is_none() {
        child.layout.prepend(Wire {
            name: tag.clone(),
            kind: codec.selector(extra),
        });
        child.values.insert(0, quote!(#tag));
    }
    mark(codec, &mut child.layout, tag, layout);
    Ok(child)
}

fn mark<C: Codec>(
    codec: &C,
    tree: &mut WireLayout<C::Wire, C::Group>,
    tag: syn::Ident,
    layout: OptionalLayout<C::Niche>,
) {
    let discriminator = layout
        .niche
        .as_ref()
        .map(|(i, _)| tree[*i].name.clone())
        .unwrap_or(tag.clone());
    let remaining = layout
        .niche
        .as_ref()
        .and_then(|(i, n)| codec.remaining_niche(&tree[*i].kind, n));
    for wire in tree.iter_mut() {
        codec.set_niche(&mut wire.kind, None);
    }
    if let Some((i, _)) = &layout.niche {
        codec.set_niche(&mut tree[*i].kind, remaining);
    }
    tree.enclose(codec.optional_group(tag, discriminator, layout.niche.map(|(_, n)| n)));
}
