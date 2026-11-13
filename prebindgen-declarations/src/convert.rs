//! Single-value conversion declarations and preparation into Flat.
use crate::DeclFunction;
use prebindgen_flat::{
    same_type, type_key, Flat, Primitive, Result, Schema, ValueConversion, ValueSource,
};
use syn::Type;

/// Declare a Rust type's canonical single-value representation.
#[macro_export]
macro_rules! convert {($($ty:tt)+)=>{$crate::DeclConvert::<::std::convert::Infallible>::new($crate::declarations::parse_type(stringify!($($ty)+)))};}
/// Construct a source value through `Repr: Into<T>`.
#[macro_export]
macro_rules! from {($($ty:tt)+)=>{$crate::DeclConvertSource::input($crate::declarations::parse_type(stringify!($($ty)+)),false)};}
/// Construct a source value through `Repr: TryInto<T>`.
#[macro_export]
macro_rules! try_from {($($ty:tt)+)=>{$crate::DeclConvertSource::input($crate::declarations::parse_type(stringify!($($ty)+)),true)};}
/// Produce a representation through `T: Into<Repr>`.
#[macro_export]
macro_rules! into {($($ty:tt)+)=>{$crate::DeclConvertSource::output($crate::declarations::parse_type(stringify!($($ty)+)),false)};}
/// Produce a representation through `T: TryInto<Repr>`.
#[macro_export]
macro_rules! try_into {($($ty:tt)+)=>{$crate::DeclConvertSource::output($crate::declarations::parse_type(stringify!($($ty)+)),true)};}

