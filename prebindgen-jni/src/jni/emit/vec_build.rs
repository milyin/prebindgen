//! Slice/`Vec` inputs built as transient Rust-side `Vec` handles
//! (`New`/`Push`/`Free` helper trio).

// `flat` as a module for `TypeKind`: the bare name in this scope is jnigen's own
// classifier (reached through `use super::*`), and an explicit import would win
// over the glob and silently retarget it.
use prebindgen_registry::flat::{self, TypeRef};

use super::*;
use crate::jni::trait_impl::{build_through_erased_wrappers, build_through_wrappers};

// `slice_or_vec_elem` lived here: it matched `&[T]` / `Vec<T>` off the SPELLING
// and returned the element. `vec_build_elem` was its only caller and now reads
// the same two facts off the model (`sequence_elem`, through `borrow_target`),
// where a `Box<Vec<T>>` answers as `Vec<T>` does instead of failing a
// last-path-segment test. Its one non-structural rule — `&mut [T]` is refused,
// because mutate-back semantics keep the `input_vec` path — survives as the
// `mutable: false` guard on that match.

/// What a slice/`Vec` parameter contributes to the Vec-build path: the
/// **canonical** element, whether the run is borrowed, and the element wrappers
/// the storage does not carry.
///
/// A named return rather than a tuple because the third field is only meaningful
/// against the first: `elem` is the element with every transparent wrapper taken
/// off, and `elem_wrappers` is what has to go back on. Reading `(a, b, c)` at
/// three call sites would not say that.
pub(crate) struct VecBuildElem {
    /// The element with its transparent wrappers removed — what the helper trio
    /// stores, what its name is derived from, and what the flatten plan is built
    /// for. `Vec<Payload>` and `Vec<Box<Payload>>` answer with the same node,
    /// which is the whole mechanism: one trio per Kotlin class.
    pub elem: TypeRef,
    /// The run is a `&[T]`/`&Vec<T>` borrow rather than an owned value.
    pub by_ref: bool,
    /// The wrappers taken off the element, outermost first — empty for the
    /// ordinary case. Applied per element where the Vec is consumed, since the
    /// storage holds the canonical type. A list and not a `TypeRef`: it is the
    /// only part a rebuild uses, and it rides in [`InputKind::VecBuild`], whose
    /// size every variant pays.
    pub elem_wrappers: Vec<&'static str>,
}

