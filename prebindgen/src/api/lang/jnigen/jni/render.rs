//! Kotlin source-file renderers (enum / data-class / typed-handle /
//! package / native / wrapper-fn) for the JNI back-end.
//!
//! Carved from the former `jni_kotlin_ext.rs`; shares the `jni` namespace
//! via `use super::*`.

use super::*;

// ── Safe-wrapper emitters ──────────────────────────────────────────────

/// One generated Kotlin `enum class` source — variants in
/// SCREAMING_SNAKE_CASE, each carrying the Rust discriminant as a
/// `val value: Int`, plus a `fromInt(value: Int)` companion. Mirrors
/// the hand-written `io.zenoh.qos.Priority` shape so adapter code that
/// already speaks the `.value` / `.fromInt(...)` idiom keeps working.
pub(crate) fn build_enum_class(class_name: &str, item_enum: &syn::ItemEnum) -> kt::KtClass {
    // Same discriminant source of truth the Rust `jint → variant` decode
    // uses, so Kotlin `value(N)` and the generated decode agree.
    let entries: Vec<kt::KtEnumEntry> =
        crate::api::lang::jnigen::util::enum_discriminant_values(item_enum)
            .into_iter()
            .map(|(ident, value)| kt::KtEnumEntry {
                name: crate::api::lang::jnigen::util::camel_to_screaming_snake(&ident.to_string()),
                args: Some(value.to_string()),
            })
            .collect();

    kt::KtClass::new(kt::ClassKind::Enum(entries), class_name)
        .vis(kt::Vis::Public)
        .kdoc(format!(
            "JVM-side surface for the native Rust `{}` enum.",
            item_enum.ident
        ))
        .ctor_param(
            kt::KtCtorParam::new("value", kt::KtType::int())
                .val()
                .vis(kt::Vis::Public),
        )
        // `@JvmStatic` exposes `fromInt` as a real static method on the enum
        // class itself (rather than only on the `Companion` nested class). The
        // generated struct-encoder calls it via `env.call_static_method`,
        // which wouldn't find a companion-only method.
        .companion(
            kt::KtClass::companion_object().vis(kt::Vis::Public).member(
                kt::KtFun::new("fromInt")
                    .vis(kt::Vis::Public)
                    .annotation("JvmStatic")
                    .param(kt::KtParam::new("value", kt::KtType::int()))
                    .returns(kt::KtType::cls(class_name))
                    .expr_body(kt::Code::new().line("entries.first { it.value == value }")),
            ),
        )
}

/// Build the Kotlin `data class` declaration for a `data_class`-declared
/// Rust struct. Returns the class plus the FQN imports its (pre-shortened)
/// field/factory type strings reference.
pub(crate) fn build_data_class(
    ext: &JniGen<impl JniGenState>,
    class_name: &str,
    item_struct: &syn::ItemStruct,
    registry: &Registry<KotlinMeta>,
) -> (kt::KtClass, BTreeSet<String>) {
    let fields_named = match &item_struct.fields {
        syn::Fields::Named(n) => &n.named,
        _ => {
            panic!(
                "render_data_class_source: struct `{}` must use named fields to map onto Kotlin data class properties",
                item_struct.ident
            )
        }
    };

    let mut imports: BTreeSet<String> = BTreeSet::new();
    let mut ctor_params: Vec<kt::KtCtorParam> = Vec::new();
    // Track per-field destructible (name, folded close strategy) so the
    // bottom emitter can produce a matching `close()` body for each.
    let mut destructible_fields: Vec<(String, crate::api::lang::jnigen::jni::FoldStrategy)> =
        Vec::new();
    for field in fields_named {
        let field_ident = field.ident.as_ref().unwrap_or_else(|| {
            panic!(
                "render_data_class_source: struct `{}` has an unnamed field in named-fields context",
                item_struct.ident
            )
        });
        let kotlin_field_name = kt_snake_to_camel(&field_ident.to_string());

        // Projection field (opaque handle or value class): the typed Kotlin
        // type (`ZKeyExpr?`, `List<ZKeyExpr>`, `ZenohId`, `List<ZenohId>`, …)
        // is derived from the folded `Projection` the type-unfolding mechanism
        // propagated onto this field's converter metadata, instead of a
        // syntactic `Option<T>` peel. Closeable `close()` emission applies
        // only to `Handle` kind (and only when owned); value classes are
        // erased to their inner wire, so they never close.
        let field_projection = registry
            .output_entry(&field.ty)
            .and_then(|e| e.metadata.projection.clone())
            .or_else(|| {
                registry
                    .input_entry(&field.ty)
                    .and_then(|e| e.metadata.projection.clone())
            });
        if let Some(h) = field_projection {
            let fqn = ext
                .kotlin_fqn(&h.leaf_key)
                .map(|v| v.to_string())
                .unwrap_or_else(|| {
                    panic!(
                        "render_data_class_source: projection field `{}.{}` leaf `{}` has no \
                         Kotlin FQN registered (ptr_class / value_class)",
                        item_struct.ident, field_ident, h.leaf_key
                    )
                });
            let short = register_fqn(&fqn, &mut imports);
            ctor_params.push(
                kt::KtCtorParam::new(
                    &kotlin_field_name,
                    handle_kt_type(&h.strategy, &kt::KtType::cls(short)),
                )
                .val(),
            );
            if matches!(
                h.kind,
                crate::api::lang::jnigen::jni::ProjectionKind::Handle
            ) && h.owned
            {
                destructible_fields.push((kotlin_field_name, h.strategy));
            }
            continue;
        }

        let kotlin_ty = registry
            .output_entry(&field.ty)
            .and_then(|e| e.metadata.kotlin_name.clone())
            .or_else(|| registry.input_entry(&field.ty).and_then(|e| e.metadata.kotlin_name.clone()))
            .unwrap_or_else(|| {
                panic!(
                    "render_data_class_source: field `{}.{}` has no Kotlin type mapping; register converters before declaring data_class",
                    item_struct.ident,
                    field_ident
                )
            });
        let ty = register_kt_type(&kotlin_ty, &mut imports);
        // `Option<T>` whose wire is a JNI primitive (jlong/jint/jboolean/…)
        // and that *isn't* an opaque handle (handled above) is encoded by
        // the struct emitter as the bare primitive with a sentinel for
        // `None` (0 / 0.0 / false). The Kotlin field must match that JVM
        // slot: declare it non-nullable so the constructor signature
        // stays primitive (`J` vs `Ljava/lang/Long;`). Nullable boxing
        // for non-handle primitives would require generator-side changes
        // in `struct_output_body` to `Long.valueOf(...)`.
        let wire = registry
            .output_entry(&field.ty)
            .map(|e| e.destination.clone());
        let primitive_wire = wire
            .as_ref()
            .map(|w| crate::api::lang::jnigen::jni::is_jni_primitive(w))
            .unwrap_or(false);
        let ty = if is_option_type(&field.ty) && !primitive_wire {
            ty.nullable()
        } else {
            ty
        };
        ctor_params.push(kt::KtCtorParam::new(&kotlin_field_name, ty).val());
    }

    // `fromParts` companion factory — recursively flattened the same way as the
    // native `flatten_struct_encode`: nested data-class fields are inlined as
    // their leaf wires, so native builds the whole object graph with ONE
    // `call_static_method`. Any nested child FQN it references is registered
    // into `imports`.
    let (factory_params, factory_reconstruct) =
        flatten_struct_factory(ext, registry, item_struct, "", class_name, &mut imports, 0)
            .unwrap_or_else(|| {
                panic!(
                    "render_data_class_source: could not build fromParts factory for `{class_name}`"
                )
            });

    let mut class = kt::KtClass::new(kt::ClassKind::Data, class_name).vis(kt::Vis::Public);
    for p in ctor_params {
        class = class.ctor_param(p);
    }
    // Supertype clause: a data class with a destructible native-handle field
    // implements `AutoCloseable`; otherwise no supertype.
    if !destructible_fields.is_empty() {
        class = class.supertype(kt::KtType::cls("AutoCloseable"), None);
        // `close()` walks every destructible field via its folded close
        // strategy. `JNINativeHandle.close()` is idempotent
        // (Cleaner.Cleanable.clean() invokes exactly once), so calling
        // this multiple times — or alongside the cleaner's own firing on
        // GC — is safe. NOTE: `data class` copy() shares the handle
        // reference between copies; if you intend to close independently,
        // don't copy this class.
        let mut body = kt::Code::new();
        for (fname, strategy) in &destructible_fields {
            body = body.line(render_handle_close(strategy, fname));
        }
        class = class.member(kt::KtFun::new("close").modifier("override").body(body));
    }
    // `fromParts` factory: native (`struct_output_body`) makes ONE
    // `call_static_method` passing the whole graph's flattened leaf wires;
    // this factory reassembles it (incl. nested `Child.fromParts(...)`) in
    // JVM bytecode. `public`, not `internal`: an `internal` fun is mangled
    // to `fromParts$<module>`, unresolvable by native (`NoSuchMethodError`).
    let mut factory = kt::KtFun::new("fromParts")
        .vis(kt::Vis::Public)
        .annotation("JvmStatic")
        .returns(kt::KtType::cls(class_name))
        .expr_body(kt::Code::new().line(factory_reconstruct));
    for (name, ty) in &factory_params {
        factory = factory.param(kt::KtParam::new(name, ty.clone()));
    }
    class = class.companion(
        kt::KtClass::companion_object()
            .vis(kt::Vis::Public)
            .member(factory),
    );
    (class, imports)
}

