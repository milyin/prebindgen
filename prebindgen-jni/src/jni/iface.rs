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
//! classes, primitives unboxed, nullable primitives boxed. The native side calls
//! `run` with raw typed `jvalue`s: no per-leaf boxing upcalls, no erased
//! `FunctionN`.
//!
//! Each identity's spec is derived ONCE, through the [`Declarations::iface_spec`]
//! memo keyed by [`SpecKey`], and shared by all three sites — the
//! FQN/descriptor pair cannot drift between the artifact tiers (issue #107).
//! The constructors stay deterministic over `(ext, registry)`; in debug
//! builds every memo hit re-derives and asserts equality, so the
//! determinism is a checked invariant rather than a convention.

use kotlin_codegen::{KtCode, KtDecl, KtFun, KtFunInterface, KtFunSig, KtParam, KtType, KtVis};
use prebindgen_registry::{
    unfold::{dedup_names, DeconId, LeafSource, UnfoldPlan},
    Conversions,
};

use super::*;

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
    /// Rust `u64`: raw `Long` bit pattern → typed Kotlin `ULong`. A bounded
    /// optional representation carries its `None` niche as a primitive value.
    Unsigned64 { niche_sentinel: Option<String> },
}

impl WrapKind {
    /// The Kotlin FQN the wrap constructs, if any (for import registration).
    pub fn class_fqn(&self) -> Option<&str> {
        match self {
            WrapKind::None | WrapKind::Unsigned64 { .. } => None,
            WrapKind::Handle(f) | WrapKind::HandleOwned(f) => Some(f),
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
        if let WrapKind::Unsigned64 { niche_sentinel } = self {
            return match (raw_nullable, niche_sentinel) {
                (true, Some(sentinel)) => {
                    format!("{arg}?.let {{ if (it == {sentinel}) null else it.toULong() }}")
                }
                (true, None) => format!("{arg}?.toULong()"),
                (false, Some(sentinel)) => {
                    format!("if ({arg} == {sentinel}) null else {arg}.toULong()")
                }
                (false, None) => format!("{arg}.toULong()"),
            };
        }
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
    /// User-facing type (typed handle class, …).
    pub typed: KtType,
    /// JNI-called raw-twin type (`Long`, `ByteArray`, …) — what the
    /// descriptor and the native jvalues match.
    pub raw: KtType,
    /// How the proxy wraps raw → typed.
    pub wrap: WrapKind,
}

impl IfaceParam {
    fn same(name: String, ty: KtType) -> Self {
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
    pub typed: KtType,
    /// `Some(expr)` ⇒ this group's raw leaves reassemble into ONE typed value
    /// through `expr` — `Class.fromParts($0, $1, …)` for a fixed product, a
    /// `when` over the tag for a sum. `None` ⇒ a single passthrough leaf (the
    /// typed value IS the one raw param, optionally wrapped by its
    /// [`IfaceParam::wrap`]).
    ///
    /// Written with **positional placeholders** `$0`, `$1`, … over this group's
    /// leaves rather than with their names: the signature-wide
    /// [`dedup_names`] pass runs after a group is described, so a name captured
    /// here could be stale by the time [`IfaceSpec::to_as_raw_fun`] renders it.
    pub reassemble: Option<String>,
    /// FQNs the reassembly expression names in raw text (variant classes,
    /// enums) and therefore needs imported where it is rendered.
    pub imports: Vec<String>,
    /// Number of consecutive `params` (raw leaves) this group consumes.
    pub leaf_count: usize,
    /// How the `asRaw` proxy closes this value after `run`, or `None` when it
    /// reaches nothing owned — [`type_close_strategy`](crate::jni::struct_plan::type_close_strategy)'s
    /// answer, carried rather than collapsed to a bool, so the proxy renders it
    /// through the same [`render_handle_close`] the sealed classes use instead
    /// of hand-rolling `x.close()` and assuming the answer is `Base`.
    ///
    /// Closing at all is the same close-unless-taken contract
    /// [`WrapKind::HandleOwned`] gives a plan-less handle arg. Without it a
    /// handle reached through a sum or a nested data class was nobody's to
    /// close, while the same handle passed directly was the proxy's, so the
    /// owner changed with the argument's spelling (#218).
    pub close: Option<FoldStrategy>,
}

/// One `val` the `asRaw` proxy binds before calling `run`, and how it is
/// released afterwards. `close: None` is a value bound only to keep a raw
/// reassembly expression off the argument list.
struct Bind {
    line: String,
    close: Option<String>,
}

/// Render the bindings and the `run(…)` call, nesting a `try`/`finally` per
/// closeable one so everything bound after it happens **inside** its `try`.
///
/// Reverse order comes out of the nesting for free: the innermost `finally`
/// runs first, which is the order the locals were taken in, reversed.
fn nest_binds(mut code: KtCode, binds: &[Bind], call_args: &str) -> KtCode {
    let Some((first, rest)) = binds.split_first() else {
        return code.wline(format!("run({call_args})"));
    };
    // `line`, not `wline`: the bound value is a reassembly expression rendered
    // as one piece of raw text (a `when` over the tag), and width-breaking it
    // splits the arms mid-expression into something valid but unreadable.
    code = code.line(first.line.clone());
    match &first.close {
        Some(close) => code.try_finally(
            "",
            nest_binds(KtCode::new(), rest, call_args),
            KtCode::new().line(close.clone()),
        ),
        None => nest_binds(code, rest, call_args),
    }
}

/// A close strategy for a receiver whose Kotlin type is nullable for a reason
/// the *type* does not name: a sum under a conditional value form reconstructs
/// to `null` where the form was absent, so its selector — not its type — is
/// what makes the local `T?`. `Optional` already renders as `?.`, so a strategy
/// that carries one needs nothing added.
fn nullable_close(strategy: FoldStrategy, nullable: bool) -> FoldStrategy {
    match strategy {
        FoldStrategy::Optional(..) => strategy,
        inner if nullable => FoldStrategy::Optional(NullableKind::Boxed, Box::new(inner)),
        inner => inner,
    }
}

/// Substitute a [`TypedGroup::reassemble`] expression's `$k` placeholders with
/// the group's actual parameter names. Scans left to right taking the longest
/// digit run, so `$10` never resolves as `$1` followed by `0`.
fn fill_placeholders(expr: &str, names: &[&str]) -> String {
    let mut out = String::with_capacity(expr.len());
    let mut rest = expr;
    while let Some(pos) = rest.find('$') {
        out.push_str(&rest[..pos]);
        let digits: String = rest[pos + 1..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if digits.is_empty() {
            out.push('$');
            rest = &rest[pos + 1..];
            continue;
        }
        let idx: usize = digits.parse().expect("digit run parses");
        out.push_str(names.get(idx).copied().unwrap_or_else(|| {
            panic!(
                "reassembly placeholder ${idx} has no leaf in a {}-leaf group",
                names.len()
            )
        }));
        rest = &rest[pos + 1 + digits.len()..];
    }
    out.push_str(rest);
    out
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
    pub ret: KtType,
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
        ret: KtType,
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

    /// A [`KtType`] reference to this interface, instantiated with
    /// `args` (empty for a non-generic interface).
    pub fn kt_ref(&self, args: Vec<KtType>) -> KtType {
        if args.is_empty() {
            KtType::cls(self.fqn())
        } else {
            KtType::generic(self.fqn(), args)
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
    /// into the handler arguments *before* any re-entrant call. Slots are
    /// uniform — one nullable `ze{i}` per interface param, no special leading
    /// `je` — so the binding `JniErrorHandler` (single `je` param → `ze0`) and a
    /// typed domain `<Src>Handler` (the decomposed leaves) share one shape.
    pub fn to_capture_decl(&self) -> KtDecl {
        let cap = self.capture_name();
        let raw = self.raw_name();
        let n_ze = self.params.len();

        let mut fields = KtCode::new().line("@JvmField var failed: Boolean = false");
        for (i, p) in self.params.iter().enumerate() {
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
        let mut run_body = String::from("failed = true");
        for (i, p) in self.params.iter().enumerate() {
            run_body.push_str(&format!("; this.ze{i} = {}", p.name));
        }

        let mut reset = String::from("c.failed = false");
        for i in 0..n_ze {
            reset.push_str(&format!("; c.ze{i} = null"));
        }

        let code = KtCode::new().blk(format!("internal class {cap} : {raw}<Unit> {{"), |c| {
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
        KtDecl::Raw { name: cap, code }
    }

    /// The typed (user-facing) Kotlin declaration. With [`Self::typed_groups`]
    /// the `run` parameters are the groups (each a whole reassembled value);
    /// otherwise they are the `params` 1:1 (typed view).
    pub fn to_decl(&self) -> KtFunInterface {
        // A `KtFunSig`, not a `KtFun`: a SAM interface's one method is abstract
        // by definition, and a signature has nowhere to put a body.
        let mut m = KtFunSig::new(IFACE_METHOD).vis(KtVis::Public);
        if self.typed_groups.is_empty() {
            for p in &self.params {
                m = m.param(KtParam::new(&p.name, p.typed.clone()));
            }
        } else {
            for g in &self.typed_groups {
                m = m.param(KtParam::new(&g.name, g.typed.clone()));
            }
        }
        m = m.returns(self.ret.clone());
        let mut i = KtFunInterface::new(&self.name, m).vis(KtVis::Public);
        if let Some(doc) = &self.kdoc {
            i = i.kdoc(doc.clone());
        }
        for tp in &self.type_params {
            i = i.type_param(tp);
        }
        i
    }

    /// The raw-twin declaration (call only when [`Self::needs_raw`]).
    pub fn to_raw_decl(&self) -> KtFunInterface {
        let mut m = KtFunSig::new(IFACE_METHOD).vis(KtVis::Public);
        for p in &self.params {
            m = m.param(KtParam::new(&p.name, p.raw.clone()));
        }
        m = m.returns(self.ret.clone());
        let mut i = KtFunInterface::new(self.raw_name(), m).vis(KtVis::Public);
        for tp in &self.type_params {
            i = i.type_param(tp);
        }
        i
    }

    /// The generated proxy: `fun <G> <Name><G>.asRaw(): <Name>Raw<G> =
    /// <Name>Raw { raw leaves… -> run(<wraps…>) }` — constructed once per
    /// registration; per message it performs exactly the typed-object
    /// constructions the consumer needs anyway, in JVM bytecode.
    pub fn to_as_raw_fun(&self) -> KtFun {
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
        // Type arguments as a structured list, NOT baked into the name: a
        // `KtType::Named` whose `fqn` reads `Foo<R>` while its `args` are empty
        // is a lie about the type, and anything that inspects `args` (JVM
        // erasure, descriptors) reads it wrong. `generic` with an empty list
        // renders exactly like `cls`, so the non-generic case needs no branch.
        let type_args: Vec<KtType> = bare_generics
            .iter()
            .map(|g| KtType::var_(g.as_str()))
            .collect();
        // The extension receiver rides `KtFun::receiver`, not the name, so
        // `asRaw` stays a plain identifier the Kotlin checker can accept.
        let recv = KtType::generic(&self.name, type_args.clone());
        // `run` arguments: per-param `wrap` (1:1 view) or, with `typed_groups`,
        // one expression per group — a whole value reassembled from its leaves
        // via `Class.fromParts(leaves…)`, or a single passthrough leaf's wrap.
        // An owned-handle arg (`HandleOwned`) must be bound to a local and
        // `close()`-d after `run` (close-unless-taken), so track it per arg.
        struct RunArg {
            expr: String,
            /// How to close it after `run`, or `None` when it is not ours.
            close: Option<FoldStrategy>,
            /// Whether the Kotlin receiver is nullable — which for a
            /// reassembled sum is a fact about its SELECTOR, not about its
            /// type, so `close` cannot be asked for it. See [`nullable_close`].
            nullable: bool,
            /// A whole value rebuilt from its leaves (`Class.fromParts(…)`, a
            /// `when` over a tag) rather than one wrapped leaf. Rendered as one
            /// piece of raw text, so once anything binds a local — see
            /// `any_owned` below — these bind too: width-breaking a `when`
            /// inside a nested call splits its arms into something valid and
            /// unreadable, and a `val` keeps each on its own line.
            reassembled: bool,
        }
        let run_args: Vec<RunArg> = if self.typed_groups.is_empty() {
            self.params
                .iter()
                .map(|p| RunArg {
                    expr: p.wrap.wrap_expr(&p.name, p.raw.is_nullable()),
                    // A wrapped handle leaf closes as itself; its nullability
                    // rides `nullable` below, exactly as for a group.
                    close: p.wrap.is_owned_handle().then_some(FoldStrategy::Base),
                    nullable: p.raw.is_nullable(),
                    reassembled: false,
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
                    Some(expr) => args.push(RunArg {
                        expr: fill_placeholders(expr, &names),
                        close: g.close.clone(),
                        // The reassembled value is closed through its own
                        // generated `close()`, which is declared on a
                        // non-nullable receiver; a group whose typed view is
                        // nullable needs the `?.` guard.
                        nullable: g.typed.is_nullable(),
                        reassembled: true,
                    }),
                    None => {
                        let p = &self.params[at];
                        args.push(RunArg {
                            expr: p.wrap.wrap_expr(&p.name, p.raw.is_nullable()),
                            close: p.wrap.is_owned_handle().then_some(FoldStrategy::Base),
                            nullable: p.raw.is_nullable(),
                            reassembled: false,
                        });
                    }
                }
                at += g.leaf_count;
            }
            args
        };
        let any_owned = run_args.iter().any(|a| a.close.is_some());
        let lambda_open = format!("{}{gen_args} {{", self.raw_name());
        let body = if self.params.is_empty() {
            KtCode::new().blk(lambda_open, |c| c.line("run()"))
        } else if !any_owned {
            // Common case: a single `run(<wrapped…>)` call (no transient handle).
            let wrapped: Vec<&str> = run_args.iter().map(|a| a.expr.as_str()).collect();
            KtCode::new().blk(lambda_open, |mut c| {
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
            //
            // Each owned local opens its OWN try/finally and everything after
            // it is bound INSIDE that try: with one flat `try` around `run`
            // only, a later reassembly that throws (a `when` over an invalid
            // tag) would strand every handle already taken by an earlier one.
            let mut binds: Vec<Bind> = Vec::new();
            let mut call_args: Vec<String> = Vec::with_capacity(run_args.len());
            let mut owned_idx = 0usize;
            let mut val_idx = 0usize;
            for a in &run_args {
                if let Some(strategy) = &a.close {
                    let local = format!("__own{owned_idx}");
                    owned_idx += 1;
                    call_args.push(local.clone());
                    binds.push(Bind {
                        line: format!("val {local} = {}", a.expr),
                        close: Some(render_handle_close(
                            &nullable_close(strategy.clone(), a.nullable),
                            &local,
                        )),
                    });
                } else if a.reassembled {
                    // Not ours to close, but still one piece of raw text that
                    // must not be broken up inside the nested `run(…)`.
                    let local = format!("__val{val_idx}");
                    val_idx += 1;
                    call_args.push(local.clone());
                    binds.push(Bind {
                        line: format!("val {local} = {}", a.expr),
                        close: None,
                    });
                } else {
                    call_args.push(a.expr.clone());
                }
            }
            KtCode::new().blk(lambda_open, |mut c| {
                // Lambda parameters one per line, exactly as the common path
                // above lays them out — the `finally` is the only difference
                // between the two shapes, and the parameter list should not
                // move just because a handle appeared among the arguments.
                for (idx, name) in self.params.iter().map(|p| p.name.as_str()).enumerate() {
                    let suffix = if idx + 1 == self.params.len() {
                        " ->"
                    } else {
                        ","
                    };
                    c = c.line(format!("{name}{suffix}"));
                }
                nest_binds(c, &binds, &call_args.join(", "))
            })
        };
        let mut f = KtFun::new("asRaw").vis(KtVis::Public).receiver(recv);
        for g in &bare_generics {
            f = f.generic(g);
        }
        f = f.returns(KtType::generic(self.raw_name(), type_args));
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
fn kt_jvm_descriptor(ty: &KtType, type_params: &[String]) -> String {
    let KtType::Named {
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
        if let Some(d) = crate::jni::wire_access::kotlin_array_descriptor(simple) {
            return d.to_string();
        }
        return match simple {
            "Unit" => "V".to_string(),
            "String" => "Ljava/lang/String;".to_string(),
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

fn method_descr(params: &[IfaceParam], ret: &KtType, type_params: &[String]) -> String {
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
fn subject_short(ty: &prebindgen_registry::flat::TypeRef) -> String {
    use crate::util::{head_name, head_type};
    // One `&` off, then a slice — the `List` suffix is decided before the
    // general peel, exactly as it was, because `&[E]` and `Vec<E>` must not
    // collapse to the same interface name.
    let no_ref = ty.borrow_target().unwrap_or(ty);
    if let prebindgen_registry::flat::TypeKind::Slice(elem) = no_ref.kind() {
        return format!("{}List", subject_short(elem));
    }
    let peeled = head_type(ty);
    head_name(peeled).unwrap_or_else(|| peeled.key().to_string().replace([' ', ':', '<', '>'], ""))
}

/// Package a subject type's interface lives in: the package of the type's
/// registered Kotlin FQN, the root `ext.package` otherwise.
fn subject_package(ext: &Declarations, subject: &prebindgen_registry::flat::TypeRef) -> String {
    let key = crate::util::head_type(subject).key();
    ext.kotlin_fqn(&key)
        .and_then(|fqn| fqn.rsplit_once('.').map(|(p, _)| p.to_string()))
        .unwrap_or_else(|| ext.package.clone())
}

/// The interface param list for a decomposition's leaves: names from
/// [`plan_leaf_names`], typed + raw views per leaf.
fn plan_leaf_params(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    leaves: &[prebindgen_registry::unfold::UnfoldLeaf],
) -> Option<Vec<IfaceParam>> {
    // Decomposition leaf names are author-supplied, literal, and unique by
    // construction (enforced in `core::unfold`) — no dedup/casing here.
    let names = plan_leaf_names(leaves);
    let mut out = Vec::with_capacity(leaves.len());
    for (name, leaf) in names.into_iter().zip(leaves.iter()) {
        out.push(plan_leaf_param(ext, registry, name, leaf)?);
    }
    Some(out)
}

/// Both interface views of ONE decomposition leaf. The single entry point for a
/// plan leaf, so the builder/folder/handler signatures and a callback's
/// flattened `run` params classify it identically — the tag slot and the
/// inert-group nullability rule below have to hold at every one of those sites,
/// not just where the plan happens to be walked as a whole.
fn plan_leaf_param(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    name: String,
    leaf: &prebindgen_registry::unfold::UnfoldLeaf,
) -> Option<IfaceParam> {
    use prebindgen_registry::unfold::LeafSource;
    // The sum selector has no converter behind it — it is a plain `Int` the
    // emitter assigns per `match` arm. Nullable when the sum sits under a
    // conditional value form: null is the absent case, which the tag's own
    // variants cannot express.
    if leaf.source == LeafSource::SumTag {
        let ty = KtType::int();
        let ty = if leaf.nullable { ty.nullable() } else { ty };
        return Some(IfaceParam::same(name, ty));
    }
    // A group slot is INERT whenever another variant is live, and the encoder
    // fills an inert object slot with `JObject::null()`. A JVM null handed to a
    // non-null Kotlin parameter throws inside the intrinsic null-check before
    // any generated code runs, so every object-shaped group slot is nullable
    // here and re-asserted (`!!`) inside its own live arm — the same rule
    // `nullable_group_part` applies to the parent-inlined `fromParts`. Primitive
    // slots take their `0`/`false` default and stay unboxed.
    let inert_nullable = leaf.group.is_some() && !leaf_ty_is_prim(registry, &leaf.out_ty);
    leaf_iface_param(
        ext,
        registry,
        name,
        &leaf.out_ty,
        leaf.nullable || inert_nullable,
        true,
    )
}

/// Both interface views of one delivered leaf.
///
/// * **typed** (user-facing, Kotlin-called): handles as their typed handle
///   classes — legal here because the JNI border never touches this method.
/// * **raw** (JNI-called twin): a PLAN leaf (`raw_handle`) crosses handles
///   as the raw `jlong` (`Long`/boxed `Long?` — the proxy constructs the
///   class in bytecode; a native `new_object` would cost descriptor parse +
///   FindClass + GetMethodID + NewObjectA per message). A whole
///   (plan-less callback) arg keeps the typed handle class in BOTH views —
///   the close-unless-taken contract needs the native side to `close()` the
///   wrapped object after the invoke.
fn leaf_iface_param(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    name: String,
    out_ty: &prebindgen_registry::flat::TypeRef,
    nullable: bool,
    raw_handle: bool,
) -> Option<IfaceParam> {
    // The declaration-canonical spec normalizes its identity leaf to the
    // borrowed `&T` form; a function set that only ever returns the type
    // OWNED resolved only `T`'s output entry. Both classify to the same
    // projection/class, so fall back to the peeled form when the borrowed
    // entry was never required.
    //
    // `TypeKind::Ref` and NOT `borrow_target`: this peels the spelling's own
    // outermost borrow, where `borrow_target` reaches the target *through* the
    // erased wrappers. `Box<&T>` has a borrow target and is deliberately not
    // servable here (`a_wrapped_borrow_callback_arg_declines`) — peeling it
    // would resolve a shape the wrapper arms own.
    let mut out_ty = out_ty;
    if registry.output_entry(out_ty).is_none() {
        if let prebindgen_registry::flat::TypeKind::Ref { inner, .. } = out_ty.kind() {
            out_ty = inner;
        }
    }
    let (builder_kt, _wire_kt, _wrap, is_value_projection) =
        unfold_leaf_kt(ext, registry, out_ty, nullable, "x")?;
    let proj = registry
        .output_entry(out_ty)
        .and_then(|e| e.metadata.projection.as_ref());
    let nullable_kt = |t: KtType| {
        if builder_kt.is_nullable() {
            t.nullable()
        } else {
            t
        }
    };
    if is_value_projection {
        match proj?.kind {
            ProjectionKind::Unsigned64 => {
                let mut raw = projection_wire_return(proj?);
                if nullable && !raw.is_nullable() {
                    raw = raw.nullable();
                }
                // The niche sentinel IS the absent representation, so it means
                // something only where the leaf can be absent. A non-optional
                // leaf (a required sum payload, say) owns the whole domain:
                // mapping its sentinel to null would both invent an absence the
                // type does not have and contradict the non-nullable typed view
                // this same param declares.
                let niche_sentinel = if builder_kt.is_nullable() {
                    projection_leaf_sentinel(proj?)
                } else {
                    None
                };
                return Some(IfaceParam {
                    name,
                    typed: builder_kt.clone(),
                    raw,
                    wrap: WrapKind::Unsigned64 { niche_sentinel },
                });
            }
            ProjectionKind::Handle => {}
        }
    }
    if let Some(p) = proj.filter(|p| p.kind == ProjectionKind::Handle) {
        let fqn = ext.kotlin_fqn(&p.leaf_key)?.to_string();
        if raw_handle {
            return Some(IfaceParam {
                name,
                typed: nullable_kt(KtType::cls(fqn.clone())),
                raw: nullable_kt(KtType::long()),
                wrap: WrapKind::Handle(fqn),
            });
        }
        // Whole arg: typed class in both views (no proxy wrap).
        return Some(IfaceParam::same(name, nullable_kt(KtType::cls(fqn))));
    }
    // A whole generated class (e.g. a field-based `data_class` delivered to a
    // callback by `impl Fn(&T)`): `builder_kt` carries the unqualified short name
    // (the wrapper relies on an import), but the `raw` view — which drives the JNI
    // method descriptor — must be the fully-qualified class so `get_method_id("run",
    // …)` resolves. Apply only when `builder_kt` actually IS that class (its short
    // name matches the registered FQN); never for enums (→ `Int`) or builtins. `wrap`
    // stays `None` (typed and raw are the same JVM type, just spelled differently), so
    // no `asRaw` proxy is generated.
    if let KtType::Named { fqn: bk_fqn, .. } = &builder_kt {
        if !bk_fqn.contains('.') {
            if let Some(reg_fqn) = ext.kotlin_fqn(&out_ty.key()) {
                let reg_short = reg_fqn.rsplit('.').next().unwrap_or(&reg_fqn);
                if reg_fqn.contains('.') && reg_short == bk_fqn {
                    let raw = KtType::cls(reg_fqn.to_string());
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
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    name: String,
    out_ty: &prebindgen_registry::flat::TypeRef,
    nullable: bool,
) -> Option<IfaceParam> {
    let proj = registry.output_entry(out_ty)?.metadata.projection.clone()?;
    let fqn = ext.kotlin_fqn(&proj.leaf_key)?.to_string();
    let typed = KtType::cls(fqn.clone());
    let (typed, raw) = if nullable {
        (typed.nullable(), KtType::long().nullable())
    } else {
        (typed, KtType::long())
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
/// store the [`TypeKey`] itself — the derivation reads the key's stored
/// normalized type, so a key alone fully determines its spec with no
/// string round trip.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SpecKey {
    /// impl-Fn delivery — the args' canonical type keys (each arg either
    /// decomposes via its type's canonical plan or crosses whole).
    Callback(Vec<TypeKey>),
    Builder(DeconId),
    Folder(DeconId),
    /// Whole-element fold — no declaration; keyed by element type.
    WholeFolder(TypeKey),
    Handler(DeconId),
    JniErrorHandler,
}

impl SpecKey {
    /// The impl-Fn identity for a callback's arg types.
    ///
    /// Off the readings, like its [`whole_folder`](Self::whole_folder) peer:
    /// the key IS a `Vec<TypeKey>`, and every caller was spelling each arg into
    /// a throwaway `Vec<syn::Type>` for `TypeKey::from_type` to read back.
    pub fn callback(args: &[prebindgen_registry::flat::TypeRef]) -> Self {
        SpecKey::Callback(args.iter().map(|a| a.key()).collect())
    }

    /// The whole-element fold identity for an element type.
    pub fn whole_folder(element: &prebindgen_registry::flat::TypeRef) -> Self {
        SpecKey::WholeFolder(element.key())
    }
}

/// DeconIds whose builder/folder is a synthesized by-value `data_class`
/// (`fixed_builder` plans). Fixedness is a property of the DECLARATION
/// identity — the JVM resolves calls against the single declared interface —
/// so it is computed per `DeconId` over all plans, shared by the memo
/// derivation ([`SpecKey::Folder`]'s typed groups) and the declaration
/// emitter (the hoisted `fromParts`/appender singletons).
pub(crate) fn fixed_decon_ids(
    registry: &impl Conversions<KotlinMeta>,
) -> std::collections::HashSet<DeconId> {
    let fixed: std::collections::HashSet<DeconId> = registry
        .unfold_plans()
        .values()
        .chain(registry.callback_arg_plans().values())
        .filter(|p| p.fixed_builder)
        .filter_map(|p| p.decon.clone())
        .collect();
    // Fixedness must be UNIFORM per identity, and two consumers now depend on
    // that: the memo swaps in the fixed typed-group view for the whole
    // identity, and the emitter suppresses the typed interface entirely when
    // the singleton is its only implementation. A mixed identity would give a
    // non-fixed wrapper a `build`/`fold` param whose interface was shaped —
    // or removed — for the fixed case.
    //
    // It holds by construction: `apply_value_structs` / `apply_sum_returns` run
    // AFTER `unfold::apply` and wire every reachable position of a type at
    // once, so a `DeconId::Default` is all-fixed or all-not; a `DeconId::PerFn`
    // is unique to one function and so trivially uniform. Asserted rather than
    // assumed, so a future wiring change fails here instead of emitting a
    // wrapper that references an interface which no longer exists.
    debug_assert!(
        !registry
            .unfold_plans()
            .values()
            .chain(registry.callback_arg_plans().values())
            .any(|p| !p.fixed_builder && p.decon.as_ref().is_some_and(|d| fixed.contains(d))),
        "fixed and non-fixed plans share one DeconId — the typed interface \
         cannot be shaped (or suppressed) for both"
    );
    fixed
}

/// Element type keys whose whole-element fold is fixed (a synthesized
/// single-leaf `Vec<T>` fold) — the leaf dual of [`fixed_decon_ids`], used
/// by the declaration emitter for the hoisted appender singleton.
pub(crate) fn fixed_leaf_element_keys(
    registry: &Registry<KotlinMeta>,
) -> std::collections::HashSet<TypeKey> {
    registry
        .unfold_plans()
        .values()
        .chain(registry.callback_arg_plans().values())
        .filter(|p| p.fixed_builder)
        .filter_map(|p| p.element.as_ref())
        .map(|el| el.key())
        .collect()
}

/// Derive the spec for one identity — the SINGLE construction point behind
/// [`Declarations::iface_spec`]. Any `syn` context is **looked up**, never
/// rebuilt from the key: `Registry::reading` answers from the type table, and a
/// key it cannot answer for defers rather than producing tokens nothing
/// classified (#291). A `Folder` derivation folds the fixed-builder
/// typed-group view in per `DeconId` (see [`fixed_decon_ids`]).
fn derive_iface_spec(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    key: &SpecKey,
) -> Option<IfaceSpec> {
    match key {
        SpecKey::Callback(arg_keys) => {
            // The one deliberate round trip in this file, and the memo forces
            // it: `SpecKey` needs `Ord`, so it holds `TypeKey`s — a `TypeRef`
            // could not go in one (its `Origin` carries a `SourceLocation`, so
            // two identical readings from different files would compare
            // unequal), and `derive_iface_spec` is contractually a pure
            // function of the key.
            //
            // A LOOKUP, not a classification: `Registry::reading` has answered
            // only from the type table since #267. `None` means the arg has
            // not entered the pipeline yet, which is the same "not yet
            // derivable" state `iface_spec` already documents and retries —
            // so it defers rather than answering "not optional", which is what
            // the accessor used to do here.
            let args: Vec<prebindgen_registry::flat::TypeRef> = arg_keys
                .iter()
                .map(|k| registry.reading(k))
                .collect::<Option<_>>()?;
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
        // Same round trip, same reason, same answer as the `Callback` arm
        // above: the memo key holds an identity, and the reading behind it is a
        // lookup. `None` defers, exactly as it does there (#291).
        SpecKey::WholeFolder(el_key) => {
            whole_folder_iface_spec(ext, registry, &registry.reading(el_key)?)
        }
        SpecKey::Handler(d) => error_handler_iface_spec(ext, registry, d),
        SpecKey::JniErrorHandler => Some(jni_error_handler_iface_spec(ext)),
    }
}

impl Declarations {
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
        registry: &impl Conversions<KotlinMeta>,
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

/// The [`TypedGroup::reassemble`] expression (and its raw-text imports) for a
/// **fixed-builder** decomposition's leaves — the one place the two shapes are
/// distinguished, so the callback proxy and the `Vec` folder cannot pick
/// different reassemblies for the same type.
///
/// A product reassembles through the class's own `fromParts`; a **sum** cannot
/// — its `fromParts` is the Kotlin-facing convenience whose parameters are
/// property types, not the wire — so it reassembles through the same inlined
/// `when` over the tag that a sum-typed struct field gets.
fn fixed_reassembly(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    source: &TypeKey,
    leaves: &[prebindgen_registry::unfold::UnfoldLeaf],
    class_fqn: &str,
) -> (String, Vec<String>) {
    let slots: Vec<String> = (0..leaves.len()).map(|i| format!("${i}")).collect();
    if !is_sum_leaves(leaves) {
        let class_short = class_fqn.rsplit('.').next().unwrap_or(class_fqn);
        return (
            format!("{class_short}.fromParts({})", slots.join(", ")),
            Vec::new(),
        );
    }
    let params = plan_leaf_params(ext, registry, leaves).unwrap_or_default();
    let mut imports: BTreeSet<String> = BTreeSet::new();
    let (_, when) = ext.sum_reconstruct(registry, source, leaves, &params, &slots, &mut imports);
    (when, imports.into_iter().collect())
}

/// Interface for an `impl Fn(args)` delivery: one `run` parameter per
/// flattened leaf of each arg's callback plan (the arg whole when plan-less),
/// returning `Unit`. Named `<ArgShorts>Callback` (`Fn()` → `VoidCallback`),
/// placed in the first arg type's package (root for `Fn()`).
pub(crate) fn callback_iface_spec(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    cb_args: &[prebindgen_registry::flat::TypeRef],
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
        typed: Option<KtType>,
        reassemble: Option<String>,
        imports: Vec<String>,
        leaf_count: usize,
        /// How the proxy closes the reassembled value after `run`, or `None`
        /// when it reaches nothing owned — the close-unless-taken contract a
        /// plan-less handle arg already had (#218). Always `None` for a
        /// passthrough group, whose own [`IfaceParam::wrap`] answers instead.
        close: Option<FoldStrategy>,
    }
    /// One flattened `run` parameter before naming/dedup. A **plan** leaf keeps
    /// the leaf itself, so it classifies through the shared
    /// [`plan_leaf_param`] (tag slot, inert-group nullability) rather than a
    /// second, weaker derivation here; a **whole** arg carries just its type,
    /// and `owned_handle` marks a plan-less opaque handle delivered as a raw
    /// `jlong` and wrapped + closed Kotlin-side (Phase 3).
    enum LeafDesc {
        Plan(String, prebindgen_registry::unfold::UnfoldLeaf),
        Whole {
            name: String,
            ty: prebindgen_registry::flat::TypeRef,
            nullable: bool,
            owned_handle: bool,
        },
    }
    impl LeafDesc {
        fn name(&self) -> &str {
            match self {
                LeafDesc::Plan(n, _) => n,
                LeafDesc::Whole { name, .. } => name,
            }
        }
    }
    let mut leaf_tys: Vec<LeafDesc> = Vec::new();
    let mut groups: Vec<GroupDesc> = Vec::new();
    let mut any_fixed = false;

    for (i, t) in cb_args.iter().enumerate() {
        // An `Iterable` fixed-folder plan (an `&[data_class]` arg) is delivered by
        // FOLDING in the trampoline — the user callback still sees ONE whole
        // `List<Element>`, so it takes the plain whole-value path below (no leaf
        // params, no reassembly group). Only `Base`/accessor plans decompose the
        // arg into the callback's `run` params here.
        let plan = registry
            .callback_arg_plans()
            .get(&t.key())
            .filter(|p| !super::render::is_iterable_fold(&p.shape));
        if let Some(plan) = plan {
            let leaf_names = plan_leaf_names(&plan.leaves);
            for (n, l) in leaf_names.iter().zip(plan.leaves.iter()) {
                leaf_tys.push(LeafDesc::Plan(n.clone(), l.clone()));
            }
            if plan.fixed_builder {
                any_fixed = true;
                // Peeled off the reading — `borrow_target` is the model's
                // answer to "is this a borrow", not a syn match.
                let core = t.borrow_target().unwrap_or(t);
                let fqn = ext.kotlin_fqn(&core.key())?;
                let (reassemble, imports) =
                    fixed_reassembly(ext, registry, &core.key(), &plan.leaves, &fqn);
                groups.push(GroupDesc {
                    name: whole_value_name(t, i),
                    typed: Some(KtType::cls(fqn.to_string())),
                    reassemble: Some(reassemble),
                    imports,
                    leaf_count: plan.leaves.len(),
                    close: crate::jni::struct_plan::type_close_strategy(ext, registry, core, 0),
                });
            } else {
                // Accessor-plan arg: each leaf is its own passthrough group, so
                // the user callback still sees the flattened leaves (unchanged)
                // — EXCEPT a sum segment, whose selector and group slots are one
                // value and collapse into a single typed parameter rebuilt by a
                // `when` over the tag. Handing those slots over raw would defeat
                // the `sealed_class!` the sum was declared as.
                let mut k = 0usize;
                while k < plan.leaves.len() {
                    let leaf = &plan.leaves[k];
                    let seg = if leaf.source == LeafSource::SumTag {
                        (k + 1..plan.leaves.len())
                            .take_while(|&j| plan.leaves[j].group.is_some())
                            .last()
                            .map_or(k + 1, |j| j + 1)
                    } else {
                        k + 1
                    };
                    if leaf.source == LeafSource::SumTag {
                        any_fixed = true;
                        let fqn = ext.kotlin_fqn(&leaf.out_ty.key())?;
                        let (reassemble, imports) = fixed_reassembly(
                            ext,
                            registry,
                            &leaf.out_ty.key(),
                            &plan.leaves[k..seg],
                            &fqn,
                        );
                        groups.push(GroupDesc {
                            // The tag leaf is named `<field>__tag`; the value it
                            // selects over is that field.
                            name: leaf_names[k]
                                .strip_suffix(&format!("__{SUM_TAG_LEAF}"))
                                .unwrap_or(&leaf_names[k])
                                .to_string(),
                            // Nullable exactly when its selector is: a sum under
                            // a conditional value form reconstructs to `null`
                            // where the form was absent.
                            typed: Some({
                                let t = KtType::cls(fqn.to_string());
                                if leaf.nullable {
                                    t.nullable()
                                } else {
                                    t
                                }
                            }),
                            reassemble: Some(reassemble),
                            imports,
                            leaf_count: seg - k,
                            close: crate::jni::struct_plan::type_close_strategy(
                                ext,
                                registry,
                                &leaf.out_ty,
                                0,
                            ),
                        });
                    } else {
                        groups.push(GroupDesc {
                            name: leaf_names[k].clone(),
                            typed: None,
                            reassemble: None,
                            imports: Vec::new(),
                            leaf_count: 1,
                            close: None,
                        });
                    }
                    k = seg;
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
            leaf_tys.push(LeafDesc::Whole {
                name: whole_value_name(t, i),
                ty: (*t).clone(),
                nullable: t.optional_inner().is_some(),
                owned_handle,
            });
            groups.push(GroupDesc {
                name: whole_value_name(t, i),
                typed: None,
                reassemble: None,
                imports: Vec::new(),
                leaf_count: 1,
                close: None,
            });
        }
    }
    let mut names: Vec<String> = leaf_tys.iter().map(|d| d.name().to_string()).collect();
    dedup_names(&mut names);
    let mut params = Vec::with_capacity(leaf_tys.len());
    for (k, desc) in leaf_tys.iter().enumerate() {
        let name = names[k].clone();
        let param = match desc {
            LeafDesc::Plan(_, leaf) => plan_leaf_param(ext, registry, name, leaf)?,
            LeafDesc::Whole {
                ty,
                nullable,
                owned_handle: true,
                ..
            } => owned_handle_iface_param(ext, registry, name, ty, *nullable)?,
            LeafDesc::Whole { ty, nullable, .. } => {
                leaf_iface_param(ext, registry, name, ty, *nullable, false)?
            }
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
                imports: g.imports.clone(),
                leaf_count: g.leaf_count,
                close: g.close.clone(),
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
        ..IfaceSpec::assemble(package, name, vec![], params, KtType::unit())
    })
}

/// Interface for an output-expansion **builder** (`Decompose`/`Optional`
/// callback delivery): `run(leaves…): R`, `<out R>`. Keyed by the
/// deconstructor declaration — the signature derives from the declaration's
/// representative plan in `registry.decon_plans`, never from a using
/// function's own plan. Named `<decl-base>Builder`, placed in the source
/// type's package.
pub(crate) fn builder_iface_spec(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    decon: &DeconId,
) -> Option<IfaceSpec> {
    let spec = registry.decon_plans().get(decon)?;
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
        KtType::var_r(),
    ))
}

/// Interface for a **decomposed-element fold** (`Iterable` delivery over a
/// type with a deconstructor): `run(acc: A, element-leaves…): A`, `<A>`
/// (invariant — `A` appears in both parameter and return position). Keyed by
/// the element's deconstructor declaration. Named `<decl-base>Folder`,
/// placed in the element type's package.
pub(crate) fn folder_iface_spec(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    decon: &DeconId,
) -> Option<IfaceSpec> {
    let spec = registry.decon_plans().get(decon)?;
    let mut params: Vec<IfaceParam> = vec![IfaceParam::same("acc".to_string(), KtType::var_("A"))];
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
        KtType::var_("A"),
    ))
}

/// Interface for a **whole-element fold** (`Iterable` delivery of a type
/// without a deconstructor — no declaration involved):
/// `run(acc: A, element): A`. One shape per element type by construction.
pub(crate) fn whole_folder_iface_spec(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    element: &prebindgen_registry::flat::TypeRef,
) -> Option<IfaceSpec> {
    let mut params: Vec<IfaceParam> = vec![IfaceParam::same("acc".to_string(), KtType::var_("A"))];
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
        KtType::var_("A"),
    ))
}

/// The folder spec for an `Iterable` plan: declaration-keyed when the
/// element decomposes, whole-element otherwise. Thin KEY dispatch into the
/// [`Declarations::iface_spec`] memo — the fixed-builder typed-group view is
/// applied there per `DeconId` (the declaration identity the JVM resolves
/// against), not per this plan's own `fixed_builder` flag.
pub(crate) fn folder_iface_for_plan(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
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
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    decon: &DeconId,
) -> Option<Vec<TypedGroup>> {
    let spec = registry.decon_plans().get(decon)?;
    let fqn = ext.kotlin_fqn(&spec.source.key())?;
    let (reassemble, imports) =
        fixed_reassembly(ext, registry, &spec.source.key(), &spec.leaves, &fqn);
    Some(vec![
        TypedGroup {
            name: "acc".to_string(),
            typed: KtType::var_("A"),
            reassemble: None,
            imports: Vec::new(),
            leaf_count: 1,
            // The accumulator is the consumer's own value, of the consumer's
            // own type variable — never ours to close.
            close: None,
        },
        TypedGroup {
            name: "element".to_string(),
            typed: KtType::cls(fqn.to_string()),
            reassemble: Some(reassemble),
            imports,
            leaf_count: spec.leaves.len(),
            // Same rule as every other reassembled value — and BREAKING for a
            // fold whose element reaches a handle: the element is now closed
            // after each `run`, so a body that accumulates its elements
            // (`{ acc, e -> acc + e }`) accumulates closed handles unless it
            // `take()`s each one. Note this is not a sum-only rule; a nested
            // `data_class` element lands here too.
            //
            // A fold over a BORROWED run never trips it: a borrowed handle
            // projection carries `owned: false`, so the element reaches nothing
            // to release and the per-iteration close that would double-free is
            // not emitted.
            close: crate::jni::struct_plan::type_close_strategy(ext, registry, &spec.source, 0),
        },
    ])
}

/// Interface for a fallible function's **domain** onError handler:
/// `run(ze-leaves…): R`, `<out R>`. The `ze` leaves are typed EXACTLY like a
/// builder's for the same decomposition — the domain-error channel IS the
/// output channel. This handler is called ONLY on a domain error (`Err(E)`);
/// binding/system failures go to the separate [`jni_error_handler_iface_spec`]
/// (`JniErrorHandler`) channel, so there is no discriminator and no defaults.
/// Keyed by the error type's deconstructor declaration. Named
/// `<decl-base>Handler`, placed in the error type's package.
pub(crate) fn error_handler_iface_spec(
    ext: &Declarations,
    registry: &impl Conversions<KotlinMeta>,
    decon: &DeconId,
) -> Option<IfaceSpec> {
    let spec = registry.decon_plans().get(decon)?;
    let params: Vec<IfaceParam> = plan_leaf_params(ext, registry, &spec.leaves)?;
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
        KtType::var_r(),
    );
    iface.kdoc = Some(format!(
        "Domain-error callback: called only when the native function returns `Err` — the\n\
         parameters carry the decomposed `{source_short}`. Binding/system failures go to the\n\
         separate `onBindingError` (`JniErrorHandler`) channel instead, so there is no `je`\n\
         discriminator here. The wrapper returns whatever `run` returns;\n\
         throwing from `run` is safe (it executes after the native call has returned)."
    ));
    Some(iface)
}

/// The shared infallible handler `JniErrorHandler<out R> { run(je: String?): R }`
/// — every function without an error plan takes one; placed in the root
/// package.
pub(crate) fn jni_error_handler_iface_spec(ext: &Declarations) -> IfaceSpec {
    let params = vec![IfaceParam::same(
        "je".to_string(),
        KtType::string().nullable(),
    )];
    let mut iface = IfaceSpec::assemble(
        ext.package.clone(),
        "JniErrorHandler".to_string(),
        vec!["out R".to_string()],
        params,
        KtType::var_r(),
    );
    iface.kdoc = Some(
        "Binding-error callback — every wrapper's binding/system failure channel (any\n\
         converter in the chain may fail, or a handle may be closed). `je` is the\n\
         failure message. For an infallible wrapper this is the sole `onError`; a\n\
         fallible wrapper takes it as `onBindingError` alongside the typed domain\n\
         handler. The wrapper returns whatever `run` returns;\n\
         throwing from `run` is safe (it executes after the native call has returned)."
            .to_string(),
    );
    iface
}

/// The two onError handler interfaces for a declared function. A function has
/// two independent error channels: a binding/system failure (any converter in
/// the chain, a closed handle, …) and — for a `Result<_, E>` function — a
/// domain error (`Err(E)`). Each is delivered to its own SAM handler, so
/// neither carries a discriminator or fabricated defaults.
pub(crate) struct ErrorIfaces {
    /// Binding/system-failure channel — always the base `JniErrorHandler`.
    pub binding: std::sync::Arc<IfaceSpec>,
    /// Domain-error channel — the typed `<Src>Handler` (no `je`); `Some` iff
    /// the function has a declared error plan (returns `Result<_, E>`).
    pub domain: Option<std::sync::Arc<IfaceSpec>>,
}

/// The onError handler interfaces for a declared function — the always-present
/// binding `JniErrorHandler` plus, for a fallible function, its
/// declaration-keyed typed domain `<Src>Handler`. Thin KEY dispatch into the
/// [`Declarations::iface_spec`] memo. `None` = the domain handler is underivable
/// (the Rust emitter panics, the Kotlin renderer skips) — the binding channel
/// alone always derives.
pub(crate) fn onerror_iface_spec(
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
    fn_ident: &syn::Ident,
) -> Option<ErrorIfaces> {
    let binding = ext.iface_spec(registry, &SpecKey::JniErrorHandler)?;
    let domain = match registry.error_plans().get(fn_ident) {
        Some(plan) => {
            let decon = plan
                .decon
                .clone()
                .expect("error plans are always record-built (decon is Some)");
            Some(ext.iface_spec(registry, &SpecKey::Handler(decon))?)
        }
        None => None,
    };
    Some(ErrorIfaces { binding, domain })
}
