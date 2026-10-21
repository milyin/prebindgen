//! Shared optional conversion composition and wire niches.
use super::*;

/// Optional layout chosen from the inner conversion's available wire niches.
#[derive(Clone, Debug)]
pub struct OptionalLayout<N> {
    /// Inner wire index and its reserved absent representation; otherwise a flag is added.
    pub niche: Option<(usize, N)>,
}
fn layout<C: Codec>(codec: &C, wires: &[Wire<C::Wire>]) -> OptionalLayout<C::Niche> {
    OptionalLayout {
        niche: wires
            .iter()
            .enumerate()
            .find_map(|(i, w)| codec.niche(&w.kind).map(|n| (i, n))),
    }
}
/// Wrap an input plan whose activation condition includes `tag == 1`.
/// The payload's construction is guarded as well as its validation/acquisition.
pub fn optional_input<C: Codec>(
    codec: &C,
    extra: &C::Extra,
    tag: syn::Ident,
    var: syn::Ident,
    active: &Condition,
    mut child: InputPlan<C::Wire, C::Context>,
) -> Result<InputPlan<C::Wire, C::Context>> {
    let layout = layout(codec, &child.wires);
    let mut validation = TokenStream::new();
    if let Some((i, niche)) = &layout.niche {
        let w = &child.wires[*i].name;
        let present = codec.is_present(&child.wires[*i].kind, niche, quote!(#w))?;
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
    let inner = child.wires.clone();
    if layout.niche.is_none() {
        child.wires.insert(
            0,
            Wire {
                name: tag.clone(),
                kind: codec.selector(extra),
            },
        );
    }
    codec.mark_optional(&mut child.wires, &inner, &tag, &layout, true)?;
    if let Some((index, niche)) = &layout.niche {
        codec.reserve_niche(&mut child.wires[*index].kind, niche)?;
    }
    Ok(child)
}
/// Prepare only a present output, retaining its owners in a deferred commit closure.
/// `child` must be unfinalized and refer to the supplied payload binding.
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
    mut child: OutputPlan<C::Wire, C::Context>,
) -> Result<OutputPlan<C::Wire, C::Context>> {
    if !child.delivery.statements.is_empty()
        || child.delivery.result.to_string() != quote!(()).to_string()
    {
        return Err("optional child has delivery code".into());
    }
    if child.wires.len() != child.values.len() {
        return Err("optional output wire/value count mismatch".into());
    }
    let layout = layout(codec, &child.wires);
    let mut defaults = Vec::new();
    for (i, w) in child.wires.iter().enumerate() {
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
    let names = (0..child.wires.len())
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
    let inner = child.wires.clone();
    if layout.niche.is_none() {
        child.wires.insert(
            0,
            Wire {
                name: tag.clone(),
                kind: codec.selector(extra),
            },
        );
        child.values.insert(0, quote!(#tag));
    }
    codec.mark_optional(&mut child.wires, &inner, &tag, &layout, false)?;
    if let Some((index, niche)) = &layout.niche {
        codec.reserve_niche(&mut child.wires[*index].kind, niche)?;
    }
    Ok(child)
}
