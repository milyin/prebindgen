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
        // replayed in the order it asked. Planning how a value comes apart reads
        // the model alone, so this is every effect those plans have on a
        // registry — after `expand::apply`, which is where the asks were made.
        for requirement in &d.requirements {
            match requirement {
                Requirement::Output(reading) => self.require_output(reading),
                Requirement::Reference(reading) => self.reference_output(reading),
                Requirement::Unrequire(reading) => self.unrequire_output(reading),
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

/// Every site an exported function has, each with the plan the adapter made of
/// it — or the first refusal, which ends the walk.
/// What the site walk produced.
///
/// A refusal does not end the walk. One site that will not compile says nothing
/// about the next, and an adapter that reports per function — naming the
/// parameter rather than the position — needs the sites after it.
pub struct Sited<C: crate::recipe::Compile> {
    /// Each site, with the plan the adapter made of it.
    pub plans: Vec<(crate::recipe::Site, C::Plan)>,
    /// The sites that refused, in the order they were reached.
    pub refusals: Vec<(crate::recipe::Site, crate::recipe::CompileError<C::Error>)>,
}

impl Registry {
    /// The `#[prebindgen]` functions this binding exports.
    pub fn exports(&self) -> &std::collections::HashSet<syn::Ident> {
        &self.declared.functions
    }

    /// Compile every site of every exported function.
    ///
    /// A **site** is one position at which a value crosses: a parameter, one
    /// argument of a callback parameter, a return, or the error arm of a
    /// fallible one. Which positions exist is the model's answer and the same
    /// whatever the target, so the registry enumerates them rather than each
    /// adapter writing the same walk.
    ///
    /// The order is fixed — by function name, then by position — so a generated
    /// file's layout does not depend on how the declarations were written. Each
    /// plan is paired with the site it answers, because an adapter that groups
    /// them, as a C callback groups its arguments, needs to know which is which.
    ///
    /// A site the bindings omitted contributes no plan. So does a `()` return:
    /// there is no value there to cross.
    pub fn compile_sites<C: crate::recipe::Compile>(
        &self,
        adapter: &mut C,
        recipes: &crate::recipe::Recipes,
        bindings: &crate::recipe::Bindings,
        compiled: crate::recipe::Compiled<C::Fragment>,
    ) -> (Sited<C>, crate::recipe::Compiled<C::Fragment>) {
        use crate::recipe::{Compiler, Crossing, Direction, Role, Site};

        let mut compiler = Compiler::resume(self, recipes, bindings, compiled);
        let mut names: Vec<&syn::Ident> = self.exports().iter().collect();
        names.sort_by_key(|name| name.to_string());
        let mut plans = Vec::new();
        let mut refusals = Vec::new();
        {
            for name in names {
                let Some(function) = self.flat().function(name) else {
                    continue;
                };
                for (index, param) in function.params.iter().enumerate() {
                    // A parameter that expands is not one site: its leaves are,
                    // and each is a position the function does not itself have.
                    if let Some(fold) = self
                        .expansion_plans()
                        .get(&(name.clone(), param.name.clone()))
                    {
                        for (leaf, of) in fold.leaves.iter().enumerate() {
                            let site = Site {
                                owner: name.clone(),
                                role: Role::ExpansionLeaf { param: index, leaf },
                            };
                            let crossing = bindings.crossing_of(&site).unwrap_or_else(|| {
                                Crossing::new(of.ty.clone(), Direction::Construct)
                            });
                            // A position the adapter has nothing to say about is not
                            // a site of its binding, and not a failure either.
                            if !adapter.plans_site(&site, &crossing) {
                                continue;
                            }
                            match compiler.site(adapter, site.clone(), crossing) {
                                Ok(Some(plan)) => plans.push((site, plan)),
                                Ok(None) => {}
                                Err(e) => refusals.push((site, e)),
                            }
                        }
                        continue;
                    }
                    let site = Site {
                        owner: name.clone(),
                        role: Role::Param { index },
                    };
                    let crossing = bindings
                        .crossing_of(&site)
                        .unwrap_or_else(|| Crossing::new(param.ty.clone(), Direction::Construct));
                    // A position the adapter has nothing to say about is not
                    // a site of its binding, and not a failure either. It does
                    // not prune the positions **inside** it: an adapter that
                    // answers a callback parameter whole still crosses a value
                    // at each of that callback's arguments, so declining the
                    // parameter must not take its arguments with it.
                    if adapter.plans_site(&site, &crossing) {
                        match compiler.site(adapter, site.clone(), crossing) {
                            Ok(Some(plan)) => plans.push((site, plan)),
                            Ok(None) => {}
                            Err(e) => refusals.push((site, e)),
                        }
                    }
                    // A callback parameter's arguments cross the other way:
                    // Rust holds them and pushes them out through the call.
                    let Some(args) = param.ty.callback_args() else {
                        continue;
                    };
                    for (arg, ty) in args.iter().enumerate() {
                        let site = Site {
                            owner: name.clone(),
                            role: Role::CallbackArg { param: index, arg },
                        };
                        let crossing = bindings
                            .crossing_of(&site)
                            .unwrap_or_else(|| Crossing::new(ty.clone(), Direction::Deconstruct));
                        // A position the adapter has nothing to say about is not
                        // a site of its binding, and not a failure either.
                        if !adapter.plans_site(&site, &crossing) {
                            continue;
                        }
                        match compiler.site(adapter, site.clone(), crossing) {
                            Ok(Some(plan)) => plans.push((site, plan)),
                            Ok(None) => {}
                            Err(e) => refusals.push((site, e)),
                        }
                    }
                }
                // The return, crossing what the model says it does — a `Result`
                // return crosses whole, and a binding that crosses something
                // else there says so through its `Bindings`.
                //
                // A fallible return also has an **error arm**, which is a
                // position in the model rather than a channel some target
                // invented: a `Result` has two arms whatever reads it. Whether a
                // binding crosses anything there is its own answer, which
                // `plans_site` gives — JniGen throws the error and declines,
                // `prebindgen-c` hands it back and plans it.
                let mut returns: Vec<(Role, &crate::flat::TypeRef)> =
                    vec![(Role::Return, &function.ret)];
                if let Some((_, err)) = function.ret.fallible_parts() {
                    returns.push((Role::Error, err));
                }
                // A `()` return is still a position. Whether a target has
                // anything to hand back there is the adapter's answer, which
                // `plans_site` gives below.
                for (role, ty) in returns {
                    let site = Site {
                        owner: name.clone(),
                        role,
                    };
                    let crossing = bindings
                        .crossing_of(&site)
                        .unwrap_or_else(|| Crossing::new(ty.clone(), Direction::Deconstruct));
                    // A position the adapter has nothing to say about is not
                    // a site of its binding, and not a failure either.
                    if !adapter.plans_site(&site, &crossing) {
                        continue;
                    }
                    match compiler.site(adapter, site.clone(), crossing) {
                        Ok(Some(plan)) => plans.push((site, plan)),
                        Ok(None) => {}
                        Err(e) => refusals.push((site, e)),
                    }
                }
            }
        }
        (Sited { plans, refusals }, compiler.finish())
    }
}
