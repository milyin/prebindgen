//! Immutable JNI generation decisions shared by the Rust and Kotlin writers.
//!
//! Resolution may derive function, interface, and whole-value struct plans in
//! whatever dependency order their converters require. [`JniGenerationPlan`]
//! is the phase boundary: it drains those mutable planning memos once the
//! registry is complete, and every artifact writer subsequently reads the same
//! allocations. A writer cannot create a new ABI, projection, callback
//! descriptor, expansion, error channel, or struct-leaf layout.

use std::rc::Rc;

use super::*;

/// All cross-artifact JNI decisions frozen at the end of resolution.
pub(crate) struct JniGenerationPlan {
    /// The canonical plan: every reached fragment and every site of every
    /// exported function, validated as one set and frozen in dependency order.
    ///
    /// Building it is the production gate: `build` is the only thing that can
    /// see a chain whose endpoints do not join up, an edge naming a fragment
    /// nothing compiled, a duplicate site identity, or a failure route that
    /// does not match its fragment's fallibility — and an invalid plan now
    /// fails the binding rather than one test.
    ///
    /// **Retained only where something reads it.** Keeping a carrier no
    /// emitter consults would be exactly the parallel answer this umbrella
    /// deletes, so today it is held for the freeze check alone; step 5b ungates
    /// it with the first reader that needs it. Fragment lookups stay on
    /// `Declarations::compiled` until then, and 5c deletes that one.
    #[cfg(test)]
    plan: prebindgen_registry::generation::GenerationPlan<crate::jni::compile::JRepresentation>,
    functions: HashMap<syn::Ident, Rc<JniFunctionPlan>>,
    interfaces: BTreeMap<SpecKey, Arc<IfaceSpec>>,
    /// Every declared data class is recorded, including an explicit `None` for
    /// a shape that has no complete whole-value bridge. Recording refusals
    /// makes the writer a lookup rather than a second attempt at
    /// classification.
    structs: HashMap<TypeKey, Option<Rc<StructPlan>>>,
    sums: HashMap<TypeKey, Rc<crate::jni::kotlin_emit::SealedClassPlan>>,
    vec_builds: HashMap<TypeKey, Rc<VecBuildHelpers>>,
    /// Every final artifact of the generated Rust file, frozen in the order it
    /// is written. Built once the last fragment is compiled, and the only
    /// thing `write_rust` reads the file's converters and externs from.
    assembly: prebindgen_registry::write::Assembly<JFinalArtifact>,
}

/// The prelude's identity, which every extern depends on: its body routes
/// failure through the error-channel functions the prelude renders.
pub(crate) fn prelude_key() -> prebindgen_registry::write::ArtifactKey {
    jni_artifact("jni-runtime", "prelude")
}

/// An adapter-scoped artifact identity.
fn jni_artifact(kind: &str, name: impl Into<String>) -> prebindgen_registry::write::ArtifactKey {
    prebindgen_registry::write::ArtifactKey::Artifact(
        prebindgen_registry::generation::ArtifactId::new(kind, name)
            .expect("a JNI artifact name is non-empty"),
    )
}

/// One final artifact of the generated Rust file.
#[derive(Clone)]
pub(crate) enum JFinalArtifact {
    /// A private converter, carrying one value across the boundary.
    Converter(Box<crate::jni::chain::JFunction>),
    /// The exported JNI extern for one declared `#[prebindgen]` function.
    Wrapper(Box<crate::jni::emit::JWrapper>),
    /// One declared constant: an alias to the source item, plus the nullary
    /// extern its Kotlin `val` is initialized from.
    Const(Box<JConst>),
    /// The error-channel prelude every extern body calls into.
    Prelude,
    /// One opaque handle's typed destructor.
    HandleDestructor(Box<JHandleDestructor>),
    /// One element type's `…VecNew/Push/Free` trio.
    VecBuild(Box<JVecBuild>),
    /// One binding-defined constant expression's nullary getter extern.
    ConstantExpr(Box<crate::jni::emit::JWrapper>),
}

