//! Measured layouts shared by native queries and external probe consumers.
use crate::Result;
use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
/// Size and alignment of one native allocation or value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizeAlign {
    /// Size in bytes, including trailing padding.
    pub size: usize,
    /// Required byte alignment.
    pub alignment: usize,
}
impl SizeAlign {
    /// Check the dimensions of a nonempty native storage value.
    pub fn validate(self) -> Result<()> {
        if self.size == 0 || !self.alignment.is_power_of_two() || self.size % self.alignment != 0 {
            return Err("layout requires nonzero size, power-of-two alignment, and size divisible by alignment".into());
        }
        Ok(())
    }
}
/// Actual captured features of a source crate used by an external probe.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFeatures {
    /// Enabled features, excluding dependency-qualified request syntax.
    pub features: Vec<String>,
}
/// Measurements supplied by an external target probe. This crate never runs a probe.
/// Version 1 contains opaque storage sizes, alignments and source feature metadata.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutArtifact {
    /// External document version.
    pub format_version: u32,
    /// Target triple of the measured library.
    pub target: String,
    /// Captured source features, checked against source metadata by adapters.
    #[serde(deserialize_with = "unique_map")]
    pub sources: BTreeMap<String, SourceFeatures>,
    /// Measurements keyed by stable source/record identity.
    #[serde(deserialize_with = "unique_map")]
    pub layouts: BTreeMap<String, SizeAlign>,
}
impl LayoutArtifact {
    /// Parse a strict layout document, rejecting incomplete or duplicate measurements.
    pub fn from_json(json: &str) -> Result<Self> {
        let data: Self =
            serde_json::from_str(json).map_err(|e| format!("invalid layout artifact: {e}"))?;
        if data.format_version != 1 {
            return Err(format!(
                "unsupported layout format version {}",
                data.format_version
            ));
        }
        if data.target.trim().is_empty() {
            return Err("layout artifact requires a target".into());
        }
        for (key, layout) in &data.layouts {
            if key.trim().is_empty() {
                return Err("layout key must not be empty".into());
            }
            layout
                .validate()
                .map_err(|e| format!("layout `{key}`: {e}"))?;
        }
        for (name, source) in &data.sources {
            if name.is_empty()
                || source
                    .features
                    .iter()
                    .any(|f| f.is_empty() || f.chars().any(char::is_whitespace) || f.contains('/'))
            {
                return Err(format!("invalid layout feature list for `{name}`"));
            }
            if source.features.iter().collect::<BTreeSet<_>>().len() != source.features.len() {
                return Err(format!("duplicate layout feature for `{name}`"));
            }
        }
        Ok(data)
    }
    /// Read a JSON artifact; no subprocess or target execution is performed.
    pub fn read_json(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        Self::from_json(
            &std::fs::read_to_string(path)
                .map_err(|e| format!("cannot read layouts `{}`: {e}", path.display()))?,
        )
    }
    /// Require valid opaque storage dimensions; missing entries never use host data.
    pub fn storage(&self, key: &str) -> Result<SizeAlign> {
        let layout = *self
            .layouts
            .get(key)
            .ok_or_else(|| format!("missing opaque layout `{key}` for target `{}`", self.target))?;
        layout.validate()?;
        Ok(layout)
    }
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
            let mut out = BTreeMap::new();
            while let Some((key, value)) = input.next_entry::<String, V>()? {
                if out.insert(key.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate layout key `{key}`"
                    )));
                }
            }
            Ok(out)
        }
    }
    deserializer.deserialize_map(Entries(std::marker::PhantomData))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn document(version: u32, fields: &str) -> String {
        r#"{"format_version":VERSION,"target":"aarch64-unknown-linux-gnu","sources":{"flat":{"features":["unstable"]}},"layouts":{"Row":{"size":16,"alignment":8FIELDS}}}"#
            .replace("VERSION", &version.to_string()).replace("FIELDS", fields)
    }
    #[test]
    fn opaque_storage_requires_a_complete_version_one_measurement() {
        let opaque = LayoutArtifact::from_json(&document(1, "")).unwrap();
        assert_eq!(
            opaque.storage("Row").unwrap(),
            SizeAlign {
                size: 16,
                alignment: 8
            }
        );
        assert!(opaque.storage("Missing").unwrap_err().contains("Missing"));
        assert!(LayoutArtifact::from_json(&document(2, "")).is_err());
        assert!(LayoutArtifact::from_json(&document(1, r#", "fields":{"tag":0}"#)).is_err());
    }
    #[test]
    fn malformed_measurements_never_fall_back() {
        let valid = document(1, "");
        for invalid in [
            valid.replace("\"alignment\":8", "\"alignment\":3"),
            valid.replace("\"size\":16", "\"size\":0"),
            valid.replace("\"unstable\"", "\"unstable\",\"unstable\""),
            valid.replace("\"unstable\"", "\"dep/feature\""),
            valid.replace("\"format_version\":1", "\"format_version\":2"),
            valid.replace("\"target\":", "\"unknown\":1,\"target\":"),
            valid.replace("aarch64-unknown-linux-gnu", ""),
        ] {
            assert!(LayoutArtifact::from_json(&invalid).is_err(), "{invalid}");
        }
    }
}
