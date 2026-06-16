//! Builder API for [`JniGen`].
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.

use super::*;

impl<S> JniGen<S> {
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

    fn into_state<T>(self, state: T) -> JniGen<T> {
        JniGen {
            inner: self.inner,
            state,
        }
    }
}

impl JniGen<Root> {
    /// Convenience constructor with sensible defaults; the paths still need
    /// to be set explicitly via the field-mutation builder methods.
    pub fn new() -> Self {
        let mut base = Self {
            inner: JniGenInner {
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
                active_subpackage: None,
                emit_handle_locks: true,
                expansions: crate::api::core::expand::Expansions::default(),
                deconstructors: crate::api::core::unfold::Deconstructors::default(),
                class_accessors: HashMap::new(),
            },
            state: Root,
        };
        // Built-in rank-2 `Result<_, _>` peel: every Result<T, E> succeeds
        // as T and routes E to the error-sink on Err. The error type `E` is
        // carried through the middle slot so the converter signature is
        // `Result<wire, E>` and the extern's `Err` arm can `signal_error`
        // with `E: Display`. Consumers may override per-binding by
        // registering a more specific rank-1 `Result<_, ConcreteErr>`
        // (rank-1 phase fires before rank-2).
        let pattern: syn::Type = syn::parse_quote!(Result<_, _>);
        let key = TypeKey::from_type(&pattern);
        base.output_wrappers[2].insert(
            key.clone(),
            Arc::new(|args: &[syn::Type], _: &Registry<KotlinMeta>| {
                Some((args[0].clone(), Some(args[1].clone()), syn::parse_quote!(v)))
            }),
        );
        base.note_wrapper_registration(key, 2);
        base
    }
}

impl<S> JniGen<S> {
    /// Set the Rust module path that contains the original `#[prebindgen]`
    /// items. Generated Rust wrappers call functions as
    /// `<source_module>::<function>(...)`; defaults to `crate`.
    pub fn source_module(mut self, p: syn::Path) -> Self {
        self.source_module = p;
        self
    }

    /// Set the JVM/Kotlin **base** package (dot-separated, e.g.
    /// `"io.zenoh.jni"`). All derived forms are recomputed.
    ///
    /// Use [`Self::package`] for generated subpackages below this base.
    pub fn package_prefix(mut self, p: impl Into<String>) -> Self {
        self.package = p.into().trim_matches('.').trim_matches('/').to_string();
        self.recompute_derived();
        self
    }

    /// Disable the per-call handle-lock scaffold.
    pub fn disable_handle_locks(mut self) -> Self {
        self.emit_handle_locks = false;
        self
    }

    /// Set the closure that mangles the framework "harness" class name
    /// `"Native"` (the centralized extern holder). Default = prepend
    /// `"JNI"` (yielding `JNINative`). Affects the generated Kotlin
    /// class name and the derived JNI extern symbol path on the Rust side.
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

    /// Activate a generated subpackage context below the base package set by
    /// [`Self::package_prefix`]. Despite the short name, this does **not** set
    /// the base package; use [`Self::package_prefix`] for that. Subsequent
    /// [`Self::package_fun`] calls land in this subpackage, and
    /// any class declared
    /// ([`Self::ptr_class`] / [`Self::data_class`] /
    /// [`Self::enum_class`] / [`Self::value_class`]) while the
    /// subpackage is active gets an FQN of
    /// `<package>.<subpackage>.<ClassName>`.
    ///
    /// Subpackage inheritance is **not** supported — chaining
    /// `.package("a").package("b")` does not produce
    /// `"a.b"`; each call overwrites the previous active subpackage.
    /// To nest, pass a dotted path: `.package("a.b")`.
    ///
    /// Passing an empty string clears the active subpackage (classes /
    /// functions revert to the base `<package>`).
    pub fn package(mut self, subpackage: impl Into<String>) -> JniGen<Package> {
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
        self.into_state(Package)
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
}

impl<S: TypeDeclState> JniGen<S> {
    // ── Structured type-conversion builders ──────────────────────────

