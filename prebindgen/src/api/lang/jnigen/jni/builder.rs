//! Builder API for [`JniGen`].
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.

use super::*;

impl JniGen {
    /// Look up the registered Kotlin FQN for a canonical Rust type key
    /// (the inverse of the `(key, fqn)` rows pushed into
    /// [`Self::kotlin_type_fqns`] by the type-declaration builders). Single
    /// home for what used to be ~10 open-coded `kotlin_type_fqns.iter().find`
    /// scans across the emitters.
    pub(crate) fn kotlin_fqn(&self, rust_canon: &str) -> Option<&str> {
        self.kotlin_type_fqns
            .iter()
            .find(|(k, _)| k == rust_canon)
            .map(|(_, v)| v.as_str())
    }

    /// Convenience constructor with sensible defaults; the paths still need
    /// to be set explicitly via the field-mutation builder methods.
    pub fn new() -> Self {
        let base = Self {
            source_module: syn::parse_str("crate").unwrap(),
            package: String::new(),
            java_class_prefix: String::new(),
            jni_class_path: "Java_JNINative".to_string(),
            kotlin_fun_name_mangle: None,
            kotlin_ptr_class_name_mangle: None,
            kotlin_data_class_name_mangle: None,
            kotlin_enum_name_mangle: None,
            kotlin_wrapper_name_mangle: None,
            kotlin_harness_name_mangle: None,
            kotlin_type_fqns: Vec::new(),
            types: HashMap::new(),
            packages: BTreeMap::new(),
            input_wrappers: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            output_wrappers: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            last_opaque_key: None,
            last_meta_key: None,
            active_subpackage: None,
            last_entry_ref: None,
            emit_handle_locks: true,
            expansions: crate::api::core::expand::Expansions::default(),
            deconstructors: crate::api::core::unfold::Deconstructors::default(),
        };
        // Built-in rank-2 `Result<_, _>` peel: every Result<T, E> succeeds
        // as T and routes E to the error-sink on Err. The error type `E` is
        // carried through the middle slot so the converter signature is
        // `Result<wire, E>` and the extern's `Err` arm can `signal_error`
        // with `E: Display`. Consumers may override per-binding by
        // registering a more specific rank-1 `Result<_, ConcreteErr>`
        // (rank-1 phase fires before rank-2).
        base.output_wrapper(
            syn::parse_quote!(Result<_, _>),
            |ok: &syn::Type, err: &syn::Type, _: &Registry<KotlinMeta>| {
                Some((ok.clone(), Some(err.clone()), syn::parse_quote!(v)))
            },
        )
    }
    pub fn source_module(mut self, p: syn::Path) -> Self {
        self.source_module = p;
        self
    }

    /// When `false`, generated wrappers skip the per-call
    /// `withSortedHandleLocks` scaffold (and the dispatch
    /// `as? NativeHandle` lock-adds), emitting only the raw `ptr` read +
    /// closed-handle null-check + native call. Removes per-call lock
    /// allocations / monitor entry at the cost of thread-safety (no
    /// deadlock-safe N-ary locking, no atomic consume). Default `true`.
    pub fn handle_locks(mut self, on: bool) -> Self {
        self.emit_handle_locks = on;
        self
    }

