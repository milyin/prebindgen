//! Single-value conversion declarations and preparation into Flat.
use crate::{FunctionDecl, Input, InputRequest, Output, OutputRequest};
use crate::{InputBranch, InputLeafOp, InputUnary, OutputBranch, OutputLeafOp, OutputUnary};
use crate::{InputNode, OutputNode};
use prebindgen_flat::{
    same_type, type_key, Flat, Primitive, Result, Schema, ValueConversion, ValueSource,
};
use syn::Type;

/// Declare a Rust type's canonical single-value representation.
#[macro_export]
macro_rules! convert {($($ty:tt)+)=>{$crate::ConvertDecl::new($crate::declarations::parse_type(stringify!($($ty)+)))};}
/// Construct a source value through `Repr: Into<T>`.
#[macro_export]
macro_rules! from {($($ty:tt)+)=>{$crate::ConvertSource::input($crate::declarations::parse_type(stringify!($($ty)+)),false)};}
/// Construct a source value through `Repr: TryInto<T>`.
#[macro_export]
macro_rules! try_from {($($ty:tt)+)=>{$crate::ConvertSource::input($crate::declarations::parse_type(stringify!($($ty)+)),true)};}
/// Produce a representation through `T: Into<Repr>`.
#[macro_export]
macro_rules! into {($($ty:tt)+)=>{$crate::ConvertSource::output($crate::declarations::parse_type(stringify!($($ty)+)),false)};}
/// Produce a representation through `T: TryInto<Repr>`.
#[macro_export]
macro_rules! try_into {($($ty:tt)+)=>{$crate::ConvertSource::output($crate::declarations::parse_type(stringify!($($ty)+)),true)};}

