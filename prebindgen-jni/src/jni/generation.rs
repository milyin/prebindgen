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
    /// Registry-compiled recipe fragments, frozen after the last site has been
    /// planned. This is the sole post-resolution source of private converter
    /// artifacts and fragment lookups; the mutable planning store is drained
    /// when this plan is built.
    conversions: prebindgen_registry::recipe::Compiled<crate::jni::compile::JFrag>,
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

/// One final artifact of the generated Rust file.
pub(crate) enum JFinalArtifact {
    /// A private converter, carrying one value across the boundary.
    Converter(Box<crate::jni::chain::JFunction>),
    /// The exported JNI extern for one declared `#[prebindgen]` function.
    Wrapper(Box<crate::jni::emit::JWrapper>),
    /// One declared constant: an alias to the source item, plus the nullary
    /// extern its Kotlin `val` is initialized from.
    Const(Box<JConst>),
}

/// One declared `#[prebindgen]` constant, exported to Kotlin as an eagerly
/// initialized top-level `val`.
///
/// The generated file re-states the constant as a path-alias to its
/// source-of-truth — initializer tokens are never copied, since they may name
/// source-crate internals — and exports a nullary getter extern, which is how
/// the constant's type crosses through the ordinary output-converter
/// machinery.
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
            Self::Const(constant) => prebindgen_registry::write::ArtifactKey::Artifact(
                prebindgen_registry::generation::ArtifactId::new(
                    "jni-const",
                    constant.constant.name.to_string(),
                )
                .expect("a constant name is a non-empty artifact name"),
            ),
        }
    }

    fn reachable(&self) -> bool {
        match self {
            Self::Converter(converter) => converter.should_emit(),
            Self::Wrapper(_) | Self::Const(_) => true,
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
        }
    }
}

impl JniGenerationPlan {
    /// Finish planning and take ownership of every derived memo.
    pub(crate) fn freeze(decls: &mut Declarations, registry: &Registry) -> Self {
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
            let _ = decls.struct_plan(registry, item, 0);
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

        let conversions = std::mem::take(&mut *decls.compiled.borrow_mut());
        let mut assembly = prebindgen_registry::write::AssemblyBuilder::new();
        for converter in conversions
            .fragments()
            .into_iter()
            .filter(|fragment| !fragment.composed_only)
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
        let assembly = assembly.build();
        Self {
            conversions,
            assembly,
            functions: std::mem::take(decls.fn_plans.get_mut()),
            interfaces: std::mem::take(decls.iface_specs.get_mut()),
            structs: std::mem::take(decls.struct_plans.get_mut()),
            sums: std::mem::take(decls.sum_plans.get_mut()),
            vec_builds: std::mem::take(decls.vec_build_plans.get_mut()),
        }
    }

    pub(crate) fn fragment(
        &self,
        ty: &TypeKey,
        direction: prebindgen_registry::recipe::Direction,
    ) -> Option<Rc<crate::jni::compile::JFrag>> {
        self.conversions.fragment(ty, direction)
    }

    pub(crate) fn recipe_fragment(
        &self,
        ty: &TypeKey,
        recipe: &prebindgen_registry::recipe::RecipeKey,
    ) -> Option<Rc<crate::jni::compile::JFrag>> {
        self.conversions.recipe_fragment(ty, recipe)
    }

    #[cfg(test)]
    pub(crate) fn conversions(
        &self,
    ) -> &prebindgen_registry::recipe::Compiled<crate::jni::compile::JFrag> {
        &self.conversions
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
            self.conversions.len(),
            self.functions.len(),
            self.interfaces.len(),
            self.structs.len(),
            self.sums.len(),
            self.vec_builds.len(),
        )
    }
}