    /// Set the JVM/Kotlin base package (dot-separated, e.g.
    /// `"io.zenoh.jni"`). All derived forms (`java_class_prefix`,
    /// `jni_class_path`) are recomputed.
    pub fn package_prefix(mut self, p: impl Into<String>) -> Self {
        self.package = p.into().trim_matches('.').trim_matches('/').to_string();
        self.recompute_derived();
        self
    }
    /// Set the closure that mangles the framework "harness" class name
    /// `"Native"` (the centralized extern holder). Default = prepend
    /// `"JNI"` (yielding `JNINative`). Affects the generated Kotlin
    /// class name and, via [`Self::jni_class_path`], the JNI extern
    /// symbol path on the Rust side.
    pub fn kotlin_harness_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_harness_name_mangle = Some(Arc::new(f));
        self.recompute_derived();
        self
    }
    /// Set the closure that mangles function names. Called for every
    /// scanned `#[prebindgen]` free function and the synthetic
    /// `freePtr` destructor; receives the camelCased Kotlin-side name
    /// and returns the final form (e.g. `"putPublisher"` →
    /// `"putPublisherViaJNI"`). Default = identity.
    pub fn kotlin_fun_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_fun_name_mangle = Some(Arc::new(f));
        self
    }
    /// Set the closure that mangles Kotlin ptr-class names declared
    /// via [`Self::ptr_class`]. Receives the Rust short name.
    /// Default = identity.
    pub fn kotlin_ptr_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_ptr_class_name_mangle = Some(Arc::new(f));
        self
    }
    /// Set the closure that mangles Kotlin data-class names declared
    /// via [`Self::data_class`]. Receives the Rust short name.
    /// Default = identity.
    pub fn kotlin_data_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_data_class_name_mangle = Some(Arc::new(f));
        self
    }
    /// Set the closure that mangles [`Self::enum_class`]-declared
    /// enum class names. Receives the Rust short name. Default =
    /// identity.
    pub fn kotlin_enum_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_enum_name_mangle = Some(Arc::new(f));
        self
    }
    /// Set the closure that mangles rank-0
    /// [`Self::input_wrapper`] / [`Self::output_wrapper`] pattern
    /// names (e.g. `"Encoding"`). Rank-N patterns are NOT routed
    /// through this closure — they inherit from the inner type's
    /// metadata via the existing rank-N handlers, preserving the
    /// structural invariant `Option<Encoding>` ↔ `JNIEncoding?`.
    /// Default = identity.
    pub fn kotlin_wrapper_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_wrapper_name_mangle = Some(Arc::new(f));
        self
    }

    /// Activate a subpackage context. Subsequent [`Self::function`]
    /// calls land in this subpackage, and any class declared
    /// ([`Self::ptr_class`] / [`Self::data_class`] /
    /// [`Self::enum_class`] / [`Self::value_class`]) while the
    /// subpackage is active gets an FQN of
    /// `<package>.<subpackage>.<ClassName>`.
    ///
    /// Package inheritance is **not** supported — chaining
    /// `.package("a").package("b")` does not produce
    /// `"a.b"`; each call overwrites the previous active subpackage.
    /// To nest, pass a dotted path: `.package("a.b")`.
    ///
    /// Passing an empty string clears the active subpackage (classes /
    /// functions revert to the base `<package>`).
    pub fn package(mut self, subpackage: impl Into<String>) -> Self {
        self.last_opaque_key = None;
        self.last_meta_key = None;
        self.last_entry_ref = None;
        let sub = subpackage
            .into()
            .trim_matches('.')
            .trim_matches('/')
            .to_string();
        if sub.is_empty() {
            self.active_subpackage = None;
        } else {
            self.packages.entry(sub.clone()).or_default();
            self.active_subpackage = Some(sub);
        }
        self
    }

    /// Recompute the derived caches (`java_class_prefix`,
    /// `jni_class_path`) from (`package`,
    /// `kotlin_harness_name_mangle`). Called by
    /// every setter that touches one of those source fields. The JNI
    /// extern symbol path resolves to the centralized Native object,
    /// whose mangled name comes from the harness mangle (default
    /// `"JNI" + n` → `JNINative`).
    fn recompute_derived(&mut self) {
        self.java_class_prefix = self.package.replace(".", "/");
        let native_class = self.mangle_harness("Native");
        self.jni_class_path = if self.package.is_empty() {
            format!("Java_{}", native_class)
        } else {
            format!("Java_{}_{}", self.package.replace(".", "_"), native_class)
        };
    }

    /// Apply [`Self::kotlin_fun_name_mangle`] to `name`, returning the
    /// closure result or `name` verbatim when unset. Called everywhere
    /// the framework derives a function-shaped Kotlin/JNI short name —
    /// scanned `#[prebindgen]` extern symbols, the synthetic `freePtr`
    /// destructor, and the Kotlin-side `external fun` that pairs with
    /// each.
    pub(crate) fn mangle_fun(&self, name: &str) -> String {
        match &self.kotlin_fun_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_ptr_class_name_mangle`] to `name`,
    /// returning the closure result or `name` verbatim when unset.
    pub(crate) fn mangle_ptr_class(&self, name: &str) -> String {
        match &self.kotlin_ptr_class_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_data_class_name_mangle`] to `name`,
    /// returning the closure result or `name` verbatim when unset.
    pub(crate) fn mangle_data_class(&self, name: &str) -> String {
        match &self.kotlin_data_class_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_enum_name_mangle`] to `name`, returning the
    /// closure result or `name` verbatim when unset.
    pub(crate) fn mangle_enum(&self, name: &str) -> String {
        match &self.kotlin_enum_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_wrapper_name_mangle`] to `name`, returning
    /// the closure result or `name` verbatim when unset.
    pub(crate) fn mangle_wrapper(&self, name: &str) -> String {
        match &self.kotlin_wrapper_name_mangle {
            Some(f) => f(name),
            None => name.to_string(),
        }
    }
    /// Apply [`Self::kotlin_harness_name_mangle`] to `name`. The
    /// closure defaults to `|n| format!("JNI{n}")` when unset, so calling
    /// `mangle_harness("Native")` yields `"JNINative"`.
    pub(crate) fn mangle_harness(&self, name: &str) -> String {
        match &self.kotlin_harness_name_mangle {
            Some(f) => f(name),
            None => format!("JNI{name}"),
        }
    }
    /// The mangled name of the centralized Native object that hosts
    /// every JNI `external fun`. Drives both the Kotlin class emission
    /// and the JNI extern symbol path on the Rust side.
    pub(crate) fn jni_native_class_name(&self) -> String {
        self.mangle_harness("Native")
    }

    /// Resolve a relative class name against [`Self::package`]. Panics
    /// if `name` contains a `.` (a check that catches accidental FQNs in
    /// the relative-name builders). The framework refuses dotted names
    /// on purpose: a binding crate owns one package and must not write
    /// classes into anyone else's namespace. Higher layers wrap or
    /// re-export — they don't get injected into.
    pub(crate) fn resolve_class_fqn(&self, name: &str) -> String {
        assert!(
            !name.contains('.'),
            "Kotlin class name `{}` must be relative (no dots) — FQNs are derived from JniGen::package",
            name
        );
        // If a `package(p)` context is active, classes declared
        // while it's active land under `<package>.<p>` instead of just
        // `<package>`. The user explicitly opts in by ordering the
        // declaration after the `package` call.
        let base: String = match (&self.package, &self.active_subpackage) {
            (p, Some(sub)) if !p.is_empty() => format!("{}.{}", p, sub),
            (p, Some(sub)) if p.is_empty() => sub.clone(),
            (p, None) => p.clone(),
            _ => String::new(),
        };
        if base.is_empty() {
            name.to_string()
        } else {
            format!("{}.{}", base, name)
        }
    }

    // ── Structured type-conversion builders ──────────────────────────

    /// Declare a typed Kotlin handle class backed by an opaque Rust
    /// type. Configures: jlong wire for both input and output,
    /// `Box::into_raw`/`Box::from_raw` lifecycle, the `instanceof`
    /// dispatch class, and the Kotlin typed-handle class FQN. By
    /// default a `.kt` shell is auto-emitted — chain
    /// [`Self::suppress_kotlin_code`] to keep the file hand-maintained,
    /// or chain one or more [`Self::method`] calls to promote
    /// `#[prebindgen]` functions onto the class as instance methods.
    pub fn ptr_class(mut self, rust_type: syn::Type) -> Self {
        let key = TypeKey::from_type(&rust_type);
        let short = rust_short_name(&key);
        let fqn = self.resolve_class_fqn(&self.mangle_ptr_class(&short));
        let entry = self.types.entry(key.clone()).or_default();
        entry.opaque = Some(OpaqueConfig::default());
        // `kotlin_name` holds the typed-handle FQN for FQN-consumers
        // (typed-handle class emission, `instanceof` dispatch, return-
        // value constructor wrap). The value-context Kotlin name for
        // opaque types — `"Long"` — flows separately through
        // [`KotlinMeta::kotlin_name`] produced by the rank-0 opaque
        // handler, so wire-level mentions don't collide with the FQN.
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.last_opaque_key = Some(key.clone());
        self.last_meta_key = Some(key);
        self.last_entry_ref = None;
        self
    }

    /// Declare a `#[prebindgen]` function as a free-standing wrapper
    /// under the currently-active [`Self::package`] context. If a
    /// class context is also live, calling `fun` clears it — the
    /// idea being that "leak class context to package level" makes the
    /// chain unambiguous after one fn-level declaration. Panics if no
    /// `package` is active.
    pub fn fun(self, ident: syn::Ident) -> Self {
        self.push_fun(MethodEntry::new(ident))
    }

    /// Declare a `#[prebindgen]` **read accessor** (`f(&T) -> …`) — like
    /// [`Self::fun`] (exported wrapper) but flagged
    /// [`MethodEntry::is_accessor`]: the parameter composer is never applied to
    /// it (explicit [`Self::construct`] on its params is a build error, and
    /// constructor [`Self::default`] auto-apply skips it), and it is the only
    /// kind of function a decomposer record ([`Self::deconstructor_record`] /
    /// [`Self::converter`] / [`Self::deconstructor_record_nested`]) may reference.
    pub fn fun_accessor(self, ident: syn::Ident) -> Self {
        self.push_fun(MethodEntry::new_accessor(ident))
    }

    /// Shared body of [`Self::fun`] / [`Self::fun_accessor`].
    fn push_fun(mut self, entry: MethodEntry) -> Self {
        let sub = self
            .active_subpackage
            .clone()
            .expect("JniGen::fun must be chained inside a `package(...)` context");
        // Leak any class context back to package level.
        self.last_meta_key = None;
        self.last_opaque_key = None;
        let pkg = self.packages.entry(sub.clone()).or_default();
        let idx = pkg.functions.len();
        pkg.functions.push(entry);
        self.last_entry_ref = Some(NamedEntryRef::Function(sub, idx));
        self
    }

    /// Override the Kotlin-side name for the most recent [`Self::fun`] /
    /// [`Self::fun_accessor`] entry. Default (without `.name(...)`) is
    /// `snake_to_camel(rust_ident)` (e.g. `z_hello_whatami` → `zHelloWhatami`).
    /// Panics if not chained immediately after a fn-level builder.
    pub fn name(mut self, kotlin_name: impl Into<String>) -> Self {
        let r = self.last_entry_ref.clone().expect(
            "JniGen::name must be chained immediately after `.fun(...)` / `.fun_accessor(...)`",
        );
        let name = kotlin_name.into();
        let NamedEntryRef::Function(sub, idx) = r;
        let pkg = self.packages.get_mut(&sub).expect("package entry vanished");
        pkg.functions[idx].kotlin_name_override = Some(name);
        self
    }

    // ── Canonical type representation (input / output on the ptr_class) ──

    /// Rust type of the most recent [`Self::ptr_class`], for the
    /// `.ptr_class_input*` / `.ptr_class_output*` chain. Panics otherwise.
    fn current_ptr_class(&self) -> syn::Type {
        self.last_opaque_key
            .clone()
            .expect("ptr_class_input/output must be chained after `.ptr_class(...)`")
            .to_type()
    }

    /// **Identity input variant**: the canonical input of the current
    /// `ptr_class` accepts the handle directly (alongside any `.ptr_class_input`
    /// build-from variants, selector-dispatched).
    pub fn ptr_class_input_direct(mut self) -> Self {
        let t = self.current_ptr_class();
        self.expansions.ensure_canonical_constructor(t);
        self.expansions.add_constructor_variant_id();
        self
    }

    /// **Build-from input variant**: the canonical input may build the current
    /// `ptr_class` by calling `func` with `func`'s (recursively expanded) params.
    pub fn ptr_class_input(mut self, func: syn::Ident) -> Self {
        let t = self.current_ptr_class();
        self.expansions.ensure_canonical_constructor(t);
        self.expansions.add_constructor_variant(func);
        self
    }

    /// **Identity output record**: the current `ptr_class`'s canonical output
    /// includes the handle itself (one of possibly several outputs).
    pub fn ptr_class_output_direct(mut self) -> Self {
        let t = self.current_ptr_class();
        self.deconstructors.ensure_canonical_deconstructor(t);
        self.deconstructors.add_deconstructor_record_id();
        self
    }

    /// **Accessor output record**: the current `ptr_class`'s canonical output
    /// includes `func`'s result, unwrapped per the return type's own canonical
    /// output (one leaf for a scalar/string/enum; spliced for a nested ptr_class).
    pub fn ptr_class_output(mut self, func: syn::Ident) -> Self {
        let t = self.current_ptr_class();
        self.deconstructors.ensure_canonical_deconstructor(t);
        self.deconstructors.add_deconstructor_record(func);
        self
    }

    // ── Per-function overrides of the canonical representation ──────────

    /// Per-fn: `param` skips the canonical input and takes the raw handle.
    pub fn fun_input_direct(mut self, param: syn::Ident) -> Self {
        let func = self.current_fn_ident();
        self.expansions.add_skip_default_construct(func, param);
        self
    }

    /// Per-fn: the return value skips the canonical output and stays a raw handle.
    pub fn fun_output_direct(mut self) -> Self {
        let func = self.current_fn_ident();
        self.deconstructors.add_skip_default_output(func);
        self
    }

    /// Per-fn: `param` is built from only the named subset of the canonical
    /// input's build-from variants (plus identity if the canonical input has it).
    pub fn fun_input(
        mut self,
        param: syn::Ident,
        funcs: impl IntoIterator<Item = syn::Ident>,
    ) -> Self {
        let func = self.current_fn_ident();
        self.expansions
            .add_construct_subset(func, param, funcs.into_iter().collect());
        self
    }

    /// Per-fn: replace the canonical output with an explicit record list (the
    /// `func`s, each unwrapped per its return type's canonical output).
    pub fn fun_output(mut self, funcs: impl IntoIterator<Item = syn::Ident>) -> Self {
        let func = self.current_fn_ident();
        self.deconstructors
            .add_output_inline(func, funcs.into_iter().collect());
        self
    }

    /// Rust ident of the function the current per-fn override chain targets,
    /// resolved from the live [`Self::fun`] cursor.
    fn current_fn_ident(&self) -> syn::Ident {
        let r = self
            .last_entry_ref
            .clone()
            .expect("JniGen::expand must be chained after `.fun(...)`");
        let NamedEntryRef::Function(sub, idx) = r;
        self.packages
            .get(&sub)
            .expect("package entry vanished")
            .functions[idx]
            .rust_ident
            .clone()
    }

    /// Opt out of Kotlin class emission for the most recent
    /// [`Self::ptr_class`] / [`Self::enum_class`] — the `.kt` file is
    /// assumed to be hand-written. Without this, a typed-handle shell
    /// class (or an `enum class`) is auto-emitted. Panics if no
    /// `ptr_class` / `enum_class` is in scope.
    pub fn suppress_kotlin_code(mut self) -> Self {
        let key = self.last_meta_key.clone().expect(
            "JniGen::suppress_kotlin_code must be chained immediately after a \
             `ptr_class` or `enum_class` call",
        );
        let entry = self.types.get_mut(&key).expect("type entry vanished");
        if let Some(opaque) = entry.opaque.as_mut() {
            opaque.suppress_kotlin_code = true;
        } else if let Some(enum_cfg) = entry.enum_cfg.as_mut() {
            enum_cfg.suppress_kotlin_code = true;
        } else {
            panic!(
                "JniGen::suppress_kotlin_code: type entry for `{}` has neither \
                 `opaque` nor `enum_cfg` set — chain after `ptr_class` or \
                 `enum_class`",
                key.as_str()
            );
        }
        self
    }

    /// Whether `ty` was registered via [`Self::enum_class`] — used by
    /// the Kotlin wrapper generator to decide if a parameter needs a
    /// `.value` projection between the typed enum (Kotlin signature) and
    /// the `Int` wire (JNI `external fun`).
    pub(crate) fn is_kotlin_enum(&self, ty: &syn::Type) -> bool {
        let key = TypeKey::from_type(ty);
        self.types
            .get(&key)
            .and_then(|c| c.enum_cfg.as_ref())
            .is_some()
    }

    /// Declare a `#[prebindgen]`-marked `enum` as a Kotlin `enum class`.
    /// Configures: `jni::sys::jint` wire (input + output), `TryFrom<i32>`
    /// decode on input, `as jint` encode on output, and Kotlin enum-file
    /// emission. The enum must be C-like (unit variants only) and either
    /// `#[repr(i32)]` / `#[repr(u8)]` (or similar) with explicit
    /// discriminants — the Kotlin emitter and the generated
    /// `TryFrom<i32>` decode rely on the discriminant values matching the
    /// jint wire.
    ///
    /// By default a `.kt` file is auto-emitted under [`Self::package`]; chain
    /// [`Self::suppress_kotlin_code`] to keep the file hand-maintained.
    /// The class name passes through
    /// [`Self::kotlin_enum_name_mangle`] (default = Rust short name).
    pub fn enum_class(mut self, rust_type: syn::Type) -> Self {
        let key = TypeKey::from_type(&rust_type);
        let short = rust_short_name(&key);
        let fqn = self.resolve_class_fqn(&self.mangle_enum(&short));
        let entry = self.types.entry(key.clone()).or_default();
        assert!(
            entry.opaque.is_none(),
            "JniGen::enum_class: `{}` is already registered as an opaque \
             handle via `ptr_class` — a type can be one or the other, \
             not both",
            short
        );
        entry.enum_cfg = Some(EnumConfig::default());
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        // Clear opaque tracker so a stray `.method(...)` doesn't latch onto
        // this entry; `last_meta_key` is what `.suppress_kotlin_code` reads
        // for chained config.
        self.last_opaque_key = None;
        self.last_meta_key = Some(key);
        self.last_entry_ref = None;
        self
    }

    /// Stamp a verbatim Kotlin type expression (e.g. `"List<ByteArray>"`)
    /// onto the entry registered by the most recent type-config builder.
    /// Use this when the Kotlin type is not a class FQN (generics,
    /// primitives, container types). For class names, the per-kind
    /// `kotlin_*_name_mangle` closures (configured on [`JniGen`]) own
    /// derivation — `with_kotlin_type` is the escape hatch for verbatim
    /// expressions that don't map onto any one element kind.
    pub fn with_kotlin_type(mut self, kotlin_expr: impl Into<String>) -> Self {
        let key = self
            .last_meta_key
            .clone()
            .or_else(|| self.last_opaque_key.clone())
            .expect(
                "JniGen::with_kotlin_type must be chained immediately after a \
                 type-config builder",
            );
        let expr = kotlin_expr.into();
        let entry = self.types.get_mut(&key).expect("meta entry vanished");
        entry.kotlin_name = Some(expr.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), expr));
        self
    }

    /// Declare a Rust struct that should appear in Kotlin as a data
    /// class under a derived name. The name passes through
    /// [`Self::kotlin_data_class_name_mangle`] (default = Rust short
    /// name, generics / lifetimes stripped). Only affects Kotlin
    /// emission — no Rust-side converter override.
    pub fn data_class(mut self, rust_type: syn::Type) -> Self {
        let key = TypeKey::from_type(&rust_type);
        let short = rust_short_name(&key);
        let fqn = self.resolve_class_fqn(&self.mangle_data_class(&short));
        let entry = self.types.entry(key.clone()).or_default();
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.last_opaque_key = None;
        self.last_meta_key = Some(key);
        self.last_entry_ref = None;
        self
    }

    /// Declare a **`Copy` value-blob** type: a Rust type passed across the
    /// JNI boundary **by value as its raw memory bytes** in a `ByteArray`,
    /// rather than as a closeable `jlong` heap handle. The value-level peer
    /// of [`Self::ptr_class`] — `ByteArray` is to a blob what `Long` is to a
    /// handle. Use it for small `Copy` types (e.g. `ZenohId`) so they need no
    /// `close()` and so `Vec<T>` can surface as `List<ByteArray>` (a
    /// `Vec<closeable-handle>` is rejected; see the `Vec<_>` handler).
    ///
    /// The type **must be `Copy`** — the generator emits a compile-time
    /// assertion to that effect (a non-`Copy` declaration is a hard build
    /// error). Conversions reinterpret the bytes (`read_unaligned` on input,
    /// raw-bytes read on output), so the blob is valid only same-architecture
    /// in-process, exactly like an opaque handle pointer. Mutually exclusive
    /// with `ptr_class` / `enum_class`.
    pub fn value_blob(mut self, rust_type: syn::Type) -> Self {
        let key = TypeKey::from_type(&rust_type);
        let short = rust_short_name(&key);
        // Typed Kotlin FQN for the emitted `@JvmInline value class` — the same
        // FQN-consumer slot a `ptr_class` / `value_class` uses (typed-class
        // emission, projection-leaf lookup, `instanceof` imports). The
        // *value-level* name (`"ByteArray"`) is set separately on the rank-0
        // converter's metadata, so wire mentions stay `ByteArray` while typed
        // positions render the value class.
        let fqn = self.resolve_class_fqn(&self.mangle_data_class(&short));
        let entry = self.types.entry(key.clone()).or_default();
        entry.value_blob = true;
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.last_opaque_key = None;
        self.last_meta_key = Some(key);
        self.last_entry_ref = None;
        self
    }

    /// Register a rank-N **input converter**. `pattern` contains 0–3
    /// `_` placeholders; the closure's arity selects the rank table.
    /// The closure returns `Some((ty, exc, body))` (see [`WrapperFn`]
    /// for the triple's full semantics) or `None` (defer to a later
    /// resolver phase). The body sees `env: &mut JNIEnv` and `v: &<wire>`
    /// in scope.
    ///
    /// * `exc = None` ⇒ non-throwing: `body` evaluates to a bare `ty`;
    ///   framework emits `-> Result<ty, __JniErr>` with an `Ok(...)`
    ///   wrap, and `?` inside propagates the framework error.
    /// * `exc = Some(<Rust type>)` ⇒ throwing: `body` evaluates to
    ///   `Result<ty, <Rust type>>`; framework emits it verbatim. The
    ///   type must match a [`Self::throwable`] declaration
    ///   by **exact canonical-form equality** with its `rust_type` (see
    ///   [`Self::find_exception`] — no short-name fallback). The match
    ///   is validated at lookup time.
    ///
    /// `ty` is auto-classified at resolve: a wire shape ⇒ terminal
    /// converter; a distinct rust type with its own converter ⇒ a
    /// value-inspecting stage composed onto that converter's chain
    /// (see [`Self::lookup_input`]).
    pub fn input_wrapper<A, B>(self, pattern: syn::Type, builder: B) -> Self
    where
        B: WrapperBuilder<A>,
    {
        let key = TypeKey::from_type(&pattern);
        let rank = B::rank();
        let mut s = self;
        s.input_wrappers[rank].insert(key.clone(), builder.into_wrapper_fn());
        s.note_wrapper_registration(key, rank);
        s
    }

    /// Output-direction counterpart of [`Self::input_wrapper`]. Same
    /// closure shape, same `exc = None` / `Some(<Rust type>)` semantics,
    /// same terminal-vs-composed classification — see that method's docs.
    /// (`Some(parse_quote!(<full path>))` with a rust-typed `ty`, e.g.
    /// `(T, Some(parse_quote!(zenoh_flat::errors::ZError)), v)` for
    /// `ZResult<T>`, gives the auto-composed peel that the deleted
    /// `output_throw_stage` used to register.)
    pub fn output_wrapper<A, B>(self, pattern: syn::Type, builder: B) -> Self
    where
        B: WrapperBuilder<A>,
    {
        let key = TypeKey::from_type(&pattern);
        let rank = B::rank();
        let mut s = self;
        s.output_wrappers[rank].insert(key.clone(), builder.into_wrapper_fn());
        s.note_wrapper_registration(key, rank);
        s
    }

    /// Shared post-registration bookkeeping for wrapper inserts. Rank-0
    /// patterns identify a concrete type — auto-stamp `kotlin_name` via
    /// [`Self::mangle_wrapper`] (skipping non-path patterns like `()`
    /// where there is no sensible short name). Rank ≥1 patterns are
    /// wildcards — per-outer-type names come from inner-metadata
    /// propagation via [`Self::override_kotlin_name`].
    fn note_wrapper_registration(&mut self, key: TypeKey, rank: usize) {
        self.last_opaque_key = None;
        self.last_entry_ref = None;
        if rank == 0 {
            let entry = self.types.entry(key.clone()).or_default();
            // Skip any entry whose kotlin_name has already been stamped
            // (e.g. by an earlier data_class / ptr_class call for the
            // same type — a wrapper layered on top should not override
            // it). Then derive the short name from the canonical
            // TypeKey; non-path patterns ($()$, references, etc.)
            // yield no Kotlin class name and are left as `None`.
            if entry.kotlin_name.is_none() {
                if let Some(short) = rust_short_name_opt(&key) {
                    let fqn = self.resolve_class_fqn(&self.mangle_wrapper(&short));
                    let entry = self.types.get_mut(&key).expect("just-inserted entry");
                    entry.kotlin_name = Some(fqn.clone());
                    self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
                }
            }
            self.last_meta_key = Some(key);
        } else {
            self.last_meta_key = None;
        }
    }

    /// Build a `KotlinMeta` carrying just the value-context Kotlin name.
    /// Used by every built-in converter (primitives, structs, `Option<_>`,
    /// `Vec<_>`, `impl Fn(...)` lambdas). Errors are routed uniformly to the
    /// per-call `signal_error` sink by the extern emitter, so no
    /// per-converter exception metadata is carried.
    pub(crate) fn framework_meta(&self, kotlin_name: Option<kt::KtType>) -> KotlinMeta {
        KotlinMeta {
            kotlin_name,
            value_rust_key: None,
            projection: None,
        }
    }

    // ── Wrapper-table lookups (used by Prebindgen impl) ───────────

    /// Look up a registered input converter for `pat` with `args`
    /// substituted into its `_` slots. The closure's middle slot (see
    /// [`WrapperFn`]) carries the bound exception — `None` ⇒ framework
    /// `__JniErr` with an `Ok`-wrap, `Some(<Rust type>)` ⇒
    /// `Result<ty, <Rust type>>` emitted verbatim, decided in
    /// [`Self::build_input_fn`].
    ///
    /// The closure's returned type is classified by [`is_wire_type`]:
    /// * **wire** ⇒ terminal: a single converter `wire → outer`.
    /// * **rust type** ⇒ composed: that type's input converter runs
    ///   first (`wire → ty`), then this registration's body is a
    ///   value-inspecting stage `ty → outer` (built by-value via
    ///   [`Self::build_output_fn`]) prepended to the inner chain. Defer
    ///   (`None`) if the inner converter isn't resolved yet.
    pub(crate) fn lookup_input(
        &self,
        pat: &syn::Type,
        args: &[syn::Type],
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let rank = args.len();
        if rank > 3 {
            return None;
        }
        let key = TypeKey::from_type(pat);
        let f = self.input_wrappers[rank].get(&key)?;
        let (ty, exc_ty, body) = f(args, registry)?;
        // The closure's middle slot carries the `Result`'s raw Rust error
        // type (or `None` for the framework `__JniErr`); it feeds the
        // converter signature `Result<_, E>` directly — no registration.
        let exc = exc_ty.as_ref();
        let outer = substitute_wildcards(pat, args);
        // Terminal vs composed: `ty` is composed iff it's a *distinct*
        // rust type with its own input converter. The self-check guards
        // the void/identity case (`output_wrapper("()")` returns `ty ==
        // outer`), and the registered-converter probe distinguishes a
        // rust continue-type (compose) from a wire (terminal) without
        // forcing `()` either way. A non-wire `ty` that isn't yet
        // resolved defers.
        let is_self = TypeKey::from_type(&ty) == TypeKey::from_type(&outer);
        let inner = if is_self {
            None
        } else {
            registry.input_entry(&ty)
        };
        match inner {
            None if is_self || is_wire_type(&ty) => {
                // Terminal: `ty` is the wire; the body produces `outer`.
                let (niches, kotlin_name) = if rank == 0 {
                    let kn = self
                        .types
                        .get(&key)
                        .and_then(|c| c.kotlin_name.clone())
                        .map(kt::KtType::cls)
                        .or_else(|| kotlin_for_wire(&ty));
                    (Niches::empty(), kn)
                } else {
                    (default_niches_for_wire(&ty), None)
                };
                Some(ConverterImpl {
                    pre_stages: vec![],
                    function: self.build_input_fn(&outer, &ty, &body, exc),
                    destination: ty,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        value_rust_key: None,
                        // Terminal: body produces the wire directly, no inner
                        // converter composed, so no handle to carry.
                        projection: None,
                    },
                })
            }
            // Non-wire `ty` whose converter isn't resolved yet — defer.
            None => None,
            Some(inner) => {
                // Composed: `ty` is the inner source rust type. Its input
                // converter (`wire → ty`) is the wire-facing function;
                // this body is a stage `ty → outer` that runs after it.
                // The stage takes the inner-produced value BY VALUE and
                // yields `outer`, i.e. the same shape an output converter
                // has — so it's built with `build_output_fn`.
                let stage = Stage {
                    function: self.build_output_fn(&ty, &outer, &body, exc),
                    metadata: KotlinMeta::default(),
                };
                let mut pre_stages = vec![stage];
                pre_stages.extend(inner.pre_stages.iter().cloned());
                let (kotlin_name, value_rust_key) = if rank >= 1 {
                    (
                        inner.metadata.kotlin_name.clone(),
                        Some(TypeKey::from_type(&args[0]).as_str().to_string()),
                    )
                } else {
                    (inner.metadata.kotlin_name.clone(), None)
                };
                let niches = if rank == 0 {
                    Niches::empty()
                } else {
                    default_niches_for_wire(&inner.destination)
                };
                Some(ConverterImpl {
                    function: inner.function.clone(),
                    destination: inner.destination.clone(),
                    pre_stages,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        value_rust_key,
                        // Identity propagation: a composed wrapper (e.g.
                        // `Result<Handle,Error>`) projects to its inner value,
                        // so a handle inner stays a handle (same strategy).
                        projection: inner.metadata.projection.clone(),
                    },
                })
            }
        }
    }

    /// Look up a registered output converter for `pat` with `args`
    /// substituted into its `_` slots. Mirror of [`Self::lookup_input`].
    ///
    /// The closure's returned type is classified by [`is_wire_type`]:
    /// * **wire** ⇒ terminal: a single converter `outer → wire`,
    ///   returning `Result<wire, err>` (throwing iff [`ConverterReg::exc`]
    ///   is set).
    /// * **rust type** ⇒ composed: this body is a value-inspecting stage
    ///   `outer → ty` prepended to `ty`'s own output converter chain
    ///   (e.g. `ZResult<T>` returns rust `T`, so the peel stage raises
    ///   its exception and `T`'s converter marshals the wire). Defer
    ///   (`None`) if `ty`'s converter isn't resolved yet.
    pub(crate) fn lookup_output(
        &self,
        pat: &syn::Type,
        args: &[syn::Type],
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let rank = args.len();
        if rank > 3 {
            return None;
        }
        let key = TypeKey::from_type(pat);
        let f = self.output_wrappers[rank].get(&key)?;
        let (ty, exc_ty, body) = f(args, registry)?;
        // The closure's middle slot carries the `Result`'s raw Rust error
        // type (or `None` for the framework `__JniErr`) — see lookup_input.
        let exc = exc_ty.as_ref();
        let outer = substitute_wildcards(pat, args);
        // Terminal vs composed — see [`Self::lookup_input`] for the rule.
        let is_self = TypeKey::from_type(&ty) == TypeKey::from_type(&outer);
        let inner = if is_self {
            None
        } else {
            registry.output_entry(&ty)
        };
        match inner {
            None if is_self || is_wire_type(&ty) => {
                // Terminal: `ty` is the wire; the body produces it from `outer`.
                let (kotlin_name, value_rust_key) = if rank >= 1 {
                    registry
                        .output_entry(&args[0])
                        .map(|e| {
                            (
                                e.metadata.kotlin_name.clone(),
                                Some(TypeKey::from_type(&args[0]).as_str().to_string()),
                            )
                        })
                        .unwrap_or((None, None))
                } else {
                    let kn = self
                        .types
                        .get(&key)
                        .and_then(|c| c.kotlin_name.clone())
                        .map(kt::KtType::cls)
                        .or_else(|| kotlin_for_wire(&ty));
                    (kn, None)
                };
                let niches = if rank == 0 {
                    Niches::empty()
                } else {
                    default_niches_for_wire(&ty)
                };
                Some(ConverterImpl {
                    pre_stages: vec![],
                    function: self.build_output_fn(&outer, &ty, &body, exc),
                    destination: ty,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        value_rust_key,
                        // Terminal: body produces the wire directly, no inner
                        // converter composed, so no handle to carry.
                        projection: None,
                    },
                })
            }
            // Non-wire `ty` whose converter isn't resolved yet — defer.
            None => None,
            Some(inner) => {
                // Composed: `ty` is the continue rust type; chain its converter.
                let stage = Stage {
                    function: self.build_output_fn(&outer, &ty, &body, exc),
                    metadata: KotlinMeta::default(),
                };
                let mut pre_stages = vec![stage];
                pre_stages.extend(inner.pre_stages.iter().cloned());
                let (kotlin_name, value_rust_key) = if rank >= 1 {
                    (
                        inner.metadata.kotlin_name.clone(),
                        Some(TypeKey::from_type(&args[0]).as_str().to_string()),
                    )
                } else {
                    (inner.metadata.kotlin_name.clone(), None)
                };
                let niches = if rank == 0 {
                    Niches::empty()
                } else {
                    default_niches_for_wire(&inner.destination)
                };
                Some(ConverterImpl {
                    function: inner.function.clone(),
                    destination: inner.destination.clone(),
                    pre_stages,
                    niches,
                    metadata: KotlinMeta {
                        kotlin_name,
                        value_rust_key,
                        // Identity propagation: a composed wrapper (e.g.
                        // `Result<Handle,Error>`) projects to its inner value,
                        // so a handle inner stays a handle (same strategy).
                        projection: inner.metadata.projection.clone(),
                    },
                })
            }
        }
    }
}