/// One opaque handle's typed destructor, and the alignment assertion the
/// handle encoding rests on.
///
/// The source type is retained as a reading and spelled by the writer, since
/// only the writer knows how to qualify it in the generated file.
#[derive(Clone)]
pub(crate) struct JHandleDestructor {
    /// The handle's source type.
    reading: prebindgen_registry::flat::TypeRef,
    /// The exported `freePtr` symbol, which is also this artifact's identity.
    symbol: String,
}

impl JHandleDestructor {
    /// Retain one planned destructor.
    pub(crate) fn new(reading: prebindgen_registry::flat::TypeRef, symbol: String) -> Self {
        Self { reading, symbol }
    }

    /// The exported symbol, which orders the destructors in the file.
    pub(crate) fn symbol(&self) -> &str {
        &self.symbol
    }

    fn render(&self, emit: &prebindgen_registry::RustWriter) -> Vec<syn::Item> {
        let ty = emit.emit_source_type(&self.reading);
        let ident = syn::Ident::new(&self.symbol, Span::call_site());
        vec![
            syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub(crate) unsafe extern "C" fn #ident(
                    _env: jni::JNIEnv,
                    _class: jni::objects::JClass,
                    ptr: jni::sys::jlong,
                ) {
                    if ptr != 0 && (ptr & 1) == 0 {
                        drop(Box::from_raw(ptr as *mut #ty));
                    }
                }
            ),
            // Bit 0 of the jlong is the Kotlin-side closed tag, so every handle
            // type must leave it free: `Box` pointers to `T` are
            // `align_of::<T>()` aligned, hence the compile-time floor of 2.
            // Written after the destructor, where the symbol sort that
            // preceded this artifact put it; an anonymous const asserts from
            // wherever it stands.
            syn::parse_quote!(
                const _: () = {
                    if ::core::mem::align_of::<#ty>() < 2 {
                        panic!(
                            "opaque handle types must have alignment >= 2 (bit 0 is the closed tag)"
                        );
                    }
                };
            ),
        ]
    }
}

/// One element type's `…VecNew/Push/Free` trio: the frozen flatten plan the
/// push leaves come from, and the three exported symbols.
#[derive(Clone)]
pub(crate) struct JVecBuild {
    helpers: Rc<VecBuildHelpers>,
    new_symbol: String,
    push_symbol: String,
    free_symbol: String,
}

impl JVecBuild {
    /// Retain one planned trio.
    pub(crate) fn new(
        helpers: Rc<VecBuildHelpers>,
        new_symbol: String,
        push_symbol: String,
        free_symbol: String,
    ) -> Self {
        Self {
            helpers,
            new_symbol,
            push_symbol,
            free_symbol,
        }
    }

    /// The frozen element flatten plan.
    pub(crate) fn helpers(&self) -> &VecBuildHelpers {
        &self.helpers
    }

    pub(crate) fn new_symbol(&self) -> &str {
        &self.new_symbol
    }

    pub(crate) fn push_symbol(&self) -> &str {
        &self.push_symbol
    }

    pub(crate) fn free_symbol(&self) -> &str {
        &self.free_symbol
    }

    /// The converters `Push` calls, one per leaf of the element it builds.
    pub(crate) fn calls(&self) -> Vec<prebindgen_registry::write::ArtifactKey> {
        self.helpers
            .plan
            .leaves
            .iter()
            .filter(|leaf| !leaf.is_present_flag())
            .map(|leaf| {
                prebindgen_registry::write::ArtifactKey::Operation(
                    leaf.conv()
                        .expect("a non-present leaf has a converter")
                        .clone(),
                )
            })
            .collect()
    }

    /// The first of the three symbols, which orders the trios in the file.
    pub(crate) fn sort_key(&self) -> &str {
        [&self.free_symbol, &self.new_symbol, &self.push_symbol]
            .into_iter()
            .min()
            .expect("three symbols")
    }
}

