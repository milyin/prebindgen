//! Occurrence-level composition facts referencing flat wires by their existing names.
use super::*;
use std::collections::BTreeMap;

/// Flat group graph and per-wire domain overrides for one conversion occurrence.
/// Wire names and group names are generated identifiers, not positional indices;
/// concatenating plans never renumbers existing references.
///
/// For example, two niche-encoded `Option` layers may share one `char` carrier:
/// the flat wire list contains only `value`; the outer group names `value` as
/// its discriminator and references the inner group as a child. Each group
/// retains its own absent representation. The wire itself contains neither
/// an option nor another wire. A flag-encoded layer instead references its
/// separate flag and payload wires in order.
#[derive(Clone, Debug)]
pub struct Composition<N, G> {
    /// Groups keyed by their generated identifiers. Groups contain no wire values.
    pub groups: BTreeMap<String, CompositionGroup<N, G>>,
    /// Outermost groups, in composition order.
    pub roots: Vec<String>,
    /// Wire-name overrides: `Some(n)` is a remaining niche, `None` suppresses niches.
    /// Missing entries defer to the codec's domain for the concrete wire type.
    pub domains: BTreeMap<String, Option<N>>,
}
impl<N, G> Default for Composition<N, G> {
    fn default() -> Self {
        Self {
            groups: BTreeMap::new(),
            roots: vec![],
            domains: BTreeMap::new(),
        }
    }
}
/// One logical value's relationship to a flat set of FFI wires.
#[derive(Clone, Debug)]
pub struct CompositionGroup<N, G> {
    /// Stable generated name identifying this group within the occurrence.
    pub name: String,
    /// Ordered names of all wires used by the group, including a presence flag.
    pub members: Vec<syn::Ident>,
    /// Direct child group names; nesting belongs here, never in a wire type.
    pub children: Vec<String>,
    /// Optional presence rule or adapter-specific logical grouping.
    pub kind: GroupKind<N, G>,
}
/// Meaning of a group; destination record metadata remains adapter-owned.
#[derive(Clone, Debug)]
pub enum GroupKind<N, G> {
    /// An `Option` layer.
    Optional(OptionalGroup<N>),
    /// An adapter-specific logical value, such as a Kotlin data class.
    Adapter(G),
}
/// Presence rule for one optional layer, shared by all its member wires.
#[derive(Clone, Debug)]
pub struct OptionalGroup<N> {
    /// Existing wire identifier carrying the niche or the separate 0/1 flag.
    pub discriminator: syn::Ident,
    /// Representation consumed by this layer; `None` means a separate flag.
    pub niche: Option<N>,
    /// Payload wire names, excluding a separate presence flag.
    pub payload: Vec<syn::Ident>,
}
impl<N, G> PlanMetadata for Composition<N, G> {
    fn merge(&mut self, other: Self) -> Result<()> {
        if other.groups.keys().any(|key| self.groups.contains_key(key))
            || other
                .domains
                .keys()
                .any(|key| self.domains.contains_key(key))
        {
            return Err("duplicate composition identifier".into());
        }
        self.groups.extend(other.groups);
        self.domains.extend(other.domains);
        self.roots.extend(other.roots);
        Ok(())
    }
}
impl<N, G> Composition<N, G> {
    /// Suppress niches of conditionally active wires without changing their ABI types.
    pub fn suppress<W: prebindgen_flat::WireKind>(&mut self, wires: &[Wire<W>]) {
        for wire in wires {
            self.domains.insert(wire.name.to_string(), None);
        }
    }
    /// Enclose all current root groups in one new group referencing the supplied wires.
    pub fn enclose(
        &mut self,
        name: String,
        members: Vec<syn::Ident>,
        kind: GroupKind<N, G>,
    ) -> Result<()> {
        if self.groups.contains_key(&name) {
            return Err("duplicate composition group".into());
        }
        let children = std::mem::take(&mut self.roots);
        self.groups.insert(
            name.clone(),
            CompositionGroup {
                name: name.clone(),
                members,
                children,
                kind,
            },
        );
        self.roots.push(name);
        Ok(())
    }
    /// Find an outer group beginning at a wire in the supplied group scope.
    pub fn starting_at(
        &self,
        roots: &[String],
        name: &syn::Ident,
    ) -> Option<&CompositionGroup<N, G>> {
        roots
            .iter()
            .filter_map(|id| self.groups.get(id))
            .find(|g| g.members.first() == Some(name))
    }
}