    /// Declare a typed Kotlin handle class backed by an opaque Rust
    /// type. Configures: jlong wire for both input and output,
    /// `Box::into_raw`/`Box::from_raw` lifecycle, the `instanceof`
    /// dispatch class, and the Kotlin typed-handle class FQN. By
    /// default a `.kt` shell is auto-emitted — chain
    /// [`Self::suppress_kotlin_code`] to keep the file hand-maintained,
    /// or chain `.ptr_class_input*` / `.ptr_class_output*` calls to define its
    /// canonical conversion shape.
    pub fn ptr_class(mut self, rust_type: syn::Type) -> JniGen<PtrClass> {
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
        self.into_state(PtrClass {
            key,
            package: std::marker::PhantomData,
        })
    }
}

impl<S: PackageState> JniGen<S> {
    /// Declare a `#[prebindgen]` function as a free-standing package wrapper
    /// under the currently-active [`Self::package`] subpackage context. If a
    /// class context is also live, calling `package_fun` clears it — the idea
    /// being that "leak class context to package level" makes the chain
    /// unambiguous after one fn-level declaration.
    pub fn package_fun(self, ident: syn::Ident) -> JniGen<Function> {
        self.push_fun(MethodEntry::new(ident))
    }

    /// Shared body of [`Self::package_fun`].
    fn push_fun(mut self, entry: MethodEntry) -> JniGen<Function> {
        let sub = self
            .active_subpackage
            .clone()
            .expect("JniGen::fun must be chained inside a `package(...)` context");
        let pkg = self.packages.entry(sub.clone()).or_default();
        let idx = pkg.functions.len();
        pkg.functions.push(entry);
        self.into_state(Function {
            package: sub,
            index: idx,
        })
    }
}

impl JniGen<Function> {
    /// Override the Kotlin-side function name for this [`Self::package_fun`]
    /// entry. Default (without `.name(...)`) is
    /// `snake_to_camel(rust_ident)` (e.g. `z_hello_whatami` → `zHelloWhatami`).
    ///
    /// This is unrelated to the deconstructor ids used by
    /// [`Self::ptr_class_deconstructor`], [`Self::output_named`], and
    /// [`Self::error_named`].
    pub fn name(mut self, kotlin_name: impl Into<String>) -> Self {
        let name = kotlin_name.into();
        let package = self.state.package.clone();
        let index = self.state.index;
        let pkg = self
            .packages
            .get_mut(&package)
            .expect("package entry vanished");
        pkg.functions[index].kotlin_name_override = Some(name);
        self
    }
}

impl JniGen<PtrClass> {
    // ── Canonical type representation (input / output on the ptr_class) ──