/// Render one typed-handle Kotlin source file. Pure-shell form (with
/// the closure `|n| format!("{n}ViaJNI")` installed via
/// [`JniGen::kotlin_fun_name_mangle`]):
///
/// ```kotlin
/// public class JNIFoo(initialPtr: Long) : NativeHandle(initialPtr) {
///     public fun free() = free { freePtrViaJNI(it) }
///     private external fun freePtrViaJNI(ptr: Long)
/// }
/// ```
///
/// When `promoted_functions` is non-empty, one extra instance method is
/// appended per `#[prebindgen]` fn — the matching opaque first param
/// (Rust type-key = `promoted_rust_key`) is dropped from the Kotlin
/// signature, and its `withPtr` / `consume` wrapper uses the
/// inherited [`NativeHandle`] scope.
///
/// The free-pointer extern name is built as
/// `<mangle_fun("freePtr")>`. Kotlin/JVM's JNI name mangler binds it
/// to the matching `Java_<pkg>_<class>_<mangle_fun("freePtr")>`
/// extern on the Rust side (the auto-generated destructor).
pub(crate) fn build_typed_handle(
    ext: &JniGen<impl JniGenState>,
    registry: &Registry<KotlinMeta>,
    class_name: &str,
    rust_doc_name: &str,
    key: &TypeKey,
    imports: &mut BTreeSet<String>,
) -> kt::KtClass {
    // The typed handle is a pure shell — `ptr` slot + `close()`/`take()` +
    // the `freePtr` extern. All functions are emitted as flat free functions
    // in their namespace package; nothing is promoted onto the class.
    //
    // Every typed handle extends the shared `NativeHandle` base, which owns
    // the `@Volatile` pointer slot (`ptr`) and its monitor — that common
    // supertype is what lets `render_wrapper_fn` collect a `List<NativeHandle>`
    // and lock it in one pointer-sorted, deadlock-safe pass. The subclass keeps
    // its own type-specific `close()`/`take()`/`freePtr`.
    let free_extern = ext.mangle_fun("freePtr");
    let base_fqn = if ext.package.is_empty() {
        "NativeHandle".to_string()
    } else {
        format!("{}.NativeHandle", ext.package)
    };
    let members = ext.class_members.get(key).map(Vec::as_slice).unwrap_or(&[]);
    if !members.is_empty() && !ext.package.is_empty() {
        imports.insert(format!("{}.{}", ext.package, ext.jni_native_class_name()));
    }

    // Companion object: the `@JvmStatic external fun freePtr(ptr: Long)` called
    // by `close()`, plus one **factory** member per `.constructor(f, name)`
    // (a free wrapper — no receiver — returning the class).
    let mut companion = kt::KtClass::companion_object().vis(kt::Vis::Public).member(
        kt::KtFun::new(free_extern.clone())
            .annotation("JvmStatic")
            .modifier("external")
            .param(kt::KtParam::new("ptr", kt::KtType::long())),
    );
    for m in members.iter().filter(|m| m.kind == MemberKind::Constructor) {
        if let Some((item_fn, _)) = registry.functions.get(&m.rust_ident) {
            if let Some(f) = render_wrapper_fn(
                ext,
                item_fn,
                registry,
                imports,
                Some(m.kotlin_name.as_str()),
                None,
            ) {
                companion = companion.member(f);
            }
        }
    }

    let mut class = kt::KtClass::new(kt::ClassKind::Plain, class_name)
        .vis(kt::Vis::Public)
        .kdoc(format!(
            "Typed handle for a native Zenoh `{rust_doc_name}`."
        ))
        .ctor_param(kt::KtCtorParam::new("initialPtr", kt::KtType::long()))
        .supertype(kt::KtType::cls(base_fqn), Some("initialPtr"))
        .member(
            kt::KtFun::new("close")
                .annotation("Synchronized")
                .modifier("override")
                .body(
                    kt::Code::new()
                        .line("val p = ptr")
                        .blk("if (p != 0L) {", |c| {
                            c.line("ptr = 0L").line(format!("{free_extern}(p)"))
                        }),
                ),
        )
        // Transfer ownership of the native pointer into a fresh handle,
        // leaving this one empty. Lets a callback receiver retain a handle
        // that the framework would otherwise `close()` when the callback
        // returns.
        .member(
            kt::KtFun::new("take")
                .vis(kt::Vis::Public)
                .annotation("Synchronized")
                .returns(kt::KtType::cls(class_name))
                .body(
                    kt::Code::new()
                        .line("val p = ptr")
                        .line("ptr = 0L")
                        .line(format!("return {class_name}(p)")),
                ),
        )
        .companion(companion);

    // Promoted instance methods: each `.accessor(f, name)` / `.method(f, name)`
    // becomes an instance method (receiver bound to `this`), delegating to the
    // same centralized `JNINative` extern as a free wrapper would.
    for m in members
        .iter()
        .filter(|m| matches!(m.kind, MemberKind::Accessor | MemberKind::Method))
    {
        if let Some((item_fn, _)) = registry.functions.get(&m.rust_ident) {
            if let Some(f) = render_wrapper_fn(
                ext,
                item_fn,
                registry,
                imports,
                Some(m.kotlin_name.as_str()),
                Some(key),
            ) {
                class = class.member(f);
            }
        }
    }
    class
}

/// Render one `external fun <mangle_fun(name)>(…): <wire-return>` line
/// at the JNI **wire** level (matches what the Rust extern receives):
///   * opaque-handle (Borrow/Consume) → jlong → `Long`
///   * `enum_class`                  → jint  → `Int` (call passes `.value`)
///   * `Any` (impl-Into Dispatch)     → JObject → `Any`
///   * everything else                → entry's high-level Kotlin name
///
/// Opaque returns become `Long`; every other return uses
/// [`classify_return`]'s `kt_return` (Unit is empty string).
/// Returns `None` if any parameter's input converter isn't resolved.
///
/// Expand a function's inputs into the effective parameter list seen by the
/// Kotlin wrapper + extern declaration: a parameter carrying a
/// constructor-expansion [`FoldPlan`] is replaced by its flattened leaves
/// (each a normal `(name, type)`); every other parameter passes through. The
/// Rust extern (`emit_jni_function_wrapper`) folds the leaves back into the
/// built value separately.
pub(crate) fn effective_inputs(
    registry: &Registry<KotlinMeta>,
    f: &syn::ItemFn,
) -> Vec<(syn::Ident, syn::Type)> {
    let mut out = Vec::new();
    for input in &f.sig.inputs {
        let syn::FnArg::Typed(pt) = input else {
            continue;
        };
        let syn::Pat::Ident(pid) = &*pt.pat else {
            continue;
        };
        if let Some(plan) = registry
            .expansion_plans
            .get(&(f.sig.ident.clone(), pid.ident.clone()))
        {
            for leaf in &plan.leaves {
                out.push((leaf.name.clone(), leaf.ty.clone()));
            }
        } else {
            out.push((pid.ident.clone(), (*pt.ty).clone()));
        }
    }
    out
}

/// True for an `Iterable` fold delivery, including one wrapped in a single
/// `Optional` layer (`Option<Vec<T>>` → a nullable `List`). Selects the fold
/// surface (`acc` + `fold`) over a scalar `Optional`/`Base` builder.
pub(crate) fn is_iterable_fold(shape: &crate::api::core::unfold::UnfoldShape) -> bool {
    use crate::api::core::unfold::UnfoldShape;
    matches!(shape, UnfoldShape::Iterable(_))
        || matches!(shape, UnfoldShape::Optional((), inner) if matches!(**inner, UnfoldShape::Iterable(_)))
}

