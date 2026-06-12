//! Typed callback `fun interface` specs — the single source of truth shared
//! by the three emission sites:
//!
//!   * `kotlin_emit` — emits the `fun interface` declaration per package;
//!   * `render` — types the wrapper's callback/builder/fold/onError params
//!     as references to these interfaces;
//!   * `emit` / `trait_impl` — builds the native upcall (`run` + the JVM
//!     descriptor recorded here).
//!
//! Every callback position (impl-`Fn` delivery, output-expansion `build`,
//! `fold`, `onError`) gets a generated interface whose single method is
//! `public fun run(...)` with **JVM-stable parameter types** — typed handle
//! classes, `ByteArray` for `value_blob` (never the `@JvmInline` class —
//! Kotlin would mangle the method name and `GetMethodID` would fail),
//! primitives unboxed, nullable primitives boxed. The native side calls
//! `run` with raw typed `jvalue`s: no per-leaf boxing upcalls, no erased
//! `FunctionN`.
//!
//! All constructors are deterministic over `(ext, registry)`, so the three
//! sites independently derive identical specs.

use super::*;
use crate::api::core::unfold::{DeconId, UnfoldPlan, UnfoldShape};

/// The JVM-visible single method name of every generated callback interface.
pub(crate) const IFACE_METHOD: &str = "run";

/// One generated `fun interface`: identity, Kotlin surface, and the JVM
/// descriptor of its `run` method.
#[derive(Clone, Debug)]
pub(crate) struct IfaceSpec {
    /// Kotlin package the interface is declared in.
    pub package: String,
    /// Interface short name (`ZSampleCallback`, `ZKeyExprBuilder`, …).
    pub name: String,
    /// Type parameters with variance as written (`["out R"]`, `["A"]`).
    pub type_params: Vec<String>,
    /// `run` parameters: `(name, Kotlin type)`. Generic positions use the
    /// bare type-variable name (`A`).
    pub params: Vec<(String, kt::KtType)>,
    /// `run` return type (`Unit`, or a bare type variable `R`/`A`).
    pub ret: kt::KtType,
    /// Full JVM descriptor of `run`, e.g. `"(Ljava/lang/String;[BI)V"`.
    /// Generic positions erase to `Ljava/lang/Object;`.
    pub descr: String,
}

impl IfaceSpec {
    pub fn fqn(&self) -> String {
        if self.package.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.package, self.name)
        }
    }

    /// Slash form for `FindClass`.
    pub fn slash_fqn(&self) -> String {
        self.fqn().replace('.', "/")
    }

    /// A [`kt::KtType`] reference to this interface, instantiated with
    /// `args` (empty for a non-generic interface).
    pub fn kt_ref(&self, args: Vec<kt::KtType>) -> kt::KtType {
        if args.is_empty() {
            kt::KtType::cls(self.fqn())
        } else {
            kt::KtType::generic(self.fqn(), args)
        }
    }

    /// The Kotlin declaration.
    pub fn to_decl(&self) -> kt::KtFunInterface {
        let mut m = kt::KtFun::new(IFACE_METHOD).vis(kt::Vis::Public);
        for (n, t) in &self.params {
            m = m.param(kt::KtParam::new(n, t.clone()));
        }
        m = m.returns(self.ret.clone());
        let mut i = kt::KtFunInterface::new(&self.name, m).vis(kt::Vis::Public);
        for tp in &self.type_params {
            i = i.type_param(tp);
        }
        i
    }
}

