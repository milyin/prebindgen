//! The return side, read off the rows rather than off the decomposition.
//!
//! `unfold` still builds every plan the emitters use. This reads the same
//! decompositions off the `parts` rows the recipe table already carries, and
//! holds the two to each other — a build fails on any difference (#701 step 3).
//!
//! The comparison goes with the older path, once every decomposition this
//! binding declares has been through it. Until then it is the oracle a fixture
//! cannot be: it runs over `covertest-kotlin`, `perftest-kotlin`, `emitcheck`
//! and zenoh-flat-jni, which is the whole declared surface.

use prebindgen_registry::{
    flat::{Flat, ScalarKind, TypeRef},
    fold::{Folding, UnfoldPolicy},
    leaf::{Hoist, LeafSource, UnfoldLeaf},
    recipe::{Reach, RecipeName, Recipes},
};

use super::Declarations;

/// What the JVM calls the values a decomposition delivers.
///
/// Every answer is one the decomposition writes into a leaf today, and the
/// comparison below is what holds them to it.
pub(crate) struct JniUnfold<'a> {
    decls: &'a Declarations,
    /// The registry, for the one name lookup that needs a signature.
    registry: &'a dyn prebindgen_registry::Conversions,
    /// The type whose declaration names these parts. A part name is the
    /// declaration's answer, and a declaration is per type.
    key: prebindgen_registry::TypeKey,
}

impl UnfoldPolicy for JniUnfold<'_> {
    fn selector(&self, source: &TypeRef) -> UnfoldLeaf {
        UnfoldLeaf {
            name: "tag".to_string(),
            path: Vec::new(),
            // The sum, not the `i32` it crosses as: this is how the emitter
            // finds the enum to match on. Registered and not required — a sum
            // has no whole-value output converter (#282).
            out_ty: source.clone(),
            identity: false,
            nullable: false,
            source: LeafSource::SumTag,
            groups: Vec::new(),
        }
    }

    fn presence(&self, name: &str) -> UnfoldLeaf {
        UnfoldLeaf {
            name: format!("{name}__present"),
            path: Vec::new(),
            out_ty: TypeRef::scalar(ScalarKind::Bool),
            identity: false,
            nullable: false,
            source: LeafSource::Presence,
            groups: Vec::new(),
        }
    }

    fn part_name(&self, reach: &Reach, index: usize, field: Option<&syn::Ident>) -> String {
        match reach {
            Reach::Accessor(func) => self.decls.leaf_name_of(&self.key, func, index),
            // A synthesized `data_class` decomposition names its slots after
            // the struct's own fields — as the KOTLIN property is named, which
            // camel-cases and escapes a name that is a keyword there. A
            // declared decomposition names its parts itself.
            _ => match field {
                Some(name) => crate::jni::mangle_kotlin_ident(&crate::jni::kt_snake_to_camel(
                    &name.to_string(),
                )),
                None => self.decls.leaf_name_at(&self.key, index),
            },
        }
    }

    fn arm_part_name(&self, variant: &syn::Ident, member: &syn::Member, _index: usize) -> String {
        // The Kotlin variant class's name and the Kotlin property's, which is
        // what the slot is called on the far side — neither is the Rust name.
        let kotlin = match self.decls.types.get(&self.key).and_then(|c| c.sum()) {
            Some(cfg) => self.decls.sum_variant_class_name(cfg, variant),
            None => variant.to_string(),
        };
        crate::jni::struct_plan::sum_slot_fragment(
            &kotlin,
            &crate::jni::struct_plan::sum_field_prop_name(member),
        )
    }

    fn value_form_part(&self, index: usize) -> Option<String> {
        // A value form's parts are named by the declaration that lists them,
        // which is the one record `field_list` holds for such a declaration.
        self.decls
            .value_form_part_name(self.registry, &self.key, index)
    }

    fn identity_name(&self) -> String {
        // A root identity leaf is `handle`; a nested one takes the name of the
        // part it was reached through, which the view supplies instead.
        "handle".to_string()
    }

    fn nest(&self, outer: &str, inner: &str) -> String {
        format!("{outer}__{inner}")
    }
}

