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
use crate::api::core::unfold::{dedup_names, DeconId, UnfoldPlan, UnfoldShape};

/// The JVM-visible single method name of every generated callback interface.
pub(crate) const IFACE_METHOD: &str = "run";

/// How a raw leaf value is wrapped into its typed form by the generated
/// `asRaw` proxy adapter (and by the onError redispatch / guard defaults).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WrapKind {
    /// Typed == raw (primitives, String, arrays, …): passthrough.
    None,
    /// Opaque handle: raw `Long` → typed handle class (dotted FQN).
    Handle(String),
    /// `Copy` value blob: raw `ByteArray` → `@JvmInline` value class (FQN).
    Blob(String),
}

impl WrapKind {
    /// The Kotlin FQN the wrap constructs, if any (for import registration).
    pub fn class_fqn(&self) -> Option<&str> {
        match self {
            WrapKind::None => None,
            WrapKind::Handle(f) | WrapKind::Blob(f) => Some(f),
        }
    }

    /// The wrapping expression over `arg` (`raw_nullable` adds a null-safe
    /// `?.let`): `ZKeyExpr(x)` / `x?.let { ZKeyExpr(it) }` / passthrough.
    pub fn wrap_expr(&self, arg: &str, raw_nullable: bool) -> String {
        match self.class_fqn() {
            None => arg.to_string(),
            Some(fqn) => {
                let short = fqn.rsplit('.').next().unwrap_or(fqn);
                if raw_nullable {
                    format!("{arg}?.let {{ {short}(it) }}")
                } else {
                    format!("{short}({arg})")
                }
            }
        }
    }
}

/// One `run` parameter of a generated callback interface, in both views.
#[derive(Clone, Debug)]
pub(crate) struct IfaceParam {
    pub name: String,
    /// User-facing type (typed handle class, value class, …).
    pub typed: kt::KtType,
    /// JNI-called raw-twin type (`Long`, `ByteArray`, …) — what the
    /// descriptor and the native jvalues match.
    pub raw: kt::KtType,
    /// How the proxy wraps raw → typed.
    pub wrap: WrapKind,
}

impl IfaceParam {
    fn same(name: String, ty: kt::KtType) -> Self {
        Self {
            name,
            typed: ty.clone(),
            raw: ty,
            wrap: WrapKind::None,
        }
    }
}

/// One generated `fun interface`: identity, Kotlin surface (typed + raw
/// views), and the JVM descriptor of the raw `run` the native side calls.
#[derive(Clone, Debug)]
pub(crate) struct IfaceSpec {
    /// Kotlin package the interface is declared in.
    pub package: String,
    /// Interface short name (`ZSampleCallback`, `ZKeyExprBuilder`, …).
    pub name: String,
    /// Type parameters with variance as written (`["out R"]`, `["A"]`).
    pub type_params: Vec<String>,
    /// `run` parameters in both views. Generic positions use the bare
    /// type-variable name (`A`).
    pub params: Vec<IfaceParam>,
    /// `run` return type (`Unit`, or a bare type variable `R`/`A`).
    pub ret: kt::KtType,
    /// Full JVM descriptor of the RAW `run`, e.g. `"(JLjava/lang/String;)V"`.
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

    /// A [`kt::KtType`] reference to this interface, instantiated with
    /// `args` (empty for a non-generic interface).
    pub fn kt_ref(&self, args: Vec<kt::KtType>) -> kt::KtType {
        if args.is_empty() {
            kt::KtType::cls(self.fqn())
        } else {
            kt::KtType::generic(self.fqn(), args)
        }
    }

    /// True when any param's raw view differs from its typed view — only
    /// then are a raw twin + `asRaw` proxy generated; otherwise the typed
    /// interface IS the JNI-called shape.
    pub fn needs_raw(&self) -> bool {
        self.params.iter().any(|p| p.wrap != WrapKind::None)
    }

    /// Short name of the raw twin (`<Name>Raw`); = `name` when no twin.
    pub fn raw_name(&self) -> String {
        if self.needs_raw() {
            format!("{}Raw", self.name)
        } else {
            self.name.clone()
        }
    }

