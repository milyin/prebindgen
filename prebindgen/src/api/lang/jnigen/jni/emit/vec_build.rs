//! Slice/`Vec` inputs built as transient Rust-side `Vec` handles
//! (`New`/`Push`/`Free` helper trio).

use super::*;

/// Classify a slice/`Vec` **input** param for the "build the Rust-side Vec
/// incrementally" path: an immutable slice `&[T]` (`by_ref = true`, the target
/// borrows the boxed Vec) or a by-value `Vec<T>` (`by_ref = false`, the target
/// moves it out via `mem::take`). `&mut [T]` (mutate-back semantics) and every
/// other shape return `None`, keeping the existing `input_vec` `List<JObject>`
/// path. (Element flattenability is checked separately by [`vec_build_elem`].)
pub(crate) fn slice_or_vec_elem(arg_ty: &syn::Type) -> Option<(syn::Type, bool)> {
    match arg_ty {
        syn::Type::Reference(r) if r.mutability.is_none() => match &*r.elem {
            syn::Type::Slice(s) => Some(((*s.elem).clone(), true)),
            _ => None,
        },
        _ => vec_inner_type(arg_ty).map(|t| (t, false)),
    }
}

/// `Some((element_type, by_ref))` when `arg_ty` is a slice/`Vec` input whose
/// element is a **flattenable `data_class`** — i.e. it decomposes into the
/// conservative leaf set [`build_flat_input_plan`] accepts, so each element can
/// cross as decoupled raw params and be rebuilt on the Rust side with no
/// `env.get_field(...)`. `None` for any other shape (opaque handles, enums,
/// value blobs, nested-`Option` structs), which keep the `input_vec` path.
///
/// This is the single detection seam shared by `emit_input_param`, the param
/// classifier, `render_extern_decl`, and the synthetic-extern emitter so all
/// four sites agree on which params take the handle path.
pub(crate) fn vec_build_elem(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    arg_ty: &syn::Type,
) -> Option<(syn::Type, bool)> {
    let (elem, by_ref) = slice_or_vec_elem(arg_ty)?;
    // The element must flatten; the probe ident is irrelevant here.
    build_flat_input_plan(ext, registry, &format_ident!("e"), &elem)?;
    Some((elem, by_ref))
}

/// Every distinct flattenable element type `T` that a scanned, declared function
/// takes as a `&[T]`/`Vec<T>` input — the set the synthetic `…VecNew/Push/Free`
/// externs are emitted for (once per type, shared across all such functions).
/// Deduped by [`TypeKey`] and sorted for deterministic output (mirrors
/// [`build_handle_destructor_items`]).
pub(crate) fn collect_vec_build_elem_types(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
) -> Vec<syn::Type> {
    let declared = ext.declared_functions();
    let mut seen: std::collections::BTreeMap<String, syn::Type> = std::collections::BTreeMap::new();
    for (ident, (item_fn, _)) in &registry.functions {
        if !declared.contains(ident) {
            continue;
        }
        for input in &item_fn.sig.inputs {
            if let syn::FnArg::Typed(pt) = input {
                if let Some((elem, _)) = vec_build_elem(ext, registry, &pt.ty) {
                    seen.insert(TypeKey::from_type(&elem).as_str().to_string(), elem);
                }
            }
        }
    }
    seen.into_values().collect()
}

/// One element type's `…VecNew/Push/Free` helper trio: the flatten plan whose
/// leaves are the per-element push params, plus the camelCase base name
/// (`payloadVec`) the Kotlin methods and Rust JNI symbols share.
pub(crate) struct VecBuildHelpers {
    /// camelCase base, e.g. `"payloadVec"` (Kotlin method = `<base>New/Push/Free`).
    pub base: String,
    /// Element flatten plan (built with the synthetic param ident `e`).
    pub plan: FlatInputPlan,
}

/// Build the helper descriptor for one flattenable element type, or `None` if it
/// doesn't flatten (caller keeps the `input_vec` path). The base name is derived
/// from the element's **Kotlin** data-class short name (first char lowercased) so
/// the generated methods read naturally (`Payload` → `payloadVec`).
pub(crate) fn vec_build_helpers(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    elem: &syn::Type,
) -> Option<VecBuildHelpers> {
    let plan = build_flat_input_plan(ext, registry, &format_ident!("e"), elem)?;
    let key = TypeKey::from_type(elem);
    let kt_fqn = ext
        .types
        .get(&key)
        .and_then(|c| c.name_spec.as_ref())
        .map(|s| ext.fqn_of(s))?;
    let short = kt_fqn.rsplit('.').next().unwrap_or(&kt_fqn);
    let mut chars = short.chars();
    let base_lc = match chars.next() {
        Some(f) => format!("{}{}", f.to_lowercase(), chars.as_str()),
        None => short.to_string(),
    };
    Some(VecBuildHelpers {
        base: format!("{base_lc}Vec"),
        plan,
    })
}

/// Kotlin `external fun` short name for a vec helper (`payloadVecNew`), routed
/// through the method mangler like every other `JNINative` extern. The Rust JNI symbol
/// (see [`vec_helper_symbol`]) and the Kotlin call site both use this, so they
/// agree.
pub(crate) fn vec_helper_method_name(ext: &JniGen, base: &str, suffix: &str) -> String {
    ext.mangle_jni_method(&format!("{base}{suffix}"))
}