impl<N, G> Composition<N, G> {
    /// Check identifier uniqueness, references and ordered contiguous group spans.
    /// Ungrouped wires are ordinary scalar values; roots list only explicit groups.
    pub fn validate<W: prebindgen_flat::WireKind>(&self, wires: &[Wire<W>]) -> Result<()> {
        use std::collections::BTreeSet;
        let names: Vec<_> = wires.iter().map(|w| w.name.clone()).collect();
        let unique: BTreeSet<_> = names.iter().map(ToString::to_string).collect();
        if unique.len() != names.len() {
            return Err("duplicate wire identifier in composition".into());
        }
        if self.domains.keys().any(|id| !unique.contains(id)) {
            return Err("composition domain references missing wire".into());
        }
        fn visit<N, G>(
            m: &Composition<N, G>,
            scope: &[String],
            names: &[syn::Ident],
            seen: &mut BTreeSet<String>,
        ) -> Result<()> {
            let mut cursor = 0;
            for id in scope {
                if !seen.insert(id.clone()) {
                    return Err("repeated or cyclic composition group".into());
                }
                let g = m
                    .groups
                    .get(id)
                    .ok_or("composition references missing group")?;
                if &g.name != id {
                    return Err("composition group name does not match its key".into());
                }
                let first = g.members.first().ok_or("empty composition group")?;
                let start = names[cursor..]
                    .iter()
                    .position(|n| n == first)
                    .map(|i| i + cursor)
                    .ok_or("overlapping or unordered composition group")?;
                let end = start + g.members.len();
                if names.get(start..end) != Some(g.members.as_slice()) {
                    return Err("non-contiguous composition group".into());
                }
                let child_names = if let GroupKind::Optional(o) = &g.kind {
                    if !g.members.contains(&o.discriminator) {
                        return Err("optional discriminator outside group".into());
                    }
                    let payload = if o.niche.is_some() {
                        g.members.as_slice()
                    } else {
                        if first != &o.discriminator {
                            return Err("optional flag must precede payload".into());
                        }
                        &g.members[1..]
                    };
                    if payload != o.payload {
                        return Err("optional payload does not match group".into());
                    }
                    payload
                } else {
                    g.members.as_slice()
                };
                visit(m, &g.children, child_names, seen)?;
                cursor = end;
            }
            Ok(())
        }
        let mut seen = BTreeSet::new();
        visit(self, &self.roots, &names, &mut seen)?;
        if seen.len() != self.groups.len() {
            return Err("unreachable composition group".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wires() -> Vec<Wire<syn::Type>> {
        ["a", "b", "c"]
            .iter()
            .map(|n| Wire {
                name: format_ident!("{n}"),
                kind: syn::parse_quote!(i32),
            })
            .collect()
    }
    fn group(members: &[&str]) -> Composition<(), ()> {
        let mut m = Composition::default();
        m.enclose(
            "group".into(),
            members.iter().map(|n| format_ident!("{n}")).collect(),
            GroupKind::Adapter(()),
        )
        .unwrap();
        m
    }
    #[test]
    fn validates_flat_groups_and_rejects_invalid_references() {
        let w = wires();
        let m = group(&["b", "c"]);
        m.validate(&w).unwrap();
        assert!(m
            .validate(&[w[0].clone(), w[0].clone()])
            .unwrap_err()
            .contains("duplicate wire"));
        assert!(group(&[]).validate(&w).unwrap_err().contains("empty"));
        assert!(group(&["a", "c"])
            .validate(&w)
            .unwrap_err()
            .contains("non-contiguous"));
        let mut m = group(&["a"]);
        m.domains.insert("missing".into(), None);
        assert!(m.validate(&w).unwrap_err().contains("missing wire"));
        let mut m = group(&["a"]);
        m.groups
            .get_mut("group")
            .unwrap()
            .children
            .push("group".into());
        assert!(m.validate(&w).unwrap_err().contains("cyclic"));
        let mut m = group(&["a"]);
        m.roots.push("missing".into());
        assert!(m.validate(&w).unwrap_err().contains("missing group"));
        let mut m = group(&["a"]);
        assert!(m.merge(group(&["b"])).unwrap_err().contains("duplicate"));
        m.groups.get_mut("group").unwrap().kind = GroupKind::Optional(OptionalGroup {
            discriminator: format_ident!("c"),
            niche: Some(()),
            payload: vec![format_ident!("a")],
        });
        assert!(m
            .validate(&w)
            .unwrap_err()
            .contains("discriminator outside"));
    }
}