    pub fn raw_fqn(&self) -> String {
        if self.package.is_empty() {
            self.raw_name()
        } else {
            format!("{}.{}", self.package, self.raw_name())
        }
    }

    /// Slash form of the JNI-called interface for `FindClass`.
    pub fn raw_slash_fqn(&self) -> String {
        self.raw_fqn().replace('.', "/")
    }

    /// Short name of the generated capture holder (`<RawName>Capture`).
    pub fn capture_name(&self) -> String {
        format!("{}Capture", self.raw_name())
    }

    pub fn capture_fqn(&self) -> String {
        if self.package.is_empty() {
            self.capture_name()
        } else {
            format!("{}.{}", self.package, self.capture_name())
        }
    }

    /// A **zero-allocation thread-local capture holder** for an error-handler
    /// interface — replaces the per-call SAM lambda (and the `Ref` boxes
    /// Kotlin allocates for its captured mutable `var`s: ~4 heap objects on
    /// every fallible/infallible outbound call). A final class implementing
    /// the (raw twin) handler, with `@JvmField` slots the native `signal_error`
    /// writes via `run`; one instance per calling thread, reset by `acquire()`.
    ///
    /// Safe to reuse per thread: the error sink is invoked **synchronously**
    /// inside the extern on the calling thread (never the async daemon-callback
    /// thread, which has its own thread-local), and the wrapper reads the slots
    /// into the `onError` arguments *before* any re-entrant call. Assumes the
    /// error-handler shape: `params[0]` is `je: String?`, the rest are `ze`.
    pub fn to_capture_decl(&self) -> kt::KtDecl {
        let cap = self.capture_name();
        let raw = self.raw_name();
        let n_ze = self.params.len() - 1;

        let mut fields = String::from("@JvmField var failed: Boolean = false\n");
        fields.push_str("@JvmField var je: String? = null\n");
        for (i, p) in self.params[1..].iter().enumerate() {
            // The slot is nullable (null until the capture fires).
            let ty = p.raw.clone().nullable();
            fields.push_str(&format!("@JvmField var ze{i}: {ty} = null\n"));
        }

        let run_params = self
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.raw))
            .collect::<Vec<_>>()
            .join(", ");
        let mut run_body = String::from("failed = true; this.je = je");
        for (i, p) in self.params[1..].iter().enumerate() {
            run_body.push_str(&format!("; this.ze{i} = {}", p.name));
        }

        let mut reset = String::from("c.failed = false; c.je = null");
        for i in 0..n_ze {
            reset.push_str(&format!("; c.ze{i} = null"));
        }

        let code = format!(
            "internal class {cap} : {raw}<Unit> {{\n\
             {fields}\
             override fun run({run_params}) {{ {run_body} }}\n\
             companion object {{\n\
             private val TL: ThreadLocal<{cap}> = ThreadLocal.withInitial {{ {cap}() }}\n\
             @JvmStatic fun acquire(): {cap} {{\n\
             val c = TL.get()\n\
             {reset}\n\
             return c\n\
             }}\n\
             }}\n\
             }}"
        );
        kt::KtDecl::Raw {
            name: cap,
            code: kt::Code::raw_reindent(&code),
        }
    }

    /// The typed (user-facing) Kotlin declaration.
    pub fn to_decl(&self) -> kt::KtFunInterface {
        let mut m = kt::KtFun::new(IFACE_METHOD).vis(kt::Vis::Public);
        for p in &self.params {
            m = m.param(kt::KtParam::new(&p.name, p.typed.clone()));
        }
        m = m.returns(self.ret.clone());
        let mut i = kt::KtFunInterface::new(&self.name, m).vis(kt::Vis::Public);
        for tp in &self.type_params {
            i = i.type_param(tp);
        }
        i
    }

    /// The raw-twin declaration (call only when [`Self::needs_raw`]).
    pub fn to_raw_decl(&self) -> kt::KtFunInterface {
        let mut m = kt::KtFun::new(IFACE_METHOD).vis(kt::Vis::Public);
        for p in &self.params {
            m = m.param(kt::KtParam::new(&p.name, p.raw.clone()));
        }
        m = m.returns(self.ret.clone());
        let mut i = kt::KtFunInterface::new(self.raw_name(), m).vis(kt::Vis::Public);
        for tp in &self.type_params {
            i = i.type_param(tp);
        }
        i
    }

    /// The generated proxy: `fun <G> <Name><G>.asRaw(): <Name>Raw<G> =
    /// <Name>Raw { raw leaves… -> run(<wraps…>) }` — constructed once per
    /// registration; per message it performs exactly the typed-object
    /// constructions the consumer needs anyway, in JVM bytecode.
    pub fn to_as_raw_fun(&self) -> kt::KtFun {
        let bare_generics: Vec<String> = self
            .type_params
            .iter()
            .map(|tp| tp.strip_prefix("out ").unwrap_or(tp).trim().to_string())
            .collect();
        let gen_args = if bare_generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", bare_generics.join(", "))
        };
        let recv = format!("{}{gen_args}.asRaw", self.name);
        let wrapped = self
            .params
            .iter()
            .map(|p| p.wrap.wrap_expr(&p.name, p.raw.is_nullable()))
            .collect::<Vec<_>>();
        let body = if self.params.is_empty() {
            kt::Code::new().blk(format!("{}{gen_args} {{", self.raw_name()), |c| {
                c.line("run()")
            })
        } else {
            kt::Code::new().blk(format!("{}{gen_args} {{", self.raw_name()), |mut c| {
                for (idx, name) in self.params.iter().map(|p| p.name.as_str()).enumerate() {
                    let suffix = if idx + 1 == self.params.len() {
                        " ->"
                    } else {
                        ","
                    };
                    c = c.line(format!("{name}{suffix}"));
                }
                c.blk_with("run(", ")", |mut call| {
                    for (idx, expr) in wrapped.iter().enumerate() {
                        let suffix = if idx + 1 == wrapped.len() { "" } else { "," };
                        call = call.line(format!("{expr}{suffix}"));
                    }
                    call
                })
            })
        };
        let mut f = kt::KtFun::new(recv).vis(kt::Vis::Public);
        for g in &bare_generics {
            f = f.generic(g);
        }
        f = f.returns(kt::KtType::cls(format!("{}{gen_args}", self.raw_name())));
        f.expr_body(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_as_raw(spec: IfaceSpec) -> String {
        kt::KtFile::new(&spec.package)
            .decl(spec.to_as_raw_fun())
            .render()
    }

    #[test]
    fn as_raw_adapter_is_multiline_even_when_short() {
        let spec = IfaceSpec {
            package: "io.test".to_string(),
            name: "ThingCallback".to_string(),
            type_params: vec![],
            params: vec![IfaceParam {
                name: "handle".to_string(),
                typed: kt::KtType::cls("io.test.Thing"),
                raw: kt::KtType::long(),
                wrap: WrapKind::Handle("io.test.Thing".to_string()),
            }],
            ret: kt::KtType::unit(),
            descr: "(J)V".to_string(),
        };

        let src = render_as_raw(spec);
        assert!(
            src.contains(
                "public fun ThingCallback.asRaw(): ThingCallbackRaw =\n    \
                 ThingCallbackRaw {\n        \
                 handle ->\n        \
                 run(\n            \
                 Thing(handle)\n        \
                 )\n    \
                 }"
            ),
            "{src}"
        );
    }

    #[test]
    fn as_raw_adapter_breaks_wide_lambda_params_and_run_args() {
        let spec = IfaceSpec {
            package: "io.test".to_string(),
            name: "ReplyCallback".to_string(),
            type_params: vec![],
            params: vec![
                IfaceParam {
                    name: "replierZid".to_string(),
                    typed: kt::KtType::cls("io.test.ZenohId").nullable(),
                    raw: kt::KtType::byte_array().nullable(),
                    wrap: WrapKind::Blob("io.test.ZenohId".to_string()),
                },
                IfaceParam::same("replierEid".to_string(), kt::KtType::int()),
                IfaceParam::same("isOk".to_string(), kt::KtType::boolean()),
                IfaceParam {
                    name: "sample__keyExpr".to_string(),
                    typed: kt::KtType::cls("io.test.KeyExpr").nullable(),
                    raw: kt::KtType::long().nullable(),
                    wrap: WrapKind::Handle("io.test.KeyExpr".to_string()),
                },
                IfaceParam {
                    name: "sample__payload".to_string(),
                    typed: kt::KtType::cls("io.test.ZBytes").nullable(),
                    raw: kt::KtType::long().nullable(),
                    wrap: WrapKind::Handle("io.test.ZBytes".to_string()),
                },
            ],
            ret: kt::KtType::unit(),
            descr: "([BIZLjava/lang/Long;Ljava/lang/Long;)V".to_string(),
        };

        let src = render_as_raw(spec);
        assert!(
            src.contains("public fun ReplyCallback.asRaw(): ReplyCallbackRaw =\n"),
            "{src}"
        );
        assert!(src.contains("    ReplyCallbackRaw {\n"), "{src}");
        assert!(src.contains("        replierZid,\n"), "{src}");
        assert!(src.contains("        sample__payload ->\n"), "{src}");
        assert!(src.contains("        run(\n"), "{src}");
        assert!(
            src.contains("            replierZid?.let { ZenohId(it) },\n"),
            "{src}"
        );
        assert!(
            src.contains("            sample__payload?.let { ZBytes(it) }\n"),
            "{src}"
        );
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
            return if *nullable {
                boxed.to_string()
            } else {
                p.to_string()
            };
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

fn method_descr(params: &[IfaceParam], ret: &kt::KtType, type_params: &[String]) -> String {
    let mut d = String::from("(");
    for p in params {
        d.push_str(&kt_jvm_descriptor(&p.raw, type_params));
    }
    d.push(')');
    d.push_str(&kt_jvm_descriptor(ret, type_params));
    d
}

/// The interface base name for a decomposition: the subject type's short
/// name, extended by the deconstructor declaration's identity. The type's
/// canonical (unnamed) declaration keeps the bare short; a named alternative
/// appends its UpperCamel name (`ZError` + `"full"` → `ZErrorFull`); per-fn
/// inline records (`.output`) append the function's UpperCamel ident.
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
        None | Some(DeconId::Default(_)) => short.to_string(),
        Some(DeconId::Named(_, n)) => format!("{short}{}", upper_camel(n)),
        Some(DeconId::PerFn(_, f)) => format!("{short}{}", upper_camel(f)),
    }
}

