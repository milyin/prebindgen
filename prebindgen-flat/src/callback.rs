//! Source signatures of retained callbacks.
use crate::{Result, Type};

/// Arguments and return type of an `impl Fn(...) -> R` source parameter.
#[derive(Clone, Debug)]
pub struct CallbackSignature {
    /// Source argument types, in invocation order.
    pub arguments: Vec<Type>,
    /// Value returned by the callback; omitted returns are represented as `()`.
    pub return_type: Type,
}

/// Read an `impl Fn(...) -> R` callback, or return `None` for other types.
///
/// Only `Fn`, optional `Send`/`Sync`, and an optional `'static` bound are accepted.
/// Higher-ranked bounds and other callable traits are rejected. Adapters decide
/// which argument and return conversions their foreign invocation supports.
pub fn callback_signature(ty: &Type) -> Result<Option<CallbackSignature>> {
    let Type::ImplTrait(callback) = ty else {
        return Ok(None);
    };
    let mut signature = None;
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
                    ("Fn", syn::PathArguments::Parenthesized(call)) if signature.is_none() => {
                        signature = Some(CallbackSignature {
                            arguments: call.inputs.iter().cloned().collect(),
                            return_type: match &call.output {
                                syn::ReturnType::Default => syn::parse_quote!(()),
                                syn::ReturnType::Type(_, ty) => (**ty).clone(),
                            },
                        });
                    }
                    ("Send" | "Sync", syn::PathArguments::None) => {}
                    _ => {
                        return Err(
                            "only Fn callbacks with Send, Sync and 'static bounds are supported"
                                .into(),
                        )
                    }
                }
            }
            _ => return Err("unsupported callback lifetime or higher-ranked bound".into()),
        }
    }
    Ok(Some(signature.ok_or("callback requires Fn(...)")?))
}
