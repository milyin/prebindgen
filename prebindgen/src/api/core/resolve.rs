//! Structural resolver and the post-resolution `required` propagation pass.
//!
//! The resolver fills `Registry::input_types` / `output_types` cells by asking
//! the language adapter for each unresolved type's converter via
//! [`Prebindgen::on_input_type`] / [`Prebindgen::on_output_type`]. The adapter
//! peels the type's outermost structure itself and either returns a *terminal*
//! converter or a *wrapper* that looked up inner converters in the registry
//! (declaring those inners in [`ConverterImpl::subs`]); it returns `None` to
//! **defer** when an inner isn't resolved yet.
//!
//! A fixed-point loop runs PASS A (read-only, build deltas) then PASS B (apply
//! deltas) until no entry advances. This handles inner-before-outer
//! dependencies (e.g. `Vec<Option<u64>>` whose `Vec<_>` wrapper needs
//! `Option<u64>`'s wire) and the cross-direction `impl Fn` seam (a callback's
//! args resolve in the opposite direction). New slots only go `None → Some`, so
//! the loop terminates.
//!
//! After the loop, [`required_set`] performs a BFS from the **root** cells — the
//! ones the binding asked for directly — through `subs` edges. It returns the
//! reachable set rather than storing it: needing a converter is a property of the
//! graph, so it is derived once at the end instead of written back into every
//! cell it was computed from. The final invariant is that every reachable-but-
//! unresolved cell is reported as an error.

use std::collections::{HashSet, VecDeque};

use crate::{
    api::core::{
        prebindgen::{ConverterImpl, Prebindgen},
        registry::{Direction, Registry, TypeEntry, TypeKey},
    },
    SourceLocation,
};

/// Errors surfaced by the resolution phase.
#[derive(Debug)]
pub enum ResolveError {
    /// A type that was scanned as required (or transitively reached from a
    /// required type via `subs`) ended up with no converter.
    Unresolved { entries: Vec<UnresolvedEntry> },
}

