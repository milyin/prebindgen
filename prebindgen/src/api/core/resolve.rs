//! Structural resolver and the post-resolution `required` propagation pass.
//!
//! The resolver fills `Registry::input_types` / `output_types` cells by asking
//! the language ext for each unresolved type's converter via
//! [`Prebindgen::on_input_type`] / [`Prebindgen::on_output_type`]. The ext peels
//! the type's outermost structure itself and either returns a *terminal*
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
//! After the loop, [`propagate_required`] performs a BFS from the scan-time
//! required entries through `subs` edges; the final invariant is that every
//! `required: true && None` is reported as an error.

use std::collections::VecDeque;

use crate::SourceLocation;

use crate::api::core::prebindgen::{ConverterImpl, Prebindgen};
use crate::api::core::registry::{Direction, Registry, TypeEntry, TypeKey};

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
        // ask the ext for a converter. No per-rank phasing — the ext peels the
        // type itself; inner-before-outer ordering falls out of the fixed-point
        // loop (a wrapper that needs an unresolved inner returns `None` and is
        // retried next iteration).
        let deltas_in = collect_deltas(registry, Direction::Input, ext);
        let deltas_out = collect_deltas(registry, Direction::Output, ext);
        if deltas_in.is_empty() && deltas_out.is_empty() {
            break;
        }
        // PASS B: apply.
        apply_deltas(registry, Direction::Input, deltas_in);
        apply_deltas(registry, Direction::Output, deltas_out);
    }
    propagate_required(registry);
    final_invariant_check(registry)
}

/// PASS A — walk every unresolved entry in `dir`, ask the ext, collect
/// successful results without mutating the registry.
fn collect_deltas<E: Prebindgen>(
    registry: &Registry<E::Metadata>,
    dir: Direction,
    ext: &E,
) -> Vec<(TypeKey, TypeEntry<E::Metadata>)> {
    let mut deltas: Vec<(TypeKey, TypeEntry<E::Metadata>)> = Vec::new();
    let table = match dir {
        Direction::Input => &registry.input_types,
        Direction::Output => &registry.output_types,
    };
    for (key, slot) in table {
        if slot.is_some() {
            continue;
        }
        let key_ty = key.to_type();
        let scan_required = match dir {
            Direction::Input => registry.is_required_input_at_scan(key),
            Direction::Output => registry.is_required_output_at_scan(key),
        };
        if let Some(entry) = resolve_one(ext, &key_ty, dir, scan_required, registry) {
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
    let table = match dir {
        Direction::Input => &mut registry.input_types,
        Direction::Output => &mut registry.output_types,
    };
    for (key, entry) in deltas {
        if let Some(slot) = table.get_mut(&key) {
            if slot.is_none() {
                *slot = Some(entry);
            }
        }
    }
}

/// Resolve one entry: ask the ext for a converter (it inspects `key_ty`
/// structurally), then — for an `impl Fn(args...)` input that nothing else
/// claimed — fall back to `dispatch_fn_input`. The resulting `TypeEntry::subs`
/// are the inner types the converter declared it composed from.
fn resolve_one<E: Prebindgen>(
    ext: &E,
    key_ty: &syn::Type,
    dir: Direction,
    scan_required: bool,
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
        required: scan_required,
        niches: c.niches,
        metadata: c.metadata,
    })
}

// ──────────────────────────────────────────────────────────────────────
// Required-flag propagation (BFS from required entries through `subs`)
// ──────────────────────────────────────────────────────────────────────

fn propagate_required<M>(registry: &mut Registry<M>) {
    // Seed the queue from scan-time required keys plus any `required: true`
    // already on resolved entries.
    let mut queue: VecDeque<(Direction, TypeKey)> = VecDeque::new();
    for k in &registry.required_inputs_scan {
        queue.push_back((Direction::Input, k.clone()));
    }
    for k in &registry.required_outputs_scan {
        queue.push_back((Direction::Output, k.clone()));
    }

    while let Some((dir, key)) = queue.pop_front() {
        // Mark this entry's `required: true` if it's resolved.
        let subs = mark_and_get_subs(registry, dir, &key);
        // Subs travel in the same direction as the parent — they're the
        // inner converters this body delegates to.
        for sub_key in subs {
            if !is_required_resolved(registry, dir, &sub_key) {
                set_required(registry, dir, &sub_key);
                queue.push_back((dir, sub_key));
            }
        }
    }
}