    /// Rust type of this [`Self::ptr_class`] state, for the
    /// `.ptr_class_input*` / `.ptr_class_output*` chain.
    fn current_ptr_class(&self) -> syn::Type {
        self.state.key.to_type()
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

    /// Begin a **named alternative** deconstructor for the current
    /// `ptr_class`. The records that follow (`.ptr_class_output*`) append to
    /// this declaration; functions select it via [`Self::output_named`] /
    /// [`Self::error_named`] — the type's unnamed declaration stays the
    /// canonical (auto-applied) one. Each named decomposition gets its own
    /// generated callback interfaces (`<Type><Name>Builder` / `…Handler`).
    /// Declare the canonical records BEFORE any named alternative — record
    /// calls append to the most recent declaration of the type.
    pub fn ptr_class_deconstructor(mut self, name: impl Into<String>) -> Self {
        let t = self.current_ptr_class();
        self.deconstructors.add_deconstructor(t);
        self.deconstructors.add_deconstructor_name(name);
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
    /// includes the result of the accessor previously declared via
    /// [`JniGen::class_accessor`] under the method name `name`, unwrapped per the
    /// return type's own canonical output (one leaf for a scalar/string/enum;
    /// spliced for a nested ptr_class). `name` is also the literal
    /// callback-parameter name for this leaf (emitted verbatim); it must be
    /// unique within the deconstructor and must not contain the reserved `"__"`
    /// separator. For a spliced nested ptr_class it prefixes the child leaf
    /// names (`name__<child>`). The accessor must be declared on this class
    /// before it is referenced here.
    pub fn ptr_class_output(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        let key = self.state.key.clone();
        let func = self
            .class_accessors
            .get(&key)
            .and_then(|accs| accs.iter().find(|a| a.method_name == name))
            .unwrap_or_else(|| {
                panic!(
                    "ptr_class_output(\"{name}\"): no `.class_accessor(.., \"{name}\")` declared on \
                     `{}` — declare the accessor on this class before referencing it as an output \
                     record.",
                    key.as_str()
                )
            })
            .rust_ident
            .clone();
        let t = self.current_ptr_class();
        self.deconstructors.ensure_canonical_deconstructor(t);
        self.deconstructors.add_deconstructor_record(func, name);
        self
    }
}

impl JniGen<Function> {
    // ── Per-function overrides of the canonical representation ──────────

    /// Per-fn: `param` skips the canonical input and takes the raw handle.
    pub fn input_direct(mut self, param: syn::Ident) -> Self {
        let func = self.current_fn_ident();
        self.expansions.add_skip_default_construct(func, param);
        self
    }

    /// Per-fn: the return value skips the canonical output and stays a raw handle.
    pub fn output_direct(mut self) -> Self {
        let func = self.current_fn_ident();
        self.deconstructors.add_skip_default_output(func);
        self
    }

    /// Per-fn: `param` is built from only the named subset of the canonical
    /// input's build-from variants (plus identity if the canonical input has it).
    pub fn input(
        mut self,
        param: syn::Ident,
        funcs: impl IntoIterator<Item = syn::Ident>,
    ) -> Self {
        let func = self.current_fn_ident();
        self.expansions
            .add_construct_subset(func, param, funcs.into_iter().collect());
        self
    }

    /// Per-fn: replace the canonical output with an explicit record list — each
    /// `(accessor, name)` unwrapped per its return type's canonical output, with
    /// `name` the literal leaf/callback-parameter name.
    pub fn output(
        mut self,
        records: impl IntoIterator<Item = (syn::Ident, impl Into<String>)>,
    ) -> Self {
        let func = self.current_fn_ident();
        self.deconstructors.add_output_inline(
            func,
            records
                .into_iter()
                .map(|(f, n)| (f, n.into()))
                .collect(),
        );
        self
    }

    /// Per-fn: decompose the return value with the **named** deconstructor
    /// (declared via [`Self::ptr_class_deconstructor`]) instead of the
    /// canonical one.
    pub fn output_named(mut self, name: impl Into<String>) -> Self {
        let func = self.current_fn_ident();
        self.deconstructors.add_deconstruct_output_with(func, name);
        self
    }

    /// Per-fn: decompose the `Result<_, E>` domain error with the **named**
    /// deconstructor instead of `E`'s canonical one — the `onError` handler
    /// becomes the named decomposition's `<Type><Name>Handler` interface.
    pub fn error_named(mut self, name: impl Into<String>) -> Self {
        let func = self.current_fn_ident();
        self.deconstructors.add_deconstruct_error_with(func, name);
        self
    }

    /// Rust ident of the function the current per-fn override chain targets,
    /// resolved from the live [`Self::package_fun`] state.
    fn current_fn_ident(&self) -> syn::Ident {
        self.packages
            .get(&self.state.package)
            .expect("package entry vanished")
            .functions[self.state.index]
            .rust_ident
            .clone()
    }
}

impl JniGen<PtrClass> {
    /// Opt out of Kotlin typed-handle class emission for this
    /// [`Self::ptr_class`] declaration — the `.kt` file is assumed to be
    /// hand-written.
    pub fn suppress_kotlin_code(mut self) -> Self {
        let key = self.state.key.clone();
        let entry = self.types.get_mut(&key).expect("type entry vanished");
        let opaque = entry
            .opaque
            .as_mut()
            .expect("ptr_class state has no opaque config");
        opaque.suppress_kotlin_code = true;
        self
    }
}

impl JniGen<EnumClass> {
    /// Opt out of Kotlin enum class emission for the most recent
    /// [`Self::enum_class`] declaration.
    pub fn suppress_kotlin_code(mut self) -> Self {
        let key = self.state.key.clone();
        let entry = self.types.get_mut(&key).expect("type entry vanished");
        let enum_cfg = entry
            .enum_cfg
            .as_mut()
            .expect("enum_class state has no enum config");
        enum_cfg.suppress_kotlin_code = true;
        self
    }
}

impl<S> JniGen<S> {
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
}

impl<S: TypeDeclState> JniGen<S> {
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
    pub fn enum_class(mut self, rust_type: syn::Type) -> JniGen<EnumClass> {
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
        self.into_state(EnumClass {
            key,
            package: std::marker::PhantomData,
        })
    }

    /// Declare a Rust struct that should appear in Kotlin as a data
    /// class under a derived name. The name passes through
    /// [`Self::kotlin_data_class_name_mangle`] (default = Rust short
    /// name, generics / lifetimes stripped). Only affects Kotlin
    /// emission — no Rust-side converter override.
    pub fn data_class(mut self, rust_type: syn::Type) -> JniGen<TypeMeta> {
        let key = TypeKey::from_type(&rust_type);
        let short = rust_short_name(&key);
        let fqn = self.resolve_class_fqn(&self.mangle_data_class(&short));
        let entry = self.types.entry(key.clone()).or_default();
        entry.kotlin_name = Some(fqn.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), fqn));
        self.into_state(TypeMeta {
            key,
            package: std::marker::PhantomData,
        })
    }