/// The JVM descriptor chunk for a parameter/return Kotlin type.
/// `type_params` are the interface's bare type-variable names (variance
/// stripped) — they erase to `Object`. `Unit` maps to `V` (valid only in
/// return position; parameters never carry `Unit`).
///
/// Loud panic on anything unrecognized: a silently-wrong descriptor would
/// surface as a runtime `GetMethodID` failure (or worse, a mistyped jvalue).
pub(crate) fn kt_jvm_descriptor(ty: &kt::KtType, type_params: &[String]) -> String {
    let kt::KtType::Named {
        fqn,
        args,
        nullable,
    } = ty
    else {
        panic!("kt_jvm_descriptor: function types cannot appear in a typed callback interface");
    };
    let simple = fqn.rsplit('.').next().unwrap_or(fqn);
    // Generic type variable → Object.
    if type_params
        .iter()
        .map(|p| p.strip_prefix("out ").unwrap_or(p).trim())
        .any(|p| p == fqn)
    {
        return "Ljava/lang/Object;".to_string();
    }
    if !fqn.contains('.') {
        // Kotlin builtins (the only dot-free names a leaf type may use).
        let prim = match simple {
            "Int" => Some(("I", "Ljava/lang/Integer;")),
            "Long" => Some(("J", "Ljava/lang/Long;")),
            "Boolean" => Some(("Z", "Ljava/lang/Boolean;")),
            "Byte" => Some(("B", "Ljava/lang/Byte;")),
            "Short" => Some(("S", "Ljava/lang/Short;")),
            "Char" => Some(("C", "Ljava/lang/Character;")),
            "Float" => Some(("F", "Ljava/lang/Float;")),
            "Double" => Some(("D", "Ljava/lang/Double;")),
            _ => None,
        };
        if let Some((p, boxed)) = prim {
            return if *nullable { boxed.to_string() } else { p.to_string() };
        }
        return match simple {
            "Unit" => "V".to_string(),
            "String" => "Ljava/lang/String;".to_string(),
            "ByteArray" => "[B".to_string(),
            "List" | "MutableList" => "Ljava/util/List;".to_string(),
            "Any" => "Ljava/lang/Object;".to_string(),
            // A dot-free non-builtin: a generated class with no package
            // prefix configured (default-package; mainly test fixtures).
            other => format!("L{other};"),
        };
    }
    let _ = args;
    // A class FQN (typed handle, generated class).
    format!("L{};", fqn.replace('.', "/"))
}

fn method_descr(params: &[(String, kt::KtType)], ret: &kt::KtType, type_params: &[String]) -> String {
    let mut d = String::from("(");
    for (_, t) in params {
        d.push_str(&kt_jvm_descriptor(t, type_params));
    }
    d.push(')');
    d.push_str(&kt_jvm_descriptor(ret, type_params));
    d
}

/// The interface base name for a decomposition: the subject type's short
/// name, extended by the deconstructor declaration's identity. The type's
/// canonical (unnamed) declaration keeps the bare short; a named alternative
/// appends its UpperCamel name (`ZError` + `"full"` → `ZErrorFull`); per-fn
/// inline records (`.fun_output`) append the function's UpperCamel ident.
/// This is what makes interface identity == declaration identity: functions
/// sharing a declaration share the interface, differently-declared
/// decompositions of one type get distinct interfaces.
fn decon_base_name(short: &str, decon: Option<&DeconId>) -> String {
    let upper_camel = |s: &str| -> String {
        let camel = snake_to_camel(s);
        let mut c = camel.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => camel,
        }
    };
    match decon {
        None | Some(DeconId::Canonical(_)) => short.to_string(),
        Some(DeconId::Named(_, n)) => format!("{short}{}", upper_camel(n)),
        Some(DeconId::PerFn(_, f)) => format!("{short}{}", upper_camel(f)),
    }
}

/// Short name of a Rust type key (`zenoh_flat::ZSample` → `ZSample`),
/// peeled of `&` / `Option`.
fn subject_short(ty: &syn::Type) -> String {
    let peeled = peel_ref_option(ty);
    if let syn::Type::Path(tp) = &peeled {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident.to_string();
        }
    }
    TypeKey::from_type(&peeled).to_string().replace([' ', ':', '<', '>'], "")
}