pub(crate) fn render_extern_decl(
    ext: &JniGen<impl JniGenState>,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
) -> Option<kt::Code> {
    let rust_name = f.sig.ident.to_string();
    let kt_name = kt_snake_to_camel(&rust_name);
    let jni_call = ext.mangle_fun(&kt_name);

    let mut params: Vec<(String, String)> = Vec::new();
    for (eff_ident, eff_ty) in effective_inputs(registry, f) {
        let name = kt_param_name(&eff_ident.to_string());
        let arg_ty = &eff_ty;

        // Flattenable data_class param → expand into its leaf wire params
        // (same plan the native wrapper + call site use).
        if let Some(plan) = crate::api::lang::jnigen::jni::build_flat_input_plan(
            ext, registry, &eff_ident, arg_ty, "",
        ) {
            for leaf in &plan.leaves {
                let short = register_fqn(&leaf.kt_wire_ty, imports);
                params.push((leaf.kt_name.clone(), short));
            }
            continue;
        }

        // Bare `Option<primitive>` / `Option<enum>` param → a `(present:
        // Boolean, value: <Prim>)` pair (no boxed `java.lang.*` wire). Same plan
        // the native wrapper + Kotlin call site use.
        if let Some(sp) = crate::api::lang::jnigen::jni::build_option_scalar_input_plan(
            ext, registry, &eff_ident, arg_ty,
        ) {
            let pshort = register_fqn(&"Boolean".to_string(), imports);
            params.push((sp.present_kt.clone(), pshort));
            let vshort = register_fqn(&sp.value_kt_type, imports);
            params.push((sp.value_kt.clone(), vshort));
            continue;
        }

        // Slice/Vec of a flattenable data_class → a single `jlong` Vec-handle
        // param (the Rust extern decodes the boxed `Vec<T>`). Elements cross
        // through the synthetic `…VecPush` extern, not this one. Must match the
        // `jlong` wire emitted by `emit_input_param`'s VecBuild branch.
        if crate::api::lang::jnigen::jni::vec_build_elem(ext, registry, arg_ty).is_some() {
            params.push((name, "Long".to_string()));
            continue;
        }

        let entry = registry.input_entry(arg_ty)?;

        // An opaque-**handle** projection (direct `&T`/`T`, `Option<&T>`, or
        // by-value `Option<T>`) crosses the JNI wire as a primitive `jlong`
        // with `0` encoding `None` — so the extern param is a non-null `Long`,
        // and the `?` lives only on the typed-wrapper surface. (`value_blob`
        // projections are NOT handles; they keep their `ByteArray` wire and
        // can be nullable.)
        let proj_is_handle = entry
            .metadata
            .projection
            .as_ref()
            .map(|p| p.kind == crate::api::lang::jnigen::jni::ProjectionKind::Handle)
            .unwrap_or(false);
        let optional = is_option_type(arg_ty) && !proj_is_handle;

        let kt_type_raw = if proj_is_handle {
            kt::KtType::long()
        } else if ext.is_kotlin_enum(&enum_probe_type(arg_ty)) {
            // Enum (incl. `Option<enum>`) crosses as jint → Kotlin `Int`; the
            // wrapper passes `.value` / `?.value`. The Rust converter unboxes a
            // `java.lang.Integer`, so the extern must declare `Int`/`Int?`, never
            // the enum object.
            kt::KtType::int()
        } else {
            entry.metadata.kotlin_name.clone()?
        };
        // The extern block is raw text — render the registered/shortened type.
        let short = register_kt_type(&kt_type_raw, imports).to_string();
        let suffix = if optional { "?" } else { "" };
        params.push((name, format!("{short}{suffix}")));
    }
    // Output (data) expansion: a **callback** delivery (`deconstruct_output`)
    // appends the lambda(s) before the error sink and returns the erased result
    // (`Any?`). A **return** delivery (`convert_output`) appends nothing and
    // returns the real converted wire (handled in `wire_return` below, keyed on
    // `convert_out_ty`).
    use crate::api::core::unfold::Delivery;
    let unfold = registry.unfold_plans.get(&f.sig.ident);
    let callback_unfold = unfold.filter(|p| p.delivery == Delivery::Callback);
    if let Some(plan) = callback_unfold {
        if is_iterable_fold(&plan.shape) {
            // `acc` is the unbounded accumulator `A` (may be nullable) → `Any?`;
            // `fold` is the non-null adapter callback.
            params.push(("acc".to_string(), "Any?".to_string()));
            params.push(("fold".to_string(), "Any".to_string()));
        } else {
            params.push(("build".to_string(), "Any".to_string()));
        }
    }
    // Trailing error-sink callback — every extern accepts one (see
    // `signal_error` / the wrapper's default sink). `Any` at the wire level
    // (JObject); the wrapper passes an `ErrorSink` instance.
    params.push(("errorSink".to_string(), "Any".to_string()));

    let wire_return = if callback_unfold.is_some() {
        "Any?".to_string()
    } else {
        // For a `convert_output` (Return) the wire is the converted single
        // value's; otherwise the function's own return.
        let ret_decl: syn::ReturnType = match unfold.and_then(|p| p.convert_out_ty.clone()) {
            Some(cv) => syn::parse_quote!(-> #cv),
            None => f.sig.output.clone(),
        };
        let (kt_return, projection) = classify_return(ext, &ret_decl, registry, imports)?;
        // enum_class returns cross the JNI wire as jint → Kotlin `Int`.
        // The public wrapper converts back using `EnumType.fromInt(Int)`.
        let is_enum_return = return_is_kotlin_enum(ext, &ret_decl, registry);
        // `Option<enum>` returns cross as the boxed discriminant → `Int?`.
        let is_option_enum_return = return_is_kotlin_option_enum(ext, &ret_decl, registry);
        // JNI extern's wire return: handle projections wire as `Long` (the boxed
        // jlong gets wrapped); value-class projections wire as their inner
        // converter's Kotlin type folded through the projection's strategy (the
        // value class is erased to that inner). Enums wire as `Int` (`Int?`
        // under `Option`); everything else is the declared return.
        match &projection {
            Some(p) => projection_wire_return(p),
            None if is_enum_return => "Int".to_string(),
            None if is_option_enum_return => "Int?".to_string(),
            None => kt_return.map(|t| t.to_string()).unwrap_or_default(),
        }
    };

    let formals: Vec<String> = params.iter().map(|(n, t)| format!("{n}: {t}")).collect();
    let ret_suffix = if wire_return.is_empty() {
        String::new()
    } else {
        format!(": {wire_return}")
    };
    let head = format!("external fun {jni_call}");
    let single = format!("{head}({}){ret_suffix}", formals.join(", "));
    // Externs render as members of `object JNINative` at one indent level
    // (4 columns). Past the shared signature-width budget, wrap to one
    // parameter per line — the same treatment the model renderer gives the
    // public wrappers — so long native declarations stay readable.
    const EXTERN_INDENT_COLS: usize = 4;
    if !formals.is_empty() && EXTERN_INDENT_COLS + single.len() > kt::render::MAX_SIGNATURE_WIDTH {
        let opener = format!("{head}(");
        let closer = format!("){ret_suffix}");
        Some(kt::Code::new().blk_with(opener, closer, move |mut c| {
            for f in &formals {
                c = c.line(format!("{f},"));
            }
            c
        }))
    } else {
        Some(kt::Code::new().line(single))
    }
}

struct Param {
    kt_name: String,
    kt_type: kt::KtType,
    mode: ParamMode,
    /// `true` when the param's Rust type is a `enum_class`-declared enum: the
    /// high-level Kotlin signature uses the typed enum (`Priority`), but the
    /// underlying JNI `external fun` declares the param as `Int` (jint wire).
    /// The wrapper bridges the two by passing `<name>.value` at the call site.
    as_enum_value: bool,
}

enum ParamMode {
    Borrow,  // &T opaque-handle → withPtr
    Consume, // T  opaque-handle → consume
    /// `Option<&T>` / `Option<&mut T>` opaque-handle → `withPtrOrZero`.
    /// Nullable typed-handle param; the wrapper runs the body under the read
    /// lock when the handle is non-null and with `0L` when null. The Rust
    /// converter materializes `Option<OwnedObject<T>>` and the call site uses
    /// `.as_deref()` / `.as_deref_mut()`.
    BorrowNullable,
    /// By-value `Option<T>` opaque-handle → nullable consume. Same as
    /// [`Self::Consume`] but the typed param is nullable (`T?`): `0L` when
    /// absent, and the handle's `ptr` slot is nulled after the call only when
    /// present. The Rust converter consumes the `Box` to `Option<T>`.
    ConsumeNullable,
    PassThrough,
    /// Value-projection param (`value_blob`): a Kotlin `@JvmInline value class`
    /// that is **not** a lockable handle. The Kotlin param type is the
    /// value-class FQN; the call site passes the unwrapped inline-class field
    /// (`<name>.<field>`) so the `JNINative` extern receives the erased inner
    /// wire (e.g. `ByteArray`). No lock.
    ValueUnwrap {
        field: String,
    },
    /// Flattenable `data_class` param: the high-level Kotlin signature keeps the
    /// typed object, but the `JNINative` call destructures it into the leaf
    /// access expressions (no `JObject` crosses, so the Rust side skips
    /// `env.get_field(...)`). The strings are the per-leaf call-site
    /// expressions in plan order.
    FlattenStruct {
        accesses: Vec<String>,
    },
    /// `&[T]` / `Vec<T>` of a flattenable data_class `T`: the public Kotlin
    /// signature keeps `List<T>`, but the wrapper allocates a Rust-side `Vec<T>`
    /// (opaque `jlong` handle), pushes each element's decoupled leaves in a loop
    /// via `<base>Push`, passes the handle to the extern, then frees it in a
    /// `finally`. No `JObject` list crosses, so the Rust side skips per-element
    /// `env.get_field(...)`. `base` is the helper method base (`payloadVec`);
    /// `elem_accesses` are the per-element leaf push expressions rooted at the
    /// loop variable (`__e.id`, `__e.seq`, …), in plan order.
    VecBuild {
        base: String,
        elem_accesses: Vec<String>,
    },
    /// `impl Fn(args)` callback param: typed Kotlin lambda over the flattened
    /// leaves of each arg's callback plan (whole arg when plan-less), erased to
    /// `Any` at the extern tier — the same shape as the unfold `build`/`onError`
    /// lambdas. `call_arg` is the call-site expression: the param itself, or a
    /// value-blob rebuilding adapter.
    Callback {
        call_arg: String,
    },
    /// Bare `Option<primitive>` / `Option<enum>` param decomposed into a
    /// `(present: Boolean, value: <prim>)` pair so no boxed `java.lang.*`
    /// crosses (and the Rust side does no `intValue()` unboxing). The public
    /// Kotlin signature keeps `T?`; the call site passes `present_expr`
    /// (`<name> != null`) then `value_expr` (`<name> ?: 0` / `<name>?.value ?:
    /// 0`). See [`crate::api::lang::jnigen::jni::OptionScalarInputPlan`].
    OptionScalar {
        present_expr: String,
        value_expr: String,
    },
}

struct Opaque {
    /// Kotlin param name (e.g. `"b"`).
    name: String,
    /// Object to synchronize on and read the pointer from (`<name>`).
    target: String,
    /// Statement that nulls the pointer slot after consume (`"<target>.ptr =
    /// 0L"`), or `None` for borrow modes.
    consume_null: Option<String>,
    /// `true` for `Option<&T>` — nullable param, branches before lock.
    nullable: bool,
}

/// Peel `&` / `Option<…>` / `Option<&…>` layers and return the inner type's
/// [`TypeKey`] — used to match an accessor's receiver parameter against its
/// owning class key in [`render_wrapper_fn`].
fn peel_receiver_key(ty: &syn::Type) -> TypeKey {
    let core = match ty {
        syn::Type::Reference(r) => &*r.elem,
        other => other,
    };
    if let syn::Type::Path(tp) = core {
        if let Some(seg) = tp.path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = ab.args.first() {
                        let inner_core = match inner {
                            syn::Type::Reference(r) => &*r.elem,
                            other => other,
                        };
                        return TypeKey::from_type(inner_core);
                    }
                }
            }
        }
    }
    TypeKey::from_type(core)
}