/// One function or direction-specific trait conversion source.
#[derive(Clone, Debug)]
pub struct ConvertSource {
    source: ValueSource,
    representation: Option<Type>,
    input: Option<bool>,
    result_value: bool,
}
impl ConvertSource {
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
    pub(crate) fn resolve<S: Schema>(
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
impl From<FunctionDecl> for ConvertSource {
    fn from(mut f: FunctionDecl) -> Self {
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
pub struct ConvertDecl {
    /// Source type whose representation is selected.
    pub ty: Type,
    pub(crate) input: Option<ConvertSource>,
    pub(crate) output: Option<ConvertSource>,
    duplicate: bool,
}
impl ConvertDecl {
    /// Start a single-value conversion declaration for a source type.
    pub fn new(ty: Type) -> Self {
        Self {
            ty,
            input: None,
            output: None,
            duplicate: false,
        }
    }
    /// Convert one representation into the source value.
    pub fn input(mut self, source: impl Into<ConvertSource>) -> Self {
        self.duplicate |= self.input.is_some();
        self.input = Some(source.into());
        self
    }
    /// Convert a source value into its representation.
    pub fn output(mut self, source: impl Into<ConvertSource>) -> Self {
        self.duplicate |= self.output.is_some();
        self.output = Some(source.into());
        self
    }
    pub(crate) fn validate(&self) -> Result<()> {
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
impl From<ConvertDecl> for Input<InputRequest, OutputRequest> {
    fn from(decl: ConvertDecl) -> Self {
        Self::for_type(
            decl.ty.clone(),
            Self::Request(InputRequest::Conversion(Box::new(decl))),
        )
    }
}
impl From<ConvertDecl> for Output<OutputRequest, InputRequest> {
    fn from(decl: ConvertDecl) -> Self {
        Self::for_type(
            decl.ty.clone(),
            Self::Request(OutputRequest::Conversion(Box::new(decl))),
        )
    }
}
/// Prepare canonical type conversions in the authoritative Flat type table.
/// No slots are compiled here; declarations and independent codecs can use them.
pub fn prepare_value_conversions<S: Schema>(
    flat: &mut Flat<S>,
    decls: Vec<ConvertDecl>,
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
        for (input, source) in [(true, decl.input), (false, decl.output)] {
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

use crate::{DeclarationCodec, Declarations};
impl Declarations {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_input_conversion<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        conversion: ValueConversion,
        child: &Input<InputRequest, OutputRequest>,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<crate::ResolvedInput<C>> {
        let key = format!("value-input:{}", type_key(ty));
        if stack.contains(&key) {
            return Err(format!("recursive canonical conversion `{}`", type_key(ty)));
        }
        stack.push(key);
        let input = self.input(flat, codec, &conversion.representation, child, extra, stack);
        stack.pop();
        Ok(InputNode::Unary {
            operation: codec.resolve_input_unary(InputUnary::Convert {
                conversion: Box::new(conversion),
            })?,
            child: Box::new(input?),
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_output_conversion<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        conversion: ValueConversion,
        child: &Output<OutputRequest, InputRequest>,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<crate::ResolvedOutput<C>> {
        let key = format!("value-output:{}", type_key(ty));
        if stack.contains(&key) {
            return Err(format!("recursive canonical conversion `{}`", type_key(ty)));
        }
        stack.push(key);
        let output = self.output(
            flat,
            codec,
            &conversion.representation,
            child,
            extra,
            stack,
            true,
        );
        stack.pop();
        Ok(OutputNode::Unary {
            operation: codec.resolve_output_unary(OutputUnary::Convert {
                conversion: Box::new(conversion),
            })?,
            child: Box::new(output?),
        })
    }
    pub(crate) fn resolve_input_value<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<crate::ResolvedInput<C>> {
        if type_key(ty) == "()" {
            return Ok(InputNode::Branch {
                operation: codec.resolve_input_branch(InputBranch::Fields)?,
                children: vec![],
            });
        }
        if let Some(c) = flat
            .type_info(ty)
            .ok()
            .and_then(|i| i.conversion.input.clone())
        {
            return self.resolve_input_conversion(
                flat,
                codec,
                ty,
                c,
                &Input::value(),
                extra,
                stack,
            );
        }
        if let Some(inner) = prebindgen_flat::option_inner(ty) {
            return Ok(InputNode::Unary {
                operation: codec.resolve_input_unary(InputUnary::Optional)?,
                child: Box::new(self.resolve_input_value(flat, codec, inner, extra, stack)?),
            });
        }
        if let Some((inner, _)) = prebindgen_flat::sequence_parts(ty)
            .filter(|_| !codec.input_collection_is_leaf(flat, ty, self.input_role.get(), extra))
        {
            return Ok(InputNode::Unary {
                operation: codec.resolve_input_unary(InputUnary::Sequence)?,
                child: Box::new(self.resolve_input_value(flat, codec, inner, extra, stack)?),
            });
        }
        self.input(
            flat,
            codec,
            ty,
            &Input::Leaf(InputLeafOp::Value),
            extra,
            stack,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_output_value<C: DeclarationCodec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        extra: &C::Extra,
        stack: &mut Vec<String>,
        getter: bool,
        policy: Option<&crate::Projection>,
    ) -> Result<crate::ResolvedOutput<C>> {
        if type_key(ty) == "()" {
            return Ok(OutputNode::Branch {
                operation: codec.resolve_output_branch(OutputBranch::Fields)?,
                children: vec![],
            });
        }
        let c = flat
            .type_info(ty)
            .ok()
            .and_then(|i| i.conversion.output.clone())
            .or_else(|| {
                crate::output_referent(ty)
                    .and_then(|t| flat.type_info(t).ok())
                    .and_then(|i| i.conversion.output.clone())
            });
        if let Some(c) = c {
            if policy.is_some_and(|p| !p.fields.is_empty()) {
                return Err(
                    "field projections on a canonical value require an explicit output tree".into(),
                );
            }
            if policy.is_some_and(|p| p.mode == crate::ProjectionMode::Borrowed)
                && !conversion_borrows(flat, &c)?
            {
                return Err(
                    "a borrowed projection requires a shared-reference conversion helper".into(),
                );
            }
            return self.resolve_output_conversion(
                flat,
                codec,
                ty,
                c,
                &Output::value(),
                extra,
                stack,
            );
        }
        if let Some(inner) = prebindgen_flat::option_inner(ty) {
            return Ok(OutputNode::Unary {
                operation: codec.resolve_output_unary(OutputUnary::Optional)?,
                child: Box::new(
                    self.resolve_output_value(flat, codec, inner, extra, stack, getter, policy)?,
                ),
            });
        }
        if let Some((inner, _)) = prebindgen_flat::sequence_parts(ty)
            .filter(|_| !codec.output_collection_is_leaf(flat, ty, extra))
        {
            return Ok(OutputNode::Unary {
                operation: codec.resolve_output_unary(OutputUnary::Sequence {
                    delivery: crate::SequenceDelivery::Value,
                    mode: policy.map(|p| p.mode),
                })?,
                child: Box::new(
                    self.resolve_output_value(flat, codec, inner, extra, stack, getter, policy)?,
                ),
            });
        }
        self.output(
            flat,
            codec,
            ty,
            &policy.map_or(Output::Leaf(OutputLeafOp::Value), |p| {
                Output::Leaf(OutputLeafOp::Projected(p.clone()))
            }),
            extra,
            stack,
            getter,
        )
    }
}

fn conversion_borrows<S: Schema>(flat: &Flat<S>, conversion: &ValueConversion) -> Result<bool> {
    match &conversion.source {
        ValueSource::Function(name) => Ok(flat
            .function(name)?
            .params
            .first()
            .is_some_and(|p| matches!(&p.ty.ty,Type::Reference(r) if r.mutability.is_none()))),
        ValueSource::Into { .. } => Ok(false),
    }
}
pub(crate) fn validate_conversion<S: Schema>(
    flat: &Flat<S>,
    ty: &Type,
    conversion: &ValueConversion,
    input: bool,
) -> Result<()> {
    let resolved = ConvertSource {
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