fn mark_and_get_subs<M>(registry: &mut Registry<M>, dir: Direction, key: &TypeKey) -> Vec<TypeKey> {
    let table = match dir {
        Direction::Input => &mut registry.input_types,
        Direction::Output => &mut registry.output_types,
    };
    match table.get_mut(key) {
        Some(Some(entry)) => {
            entry.required = true;
            entry.subs.clone()
        }
        _ => vec![],
    }
}

fn is_required_resolved<M>(registry: &Registry<M>, dir: Direction, key: &TypeKey) -> bool {
    let table = match dir {
        Direction::Input => &registry.input_types,
        Direction::Output => &registry.output_types,
    };
    table
        .get(key)
        .and_then(|slot| slot.as_ref())
        .is_some_and(|e| e.required)
}

fn set_required<M>(registry: &mut Registry<M>, dir: Direction, key: &TypeKey) {
    match dir {
        Direction::Input => {
            registry.required_inputs_scan.insert(key.clone());
        }
        Direction::Output => {
            registry.required_outputs_scan.insert(key.clone());
        }
    }
    let table = match dir {
        Direction::Input => &mut registry.input_types,
        Direction::Output => &mut registry.output_types,
    };
    if let Some(Some(entry)) = table.get_mut(key) {
        entry.required = true;
    }
}

fn lookup_slot<'a, M>(
    registry: &'a Registry<M>,
    dir: Direction,
    key: &TypeKey,
) -> Option<&'a Option<TypeEntry<M>>> {
    let table = match dir {
        Direction::Input => &registry.input_types,
        Direction::Output => &registry.output_types,
    };
    table.get(key)
}

/// BFS from unresolved required-roots through the type graph, surfacing
/// further unresolved entries reachable through struct fields, enum variants,
/// generic args, and `impl Fn(...)` args. Stops at resolved nodes — their
/// `subs` were already walked by `propagate_required`, so traversing through
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
        match lookup_slot(registry, dir, &key) {
            Some(None) => {
                // Registered but unresolved — report it and keep walking.
                out.push(UnresolvedEntry {
                    key: key.clone(),
                    direction: dir,
                    location: registry.type_locations.get(&key).cloned(),
                });
                enqueue_edges_from(dir, &key, &mut queue, seen);
            }
            None => {
                // Not in the registry at all — can't report (no key/location
                // worth surfacing), but its structural children may still
                // include registered-but-unresolved types worth flagging.
                enqueue_edges_from(dir, &key, &mut queue, seen);
            }
            Some(Some(_)) => {
                // Resolved — `propagate_required` already walked its `subs`.
                // Stop here to avoid spurious reports for descendants the
                // resolved converter doesn't need.
            }
        }
    }
}

