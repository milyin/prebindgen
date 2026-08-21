//! Bind a finished registry to the generator that filled it.

use super::*;

impl Registry {
    pub(super) fn apply_adapter_plans(
        &mut self,
        declared: &mut Declared,
    ) -> Result<(), WriteRustError> {
        // The set of declared fns drives `.default()` auto-apply: a defaulted
        // constructor/deconstructor is synthesized for every matching declared
        // fn. `accessors` is the `.fun_accessor` subset: excluded from
        // constructor composition and the only fns a decomposer record may
        // reference.
        let d = &mut declared.decompositions;
        if let Some(exp) = &d.expansions {
            crate::expand::apply(
                self,
                exp,
                &declared.functions,
                &declared.accessors,
                &declared.method_receivers,
            )?;
        }
        if let Some(dec) = &d.deconstructors {
            crate::unfold::apply(self, dec, &declared.functions, &declared.accessors)?;
        }
        // Synthesized by-value `data_class` decompositions: the adapter already
        // built the leaves; this wires them into fixed-builder plans.
        if !d.value_structs.is_empty() {
            crate::unfold::apply_value_structs(
                self,
                std::mem::take(&mut d.value_structs),
                &declared.functions,
            )?;
        }
        // The same wiring for a value whose alternatives are chosen at runtime
        // (tag + one leaf group per variant) rather than being a fixed product.
        if !d.sums.is_empty() {
            crate::unfold::apply_sum_returns(
                self,
                std::mem::take(&mut d.sums),
                &declared.functions,
            )?;
        }
        // Single-leaf `Vec<T>`/`&[T]` whole-element folds — the dual of the
        // `data_class` folds above, for String / scalar / handle elements
        // (so the list is built on the foreign side, not via a Rust ArrayList).
        if !d.leaf_vec_elements.is_empty() {
            crate::unfold::apply_leaf_vec_folds(
                self,
                std::mem::take(&mut d.leaf_vec_elements),
                &declared.functions,
            )?;
        }
        // Every crossing these types make is now covered by a plan, so the
        // scan-time direct converter requirement is stale — and typically
        // unresolvable, since such a type has no destination representation.
        // Drop it both ways; the cell stays, so a converter is still produced
        // if one happens to resolve.
        for key in declared.decompositions.replaces.keys() {
            // The key is what a root flag is stored under, so it goes straight
            // in — no `to_type()` round trip to be re-keyed on the far side.
            self.clear_root(Direction::Input, key);
            self.clear_root(Direction::Output, key);
        }
        Ok(())
    }
}