/// Build a single top-level (free-function) wrapper as a [`kt::KtFun`].
/// Returns `None` if the function has a parameter whose Kotlin type isn't
/// registered (in that case we skip the function rather than panicking — the
/// legacy `JNINative.kt` retains the unwrapped external fun so callers still
/// have an escape hatch).
///
/// Every `#[prebindgen]` function is emitted as a flat namespaced free function
/// — opaque-handle parameters are ordinary `NativeHandle` params, locked via the
/// per-call `withSortedHandleLocks` scaffold.
///
/// When `receiver_key` is `Some(class_key)` the function is emitted as an
/// **instance method** of that class: the first parameter whose (peeled) Rust
/// type equals `class_key` is dropped from the signature and bound to `this`
/// (the inherited `NativeHandle` scope for a `ptr_class` — `this.ptr` + lock —
/// or `this.bytes` for a `value_class` blob). The JNINative extern/call is
/// unchanged (keyed on the Rust ident), so only the Kotlin wrapper relocates.
pub(crate) fn render_wrapper_fn(
    ext: &JniGen<impl JniGenState>,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
    kotlin_name_override: Option<&str>,
    receiver_key: Option<&TypeKey>,
) -> Option<kt::KtFun> {
    let rust_name = f.sig.ident.to_string();
    // The Kotlin extern in `JNINative` is keyed on the Rust ident
    // (`kt_snake_to_camel(rust_name)` → `ext.mangle_fun`). The per-entry
    // `.name("...")` override only changes the *user-facing* Kotlin
    // wrapper name; the JNI call still has to hit the one extern that
    // the Rust extern actually emits.
    let default_kt_name = kt_snake_to_camel(&rust_name);
    let kt_name = match kotlin_name_override {
        Some(n) => n.to_string(),
        None => default_kt_name.clone(),
    };
    let jni_call = ext.mangle_fun(&default_kt_name);

    let (params, receiver_idx) = classify_params(ext, f, registry, imports, receiver_key)?;
    let out = classify_output(ext, f, registry, imports)?;
    let body_expr = build_native_call(ext, &jni_call, &params, &out);

    // Collect the opaque-handle params so we can scaffold pointer-ordered
    // synchronized blocks around them.
    let opaques = collect_opaques(&params);
    let is_unit = out.kt_return.is_none();
    let r_ty = out.kt_return.clone().unwrap_or_else(kt::KtType::unit);
    let sink = error_sink_parts(ext, f, registry, imports, &r_ty)?;
    let prelock_guards = render_prelock_guards(&opaques, &sink.guard_args, is_unit);
    let core_expr = render_core_expr(ext, &opaques, &body_expr, imports);

    let mut fun = kt::KtFun::new(&kt_name).vis(kt::Vis::Public);
    if let Some(g) = &out.generic {
        fun = fun.generic(g);
    }
    for (i, p) in params.iter().enumerate() {
        // The receiver param is bound to `this` — not a rendered parameter.
        if Some(i) == receiver_idx {
            continue;
        }
        fun = fun.param(kt::KtParam::new(&p.kt_name, p.kt_type.clone()));
    }
    // The error callback — **required**: the generated code never throws; the
    // consumer decides how a failure surfaces (e.g. by throwing its own type).
    // When an output-expansion builder/fold lambda exists, it must remain the
    // **trailing** lambda (Kotlin trailing-lambda call syntax), so `onError` is
    // placed *before* it — but *after* any non-lambda `builder_lead` (`acc: A`),
    // which is passed positionally. Without a builder lambda, `onError` is the
    // last param.
    let onerr = kt::KtParam::new("onError", sink.onerr_type.clone());
    if let Some((bp_name, bp_ty)) = &out.builder_param {
        if let Some((lead_name, lead_ty)) = &out.builder_lead {
            fun = fun.param(kt::KtParam::new(lead_name, lead_ty.clone()));
        }
        fun = fun
            .param(onerr)
            .param(kt::KtParam::new(bp_name, bp_ty.clone()))
            .annotation("Suppress(\"UNCHECKED_CAST\")");
    } else {
        fun = fun.param(onerr);
    }
    if let Some(rt) = &out.kt_return {
        fun = fun.returns(rt.clone());
    }
    let body = render_body(ext, &params, &prelock_guards, &sink, &core_expr, is_unit);
    // The body is assembled as flat text (`core_expr` nests run/lock blocks);
    // the generator recomputes its indentation from brace structure.
    Some(fun.body(kt::Code::raw_reindent_wrapped(body.trim_end())))
}

/// The classified output side of a wrapper: return type, projection wrap,
/// output-expansion (builder/fold) params, and the extra call-site args —
/// everything the call-expression builder and the signature assembly must
/// agree on.
struct OutputPlan {
    kt_return: Option<kt::KtType>,
    /// Kotlin-newtype return (opaque handle / value class) — the wrap the
    /// call expression folds around the extern result.
    projection: Option<Projection>,
    /// Trailing **lambda** param (`build` / `fold`) of an output expansion.
    builder_param: Option<(String, kt::KtType)>,
    /// Non-lambda lead param (`acc: A`) — precedes `onError` positionally.
    builder_lead: Option<(String, kt::KtType)>,
    /// Type variable (`R` / `A`) when the wrapper is generic.
    generic: Option<String>,
    /// Extra call-site args injected before `__cap` (builder/adapter, or
    /// `acc` + fold callback for `Iterable`).
    unfold_call_args: Vec<String>,
    /// Callback delivery: cast the extern's erased `Any?` to `R`/`A`.
    cast_return: bool,
    /// enum_class return crossing as jint — wrap with `fromInt`.
    is_enum_return: bool,
    /// `Option<enum>` return crossing boxed — `?.let { fromInt(it) }`.
    is_option_enum_return: bool,
}

/// The `onError` wiring: the handler's Kotlin type, the capture holder, and
/// the two prebuilt argument lists (post-call redispatch / pre-lock guard).
struct ErrorSink {
    onerr_type: kt::KtType,
    /// Short name of the generated per-thread raw capture holder.
    capture_short: String,
    /// je/ze args for the post-call `onError.run(...)` redispatch.
    call_args: String,
    /// Wrapped-default args for the pre-lock closed-handle guard.
    guard_args: String,
}

/// Classify every effective input into a [`Param`] (Kotlin name/type +
/// call-site [`ParamMode`]). Returns the params plus the index of the
/// instance-method receiver (the first param whose peeled type matches
/// `receiver_key`), which is bound to `this` and dropped from the signature.
fn classify_params(
    ext: &JniGen<impl JniGenState>,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
    receiver_key: Option<&TypeKey>,
) -> Option<(Vec<Param>, Option<usize>)> {
    let mut receiver_idx: Option<usize> = None;
    let mut params: Vec<Param> = Vec::new();
    for (eff_ident, eff_ty) in effective_inputs(registry, f) {
        let mut name = kt_param_name(&eff_ident.to_string());
        let arg_ty = &eff_ty;

        // Instance-method receiver: the first parameter whose peeled Rust type
        // is the owning class binds to `this` (so `this_ptr`/`this.ptr`/lock or
        // `this.bytes` fall out of the normal param handling) and is dropped
        // from the rendered signature.
        if receiver_idx.is_none() {
            if let Some(rk) = receiver_key {
                if &peel_receiver_key(arg_ty) == rk {
                    receiver_idx = Some(params.len());
                    name = "this".to_string();
                }
            }
        }

        // `impl Fn(args)` param: a generated typed `fun interface`
        // (`<ArgShorts>Callback`) whose `run` parameters are the flattened
        // leaves of each arg's callback plan (the arg whole when plan-less).
        // The extern receives it erased (`Any`) and the native trampoline
        // calls the typed `run` — value-blob leaves surface as their raw
        // `ByteArray` wire (the SDK wraps), so no call-site adapter exists.
        // Lambda-literal call sites SAM-convert unchanged.
        if let Some(cb_args) = extract_fn_trait_args(arg_ty) {
            let spec = callback_iface_spec(ext, registry, &cb_args)?;
            let kt_type = spec.kt_ref(vec![]);
            // The extern receives the RAW twin: the generated `asRaw()`
            // proxy (built once per registration) wraps raw leaves into the
            // typed objects the user's interface declares.
            let call_arg = if spec.needs_raw() {
                imports.insert(format!("{}.asRaw", spec.package));
                format!("{name}.asRaw()")
            } else {
                name.clone()
            };
            params.push(Param {
                kt_name: name.clone(),
                kt_type,
                mode: ParamMode::Callback { call_arg },
                as_enum_value: false,
            });
            continue;
        }

        // Strip leading reference for the type-map lookup; the registry's
        // input entry is keyed by the param as-written.
        let entry = registry.input_entry(arg_ty)?;
        // Opaque-handle params surface as their typed-handle FQN — every
        // handle is self-contained (its own ptr slot + monitor), so the
        // wrapper body inlines synchronized blocks directly on the typed
        // receiver. Detection flows from the folded `Projection` — present
        // for both `&T` and by-value `T` (the `owned` flag is orthogonal
        // to presence) — so it's the same source of truth the typed-surface
        // emitters use.
        let is_opaque = entry.metadata.projection.is_some();

        // `Option<&T>` / `Option<&mut T>` for opaque T marks the param
        // nullable; the wrapper body branches on null before lock selection.
        let is_opt_ref_opaque = is_opaque && is_option_ref(arg_ty);
        let (kt_type_raw, optional) = if is_opaque {
            let h = entry.metadata.projection.as_ref()?;
            let fqn = ext.kotlin_fqn(&h.leaf_key).map(|v| v.to_string())?;
            // Any `Option<_>` opaque param (borrowed `Option<&T>` or by-value
            // `Option<T>`) is nullable; value projections likewise. The handle
            // wire stays `jlong` with `0` = absent, so the `?` is purely the
            // typed-wrapper surface.
            (kt::KtType::cls(fqn), is_option_type(arg_ty))
        } else {
            // Read the Kotlin name straight off the resolved entry's
            // metadata — the rank-N handler that built this converter
            // is also the one that derived the Kotlin name (primitives
            // from `kotlin_for_wire`, wrappers inherit from inner,
            // user-declared decoders from `with_kotlin_name`).
            let kt = entry.metadata.kotlin_name.clone()?;
            let opt = is_option_type(arg_ty);
            (kt, opt)
        };

        // A projection can be a lockable opaque **handle** or a non-lockable
        // **value projection** (`value_blob` — an inline value class). Only
        // handles participate in the lock scaffold and pass a `_ptr`; value
        // projections pass their unwrapped inner field.
        let proj_kind = entry.metadata.projection.as_ref().map(|p| p.kind.clone());
        let is_handle = matches!(
            proj_kind,
            Some(crate::api::lang::jnigen::jni::ProjectionKind::Handle)
        );
        let is_value_proj = matches!(
            proj_kind,
            Some(crate::api::lang::jnigen::jni::ProjectionKind::ValueBlob)
        );

        // Mode: handle → Borrow/Consume by Rust syntactic shape (locked).
        // Value projection → ValueUnwrap (inline-class field, no lock).
        // Everything else (primitives, callbacks, data classes) passes through.
        //
        // Flattenable data_class params are detected first: the high-level
        // signature keeps the typed object (`kt_type_raw`), but the
        // `JNINative` call destructures it into leaf args (same plan the
        // native wrapper + extern decl consume). The decision is purely
        // type-based so all three sites agree.
        let flat_plan = crate::api::lang::jnigen::jni::build_flat_input_plan(
            ext,
            registry,
            &eff_ident,
            arg_ty,
            name.as_str(),
        );
        let mode = if let Some((elem, _by_ref)) =
            crate::api::lang::jnigen::jni::vec_build_elem(ext, registry, arg_ty)
        {
            // Slice/Vec of a flattenable data_class: build the Rust-side Vec by
            // pushing each element's leaves, pass the handle (see the body
            // assembly + `build_vec_build_helper_items`). High-level signature
            // stays `List<T>` (kt_type_raw, registered below).
            let h = crate::api::lang::jnigen::jni::vec_build_helpers(ext, registry, &elem)
                .expect("vec_build_elem Some ⇒ vec_build_helpers Some");
            let elem_accesses = h
                .plan
                .leaves
                .iter()
                .filter(|l| !l.is_present_flag)
                .map(|l| l.kt_access.clone())
                .collect();
            ParamMode::VecBuild {
                base: h.base,
                elem_accesses,
            }
        } else if let Some(sp) = crate::api::lang::jnigen::jni::build_option_scalar_input_plan(
            ext, registry, &eff_ident, arg_ty,
        ) {
            // Bare `Option<primitive>` / `Option<enum>`: cross as a `(present,
            // value)` pair (no boxed object). The high-level signature keeps
            // `T?` (computed below); only the call-site args split in two.
            let present_expr = format!("{name} != null");
            let value_expr = if sp.is_enum {
                format!("{name}?.value ?: {}", sp.value_kt_zero)
            } else {
                format!("{name} ?: {}", sp.value_kt_zero)
            };
            ParamMode::OptionScalar {
                present_expr,
                value_expr,
            }
        } else if let Some(plan) = flat_plan {
            ParamMode::FlattenStruct {
                accesses: plan.leaves.iter().map(|l| l.kt_access.clone()).collect(),
            }
        } else if is_handle {
            let borrow = matches!(arg_ty, syn::Type::Reference(_));
            if is_opt_ref_opaque {
                ParamMode::BorrowNullable
            } else if is_option_type(arg_ty) {
                // by-value `Option<T>` opaque → nullable consume
                ParamMode::ConsumeNullable
            } else if borrow {
                ParamMode::Borrow
            } else {
                ParamMode::Consume
            }
        } else if is_value_proj {
            // `@JvmInline value class` (value_blob) param: pass the erased inner
            // field (`<name>.bytes`, or `<name>?.bytes` when Option) to the
            // extern, no lock. Supports the Direct (`T` / `&T`) and Nullable
            // (`Option<T>`) shapes; a collection layer (`Vec<value-blob>`,
            // i.e. an `Iterable` projection) still needs array codegen and is a
            // loud build-time error. The inline field is resolved off the
            // folded projection's `leaf_key`, so `Option<_>` wrappers around the
            // value blob resolve correctly.
            let proj = entry.metadata.projection.as_ref()?;
            if matches!(
                proj.strategy,
                crate::api::lang::jnigen::jni::FoldStrategy::Iterable(_)
            ) {
                panic!(
                    "render_wrapper_fn: value-blob `Vec<_>` params aren't \
                     supported yet (param `{name}`); add array codegen to lift this guard."
                );
            }
            let field =
                crate::api::lang::jnigen::jni::value_projection_field_for_leaf(ext, &proj.leaf_key)
                    .unwrap_or_else(|| {
                        panic!(
                            "render_wrapper_fn: cannot determine inline-class field for value \
                     projection param `{name}`"
                        )
                    });
            ParamMode::ValueUnwrap { field }
        } else {
            ParamMode::PassThrough
        };

        let ty = register_kt_type(&kt_type_raw, imports);
        let kt_type = if optional { ty.nullable() } else { ty };
        // Strip a leading `&` before the enum check — the `&Priority`
        // input converter shares Priority's converter (see the rank-1
        // `& _` arm), and the same `.value` projection applies either
        // way at the call site.
        // Detect enums through a leading `&` and through `Option<…>`, so a
        // nullable enum param passes `?.value` to the (Int?-typed) extern.
        let as_enum_value = ext.is_kotlin_enum(&enum_probe_type(arg_ty));
        params.push(Param {
            kt_name: name,
            kt_type,
            mode,
            as_enum_value,
        });
    }
    Some((params, receiver_idx))
}