/// Short name of a Rust type key (`zenoh_flat::ZSample` → `ZSample`),
/// peeled of `&` / `Option`.
fn subject_short(ty: &syn::Type) -> String {
    let peeled = crate::api::core::types_util::peel_ref_option_vec(ty);
    if let syn::Type::Path(tp) = &peeled {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident.to_string();
        }
    }
    TypeKey::from_type(&peeled)
        .to_string()
        .replace([' ', ':', '<', '>'], "")
}

/// Package a subject type's interface lives in: the package of the type's
/// registered Kotlin FQN, the root `ext.package` otherwise.
fn subject_package(ext: &JniGen<impl JniGenState>, subject: &syn::Type) -> String {
    let key =
        TypeKey::from_type(&crate::api::core::types_util::peel_ref_option_vec(subject)).to_string();
    ext.kotlin_fqn(&key)
        .and_then(|fqn| fqn.rsplit_once('.').map(|(p, _)| p.to_string()))
        .unwrap_or_else(|| ext.package.clone())
}

/// The interface param list for a decomposition's leaves: names from
/// [`plan_leaf_names`], typed + raw views per leaf.
fn plan_leaf_params(
    ext: &JniGen<impl JniGenState>,
    registry: &Registry<KotlinMeta>,
    leaves: &[crate::api::core::unfold::UnfoldLeaf],
) -> Option<Vec<IfaceParam>> {
    // Decomposition leaf names are author-supplied, literal, and unique by
    // construction (enforced in `core::unfold`) — no dedup/casing here.
    let names = plan_leaf_names(leaves);
    let mut out = Vec::with_capacity(leaves.len());
    for (name, leaf) in names.into_iter().zip(leaves.iter()) {
        out.push(leaf_iface_param(
            ext,
            registry,
            name,
            &leaf.out_ty,
            leaf.nullable,
            true,
        )?);
    }
    Some(out)
}

