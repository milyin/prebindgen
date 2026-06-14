//! Structural converter-selection policy for [`JniGen`].

use super::*;

/// Clone a single-type-arg generic (`Option<X>` / `Vec<X>` / any `Path<X, …>`)
/// replacing its last segment's first type argument with `repl` — yielding the
/// canonical wildcard pattern (`Option<_>`) the rank-1 handlers `pat_match`,
/// with the type's own path/qualification preserved exactly as the enumerator
/// would have produced it.
fn with_first_arg(ty: &syn::Type, repl: syn::Type) -> syn::Type {
    let mut out = ty.clone();
    if let syn::Type::Path(tp) = &mut out {
        if let Some(seg) = tp.path.segments.last_mut() {
            if let syn::PathArguments::AngleBracketed(ab) = &mut seg.arguments {
                for a in ab.args.iter_mut() {
                    if let syn::GenericArgument::Type(t) = a {
                        *t = repl;
                        break;
                    }
                }
            }
        }
    }
    out
}

/// Clone a reference type replacing its referent with the `_` wildcard,
/// preserving the lifetime and mutability (`&'a T` → `&'a _`, `&mut T` →
/// `&mut _`) so the reconstructed pattern matches what the enumerator emitted.
fn ref_wildcard(r: &syn::TypeReference) -> syn::Type {
    let mut pr = r.clone();
    *pr.elem = syn::parse_quote!(_);
    syn::Type::Reference(pr)
}

impl<S: JniGenState> JniGen<S> {
    /// Select the input converter for `ty`: terminals, user wrappers, then
    /// built-in structural wrappers.
    pub(crate) fn select_input_type(
        &self,
        ty: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // 1. Terminal categories (incl. the terminal user-wrapper lookup).
        if let Some(c) = self.input_terminal(ty, registry) {
            return Some(c);
        }
        // 2. Higher-arity user-registered input patterns (any depth).
        if let Some(c) = self.match_user_input(ty, registry) {
            return Some(c);
        }
        // 3. Built-in wrapper shapes. `Option<&T>` tries the DEEP `Option<&_>`
        //    (borrowed-handle → `Option<OwnedObject<T>>`) before the shallow
        //    `Option<_>`; the shape that resolves correctly wins.
        if let Some(inner) = option_inner_type(ty) {
            if let syn::Type::Reference(r) = &inner {
                let pat = with_first_arg(ty, ref_wildcard(r));
                let t1 = (*r.elem).clone();
                if let Some(mut c) = self.input_wrapper_shape(&pat, &t1, registry) {
                    c.subs = vec![t1];
                    return Some(c);
                }
            }
            let pat = with_first_arg(ty, syn::parse_quote!(_));
            if let Some(mut c) = self.input_wrapper_shape(&pat, &inner, registry) {
                c.subs = vec![inner];
                return Some(c);
            }
            return None;
        }
        if let Some(elem) = vec_inner_type(ty) {
            let pat = with_first_arg(ty, syn::parse_quote!(_));
            if let Some(mut c) = self.input_wrapper_shape(&pat, &elem, registry) {
                c.subs = vec![elem];
                return Some(c);
            }
            return None;
        }
        if let syn::Type::Reference(r) = ty {
            let pat = ref_wildcard(r);
            let t1 = (*r.elem).clone();
            if let Some(mut c) = self.input_wrapper_shape(&pat, &t1, registry) {
                c.subs = vec![t1];
                return Some(c);
            }
        }
        None
    }

    /// Select the output converter for `ty`: terminals, user wrappers, then
    /// built-in structural wrappers.
    pub(crate) fn select_output_type(
        &self,
        ty: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // 1. Terminal categories (incl. the terminal user-wrapper lookup).
        if let Some(c) = self.output_terminal(ty, registry) {
            return Some(c);
        }
        // 2. User-registered patterns, specificity-ordered — the built-in
        //    `Result<_, _>` peel and any consumer override (`Result<_,
        //    ConcreteErr>` wins over the catch-all). Any depth.
        if let Some(c) = self.match_user_output(ty, registry) {
            return Some(c);
        }
        // 3. Built-in wrapper shapes (`Option<_>`, `Vec<_>`, `&T` borrow). An
        //    `Option<&Handle>` resolves via the shallow `Option<_>` whose inner
        //    converter is the `&Handle` borrow entry (no deep output handler).
        if let Some(inner) = option_inner_type(ty) {
            let pat = with_first_arg(ty, syn::parse_quote!(_));
            if let Some(mut c) = self.output_wrapper_shape(&pat, &inner, registry) {
                c.subs = vec![inner];
                return Some(c);
            }
            return None;
        }
        if let Some(elem) = vec_inner_type(ty) {
            let pat = with_first_arg(ty, syn::parse_quote!(_));
            if let Some(mut c) = self.output_wrapper_shape(&pat, &elem, registry) {
                c.subs = vec![elem];
                return Some(c);
            }
            return None;
        }
        if let syn::Type::Reference(r) = ty {
            let pat = ref_wildcard(r);
            let t1 = (*r.elem).clone();
            if let Some(mut c) = self.output_wrapper_shape(&pat, &t1, registry) {
                c.subs = vec![t1];
                return Some(c);
            }
        }
        None
    }
}
