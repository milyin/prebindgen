//! Slice/`Vec` inputs built as transient Rust-side `Vec` handles
//! (`New`/`Push`/`Free` helper trio).

use super::*;
// `flat` as a module for `TypeKind`: the bare name in this scope is jnigen's own
// classifier (reached through `use super::*`), and an explicit import would win
// over the glob and silently retarget it.
use crate::api::core::flat::{self, RefMode, TypeRef};

// `slice_or_vec_elem` lived here: it matched `&[T]` / `Vec<T>` off the SPELLING
// and returned the element. `vec_build_elem` was its only caller and now reads
// the same two facts off the model (`sequence_elem`, through `borrow_target`),
// where a `Box<Vec<T>>` answers as `Vec<T>` does instead of failing a
// last-path-segment test. Its one non-structural rule — `&mut [T]` is refused,
// because mutate-back semantics keep the `input_vec` path — survives as the
// `RefMode::Shared` guard on that match.

/// `Some((element_type, by_ref))` when `arg_ty` is a slice/`Vec` input whose
/// element is a **flattenable `data_class`** — i.e. it decomposes into the
/// conservative leaf set [`build_flat_input_plan`] accepts, so each element can
/// cross as decoupled raw params and be rebuilt on the Rust side with no
/// `env.get_field(...)`. `None` for any other shape (opaque handles, enums,
/// nested-`Option` structs), which keep the `input_vec` path.
///
/// This is the single detection seam shared by `emit_input_param`, the param
/// classifier, `render_extern_decl`, and the synthetic-extern emitter so all
/// four sites agree on which params take the handle path.
pub(crate) fn vec_build_elem(
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
    arg: &TypeRef,
) -> Option<(TypeRef, bool)> {
    // The run and its element off the MODEL. `&mut [T]` is still refused —
    // mutate-back semantics keep the `input_vec` path — and that is the one
    // fact the layer accessors do not carry, so `RefMode` is read directly.
    //
    // The conversion follows the SYNTAX, and must: this path builds a Rust-side
    // `Vec<T>` and hands the source fn a borrow of it (or `mem::take`s it), so
    // the referent has to be a form that built value satisfies. `&[T]`
    // deref-coerces and `Vec<T>` is the thing itself; `Box<Vec<T>>` and
    // `Cow<'_, [T]>` classify identically and cannot be rebuilt from the local.
    // `decoded_vec_satisfies` in `selector.rs` is the same rule guarding the
    // general converter path — asked here of the model, which holds both halves.
    let (run, by_ref) = match arg.kind() {
        flat::TypeKind::Ref { mode, inner } if *mode == RefMode::Shared => (&**inner, true),
        _ => (arg, false),
    };
    if run.erased_wrapper().is_some() {
        return None;
    }
    let elem = run.sequence_elem()?;
    // The element is spelled into `Vec<#elem>` and rebuilt per push, so a
    // wrapped element spelling is unbuildable for the same reason.
    if elem.erased_wrapper().is_some() {
        return None;
    }
    // The element must flatten; the probe ident is irrelevant here.
    let plan = build_flat_input_plan(ext, registry, &format_ident!("e"), elem)
        .ok()
        .flatten()?;
    // Recursive/optional element decomposition is intentionally outside this
    // increment: retain the existing List/JObject collection path rather than
    // silently changing the Vec helper ABI.
    if plan.contains_nested
        || plan.leaves.iter().any(|l| {
            l.is_present_flag
                || l.handle_target_tail.is_some()
                || l.entry.as_ref().is_none_or(|e| !e.pre_stages.is_empty())
        })
    {
        return None;
    }
    Some((elem.clone(), by_ref))
}

/// Every distinct flattenable element type `T` that a scanned, declared function
/// takes as a `&[T]`/`Vec<T>` input — the set the synthetic `…VecNew/Push/Free`
/// externs are emitted for (once per type, shared across all such functions).
/// Deduped by [`TypeKey`] and sorted for deterministic output (mirrors
/// [`build_handle_destructor_items`]).
pub(crate) fn collect_vec_build_elem_types(
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
) -> Vec<TypeRef> {
    let declared = ext.declared_functions();
    let mut seen: std::collections::BTreeMap<String, TypeRef> = std::collections::BTreeMap::new();
    // Over the model's params, which already carry a reading each — the
    // `sig.inputs` walk had to re-derive one per argument.
    for f in registry.flat().functions() {
        if !declared.contains(&f.name) {
            continue;
        }
        for p in &f.params {
            if let Some((elem, _)) = vec_build_elem(ext, registry, &p.ty) {
                seen.insert(elem.key().as_str().to_string(), elem);
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
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
    elem: &TypeRef,
) -> Option<VecBuildHelpers> {
    let plan = build_flat_input_plan(ext, registry, &format_ident!("e"), elem)
        .ok()
        .flatten()?;
    if plan.contains_nested
        || plan.leaves.iter().any(|l| {
            l.is_present_flag
                || l.handle_target_tail.is_some()
                || l.entry.as_ref().is_none_or(|e| !e.pre_stages.is_empty())
        })
    {
        return None;
    }
    let key = elem.key();
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
pub(crate) fn vec_helper_method_name(ext: &Declarations, base: &str, suffix: &str) -> String {
    ext.mangle_jni_method(&format!("{base}{suffix}"))
}

/// Full Rust JNI symbol for a vec helper — the same spec-escaped
/// `Java_<pkg>_<JNINative>_…` scheme function wrappers use via the plan's
/// `native_symbol` (see `symbol`, #86); these helpers live on the
/// `JNINative` object, so they share its class path.
fn vec_helper_symbol(ext: &Declarations, base: &str, suffix: &str) -> String {
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
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
) -> Vec<syn::Item> {
    let mut named: Vec<(String, syn::Item)> = Vec::new();
    for elem_reading in collect_vec_build_elem_types(ext, registry) {
        // Generated Rust spells `origin.syntax`; the reading is what the plan
        // and the key are taken from.
        let elem = elem_reading.syntax();
        let Some(h) = vec_build_helpers(ext, registry, &elem_reading) else {
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
        let module = &h.plan.root.struct_module;
        let sid = &h.plan.root.struct_ident;
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