/// Both interface views of one delivered leaf.
///
/// * **typed** (user-facing, Kotlin-called): handles as their typed handle
///   classes, value blobs as their `@JvmInline` value classes — legal here
///   because the JNI border never touches this method.
/// * **raw** (JNI-called twin): a PLAN leaf (`raw_handle`) crosses handles
///   as the raw `jlong` (`Long`/boxed `Long?` — the proxy constructs the
///   class in bytecode; a native `new_object` would cost descriptor parse +
///   FindClass + GetMethodID + NewObjectA per message) and blobs as
///   `ByteArray` (the `@JvmInline` class would mangle `run`). A whole
///   (plan-less callback) arg keeps the typed handle class in BOTH views —
///   the close-unless-taken contract needs the native side to `close()` the
///   wrapped object after the invoke.
pub(crate) fn leaf_iface_param(
    ext: &JniGen<impl JniGenState>,
    registry: &Registry<KotlinMeta>,
    name: String,
    out_ty: &syn::Type,
    nullable: bool,
    raw_handle: bool,
) -> Option<IfaceParam> {
    // The declaration-canonical spec normalizes its identity leaf to the
    // borrowed `&T` form; a function set that only ever returns the type
    // OWNED resolved only `T`'s output entry. Both classify to the same
    // projection/class, so fall back to the peeled form when the borrowed
    // entry was never required.
    let mut out_ty = out_ty;
    let peeled: syn::Type;
    if registry.output_entry(out_ty).is_none() {
        if let syn::Type::Reference(r) = out_ty {
            peeled = (*r.elem).clone();
            out_ty = &peeled;
        }
    }
    let mut throwaway = BTreeSet::new();
    let (builder_kt, _wire_kt, _wrap, is_vb) =
        unfold_leaf_kt(ext, registry, out_ty, nullable, "x", &mut throwaway)?;
    let proj = registry
        .output_entry(out_ty)
        .and_then(|e| e.metadata.projection.as_ref());
    let nullable_kt = |t: kt::KtType| {
        if builder_kt.is_nullable() {
            t.nullable()
        } else {
            t
        }
    };
    if is_vb {
        let fqn = proj
            .and_then(|p| ext.kotlin_fqn(&p.leaf_key))
            .map(|f| f.to_string())?;
        return Some(IfaceParam {
            name,
            typed: nullable_kt(kt::KtType::cls(fqn.clone())),
            raw: nullable_kt(kt::KtType::byte_array()),
            wrap: WrapKind::Blob(fqn),
        });
    }
    if let Some(p) = proj.filter(|p| p.kind == ProjectionKind::Handle) {
        let fqn = ext.kotlin_fqn(&p.leaf_key)?.to_string();
        if raw_handle {
            return Some(IfaceParam {
                name,
                typed: nullable_kt(kt::KtType::cls(fqn.clone())),
                raw: nullable_kt(kt::KtType::long()),
                wrap: WrapKind::Handle(fqn),
            });
        }
        // Whole arg: typed class in both views (no proxy wrap).
        return Some(IfaceParam::same(name, nullable_kt(kt::KtType::cls(fqn))));
    }
    Some(IfaceParam::same(name, builder_kt))
}

