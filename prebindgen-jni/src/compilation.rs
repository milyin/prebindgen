//! Interpretation of this adapter's resolved conversion operations.
#![allow(missing_docs)]
use crate::{JniInputExt, JniOutputExt};

use super::operations::{
    JniInput, JniInputArrow, JniInputBranch, JniInputLeafOp, JniInputOps, JniInputUnary, JniOutput,
    JniOutputBranch, JniOutputLeafOp, JniOutputOps, JniOutputUnary,
};
use prebindgen_declarations::*;
use prebindgen_flat::{Flat, Result};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;
#[derive(Default)]
pub struct JniCompileState {
    bulk: std::cell::Cell<bool>,
    ordinary_collections: std::cell::Cell<bool>,
}
pub(crate) struct JniCompiler<'a, 'b, C: Codec> {
    pub(crate) cx: &'a Compiler<'b, C>,
}
impl<'b, C: Codec> std::ops::Deref for JniCompiler<'_, 'b, C> {
    type Target = Compiler<'b, C>;
    fn deref(&self) -> &Self::Target {
        self.cx
    }
}
trait Consumes {
    fn consumes<
        C: Codec<InOps = JniInputOps, OutOps = JniOutputOps, CompileState = JniCompileState>,
    >(
        &self,
        flat: &Flat<C::Schema>,
        ty: &Type,
        extra: &C::Extra,
        codec: &C,
        borrowed: bool,
    ) -> bool;
}
impl Consumes for JniOutput {
    fn consumes<
        C: Codec<InOps = JniInputOps, OutOps = JniOutputOps, CompileState = JniCompileState>,
    >(
        &self,
        flat: &prebindgen_flat::Flat<C::Schema>,
        ty: &Type,
        extra: &C::Extra,
        codec: &C,
        borrowed: bool,
    ) -> bool {
        match self {
            JniOutput::Unary {
                operation: JniOutputUnary::Convert { conversion, .. },
                ..
            } => match &conversion.source {
                prebindgen_flat::ValueSource::Function(name) => {
                    flat.function(name).is_ok_and(|f| {
                        f.params
                            .first()
                            .is_some_and(|p| !matches!(p.ty.ty, Type::Reference(_)))
                    })
                }
                prebindgen_flat::ValueSource::Into { .. } => true,
            },
            JniOutput::Unary {
                operation: JniOutputUnary::Sequence { mode, .. },
                ..
            } => match mode {
                Some(mode) => *mode == ProjectionMode::Owned,
                None => !borrowed && sequence_parts(ty).is_some_and(|(_, view)| view.is_owned()),
            },
            JniOutput::Unary {
                operation: JniOutputUnary::Bulk,
                child,
            } => child.consumes(flat, ty, extra, codec, borrowed),
            JniOutput::Unary {
                operation: JniOutputUnary::Named { .. },
                child,
                ..
            } => child.consumes(flat, ty, extra, codec, borrowed),
            JniOutput::Unary {
                operation: JniOutputUnary::Optional,
                child,
            } => child.consumes(flat, option_inner(ty).unwrap_or(ty), extra, codec, borrowed),
            JniOutput::Leaf(JniOutputLeafOp::Value) => {
                !borrowed && output_referent(ty).is_none() && codec.output_consumes(flat, ty, extra)
            }
            JniOutput::Leaf(JniOutputLeafOp::Projected(p)) => p.mode == ProjectionMode::Owned,
            JniOutput::Branch {
                operation: JniOutputBranch::OwnedFields,
                children: _,
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Into { .. },
                ..
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Result { .. },
                ..
            } => true,
            JniOutput::Branch {
                operation: JniOutputBranch::Record { mode, .. },
                ..
            } => mode.map_or(!borrowed && output_referent(ty).is_none(), |m| {
                m == ProjectionMode::Owned
            }),
            JniOutput::Branch {
                operation: JniOutputBranch::Enum { mode, .. },
                ..
            } => mode.map_or(!borrowed && output_referent(ty).is_none(), |m| {
                m == ProjectionMode::Owned
            }),
            JniOutput::Leaf(
                JniOutputLeafOp::ConfigDefault(never)
                | JniOutputLeafOp::ConfigValue(never)
                | JniOutputLeafOp::ConfigConversion(never),
            ) => match *never {},
            JniOutput::Branch {
                operation: JniOutputBranch::Combine,
                children: p,
            } => p
                .iter()
                .any(|p| p.consumes(flat, ty, extra, codec, borrowed)),
            _ => false,
        }
    }
}
impl<C: Codec<InOps = JniInputOps, OutOps = JniOutputOps, CompileState = JniCompileState>>
    JniCompiler<'_, '_, C>
{
    pub(crate) fn input_named(
        &self,
        ty: &Type,
        recipe: &JniInput,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInputPlan<C>> {
        self.input_unprepared_named(ty, recipe, extra, active, names)
    }

    pub(crate) fn input_unprepared_named(
        &self,
        ty: &Type,
        recipe: &JniInput,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInputPlan<C>> {
        if prebindgen_flat::callback_signature(ty)?.is_some()
            && !matches!(
                recipe,
                JniInput::Leaf(JniInputLeafOp::Value) | JniInput::Arrow { .. }
            )
        {
            return Err("source callback parameters only support leaf inputs".into());
        }
        let var = names
            .map(|(_, var)| var.clone())
            .unwrap_or_else(|| self.name());
        match recipe {
            JniInput::None => self.cx.input_unit(ty),
            JniInput::Branch {
                operation: JniInputBranch::Variant,
                ..
            } => Err("variant branch requires an enum parent".into()),
            JniInput::Branch {
                operation: JniInputBranch::Enum { value },
                children: variants,
            } => self.cx.enum_input(
                ty,
                &variants
                    .iter()
                    .map(|v| v.variant_fields())
                    .collect::<Result<Vec<_>>>()?,
                *value,
                extra,
                active,
                names,
            ),
            JniInput::Unary {
                operation: JniInputUnary::Convert { conversion },
                child: input,
            } => self.input_conversion(ty, conversion, input, extra, active, names),
            JniInput::Unary {
                operation: JniInputUnary::Temporary,
                child: inner,
            } => self.cx.input_temporary(ty, extra, active, names, inner),
            JniInput::Unary {
                operation: JniInputUnary::Bulk,
                child: inner,
            } => {
                require_bulk(self.codec, Direction::Input)?;
                if self.state.bulk.get() {
                    return self.input_named(ty, inner, extra, active, names);
                }
                let wire = self.name();
                self.state.bulk.set(true);
                let child = self.input_named(ty, inner, extra, active, names);
                self.state.bulk.set(false);
                self.codec
                    .bulk_input(self.flat, ty, extra, wire, child?, active)
            }
            JniInput::Arrow {
                operation: operation @ JniInputArrow::BoundCallback(callback),
                domain,
                codomain,
            } => {
                callback.validate_for(self.flat, ty)?;
                let signature = prebindgen_flat::callback_signature(ty)?
                    .ok_or("arrow requires a callable source")?;
                if domain.argument_nodes()?.len() != signature.arguments.len() {
                    return Err("wrong arrow domain arity".into());
                }
                self.codec.input_arrow(InputArrow {
                    flat: self.flat,
                    operation,
                    domain,
                    codomain,
                    extra,
                    wire: names
                        .map(|(wire, _)| wire.clone())
                        .unwrap_or_else(|| self.name()),
                    target: var,
                    active,
                })
            }
            JniInput::Unary {
                operation: JniInputUnary::Sequence,
                child: element_recipe,
            } => {
                if !self.state.bulk.get() {
                    if let JniInput::Unary {
                        operation: JniInputUnary::Bulk,
                        child: inner,
                    } = element_recipe.as_ref()
                    {
                        return self.input_named(
                            ty,
                            &JniInput::Unary {
                                operation: JniInputUnary::Bulk,
                                child: Box::new(JniInput::Unary {
                                    operation: JniInputUnary::Sequence,
                                    child: inner.clone(),
                                }),
                            },
                            extra,
                            active,
                            names,
                        );
                    }
                }
                self.cx.input_sequence(
                    ty,
                    element_recipe,
                    extra,
                    active,
                    names,
                    var,
                    self.state.bulk.get(),
                )
            }
            JniInput::Unary {
                operation: JniInputUnary::Optional,
                child: inner,
            } => self.cx.input_optional(ty, extra, active, names, var, inner),
            JniInput::Leaf(JniInputLeafOp::Value) => {
                let w = names
                    .map(|(wire, _)| wire.clone())
                    .unwrap_or_else(|| self.name());
                let representation = if self.state.bulk.get() {
                    Representation::Bulk
                } else {
                    Representation::Ordinary
                };
                if representation == Representation::Bulk {
                    require_bulk(self.codec, Direction::Input)?;
                }
                self.codec.input_leaf(InputLeaf {
                    flat: self.flat,
                    ty,
                    extra,
                    wire: w,
                    target: var,
                    active,
                    representation,
                    role: self.input_role.get(),
                })
            }
            JniInput::Leaf(
                JniInputLeafOp::ConfigDefault(never)
                | JniInputLeafOp::ConfigValue(never)
                | JniInputLeafOp::ConfigConversion(never)
                | JniInputLeafOp::ConfigConstructor(never),
            ) => match *never {},
            JniInput::Unary {
                operation: JniInputUnary::ConfigForType { ty: never },
                ..
            }
            | JniInput::Branch {
                operation:
                    JniInputBranch::ConfigEnumFields(never) | JniInputBranch::ConfigVariantFields(never),
                ..
            }
            | JniInput::Arrow {
                operation: JniInputArrow::ConfigCallback(never),
                ..
            } => match *never {},
            JniInput::Branch {
                operation: JniInputBranch::Fields,
                children,
            }
            | JniInput::Branch {
                operation: JniInputBranch::Record,
                children,
            } => self.cx.input_fields(
                ty,
                extra,
                active,
                names,
                var,
                children,
                matches!(
                    recipe,
                    JniInput::Branch {
                        operation: JniInputBranch::Record,
                        ..
                    }
                ),
            ),
            JniInput::Branch {
                operation: JniInputBranch::Constructor { function },
                children: args,
            } => self
                .cx
                .input_constructor(ty, extra, active, var, function, args),
            JniInput::Branch {
                operation: JniInputBranch::Selector,
                children: arms,
            } => self.cx.input_selector(ty, extra, active, var, arms),
        }
    }

    pub(crate) fn output_root(
        &self,
        ty: &Type,
        recipe: &JniOutput,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        reference_return: bool,
    ) -> Result<CodecOutputPlan<C>> {
        if let JniOutput::Unary {
            operation: JniOutputUnary::Result { error },
            child: ok,
        } = recipe
        {
            return self
                .cx
                .output_result(ty, ok, error, extra, value, borrowed, reference_return);
        }
        if reference_return {
            self.output_return(ty, recipe, extra, value, borrowed)
        } else {
            self.output(ty, recipe, extra, value, borrowed)
        }
    }
    // Preserve reference semantics for source returns and getter results.
    // The codec checks whether a getter's owner survives the generated call.
    pub(crate) fn output_return(
        &self,
        ty: &Type,
        recipe: &JniOutput,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutputPlan<C>> {
        if !borrowed {
            if matches!(ty, Type::Reference(_)) {
                if let JniOutput::Branch {
                    operation: JniOutputBranch::Combine,
                    children: parts,
                } = recipe
                {
                    return self.cx.output_reference_views(ty, parts, extra, value);
                }
            }
            if let (Type::Reference(reference), JniOutput::Leaf(JniOutputLeafOp::Value)) =
                (ty, recipe)
            {
                let wire = self.name();
                if let Some(plan) = self.codec.output_return_reference(
                    self.flat,
                    reference,
                    extra,
                    wire.clone(),
                    value.clone(),
                )? {
                    return Ok(plan);
                }
                return self.output_named(ty, recipe, extra, value, false, Some(&wire));
            }
            if let (
                Some(payload_ty),
                JniOutput::Unary {
                    operation: JniOutputUnary::Optional,
                    child: inner,
                },
            ) = (option_inner(ty), recipe)
            {
                let tag = self.name();
                let payload = self.name();
                let child =
                    self.output_return(payload_ty, inner, extra, quote!(#payload), false)?;
                return optional_output(self.codec, extra, tag, payload, value, false, child);
            }
        }
        self.output(ty, recipe, extra, value, borrowed)
    }
    fn output(
        &self,
        ty: &Type,
        recipe: &JniOutput,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutputPlan<C>> {
        self.output_named(ty, recipe, extra, value, borrowed, None)
    }
    pub(crate) fn output_named(
        &self,
        ty: &Type,
        recipe: &JniOutput,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire_name: Option<&syn::Ident>,
    ) -> Result<CodecOutputPlan<C>> {
        if let Some(referent) = output_referent(ty) {
            if matches ! (recipe , JniOutput :: Leaf (JniOutputLeafOp :: Projected (p)) if p . mode == ProjectionMode :: Owned)
            {
                return Err("owned projection cannot consume a reference payload".into());
            }
            let value = if borrowed {
                quote ! (* (# value))
            } else {
                value
            };
            return self.output_named(referent, recipe, extra, value, true, wire_name);
        }
        let mut out = self.cx.empty_output()?;
        match recipe {
            JniOutput::Unary {
                operation: JniOutputUnary::CallbackArgument { .. },
                ..
            } => return Err("callback argument ownership requires an arrow domain".into()),
            JniOutput::None => {
                return self.cx.output_unit(ty, out);
            }
            JniOutput::Arrow {
                operation: never, ..
            } => match *never {},
            JniOutput::Branch {
                operation: JniOutputBranch::Variant | JniOutputBranch::Arguments,
                ..
            } => return Err("variant/argument branch requires its parent operation".into()),
            JniOutput::Branch {
                operation:
                    JniOutputBranch::Enum {
                        value: grouped,
                        mode,
                    },
                children: variants,
            } => {
                return self.cx.enum_output(
                    ty,
                    &variants
                        .iter()
                        .map(|v| v.variant_fields())
                        .collect::<Result<Vec<_>>>()?,
                    *grouped,
                    *mode,
                    extra,
                    value,
                    borrowed,
                    wire_name,
                );
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Convert { conversion },
                child: output,
            } => return self.output_conversion(ty, conversion, output, extra, value, borrowed),
            JniOutput::Unary {
                operation: JniOutputUnary::Bulk,
                child: inner,
            } => {
                require_bulk(self.codec, Direction::Output)?;
                if self.state.bulk.get() {
                    return self.output_named(ty, inner, extra, value, borrowed, wire_name);
                }
                let wire = self.name();
                self.state.bulk.set(true);
                let child = self.output_named(ty, inner, extra, value, borrowed, wire_name);
                self.state.bulk.set(false);
                return self.codec.bulk_output(self.flat, ty, extra, wire, child?);
            }
            JniOutput::Leaf(JniOutputLeafOp::Discard) => return Ok(out),
            JniOutput::Unary {
                operation: JniOutputUnary::Named { name },
                child: output,
            } => {
                return self
                    .cx
                    .output_named_view(ty, extra, value, borrowed, wire_name, name, output);
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Sequence { delivery, mode },
                child: element,
            } => {
                if self.state.bulk.get() && *delivery == SequenceDelivery::Callback {
                    return Err ("bulk cannot enclose callback delivery; apply bulk to the callback element expansion instead" . into ()) ;
                }
                if *delivery == SequenceDelivery::Auto
                    && !self.state.bulk.get()
                    && !self.state.ordinary_collections.get()
                    && self.codec.default_output_sequence_is_bulk(self.flat, extra)
                {
                    return self.output_named(
                        ty,
                        &JniOutput::Unary {
                            operation: JniOutputUnary::Bulk,
                            child: Box::new(recipe.clone()),
                        },
                        extra,
                        value,
                        borrowed,
                        wire_name,
                    );
                }
                if !self.state.bulk.get() && *delivery != SequenceDelivery::Callback {
                    if let JniOutput::Unary {
                        operation: JniOutputUnary::Bulk,
                        child: inner,
                    } = element.as_ref()
                    {
                        return self.output_named(
                            ty,
                            &JniOutput::Unary {
                                operation: JniOutputUnary::Bulk,
                                child: Box::new(JniOutput::Unary {
                                    operation: JniOutputUnary::Sequence {
                                        delivery: *delivery,
                                        mode: *mode,
                                    },
                                    child: inner.clone(),
                                }),
                            },
                            extra,
                            value,
                            borrowed,
                            wire_name,
                        );
                    }
                }
                let ordinary = self.state.ordinary_collections.replace(
                    self.state.ordinary_collections.get() || *delivery != SequenceDelivery::Auto,
                );
                let result = self.cx.output_sequence(
                    ty,
                    element,
                    extra,
                    value,
                    borrowed,
                    delivery,
                    mode,
                    self.state.bulk.get(),
                );
                self.state.ordinary_collections.set(ordinary);
                return result;
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Result { .. },
                ..
            } => {
                return Err("Result error routing is supported only at the outermost return".into())
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Optional,
                child: inner,
            } => {
                return self
                    .cx
                    .output_optional(ty, extra, value, borrowed, wire_name, inner);
            }
            JniOutput::Leaf(
                JniOutputLeafOp::ConfigDefault(never)
                | JniOutputLeafOp::ConfigValue(never)
                | JniOutputLeafOp::ConfigConversion(never),
            ) => match *never {},
            JniOutput::Unary {
                operation:
                    JniOutputUnary::ConfigForType { ty: never }
                    | JniOutputUnary::ConfigSequenceCallback { structural: never },
                ..
            }
            | JniOutput::Branch {
                operation:
                    JniOutputBranch::ConfigEnumFields(never)
                    | JniOutputBranch::ConfigVariantFields(never)
                    | JniOutputBranch::ConfigNamedFields(never),
                ..
            } => match *never {},
            JniOutput::Leaf(JniOutputLeafOp::Value) => {
                let representation = if self.state.bulk.get() {
                    Representation::Bulk
                } else {
                    Representation::Ordinary
                };
                return self
                    .cx
                    .output_value(ty, extra, value, borrowed, wire_name, representation);
            }
            JniOutput::Leaf(JniOutputLeafOp::Projected(projection)) => {
                let representation = if self.state.bulk.get() {
                    Representation::Bulk
                } else {
                    Representation::Ordinary
                };
                return self.cx.output_projection(
                    ty,
                    extra,
                    value,
                    borrowed,
                    wire_name,
                    projection,
                    representation,
                );
            }
            JniOutput::Branch {
                operation: JniOutputBranch::Fields,
                children,
            }
            | JniOutput::Branch {
                operation: JniOutputBranch::OwnedFields,
                children,
            }
            | JniOutput::Branch {
                operation: JniOutputBranch::Record { .. },
                children,
                ..
            } => {
                let grouped = matches!(
                    recipe,
                    JniOutput::Branch {
                        operation: JniOutputBranch::Record { .. },
                        ..
                    }
                );
                let owned = match recipe {
                    JniOutput::Branch {
                        operation: JniOutputBranch::OwnedFields,
                        ..
                    } => true,
                    JniOutput::Branch {
                        operation: JniOutputBranch::Record { mode },
                        ..
                    } => mode.map_or(!borrowed, |m| m == ProjectionMode::Owned),
                    _ => false,
                };
                let explicit_copy = matches!(
                    recipe,
                    JniOutput::Branch {
                        operation: JniOutputBranch::Record {
                            mode: Some(ProjectionMode::Borrowed)
                        },
                        ..
                    }
                );
                return self.cx.output_fields(
                    ty,
                    extra,
                    value,
                    borrowed,
                    wire_name,
                    children,
                    grouped,
                    owned,
                    explicit_copy,
                    out,
                );
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Getter { function },
                child: output,
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Into { function },
                child: output,
            } => {
                let consuming = matches!(
                    recipe,
                    JniOutput::Unary {
                        operation: JniOutputUnary::Into { .. },
                        ..
                    }
                );
                return self
                    .cx
                    .output_getter(ty, extra, value, borrowed, function, output, consuming, out);
            }
            JniOutput::Branch {
                operation: JniOutputBranch::Combine,
                children: parts,
            } => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                if parts
                    .iter()
                    .filter(|p| p.consumes(self.flat, ty, extra, self.codec, borrowed))
                    .count()
                    > 1
                {
                    return Err ("a combination cannot consume the same value twice; use field_self_ref() for an explicit copy view" . into ()) ;
                }
                if !borrowed {
                    if let Some(plan) = self.output_owner_views(ty, parts, extra, value.clone())? {
                        return Ok(plan);
                    }
                }
                let mut bodies = Vec::new();
                for (index, child) in parts.iter().enumerate() {
                    let consuming = child.consumes(self.flat, ty, extra, self.codec, borrowed);
                    let arg = if borrowed || consuming {
                        value.clone()
                    } else {
                        quote ! (&# value)
                    };
                    let mut p = self.with_temporary_receiver(!borrowed && !consuming, || {
                        self.output(ty, child, extra, arg, borrowed || !consuming)
                    })?;
                    self.prefix(&mut p.outcomes.success, ComponentSegment::View(index));
                    bodies.push((consuming, std::mem::take(&mut p.statements)));
                    append(&mut out, p)?;
                }
                bodies.sort_by_key(|(consumes, _)| *consumes);
                for (_, body) in bodies {
                    out.statements.extend(body);
                }
            }
        }
        Ok(out)
    }

    fn output_owner_views(
        &self,
        ty: &Type,
        parts: &[JniOutput],
        extra: &C::Extra,
        value: TokenStream,
    ) -> Result<Option<CodecOutputPlan<C>>> {
        let owner = parts.iter().position(|part| {
            matches!(part,
            JniOutput::Leaf(JniOutputLeafOp::Projected(p)) if p.mode == ProjectionMode::Owned && p.fields.is_empty())
                || matches!(part, JniOutput::Leaf(JniOutputLeafOp::Value)) && self.codec.output_consumes(self.flat, ty, extra)
        });
        let Some(owner) = owner else {
            return Ok(None);
        };
        if parts.len() < 2 {
            return Ok(None);
        }
        // Explicit copy projections use observation-before-move ordering. They
        // do not require an address-preserving loan from the delivered owner.
        if parts
            .iter()
            .any(|part| matches!(part,JniOutput::Leaf(JniOutputLeafOp::Projected(p)) if p.mode==ProjectionMode::Borrowed))
        {
            return Ok(None);
        }
        let views = parts
            .iter()
            .map(|part| match part {
                JniOutput::Unary {
                    operation: JniOutputUnary::Getter { function },
                    child,
                } => Some((function.as_str(), child.as_ref())),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.cx
            .output_owner_getters(ty, &views, owner, extra, value)
    }
}

fn require_bulk<C: Codec + ?Sized>(codec: &C, direction: Direction) -> Result<()> {
    if codec.supports_bulk(direction) {
        Ok(())
    } else {
        Err(match direction {
            Direction::Input => "bulk inputs are unsupported by this codec",
            Direction::Output => "bulk outputs are unsupported by this codec",
        }
        .into())
    }
}
