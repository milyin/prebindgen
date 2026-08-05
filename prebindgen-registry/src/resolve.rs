//! Completeness: which conversions a binding actually needs, and whether it has
//! them.
//!
//! What survives here is the completeness check. The generator fills the cells
//! itself (`RegistryBuilder::crossings` → `convert_with`); this decides whether the
//! set it produced covers everything reachable from an exported root.
//!
//! There is no loop. `Registry::crossings` hands the demand out inner-first, so
//! a generator answers each crossing once, with everything it composes from
//! already built — including across the `impl Fn` seam, whose args cross in the
//! opposite direction.
//!
//! After the loop, [`required_set`] performs a BFS from the **root** cells — the
//! ones the binding asked for directly — through `subs` edges. It returns the
//! reachable set rather than storing it: needing a converter is a property of the
//! graph, so it is derived once at the end instead of written back into every
//! cell it was computed from. The final invariant is that every reachable-but-
//! unresolved cell is reported as an error.

use std::collections::{HashSet, VecDeque};

use prebindgen::SourceLocation;

use crate::registry::{Direction, Registry, TypeKey};

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
        // which is why this cannot run before the conversions have filled them.
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
            for (child_dir, sub) in registry.immediate_edges(dir, key) {
                let dep = (child_dir, sub.key());
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
                    location: Some(cell.subject.location())
                        .filter(|l| l.has_position())
                        .cloned(),
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

pub(crate) fn check_complete<M>(registry: &Registry<M>) -> Result<(), ResolveError> {
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
                location: Some(cell.subject.location())
                    .filter(|l| l.has_position())
                    .cloned(),
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