/// Interface for an `impl Fn(args)` delivery: one `run` parameter per
/// flattened leaf of each arg's callback plan (the arg whole when plan-less),
/// returning `Unit`. Named `<ArgShorts>Callback` (`Fn()` → `VoidCallback`),
/// placed in the first arg type's package (root for `Fn()`).
pub(crate) fn callback_iface_spec(
    ext: &JniGen<impl JniGenState>,
    registry: &Registry<KotlinMeta>,
    cb_args: &[syn::Type],
) -> Option<IfaceSpec> {
    let mut leaf_tys: Vec<(String, syn::Type, bool, bool)> = Vec::new();
    for (i, t) in cb_args.iter().enumerate() {
        if let Some(plan) = registry.callback_arg_plans.get(&TypeKey::from_type(t)) {
            leaf_tys.extend(
                plan_leaf_names(&plan.leaves)
                    .into_iter()
                    .zip(plan.leaves.iter())
                    .map(|(n, l)| (n, l.out_ty.clone(), l.nullable, true)),
            );
        } else {
            leaf_tys.push((whole_value_name(t, i), t.clone(), is_option_type(t), false));
        }
    }
    let mut names: Vec<String> = leaf_tys.iter().map(|(n, _, _, _)| n.clone()).collect();
    dedup_names(&mut names);
    let mut params = Vec::with_capacity(leaf_tys.len());
    for (k, (_, out_ty, nullable, from_plan)) in leaf_tys.iter().enumerate() {
        params.push(leaf_iface_param(
            ext,
            registry,
            names[k].clone(),
            out_ty,
            *nullable,
            *from_plan,
        )?);
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
    ext: &JniGen<impl JniGenState>,
    registry: &Registry<KotlinMeta>,
    decon: &DeconId,
) -> Option<IfaceSpec> {
    let spec = registry.decon_plans.get(decon)?;
    let params = plan_leaf_params(ext, registry, &spec.leaves)?;
    let name = format!(
        "{}Builder",
        decon_base_name(&subject_short(&spec.source), Some(decon))
    );
    let package = subject_package(ext, &spec.source);
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
    ext: &JniGen<impl JniGenState>,
    registry: &Registry<KotlinMeta>,
    decon: &DeconId,
) -> Option<IfaceSpec> {
    let spec = registry.decon_plans.get(decon)?;
    let mut params: Vec<IfaceParam> =
        vec![IfaceParam::same("acc".to_string(), kt::KtType::var_("A"))];
    params.extend(plan_leaf_params(ext, registry, &spec.leaves)?);
    let name = format!(
        "{}Folder",
        decon_base_name(&subject_short(&spec.source), Some(decon))
    );
    let package = subject_package(ext, &spec.source);
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
    ext: &JniGen<impl JniGenState>,
    registry: &Registry<KotlinMeta>,
    element: &syn::Type,
) -> Option<IfaceSpec> {
    let mut params: Vec<IfaceParam> =
        vec![IfaceParam::same("acc".to_string(), kt::KtType::var_("A"))];
    params.push(leaf_iface_param(
        ext,
        registry,
        "element".to_string(),
        element,
        false,
        false,
    )?);
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
    ext: &JniGen<impl JniGenState>,
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
/// ze-leaves…): R`, `<out R>`. The `ze` leaves are typed EXACTLY like a
/// builder's for the same decomposition — the error channel IS the output
/// channel with a fixed leading `je`. Contract: `je != null` ⇒ binding/system
/// error, the ze carry **default values** (0 / "" / empty / closed handle /
/// null for plan-nullable leaves); `je == null` ⇒ domain error, the ze carry
/// the decomposed error. Keyed by the error type's deconstructor declaration.
/// Named `<decl-base>Handler`, placed in the error type's package.
pub(crate) fn error_handler_iface_spec(
    ext: &JniGen<impl JniGenState>,
    registry: &Registry<KotlinMeta>,
    decon: &DeconId,
) -> Option<IfaceSpec> {
    let spec = registry.decon_plans.get(decon)?;
    let mut params: Vec<IfaceParam> = vec![IfaceParam::same(
        "je".to_string(),
        kt::KtType::string().nullable(),
    )];
    params.extend(plan_leaf_params(ext, registry, &spec.leaves)?);
    let name = format!(
        "{}Handler",
        decon_base_name(&subject_short(&spec.source), Some(decon))
    );
    let package = subject_package(ext, &spec.source);
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
pub(crate) fn jni_error_handler_iface_spec(ext: &JniGen<impl JniGenState>) -> IfaceSpec {
    let params = vec![IfaceParam::same(
        "je".to_string(),
        kt::KtType::string().nullable(),
    )];
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
    ext: &JniGen<impl JniGenState>,
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
