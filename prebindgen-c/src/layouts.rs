//! Build-time selection of native or externally measured opaque storage.
use super::*;
use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// How to obtain an inline owner's size and alignment.
/// Native builds can measure a Rust type available to the build script; cross
/// builds always require an external artifact, never a host-size fallback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpaqueLayout {
    native: Option<StorageLayout>,
    key: Option<String>,
}
impl OpaqueLayout {
    /// Measure `T` only for a native binding build. During cross compilation,
    /// use the external entry named by the C declaration (or a built-in key).
    pub const fn auto<T>() -> Self {
        Self {
            native: Some(StorageLayout {
                size: std::mem::size_of::<T>(),
                alignment: std::mem::align_of::<T>(),
            }),
            key: None,
        }
    }
    /// Require external data even on native builds. No host type is needed.
    pub const fn external() -> Self {
        Self {
            native: None,
            key: None,
        }
    }
    /// Select a shared artifact key instead of the declaration's default key.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StorageLayout {
    pub(crate) size: usize,
    pub(crate) alignment: usize,
}
impl StorageLayout {
    pub(crate) fn validate(self) -> Result<()> {
        if self.size == 0 || !self.alignment.is_power_of_two() || self.size % self.alignment != 0 {
            return Err("opaque layout requires nonzero size, power-of-two alignment, and size divisible by alignment".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Features {
    features: Vec<String>,
}
/// Target size/alignment measurements produced by an external tool.
/// JSON contains `format_version:1`, a `target` triple, `sources` mapping source
/// crate names to `{features:[...]}`, and `layouts` mapping keys to
/// `{size:...,alignment:...}`. No probe tool is invoked by this library.
#[derive(Clone, Debug)]
pub struct OpaqueLayouts {
    format_version: u32,
    target: String,
    sources: BTreeMap<String, Features>,
    layouts: BTreeMap<String, StorageLayout>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayoutDocument {
    format_version: u32,
    target: String,
    #[serde(deserialize_with = "unique_map")]
    sources: BTreeMap<String, Features>,
    #[serde(deserialize_with = "unique_map")]
    layouts: BTreeMap<String, StorageLayout>,
}
fn unique_map<'de, D, V>(deserializer: D) -> std::result::Result<BTreeMap<String, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct Entries<V>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>> serde::de::Visitor<'de> for Entries<V> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an object with unique keys")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut input: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut map = BTreeMap::new();
            while let Some((key, value)) = input.next_entry::<String, V>()? {
                if map.insert(key.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate opaque layout artifact key `{key}`"
                    )));
                }
            }
            Ok(map)
        }
    }
    deserializer.deserialize_map(Entries(std::marker::PhantomData))
}
impl OpaqueLayouts {
    /// Parse an external artifact; reject malformed/unknown/duplicate fields.
    /// Its target and source features are checked when the binding is prepared.
    pub fn from_json(json: &str) -> Result<Self> {
        let raw: LayoutDocument = serde_json::from_str(json)
            .map_err(|e| format!("invalid opaque layout artifact: {e}"))?;
        let data = Self {
            format_version: raw.format_version,
            target: raw.target,
            sources: raw.sources,
            layouts: raw.layouts,
        };
        if data.format_version != 1 {
            return Err(format!(
                "unsupported opaque layout format version {}",
                data.format_version
            ));
        }
        if data.target.trim().is_empty() {
            return Err("opaque layout artifact requires a target".into());
        }
        for (key, layout) in &data.layouts {
            if key.trim().is_empty() {
                return Err("opaque layout key must not be empty".into());
            }
            layout
                .validate()
                .map_err(|e| format!("opaque layout `{key}`: {e}"))?;
        }
        for (name, source) in &data.sources {
            if name.is_empty()
                || source
                    .features
                    .iter()
                    .any(|f| f.is_empty() || f.chars().any(char::is_whitespace) || f.contains('/'))
            {
                return Err(format!("invalid opaque layout feature list for `{name}`"));
            }
            let unique: BTreeSet<_> = source.features.iter().collect();
            if unique.len() != source.features.len() {
                return Err(format!("duplicate opaque layout feature for `{name}`"));
            }
        }
        Ok(data)
    }
    /// Read an artifact without running a compiler, subprocess or probe.
    pub fn read_json(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read opaque layouts `{}`: {e}", path.display()))?;
        Self::from_json(&json)
    }
    fn resolve(&self, key: &str) -> Result<StorageLayout> {
        self.layouts
            .get(key)
            .copied()
            .ok_or_else(|| format!("missing opaque layout `{key}` for target `{}`", self.target))
    }
}
pub(crate) fn builtin(ty: &syn::Type) -> OpaqueLayout {
    if prebindgen_flat::type_key(ty) == "String" {
        OpaqueLayout::auto::<String>().key("std.string")
    } else {
        OpaqueLayout::auto::<Vec<u8>>().key("std.vec")
    }
}
fn nodes(
    plan: &OutputPlan<CWire>,
) -> impl Iterator<Item = &prebindgen_flat::WireNode<CWire, CChain, CBranch>> {
    std::iter::once(&plan.outcomes.success)
        .chain(plan.outcomes.source_error.iter())
        .chain(plan.outcomes.binding_failure.iter())
}
fn used_node(
    node: &prebindgen_flat::WireNode<CWire, CChain, CBranch>,
    names: &mut BTreeSet<String>,
) {
    for wire in prebindgen_flat::wire_definitions(std::iter::once(node)) {
        match &wire.kind {
            CWire::Owned { owner, .. } => {
                names.insert(owner.name.clone());
            }
            CWire::Callback { callback } => callback.layout_names(names),
            _ => {}
        }
    }
}
pub(crate) fn output_names(plan: &OutputPlan<CWire>, names: &mut BTreeSet<String>) {
    for node in nodes(plan) {
        used_node(node, names);
    }
}
pub(crate) fn input_names(plan: &InputPlan<CWire>, names: &mut BTreeSet<String>) {
    used_node(&plan.node, names);
}
pub(crate) fn prepare(
    flat: &mut Bindings,
    target: &str,
    artifact: Option<&OpaqueLayouts>,
    sources: &[prebindgen::Source],
) -> Result<()> {
    let mut used = BTreeSet::new();
    for f in flat.functions.values().filter(|f| f.ret.slot.is_some()) {
        for p in &f.params {
            if let Some(plan) = &p.ty.slot {
                input_names(plan, &mut used);
            }
        }
        output_names(f.ret.slot.as_ref().unwrap(), &mut used);
    }
    for info in flat.types.values() {
        if let Some(error) = &info.metadata.error {
            output_names(error.plan()?, &mut used);
        }
    }
    // Keep nested standard owners so Vec<String>/Vec<Vec<T>> can expose typed
    // borrowed element pointers and their ordinary accessors.
    loop {
        let before = used.len();
        for info in flat.types.values() {
            if let Some(owner) = &info.metadata.owned {
                if used.contains(&owner.name) || owner.explicit {
                    if let Some((element, _)) = prebindgen_flat::sequence_parts(&owner.ty) {
                        if let Some(child) = owned::declaration(flat, element) {
                            used.insert(child.name.clone());
                        }
                    }
                }
            }
        }
        if before == used.len() {
            break;
        }
    }
    for info in flat.types.values_mut() {
        if info
            .metadata
            .owned
            .as_ref()
            .is_some_and(|o| !o.explicit && !used.contains(&o.name))
        {
            info.metadata.owned = None;
        }
    }
    if let Some(data) = artifact {
        if data.target != target {
            return Err(format!(
                "opaque layout target mismatch: artifact `{}`, binding `{target}`",
                data.target
            ));
        }
        let mut verified = BTreeSet::new();
        for source in sources {
            let name = source.crate_name();
            verified.insert(name.to_owned());
            let features = data
                .sources
                .get(name)
                .ok_or_else(|| format!("missing opaque layout source features for `{name}`"))?;
            let guard = source.require_features(&features.features)?;
            if !flat
                .guards
                .iter()
                .any(|g| quote!(#g).to_string() == quote!(#guard).to_string())
            {
                flat.guards.push(guard);
            }
        }
        for function in flat.functions.values() {
            if let Some(name) = &function.location.crate_name {
                if !verified.contains(name) {
                    return Err(format!(
                        "register layout source `{name}` to verify captured features"
                    ));
                }
            }
        }
    }
    flat.metadata.opaque_layouts.clear();
    for info in flat.types.values() {
        if let Some(owner) = &info.metadata.owned {
            let key = owner.layout.key.clone().unwrap_or_else(|| {
                if owned::std_owner(&owner.ty) {
                    if prebindgen_flat::type_key(&owner.ty) == "String" {
                        "std.string".into()
                    } else {
                        "std.vec".into()
                    }
                } else {
                    owner.name.clone()
                }
            });
            if key.trim().is_empty() {
                return Err(format!(
                    "opaque owner `{}` has an empty layout key",
                    owner.name
                ));
            }
            let layout = if let Some(data) = artifact {
                data.resolve(&key)?
            } else if target != env!("PREBINDGEN_C_GENERATOR_TARGET") {
                return Err(format!("cross-compilation to `{target}` requires an external opaque layout artifact (missing `{key}`); generator platform is `{}`",env!("PREBINDGEN_C_GENERATOR_TARGET")));
            } else {
                owner.layout.native.ok_or_else(|| {
                    format!(
                        "opaque owner `{}` requires external layout `{key}`",
                        owner.name
                    )
                })?
            };
            layout.validate()?;
            flat.metadata
                .opaque_layouts
                .insert(owner.name.clone(), layout);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use prebindgen_declarations::fun;
    fn model(source: &str) -> Bindings {
        Bindings::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap()
    }
    fn artifact(
        target: &str,
        layouts: serde_json::Value,
        sources: serde_json::Value,
    ) -> OpaqueLayouts {
        OpaqueLayouts::from_json(&serde_json::json!({"format_version":1,"target":target,"sources":sources,"layouts":layouts}).to_string()).unwrap()
    }
    fn native() -> &'static str {
        env!("PREBINDGEN_C_GENERATOR_TARGET")
    }
    #[test]
    fn native_measurement_and_strict_cross_lookup_are_order_independent() {
        let make = || Cbindgen::new(model("pub type Item=i64;pub fn make()->Item{}"));
        let native = make()
            .opaque_owned_struct("item", syn::parse_quote!(Item), OpaqueLayout::auto::<i64>())
            .function(fun!(make))
            .prepare()
            .unwrap();
        assert_eq!(
            native.metadata.opaque_layouts["item"].size,
            std::mem::size_of::<i64>()
        );
        let cross = "i686-unknown-linux-gnu";
        let error = make()
            .target(cross)
            .opaque_owned_struct("item", syn::parse_quote!(Item), OpaqueLayout::auto::<i64>())
            .function(fun!(make))
            .prepare()
            .err()
            .unwrap();
        assert!(
            error.contains("requires an external opaque layout artifact"),
            "{error}"
        );
        let data = artifact(
            cross,
            serde_json::json!({"item":{"size":8,"alignment":4}}),
            serde_json::json!({}),
        );
        // A zero-sized host measurement must be ignored for a cross target.
        let first = make()
            .target(cross)
            .opaque_layouts(data.clone())
            .opaque_owned_struct("item", syn::parse_quote!(Item), OpaqueLayout::auto::<()>())
            .function(fun!(make))
            .prepare()
            .unwrap();
        let second = make()
            .opaque_owned_struct("item", syn::parse_quote!(Item), OpaqueLayout::auto::<()>())
            .function(fun!(make))
            .opaque_layouts(data)
            .target(cross)
            .prepare()
            .unwrap();
        assert_eq!(
            first.metadata.opaque_layouts,
            second.metadata.opaque_layouts
        );
        assert_eq!(
            first.metadata.opaque_layouts["item"],
            StorageLayout {
                size: 8,
                alignment: 4
            }
        );
    }
    #[test]
    fn explicit_artifacts_never_fall_back_to_native_measurements() {
        let make = || {
            Cbindgen::new(model("pub type Item=i64;pub fn make()->Item{}"))
                .opaque_owned_struct("item", syn::parse_quote!(Item), OpaqueLayout::auto::<i64>())
                .function(fun!(make))
        };
        let error = make()
            .opaque_layouts(artifact(
                native(),
                serde_json::json!({}),
                serde_json::json!({}),
            ))
            .prepare()
            .err()
            .unwrap();
        assert!(error.contains("missing opaque layout `item`"), "{error}");
        let error = make()
            .opaque_layouts(artifact(
                "other",
                serde_json::json!({}),
                serde_json::json!({}),
            ))
            .prepare()
            .err()
            .unwrap();
        assert!(error.contains("target mismatch"), "{error}");
        assert!(make()
            .opaque_layouts_file("/missing/prebindgen-layouts.json")
            .prepare()
            .err()
            .unwrap()
            .contains("cannot read opaque layouts"));
        assert!(Cbindgen::new(model("pub type Item=i64;"))
            .opaque_owned_struct("item", syn::parse_quote!(Item), OpaqueLayout::external())
            .prepare()
            .err()
            .unwrap()
            .contains("requires external layout"));
    }
    #[test]
    fn pointer_only_and_unused_standard_types_need_no_layouts() {
        Cbindgen::new(model("pub fn make()->Vec<u8>{}"))
            .target("i686-unknown-linux-gnu")
            .opaque_ptr("bytes", syn::parse_quote!(Vec<u8>))
            .function(fun!(make))
            .build()
            .unwrap();
        let prepared = Cbindgen::new(model(
            "pub type Item=String;pub fn make(s:&str)->Item{}pub fn unused()->Vec<i64>{}",
        ))
        .target("i686-unknown-linux-gnu")
        .opaque_ptr("item", syn::parse_quote!(Item))
        .function(fun!(make))
        .panic()
        .prepare()
        .unwrap();
        assert!(prepared.metadata.opaque_layouts.is_empty());
    }
    #[test]
    fn invalid_targets_and_header_symbol_collisions_are_diagnostics() {
        assert!(Cbindgen::new(model("pub fn make()->i64{}"))
            .target("not-a-rust-target")
            .function(fun!(make))
            .prepare()
            .err()
            .unwrap()
            .contains("invalid binding target"));
        assert!(Cbindgen::new(model("pub fn TARGET()->i64{}"))
            .function(fun!(TARGET))
            .build()
            .err()
            .unwrap()
            .contains("duplicate C export"));
    }
    #[test]
    fn unrelated_qualified_types_do_not_require_source_registration() {
        let mut flat = model("pub type Item=i64;");
        flat.types.insert(
            "foreign :: PointerOnly".into(),
            prebindgen_flat::TypeInfo {
                ty: syn::parse_quote!(foreign::PointerOnly),
                metadata: CType::default(),
            },
        );
        Cbindgen::new(flat)
            .opaque_owned_struct("item", syn::parse_quote!(Item), OpaqueLayout::external())
            .opaque_layouts(artifact(
                native(),
                serde_json::json!({"item":{"size":8,"alignment":8}}),
                serde_json::json!({}),
            ))
            .prepare()
            .unwrap();
    }
    #[test]
    fn generic_vectors_and_strings_use_separate_shared_entries() {
        let source =
            "pub fn numbers()->Vec<i64>{}pub fn text()->String{}pub fn strings()->Vec<String>{}";
        let prepared=Cbindgen::new(model(source)).target("i686-unknown-linux-gnu")
            .opaque_layouts(artifact("i686-unknown-linux-gnu",serde_json::json!({"std.vec":{"size":12,"alignment":4},"std.string":{"size":16,"alignment":4}}),serde_json::json!({})))
            .function(fun!(numbers)).function(fun!(text)).function(fun!(strings)).prepare().unwrap();
        let string = owned::declaration(&prepared, &syn::parse_quote!(String)).unwrap();
        assert_eq!(prepared.metadata.opaque_layouts[&string.name].size, 16);
        let vec = owned::declaration(&prepared, &syn::parse_quote!(Vec<String>)).unwrap();
        assert_eq!(prepared.metadata.opaque_layouts[&vec.name].size, 12);
        let header = generate(&prepared).unwrap().header;
        assert!(header.contains("const pb_owned_string *"), "{header}");
    }
    #[test]
    fn malformed_artifacts_are_diagnostics() {
        for json in [
            r#"{"format_version":2,"target":"x","sources":{},"layouts":{}}"#,
            r#"{"format_version":1,"target":"x","sources":{},"layouts":{"x":{"size":0,"alignment":8}}}"#,
            r#"{"format_version":1,"target":"x","sources":{},"layouts":{"x":{"size":8,"alignment":3}}}"#,
            r#"{"format_version":1,"target":"x","sources":{},"layouts":{"x":{"size":9,"alignment":8}}}"#,
            r#"{"format_version":1,"target":"x","sources":{},"layouts":{"x":{"size":8,"alignment":8},"x":{"size":8,"alignment":8}}}"#,
            r#"{"format_version":1,"target":"x","sources":{"src":{"features":[]},"src":{"features":[]}},"layouts":{}}"#,
            r#"{"format_version":1,"target":"x","sources":{"src":{"features":["a","a"]}},"layouts":{}}"#,
            r#"{"format_version":1,"target":"x","sources":{},"layouts":{},"unknown":1}"#,
            r#"{"format_version":1,"target":"x","layouts":{}}"#,
        ] {
            assert!(OpaqueLayouts::from_json(json).is_err(), "{json}");
        }
    }
    fn source_dir(features: &str) -> (tempfile::TempDir, prebindgen::Source) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("crate_name.txt"), "flat").unwrap();
        std::fs::write(dir.path().join("features.txt"), features).unwrap();
        let source = prebindgen::Source::builder(dir.path())
            .enable_target_filtering(None::<String>)
            .build();
        (dir, source)
    }
    #[test]
    fn probe_features_are_checked_and_guarded_against_the_target_library() {
        let (_dir, source) = source_dir("b\na\n");
        let make = || {
            Cbindgen::new(model("pub type Item=i64;")).opaque_owned_struct(
                "item",
                syn::parse_quote!(Item),
                OpaqueLayout::external(),
            )
        };
        let data = artifact(
            native(),
            serde_json::json!({"item":{"size":8,"alignment":8}}),
            serde_json::json!({"flat":{"features":["a","b"]}}),
        );
        let prepared = make()
            .opaque_layouts(data.clone())
            .layout_source(source.clone())
            .layout_source(source.clone())
            .prepare()
            .unwrap();
        let guards = prepared
            .guards
            .iter()
            .map(|g| quote!(#g).to_string())
            .collect::<String>();
        assert!(guards.contains("flat :: FEATURES"));
        assert!(guards.contains("flat/a flat/b"));
        assert_eq!(
            prepared
                .guards
                .iter()
                .filter(|g| quote!(#g).to_string().contains("flat :: FEATURES"))
                .count(),
            1
        );
        let unverified = prebindgen::Source::builder(_dir.path())
            .enable_feature_filtering(None::<String>)
            .build();
        assert!(unverified
            .require_features(&["a".into(), "b".into()])
            .unwrap_err()
            .contains("enable feature verification"));
        let renamed = prebindgen::Source::builder(_dir.path())
            .crate_name("renamed")
            .enable_target_filtering(None::<String>)
            .build();
        let guard = renamed.require_features(&["a".into(), "b".into()]).unwrap();
        let guard = quote!(#guard).to_string();
        assert!(guard.contains("renamed :: FEATURES"), "{guard}");
        assert!(guard.contains("flat/a flat/b"), "{guard}");
        make().opaque_layouts(data.clone()).prepare().unwrap();
        let mut captured = model("pub fn make()->i64{}");
        captured.function_mut("make").unwrap().location.crate_name = Some("flat".into());
        assert!(Cbindgen::new(captured)
            .function(fun!(make))
            .opaque_layouts(data)
            .prepare()
            .err()
            .unwrap()
            .contains("register layout source"));
        let wrong = artifact(
            native(),
            serde_json::json!({"item":{"size":8,"alignment":8}}),
            serde_json::json!({"flat":{"features":["a"]}}),
        );
        assert!(make()
            .opaque_layouts(wrong)
            .layout_source(source)
            .prepare()
            .err()
            .unwrap()
            .contains("features for `flat` differ"));
        let (_dir, source) = source_dir("");
        assert!(make()
            .opaque_layouts(artifact(
                native(),
                serde_json::json!({"item":{"size":8,"alignment":8}}),
                serde_json::json!({})
            ))
            .layout_source(source)
            .prepare()
            .err()
            .unwrap()
            .contains("missing opaque layout source features"));
    }
}
