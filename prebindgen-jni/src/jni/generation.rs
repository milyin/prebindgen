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
    functions: HashMap<syn::Ident, Rc<JniFunctionPlan>>,
    interfaces: BTreeMap<SpecKey, Arc<IfaceSpec>>,
    /// Every declared data class is recorded, including an explicit `None` for
    /// a shape that has no complete whole-value bridge. Recording refusals
    /// makes the writer a lookup rather than a second attempt at
    /// classification.
    structs: HashMap<TypeKey, Option<Rc<StructPlan>>>,
    sums: HashMap<TypeKey, Rc<crate::jni::kotlin_emit::SealedClassPlan>>,
    vec_builds: HashMap<TypeKey, Rc<VecBuildHelpers>>,
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

        Self {
            functions: std::mem::take(decls.fn_plans.get_mut()),
            interfaces: std::mem::take(decls.iface_specs.get_mut()),
            structs: std::mem::take(decls.struct_plans.get_mut()),
            sums: std::mem::take(decls.sum_plans.get_mut()),
            vec_builds: std::mem::take(decls.vec_build_plans.get_mut()),
        }
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
    pub(crate) fn counts(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.functions.len(),
            self.interfaces.len(),
            self.structs.len(),
            self.sums.len(),
            self.vec_builds.len(),
        )
    }
}