/// One declared `#[prebindgen]` constant, exported to Kotlin as an eagerly
/// initialized top-level `val`.
///
/// The generated file re-states the constant as a path-alias to its
/// source-of-truth — initializer tokens are never copied, since they may name
/// source-crate internals — and exports a nullary getter extern, which is how
/// the constant's type crosses through the ordinary output-converter
/// machinery.
#[derive(Clone)]
pub(crate) struct JConst {
    /// The constant as the model holds it.
    constant: prebindgen_registry::flat::Constant,
    /// Module the source constant is reached through.
    source_module: syn::Path,
    /// The getter extern, planned like any other.
    getter: crate::jni::emit::JWrapper,
}

impl JConst {
    /// Plan the alias and getter for one declared constant.
    pub(crate) fn new(
        decls: &Declarations,
        registry: &Registry,
        constant: &prebindgen_registry::flat::Constant,
    ) -> Self {
        crate::jni::reject_handle_const(decls, constant);
        let ident = &constant.name;
        let source_module = decls.fn_module(registry, ident);
        let callee: syn::Expr = syn::parse_quote!(#source_module::#ident);
        let getter = crate::jni::emit::JWrapper::new(
            decls,
            registry,
            &crate::jni::const_getter_fn(constant),
            Some(callee),
        );
        Self {
            constant: constant.clone(),
            source_module,
            getter,
        }
    }
}

impl prebindgen_registry::write::RustArtifact for JFinalArtifact {
    fn key(&self) -> prebindgen_registry::write::ArtifactKey {
        match self {
            Self::Converter(converter) => converter.key(),
            Self::Wrapper(wrapper) => prebindgen_registry::write::ArtifactKey::Artifact(
                prebindgen_registry::generation::ArtifactId::new("jni-wrapper", wrapper.symbol())
                    .expect("an exported symbol is a non-empty artifact name"),
            ),
            Self::Const(constant) => jni_artifact("jni-const", constant.constant.name.to_string()),
            Self::Prelude => prelude_key(),
            Self::HandleDestructor(destructor) => {
                jni_artifact("jni-handle-destructor", destructor.symbol())
            }
            Self::VecBuild(helpers) => jni_artifact("jni-vec-build", helpers.sort_key()),
            Self::ConstantExpr(getter) => jni_artifact("jni-constant-expr", getter.symbol()),
        }
    }

    fn reachable(&self) -> bool {
        match self {
            Self::Converter(converter) => converter.should_emit(),
            Self::Wrapper(_)
            | Self::Const(_)
            | Self::Prelude
            | Self::HandleDestructor(_)
            | Self::VecBuild(_)
            | Self::ConstantExpr(_) => true,
        }
    }

    fn provides(&self) -> Vec<prebindgen_registry::write::ArtifactKey> {
        // Each JNI artifact renders its own items and no one else's: unlike
        // the C callback, nothing here stands in for an identity whose own
        // artifact was never planned. Stated rather than inherited, so that an
        // artifact which does needs a deliberate change here.
        vec![self.key()]
    }

    fn calls(&self) -> Vec<prebindgen_registry::write::ArtifactKey> {
        match self {
            Self::Converter(converter) => converter.calls(),
            Self::Wrapper(wrapper) | Self::ConstantExpr(wrapper) => wrapper.calls(),
            Self::Const(constant) => constant.getter.calls(),
            Self::VecBuild(helpers) => helpers.calls(),
            // The prelude is self-contained, and a destructor drops a box.
            Self::Prelude | Self::HandleDestructor(_) => Vec::new(),
        }
    }