impl Declarations {
    /// Read every decomposition back off its row and hold the older path to it.
    ///
    /// Temporary, and deleted with `unfold::apply`. A decomposition whose row
    /// the view does not read yet is skipped rather than failing: the forms it
    /// refuses are the ones step 3 is still replacing, and the point of this is
    /// what the two paths say about the forms it DOES read.
    pub(crate) fn check_unfold_parity(
        &self,
        model: &Flat,
        registry: &dyn prebindgen_registry::Conversions,
        recipes: &Recipes,
        bindings: &prebindgen_registry::recipe::Bindings,
    ) -> Result<(), String> {
        let folding = Folding::new(recipes, model);
        let unfolded = self.unfolded();
        let mut plans: Vec<(String, &crate::unfold::UnfoldPlan)> = unfolded
            .unfold_plans
            .iter()
            .map(|(f, p)| (format!("`{f}`'s return"), p))
            .chain(
                unfolded
                    .error_plans
                    .iter()
                    .map(|(f, p)| (format!("`{f}`'s error"), p)),
            )
            .chain(
                unfolded
                    .callback_arg_plans
                    .iter()
                    .map(|(k, p)| (format!("the callback argument `{k}`"), p)),
            )
            .collect();
        // Sorted, so a binding with two differences reports the same one every
        // time rather than whichever the hash order reached first.
        plans.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (what, plan) in plans {
            // A whole-element fold takes nothing apart: each element crosses
            // through its own converter, so there is no row and nothing to
            // compare.
            let Some(decon) = &plan.decon else {
                continue;
            };
            // A per-function `.expand_return(...)` states a decomposition of
            // its own, which is a row of its own under a name of its own —
            // the shape step 2 gave the parameter side, and step 3's remaining
            // work here. The type's row does not describe it, so comparing the
            // two would compare two different decompositions.
            if !matches!(decon, prebindgen_registry::leaf::DeconId::Default(_)) {
                continue;
            }
            // A `sealed_class` that ALSO carries an `expand_return!` states two
            // decompositions under one row name: the sum's arms, declared by
            // the class, and the fields, declared by the author. The row table
            // keeps the first and the decomposition used the second, so the two
            // describe different things and comparing them says nothing. One
            // row name for two meanings is step 3's to resolve.
            if self.is_sum(&plan.source) && self.declares_return_expand(&plan.source) {
                continue;
            }
            let policy = JniUnfold {
                decls: self,
                registry,
                key: plan.source.stripped_key(),
            };
            let row = RecipeName::new("parts");
            let (leaves, hoists, coverage) =
                match folding.unfold(&policy, bindings, &plan.source, &row) {
                    Ok(read) => read,
                    // A shape the view does not represent yet — no row states
                    // this decomposition, or it states one in a form still
                    // being replaced. Step 3's remaining work rather than a
                    // disagreement.
                    Err(e) if e.is_not_yet_readable() => continue,
                    // Anything else is a row that is WRONG: two leaves of one
                    // name, two identities, a cycle, an accessor or an
                    // alternative the model does not have. Skipping those would
                    // let a bad row disable the check that found it.
                    Err(e) => return Err(format!("{what} has a row that cannot be read: {e}")),
                };
            // The walk says where it stopped; nothing is inferred from the
            // SIZE of what came back. A measure that skips whenever one side is
            // smaller cannot tell a shape the rows do not state yet from a
            // defect that made the read smaller — and a defect of exactly that
            // kind is what the previous commit fixed, invisible because it made
            // one side smaller.
            //
            // So: an incomplete read is skipped and named, and a complete one
            // is compared in full, with no exemptions.
            if !coverage.is_complete() {
                continue;
            }
            // What THIS adapter has not lowered to a binding yet. The walk read
            // what the rows say; whether the older statement says more is a
            // question about the declarations, which only this side can answer
            // — and answering it here names the exact declaration rather than
            // inferring from what came back.
            if let Some(unlowered) = self.unlowered_parts(model, recipes, &plan.source) {
                let _ = unlowered;
                continue;
            }
            let from_row = describe(&leaves, &hoists);
            let from_decl = describe(&plan.leaves, &plan.hoists);
            if from_row != from_decl {
                return Err(format!(
                    "{what} reads back differently from its row.\nfrom the row:\n{from_row}\n\
                     from the declarations:\n{from_decl}"
                ));
            }
        }
        Ok(())
    }
}