    /// Declare a **`Copy` value class** type: a Rust type passed across the
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
    pub fn value_class(mut self, rust_type: syn::Type) -> JniGen<TypeMeta> {
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
        self.into_state(TypeMeta {
            key,
            package: std::marker::PhantomData,
        })
    }

}

impl<S: TypeKeyState> JniGen<S> {
    /// Stamp a verbatim Kotlin type expression (e.g. `"List<ByteArray>"`)
    /// onto the entry registered by the most recent type-config builder.
    /// Use this when the Kotlin type is not a class FQN (generics,
    /// primitives, container types). For class names, the per-kind
    /// `kotlin_*_name_mangle` closures (configured on [`JniGen`]) own
    /// derivation — `kotlin_type` is the escape hatch for verbatim
    /// expressions that don't map onto any one element kind.
    pub fn kotlin_type(mut self, kotlin_expr: impl Into<String>) -> Self {
        let key = self.state.type_key().clone();
        let expr = kotlin_expr.into();
        let entry = self.types.get_mut(&key).expect("meta entry vanished");
        entry.kotlin_name = Some(expr.clone());
        self.kotlin_type_fqns.push((key.as_str().to_string(), expr));
        self
    }

    /// Declare a `#[prebindgen]` **read accessor** (`f(&Self) -> R`) as an
    /// **instance method** of the current class (`ptr_class` / `value_class`),
    /// named `method_name`. The receiver parameter (the first `&Self` /
    /// `Self`) is dropped from the Kotlin signature and bound to `this`; the
    /// remaining parameters/return are emitted exactly as a free wrapper would.
    ///
    /// Replaces the former `.package_fun(f).accessor()`: the accessor is the
    /// membership token for the input/output composers — its params are never
    /// input-composed and its return is never output-decomposed — and a
    /// decomposition record selects it via [`JniGen::ptr_class_output`] by its
    /// `method_name` (which doubles as the leaf/parameter name). Declare an
    /// accessor BEFORE any `.ptr_class_output(name)` that references it.
    pub fn class_accessor(mut self, rust_fun: syn::Ident, method_name: impl Into<String>) -> Self {
        let key = self.state.type_key().clone();
        self.class_accessors
            .entry(key)
            .or_default()
            .push(crate::api::lang::jnigen::jni::ClassAccessor {
                rust_ident: rust_fun,
                method_name: method_name.into(),
            });
        self
    }
}

impl<S> JniGen<S> {
    /// Register a rank-N **input converter**. `pattern` contains 0–3
    /// `_` placeholders; the closure's arity selects the rank table.
    /// The closure returns `Some((ty, exc, body))` (see the internal wrapper
    /// function type
    /// for the triple's full semantics) or `None` (defer to a later
    /// resolver phase). The body sees `env: &mut JNIEnv` and `v: &<wire>`
    /// in scope.
    ///
    /// * `exc = None` ⇒ binding-fallible only: `body` evaluates to a bare
    ///   `ty`; framework emits `-> Result<ty, __JniErr>` with an `Ok(...)`
    ///   wrap, and `?` inside propagates the framework error.
    /// * `exc = Some(<Rust type>)` ⇒ domain-fallible: `body` evaluates to
    ///   `Result<ty, <Rust type>>`; framework emits it verbatim. The type
    ///   is the `E` peeled from the source `Result<T, E>`, matched by
    ///   **exact canonical-form equality** (no short-name fallback); a
    ///   failure routes to the wrapper's error sink, never a JVM throw.
    ///
    /// `ty` is auto-classified at resolve: a wire shape ⇒ terminal
    /// converter; a distinct rust type with its own converter ⇒ a
    /// value-inspecting stage composed onto that converter's chain
    /// (resolved by the adapter's wrapper lookup).
    pub fn input_wrapper<A, B>(self, pattern: syn::Type, builder: B) -> JniGen<B::NextState>
    where
        B: WrapperBuilder<A>,
    {
        let key = TypeKey::from_type(&pattern);
        let rank = B::rank();
        let mut s = self;
        s.input_wrappers[rank].insert(key.clone(), builder.into_wrapper_fn());
        let next = B::NextState::from_wrapper(key.clone());
        s.note_wrapper_registration(key, rank);
        s.into_state(next)
    }

