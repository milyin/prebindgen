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
//! Each identity's spec is derived ONCE, through the [`JniGen::iface_spec`]
//! memo keyed by [`SpecKey`], and shared by all three sites — the
//! FQN/descriptor pair cannot drift between the artifact tiers (issue #107).
//! The constructors stay deterministic over `(ext, registry)`; in debug
//! builds every memo hit re-derives and asserts equality, so the
//! determinism is a checked invariant rather than a convention.

use super::*;
use crate::api::core::unfold::{dedup_names, DeconId, UnfoldPlan};

/// The JVM-visible single method name of every generated callback interface.
pub(crate) const IFACE_METHOD: &str = "run";

/// The `@JvmField` name holding a hoisted singleton inside its holder object
/// (see [`IfaceSpec::singleton_holder_name`]).
pub(crate) const SINGLETON_FIELD: &str = "instance";

/// How a raw leaf value is wrapped into its typed form by the generated
/// `asRaw` proxy adapter (and by the onError redispatch / guard defaults).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WrapKind {
    /// Typed == raw (primitives, String, arrays, …): passthrough.
    None,
    /// Opaque handle: raw `Long` → typed handle class (dotted FQN).
    Handle(String),
    /// Opaque handle delivered to a callback as a **transient owned** value
    /// (raw `Long` → typed handle class, dotted FQN): the `asRaw` proxy wraps it
    /// AND closes it in a `finally` after `run` returns — the close-unless-taken
    /// contract (a no-op if the user consumed/`take()`-ed the handle, which
    /// zeroes its `ptr`). Replaces the former Rust-side `new_object` + post-invoke
    /// `close()` for a plan-less `impl Fn(Handle)` arg (Phase 3).
    HandleOwned(String),
    /// `Copy` value blob: raw `ByteArray` → `@JvmInline` value class (FQN).
    Blob(String),
}

impl WrapKind {
    /// The Kotlin FQN the wrap constructs, if any (for import registration).
    pub fn class_fqn(&self) -> Option<&str> {
        match self {
            WrapKind::None => None,
            WrapKind::Handle(f) | WrapKind::HandleOwned(f) | WrapKind::Blob(f) => Some(f),
        }
    }