impl Declarations {
    /// Which part of this type's decomposition declaration has no binding yet,
    /// if any.
    ///
    /// `Declarations::bindings` writes a part binding for an accessor part
    /// whose type carries a declaration of its own, and only where that part is
    /// the value itself. Three shapes are left, each waiting on work #701's
    /// step 3 still has: a part reached through an `Option`, which is the
    /// `Optional` part of its decision 3; a part of a value form, whose fields
    /// the row states as the returned struct's; and a value form that CONSUMES
    /// its receiver, whose parts may be handed over rather than read.
    ///
    /// Named here rather than guessed at from the leaves, because this is where
    /// the declaration is.
    fn unlowered_parts(
        &self,
        model: &Flat,
        recipes: &Recipes,
        source: &prebindgen_registry::flat::TypeRef,
    ) -> Option<&'static str> {
        let key = source.stripped_key();
        let decl = self.return_expand_decls.iter().find(|d| *d.key() == key)?;
        if let [crate::jni::LocalField::Fields(form)] = decl.field_list() {
            return match model
                .function(&form.func())
                .and_then(|f| f.params.first())
                .is_some_and(|p| p.ty.borrow_target().is_none())
            {
                true => Some("a consuming value form, whose parts may be handed over"),
                false => Some("a value form, whose parts the row states as its result's fields"),
            };
        }
        for field in decl.field_list() {
            let crate::jni::LocalField::Named(func, _) = field else {
                continue;
            };
            let Some(ret) = model.function(func).map(|f| f.ret.clone()) else {
                continue;
            };
            let core = ret.optional_inner().unwrap_or(&ret);
            let core = core.borrow_target().unwrap_or(core);
            let Some(child) = self
                .return_expand_decls
                .iter()
                .find(|d| *d.key() == core.stripped_key())
            else {
                continue;
            };
            if ret.optional_inner().is_some() {
                return Some("a part reached through an `Option`");
            }
            // A child that hands ITSELF over as one of its parts states an
            // ownership fact the row does not carry: the part is moved or
            // cloned rather than read. `bindings` writes no binding for that,
            // so the row stops at the part where the declaration goes through
            // it and hands the value over as well.
            if child
                .field_list()
                .iter()
                .any(|f| matches!(f, crate::jni::LocalField::SelfField))
            {
                return Some("a part whose own declaration hands the value over");
            }
            // `bindings` writes a binding only where the row it would name
            // exists. A declared type whose `parts` row this table does not
            // state yet is the same unfinished work, seen from the other side.
            let part = prebindgen_registry::recipe::Crossing::new(
                core.clone(),
                prebindgen_registry::recipe::Direction::Deconstruct,
            );
            if recipes
                .key_of(&part.key(), &crate::jni::recipes::parts())
                .is_none()
            {
                return Some("a part whose type states no `parts` row yet");
            }
        }
        None
    }
}

/// One decomposition, flattened far enough that two can be compared.
fn describe(leaves: &[UnfoldLeaf], hoists: &[Hoist]) -> String {
    use std::fmt::Write;
    let path_of = |steps: &[prebindgen_registry::leaf::PathStep]| {
        steps
            .iter()
            .map(|step| {
                format!(
                    "{}{}",
                    step.ident(),
                    if step.is_optional() { "?" } else { "" }
                )
            })
            .collect::<Vec<_>>()
            .join(".")
    };
    let mut out = String::new();
    for leaf in leaves {
        let _ = writeln!(
            out,
            "  {} : {} path=[{}] identity={} nullable={} source={} groups={:?}",
            leaf.name,
            leaf.out_ty.key(),
            path_of(&leaf.path),
            leaf.identity,
            leaf.nullable,
            source_of(&leaf.source),
            leaf.groups
        );
    }
    for hoist in hoists {
        let _ = writeln!(
            out,
            "  hoist [{}] consuming={}",
            path_of(&hoist.prefix),
            hoist.consuming
        );
    }
    out
}

/// How a leaf is reached, for the comparison.
///
/// A nested sum's payload is deliberately not distinguished from a plain reach.
/// The two paths record it differently — `synth_value_struct_leaves` maps a
/// rebased payload wire to `Reach`, dropping the alternative, where the view
/// keeps `VariantField` — and which of the two is right is not this check's
/// question. What the emitter reads off such a leaf is its `groups`, which both
/// paths agree on and this compares.
fn source_of(source: &LeafSource) -> String {
    match source {
        LeafSource::VariantField { .. } | LeafSource::Reach => "reach".to_string(),
        LeafSource::SumTag => "tag".to_string(),
        LeafSource::Presence => "present".to_string(),
    }
}