    /// Output-direction counterpart of [`Self::input_wrapper`]. Same
    /// closure shape, same `exc = None` / `Some(<Rust type>)` semantics,
    /// same terminal-vs-composed classification — see that method's docs.
    /// (`Some(parse_quote!(<full path>))` with a rust-typed `ty`, e.g.
    /// `(T, Some(parse_quote!(zenoh_flat::errors::ZError)), v)` for
    /// `ZResult<T>`, gives the auto-composed peel that the deleted
    /// `output_throw_stage` used to register.)
    pub fn output_wrapper<A, B>(self, pattern: syn::Type, builder: B) -> JniGen<B::NextState>
    where
        B: WrapperBuilder<A>,
    {
        let key = TypeKey::from_type(&pattern);
        let rank = B::rank();
        let mut s = self;
        s.output_wrappers[rank].insert(key.clone(), builder.into_wrapper_fn());
        let next = B::NextState::from_wrapper(key.clone());
        s.note_wrapper_registration(key, rank);
        s.into_state(next)
    }

    /// Shared post-registration bookkeeping for wrapper inserts. Rank-0
    /// patterns identify a concrete type — auto-stamp `kotlin_name` via
    /// [`Self::mangle_wrapper`] (skipping non-path patterns like `()`
    /// where there is no sensible short name). Rank ≥1 patterns are
    /// wildcards — per-outer-type names come from inner-metadata
    /// propagation via [`Self::override_kotlin_name`].
    fn note_wrapper_registration(&mut self, key: TypeKey, rank: usize) {
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
    /// Structurally match `ty` against every registered **input** wrapper
    /// pattern, most-specific-first (fewest wildcards win, e.g.
    /// `Result<_, ConcreteErr>` over `Result<_, _>`), and build the first hit.
    /// Replaces the rank resolver's per-arity enumeration for the user table.
    pub(crate) fn match_user_input(
        &self,
        ty: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        for pat in self.ordered_input_patterns() {
            if let Some(args) = crate::api::core::types_util::match_pattern(ty, &pat) {
                if let Some(c) = self.lookup_input(&pat, &args, registry) {
                    return Some(c);
                }
            }
        }
        None
    }

    /// Output-direction peer of [`Self::match_user_input`].
    pub(crate) fn match_user_output(
        &self,
        ty: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        for pat in self.ordered_output_patterns() {
            if let Some(args) = crate::api::core::types_util::match_pattern(ty, &pat) {
                if let Some(c) = self.lookup_output(&pat, &args, registry) {
                    return Some(c);
                }
            }
        }
        None
    }

    /// Registered input-wrapper patterns, ordered most-specific (fewest
    /// wildcards) first; ties keep registration-independent but stable order
    /// (by canonical key) so resolution is deterministic.
    fn ordered_input_patterns(&self) -> Vec<syn::Type> {
        ordered_patterns(&self.input_wrappers)
    }
    fn ordered_output_patterns(&self) -> Vec<syn::Type> {
        ordered_patterns(&self.output_wrappers)
    }

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
                    subs: vec![],
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
                    subs: vec![],
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
                    subs: vec![],
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
                    subs: vec![],
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

/// Flatten the rank-bucketed wrapper tables into one pattern list ordered
/// most-specific-first: ascending wildcard count (so `Result<_, ConcreteErr>`
/// is tried before `Result<_, _>`), then by canonical key for a deterministic
/// tiebreak independent of `HashMap` iteration order.
fn ordered_patterns(buckets: &[HashMap<TypeKey, WrapperFn>; 4]) -> Vec<syn::Type> {
    let mut keys: Vec<(usize, String, syn::Type)> = buckets
        .iter()
        .flat_map(|m| m.keys())
        .map(|k| {
            let ty = k.to_type();
            (
                crate::api::core::types_util::wildcard_count(&ty),
                k.as_str().to_string(),
                ty,
            )
        })
        .collect();
    keys.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    keys.into_iter().map(|(_, _, ty)| ty).collect()
}

impl Default for JniGen {
    fn default() -> Self {
        Self::new()
    }
}