/// `Some(..)` when `arg_ty` is a slice/`Vec` input whose element is a
/// **flattenable `data_class`** — i.e. it decomposes into the conservative leaf
/// set [`build_flat_input_plan`] accepts, so each element can cross as decoupled
/// raw params and be rebuilt on the Rust side with no `env.get_field(...)`.
/// `None` for any other shape (opaque handles, enums, nested-`Option` structs),
/// which keep the `input_vec` path.
///
/// This is the single detection seam shared by `emit_input_param`, the param
/// classifier, `render_extern_decl`, and the synthetic-extern emitter so all
/// four sites agree on which params take the handle path.
pub(crate) fn vec_build_elem(
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
    arg: &TypeRef,
) -> Option<VecBuildElem> {
    // The run and its element off the MODEL. `&mut [T]` is still refused —
    // mutate-back semantics keep the `input_vec` path — and that is the one
    // fact the layer accessors do not carry, so the borrow's mutability is read
    // off the kind directly.
    //
    // The conversion follows the SYNTAX, and must: this path builds a Rust-side
    // `Vec<T>` and hands the source fn a borrow of it (or `mem::take`s it), so
    // the referent has to be a form that built value satisfies. `&[T]`
    // deref-coerces and `Vec<T>` is the thing itself; `Box<Vec<T>>` and
    // `Cow<'_, [T]>` classify identically and cannot be rebuilt from the local.
    // `decoded_vec_satisfies` in `selector.rs` is the same rule guarding the
    // general converter path — asked here of the model, which holds both halves.
    //
    // BEFORE the peel as well as after, and the order is the whole point: the
    // erasure happens OUTSIDE the layer it wraps, so `Box<&Vec<T>>` classifies
    // as `Ref` and interpreting `kind` first would replace `arg` with the inner
    // sequence — whose spelling is a clean `Vec<T>` — and let the outer `Box`
    // through unseen. Every layer is checked on the way down, the way
    // `rebuildable_target` does it.
    let (run, by_ref) = match arg.unwrapped().kind() {
        flat::TypeKind::Ref {
            mutable: false,
            inner,
            ..
        } => (&**inner, true),
        _ => (arg, false),
    };
    // A wrapper over the RUN is buildable only on the by-value path, and the
    // reason is a cost rather than a type error. By value the local is owned, so
    // `Box<Vec<T>>` is `Box::new(mem::take(..))` — free. Borrowed, the local is
    // a borrow of the Vec the Kotlin side owns, and there is no way to put a
    // `Box` between that borrow and the callee without **copying** the run
    // (`&Box::new(v.clone())`) — which needs a `T: Clone` nothing here
    // guarantees, and silently adds a per-call copy to a path whose entire point
    // is not having one.
    //
    // Definitive, not deferred: the borrowed shape keeps the `input_vec` path,
    // which is correct. If a binding ever wants the copy, it is a decision to
    // make on purpose, at the declaration.
    let wrapped_run = !arg.erased_wrappers().is_empty() || !run.erased_wrappers().is_empty();
    if wrapped_run {
        if by_ref {
            return None;
        }
        // By value: both layers must be buildable (`Cow` still declines).
        build_through_erased_wrappers(arg, quote!(__probe))?;
        build_through_erased_wrappers(run, quote!(__probe))?;
    }
    let elem = run.sequence_elem()?;
    // A wrapped ELEMENT is served here. #294 refused it and called the refusal
    // definitive; #296 re-read it as reserved and this closes it.
    //
    // The trio stores a `Vec<#elem>` and its base name comes from the element's
    // **Kotlin class** (`Payload` → `payloadVec`), so keying it on the SPELLING
    // would make `Vec<Payload>` and `Vec<Box<Payload>>` two storages wanting one
    // name. Keying it on the CANONICAL element removes the question: one trio
    // per Kotlin class, storage `Vec<Payload>`, and the element's wrapper goes
    // back on where the Vec is CONSUMED — `.into_iter().map(Box::new).collect()`
    // in `emit_input_param`. No Rust wrapper reaches a JNI symbol, which is the
    // representation leak this layer exists to prevent, and nothing collides.
    //
    // What that buys: a `Box` the model erases no longer downgrades the crossing
    // from raw scalar leaves to a per-element `JObject` plus a field read per
    // field. Correctness was never the issue — the general converter serves the
    // shape (see `input_transparent_bridge`) — the cost was, and it was silent.
    let elem_wrappers = elem.erased_wrappers();
    if !elem_wrappers.is_empty() {
        // **Definitive on the BORROWED path.** By value the local is owned, so
        // the per-element wrap is one O(n) `Box::new` over a Vec already being
        // moved. Borrowed, the local is a borrow of the Vec the Kotlin side
        // owns, and a `Vec<Box<T>>` is not a `Vec<T>` viewed differently — it is
        // a different allocation. Serving it would mean `mem::take`ing that Vec
        // and borrowing a fresh one, which turns the single arm whose whole
        // point is not copying into one that consumes its input.
        //
        // So the shapes coincide with the wrapped-run refusal above and the
        // reasons do not: that one cannot copy without a `T: Clone`, this one
        // could copy and must not.
        if by_ref {
            return None;
        }
        // `Cow` still declines, and by its own policy rather than this path's:
        // `Cow::Owned` is well-typed, and refusing it is what keeps a binding
        // from silently removing the borrow path. See its `WRAPPER_OPS` row.
        build_through_wrappers(&elem_wrappers, quote!(__probe))?;
    }
    // Canonical from here down: the flatten plan, the trio's storage and its
    // name are all the element's, and the wrapper is the consumption site's
    // business. `unwrapped` and not `stripped_syntax` because this stays a model
    // node — it still has to answer `key`, `kind` and its own spelling.
    let elem = elem.unwrapped();
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
    Some(VecBuildElem {
        elem: elem.clone(),
        by_ref,
        elem_wrappers,
    })
}

/// Every distinct flattenable element type `T` that a scanned, declared function
/// takes as a `&[T]`/`Vec<T>` input — the set the synthetic `…VecNew/Push/Free`
/// externs are emitted for (once per type, shared across all such functions).
/// Deduped by [`TypeKey`] and sorted for deterministic output (mirrors
/// [`build_handle_destructor_items`]).
///
/// The key is the **canonical** element's, since that is what
/// [`vec_build_elem`] answers with — so `Vec<Payload>` and `Vec<Box<Payload>>`
/// land on one entry and get one trio, rather than two storages contending for
/// the name `payloadVec` (#296).
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
            if let Some(v) = vec_build_elem(ext, registry, &p.ty) {
                seen.insert(v.elem.key().as_str().to_string(), v.elem);
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
    emit: &prebindgen_registry::Emit,
) -> Vec<syn::Item> {
    let mut named: Vec<(String, syn::Item)> = Vec::new();
    for elem_reading in collect_vec_build_elem_types(ext, registry) {
        // Generated Rust spells `spell()`; the reading is what the plan
        // and the key are taken from.
        let elem = emit.spell(&elem_reading);
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