/// Classify the output side into an [`OutputPlan`].
///
/// Output (data) expansion: the return value is delivered to a caller
/// callback per shape:
///   * `Decompose`/`Optional` (M1–M3): decompose into leaves → `build:
///     (L0, …) -> R` once; `<R>`, returns `R` / `R?`.
///   * `Iterable` (M4 whole / M5 decomposed): per element, fold
///     `(acc, leaves…) -> acc`; `<A>`, returns `A`, threads the accumulator.
///
/// Each leaf is delivered with its final Kotlin type; a **value_blob** leaf
/// (`@JvmInline value class`) can't be constructed Rust-side, so the wrapper
/// installs an **adapter** that applies the Kotlin-side projection wrap
/// (`ZZenohId(raw)`) before the user callback. Leaves with no value_blob ⇒
/// the callback is passed directly (M1–M4 unchanged).
fn classify_output(
    ext: &JniGen<impl JniGenState>,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
) -> Option<OutputPlan> {
    use crate::api::core::unfold::{Delivery, UnfoldShape};
    let unfold = registry.unfold_plans.get(&f.sig.ident);
    let is_convert = unfold.is_some_and(|p| p.delivery == Delivery::Return);
    // `builder_param` is the trailing **lambda** param (build / fold) as a
    // `(name, function-type)` pair. For the `Iterable` shape, the non-lambda
    // accumulator (`acc: A`) goes in `builder_lead` — it must precede
    // `onError` (a defaulted param) so the positional-`acc` call stays valid;
    // the trailing `fold` lambda follows.
    let mut builder_param: Option<(String, kt::KtType)> = None;
    let mut builder_lead: Option<(String, kt::KtType)> = None;
    let mut generic: Option<String> = None;
    // Extra call-site args injected before `__sink` (e.g. `build`/adapter, or
    // `acc` + the fold callback/adapter for `Iterable`).
    let mut unfold_call_args: Vec<String> = Vec::new();

    let (kt_return, projection) = if let Some(plan) =
        unfold.filter(|p| p.delivery == Delivery::Return)
    {
        // `convert_output` (Return): the wrapper returns the single converted
        // value directly — classify it exactly like a normal function whose
        // return type is `convert_out_ty`. No callback param, no generic, no
        // extra call args; the extern returns the real wire and `build_call`
        // applies the projection wrap (value_blob/handle) below.
        let cv = plan
            .convert_out_ty
            .clone()
            .expect("Return delivery carries convert_out_ty");
        let rt: syn::ReturnType = syn::parse_quote!(-> #cv);
        classify_return(ext, &rt, registry, imports)?
    } else if let Some(plan) = unfold.filter(|p| p.fixed_builder) {
        // Synthesized by-value `data_class` delivery via a **fixed, hoisted
        // singleton** — the wrapper takes no caller `build`/`fold` param and is
        // not generic over `R`/`A`. The native side still receives the singleton
        // as an erased `Any` and calls its cached `run`, so the whole delivery
        // machinery is reused; only the Rust-side object construction is gone.
        // The concrete return is the data class (`T` / `Option<T>`) or, for a
        // `Vec<data_class>` fold, a `List<Class>` composed on the Kotlin side.
        // The concrete element/return Kotlin type. For a decomposed `data_class`
        // builder/fold it is the data class (`plan.source`'s registered FQN); for
        // a **whole-element leaf** fold (`plan.element` set — String / value blob
        // / handle) `plan.source` (e.g. `String`) has no class FQN, so take the
        // element's typed view from the folder interface's element param instead.
        let class_ty = if plan.element.is_some() {
            let spec = folder_iface_for_plan(ext, registry, plan)?;
            register_kt_type(&spec.params[1].typed, imports)
        } else {
            let class_fqn = ext
                .kotlin_fqn(&TypeKey::from_type(&plan.source).to_string())
                .map(|s| s.to_string())?;
            register_kt_type(&kt::KtType::cls(class_fqn), imports)
        };
        if is_iterable_fold(&plan.shape) {
            // `Vec<data_class>` fold: allocate an `ArrayList<Class>` accumulator,
            // pass the hoisted **folder-appender** singleton as `fold` (it
            // rebuilds each element via `fromParts` and appends it), and return
            // the threaded accumulator as `List<Class>` (`?`-nullable for an
            // `Option<Vec<…>>` return — `None` yields a null list). Per element
            // only the raw leaves cross — no Java object is built on the Rust side.
            let spec = folder_iface_for_plan(ext, registry, plan)?;
            let holder = spec.singleton_holder_name();
            let field = crate::api::lang::jnigen::jni::SINGLETON_FIELD;
            imports.insert(spec.singleton_holder_fqn());
            unfold_call_args.push(format!("ArrayList<{class_ty}>()"));
            unfold_call_args.push(format!("{holder}.{field}"));
            let list_ty = kt::KtType::generic("List", [class_ty]);
            let kt = if matches!(plan.shape, UnfoldShape::Optional((), _)) {
                list_ty.nullable()
            } else {
                list_ty
            };
            (Some(kt), None)
        } else {
            // Scalar: the hoisted `__<Name>Builder` singleton calls `fromParts`;
            // the wrapper returns the concrete class (`?`-nullable for `Option`).
            let decon = plan
                .decon
                .as_ref()
                .expect("synthesized plan carries its DeconId");
            let spec = builder_iface_spec(ext, registry, decon)?;
            let singleton = format!("__{}", spec.raw_name());
            imports.insert(format!("{}.{singleton}", spec.package));
            unfold_call_args.push(singleton);
            let kt = match &plan.shape {
                UnfoldShape::Optional((), _) => class_ty.nullable(),
                _ => class_ty,
            };
            (Some(kt), None)
        }
    } else if let Some(plan) = unfold {
        // The builder / fold params are generated typed `fun interface`s
        // (`<Source>Builder<out R>` / `<Element>Folder<A>`); the native side
        // calls their typed `run` with raw jvalues (value-blob leaves surface
        // as `ByteArray` — no call-site adapter). Lambda-literal call sites
        // SAM-convert unchanged.
        let is_iterable = matches!(plan.shape, UnfoldShape::Iterable(_));
        if is_iterable {
            let spec = folder_iface_for_plan(ext, registry, plan)?;
            generic = Some("A".to_string());
            builder_lead = Some(("acc".to_string(), kt::KtType::var_("A")));
            builder_param = Some(("fold".to_string(), spec.kt_ref(vec![kt::KtType::var_("A")])));
            unfold_call_args.push("acc".to_string());
            if spec.needs_raw() {
                imports.insert(format!("{}.asRaw", spec.package));
                unfold_call_args.push("fold.asRaw()".to_string());
            } else {
                unfold_call_args.push("fold".to_string());
            }
            (Some(kt::KtType::var_("A")), None)
        } else {
            let decon = plan
                .decon
                .as_ref()
                .expect("record-built plan carries its DeconId");
            let spec = builder_iface_spec(ext, registry, decon)?;
            generic = Some("R".to_string());
            builder_param = Some(("build".to_string(), spec.kt_ref(vec![kt::KtType::var_r()])));
            if spec.needs_raw() {
                imports.insert(format!("{}.asRaw", spec.package));
                unfold_call_args.push("build.asRaw()".to_string());
            } else {
                unfold_call_args.push("build".to_string());
            }
            let kt = match &plan.shape {
                UnfoldShape::Optional((), _) => kt::KtType::var_r().nullable(),
                _ => kt::KtType::var_r(),
            };
            (Some(kt), None)
        }
    } else {
        classify_return(ext, &f.sig.output, registry, imports)?
    };
    // enum_class returns cross the JNI wire as jint → Kotlin `Int`.
    // Detect this so `build_call` can wrap the result with `fromInt`.
    let is_enum_return = unfold.is_none() && return_is_kotlin_enum(ext, &f.sig.output, registry);
    // `Option<enum>` returns cross as the boxed discriminant (`Int?`);
    // `build_call` maps back with `?.let { EnumType.fromInt(it) }`.
    let is_option_enum_return =
        unfold.is_none() && return_is_kotlin_option_enum(ext, &f.sig.output, registry);

    Some(OutputPlan {
        kt_return,
        projection,
        builder_param,
        builder_lead,
        generic,
        unfold_call_args,
        cast_return: unfold.is_some() && !is_convert,
        is_enum_return,
        is_option_enum_return,
    })
}

/// Build the JNINative call expression. Every param maps to exactly one call
/// arg (or several, for a flattened data_class); the output plan's extra args
/// and the trailing `__cap` follow; the result is wrapped per the return
/// classification (projection / enum / erased-`Any` cast).
fn build_native_call(
    ext: &JniGen<impl JniGenState>,
    jni_call: &str,
    params: &[Param],
    out: &OutputPlan,
) -> String {
    let mut args: Vec<String> = Vec::with_capacity(params.len());
    for p in params.iter() {
        // Flattened data_class param expands into multiple call args
        // (the leaf destructure expressions, in plan order).
        if let ParamMode::FlattenStruct { accesses } = &p.mode {
            args.extend(accesses.iter().cloned());
            continue;
        }
        // VecBuild param: the extern receives the `jlong` Vec handle the
        // wrapper body allocated and filled (`__vec_<name>`), not the `List`.
        if let ParamMode::VecBuild { .. } = &p.mode {
            args.push(format!("__vec_{}", p.kt_name));
            continue;
        }
        // OptionScalar param expands into two call args: the present flag
        // and the value-or-zero expression (in that order).
        if let ParamMode::OptionScalar {
            present_expr,
            value_expr,
        } = &p.mode
        {
            args.push(present_expr.clone());
            args.push(value_expr.clone());
            continue;
        }
        let arg = match &p.mode {
            ParamMode::Borrow
            | ParamMode::Consume
            | ParamMode::BorrowNullable
            | ParamMode::ConsumeNullable => format!("{}_ptr", p.kt_name),
            ParamMode::ValueUnwrap { field } => {
                // Inline value class → pass its erased inner field to the
                // extern (e.g. `z.bytes`: a `ByteArray`). A nullable value
                // class (`ZBytes?`) safe-navigates so it stays `ByteArray?`.
                if p.kt_type.is_nullable() {
                    format!("{}?.{}", p.kt_name, field)
                } else {
                    format!("{}.{}", p.kt_name, field)
                }
            }
            ParamMode::PassThrough => {
                if p.as_enum_value {
                    // Enum → its `Int` discriminant for the extern. Nullable
                    // enum (`Enum?`) uses `?.value` so it stays `Int?`.
                    if p.kt_type.is_nullable() {
                        format!("{}?.value", p.kt_name)
                    } else {
                        format!("{}.value", p.kt_name)
                    }
                } else {
                    p.kt_name.clone()
                }
            }
            // Callback lambda → the param itself (the extern takes the
            // erased `Any`), or its value-blob rebuilding adapter.
            ParamMode::Callback { call_arg } => call_arg.clone(),
            ParamMode::FlattenStruct { .. } => {
                unreachable!("FlattenStruct expanded before the single-arg match")
            }
            ParamMode::VecBuild { .. } => {
                unreachable!("VecBuild expanded before the single-arg match")
            }
            ParamMode::OptionScalar { .. } => {
                unreachable!("OptionScalar expanded before the single-arg match")
            }
        };
        args.push(arg);
    }
    // Output expansion: the builder / (acc, fold) cross just before the
    // error callback.
    args.extend(out.unfold_call_args.iter().cloned());
    // Every extern takes a trailing error callback. The wrapper passes a
    // **capture** (`__cap`) that records `(je, ze…)` and sets a flag — no
    // throw on the Rust upcall. The wrapper calls the user's `onError` after
    // the native call returns (see the body below).
    args.push("__cap".to_string());
    let mut call = format!(
        "{}.{jni_call}({})",
        ext.jni_native_class_name(),
        args.join(", ")
    );
    if let Some(p) = &out.projection {
        // Fold the wrap through the projection strategy. The wrap class is
        // the projection leaf's typed short name (Handle's typed-handle
        // class or value-class wrapper). The sentinel is the Kotlin
        // null-representation literal for the leaf wire — used only by
        // the `Niche+primitive` arm of `fold_projection_wrap`.
        let leaf_fqn = ext.kotlin_fqn(&p.leaf_key).unwrap_or(&p.leaf_key);
        let short = leaf_fqn.rsplit('.').next().unwrap_or(leaf_fqn).to_string();
        let sentinel = projection_leaf_sentinel(p);
        call = fold_projection_wrap(&p.strategy, &call, &short, sentinel.as_deref());
    } else if out.is_enum_return {
        let enum_kt = out
            .kt_return
            .as_ref()
            .expect("enum return has a Kotlin type");
        call = format!("{enum_kt}.fromInt({call})");
    } else if out.is_option_enum_return {
        // `kt_return` renders nullable (`Priority?`); the companion lives
        // on the non-null class name.
        let enum_kt = out
            .kt_return
            .as_ref()
            .expect("Option<enum> return has a Kotlin type")
            .to_string();
        let enum_kt = enum_kt.trim_end_matches('?');
        call = format!("{call}?.let {{ {enum_kt}.fromInt(it) }}");
    } else if out.cast_return {
        // Callback delivery: the extern returns the builder's erased `Any?`;
        // cast to the shape type (`R` / `R?` / `List<R>`). The builder always
        // produces `R`, so the unchecked cast is sound — suppressed below.
        let cast_kt = out
            .kt_return
            .as_ref()
            .expect("callback delivery returns R/A");
        call = format!("({call} as {cast_kt})");
    }
    // Return delivery (`convert_output`): the extern returns the real typed
    // wire; the projection wrap above (if any) already produced the value
    // class / handle — no cast needed.
    call
}

/// The opaque-handle params (Borrow/Consume modes) — the set the lock
/// scaffold, pre-lock guards, and consume `try/finally` operate on.
fn collect_opaques(params: &[Param]) -> Vec<Opaque> {
    params
        .iter()
        .filter_map(|p| {
            let (target, consume_null, nullable) = match p.mode {
                ParamMode::Borrow => (p.kt_name.clone(), None, false),
                ParamMode::Consume => (
                    p.kt_name.clone(),
                    Some(format!("{}.ptr = 0L", p.kt_name)),
                    false,
                ),
                ParamMode::BorrowNullable => (p.kt_name.clone(), None, true),
                // Nullable consume: null the slot only when present (null-safe).
                ParamMode::ConsumeNullable => (
                    p.kt_name.clone(),
                    Some(format!("{n}?.let {{ it.ptr = 0L }}", n = p.kt_name)),
                    true,
                ),
                _ => return None,
            };
            Some(Opaque {
                name: p.kt_name.clone(),
                target,
                consume_null,
                nullable,
            })
        })
        .collect()
}

/// Error callback `onError: <Err>Handler<R>` / `JniErrorHandler<R>` — a
/// generated typed fun interface `run(je: String?, ze…): R` whose ze params
/// are typed EXACTLY like a builder's leaves (the error channel is the
/// output channel with a fixed leading `je`). Contract: `je != null` ⇒
/// binding/system error, the native side fills the ze with defaults;
/// `je == null` ⇒ domain error, the ze carry the decomposed error. The
/// wrapper passes a SAM **capture** to the extern, then — after the native
/// call — calls `onError.run(je, ze…)` and returns its `R` if a failure
/// was recorded (no throw on the Rust upcall).
fn error_sink_parts(
    ext: &JniGen<impl JniGenState>,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
    r_ty: &kt::KtType,
) -> Option<ErrorSink> {
    let sink_spec = onerror_iface_spec(ext, registry, &f.sig.ident)?;
    let error_plan = registry.error_plans.get(&f.sig.ident);
    // Per ze leaf: (raw capture Kotlin type, raw default literal, raw→typed
    // wrap) — off the handler interface spec (its first param is the fixed
    // `je`), defaults from the matching error-plan leaf (same declaration,
    // same order). The CAPTURE is the raw twin (what the native side calls);
    // the user's handler is the TYPED interface — the redispatch wraps.
    let ze_info: Vec<(kt::KtType, String, crate::api::lang::jnigen::jni::WrapKind)> = sink_spec
        .params[1..]
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let leaf = error_plan
                .map(|pl| &pl.leaves[i])
                .expect("ze params exist only with an error plan");
            let default =
                ze_default_kotlin(&leaf_default(ext, registry, leaf), p.raw.is_nullable());
            if let Some(fqn) = p.wrap.class_fqn() {
                imports.insert(fqn.to_string());
            }
            (p.raw.clone(), default, p.wrap.clone())
        })
        .collect();
    let n_ze = ze_info.len();
    let onerr_type = sink_spec.kt_ref(vec![r_ty.clone()]);
    // The capture is a generated zero-alloc thread-local holder of the RAW
    // twin (no SAM lambda, no `Ref`-boxed captured vars) — its short name in
    // raw body text needs the import registered.
    imports.insert(sink_spec.capture_fqn());
    let capture_short = sink_spec.capture_name();
    // The je/ze argument list to call the user's typed `onError.run`. The
    // native side ALWAYS fills the raw ze (real values or defaults), so the
    // nullable capture slots are non-null whenever `__cap_failed` — assert
    // with `!!` for non-null params, then wrap raw → typed.
    let call_args = std::iter::once("__cap.je".to_string())
        .chain((0..n_ze).map(|i| {
            let (raw, _, wrap) = &ze_info[i];
            if raw.is_nullable() {
                wrap.wrap_expr(&format!("__cap.ze{i}"), true)
            } else {
                wrap.wrap_expr(&format!("__cap.ze{i}!!"), false)
            }
        }))
        .collect::<Vec<_>>()
        .join(", ");
    // Default-ze args for a synchronous (pre-call) closed-handle guard, which
    // calls the typed `onError.run` directly (a binding-class error ⇒ wrapped
    // raw defaults: `ZErr(0L)`, `ZId(ByteArray(0))`, …).
    let guard_args = std::iter::once("\"Operation on a closed native handle.\"".to_string())
        .chain(ze_info.iter().map(|(raw, def, wrap)| {
            if raw.is_nullable() {
                "null".to_string()
            } else {
                wrap.wrap_expr(def, false)
            }
        }))
        .collect::<Vec<_>>()
        .join(", ");
    Some(ErrorSink {
        onerr_type,
        capture_short,
        call_args,
        guard_args,
    })
}