fn final_invariant_check<M>(registry: &Registry<M>) -> Result<(), ResolveError> {
    let mut entries: Vec<UnresolvedEntry> = Vec::new();
    let scan_required_input = &registry.required_inputs_scan;
    let scan_required_output = &registry.required_outputs_scan;
    let mut unresolved_required_roots: Vec<(Direction, TypeKey)> = Vec::new();
    let mut seen_unresolved: std::collections::HashSet<(Direction, TypeKey)> =
        std::collections::HashSet::new();

    for (key, slot) in &registry.input_types {
        let needs = match slot {
            Some(e) => e.required,
            None => scan_required_input.contains(key),
        };
        if needs && slot.is_none() {
            unresolved_required_roots.push((Direction::Input, key.clone()));
            seen_unresolved.insert((Direction::Input, key.clone()));
            entries.push(UnresolvedEntry {
                key: key.clone(),
                direction: Direction::Input,
                location: registry.type_locations.get(key).cloned(),
            });
        }
    }
    for (key, slot) in &registry.output_types {
        let needs = match slot {
            Some(e) => e.required,
            None => scan_required_output.contains(key),
        };
        if needs && slot.is_none() {
            unresolved_required_roots.push((Direction::Output, key.clone()));
            seen_unresolved.insert((Direction::Output, key.clone()));
            entries.push(UnresolvedEntry {
                key: key.clone(),
                direction: Direction::Output,
                location: registry.type_locations.get(key).cloned(),
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
mod tests {
    use super::*;

    /// Regression: when a required type is itself unresolved AND has fields
    /// that are also unresolved, the diagnostic must list both. Previously
    /// `propagate_required` could not cross an unresolved parent (no `subs`
    /// edges exist past it), so a missing build.rs declaration for `ZKeyExpr`
    /// — only referenced as a field of an unresolved `Outer` — went silent.
    #[test]
    fn final_invariant_reports_unresolved_field_of_unresolved_struct() {
        use crate::api::core::registry::{Registry, TypeKey};

        let mut reg: Registry<()> = Registry::default();

        // Index a struct `Outer { inner: ZKeyExpr }` so the BFS can walk
        // into its field. `ZKeyExpr` itself stays *unindexed* (the user's
        // build.rs forgot to declare it), but it does appear in the type
        // tables because scan-recursion would have registered it as a field
        // of `Outer`. Simulate the post-scan registry state directly.
        let outer_struct: syn::ItemStruct =
            syn::parse_str("struct Outer { inner: ZKeyExpr }").unwrap();
        reg.structs.insert(
            outer_struct.ident.clone(),
            (outer_struct, SourceLocation::default()),
        );

        // `Outer` is a required INPUT, unresolved (slot stays `None`).
        let outer_key = TypeKey::parse("Outer");
        reg.input_types.insert(outer_key.clone(), None);
        reg.required_inputs_scan.insert(outer_key.clone());

        // `ZKeyExpr` is also in the type table (scan recursed into the
        // field) but unresolved and NOT marked required at scan time —
        // exactly the case the BFS is here to catch.
        let zke_key = TypeKey::parse("ZKeyExpr");
        reg.input_types.insert(zke_key.clone(), None);

        let err = final_invariant_check(&reg).expect_err("must surface unresolved");
        let ResolveError::Unresolved { entries } = err;
        let reported: std::collections::HashSet<String> =
            entries.iter().map(|e| e.key.to_string()).collect();
        assert!(
            reported.contains("Outer"),
            "expected `Outer` in report, got {:?}",
            reported
        );
        assert!(
            reported.contains("ZKeyExpr"),
            "expected `ZKeyExpr` (transitively unresolved via Outer.inner) in report, got {:?}",
            reported
        );
    }

    /// Counterpart to the regression above: the BFS must NOT walk through
    /// resolved nodes. `propagate_required` already covers their `subs`
    /// edges, so re-walking them risks reporting deeper unresolved entries
    /// that the resolved converter doesn't actually depend on.
    #[test]
    fn final_invariant_stops_at_resolved_nodes() {
        use crate::api::core::registry::{Direction, Registry, TypeEntry, TypeKey};
        use crate::SourceLocation as Loc;

        let mut reg: Registry<()> = Registry::default();

        let outer_struct: syn::ItemStruct =
            syn::parse_str("struct Outer { inner: Inner }").unwrap();
        let inner_struct: syn::ItemStruct =
            syn::parse_str("struct Inner { unused: Unrelated }").unwrap();
        reg.structs
            .insert(outer_struct.ident.clone(), (outer_struct, Loc::default()));
        reg.structs
            .insert(inner_struct.ident.clone(), (inner_struct, Loc::default()));

        // `Outer` required & unresolved; `Inner` RESOLVED (with a dummy
        // entry); `Unrelated` unresolved but only reachable through Inner.
        let outer_key = TypeKey::parse("Outer");
        let inner_key = TypeKey::parse("Inner");
        let unrelated_key = TypeKey::parse("Unrelated");

        reg.input_types.insert(outer_key.clone(), None);
        reg.required_inputs_scan.insert(outer_key.clone());

        reg.input_types.insert(
            inner_key.clone(),
            Some(TypeEntry {
                destination: syn::parse_quote!(i64),
                function: syn::parse_quote!(
                    fn __dummy() {}
                ),
                pre_stages: vec![],
                subs: vec![],
                required: false,
                niches: crate::api::core::niches::Niches::empty(),
                metadata: (),
            }),
        );

        reg.input_types.insert(unrelated_key.clone(), None);

        let err = final_invariant_check(&reg).expect_err("must surface Outer");
        let ResolveError::Unresolved { entries } = err;
        let reported: std::collections::HashSet<String> =
            entries.iter().map(|e| e.key.to_string()).collect();
        assert!(reported.contains("Outer"));
        // Inner is resolved -> not reported.
        assert!(!reported.contains("Inner"));
        // Unrelated sits behind a resolved Inner -> must NOT be reported.
        assert!(
            !reported.contains("Unrelated"),
            "BFS must stop at resolved nodes, got report: {:?}",
            reported
        );
        let _ = Direction::Input; // keep import used
    }
}