/// Full Rust JNI symbol for a vec helper — the same spec-escaped
/// `Java_<pkg>_<JNINative>_…` scheme function wrappers use via
/// [`mangle_jni_name`] (see `symbol`, #86); these helpers live on the
/// `JNINative` object, so they share its class path.
fn vec_helper_symbol(ext: &JniGen, base: &str, suffix: &str) -> String {
    ext.native_method_symbol(&vec_helper_method_name(ext, base, suffix))
}

/// One `#[no_mangle] extern "C"` `…VecNew/Push/Free` trio per flattenable
/// element type used as a `&[T]`/`Vec<T>` input — the Rust half of the
/// build-the-Vec-incrementally path. Modeled on [`build_handle_destructor_items`]
/// (deterministic symbol sort, emitted only for element types a scanned function
/// actually takes by slice/Vec).
///
/// `Push` is **infallible**: every leaf but a `String` is a primitive (the
/// converter can't fail), and a `String?` passed straight from Kotlin always
/// decodes. The only way a converter errs here is a JNI-internal fault (OOM /
/// pending exception), which can't arise from a valid argument — so on the cold
/// `Err` path it logs and skips the element rather than threading a per-caller
/// error sink (the sink's typed `run` descriptor varies by caller, and `Push` is
/// shared across all callers of a given element type). This keeps the Kotlin
/// push loop free of a per-element failure check.
pub(crate) fn build_vec_build_helper_items(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
) -> Vec<syn::Item> {
    let mut named: Vec<(String, syn::Item)> = Vec::new();
    for elem in collect_vec_build_elem_types(ext, registry) {
        let Some(h) = vec_build_helpers(ext, registry, &elem) else {
            continue;
        };
        let new_sym = vec_helper_symbol(ext, &h.base, "New");
        let push_sym = vec_helper_symbol(ext, &h.base, "Push");
        let free_sym = vec_helper_symbol(ext, &h.base, "Free");
        let new_id = syn::Ident::new(&new_sym, Span::call_site());
        let push_id = syn::Ident::new(&push_sym, Span::call_site());
        let free_id = syn::Ident::new(&free_sym, Span::call_site());

        named.push((
            new_sym.clone(),
            syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub(crate) unsafe extern "C" fn #new_id(
                    _env: jni::JNIEnv,
                    _class: jni::objects::JClass,
                    cap: jni::sys::jint,
                ) -> jni::sys::jlong {
                    let __cap = if cap > 0 { cap as usize } else { 0usize };
                    Box::into_raw(Box::new(Vec::<#elem>::with_capacity(__cap))) as jni::sys::jlong
                }
            ),
        ));

        let leaf_params: Vec<TokenStream> = h
            .plan
            .leaves
            .iter()
            .filter(|l| !l.is_present_flag)
            .map(|l| {
                let id = &l.native_ident;
                let ty = &l.native_wire_ty;
                quote!(#id: #ty)
            })
            .collect();
        let mut decodes: Vec<TokenStream> = Vec::new();
        let mut inits: Vec<TokenStream> = Vec::new();
        for l in h.plan.leaves.iter().filter(|l| !l.is_present_flag) {
            let conv = l.conv.as_ref().expect("non-present leaf has a converter");
            let wid = &l.native_ident;
            let fid = l.field.clone().expect("non-present leaf has a field");
            let tmp = format_ident!("__e_{}", fid);
            decodes.push(quote!(
                let #tmp = match #conv(&mut env, &#wid) {
                    ::core::result::Result::Ok(__v) => __v,
                    ::core::result::Result::Err(__e) => {
                        tracing::error!("vecPush: decoding `{}`: {}", stringify!(#fid), __e);
                        return;
                    }
                };
            ));
            inits.push(quote!(#fid: #tmp));
        }
        let module = &h.plan.struct_module;
        let sid = &h.plan.struct_ident;
        named.push((
            push_sym.clone(),
            syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
                pub(crate) unsafe extern "C" fn #push_id<'a>(
                    mut env: jni::JNIEnv<'a>,
                    _class: jni::objects::JClass<'a>,
                    handle: jni::sys::jlong,
                    #(#leaf_params,)*
                ) {
                    if handle == 0 {
                        return;
                    }
                    #(#decodes)*
                    let __elem = #module::#sid { #(#inits),* };
                    let __vec = &mut *(handle as *mut Vec<#elem>);
                    __vec.push(__elem);
                }
            ),
        ));

        named.push((
            free_sym.clone(),
            syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub(crate) unsafe extern "C" fn #free_id(
                    _env: jni::JNIEnv,
                    _class: jni::objects::JClass,
                    handle: jni::sys::jlong,
                ) {
                    if handle != 0 {
                        drop(Box::from_raw(handle as *mut Vec<#elem>));
                    }
                }
            ),
        ));
    }
    named.sort_by(|a, b| a.0.cmp(&b.0));
    named.into_iter().map(|(_, item)| item).collect()
}
