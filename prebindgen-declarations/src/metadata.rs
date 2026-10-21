//! Explain how the FFI values in a conversion plan represent the source value.
use super::*;
use std::collections::BTreeMap;

/// Describes how a conversion's FFI values fit together.
///
/// A conversion plan has a list of **wires**: the individual values passed across
/// the FFI boundary. This metadata explains how to interpret those values together.
/// It is stored in [`InputPlan::metadata`] or [`OutputPlan::metadata`] and is used
/// while generating bindings; it is not passed across the FFI boundary.
///
/// For example, C can represent an `Option<i32>` with two wires:
///
/// ```text
/// present: int32_t    0 for None, 1 for Some
/// value:   int32_t    the integer inside Some; ignored for None
/// ```
///
/// A **group** is a named description of the wires that together represent a
/// value. For this example, one optional group lists `present` and `value`, says
/// that `present` decides between `None` and `Some`, and identifies `value` as
/// the payload. The C adapter uses that information to document the arguments;
/// the JNI adapter uses optional groups to generate nullable Kotlin parameters
/// and results where possible.
///
/// Groups can contain other groups. For example, an optional record can have an
/// outer group describing whether the record is present and a child group
/// describing how its fields form the record. [`groups`](Self::groups) stores all
/// these descriptions by name. [`roots`](Self::roots) lists the groups that have
/// no parent. Groups refer to wires by name; the wires stay in the plan's list.
///
/// Some types have a **niche**: an otherwise unused representation that can stand
/// for `None` without adding a presence flag. For example, C uses `0x110000`,
/// which is not a valid Rust `char`, for `None` in `Option<char>`. Once that value
/// is taken, an enclosing option needs a different unused value or a flag.
/// [`domains`](Self::domains) records which niche, if any, each wire can still
/// offer to an enclosing option. It does not describe the wire's full set of
/// valid values.
///
/// `N` is the adapter's description of a niche. `G` holds information for groups
/// specific to an adapter, such as the Kotlin class to construct from record
/// fields. Use `()` for either type when that information is not needed.
#[derive(Clone, Debug)]
pub struct Composition<N, G> {
    /// Every group in this conversion, including nested groups, indexed by name.
    /// Each entry describes which wires represent a value and how to interpret them.
    pub groups: BTreeMap<String, CompositionGroup<N, G>>,
    /// Names of the groups that are not children of another group.
    /// They appear in the same order as their wires in the plan. A wire that needs
    /// no grouping can be omitted from all groups; it needs no entry here.
    pub roots: Vec<String>,
    /// Whether each wire may supply an unused representation for another `Option`.
    /// Keys are wire names. The three cases mean:
    ///
    /// - No entry: ask [`Codec::niche`] what the wire's type offers.
    /// - `Some(n)`: use `n` as the next unused representation for `None`.
    /// - `None`: do not use this wire to encode another `None`.
    ///
    /// For example, after `Option<char>` reserves `0x110000` on a C wire named
    /// `value`, its entry offers `0x110001` for an enclosing option. Other wires
    /// in the same optional value get `None`: their defaults for an absent value
    /// must not accidentally be used to distinguish another option layer.
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
/// Describes the wires that together represent one value, such as an option or record.
///
/// For `Option<i32>` encoded as a flag and an integer, `members` names both wires
/// and `kind` explains the flag. For a record, `members` names the wires for its
/// fields and `kind` holds the adapter's instructions for constructing the record.
/// A nested value can have its own group, listed in `children`.
#[derive(Clone, Debug)]
pub struct CompositionGroup<N, G> {
    /// Name of this group, unique within the conversion plan.
    /// Must match the key used in [`Composition::groups`].
    pub name: String,
    /// Names of all wires used to represent this value, in the plan's wire order.
    /// Includes wires used by child groups and any separate presence flag.
    /// These wires must occupy consecutive positions in the plan.
    pub members: Vec<syn::Ident>,
    /// Names of groups immediately inside this one, in wire order.
    /// For an optional record, this can name the group describing the record's
    /// fields. Each name refers to an entry in [`Composition::groups`].
    pub children: Vec<String>,
    /// How to interpret the listed wires: as an option or as an adapter-defined value.
    pub kind: GroupKind<N, G>,
}
/// What value a group's wires represent and how to interpret them.
#[derive(Clone, Debug)]
pub enum GroupKind<N, G> {
    /// An option: records how to distinguish `None` from `Some` and find its payload.
    Optional(OptionalGroup<N>),
    /// A value described by the language adapter, such as a Kotlin data class
    /// constructed from several field wires.
    Adapter(G),
}
/// How a group's wires encode `None` and `Some(payload)`.
///
/// With a separate presence flag, `discriminator` names that flag, `niche` is
/// `None`, and `payload` lists the other wires. With a niche, `discriminator`
/// names one of the payload wires and `niche` describes its reserved `None`
/// representation. Each layer of a nested option has its own description.
#[derive(Clone, Debug)]
pub struct OptionalGroup<N> {
    /// Name of the wire to inspect to decide whether the option is present.
    /// This is either a separate 0/1 flag or a payload wire with a reserved
    /// representation for `None`.
    pub discriminator: syn::Ident,
    /// `Some(n)` means this option uses `n` to encode `None` on `discriminator`.
    /// `None` means `discriminator` is a separate flag: 0 for absent, 1 for present.
    /// Unlike [`Composition::domains`], this describes the representation already
    /// chosen for this option, not one available to an enclosing option.
    pub niche: Option<N>,
    /// Names of the wires encoding the value inside `Some`, in wire order.
    /// Excludes a separate presence flag. These values are ignored for `None`.
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
    /// Prevent enclosing options from using these wires to encode `None`.
    ///
    /// Writes `None` to their [`domains`](Self::domains) entries. For example,
    /// a wire used only in one constructor alternative cannot tell us whether
    /// the whole optional input is present when another alternative is selected.
    pub fn suppress<W: prebindgen_flat::WireKind>(&mut self, wires: &[Wire<W>]) {
        for wire in wires {
            self.domains.insert(wire.name.to_string(), None);
        }
    }
    /// Add a parent group around the groups currently listed in [`roots`](Self::roots).
    ///
    /// Those groups become its children, and `roots` becomes just `[name]`.
    /// `members` must name all wires belonging to the new parent, including its
    /// children's wires. Returns an error if `name` already identifies a group.
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
    /// Among the supplied group names, find the group whose first wire is `name`.
    ///
    /// Pass [`roots`](Self::roots) to search the outermost groups, or a group's
    /// `children` to search immediately inside it. This does not search descendants.
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
    /// Check that these descriptions agree with the conversion plan's wire list.
    ///
    /// Rejects duplicate wire names, missing references, empty groups, cycles and
    /// groups whose wires are not consecutive or in order. Also checks that each
    /// optional group's flag or niche wire and payload belong to that group.
    /// Wires without a group are allowed.
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
