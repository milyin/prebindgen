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
        // What the adapter's decompositions asked the output tables for,
        // replayed in the order it asked. Building the plans reads the model
        // alone (see `unfold::Unfolding`), so this is every effect they have on
        // a registry — after `expand::apply`, which is where they were made.
        for requirement in &d.requirements {
            match requirement {
                crate::unfold::Requirement::Output(reading) => self.require_output(reading),
                crate::unfold::Requirement::Reference(reading) => self.reference_output(reading),
                crate::unfold::Requirement::Unrequire(reading) => self.unrequire_output(reading),
            }
        }
        // Every crossing these types make is now covered by a plan, so the
        // scan-time direct converter requirement is stale — and typically
        // unresolvable, since such a type has no destination representation.
        // Drop it both ways; the cell stays, so a converter is still produced
        // if one happens to resolve.
        for key in declared.decompositions.replaces.keys() {
            // The key is what a root flag is stored under, so it goes straight
            // in — no `to_type()` round trip to be re-keyed on the far side.
            self.clear_root(Direction::Construct, key);
            self.clear_root(Direction::Deconstruct, key);
        }
        Ok(())
    }
}