    /// `true` for [`Self::HandleOwned`] — the proxy must close the wrapped handle
    /// after `run`.
    pub fn is_owned_handle(&self) -> bool {
        matches!(self, WrapKind::HandleOwned(_))
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

/// One **typed-view group** over a run of raw leaf [`params`](IfaceSpec::params):
/// the user-facing callback sees this single value, the JNI-called raw twin sees
/// the leaves. Used for a by-value `data_class` callback arg, whose user
/// callback receives the whole reassembled object (`run(p: Payload)`) while the
/// wire carries decoupled leaves (so no Java object is built on the Rust side).
#[derive(Clone, Debug)]
pub(crate) struct TypedGroup {
    /// User-facing (typed) parameter name.
    pub name: String,
    /// User-facing (typed) parameter type (the whole value, or a single leaf's
    /// typed view for a passthrough group).
    pub typed: kt::KtType,
    /// `Some(class_short)` ⇒ reassemble this group's leaves via
    /// `class_short.fromParts(leaves…)`; `None` ⇒ a single passthrough leaf
    /// (the typed value IS the one raw param, optionally wrapped by its
    /// [`IfaceParam::wrap`]).
    pub reassemble: Option<String>,
    /// Number of consecutive `params` (raw leaves) this group consumes.
    pub leaf_count: usize,
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
    /// Typed-view grouping over [`Self::params`]. Empty ⇒ the typed view is the
    /// params 1:1 (the default). Non-empty ⇒ the user-facing `run` takes one
    /// param per group (a whole reassembled value), while the JNI-called raw
    /// twin still takes the leaf `params`; the `asRaw` proxy reassembles. Always
    /// forces a raw twin (see [`Self::needs_raw`]).
    pub typed_groups: Vec<TypedGroup>,
    /// Interface-level KDoc, rendered on the TYPED declaration (the raw
    /// twin is internal machinery). `None` = no doc.
    pub kdoc: Option<String>,
}

impl IfaceSpec {
    /// Assemble a spec, deriving the JVM `run` descriptor — the only
    /// computed field — from the `params` / `ret` / `type_params` triple.
    /// `typed_groups` starts empty (callback specs override it via struct
    /// update).
    fn assemble(
        package: String,
        name: String,
        type_params: Vec<String>,
        params: Vec<IfaceParam>,
        ret: kt::KtType,
    ) -> Self {
        let descr = method_descr(&params, &ret, &type_params);
        IfaceSpec {
            package,
            name,
            type_params,
            params,
            ret,
            descr,
            kdoc: None,
            typed_groups: Vec::new(),
        }
    }

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

    /// True when the typed view differs from the JNI-called raw view — only
    /// then are a raw twin + `asRaw` proxy generated; otherwise the typed
    /// interface IS the JNI-called shape. A per-param `wrap` (handle/blob) or a
    /// non-empty [`Self::typed_groups`] (whole-value reassembly) both differ.
    pub fn needs_raw(&self) -> bool {
        !self.typed_groups.is_empty() || self.params.iter().any(|p| p.wrap != WrapKind::None)
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

    /// Short name of the hoisted-singleton **holder object** (`<RawName>Holder`).
    /// A fixed builder/folder singleton lives as a `@JvmField` in this object so
    /// it has a stable JVM class + static field that native code can fetch
    /// (`FindClass` + `GetStaticField`) — unlike a top-level `val`, whose backing
    /// field hides behind a file-name-derived facade class.
    pub fn singleton_holder_name(&self) -> String {
        format!("__{}Holder", self.raw_name())
    }

    /// Dotted FQN of the singleton holder object.
    pub fn singleton_holder_fqn(&self) -> String {
        if self.package.is_empty() {
            self.singleton_holder_name()
        } else {
            format!("{}.{}", self.package, self.singleton_holder_name())
        }
    }

    /// Slash form of the singleton holder FQN (for `FindClass`).
    pub fn singleton_holder_slash_fqn(&self) -> String {
        self.singleton_holder_fqn().replace('.', "/")
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

        let mut fields = kt::Code::new()
            .line("@JvmField var failed: Boolean = false")
            .line("@JvmField var je: String? = null");
        for (i, p) in self.params[1..].iter().enumerate() {
            // The slot is nullable (null until the capture fires).
            let ty = p.raw.clone().nullable();
            fields = fields.line(format!("@JvmField var ze{i}: {ty} = null"));
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

        let code = kt::Code::new().blk(format!("internal class {cap} : {raw}<Unit> {{"), |c| {
            c.push(fields)
                .wline(format!("override fun run({run_params}) {{ {run_body} }}"))
                .blk("companion object {", |comp| {
                    comp.line(format!(
                        "private val TL: ThreadLocal<{cap}> = ThreadLocal.withInitial {{ {cap}() }}"
                    ))
                    .blk(format!("@JvmStatic fun acquire(): {cap} {{"), |acq| {
                        acq.line("val c = TL.get()").wline(reset).line("return c")
                    })
                })
        });
        kt::KtDecl::Raw { name: cap, code }
    }

    /// The typed (user-facing) Kotlin declaration. With [`Self::typed_groups`]
    /// the `run` parameters are the groups (each a whole reassembled value);
    /// otherwise they are the `params` 1:1 (typed view).
    pub fn to_decl(&self) -> kt::KtFunInterface {
        let mut m = kt::KtFun::new(IFACE_METHOD).vis(kt::Vis::Public);
        if self.typed_groups.is_empty() {
            for p in &self.params {
                m = m.param(kt::KtParam::new(&p.name, p.typed.clone()));
            }
        } else {
            for g in &self.typed_groups {
                m = m.param(kt::KtParam::new(&g.name, g.typed.clone()));
            }
        }
        m = m.returns(self.ret.clone());
        let mut i = kt::KtFunInterface::new(&self.name, m).vis(kt::Vis::Public);
        if let Some(doc) = &self.kdoc {
            i = i.kdoc(doc.clone());
        }
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
        // `run` arguments: per-param `wrap` (1:1 view) or, with `typed_groups`,
        // one expression per group — a whole value reassembled from its leaves
        // via `Class.fromParts(leaves…)`, or a single passthrough leaf's wrap.
        // An owned-handle arg (`HandleOwned`) must be bound to a local and
        // `close()`-d after `run` (close-unless-taken), so track it per arg.
        struct RunArg {
            expr: String,
            owned: bool,
            nullable: bool,
        }
        let run_args: Vec<RunArg> = if self.typed_groups.is_empty() {
            self.params
                .iter()
                .map(|p| RunArg {
                    expr: p.wrap.wrap_expr(&p.name, p.raw.is_nullable()),
                    owned: p.wrap.is_owned_handle(),
                    nullable: p.raw.is_nullable(),
                })
                .collect()
        } else {
            let mut args = Vec::with_capacity(self.typed_groups.len());
            let mut at = 0usize;
            for g in &self.typed_groups {
                let names: Vec<&str> = self.params[at..at + g.leaf_count]
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect();
                match &g.reassemble {
                    Some(cls) => args.push(RunArg {
                        expr: format!("{cls}.fromParts({})", names.join(", ")),
                        owned: false,
                        nullable: false,
                    }),
                    None => {
                        let p = &self.params[at];
                        args.push(RunArg {
                            expr: p.wrap.wrap_expr(&p.name, p.raw.is_nullable()),
                            owned: p.wrap.is_owned_handle(),
                            nullable: p.raw.is_nullable(),
                        });
                    }
                }
                at += g.leaf_count;
            }
            args
        };
        let any_owned = run_args.iter().any(|a| a.owned);
        let lambda_open = format!("{}{gen_args} {{", self.raw_name());
        let body = if self.params.is_empty() {
            kt::Code::new().blk(lambda_open, |c| c.line("run()"))
        } else if !any_owned {
            // Common case: a single `run(<wrapped…>)` call (no transient handle).
            let wrapped: Vec<&str> = run_args.iter().map(|a| a.expr.as_str()).collect();
            kt::Code::new().blk(lambda_open, |mut c| {
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
        } else {
            // Owned-handle arg(s): bind each to a local, `run(...)` with the
            // locals, and `close()` every owned local in a `finally` so the
            // per-invocation handle's `Box` is freed even if `run` threw — a
            // no-op when the consumer `take()`-ed it (its `ptr` is then 0).
            let lambda_params = self
                .params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let mut lines: Vec<String> = Vec::new();
            let mut call_args: Vec<String> = Vec::with_capacity(run_args.len());
            let mut closes: Vec<String> = Vec::new();
            let mut owned_idx = 0usize;
            for a in &run_args {
                if a.owned {
                    let local = format!("__own{owned_idx}");
                    owned_idx += 1;
                    lines.push(format!("val {local} = {}", a.expr));
                    call_args.push(local.clone());
                    let dot = if a.nullable { "?." } else { "." };
                    closes.push(format!("{local}{dot}close()"));
                } else {
                    call_args.push(a.expr.clone());
                }
            }
            kt::Code::new().blk(lambda_open, |mut c| {
                c = c.line(format!("{lambda_params} ->"));
                for l in &lines {
                    c = c.wline(l.clone());
                }
                let mut fin = kt::Code::new();
                for cl in &closes {
                    fin = fin.line(cl.clone());
                }
                c.try_finally(
                    "",
                    kt::Code::new().wline(format!("run({})", call_args.join(", "))),
                    fin,
                )
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
mod tests;

/// The JVM descriptor chunk for a parameter/return Kotlin type.
/// `type_params` are the interface's bare type-variable names (variance
/// stripped) — they erase to `Object`. `Unit` maps to `V` (valid only in
/// return position; parameters never carry `Unit`).
///
/// Loud panic on anything unrecognized: a silently-wrong descriptor would
/// surface as a runtime `GetMethodID` failure (or worse, a mistyped jvalue).
fn kt_jvm_descriptor(ty: &kt::KtType, type_params: &[String]) -> String {
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
        if let Some(p) = JniPrim::from_kotlin_name(simple) {
            return if *nullable {
                p.box_descriptor().to_string()
            } else {
                p.descriptor().to_string()
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
/// default declaration keeps the bare short; per-fn inline records
/// (`.expand_return()`) append the function's UpperCamel ident.
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
        Some(DeconId::PerFn(_, f)) => format!("{short}{}", upper_camel(f)),
    }
}

/// Short name of a Rust type key (`zenoh_flat::ZSample` → `ZSample`), peeled of
/// `&` / `Option`. A slice element (`&[E]` / `[E]`, a collection callback arg that
/// surfaces as `List<E>`) gets a `List` suffix — `<E>List` — so it yields a valid,
/// distinct interface name (`<E>ListCallback`) instead of the bracketed `[E]` and
/// without colliding with the scalar `&E` callback (`<E>Callback`).
fn subject_short(ty: &syn::Type) -> String {
    let no_ref = match ty {
        syn::Type::Reference(r) => (*r.elem).clone(),
        other => other.clone(),
    };
    if let syn::Type::Slice(s) = &no_ref {
        return format!("{}List", subject_short(&s.elem));
    }
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
fn subject_package(ext: &JniGen, subject: &syn::Type) -> String {
    let key =
        TypeKey::from_type(&crate::api::core::types_util::peel_ref_option_vec(subject)).to_string();
    ext.kotlin_fqn(&key)
        .and_then(|fqn| fqn.rsplit_once('.').map(|(p, _)| p.to_string()))
        .unwrap_or_else(|| ext.package.clone())
}

/// The interface param list for a decomposition's leaves: names from
/// [`plan_leaf_names`], typed + raw views per leaf.
fn plan_leaf_params(
    ext: &JniGen,
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
fn leaf_iface_param(
    ext: &JniGen,
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
    // A whole generated class (e.g. a field-based `data_class` delivered to a
    // callback by `impl Fn(&T)`): `builder_kt` carries the unqualified short name
    // (the wrapper relies on an import), but the `raw` view — which drives the JNI
    // method descriptor — must be the fully-qualified class so `get_method_id("run",
    // …)` resolves. Apply only when `builder_kt` actually IS that class (its short
    // name matches the registered FQN); never for enums (→ `Int`) or builtins. `wrap`
    // stays `None` (typed and raw are the same JVM type, just spelled differently), so
    // no `asRaw` proxy is generated.
    if let kt::KtType::Named { fqn: bk_fqn, .. } = &builder_kt {
        if !bk_fqn.contains('.') {
            if let Some(reg_fqn) = ext.kotlin_fqn(&TypeKey::from_type(out_ty).to_string()) {
                let reg_short = reg_fqn.rsplit('.').next().unwrap_or(&reg_fqn);
                if reg_fqn.contains('.') && reg_short == bk_fqn {
                    let raw = kt::KtType::cls(reg_fqn.to_string());
                    let raw = if builder_kt.is_nullable() {
                        raw.nullable()
                    } else {
                        raw
                    };
                    return Some(IfaceParam {
                        name,
                        typed: builder_kt.clone(),
                        raw,
                        wrap: WrapKind::None,
                    });
                }
            }
        }
    }
    Some(IfaceParam::same(name, builder_kt))
}

/// The [`IfaceParam`] for a **plan-less opaque-handle callback arg** (Phase 3):
/// typed = the handle class, raw = `Long` (the `jlong` pointer the native
/// trampoline delivers), wrap = [`WrapKind::HandleOwned`] so the generated
/// `asRaw` proxy wraps the pointer into the handle class and `close()`s it after
/// `run` (close-unless-taken). Replaces the former Rust-side `new_object` +
/// post-invoke `close()`. `None` if the arg's projection FQN can't be resolved.
pub(crate) fn owned_handle_iface_param(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    name: String,
    out_ty: &syn::Type,
    nullable: bool,
) -> Option<IfaceParam> {
    let proj = registry.output_entry(out_ty)?.metadata.projection.clone()?;
    let fqn = ext.kotlin_fqn(&proj.leaf_key)?.to_string();
    let typed = kt::KtType::cls(fqn.clone());
    let (typed, raw) = if nullable {
        (typed.nullable(), kt::KtType::long().nullable())
    } else {
        (typed, kt::KtType::long())
    };
    Some(IfaceParam {
        name,
        typed,
        raw,
        wrap: WrapKind::HandleOwned(fqn),
    })
}

// ──────────────────────────────────────────────────────────────────────
// Spec identities + the per-generator memo (issue #107)
// ──────────────────────────────────────────────────────────────────────

/// One distinct generated-interface identity — the memo key shared by the
/// resolve-time trampoline, the per-function plan, and the declaration
/// emitter (formerly `write_callback_ifaces`' local `Use` enum). `Ord` so
/// declaration emission iterates deterministically. Type-shaped identities
/// store the canonical [`TypeKey`] string; the derivation round-trips it
/// back to a `syn::Type`, so a key alone fully determines its spec.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SpecKey {
    /// impl-Fn delivery — the args' canonical type keys (each arg either
    /// decomposes via its type's canonical plan or crosses whole).
    Callback(Vec<String>),
    Builder(DeconId),
    Folder(DeconId),
    /// Whole-element fold — no declaration; keyed by element type.
    WholeFolder(String),
    Handler(DeconId),
    JniErrorHandler,
}

impl SpecKey {
    /// The impl-Fn identity for a callback's arg types.
    pub fn callback(args: &[syn::Type]) -> Self {
        SpecKey::Callback(
            args.iter()
                .map(|t| TypeKey::from_type(t).to_string())
                .collect(),
        )
    }

    /// The whole-element fold identity for an element type.
    pub fn whole_folder(element: &syn::Type) -> Self {
        SpecKey::WholeFolder(TypeKey::from_type(element).to_string())
    }
}

/// DeconIds whose builder/folder is a synthesized by-value `data_class`
/// (`fixed_builder` plans). Fixedness is a property of the DECLARATION
/// identity — the JVM resolves calls against the single declared interface —
/// so it is computed per `DeconId` over all plans, shared by the memo
/// derivation ([`SpecKey::Folder`]'s typed groups) and the declaration
/// emitter (the hoisted `fromParts`/appender singletons).
pub(crate) fn fixed_decon_ids(
    registry: &Registry<KotlinMeta>,
) -> std::collections::HashSet<DeconId> {
    registry
        .unfold_plans
        .values()
        .chain(registry.callback_arg_plans.values())
        .filter(|p| p.fixed_builder)
        .filter_map(|p| p.decon.clone())
        .collect()
}

/// Element type keys whose whole-element fold is fixed (a synthesized
/// single-leaf `Vec<T>` fold) — the leaf dual of [`fixed_decon_ids`], used
/// by the declaration emitter for the hoisted appender singleton.
pub(crate) fn fixed_leaf_element_keys(
    registry: &Registry<KotlinMeta>,
) -> std::collections::HashSet<String> {
    registry
        .unfold_plans
        .values()
        .chain(registry.callback_arg_plans.values())
        .filter(|p| p.fixed_builder)
        .filter_map(|p| p.element.as_ref())
        .map(|el| TypeKey::from_type(el).to_string())
        .collect()
}

/// Derive the spec for one identity — the SINGLE construction point behind
/// [`JniGen::iface_spec`]. Reconstructs any `syn` context from the key's
/// canonical type strings ([`TypeKey::to_type`] round-trip). A `Folder`
/// derivation folds the fixed-builder typed-group view in per `DeconId`
/// (see [`fixed_decon_ids`]).
fn derive_iface_spec(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    key: &SpecKey,
) -> Option<IfaceSpec> {
    match key {
        SpecKey::Callback(arg_keys) => {
            let args: Vec<syn::Type> = arg_keys
                .iter()
                .map(|k| {
                    TypeKey::parse(k)
                        .expect("SpecKey stores canonical type strings")
                        .to_type()
                })
                .collect();
            callback_iface_spec(ext, registry, &args)
        }
        SpecKey::Builder(d) => builder_iface_spec(ext, registry, d),
        SpecKey::Folder(d) => {
            let mut spec = folder_iface_spec(ext, registry, d)?;
            if fixed_decon_ids(registry).contains(d) {
                spec.typed_groups = fixed_folder_typed_groups(ext, registry, d)?;
            }
            Some(spec)
        }
        SpecKey::WholeFolder(el_key) => whole_folder_iface_spec(
            ext,
            registry,
            &TypeKey::parse(el_key)
                .expect("SpecKey stores canonical type strings")
                .to_type(),
        ),
        SpecKey::Handler(d) => error_handler_iface_spec(ext, registry, d),
        SpecKey::JniErrorHandler => Some(jni_error_handler_iface_spec(ext)),
    }
}

impl JniGen {
    /// The memoized spec for one interface identity: derived once per
    /// generator run and shared by every consumer — the resolve-time
    /// trampoline, the per-function plan, and the declaration emitter — so
    /// the FQN/descriptor pair cannot drift between artifact tiers (issue
    /// #107). `None` (not yet derivable — e.g. leaf converters still
    /// unresolved) is NOT cached: the resolve-time caller defers and
    /// retries. In debug builds a cache hit re-derives and asserts equality,
    /// so any nondeterminism in the constructors fails loudly under test
    /// instead of shipping descriptor drift.
    pub(crate) fn iface_spec(
        &self,
        registry: &Registry<KotlinMeta>,
        key: &SpecKey,
    ) -> Option<std::sync::Arc<IfaceSpec>> {
        let hit = self.iface_specs.borrow().get(key).cloned();
        if let Some(hit) = hit {
            #[cfg(debug_assertions)]
            {
                let fresh = derive_iface_spec(self, registry, key);
                debug_assert_eq!(
                    fresh.as_ref().map(|s| format!("{s:?}")),
                    Some(format!("{:?}", *hit)),
                    "IfaceSpec derivation drifted for {key:?}"
                );
            }
            return Some(hit);
        }
        let spec = std::sync::Arc::new(derive_iface_spec(self, registry, key)?);
        self.iface_specs
            .borrow_mut()
            .insert(key.clone(), spec.clone());
        Some(spec)
    }
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
    // Per-arg grouping over the flat raw leaves. A **fixed-builder** (by-value
    // `data_class`) arg crosses the wire as decoupled leaves but the user
    // callback receives the whole reassembled value (one group, reassembled via
    // `fromParts`). Every other arg keeps the established behavior: an
    // accessor-plan arg exposes its leaves to the user 1:1 (one passthrough
    // group each), a plan-less arg crosses whole (one passthrough group).
    struct GroupDesc {
        name: String,
        /// Whole-value typed view (`Some`) or `None` ⇒ use the leaf's own typed.
        typed: Option<kt::KtType>,
        reassemble: Option<String>,
        leaf_count: usize,
    }
    // (leaf name, out_ty, nullable, from_plan, owned_handle). `owned_handle`
    // marks a plan-less opaque-handle arg delivered as a raw `jlong` and wrapped
    // + closed Kotlin-side (Phase 3).
    let mut leaf_tys: Vec<(String, syn::Type, bool, bool, bool)> = Vec::new();
    let mut groups: Vec<GroupDesc> = Vec::new();
    let mut any_fixed = false;

    for (i, t) in cb_args.iter().enumerate() {
        // An `Iterable` fixed-folder plan (an `&[data_class]` arg) is delivered by
        // FOLDING in the trampoline — the user callback still sees ONE whole
        // `List<Element>`, so it takes the plain whole-value path below (no leaf
        // params, no reassembly group). Only `Base`/accessor plans decompose the
        // arg into the callback's `run` params here.
        let plan = registry
            .callback_arg_plans
            .get(&TypeKey::from_type(t))
            .filter(|p| !super::render::is_iterable_fold(&p.shape));
        if let Some(plan) = plan {
            let leaf_names = plan_leaf_names(&plan.leaves);
            for (n, l) in leaf_names.iter().zip(plan.leaves.iter()) {
                leaf_tys.push((n.clone(), l.out_ty.clone(), l.nullable, true, false));
            }
            if plan.fixed_builder {
                any_fixed = true;
                let core = match t {
                    syn::Type::Reference(r) => (*r.elem).clone(),
                    other => other.clone(),
                };
                let fqn = ext.kotlin_fqn(&TypeKey::from_type(&core).to_string())?;
                let class_short = fqn.rsplit('.').next().unwrap_or(&fqn).to_string();
                groups.push(GroupDesc {
                    name: whole_value_name(t, i),
                    typed: Some(kt::KtType::cls(fqn.to_string())),
                    reassemble: Some(class_short),
                    leaf_count: plan.leaves.len(),
                });
            } else {
                // Accessor-plan arg: each leaf is its own passthrough group, so
                // the user callback still sees the flattened leaves (unchanged).
                for n in &leaf_names {
                    groups.push(GroupDesc {
                        name: n.clone(),
                        typed: None,
                        reassemble: None,
                        leaf_count: 1,
                    });
                }
            }
        } else {
            // A plan-less opaque-handle arg is delivered as a raw `jlong` and
            // wrapped + closed Kotlin-side (Phase 3 — no Rust `new_object`).
            let owned_handle = registry
                .output_entry(t)
                .and_then(|e| e.metadata.projection.as_ref())
                .map(|p| p.kind == ProjectionKind::Handle)
                .unwrap_or(false);
            leaf_tys.push((
                whole_value_name(t, i),
                t.clone(),
                is_option_type(t),
                false,
                owned_handle,
            ));
            groups.push(GroupDesc {
                name: whole_value_name(t, i),
                typed: None,
                reassemble: None,
                leaf_count: 1,
            });
        }
    }
    let mut names: Vec<String> = leaf_tys.iter().map(|(n, ..)| n.clone()).collect();
    dedup_names(&mut names);
    let mut params = Vec::with_capacity(leaf_tys.len());
    for (k, (_, out_ty, nullable, from_plan, owned_handle)) in leaf_tys.iter().enumerate() {
        let param = if *owned_handle {
            owned_handle_iface_param(ext, registry, names[k].clone(), out_ty, *nullable)?
        } else {
            leaf_iface_param(
                ext,
                registry,
                names[k].clone(),
                out_ty,
                *nullable,
                *from_plan,
            )?
        };
        params.push(param);
    }
    // Typed groups only when a fixed-builder arg is present — otherwise the
    // typed view is the params 1:1 (preserving the established accessor /
    // whole-value behavior for every other callback).
    let typed_groups = if any_fixed {
        let mut tg = Vec::with_capacity(groups.len());
        let mut at = 0usize;
        let mut group_names: Vec<String> = groups.iter().map(|g| g.name.clone()).collect();
        dedup_names(&mut group_names);
        for (gi, g) in groups.iter().enumerate() {
            let typed = g.typed.clone().unwrap_or_else(|| params[at].typed.clone());
            tg.push(TypedGroup {
                name: group_names[gi].clone(),
                typed,
                reassemble: g.reassemble.clone(),
                leaf_count: g.leaf_count,
            });
            at += g.leaf_count;
        }
        tg
    } else {
        Vec::new()
    };
    let name = if cb_args.is_empty() {
        "VoidCallback".to_string()
    } else {
        format!(
            "{}Callback",
            cb_args
                .iter()
                .map(subject_short)
                .collect::<Vec<_>>()
                .join("")
        )
    };
    let package = cb_args
        .first()
        .map(|t| subject_package(ext, t))
        .unwrap_or_else(|| ext.package.clone());
    Some(IfaceSpec {
        typed_groups,
        ..IfaceSpec::assemble(package, name, vec![], params, kt::KtType::unit())
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
    let spec = registry.decon_plans.get(decon)?;
    let params = plan_leaf_params(ext, registry, &spec.leaves)?;
    let name = format!(
        "{}Builder",
        decon_base_name(&subject_short(&spec.source), Some(decon))
    );
    let package = subject_package(ext, &spec.source);
    Some(IfaceSpec::assemble(
        package,
        name,
        vec!["out R".to_string()],
        params,
        kt::KtType::var_r(),
    ))
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
    let spec = registry.decon_plans.get(decon)?;
    let mut params: Vec<IfaceParam> =
        vec![IfaceParam::same("acc".to_string(), kt::KtType::var_("A"))];
    params.extend(plan_leaf_params(ext, registry, &spec.leaves)?);
    let name = format!(
        "{}Folder",
        decon_base_name(&subject_short(&spec.source), Some(decon))
    );
    let package = subject_package(ext, &spec.source);
    Some(IfaceSpec::assemble(
        package,
        name,
        vec!["A".to_string()],
        params,
        kt::KtType::var_("A"),
    ))
}

/// Interface for a **whole-element fold** (`Iterable` delivery of a type
/// without a deconstructor — no declaration involved):
/// `run(acc: A, element): A`. One shape per element type by construction.
pub(crate) fn whole_folder_iface_spec(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    element: &syn::Type,
) -> Option<IfaceSpec> {
    let mut params: Vec<IfaceParam> =
        vec![IfaceParam::same("acc".to_string(), kt::KtType::var_("A"))];
    // `raw_handle = true`: an opaque-handle element crosses the JNI border as
    // its raw `jlong` (the folder wraps it into the typed handle class in Kotlin
    // bytecode — a native `new_object` per element would cost descriptor parse +
    // FindClass + GetMethodID + NewObjectA). Value blobs / String are unaffected
    // (they ignore the flag; see [`leaf_iface_param`]).
    params.push(leaf_iface_param(
        ext,
        registry,
        "element".to_string(),
        element,
        false,
        true,
    )?);
    let name = format!("{}Folder", subject_short(element));
    let package = subject_package(ext, element);
    Some(IfaceSpec::assemble(
        package,
        name,
        vec!["A".to_string()],
        params,
        kt::KtType::var_("A"),
    ))
}

/// The folder spec for an `Iterable` plan: declaration-keyed when the
/// element decomposes, whole-element otherwise. Thin KEY dispatch into the
/// [`JniGen::iface_spec`] memo — the fixed-builder typed-group view is
/// applied there per `DeconId` (the declaration identity the JVM resolves
/// against), not per this plan's own `fixed_builder` flag.
pub(crate) fn folder_iface_for_plan(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    plan: &UnfoldPlan,
) -> Option<std::sync::Arc<IfaceSpec>> {
    debug_assert!(
        plan.shape.has_iterable_layer(),
        "folder_iface_for_plan requires an Iterable (or Option<Iterable>) plan"
    );
    match (&plan.element, &plan.decon) {
        (Some(el), _) => ext.iface_spec(registry, &SpecKey::whole_folder(el)),
        (None, Some(d)) => ext.iface_spec(registry, &SpecKey::Folder(d.clone())),
        (None, None) => None,
    }
}

/// The typed-view groups for a **fixed-builder** decomposed fold (synthesized
/// by-value `data_class` element). `folder_iface_spec` lays its `run` params out
/// as `[acc: A, leaf0, …]`; this groups them so the user-facing folder receives
/// `(acc, element)` while the JNI-called raw twin keeps the decoupled leaves and
/// the `asRaw` proxy reassembles the whole value via `Class.fromParts(leaves…)`.
/// So no Java object is built on the Rust side — only raw leaves cross. Applied
/// in both `folder_iface_for_plan` (the wrapper's view) and the interface
/// emission (`write_iface_files`), keyed by the element's deconstructor so the
/// two stay in lockstep.
pub(crate) fn fixed_folder_typed_groups(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    decon: &DeconId,
) -> Option<Vec<TypedGroup>> {
    let spec = registry.decon_plans.get(decon)?;
    let fqn = ext.kotlin_fqn(&TypeKey::from_type(&spec.source).to_string())?;
    let class_short = fqn.rsplit('.').next().unwrap_or(&fqn).to_string();
    Some(vec![
        TypedGroup {
            name: "acc".to_string(),
            typed: kt::KtType::var_("A"),
            reassemble: None,
            leaf_count: 1,
        },
        TypedGroup {
            name: "element".to_string(),
            typed: kt::KtType::cls(fqn.to_string()),
            reassemble: Some(class_short),
            leaf_count: spec.leaves.len(),
        },
    ])
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
    ext: &JniGen,
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
    let source_short = subject_short(&spec.source);
    let mut iface = IfaceSpec::assemble(
        package,
        name,
        vec!["out R".to_string()],
        params,
        kt::KtType::var_r(),
    );
    iface.kdoc = Some(format!(
        "Error callback. Contract: `je != null` ⇒ a binding/system-tier failure — `je` is\n\
         its message and the remaining parameters carry defaults; `je == null` ⇒ a domain\n\
         error — the remaining parameters carry the decomposed `{source_short}`. The\n\
         wrapper returns whatever `run` returns; throwing from `run` is safe (it executes\n\
         after the native call has returned)."
    ));
    Some(iface)
}

/// The shared infallible handler `JniErrorHandler<out R> { run(je: String?): R }`
/// — every function without an error plan takes one; placed in the root
/// package.
pub(crate) fn jni_error_handler_iface_spec(ext: &JniGen) -> IfaceSpec {
    let params = vec![IfaceParam::same(
        "je".to_string(),
        kt::KtType::string().nullable(),
    )];
    let mut iface = IfaceSpec::assemble(
        ext.package.clone(),
        "JniErrorHandler".to_string(),
        vec!["out R".to_string()],
        params,
        kt::KtType::var_r(),
    );
    iface.kdoc = Some(
        "Error callback for wrappers without a declared error type. `je` is the\n\
         binding/system failure message (any converter in the chain may fail). The\n\
         wrapper returns whatever `run` returns; throwing from `run` is safe (it\n\
         executes after the native call has returned)."
            .to_string(),
    );
    iface
}

/// The onError handler spec for a declared function: its error plan's
/// declaration-keyed typed handler, or the shared global
/// `JniErrorHandler`. Thin KEY dispatch into the [`JniGen::iface_spec`]
/// memo.
pub(crate) fn onerror_iface_spec(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    fn_ident: &syn::Ident,
) -> Option<std::sync::Arc<IfaceSpec>> {
    let key = match registry.error_plans.get(fn_ident) {
        Some(plan) => SpecKey::Handler(
            plan.decon
                .clone()
                .expect("error plans are always record-built (decon is Some)"),
        ),
        None => SpecKey::JniErrorHandler,
    };
    ext.iface_spec(registry, &key)
}
