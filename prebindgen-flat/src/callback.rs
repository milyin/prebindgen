//! Source signatures of retained, unit-returning callbacks.
use crate::{Result, Type};

/// Arguments passed by Rust to an `impl Fn(...)` source parameter.
#[derive(Clone, Debug)]
pub struct CallbackSignature {
    /// Source argument types, in invocation order.
    pub arguments: Vec<Type>,
}

/// Read a unit-returning `impl Fn(...)` callback, or return `None` for other types.
///
/// Only `Fn`, optional `Send`/`Sync`, and an optional `'static` bound are accepted.
/// Callback return values, higher-ranked bounds, and other callable traits require
/// a different foreign invocation contract and are rejected.
pub fn callback_signature(ty: &Type) -> Result<Option<CallbackSignature>> {
    let Type::ImplTrait(callback) = ty else {
        return Ok(None);
    };
    let mut arguments = None;
    for bound in &callback.bounds {
        match bound {
            syn::TypeParamBound::Lifetime(lifetime) if lifetime.ident == "static" => {}
            syn::TypeParamBound::Trait(trait_bound)
                if trait_bound.lifetimes.is_none()
                    && matches!(trait_bound.modifier, syn::TraitBoundModifier::None) =>
            {
                let path = &trait_bound.path;
                let short = path.segments.last().ok_or("empty callback bound")?;
                let name = short.ident.to_string();
                let prefix = path
                    .segments
                    .iter()
                    .take(path.segments.len() - 1)
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");
                let canonical = prefix.is_empty()
                    || matches!(
                        (prefix.as_str(), name.as_str()),
                        ("std::ops" | "core::ops", "Fn")
                            | ("std::marker" | "core::marker", "Send" | "Sync")
                    );
                if !canonical {
                    return Err("unsupported callback trait bound".into());
                }
                match (name.as_str(), &short.arguments) {
                    ("Fn", syn::PathArguments::Parenthesized(signature)) if arguments.is_none() => {
                        if !matches!(&signature.output, syn::ReturnType::Default)
                            && !matches!(&signature.output, syn::ReturnType::Type(_, ty) if matches!(&**ty, Type::Tuple(t) if t.elems.is_empty()))
                        { return Err("callbacks must return unit".into()); }
                        arguments = Some(signature.inputs.iter().cloned().collect());
                    }
                    ("Send" | "Sync", syn::PathArguments::None) => {}
                    _ => return Err("only unit-returning Fn callbacks with Send, Sync and 'static bounds are supported".into()),
                }
            }
            _ => return Err("unsupported callback lifetime or higher-ranked bound".into()),
        }
    }
    Ok(Some(CallbackSignature {
        arguments: arguments.ok_or("callback requires Fn(...)")?,
    }))
}