/// Pre-lock closed-handle guards: a racy-but-safe `ptr == 0L` check before the
/// lock, returning `onError.run(...)` (function-level return; no throw).
fn render_prelock_guards(opaques: &[Opaque], guard_args: &str, is_unit: bool) -> String {
    let mut prelock_guards = String::new();
    for o in opaques {
        let cond = if o.nullable {
            format!("{n} != null && {t}.ptr == 0L", n = o.name, t = o.target)
        } else {
            format!("{t}.ptr == 0L", t = o.target)
        };
        if is_unit {
            prelock_guards.push_str(&format!(
                "if ({cond}) {{ onError.run({guard_args}); return }}\n"
            ));
        } else {
            prelock_guards.push_str(&format!("if ({cond}) return onError.run({guard_args})\n"));
        }
    }
    prelock_guards
}

/// `core_expr`: a single Kotlin **expression** evaluating to the call's
/// result. Handle params contribute pointer-binding statements and a
/// deadlock-safe `withSortedHandleLocks` acquisition; the whole thing is
/// expression-shaped (via `run { … }` where statements are needed) so the
/// caller can bind it to `__ret`, rethrow a captured sink error, then
/// return. A consume `try/finally` wraps the call when any handle is
/// consumed.
fn render_core_expr(
    ext: &JniGen<impl JniGenState>,
    opaques: &[Opaque],
    body_expr: &str,
    imports: &mut BTreeSet<String>,
) -> String {
    let consume_stmts: Vec<&str> = opaques
        .iter()
        .filter_map(|o| o.consume_null.as_deref())
        .collect();
    let value_expr = if consume_stmts.is_empty() {
        body_expr.to_string()
    } else {
        format!(
            "try {{\n{body_expr}\n}} finally {{\n{}\n}}",
            consume_stmts.join("\n")
        )
    };

    // Under-lock pointer reads. The closed-handle check is done pre-lock
    // (`prelock_guards`, → `onError`); these just bind the ptr the call passes.
    let mut ptr_binds = String::new();
    for o in opaques {
        if o.nullable {
            ptr_binds.push_str(&format!(
                "val {n}_ptr = {t}?.ptr ?: 0L\n",
                n = o.name,
                t = o.target
            ));
        } else {
            ptr_binds.push_str(&format!(
                "val {n}_ptr = {t}.ptr\n",
                n = o.name,
                t = o.target
            ));
        }
    }

    if opaques.is_empty() {
        // No handles — the call expression stands alone.
        value_expr
    } else if !ext.emit_handle_locks {
        // Lock-free mode: ptr binds then the value, wrapped as an expression.
        format!("run {{\n{ptr_binds}{value_expr}\n}}")
    } else {
        // Fast path: a statically-known, small (1–3), all-non-null handle set.
        // Pass the handles positionally to the allocation-free fixed-arity
        // `withSortedHandleLocks` overload. Otherwise build a `List` and use
        // the recursive overload.
        let fixed_arity = !opaques.iter().any(|o| o.nullable) && (1..=3).contains(&opaques.len());
        if !ext.package.is_empty() {
            imports.insert(format!("{}.withSortedHandleLocks", ext.package));
            if !fixed_arity {
                imports.insert(format!("{}.NativeHandle", ext.package));
            }
        }
        if fixed_arity {
            let targets = opaques
                .iter()
                .map(|o| o.target.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("withSortedHandleLocks({targets}) {{\n{ptr_binds}{value_expr}\n}}")
        } else {
            let mut adds = String::new();
            for o in opaques {
                if o.nullable {
                    adds.push_str(&format!("{n}?.let {{ __locks.add(it) }}\n", n = o.name));
                } else {
                    adds.push_str(&format!("__locks.add({t})\n", t = o.target));
                }
            }
            format!(
                "run {{\nval __locks = ArrayList<NativeHandle>()\n{adds}withSortedHandleLocks(__locks) {{\n{ptr_binds}{value_expr}\n}}\n}}"
            )
        }
    }
}

/// Assemble the wrapper's body text: pre-lock guards, the per-thread error
/// capture, the (possibly Vec-building) core call, the post-call failure
/// redispatch, and the return.
///
/// No throw from the binding: the wrapper installs a **capture** the extern
/// invokes on `Err` (a SAM literal of the same handler interface — the
/// native side calls its typed `run`), then — after the native call —
/// calls the user's `onError.run` and returns its `R` if a failure was
/// recorded. A pre-lock closed-handle guard short-circuits straight to
/// `onError.run` (it can't reach the capture).
/// Slice/Vec params built as Rust-side `Vec` handles: allocate the handle
/// before the lock, fill it by pushing each element's leaves, and free it in
/// a `finally` (always — the target wrapper either borrows the boxed `Vec` or
/// `mem::take`s it, leaving an empty `Vec` to drop). The transient handle is
/// not a `NativeHandle`, so it never joins the lock set.
fn render_body(
    ext: &JniGen<impl JniGenState>,
    params: &[Param],
    prelock_guards: &str,
    sink: &ErrorSink,
    core_expr: &str,
    is_unit: bool,
) -> String {
    let vec_build: Vec<(&String, &String, &Vec<String>)> = params
        .iter()
        .filter_map(|p| match &p.mode {
            ParamMode::VecBuild {
                base,
                elem_accesses,
            } => Some((&p.kt_name, base, elem_accesses)),
            _ => None,
        })
        .collect();
    let mut b = String::new();
    b.push_str(prelock_guards);
    // The capture is a per-thread reusable holder (zero allocation): the
    // extern writes its `@JvmField` slots via `run`, the wrapper reads
    // them after the (synchronous) call. `acquire()` resets the slots.
    b.push_str(&format!("val __cap = {}.acquire()\n", sink.capture_short));
    let failed_check = format!("if (__cap.failed) return onError.run({})\n", sink.call_args);
    if vec_build.is_empty() {
        if is_unit {
            b.push_str(&format!("{core_expr}\n"));
            b.push_str(&failed_check);
        } else {
            b.push_str(&format!("val __ret = {core_expr}\n"));
            b.push_str(&failed_check);
            b.push_str("return __ret\n");
        }
    } else {
        let native = ext.jni_native_class_name();
        for (name, base, _) in &vec_build {
            let new_m = crate::api::lang::jnigen::jni::vec_helper_method_name(ext, base, "New");
            b.push_str(&format!(
                "val __vec_{name} = {native}.{new_m}({name}.size)\n"
            ));
        }
        // `try { fill…; <core_expr> } finally { free… }`: Kotlin `try` is an
        // expression, so for a non-unit fn `__ret` binds to `core_expr`
        // (the block's last expression). A push runs no JVM upcall, so the
        // loop needs no per-element failure check.
        b.push_str(if is_unit {
            "try {\n"
        } else {
            "val __ret = try {\n"
        });
        for (name, base, accesses) in &vec_build {
            let push_m = crate::api::lang::jnigen::jni::vec_helper_method_name(ext, base, "Push");
            let args = std::iter::once(format!("__vec_{name}"))
                .chain(accesses.iter().cloned())
                .collect::<Vec<_>>()
                .join(", ");
            b.push_str(&format!(
                "for (__e in {name}) {{\n{native}.{push_m}({args})\n}}\n"
            ));
        }
        b.push_str(&format!("{core_expr}\n"));
        b.push_str("} finally {\n");
        for (name, base, _) in &vec_build {
            let free_m = crate::api::lang::jnigen::jni::vec_helper_method_name(ext, base, "Free");
            b.push_str(&format!("{native}.{free_m}(__vec_{name})\n"));
        }
        b.push_str("}\n");
        b.push_str(&failed_check);
        if !is_unit {
            b.push_str("return __ret\n");
        }
    }
    b
}

/// The Kotlin typing of one delivered lambda leaf: `(builder_kt, wire_kt,
/// wrap, is_value_blob)` — the type the *user's* lambda sees, the type the
/// extern delivers, and the expression rebuilding the former from the latter
/// (`pk` is the adapter's parameter name; passthrough unless the leaf is a
/// `value_blob`, whose `@JvmInline value class` can't be built Rust-side).
/// Shared by the unfold builder/fold lambda and the callback lambda params.
pub(crate) fn unfold_leaf_kt(
    ext: &JniGen<impl JniGenState>,
    registry: &Registry<KotlinMeta>,
    out_ty: &syn::Type,
    nullable: bool,
    pk: &str,
    imports: &mut BTreeSet<String>,
) -> Option<(kt::KtType, String, String, bool)> {
    let proj = registry
        .output_entry(out_ty)
        .and_then(|e| e.metadata.projection.clone());
    let is_vb = proj
        .as_ref()
        .map(|p| p.kind == crate::api::lang::jnigen::jni::ProjectionKind::ValueBlob)
        .unwrap_or(false);
    // builder_kt: enum → Int; otherwise the normal classified type
    // (handle class / value class / String / ByteArray / Long …).
    let builder_kt = if ext.is_kotlin_enum(&enum_probe_type(out_ty)) {
        kt::KtType::int()
    } else {
        let rt: syn::ReturnType = syn::parse_quote!(-> #out_ty);
        classify_return(ext, &rt, registry, imports)?.0?
    };
    let (mut wire_kt, wrap) = if is_vb {
        let p = proj.as_ref().unwrap();
        // Wrap class = the projection leaf's typed short name — NOT
        // `builder_kt` (which is `Short?` for an `Option<…>` leaf and would
        // leak the `?` into the constructor call).
        let leaf_fqn = ext.kotlin_fqn(&p.leaf_key).unwrap_or(&p.leaf_key);
        let short = leaf_fqn.rsplit('.').next().unwrap_or(leaf_fqn).to_string();
        let sentinel = projection_leaf_sentinel(p);
        let mut wrap = fold_projection_wrap(&p.strategy, pk, &short, sentinel.as_deref());
        // A `nullable` leaf (an `Option` nesting step on its path) makes the
        // wire nullable even when the strategy itself is `Direct` — guard the
        // wrap so a null wire stays null instead of feeding the constructor.
        if nullable
            && matches!(
                p.strategy,
                crate::api::lang::jnigen::jni::FoldStrategy::Base
            )
        {
            wrap = format!("{pk}?.let {{ {short}(it) }}");
        }
        (projection_wire_return(p), wrap)
    } else {
        (builder_kt.to_string(), pk.to_string())
    };
    let builder_kt = if nullable {
        wire_kt.push('?');
        builder_kt.nullable()
    } else {
        builder_kt
    };
    Some((builder_kt, wire_kt, wrap, is_vb))
}

/// Kotlin parameter names for a plan's delivered leaves, in leaf order. The
/// names are the author-supplied [`UnfoldLeaf::name`]s (`handle` for a root
/// identity), emitted **verbatim** — no casing/keyword escaping (the author
/// writes valid Kotlin identifiers) and no dedup (uniqueness is enforced in
/// `core::unfold`).
///
/// [`UnfoldLeaf::name`]: crate::api::core::unfold::UnfoldLeaf::name
pub(crate) fn plan_leaf_names(leaves: &[crate::api::core::unfold::UnfoldLeaf]) -> Vec<String> {
    leaves.iter().map(|leaf| leaf.name.clone()).collect()
}

/// Lambda parameter name for a whole-value (plan-less) callback arg: the
/// decapitalized bare type short (`ZQuery` → `zQuery`), peeling a `&` /
/// `Option<…>` layer; `arg{i}` for non-path shapes.
pub(crate) fn whole_value_name(ty: &syn::Type, i: usize) -> String {
    let mut t = ty.clone();
    if let syn::Type::Reference(r) = &t {
        t = (*r.elem).clone();
    }
    if let syn::Type::Path(tp) = &t {
        if let Some(last) = tp.path.segments.last() {
            if last.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ab) = &last.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = ab.args.first() {
                        t = inner.clone();
                    }
                }
            }
        }
    }
    if let syn::Type::Path(tp) = &t {
        if let Some(last) = tp.path.segments.last() {
            let s = last.ident.to_string();
            let mut cs = s.chars();
            if let Some(f) = cs.next() {
                return kt_param_name(&format!("{}{}", f.to_lowercase(), cs.as_str()));
            }
        }
    }
    format!("arg{i}")
}