    fn render(&self, emit: &prebindgen_registry::RustWriter) -> Vec<syn::Item> {
        match self {
            Self::Converter(converter) => vec![syn::Item::Fn(converter.render_fn(emit))],
            Self::Wrapper(wrapper) => vec![syn::Item::Fn(wrapper.render_fn(emit))],
            Self::Const(constant) => vec![
                syn::Item::Const(emit.const_alias(&constant.constant, &constant.source_module)),
                syn::Item::Fn(constant.getter.render_fn(emit)),
            ],
            Self::Prelude => crate::jni::trait_impl::render_prelude(),
            Self::HandleDestructor(destructor) => destructor.render(emit),
            Self::VecBuild(helpers) => crate::jni::emit::render_vec_build_helpers(helpers, emit),
            Self::ConstantExpr(getter) => vec![syn::Item::Fn(getter.render_fn(emit))],
        }
    }
}

impl JniGenerationPlan {
    /// Finish planning and take ownership of every derived memo.
    pub(crate) fn freeze(
        decls: &mut Declarations,
        registry: &Registry,
    ) -> Result<Self, prebindgen_registry::WriteRustError> {
        assert!(
            decls.generation.is_none(),
            "JNI generation plan may be frozen only once"
        );

        // Whole-value struct plans used to be the remaining independently
        // rebuilt decision: Rust encoding and Kotlin data/fromParts emission
        // each called `build_struct_plan`. Populate one answer for every
        // declared data class before the mutable planning store is drained.
        // Planning every source struct would activate converters for types the
        // adapter never declared, changing otherwise unrelated generated Rust.
        // The panic-backed frozen lookup is sound only while this filter
        // matches every top-level struct key a writer can request. Nested
        // layouts are embedded in their parent's `StructPlan`; a nested type
        // that is independently declared is also enumerated here.
        let data_classes: Vec<_> = decls
            .types
            .iter()
            .filter(|(_, cfg)| !cfg.special_decl() && cfg.name_spec.is_some())
            .filter_map(|(key, _)| {
                let ident = key.ident()?;
                registry.flat().struct_type(&ident)
            })
            .cloned()
            .collect();
        for item in &data_classes {
            let _ = decls.struct_plan(registry.flat(), item, 0);
        }

        let sealed_classes: Vec<_> = registry
            .flat()
            .types()
            .filter_map(|ty| match ty {
                prebindgen_registry::flat::Type::Variant(item)
                    if decls
                        .types
                        .get(&item.type_ref().key())
                        .is_some_and(|cfg| cfg.sum().is_some()) =>
                {
                    Some(item.clone())
                }
                _ => None,
            })
            .collect();
        // As with data classes, every writer-visible sealed-class key must be
        // covered here before lookup becomes panic-backed after the freeze.
        for item in &sealed_classes {
            let _ = decls.sealed_class_plan(registry, item);
        }

        // Externs are planned before the mutable planning store is drained,
        // since planning one reads the function plan it exports. They are
        // named in source order, so the file's layout does not depend on how
        // the declarations were written.
        let declared = decls.declared_functions();
        let mut exported: Vec<_> = registry
            .flat()
            .functions()
            .filter(|function| declared.contains(&function.name))
            .cloned()
            .collect();
        exported.sort_by_key(|function| function.name.to_string());
        let wrappers: Vec<_> = exported
            .iter()
            .map(|function| crate::jni::emit::JWrapper::new(decls, registry, function, None))
            .collect();
        // Declared constants, in source order. Undeclared ones are not
        // exported: this binding has a constant declaration mechanism, so it
        // emits exactly what the packages named.
        let declared_consts = decls.declared_consts().unwrap_or_default();
        let mut constants: Vec<_> = registry
            .flat()
            .constants()
            .filter(|constant| declared_consts.contains(&constant.name))
            .cloned()
            .collect();
        constants.sort_by_key(|constant| constant.name.to_string());
        let constants: Vec<_> = constants
            .iter()
            .map(|constant| JConst::new(decls, registry, constant))
            .collect();

        // What the extern bodies call by bare name, then the handle
        // destructors, the Vec-building helpers and the constant-expression
        // getters — the file opens with these, as it did when they were
        // prerequisites.
        let destructors = crate::jni::trait_impl::plan_handle_destructors(decls, registry);
        let vec_builds = crate::jni::emit::plan_vec_build_helpers(decls);
        let constant_exprs = crate::jni::trait_impl::plan_constant_expressions(decls, registry);

        // Borrowed, not drained. The planning store stays where the compiler
        // left it so one lookup path serves both phases — the branch on
        // "already frozen?" that used to decide between them was the thing two
        // stores made necessary (#613 step 5a).
        let conversions = decls.compiled.borrow();
        // The canonical plan, validated as one set. Every fragment the
        // compilation reached and every site the exported functions state.
        let mut collected = prebindgen_registry::generation::GenerationPlanBuilder::<
            crate::jni::compile::JRepresentation,
        >::new();
        for fragment in conversions.fragments() {
            collected.fragment(fragment.freeze());
        }
        for site in decls.site_plans.borrow().iter() {
            collected.site((**site).clone());
        }
        // Root the plan at what the file actually renders, not at sites alone.
        // Sites are the boundary positions; the converters a wrapper or helper
        // reaches are named only by that artifact's own `calls`, so without
        // these the plan prunes fragments the assembly still emits — which is
        // what made `plan.fragments()` unusable as the assembly's order
        // (#613 step 8). C already does exactly this.
        // Every operation a fragment renders, not just its wire-facing one: a
        // fragment emits its converter, its stages and one marker per chain
        // step, and an artifact may call any of them.
        let mut by_operation = std::collections::HashMap::new();
        for fragment in conversions.fragments() {
            for converter in fragment.converter_artifacts() {
                if let prebindgen_registry::write::ArtifactKey::Operation(operation) =
                    converter.key()
                {
                    by_operation
                        .entry(operation)
                        .or_insert_with(Vec::new)
                        .push(fragment.id.clone());
                }
            }
        }
        use prebindgen_registry::write::RustArtifact as _;
        // A borrow delegates to the value it borrows, and says so in
        // `ConverterImpl::subs` — the census's canonical answer for `subs` is
        // "FragmentUse edges inside ShapePlan", which an atomic borrow fragment
        // has nowhere to put. Until it does, the delegation is read from where
        // the adapter already states it, so the plan roots the owned converter
        // a borrowed one calls (#613 step 8).
        let mut declared_surface: Vec<prebindgen_registry::generation::FragmentId> = Vec::new();
        let mut delegations: std::collections::HashMap<_, Vec<_>> =
            std::collections::HashMap::new();
        for fragment in conversions.fragments() {
            for sub in &fragment.conv.subs {
                if let Some(target) = conversions.fragment(sub, fragment.id.direction()) {
                    delegations
                        .entry(fragment.id.clone())
                        .or_default()
                        .push(target.id.clone());
                }
            }
        }
        let root = |artifact: &JFinalArtifact,
                    extra: &[prebindgen_registry::generation::FragmentId]| {
            let calls = artifact.calls();
            let inputs: Vec<_> = calls
                .iter()
                .filter_map(|call| match call {
                    prebindgen_registry::write::ArtifactKey::Operation(operation) => {
                        by_operation.get(operation).cloned()
                    }
                    prebindgen_registry::write::ArtifactKey::Artifact(_) => None,
                })
                .flatten()
                .chain(extra.iter().cloned())
                .map(prebindgen_registry::generation::ArtifactInput::Fragment)
                .collect();
            if inputs.is_empty() {
                return None;
            }
            let key = artifact.key();
            let id = match key {
                prebindgen_registry::write::ArtifactKey::Artifact(id) => id,
                prebindgen_registry::write::ArtifactKey::Operation(operation) => {
                    prebindgen_registry::generation::ArtifactId::new(
                        "jni-operation",
                        operation.to_string(),
                    )
                    .expect("an operation identity is a non-empty artifact name")
                }
            };
            Some(prebindgen_registry::generation::ArtifactPlan::<
                crate::jni::compile::JRepresentation,
            >::new(id, Vec::new(), inputs, artifact.clone()))
        };
        // A declared class is binding surface in BOTH directions, whether or
        // not this build happens to export a function that returns it — the
        // same reason C roots its plan at `opaque`/`data`/`enum` declarations
        // rather than at call sites alone. Without this a data class's output
        // converter is reachable only by accident (#613 step 8).
        for key in decls.types.keys() {
            for direction in [
                prebindgen_registry::recipe::Direction::Construct,
                prebindgen_registry::recipe::Direction::Deconstruct,
            ] {
                if let Some(fragment) = conversions.fragment(key, direction) {
                    declared_surface.push(fragment.id.clone());
                }
            }
        }
        let mut seen = std::collections::HashSet::new();
        for artifact in std::iter::once(JFinalArtifact::Prelude)
            .chain(
                destructors
                    .iter()
                    .map(|d| JFinalArtifact::HandleDestructor(Box::new(d.clone()))),
            )
            .chain(
                vec_builds
                    .iter()
                    .map(|h| JFinalArtifact::VecBuild(Box::new(h.clone()))),
            )
            .chain(
                constant_exprs
                    .iter()
                    .map(|g| JFinalArtifact::ConstantExpr(Box::new(g.clone()))),
            )
            .chain(
                wrappers
                    .iter()
                    .map(|w| JFinalArtifact::Wrapper(Box::new(w.clone()))),
            )
            .chain(
                constants
                    .iter()
                    .map(|c| JFinalArtifact::Const(Box::new(c.clone()))),
            )
            // Converters call converters: a `whole` struct converter calls its
            // fields', and a callback's calls what its body converts through.
            // Those edges are not in the shape, so the converter states them.
            .chain(
                conversions
                    .fragments()
                    .into_iter()
                    // This roots the plan, so it cannot ask the plan. It asks
                    // the same statement the plan is built from: a fragment
                    // that freezes without an artifact renders nothing.
                    .filter(|fragment| fragment.freeze().artifact().is_some())
                    .flat_map(crate::jni::compile::JFrag::converter_artifacts)
                    .map(|converter| JFinalArtifact::Converter(Box::new(converter))),
            )
        {
            // A converter artifact carries its owner's delegations; the
            // exported artifacts above delegate through their calls alone.
            let extra: Vec<_> = match artifact.key() {
                prebindgen_registry::write::ArtifactKey::Operation(operation) => by_operation
                    .get(&operation)
                    .into_iter()
                    .flatten()
                    .filter_map(|owner| delegations.get(owner))
                    .flatten()
                    .cloned()
                    .collect(),
                prebindgen_registry::write::ArtifactKey::Artifact(_) => Vec::new(),
            };
            if let Some(rooted) = root(&artifact, &extra) {
                // Shared converters are reached from several fragments and
                // named once in the file; the plan names them once too.
                if seen.insert(rooted.id().clone()) {
                    collected.artifact(rooted);
                }
            }
        }
        if !declared_surface.is_empty() {
            declared_surface.sort_by_cached_key(|id| format!("{id:?}"));
            declared_surface.dedup();
            collected.artifact(prebindgen_registry::generation::ArtifactPlan::<
                crate::jni::compile::JRepresentation,
            >::new(
                prebindgen_registry::generation::ArtifactId::new("jni-declared-surface", "classes")
                    .expect("a constant artifact name is non-empty"),
                Vec::new(),
                declared_surface
                    .iter()
                    .cloned()
                    .map(prebindgen_registry::generation::ArtifactInput::Fragment)
                    .collect(),
                JFinalArtifact::Prelude,
            ));
        }
        #[cfg_attr(not(test), allow(unused_variables))]
        let plan = collected.build().map_err(|errors| {
            prebindgen_registry::ScanError::AdapterInvariant {
                message: format!("the JNI generation plan is not valid: {errors}"),
            }
        })?;
        let mut assembly = prebindgen_registry::write::AssemblyBuilder::new();
        assembly.artifact(JFinalArtifact::Prelude);
        for destructor in destructors {
            assembly.artifact(JFinalArtifact::HandleDestructor(Box::new(destructor)));
        }
        for helpers in vec_builds {
            assembly.artifact(JFinalArtifact::VecBuild(Box::new(helpers)));
        }
        for getter in constant_exprs {
            assembly.artifact(JFinalArtifact::ConstantExpr(Box::new(getter)));
        }
        // "Reached, and renders something" — both from the plan. A fragment
        // that freezes without an artifact is the canonical statement of
        // composed-only, and `JniGenerationPlan::freeze` already asserts the
        // two agree, so asking the plan removes the second source rather than
        // trusting it (#613 step 5c).
        let renders: std::collections::HashSet<_> = plan
            .fragments()
            .filter(|fragment| fragment.artifact().is_some())
            .map(|fragment| fragment.id().clone())
            .collect();
        for converter in conversions
            .fragments()
            .into_iter()
            .filter(|fragment| renders.contains(&fragment.id))
            .flat_map(crate::jni::compile::JFrag::converter_artifacts)
        {
            assembly.artifact(JFinalArtifact::Converter(Box::new(converter)));
        }
        for wrapper in wrappers {
            assembly.artifact(JFinalArtifact::Wrapper(Box::new(wrapper)));
        }
        for constant in constants {
            assembly.artifact(JFinalArtifact::Const(Box::new(constant)));
        }
        // JniGen qualifies each source reference by the item's own origin
        // module, so it declares no single one for the writer to fall back on.
        let assembly = assembly.build(registry, None);
        drop(conversions);
        Ok(Self {
            #[cfg(test)]
            plan,
            assembly,
            functions: std::mem::take(decls.fn_plans.get_mut()),
            interfaces: std::mem::take(decls.iface_specs.get_mut()),
            structs: std::mem::take(decls.struct_plans.get_mut()),
            sums: std::mem::take(decls.sum_plans.get_mut()),
            vec_builds: std::mem::take(decls.vec_build_plans.get_mut()),
        })
    }