/// Recognise the JNI **wire** shapes a converter body may return as a
/// terminal destination. Reuses the back-end's existing wire knowledge:
/// every `jni::sys::*` / `jni::objects::*` wire is recognised by
/// [`kotlin_for_wire`] (returns `Some`), plus
/// raw pointers structurally — so there is no separate wire-type
/// allowlist to keep in sync.
///
/// `()` is deliberately **not** treated as a wire here: it is ambiguous
/// (the void wire of the `output_wrapper("()")` self-converter *and* the
/// unit continue-type of `ZResult<()>`). The terminal-vs-composed
/// decision in [`JniGen::lookup_input`] / [`JniGen::lookup_output`]
/// resolves that ambiguity via the self-check + registered-converter
/// probe, so `()` flows correctly without being force-classified here.
pub(crate) fn is_wire_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Ptr(_)) || kotlin_for_wire(ty).is_some()
}

/// Bare-ident type `__JniErr` — the generated file's alias for the
/// framework [`crate::api::lang::jnigen::jni::JniBindingError`]. Built-in
/// converters use this as their `Result<…, _>` error type so their bodies'
/// `<__JniErr as From<String>>::from(...)` calls keep compiling. A
/// `Result<T, E>` return instead binds its own raw `E` (see
/// [`JniGen::lookup_output`]); the extern's `Err` arm funnels both to the
/// per-call `signal_error` sink via `E: Display`.
pub(crate) fn default_err_type() -> syn::Type {
    syn::parse_quote!(__JniErr)
}