fn peel_ref_option(ty: &syn::Type) -> syn::Type {
    let mut t = ty.clone();
    loop {
        match t {
            syn::Type::Reference(r) => t = (*r.elem).clone(),
            syn::Type::Path(ref tp)
                if tp.path.segments.last().is_some_and(|s| s.ident == "Option"
                    || s.ident == "Vec") =>
            {
                let seg = tp.path.segments.last().unwrap();
                if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = ab.args.first() {
                        t = inner.clone();
                        continue;
                    }
                }
                return t;
            }
            other => return other,
        }
    }
}

/// Package a subject type's interface lives in: the package of the type's
/// registered Kotlin FQN, the root `ext.package` otherwise.
fn subject_package(ext: &JniGen, subject: &syn::Type) -> String {
    let key = TypeKey::from_type(&peel_ref_option(subject)).to_string();
    ext.kotlin_fqn(&key)
        .and_then(|fqn| fqn.rsplit_once('.').map(|(p, _)| p.to_string()))
        .unwrap_or_else(|| ext.package.clone())
}

/// The interface param list for a plan's leaves: names from
/// [`plan_leaf_names`], types from [`unfold_leaf_kt`] — with `value_blob`
/// leaves degraded to their `ByteArray` wire (the `@JvmInline` class cannot
/// appear in a JNI-called method signature).
fn plan_leaf_params(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    plan: &UnfoldPlan,
) -> Option<Vec<(String, kt::KtType)>> {
    let mut names = plan_leaf_names(registry, plan);
    dedup_kt_param_names(&mut names);
    let mut throwaway = BTreeSet::new();
    let mut out = Vec::with_capacity(plan.leaves.len());
    for (name, leaf) in names.into_iter().zip(plan.leaves.iter()) {
        let ty = leaf_iface_kt(ext, registry, &leaf.out_ty, leaf.nullable, &mut throwaway)?;
        out.push((name, ty));
    }
    Some(out)
}

/// The interface-tier Kotlin type of one delivered leaf: the user-visible
/// classified type, except `value_blob` leaves which surface as their raw
/// `ByteArray` wire, and handle leaves which carry the typed-handle **FQN**
/// (the classified type may be the bare short name; the JVM descriptor
/// derived from this type must name the class fully).
pub(crate) fn leaf_iface_kt(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    out_ty: &syn::Type,
    nullable: bool,
    throwaway: &mut BTreeSet<String>,
) -> Option<kt::KtType> {
    let (builder_kt, wire_kt, _wrap, is_vb) =
        unfold_leaf_kt(ext, registry, out_ty, nullable, "x", throwaway)?;
    if is_vb {
        let t = kt::KtType::byte_array();
        return Some(if wire_kt.ends_with('?') {
            t.nullable()
        } else {
            t
        });
    }
    // Handle leaf: re-key the class onto its registered FQN, keeping the
    // folded nullability.
    if let Some(proj) = registry
        .output_entry(out_ty)
        .and_then(|e| e.metadata.projection.as_ref())
        .filter(|p| p.kind == ProjectionKind::Handle)
    {
        if let Some(fqn) = ext.kotlin_fqn(&proj.leaf_key) {
            let t = kt::KtType::cls(fqn.to_string());
            return Some(if builder_kt.is_nullable() {
                t.nullable()
            } else {
                t
            });
        }
    }
    Some(builder_kt)
}