/// One function or direction-specific trait conversion source.
#[derive(Clone, Debug)]
pub struct DeclConvertSource {
    source: ValueSource,
    representation: Option<Type>,
    input: Option<bool>,
    result_value: bool,
}
impl DeclConvertSource {
    /// Construct an input trait conversion (normally written with `from!`).
    pub fn input(representation: Type, fallible: bool) -> Self {
        Self {
            source: ValueSource::Into { fallible },
            representation: Some(representation),
            input: Some(true),
            result_value: false,
        }
    }
    /// Construct an output trait conversion (normally written with `into!`).
    pub fn output(representation: Type, fallible: bool) -> Self {
        Self {
            source: ValueSource::Into { fallible },
            representation: Some(representation),
            input: Some(false),
            result_value: false,
        }
    }
    /// Check the source signature and resolve its representation type.
    pub fn resolve<S: Schema>(
        &self,
        flat: &Flat<S>,
        ty: &Type,
        input: bool,
    ) -> Result<ValueConversion> {
        if self.input.is_some_and(|i| i != input) {
            return Err("conversion trait source has the wrong direction".into());
        }
        let representation = match &self.source {
            ValueSource::Function(name) => {
                let f = flat.function(name)?;
                if f.params.len() != 1 {
                    return Err(format!(
                        "conversion `{name}` requires exactly one parameter"
                    ));
                }
                let param = &f.params[0].ty.ty;
                let param = match param {
                    Type::Reference(r) if r.mutability.is_none() => &*r.elem,
                    Type::Reference(_) => {
                        return Err("conversion helpers cannot take mutable references".into())
                    }
                    _ => param,
                };
                let result = prebindgen_flat::result_parts(&f.ret.ty)
                    .filter(|_| !(self.result_value || input && same_type(&f.ret.ty, ty)))
                    .map_or(&f.ret.ty, |(ok, _)| ok);
                if input {
                    if !same_type(result, ty) {
                        return Err(format!(
                            "input conversion `{name}` must return `{}` or Result of that type",
                            type_key(ty)
                        ));
                    }
                    f.params[0].ty.ty.clone()
                } else {
                    if !same_type(param, ty) {
                        return Err(format!(
                            "output conversion `{name}` must take `{}` or its shared reference",
                            type_key(ty)
                        ));
                    }
                    result.clone()
                }
            }
            ValueSource::Into { .. } => flat.resolve_type(self.representation.as_ref().unwrap())?,
        };
        if same_type(
            match &representation {
                Type::Reference(r) => &r.elem,
                _ => &representation,
            },
            ty,
        ) {
            return Err(format!(
                "conversion for `{}` must use a different representation type",
                type_key(ty)
            ));
        }
        Ok(ValueConversion {
            representation,
            source: self.source.clone(),
        })
    }
}
impl<InTree, OutTree, Extension> From<DeclFunction<InTree, OutTree, Extension>>
    for DeclConvertSource
{
    fn from(mut f: DeclFunction<InTree, OutTree, Extension>) -> Self {
        let result_value = f.result_value;
        f.result_value = false;
        Self {
            source: ValueSource::Function(f.source_name()),
            representation: None,
            input: None,
            result_value,
        }
    }
}
/// Canonical conversions for one source type. Directions are independent.
#[derive(Clone, Debug)]
pub struct DeclConvert<Extension = std::convert::Infallible> {
    /// Adapter-defined canonical conversion settings.
    pub extension: Option<Extension>,
    /// Source type whose representation is selected.
    pub ty: Type,
    /// Canonical incoming representation, when selected.
    pub input: Option<DeclConvertSource>,
    /// Canonical outgoing representation, when selected.
    pub output: Option<DeclConvertSource>,
    duplicate: bool,
}
impl<Extension> DeclConvert<Extension> {
    /// Start a single-value conversion declaration for a source type.
    pub fn new(ty: Type) -> Self {
        Self {
            extension: None,
            ty,
            input: None,
            output: None,
            duplicate: false,
        }
    }
    /// Configure adapter-specific conversion settings.
    pub fn with_extension<Next>(self, extension: Next) -> DeclConvert<Next> {
        DeclConvert {
            extension: Some(extension),
            ty: self.ty,
            input: self.input,
            output: self.output,
            duplicate: self.duplicate,
        }
    }
    /// Convert one representation into the source value.
    pub fn input(mut self, source: impl Into<DeclConvertSource>) -> Self {
        self.duplicate |= self.input.is_some();
        self.input = Some(source.into());
        self
    }
    /// Convert a source value into its representation.
    pub fn output(mut self, source: impl Into<DeclConvertSource>) -> Self {
        self.duplicate |= self.output.is_some();
        self.output = Some(source.into());
        self
    }
    /// Reject duplicate or directionally invalid conversion declarations.
    pub fn validate(&self) -> Result<()> {
        if self.duplicate {
            return Err("duplicate conversion direction".into());
        }
        if self.input.is_none() && self.output.is_none() {
            return Err("conversion declaration requires an input or output source".into());
        }
        Ok(())
    }
    /// Captured helper names needed when selecting a source stream.
    pub fn functions(&self) -> Vec<String> {
        [&self.input, &self.output]
            .into_iter()
            .flatten()
            .filter_map(|s| match &s.source {
                ValueSource::Function(f) => Some(f.clone()),
                _ => None,
            })
            .collect()
    }
    /// Representation types explicitly named by trait conversions.
    pub fn representation_types(&self) -> Vec<Type> {
        [&self.input, &self.output]
            .into_iter()
            .flatten()
            .filter_map(|s| s.representation.clone())
            .collect()
    }
}
/// Resolve canonical value conversions into the authoritative Flat type table.
/// Declarations remain borrowed so the adapter can interpret and retain their extension settings.
pub fn prepare_value_conversions<S: Schema, Extension>(
    flat: &mut Flat<S>,
    decls: &[DeclConvert<Extension>],
) -> Result<()> {
    let mut selected = std::collections::BTreeMap::new();
    for decl in decls {
        decl.validate()?;
        let ty = flat.resolve_type(&decl.ty)?;
        if !matches!(ty, Type::Path(_))
            || Primitive::from_type(&ty).is_some()
            || prebindgen_flat::option_inner(&ty).is_some()
            || prebindgen_flat::sequence_parts(&ty).is_some()
            || matches!(type_key(&ty).as_str(), "String" | "str" | "()")
            || flat.type_info(&ty).is_err()
        {
            return Err(
                "canonical conversions require a named domain type, not a built-in representation"
                    .into(),
            );
        }
        let key = type_key(&ty);
        let pair = selected.entry(key.clone()).or_insert_with(|| {
            flat.type_info(&ty)
                .map(|i| i.conversion.clone())
                .unwrap_or_default()
        });
        for (input, source) in [(true, decl.input.as_ref()), (false, decl.output.as_ref())] {
            if let Some(source) = source {
                let dest = if input {
                    &mut pair.input
                } else {
                    &mut pair.output
                };
                if dest.is_some() {
                    return Err(format!("duplicate conversion for `{key}`"));
                }
                *dest = Some(source.resolve(flat, &ty, input)?);
            }
        }
    }
    // Traverse the representation shape as well as its root. `A -> Vec<A>`
    // is just as recursive as `A -> B -> A`, even when no function uses A.
    fn check<S: Schema>(
        flat: &Flat<S>,
        selected: &std::collections::BTreeMap<String, prebindgen_flat::ValueConversions>,
        ty: &Type,
        input: bool,
        active: &mut Vec<String>,
    ) -> Result<()> {
        let key = type_key(ty);
        let pair = selected
            .get(&key)
            .cloned()
            .or_else(|| flat.type_info(ty).ok().map(|i| i.conversion.clone()));
        if let Some(conversion) = pair.and_then(|p| if input { p.input } else { p.output }) {
            if active.contains(&key) {
                return Err(format!("recursive canonical conversion `{key}`"));
            }
            active.push(key);
            check(flat, selected, &conversion.representation, input, active)?;
            active.pop();
        } else if let Some(inner) = prebindgen_flat::option_inner(ty) {
            check(flat, selected, inner, input, active)?;
        } else if let Some((inner, _)) = prebindgen_flat::sequence_parts(ty) {
            check(flat, selected, inner, input, active)?;
        } else if !input {
            if let Some(inner) = crate::output_referent(ty) {
                check(flat, selected, inner, input, active)?;
            }
        }
        Ok(())
    }
    for key in selected.keys() {
        for input in [true, false] {
            let ty = syn::parse_str(key).map_err(|e| format!("conversion type: {e}"))?;
            check(flat, &selected, &ty, input, &mut Vec::new())?;
        }
    }
    for (key, pair) in selected {
        let ty = syn::parse_str(&key).map_err(|e| format!("conversion type: {e}"))?;
        flat.type_info_mut(&ty)?.conversion = pair;
    }
    Ok(())
}