/// The Kotlin default literal for an error `ze` leaf, used when the
/// **wrapper itself** raises a binding-class error before the native call
/// (the pre-lock closed-handle guard) and must call `onError.run` with
/// builder-typed ze params. Rendered from the shared [`leaf_default`]
/// classification — the same one the native `__ze_defaults` jvalues use —
/// so the two sides cannot drift. The [`LeafDefault::Null`] arm renders
/// `null` (valid for plan-nullable params; an unknown non-null object kind
/// gets a `null!!` assertion — no constructible default exists for it).
fn ze_default_kotlin(d: &LeafDefault, kt_nullable: bool) -> String {
    match d {
        LeafDefault::Null if kt_nullable => "null".to_string(),
        LeafDefault::Null => "null!!".to_string(),
        LeafDefault::Prim(p) => p.kotlin_zero().to_string(),
        LeafDefault::Str => "\"\"".to_string(),
        LeafDefault::Bytes => "ByteArray(0)".to_string(),
        LeafDefault::List => "emptyList()".to_string(),
    }
}

/// Fall-back Kotlin type derived directly from the JNI wire type.
/// Returns the **non-nullable** Kotlin base name — the use site adds
/// a `?` suffix when the entry's Rust type is `Option<…>` (via
/// [`is_option_type`]), so this helper must not double up.
pub(crate) fn kotlin_for_wire(wire: &syn::Type) -> Option<kt::KtType> {
    if let Some(p) = JniPrim::from_wire(wire) {
        return Some(kt::KtType::cls(p.kotlin_type()));
    }
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            let kt = match last.ident.to_string().as_str() {
                "JString" | "jstring" => "String",
                "JByteArray" | "jbyteArray" => "ByteArray",
                "JObject" | "jobject" | "JClass" => "Any",
                _ => return None,
            };
            return Some(kt::KtType::cls(kt));
        }
    }
    None
}

