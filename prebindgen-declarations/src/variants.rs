//! Recursive construction and matching of source enum payloads.
use super::*;
use std::collections::BTreeMap;

pub(crate) fn overrides<'a, T>(
    entries: impl IntoIterator<Item = (&'a str, Vec<(&'a str, T)>)>,
) -> BTreeMap<String, BTreeMap<String, T>> {
    let mut variants = BTreeMap::new();
    for (name, fields) in entries {
        let mut members = BTreeMap::new();
        for (field, tree) in fields {
            assert!(
                members.insert(field.into(), tree).is_none(),
                "duplicate enum field override"
            );
        }
        assert!(
            variants.insert(name.into(), members).is_none(),
            "duplicate enum variant override"
        );
    }
    variants
}

fn validate_overrides<V, F, T>(
    source: &prebindgen_flat::PayloadEnum<V, F>,
    entries: &BTreeMap<String, BTreeMap<String, T>>,
) -> Result<()> {
    for (variant, fields) in entries {
        let arm = source
            .variants
            .iter()
            .find(|v| v.name.to_string().trim_start_matches("r#") == variant)
            .ok_or_else(|| format!("unknown enum variant `{variant}`"))?;
        for field in fields.keys() {
            if !arm.fields.iter().any(|f| f.key() == *field) {
                return Err(format!("unknown enum field `{variant}.{field}`"));
            }
        }
    }
    Ok(())
}

impl DeclDefaults {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_input_recipe<C: DeclCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        extra: &C::Extra,
        entries: &BTreeMap<
            String,
            BTreeMap<String, DeclInput<DeclInputRequest, DeclOutputRequest>>,
        >,
        value: bool,
        stack: &mut Vec<String>,
    ) -> Result<crate::ResolvedInput<C>> {
        let source = flat.payload_enum(ty)?;
        validate_overrides(source, entries)?;
        let default = if value {
            DeclInput::value()
        } else {
            DeclInput::auto()
        };
        let variants = source
            .variants
            .iter()
            .map(|arm| {
                let fields = entries.get(arm.name.to_string().trim_start_matches("r#"));
                arm.fields
                    .iter()
                    .map(|field| {
                        let recipe = fields.and_then(|f| f.get(&field.key())).unwrap_or(&default);
                        let previous = self.input_role.get();
                        if value {
                            self.input_role.set(InputRole::EnumField);
                        }
                        let result = self.input(flat, codec, &field.ty, recipe, extra, stack);
                        self.input_role.set(previous);
                        result.map_err(|e| {
                            format!("{}::{}.{}: {e}", type_key(ty), arm.name, field.key())
                        })
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        crate::resolution::resolve_input_enum(codec, variants, value)
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_output_recipe<C: DeclCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        extra: &C::Extra,
        entries: &BTreeMap<
            String,
            BTreeMap<String, DeclOutput<DeclInputRequest, DeclOutputRequest>>,
        >,
        value: bool,
        stack: &mut Vec<String>,
        getter: bool,
    ) -> Result<crate::ResolvedOutput<C>> {
        let source = flat.payload_enum(ty)?;
        validate_overrides(source, entries)?;
        let default = if value {
            DeclOutput::value()
        } else {
            DeclOutput::auto()
        };
        let variants = source
            .variants
            .iter()
            .map(|arm| {
                let fields = entries.get(arm.name.to_string().trim_start_matches("r#"));
                arm.fields
                    .iter()
                    .map(|field| {
                        let recipe = fields.and_then(|f| f.get(&field.key())).unwrap_or(&default);
                        self.output(flat, codec, &field.ty, recipe, extra, stack, getter)
                            .map_err(|e| {
                                format!("{}::{}.{}: {e}", type_key(ty), arm.name, field.key())
                            })
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        crate::resolution::resolve_output_enum(codec, variants, value, None)
    }
}