/// Interface for an `impl Fn(args)` delivery: one `run` parameter per
/// flattened leaf of each arg's callback plan (the arg whole when plan-less),
/// returning `Unit`. Named `<ArgShorts>Callback` (`Fn()` → `VoidCallback`),
/// placed in the first arg type's package (root for `Fn()`).
pub(crate) fn callback_iface_spec(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    cb_args: &[syn::Type],
) -> Option<IfaceSpec> {
    let mut leaf_tys: Vec<(String, syn::Type, bool)> = Vec::new();
    for (i, t) in cb_args.iter().enumerate() {
        if let Some(plan) = registry.callback_arg_plans.get(&TypeKey::from_type(t)) {
            leaf_tys.extend(
                plan_leaf_names(registry, plan)
                    .into_iter()
                    .zip(plan.leaves.iter())
                    .map(|(n, l)| (n, l.out_ty.clone(), l.nullable)),
            );
        } else {
            leaf_tys.push((whole_value_name(t, i), t.clone(), is_option_type(t)));
        }
    }
    let mut names: Vec<String> = leaf_tys.iter().map(|(n, _, _)| n.clone()).collect();
    dedup_kt_param_names(&mut names);
    let mut throwaway = BTreeSet::new();
    let mut params = Vec::with_capacity(leaf_tys.len());
    for (k, (_, out_ty, nullable)) in leaf_tys.iter().enumerate() {
        let ty = leaf_iface_kt(ext, registry, out_ty, *nullable, &mut throwaway)?;
        params.push((names[k].clone(), ty));
    }
    let name = if cb_args.is_empty() {
        "VoidCallback".to_string()
    } else {
        format!(
            "{}Callback",
            cb_args
                .iter()
                .map(|t| subject_short(t))
                .collect::<Vec<_>>()
                .join("")
        )
    };
    let package = cb_args
        .first()
        .map(|t| subject_package(ext, t))
        .unwrap_or_else(|| ext.package.clone());
    let ret = kt::KtType::unit();
    let descr = method_descr(&params, &ret, &[]);
    Some(IfaceSpec {
        package,
        name,
        type_params: vec![],
        params,
        ret,
        descr,
    })
}

/// Interface for an output-expansion **builder** (`Decompose`/`Optional`
/// callback delivery): `run(leaves…): R`, `<out R>`. Keyed by the
/// deconstructor declaration — the signature derives from the declaration's
/// representative plan in `registry.decon_plans`, never from a using
/// function's own plan. Named `<decl-base>Builder`, placed in the source
/// type's package.
pub(crate) fn builder_iface_spec(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    decon: &DeconId,
) -> Option<IfaceSpec> {
    let plan = registry.decon_plans.get(decon)?;
    let params = plan_leaf_params(ext, registry, plan)?;
    let name = format!(
        "{}Builder",
        decon_base_name(&subject_short(&plan.source), Some(decon))
    );
    let package = subject_package(ext, &plan.source);
    let type_params = vec!["out R".to_string()];
    let ret = kt::KtType::var_r();
    let descr = method_descr(&params, &ret, &type_params);
    Some(IfaceSpec {
        package,
        name,
        type_params,
        params,
        ret,
        descr,
    })
}

/// Interface for a **decomposed-element fold** (`Iterable` delivery over a
/// type with a deconstructor): `run(acc: A, element-leaves…): A`, `<A>`
/// (invariant — `A` appears in both parameter and return position). Keyed by
/// the element's deconstructor declaration. Named `<decl-base>Folder`,
/// placed in the element type's package.
pub(crate) fn folder_iface_spec(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    decon: &DeconId,
) -> Option<IfaceSpec> {
    let plan = registry.decon_plans.get(decon)?;
    let mut params: Vec<(String, kt::KtType)> = vec![("acc".to_string(), kt::KtType::var_("A"))];
    params.extend(plan_leaf_params(ext, registry, plan)?);
    let name = format!(
        "{}Folder",
        decon_base_name(&subject_short(&plan.source), Some(decon))
    );
    let package = subject_package(ext, &plan.source);
    let type_params = vec!["A".to_string()];
    let ret = kt::KtType::var_("A");
    let descr = method_descr(&params, &ret, &type_params);
    Some(IfaceSpec {
        package,
        name,
        type_params,
        params,
        ret,
        descr,
    })
}