/// Returns `(kt_return, projection)` where:
/// * `kt_return` is the declared Kotlin return type written in the
///   wrapper's signature (empty for `Unit`).
/// * `projection` is `Some(Projection)` when the return is a Kotlin newtype
///   (opaque handle or value class) reached through 0+ wrappers. The
///   wrapper body uses it to fold the wrap call (`W(x)` for `Direct`,
///   `?.let { W(it) }` for `Nullable`, `.map { W(it) }` for `Iterable`)
///   and pick the JNI extern's wire return (`Long` for `Handle`,
///   the inner wire's Kotlin name for `ValueClass`). `None` for plain
///   non-projection returns.
pub(crate) fn classify_return(
    ext: &JniGen<impl JniGenState>,
    output: &syn::ReturnType,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
) -> Option<(
    Option<kt::KtType>,
    Option<crate::api::lang::jnigen::jni::Projection>,
)> {
    let ty = match output {
        syn::ReturnType::Default => return Some((None, None)),
        syn::ReturnType::Type(_, t) => &**t,
    };
    let outer_meta = registry.output_entry(ty).map(|e| e.metadata.clone());
    // Unit returns (incl. `ZResult<()>`, whose inner identity rides
    // `value_rust_key`) declare no Kotlin return type.
    let inner_canon = outer_meta
        .as_ref()
        .and_then(|m| m.value_rust_key.clone())
        .unwrap_or_else(|| ty.to_token_stream().to_string());
    let inner: syn::Type = syn::parse_str(&inner_canon).unwrap_or_else(|_| ty.clone());
    if crate::api::lang::jnigen::util::is_unit(&inner) {
        return Some((None, None));
    }
    // Projection return (opaque handle or value class): read the folded
    // `Projection` the type-unfolding mechanism propagated onto this return
    // type's converter metadata — one source of truth, no shape-specific
    // peeling. The declared return type is the concrete projection class
    // folded through `Nullable`/`Iterable`; callers fold the wrap and pick
    // the wire return based on `kind`.
    if let Some(h) = outer_meta.as_ref().and_then(|m| m.projection.clone()) {
        let fqn = ext
            .kotlin_fqn(&h.leaf_key)
            .map(|v| v.to_string())
            .unwrap_or_else(|| {
                panic!(
                    "classify_return: projection return type `{}` has no Kotlin FQN registered \
                     — every opaque/value class must be declared via `JniGen::ptr_class(...)` \
                     / `JniGen::value_class(...)`.",
                    h.leaf_key
                )
            });
        let short = register_fqn(&fqn, imports);
        return Some((
            Some(handle_kt_type(&h.strategy, &kt::KtType::cls(short))),
            Some(h),
        ));
    }
    // Non-opaque: read the Kotlin type straight off the resolved
    // output entry's metadata — the rank-N handler propagates
    // `ZResult<T>` / `Option<T>` / `Vec<T>` derivations alongside the
    // wire, so no peel-and-fallback chain is needed at the use site.
    if let Some(out_entry) = registry.output_entry(ty) {
        if let Some(kt) = out_entry.metadata.kotlin_name.clone() {
            return Some((Some(register_kt_type(&kt, imports)), None));
        }
    }
    None
}

/// Returns `true` when the function's return type resolves to a type registered
/// via [`JniGen::enum_class`]. Enum returns cross the JNI wire as `jint` (Kotlin
/// `Int`); the public wrapper must call `EnumType.fromInt(Int)` to convert back.
pub(crate) fn return_is_kotlin_enum(
    ext: &JniGen<impl JniGenState>,
    output: &syn::ReturnType,
    registry: &Registry<KotlinMeta>,
) -> bool {
    ext.is_kotlin_enum(&canonical_return_ty(output, registry))
}

/// Returns `true` when the function's return type resolves to `Option<E>` with
/// `E` a [`JniGen::enum_class`] enum. The native side delivers the discriminant
/// `box_jint`-boxed (null for `None`), so the extern returns `Int?` and the
/// public wrapper converts back with `?.let { EnumType.fromInt(it) }`.
pub(crate) fn return_is_kotlin_option_enum(
    ext: &JniGen<impl JniGenState>,
    output: &syn::ReturnType,
    registry: &Registry<KotlinMeta>,
) -> bool {
    crate::api::core::types_util::option_inner_type(&canonical_return_ty(output, registry))
        .map(|inner| ext.is_kotlin_enum(&inner))
        .unwrap_or(false)
}

/// The return type with the error channel peeled: the resolved output entry's
/// canonical value key (`Result<T, E>` → `T`) when present, else the declared
/// type verbatim.
fn canonical_return_ty(output: &syn::ReturnType, registry: &Registry<KotlinMeta>) -> syn::Type {
    let ty = match output {
        syn::ReturnType::Default => return syn::parse_quote!(()),
        syn::ReturnType::Type(_, t) => &**t,
    };
    registry
        .output_entry(ty)
        .and_then(|e| e.metadata.value_rust_key.clone())
        .and_then(|canon| syn::parse_str(&canon).ok())
        .unwrap_or_else(|| ty.clone())
}

pub(crate) fn kt_snake_to_camel(s: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in s.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Camel-case a Rust param ident into a valid Kotlin parameter name. Kotlin
/// **hard keywords** can't be used as identifiers (not even back-ticked), so a
/// collision is escaped by appending `_`. Param names don't affect JNI linkage
/// (only the function name + JVM signature do), so renaming is always safe.
pub(crate) fn kt_param_name(rust_ident: &str) -> String {
    let camel = kt_snake_to_camel(rust_ident);
    const HARD_KEYWORDS: &[&str] = &[
        "as",
        "break",
        "class",
        "continue",
        "do",
        "else",
        "false",
        "for",
        "fun",
        "if",
        "in",
        "interface",
        "is",
        "null",
        "object",
        "package",
        "return",
        "super",
        "this",
        "throw",
        "true",
        "try",
        "typealias",
        "typeof",
        "val",
        "var",
        "when",
        "while",
    ];
    if HARD_KEYWORDS.contains(&camel.as_str()) {
        format!("{camel}_")
    } else {
        camel
    }
}
