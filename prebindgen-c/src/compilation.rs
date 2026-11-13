//! Interpretation of this adapter's resolved conversion operations.
#![allow(missing_docs)]
use crate::{CInputExt, COutputExt};

use super::operations::{
    CInput, CInputBranch, CInputLeafOp, CInputOps, CInputUnary, COutput, COutputBranch,
    COutputLeafOp, COutputOps, COutputUnary,
};
use prebindgen_declarations::*;
use prebindgen_flat::{Flat, Result};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;
#[derive(Default)]
pub struct CCompileState;
pub(crate) struct CCompiler<'a, 'b, C: Codec> {
    pub(crate) cx: &'a Compiler<'b, C>,
}
impl<'b, C: Codec> std::ops::Deref for CCompiler<'_, 'b, C> {
    type Target = Compiler<'b, C>;
    fn deref(&self) -> &Self::Target {
        self.cx
    }
}
trait Consumes {
    fn consumes<C: Codec<InOps = CInputOps, OutOps = COutputOps, CompileState = CCompileState>>(
        &self,
        flat: &Flat<C::Schema>,
        ty: &Type,
        extra: &C::Extra,
        codec: &C,
        borrowed: bool,
    ) -> bool;
}
impl Consumes for COutput {
    fn consumes<C: Codec<InOps = CInputOps, OutOps = COutputOps, CompileState = CCompileState>>(
        &self,
        flat: &prebindgen_flat::Flat<C::Schema>,
        ty: &Type,
        extra: &C::Extra,
        codec: &C,
        borrowed: bool,
    ) -> bool {
        match self {
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Convert {
                        conversion,
                        ..
                    }),
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
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Sequence {
                        mode, ..
                    }),
                ..
            } => match mode {
                Some(mode) => *mode == ProjectionMode::Owned,
                None => !borrowed && sequence_parts(ty).is_some_and(|(_, view)| view.is_owned()),
            },
            COutput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(COutputUnary::Named { .. }),
                child,
                ..
            } => child.consumes(flat, ty, extra, codec, borrowed),
            COutput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(COutputUnary::Optional),
                child,
            } => child.consumes(flat, option_inner(ty).unwrap_or(ty), extra, codec, borrowed),
            COutput::Leaf(COutputLeafOp::Value) => {
                !borrowed && output_referent(ty).is_none() && codec.output_consumes(flat, ty, extra)
            }
            COutput::Leaf(COutputLeafOp::Projected(p)) => p.mode == ProjectionMode::Owned,
            COutput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(COutputBranch::OwnedFields),
                children: _,
            }
            | COutput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(COutputUnary::Into { .. }),
                ..
            }
            | COutput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(COutputUnary::Result { .. }),
                ..
            } => true,
            COutput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputBranch::Enum { mode, .. }),
                ..
            } => mode.map_or(!borrowed && output_referent(ty).is_none(), |m| {
                m == ProjectionMode::Owned
            }),
            COutput::Request(never) => match *never {},
            COutput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(COutputBranch::Combine),
                children: p,
            } => p
                .iter()
                .any(|p| p.consumes(flat, ty, extra, codec, borrowed)),
            _ => false,
        }
    }
}
impl<C: Codec<InOps = CInputOps, OutOps = COutputOps, CompileState = CCompileState>>
    CCompiler<'_, '_, C>
{
    pub(crate) fn input_named(
        &self,
        ty: &Type,
        recipe: &CInput,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInputPlan<C>> {
        self.input_unprepared_named(ty, recipe, extra, active, names)
    }

    pub(crate) fn input_unprepared_named(
        &self,
        ty: &Type,
        recipe: &CInput,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInputPlan<C>> {
        if prebindgen_flat::callback_signature(ty)?.is_some()
            && !matches!(
                recipe,
                CInput::Leaf(CInputLeafOp::Value) | CInput::Arrow { .. }
            )
        {
            return Err("source callback parameters only support leaf inputs".into());
        }
        let var = names
            .map(|(_, var)| var.clone())
            .unwrap_or_else(|| self.name());
        match recipe {
            CInput::None => self.cx.input_unit(ty),
            CInput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(CInputBranch::Variant),
                ..
            } => Err("variant branch requires an enum parent".into()),
            CInput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(CInputBranch::Enum { value }),
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
            CInput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(CInputUnary::Convert { conversion }),
                child: input,
            } => self.input_conversion(ty, conversion, input, extra, active, names),
            CInput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(CInputUnary::Temporary),
                child: inner,
            } => self.cx.input_temporary(ty, extra, active, names, inner),
            CInput::Arrow {
                operation: prebindgen_declarations::Operation::Resolved(callback),
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
                    operation: callback,
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
            CInput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(CInputUnary::Sequence),
                child: element_recipe,
            } => self
                .cx
                .input_sequence(ty, element_recipe, extra, active, names, var, false),
            CInput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(CInputUnary::Optional),
                child: inner,
            } => self.cx.input_optional(ty, extra, active, names, var, inner),
            CInput::Leaf(CInputLeafOp::Value) => {
                let w = names
                    .map(|(wire, _)| wire.clone())
                    .unwrap_or_else(|| self.name());
                let representation = Representation::Ordinary;
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
            CInput::Request(never) => match *never {},
            CInput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(CInputBranch::Fields),
                children,
            } => self
                .cx
                .input_fields(ty, extra, active, names, var, children, false),
            CInput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(CInputBranch::Constructor { function }),
                children: args,
            } => self
                .cx
                .input_constructor(ty, extra, active, var, function, args),
            CInput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(CInputBranch::Selector),
                children: arms,
            } => self.cx.input_selector(ty, extra, active, var, arms),
            CInput::Unary {
                operation: prebindgen_declarations::Operation::Request(never),
                ..
            }
            | CInput::Branch {
                operation: prebindgen_declarations::Operation::Request(never),
                ..
            }
            | CInput::Arrow {
                operation: prebindgen_declarations::Operation::Request(never),
                ..
            } => match *never {},
        }
    }

    pub(crate) fn output_root(
        &self,
        ty: &Type,
        recipe: &COutput,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        reference_return: bool,
    ) -> Result<CodecOutputPlan<C>> {
        if let COutput::Unary {
            operation: prebindgen_declarations::Operation::Resolved(COutputUnary::Result { error }),
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
        recipe: &COutput,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutputPlan<C>> {
        if !borrowed {
            if matches!(ty, Type::Reference(_)) {
                if let COutput::Branch {
                    operation: prebindgen_declarations::Operation::Resolved(COutputBranch::Combine),
                    children: parts,
                } = recipe
                {
                    return self.cx.output_reference_views(ty, parts, extra, value);
                }
            }
            if let (Type::Reference(reference), COutput::Leaf(COutputLeafOp::Value)) = (ty, recipe)
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
                COutput::Unary {
                    operation: prebindgen_declarations::Operation::Resolved(COutputUnary::Optional),
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
        recipe: &COutput,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutputPlan<C>> {
        self.output_named(ty, recipe, extra, value, borrowed, None)
    }
    pub(crate) fn output_named(
        &self,
        ty: &Type,
        recipe: &COutput,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire_name: Option<&syn::Ident>,
    ) -> Result<CodecOutputPlan<C>> {
        if let Some(referent) = output_referent(ty) {
            if matches ! (recipe , COutput :: Leaf (COutputLeafOp :: Projected (p)) if p . mode == ProjectionMode :: Owned)
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
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::CallbackArgument {
                        ..
                    }),
                ..
            } => return Err("callback argument ownership requires an arrow domain".into()),
            COutput::None => {
                return self.cx.output_unit(ty, out);
            }
            COutput::Arrow {
                operation: prebindgen_declarations::Operation::Resolved(never),
                ..
            } => match *never {},
            COutput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(
                        COutputBranch::Variant | COutputBranch::Arguments,
                    ),
                ..
            } => return Err("variant/argument branch requires its parent operation".into()),
            COutput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputBranch::Enum {
                        value: grouped,
                        mode,
                    }),
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
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Convert { conversion }),
                child: output,
            } => return self.output_conversion(ty, conversion, output, extra, value, borrowed),
            COutput::Leaf(COutputLeafOp::Discard) => return Ok(out),
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Named { name }),
                child: output,
            } => {
                return self
                    .cx
                    .output_named_view(ty, extra, value, borrowed, wire_name, name, output);
            }
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Sequence {
                        delivery,
                        mode,
                    }),
                child: element,
            } => {
                return self
                    .cx
                    .output_sequence(ty, element, extra, value, borrowed, delivery, mode, false);
            }
            COutput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(COutputUnary::Result { .. }),
                ..
            } => {
                return Err("Result error routing is supported only at the outermost return".into())
            }
            COutput::Unary {
                operation: prebindgen_declarations::Operation::Resolved(COutputUnary::Optional),
                child: inner,
            } => {
                return self
                    .cx
                    .output_optional(ty, extra, value, borrowed, wire_name, inner);
            }
            COutput::Request(never) => match *never {},
            COutput::Leaf(COutputLeafOp::Value) => {
                let representation = Representation::Ordinary;
                return self
                    .cx
                    .output_value(ty, extra, value, borrowed, wire_name, representation);
            }
            COutput::Leaf(COutputLeafOp::Projected(projection)) => {
                let representation = Representation::Ordinary;
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
            COutput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(COutputBranch::Fields),
                children,
            }
            | COutput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(COutputBranch::OwnedFields),
                children,
            } => {
                return self.cx.output_fields(
                    ty,
                    extra,
                    value,
                    borrowed,
                    wire_name,
                    children,
                    false,
                    matches!(
                        recipe,
                        COutput::Branch {
                            operation: prebindgen_declarations::Operation::Resolved(
                                COutputBranch::OwnedFields
                            ),
                            ..
                        }
                    ),
                    false,
                    out,
                );
            }
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Getter { function }),
                child: output,
            }
            | COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Into { function }),
                child: output,
            } => {
                let consuming = matches!(
                    recipe,
                    COutput::Unary {
                        operation: prebindgen_declarations::Operation::Resolved(
                            COutputUnary::Into { .. }
                        ),
                        ..
                    }
                );
                return self
                    .cx
                    .output_getter(ty, extra, value, borrowed, function, output, consuming, out);
            }
            COutput::Branch {
                operation: prebindgen_declarations::Operation::Resolved(COutputBranch::Combine),
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
            COutput::Unary {
                operation: prebindgen_declarations::Operation::Request(never),
                ..
            }
            | COutput::Branch {
                operation: prebindgen_declarations::Operation::Request(never),
                ..
            }
            | COutput::Arrow {
                operation: prebindgen_declarations::Operation::Request(never),
                ..
            } => match *never {},
        }
        Ok(out)
    }

    fn output_owner_views(
        &self,
        ty: &Type,
        parts: &[COutput],
        extra: &C::Extra,
        value: TokenStream,
    ) -> Result<Option<CodecOutputPlan<C>>> {
        let owner = parts.iter().position(|part| {
            matches!(part,
            COutput::Leaf(COutputLeafOp::Projected(p)) if p.mode == ProjectionMode::Owned && p.fields.is_empty())
                || matches!(part, COutput::Leaf(COutputLeafOp::Value)) && self.codec.output_consumes(self.flat, ty, extra)
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
            .any(|part| matches!(part,COutput::Leaf(COutputLeafOp::Projected(p)) if p.mode==ProjectionMode::Borrowed))
        {
            return Ok(None);
        }
        let views =
            parts
                .iter()
                .map(|part| match part {
                    COutput::Unary {
                        operation:
                            prebindgen_declarations::Operation::Resolved(COutputUnary::Getter {
                                function,
                            }),
                        child,
                    } => Some((function.as_str(), child.as_ref())),
                    _ => None,
                })
                .collect::<Vec<_>>();
        self.cx
            .output_owner_getters(ty, &views, owner, extra, value)
    }
}