/// The actual framework error type the `__JniErr` alias resolves to: the
/// E-agnostic `JniBindingError<()>` whose failures are always `JniError`
/// (binding-layer). A `Result<T, E>` return carries its own raw `E`, surfaced
/// as `UserError` at the extern's error site.
pub(crate) fn framework_error_type() -> syn::Type {
    syn::parse_quote!(::prebindgen::lang::JniBindingError<()>)
}

/// The body expression to splice into a converter `fn` returning
/// `Result<_, E>`: with `exc = None` the `body` is a bare value, so wrap
/// it `Ok(body)`; with `exc = Some(E)` the `body` already evaluates to
/// the `Result`, so emit it verbatim.
pub(crate) fn body_for_exc(body: &syn::Expr, exc: Option<&syn::Type>) -> syn::Expr {
    if exc.is_some() {
        body.clone()
    } else {
        syn::parse_quote!(Ok(#body))
    }
}

/// Substitute the wildcard `_` slots of `pat` with `args` (left-to-right
/// depth-first), returning the concrete outer `syn::Type`. Mirrors the
/// substitution the resolver performs to derive a wildcard pattern from
/// a concrete type.
pub(crate) fn substitute_wildcards(pat: &syn::Type, args: &[syn::Type]) -> syn::Type {
    let mut idx = 0usize;
    fn walk(ty: &mut syn::Type, args: &[syn::Type], idx: &mut usize) {
        match ty {
            syn::Type::Infer(_) => {
                if let Some(replacement) = args.get(*idx) {
                    *ty = replacement.clone();
                }
                *idx += 1;
            }
            syn::Type::Path(tp) => {
                for seg in &mut tp.path.segments {
                    if let syn::PathArguments::AngleBracketed(ab) = &mut seg.arguments {
                        for arg in &mut ab.args {
                            if let syn::GenericArgument::Type(inner) = arg {
                                walk(inner, args, idx);
                            }
                        }
                    }
                }
            }
            syn::Type::Reference(r) => walk(&mut r.elem, args, idx),
            syn::Type::Tuple(t) => {
                for e in &mut t.elems {
                    walk(e, args, idx);
                }
            }
            syn::Type::Array(a) => walk(&mut a.elem, args, idx),
            syn::Type::Slice(s) => walk(&mut s.elem, args, idx),
            syn::Type::Ptr(p) => walk(&mut p.elem, args, idx),
            syn::Type::Paren(p) => walk(&mut p.elem, args, idx),
            syn::Type::Group(g) => walk(&mut g.elem, args, idx),
            _ => {}
        }
    }
    let mut out = pat.clone();
    walk(&mut out, args, &mut idx);
    out
}

impl Default for JniGen {
    fn default() -> Self {
        Self::new()
    }
}