/// Interface for a **whole-element fold** (`Iterable` delivery of a type
/// without a deconstructor — no declaration involved):
/// `run(acc: A, element): A`. One shape per element type by construction.
pub(crate) fn whole_folder_iface_spec(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    element: &syn::Type,
) -> Option<IfaceSpec> {
    let mut throwaway = BTreeSet::new();
    let mut params: Vec<(String, kt::KtType)> = vec![("acc".to_string(), kt::KtType::var_("A"))];
    params.push((
        "element".to_string(),
        leaf_iface_kt(ext, registry, element, false, &mut throwaway)?,
    ));
    let name = format!("{}Folder", subject_short(element));
    let package = subject_package(ext, element);
    let type_params = vec!["A".to_string()];
    let ret = kt::KtType::var_("A");
    let descr = method_descr(&params, &ret, &type_params);
    Some(IfaceSpec {
        package,
        name,
        type_params,
        params,
        ret,
        descr,
    })
}

/// The folder spec for an `Iterable` plan: declaration-keyed when the
/// element decomposes, whole-element otherwise. Thin dispatch — the
/// derivation itself is keyed.
pub(crate) fn folder_iface_for_plan(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    plan: &UnfoldPlan,
) -> Option<IfaceSpec> {
    debug_assert!(matches!(plan.shape, UnfoldShape::Iterable(_)));
    match (&plan.element, &plan.decon) {
        (Some(el), _) => whole_folder_iface_spec(ext, registry, el),
        (None, Some(d)) => folder_iface_spec(ext, registry, d),
        (None, None) => None,
    }
}

/// Interface for a fallible function's **onError** handler: `run(je: String?,
/// ze-leaves…): R`, `<out R>`. The `ze` leaves are NULLABLE — a binding
/// error (`je != null`) delivers null `ze`s; consumers coalesce. Keyed by
/// the error type's deconstructor declaration. Named `<decl-base>Handler`,
/// placed in the error type's package.
pub(crate) fn error_handler_iface_spec(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    decon: &DeconId,
) -> Option<IfaceSpec> {
    let plan = registry.decon_plans.get(decon)?;
    let mut params: Vec<(String, kt::KtType)> =
        vec![("je".to_string(), kt::KtType::string().nullable())];
    for (name, ty) in plan_leaf_params(ext, registry, plan)? {
        params.push((name, ty.nullable()));
    }
    let name = format!(
        "{}Handler",
        decon_base_name(&subject_short(&plan.source), Some(decon))
    );
    let package = subject_package(ext, &plan.source);
    let type_params = vec!["out R".to_string()];
    let ret = kt::KtType::var_r();
    let descr = method_descr(&params, &ret, &type_params);
    Some(IfaceSpec {
        package,
        name,
        type_params,
        params,
        ret,
        descr,
    })
}

/// The shared infallible handler `JniErrorHandler<out R> { run(je: String?): R }`
/// — every function without an error plan takes one; placed in the root
/// package.
pub(crate) fn jni_error_handler_iface_spec(ext: &JniGen) -> IfaceSpec {
    let params = vec![("je".to_string(), kt::KtType::string().nullable())];
    let type_params = vec!["out R".to_string()];
    let ret = kt::KtType::var_r();
    let descr = method_descr(&params, &ret, &type_params);
    IfaceSpec {
        package: ext.package.clone(),
        name: "JniErrorHandler".to_string(),
        type_params,
        params,
        ret,
        descr,
    }
}

/// The onError handler spec for a declared function: its error plan's
/// declaration-keyed typed handler, or the shared
/// [`jni_error_handler_iface_spec`].
pub(crate) fn onerror_iface_spec(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    fn_ident: &syn::Ident,
) -> Option<IfaceSpec> {
    match registry.error_plans.get(fn_ident) {
        Some(plan) => error_handler_iface_spec(
            ext,
            registry,
            plan.decon
                .as_ref()
                .expect("error plans are always record-built (decon is Some)"),
        ),
        None => Some(jni_error_handler_iface_spec(ext)),
    }
}