#[derive(Debug)]
pub struct UnresolvedEntry {
    pub key: TypeKey,
    pub direction: Direction,
    pub location: Option<SourceLocation>,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::Unresolved { entries } => {
                writeln!(
                    f,
                    "{} required type(s) could not be resolved:",
                    entries.len()
                )?;
                for e in entries {
                    let dir = match e.direction {
                        Direction::Input => "input",
                        Direction::Output => "output",
                    };
                    if let Some(loc) = e.location.as_ref() {
                        writeln!(
                            f,
                            "{}:{}:{}: error: unresolved prebindgen {} type `{}`",
                            loc.file, loc.line, loc.column, dir, e.key
                        )?;
                    } else {
                        writeln!(f, "error: unresolved prebindgen {} type `{}`", dir, e.key)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// Top-level resolution entry point.
///
/// Runs ONE fixed-point loop covering both directions. Each iteration sweeps
/// every unresolved entry (both input and output); deltas are collected without
/// mutating the registry, then applied at the end of the iteration. Loops until
/// a full sweep produces zero deltas.
///
/// The single-loop design lets cross-direction dependencies converge: e.g.
/// `impl Fn(Sample)` is an INPUT entry whose callback wrapper needs `Sample`'s
/// OUTPUT converter (callback args flow Rust→foreign side). `Sample`'s output
/// resolves in one iteration, then `impl Fn(Sample)` succeeds in the next.
pub fn resolve<E: Prebindgen>(
    registry: &mut Registry<E::Metadata>,
    ext: &E,
) -> Result<(), ResolveError> {
    loop {
        // PASS A (read-only): sweep every unresolved entry once per direction,
        // ask the adapter for a converter. Inner-before-outer ordering falls out
        // of the fixed-point loop: a wrapper that needs an unresolved inner
        // returns `None` and is retried next iteration.
        let deltas_in = collect_deltas(registry, Direction::Input, ext);
        let deltas_out = collect_deltas(registry, Direction::Output, ext);
        if deltas_in.is_empty() && deltas_out.is_empty() {
            break;
        }
        // PASS B: apply.
        apply_deltas(registry, Direction::Input, deltas_in);
        apply_deltas(registry, Direction::Output, deltas_out);
    }
    final_invariant_check(registry)
}

/// PASS A — walk every unresolved entry in `dir`, ask the adapter, collect
/// successful results without mutating the registry.
fn collect_deltas<E: Prebindgen>(
    registry: &Registry<E::Metadata>,
    dir: Direction,
    ext: &E,
) -> Vec<(TypeKey, TypeEntry<E::Metadata>)> {
    let mut deltas: Vec<(TypeKey, TypeEntry<E::Metadata>)> = Vec::new();
    let table = registry.type_table(dir);
    for (key, slot) in table {
        if slot.entry.is_some() {
            continue;
        }
        let key_ty = key.to_type();
        if let Some(entry) = resolve_one(ext, &key_ty, dir, registry) {
            deltas.push((key.clone(), entry));
        }
    }
    deltas
}

/// PASS B — apply collected deltas. Sole writer to the registry maps in
/// this iteration. Only fills empty (`None`) slots, so slots are monotonic
/// `None → Some` and the fixed-point loop terminates.
fn apply_deltas<M>(
    registry: &mut Registry<M>,
    dir: Direction,
    deltas: Vec<(TypeKey, TypeEntry<M>)>,
) {
    let table = registry.type_table_mut(dir);
    for (key, entry) in deltas {
        if let Some(cell) = table.get_mut(&key) {
            if cell.entry.is_none() {
                cell.entry = Some(entry);
            }
        }
    }
}

/// Resolve one entry: ask the adapter for a converter (it inspects `key_ty`
/// structurally), then — for an `impl Fn(args...)` input that nothing else
/// claimed — fall back to `dispatch_fn_input`. The resulting `TypeEntry::subs`
/// are the inner types the converter declared it composed from.
fn resolve_one<E: Prebindgen>(
    ext: &E,
    key_ty: &syn::Type,
    dir: Direction,
    registry: &Registry<E::Metadata>,
) -> Option<TypeEntry<E::Metadata>> {
    let conv: Option<ConverterImpl<E::Metadata>> = match dir {
        Direction::Input => ext.on_input_type(key_ty, registry),
        Direction::Output => ext.on_output_type(key_ty, registry),
    };
    // `impl Fn(args...) + Send + Sync + 'static` fallback (input only): callback
    // args resolve in the OUTPUT direction, so this converter declares no
    // same-direction `subs` — the callback-arg required-ness flows through the
    // registry's direction-flipped `immediate_edges`, not through `subs`.
    let conv = conv.or_else(|| {
        if dir != Direction::Input {
            return None;
        }
        let args = crate::api::core::registry::extract_fn_trait_args(key_ty)?;
        ext.dispatch_fn_input(&args, registry)
    });
    conv.map(|c| TypeEntry {
        destination: c.destination,
        function: c.function,
        pre_stages: c.pre_stages,
        subs: c.subs.iter().map(TypeKey::from_type).collect(),
        niches: c.niches,
        metadata: c.metadata,
    })
}

// ──────────────────────────────────────────────────────────────────────
// Required-flag propagation (BFS from required entries through `subs`)
// ──────────────────────────────────────────────────────────────────────

/// The cells a converter must exist for: every root, plus everything reachable
/// from one through a resolved converter's `subs`.
///
/// Derived, never stored. Needing a converter is a property of the graph, and the
/// graph is not complete until resolution has run — so computing it once here
/// beats maintaining a flag that every edge discovery has to write back.
fn required_set<M>(registry: &Registry<M>) -> HashSet<(Direction, TypeKey)> {
    let mut required: HashSet<(Direction, TypeKey)> = HashSet::new();
    let mut queue: VecDeque<(Direction, TypeKey)> = VecDeque::new();
    for dir in [Direction::Input, Direction::Output] {
        for (key, cell) in registry.type_table(dir) {
            if cell.root && required.insert((dir, key.clone())) {
                queue.push_back((dir, key.clone()));
            }
        }
    }

    while let Some((dir, key)) = queue.pop_front() {
        // Subs travel in the same direction as the parent — they are the inner
        // converters this body delegates to. An unresolved cell has none to give,
        // which is why this cannot run before the fixed-point loop.
        let Some(entry) = registry
            .type_table(dir)
            .get(&key)
            .and_then(|c| c.entry.as_ref())
        else {
            continue;
        };
        for sub_key in &entry.subs {
            if required.insert((dir, sub_key.clone())) {
                queue.push_back((dir, sub_key.clone()));
            }
        }
    }
    required
}

/// BFS from unresolved required-roots through the type graph, surfacing
/// further unresolved entries reachable through struct fields, enum variants,
/// generic args, and `impl Fn(...)` args. Stops at resolved nodes — their
/// `subs` were already walked by `required_set`, so traversing through
/// them risks reporting dependents the resolved converter doesn't actually
/// need.
fn collect_unresolved_descendants<M>(
    registry: &Registry<M>,
    seeds: &[(Direction, TypeKey)],
    seen: &mut std::collections::HashSet<(Direction, TypeKey)>,
    out: &mut Vec<UnresolvedEntry>,
) {
    let mut queue: VecDeque<(Direction, TypeKey)> = VecDeque::new();
    let enqueue_edges_from =
        |dir: Direction,
         key: &TypeKey,
         queue: &mut VecDeque<(Direction, TypeKey)>,
         seen: &mut std::collections::HashSet<(Direction, TypeKey)>| {
            let ty = key.to_type();
            for (child_dir, sub) in registry.immediate_edges(dir, &ty) {
                let dep = (child_dir, TypeKey::from_type(&sub));
                if seen.insert(dep.clone()) {
                    queue.push_back(dep);
                }
            }
        };

    for (dir, key) in seeds {
        enqueue_edges_from(*dir, key, &mut queue, seen);
    }

    while let Some((dir, key)) = queue.pop_front() {
        match registry.type_table(dir).get(&key) {
            Some(cell) if cell.entry.is_none() => {
                // Registered but unresolved — report it and keep walking.
                out.push(UnresolvedEntry {
                    key: key.clone(),
                    direction: dir,
                    location: cell.subject.location().cloned(),
                });
                enqueue_edges_from(dir, &key, &mut queue, seen);
            }
            None => {
                // Not in the registry at all — can't report (no key/location
                // worth surfacing), but its structural children may still
                // include registered-but-unresolved types worth flagging.
                enqueue_edges_from(dir, &key, &mut queue, seen);
            }
            Some(_) => {
                // Resolved — `required_set` already walked its `subs`. Stop here
                // to avoid spurious reports for descendants the resolved
                // converter doesn't need.
            }
        }
    }
}

fn final_invariant_check<M>(registry: &Registry<M>) -> Result<(), ResolveError> {
    let required = required_set(registry);
    let mut entries: Vec<UnresolvedEntry> = Vec::new();
    let mut unresolved_required_roots: Vec<(Direction, TypeKey)> = Vec::new();
    let mut seen_unresolved: HashSet<(Direction, TypeKey)> = HashSet::new();

    for dir in [Direction::Input, Direction::Output] {
        // Sorted, so a build that fails reports the same list every time.
        let mut keys: Vec<&TypeKey> = registry.type_table(dir).keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        for key in keys {
            let cell = &registry.type_table(dir)[key];
            if cell.entry.is_some() || !required.contains(&(dir, key.clone())) {
                continue;
            }
            unresolved_required_roots.push((dir, key.clone()));
            seen_unresolved.insert((dir, key.clone()));
            entries.push(UnresolvedEntry {
                key: key.clone(),
                direction: dir,
                location: cell.subject.location().cloned(),
            });
        }
    }

    collect_unresolved_descendants(
        registry,
        &unresolved_required_roots,
        &mut seen_unresolved,
        &mut entries,
    );

    if entries.is_empty() {
        Ok(())
    } else {
        Err(ResolveError::Unresolved { entries })
    }
}

#[cfg(test)]
mod tests;
