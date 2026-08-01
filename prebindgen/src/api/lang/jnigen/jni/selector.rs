//! Structural converter-selection policy for [`Declarations`].

use super::*;
use crate::api::core::registry::Conversions;

/// Clone a single-type-arg generic (`Option<X>` / `Vec<X>` / any `Path<X, …>`)
/// replacing its last segment's first type argument with `repl` — yielding the
/// canonical shape (`Option<_>`) the built-in wrapper handlers key on, with the
/// type's own path/qualification preserved exactly.
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

impl Declarations {
    /// Select the input converter for `ty`: terminals, user wrappers, then
    /// built-in structural wrappers.
    pub(crate) fn select_input_type(
        &self,
        ty: &syn::Type,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // 1. Terminal categories (incl. the terminal user-wrapper lookup).
        if let Some(c) = self.input_terminal(ty, registry) {
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
            // `&[T]` shared slice borrow: there is no owned `[T]` to decode, so
            // reuse the `Vec<_>` shape — decode the Java `List<T>` into an owned
            // `Vec<T>`; the call site borrows it (`&Vec<T>` deref-coerces to
            // `&[T]`). Wire/Kotlin type are `List<T>`, identical to a by-value
            // `Vec<T>` input (the writer dedupes the shared converter fn by ident,
            // so the two can coexist). `&mut [T]` is intentionally not supported
            // (no write-back of the decoded Vec).
            if r.mutability.is_none() {
                if let syn::Type::Slice(s) = &*r.elem {
                    let elem = (*s.elem).clone();
                    let pat: syn::Type = syn::parse_quote!(Vec<_>);
                    if let Some(mut c) = self.input_wrapper_shape(&pat, &elem, registry) {
                        c.subs = vec![elem];
                        return Some(c);
                    }
                    return None;
                }
            }
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
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // 1. Terminal categories (incl. the terminal user-wrapper lookup).
        if let Some(c) = self.output_terminal(ty, registry) {
            return Some(c);
        }
        // 2. `Result<T, E>`: succeeds as `T`, routes `E` to the error sink.
        //    Read off the model, which calls this shape `TypeKind::Fallible`.
        //    `result_parts` covers a `Result` the adapter composed itself, which
        //    the frontend never read.
        if let Some((ok, err)) = fallible_parts(ty, registry) {
            if let Some(c) = self.result_peel(ty, &ok, &err, registry) {
                return Some(c);
            }
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
            // `&[T]` shared slice (a callback argument crossing native→JVM): build a
            // `List<T>` from the borrowed slice. Dual of the `&[T]` input branch.
            if r.mutability.is_none() {
                if let syn::Type::Slice(s) = &*r.elem {
                    let elem = (*s.elem).clone();
                    return self.output_slice(&elem, registry);
                }
            }
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

/// The `Ok`/`Err` of a `Result`, preferring the frontend's reading.
///
/// The model classifies a `Result` as [`TypeKind::Fallible`]; the syntactic
/// fallback is for a `Result` the adapter composed itself, which no captured
/// item spells and the frontend therefore never read.
///
/// **Measured: the fallback never fires in-tree** — zero occurrences across
/// covertest-kotlin and perftest-kotlin, because #246 indexes a binding-local
/// fn's types, so even a `sig!((..) -> Result<Summary, String>)` has a reading.
/// It is kept rather than made a hard error because an out-of-tree consumer may
/// compose a `Result` the model never sees, and it costs nothing: `result_parts`
/// already exists and already has six other callers.
fn fallible_parts(
    ty: &syn::Type,
    registry: &impl Conversions<KotlinMeta>,
) -> Option<(syn::Type, syn::Type)> {
    use crate::api::core::flat::TypeKind;
    if let Some(TypeKind::Fallible { ok, err }) = registry.flat().type_ref(ty).map(|t| &t.kind) {
        return Some((ok.origin.syntax.clone(), err.origin.syntax.clone()));
    }
    crate::api::core::types_util::result_parts(ty)
}