/// Whether a source conversion observes its value through a shared reference.
pub fn conversion_borrows<S: Schema>(flat: &Flat<S>, conversion: &ValueConversion) -> Result<bool> {
    match &conversion.source {
        ValueSource::Function(name) => Ok(flat
            .function(name)?
            .params
            .first()
            .is_some_and(|p| matches!(&p.ty.ty,Type::Reference(r) if r.mutability.is_none()))),
        ValueSource::Into { .. } => Ok(false),
    }
}
/// Verify a resolved conversion against its source signature and representation.
pub fn validate_conversion<S: Schema>(
    flat: &Flat<S>,
    ty: &Type,
    conversion: &ValueConversion,
    input: bool,
) -> Result<()> {
    let resolved = DeclConvertSource {
        source: conversion.source.clone(),
        representation: Some(conversion.representation.clone()),
        input: None,
        result_value: match &conversion.source {
            ValueSource::Function(name) => {
                let f = flat.function(name)?;
                same_type(
                    &f.ret.ty,
                    if input {
                        ty
                    } else {
                        &conversion.representation
                    },
                )
            }
            ValueSource::Into { .. } => false,
        },
    }
    .resolve(flat, ty, input)?;
    if !same_type(&resolved.representation, &conversion.representation) {
        return Err("conversion representation does not match its helper signature".into());
    }
    Ok(())
}

/// Adapter construction of a locally selected single-value conversion.
///
/// The implementation preserves the declaration's source-type scope around the
/// conversion leaf. This keeps later selectors, projections and representation
/// modifiers scoped to the same source type. The recipe may be incoming or
/// outgoing; the adapter's request type selects the direction.
pub trait LocalConversion<Recipe, Extension = std::convert::Infallible> {
    /// Construct the complete scoped recipe, with children on structural edges.
    fn local_conversion(declaration: DeclConvert<Extension>) -> Recipe;
}

impl<I: crate::InputOps, O: crate::OutputOps, IR, OR, E> From<DeclConvert<E>>
    for crate::InputNode<I, O, IR, OR>
where
    IR: LocalConversion<crate::InputNode<I, O, IR, OR>, E>,
{
    fn from(declaration: DeclConvert<E>) -> Self {
        IR::local_conversion(declaration)
    }
}
impl<I: crate::InputOps, O: crate::OutputOps, IR, OR, E> From<DeclConvert<E>>
    for crate::OutputNode<I, O, IR, OR>
where
    OR: LocalConversion<crate::OutputNode<I, O, IR, OR>, E>,
{
    fn from(declaration: DeclConvert<E>) -> Self {
        OR::local_conversion(declaration)
    }
}
