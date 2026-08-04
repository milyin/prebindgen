//! Hand the demand over, and grade the answers.
//!
//! The two halves of the exchange with a generator: [`Registry::crossings`]
//! sorts the derived set inner-first, `RegistryBuilder::build` takes every
//! conversion back at once and names whatever is missing.

use std::collections::HashSet;

use super::*;

impl<M> Registry<M> {
    /// Every crossing this binding needs a conversion for, **inner types
    /// first**.
    ///
    /// The order is the whole point: a generator walking this list has already
    /// built everything a given crossing can compose from, so it can work from
    /// a flat list instead of being called back per type. Derived from
    /// [`Self::immediate_edges`], which is structural — generic arguments,
    /// tuple/reference/slice targets, declared struct fields, and `impl Fn`
    /// arguments with the direction flipped — so no generator is consulted to
    /// produce it.
    ///
    /// **Cycles.** A self-referential type (`struct Node { next:
    /// Option<Box<Node>> }`) has no topological order. The walk breaks such a
    /// cycle at its entry, so exactly one member is handed out before an inner
    /// it contains; a generator that cannot build it supplies nothing, and
    /// `RegistryBuilder::build` reports it like any other gap.
    pub(crate) fn crossings(&self) -> Vec<Crossing> {
        // Post-order DFS: a node is emitted only after everything it reaches,
        // which IS inner-first. `visiting` breaks cycles — the back edge is
        // simply not followed, so the node it points at lands later than its
        // dependent, and that is the one documented exception above.
        let mut order: Vec<Crossing> = Vec::new();
        let mut done: HashSet<Crossing> = HashSet::new();
        let mut visiting: HashSet<Crossing> = HashSet::new();

        // Deterministic roots: same list every build, so a generator's output
        // cannot depend on hash order.
        let mut roots: Vec<Crossing> = Vec::new();
        for dir in [Direction::Input, Direction::Output] {
            let mut keys: Vec<&TypeKey> = self.type_table(dir).keys().collect();
            keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
            roots.extend(keys.into_iter().map(|k| (dir, k.clone())));
        }

        for root in roots {
            self.visit_crossing(root, &mut order, &mut done, &mut visiting);
        }
        order
    }

    pub(super) fn visit_crossing(
        &self,
        node: Crossing,
        order: &mut Vec<Crossing>,
        done: &mut HashSet<Crossing>,
        visiting: &mut HashSet<Crossing>,
    ) {
        if done.contains(&node) || !visiting.insert(node.clone()) {
            return;
        }
        let (dir, key) = node.clone();
        // Every node this walk reaches has a cell: the roots are the table's own
        // keys, and each edge below is filtered by `contains_key`. So the
        // spelling `plan_edges` needs is the reading the registry already
        // stored — not one re-derived from the key (#291).
        let plan_edges = self
            .type_table(dir)
            .get(&key)
            .map(|cell| self.plan_edges(dir, cell.subject.as_syn()))
            .unwrap_or_default();
        let mut edges: Vec<Crossing> = self
            .immediate_edges(dir, &key)
            .into_iter()
            // The structural edges arrive as readings, so the key is the
            // model's own answer rather than one re-derived from a spelling.
            .map(|(d, t)| (d, t.key()))
            .chain(
                plan_edges
                    .into_iter()
                    .map(|(d, t)| (d, TypeKey::from_type(&t))),
            )
            .chain(
                self.declared
                    .edges
                    .iter()
                    .filter(|(from, _)| *from == node)
                    .map(|(_, on)| (on.0, on.1.clone())),
            )
            // Only crossings the scan actually registered: a structural edge to
            // a type nothing asked for is not a crossing.
            .filter(|c| self.type_table(c.0).contains_key(&c.1))
            .collect();
        edges.sort_by(|a, b| (a.0 as u8, a.1.as_str()).cmp(&(b.0 as u8, b.1.as_str())));
        for edge in edges {
            self.visit_crossing(edge, order, done, visiting);
        }
        visiting.remove(&node);
        if done.insert(node.clone()) {
            order.push(node);
        }
    }

    /// Dependencies a **decomposition** adds, which the structural walk cannot
    /// see.
    ///
    /// A callback argument delivered as leaves needs each leaf's own conversion
    /// before the callback's can be built — and a leaf is named by a plan, not
    /// by the argument's syntax. Without this the order would be structurally
    /// correct and still wrong, which is exactly the kind of gap the old
    /// fixed-point loop papered over by retrying.
    pub(super) fn plan_edges(&self, dir: Direction, ty: &syn::Type) -> Vec<(Direction, syn::Type)> {
        if dir != Direction::Input {
            return Vec::new();
        }
        let Some(args) = crate::api::core::flat::extract_fn_trait_args(ty) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for arg in args {
            if let Some(plan) = self.callback_arg_plans.get(&TypeKey::from_type(&arg)) {
                for leaf in &plan.leaves {
                    out.push((Direction::Output, leaf.out_ty.as_syn().clone()));
                }
            }
        }
        out
    }
}