    /// The canonical plan every fragment and site was validated into.
    #[cfg(test)]
    pub(crate) fn plan(
        &self,
    ) -> &prebindgen_registry::generation::GenerationPlan<crate::jni::compile::JRepresentation>
    {
        &self.plan
    }

    /// The frozen assembly the generated Rust file is written from.
    pub(crate) fn assembly(&self) -> &prebindgen_registry::write::Assembly<JFinalArtifact> {
        &self.assembly
    }

    pub(crate) fn function(&self, ident: &syn::Ident) -> Option<Rc<JniFunctionPlan>> {
        self.functions.get(ident).cloned()
    }

    pub(crate) fn functions(&self) -> Vec<Rc<JniFunctionPlan>> {
        let mut plans: Vec<_> = self.functions.iter().collect();
        plans.sort_by_key(|(ident, _)| (*ident).clone());
        plans.into_iter().map(|(_, plan)| plan.clone()).collect()
    }

    pub(crate) fn interface(&self, key: &SpecKey) -> Option<Arc<IfaceSpec>> {
        self.interfaces.get(key).cloned()
    }

    pub(crate) fn struct_plan(&self, key: &TypeKey) -> Option<Rc<StructPlan>> {
        self.structs
            .get(key)
            .unwrap_or_else(|| panic!("frozen JNI plan has no struct entry for `{key}`"))
            .clone()
    }

    pub(crate) fn sealed_class_plan(
        &self,
        key: &TypeKey,
    ) -> Rc<crate::jni::kotlin_emit::SealedClassPlan> {
        self.sums
            .get(key)
            .unwrap_or_else(|| panic!("frozen JNI plan has no sealed-class entry for `{key}`"))
            .clone()
    }

    pub(crate) fn vec_builds(&self) -> Vec<Rc<VecBuildHelpers>> {
        let mut plans: Vec<_> = self.vec_builds.values().cloned().collect();
        plans.sort_by(|a, b| a.elem.key().as_str().cmp(b.elem.key().as_str()));
        plans
    }

    #[cfg(test)]
    pub(crate) fn counts(&self) -> (usize, usize, usize, usize, usize, usize) {
        (
            self.plan.fragments().len(),
            self.functions.len(),
            self.interfaces.len(),
            self.structs.len(),
            self.sums.len(),
            self.vec_builds.len(),
        )
    }
}
