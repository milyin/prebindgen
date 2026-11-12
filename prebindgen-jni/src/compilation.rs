//! Interpretation of this adapter's resolved conversion operations.
#![allow(missing_docs)]
use super::operations::{
    Input, InputBranch, InputLeafOp, InputOperations, InputParts, InputUnary, Output, OutputBranch,
    OutputLeafOp, OutputOperations, OutputParts, OutputUnary,
};
use prebindgen_declarations::*;
use prebindgen_flat::{Flat, Result};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;
#[derive(Default)]
pub struct State {
    bulk: std::cell::Cell<bool>,
    ordinary_collections: std::cell::Cell<bool>,
}
pub(crate) struct LanguageCompiler<'a, 'b, C: Codec> {
    pub(crate) cx: &'a Compiler<'b, C>,
}
impl<'b, C: Codec> std::ops::Deref for LanguageCompiler<'_, 'b, C> {
    type Target = Compiler<'b, C>;
    fn deref(&self) -> &Self::Target {
        self.cx
    }
}
trait Consumes {
    fn consumes<
        C: Codec<
            InputOperations = InputOperations,
            OutputOperations = OutputOperations,
            CompileState = State,
        >,
    >(
        &self,
        flat: &Flat<C::Schema>,
        ty: &Type,
        extra: &C::Extra,
        codec: &C,
        borrowed: bool,
    ) -> bool;
}
impl Consumes for Output {
    fn consumes<
        C: Codec<
            InputOperations = InputOperations,
            OutputOperations = OutputOperations,
            CompileState = State,
        >,
    >(
        &self,
        flat: &prebindgen_flat::Flat<C::Schema>,
        ty: &Type,
        extra: &C::Extra,
        codec: &C,
        borrowed: bool,
    ) -> bool {
        match self {
            Output::Unary {
                operation: OutputUnary::Convert { conversion, .. },
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
            Output::Unary {
                operation: OutputUnary::Sequence { mode, .. },
                ..
            } => match mode {
                Some(mode) => *mode == ProjectionMode::Owned,
                None => !borrowed && sequence_parts(ty).is_some_and(|(_, view)| view.is_owned()),
            },
            Output::Unary {
                operation: OutputUnary::Bulk,
                child,
            } => child.consumes(flat, ty, extra, codec, borrowed),
            Output::Unary {
                operation: OutputUnary::Named { .. },
                child,
                ..
            } => child.consumes(flat, ty, extra, codec, borrowed),
            Output::Unary {
                operation: OutputUnary::Optional,
                child,
            } => child.consumes(flat, option_inner(ty).unwrap_or(ty), extra, codec, borrowed),
            Output::Leaf(OutputLeafOp::Value) => {
                !borrowed && output_referent(ty).is_none() && codec.output_consumes(flat, ty, extra)
            }
            Output::Leaf(OutputLeafOp::Projected(p)) => p.mode == ProjectionMode::Owned,
            Output::Branch {
                operation: OutputBranch::OwnedFields,
                children: _,
            }
            | Output::Unary {
                operation: OutputUnary::Into { .. },
                ..
            }
            | Output::Unary {
                operation: OutputUnary::Result { .. },
                ..
            } => true,
            Output::Branch {
                operation: OutputBranch::Record { mode, .. },
                ..
            } => mode.map_or(!borrowed && output_referent(ty).is_none(), |m| {
                m == ProjectionMode::Owned
            }),
            Output::Branch {
                operation: OutputBranch::Enum { mode, .. },
                ..
            } => mode.map_or(!borrowed && output_referent(ty).is_none(), |m| {
                m == ProjectionMode::Owned
            }),
            Output::Request(never) => match *never {},
            Output::Branch {
                operation: OutputBranch::Combine,
                children: p,
            } => p
                .iter()
                .any(|p| p.consumes(flat, ty, extra, codec, borrowed)),
            _ => false,
        }
    }
}
impl<
        C: Codec<
            InputOperations = InputOperations,
            OutputOperations = OutputOperations,
            CompileState = State,
        >,
    > LanguageCompiler<'_, '_, C>
{
    pub(crate) fn input_named(
        &self,
        ty: &Type,
        recipe: &Input,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInput<C>> {
        self.input_unprepared_named(ty, recipe, extra, active, names)
    }

    pub(crate) fn input_unprepared_named(
        &self,
        ty: &Type,
        recipe: &Input,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInput<C>> {
        if prebindgen_flat::callback_signature(ty)?.is_some()
            && !matches!(
                recipe,
                Input::Leaf(InputLeafOp::Value) | Input::Arrow { .. }
            )
        {
            return Err("source callback parameters only support leaf inputs".into());
        }
        let var = names
            .map(|(_, var)| var.clone())
            .unwrap_or_else(|| self.name());
        match recipe {
            Input::None => self.cx.input_unit(ty),
            Input::Branch {
                operation: InputBranch::Variant,
                ..
            } => Err("variant branch requires an enum parent".into()),
            Input::Branch {
                operation: InputBranch::Enum { value },
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
            Input::Unary {
                operation: InputUnary::Convert { conversion },
                child: input,
            } => self.input_conversion(ty, conversion, input, extra, active, names),
            Input::Unary {
                operation: InputUnary::Temporary,
                child: inner,
            } => self.cx.input_temporary(ty, extra, active, names, inner),
            Input::Unary {
                operation: InputUnary::Bulk,
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
            Input::Arrow {
                operation: callback,
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
            Input::Unary {
                operation: InputUnary::Sequence,
                child: element_recipe,
            } => {
                if !self.state.bulk.get() {
                    if let Input::Unary {
                        operation: InputUnary::Bulk,
                        child: inner,
                    } = element_recipe.as_ref()
                    {
                        return self.input_named(
                            ty,
                            &Input::Unary {
                                operation: InputUnary::Bulk,
                                child: Box::new(Input::Unary {
                                    operation: InputUnary::Sequence,
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
            Input::Unary {
                operation: InputUnary::Optional,
                child: inner,
            } => self.cx.input_optional(ty, extra, active, names, var, inner),
            Input::Leaf(InputLeafOp::Value) => {
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
            Input::Request(never) => match *never {},
            Input::Branch {
                operation: InputBranch::Fields,
                children,
            }
            | Input::Branch {
                operation: InputBranch::Record,
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
                    Input::Branch {
                        operation: InputBranch::Record,
                        ..
                    }
                ),
            ),
            Input::Branch {
                operation: InputBranch::Constructor { function },
                children: args,
            } => self
                .cx
                .input_constructor(ty, extra, active, var, function, args),
            Input::Branch {
                operation: InputBranch::Selector,
                children: arms,
            } => self.cx.input_selector(ty, extra, active, var, arms),
        }
    }

    pub(crate) fn output_root(
        &self,
        ty: &Type,
        recipe: &Output,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        reference_return: bool,
    ) -> Result<CodecOutput<C>> {
        if let Output::Unary {
            operation: OutputUnary::Result { error },
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
        recipe: &Output,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<C>> {
        if !borrowed {
            if matches!(ty, Type::Reference(_)) {
                if let Output::Branch {
                    operation: OutputBranch::Combine,
                    children: parts,
                } = recipe
                {
                    return self.cx.output_reference_views(ty, parts, extra, value);
                }
            }
            if let (Type::Reference(reference), Output::Leaf(OutputLeafOp::Value)) = (ty, recipe) {
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
                Output::Unary {
                    operation: OutputUnary::Optional,
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
        recipe: &Output,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<C>> {
        self.output_named(ty, recipe, extra, value, borrowed, None)
    }
    pub(crate) fn output_named(
        &self,
        ty: &Type,
        recipe: &Output,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire_name: Option<&syn::Ident>,
    ) -> Result<CodecOutput<C>> {
        if let Some(referent) = output_referent(ty) {
            if matches ! (recipe , Output :: Leaf (OutputLeafOp :: Projected (p)) if p . mode == ProjectionMode :: Owned)
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
            Output::Unary {
                operation: OutputUnary::CallbackArgument { .. },
                ..
            } => return Err("callback argument ownership requires an arrow domain".into()),
            Output::None => {
                return self.cx.output_unit(ty, out);
            }
            Output::Arrow { operation, .. } => match *operation {},
            Output::Branch {
                operation: OutputBranch::Variant | OutputBranch::Arguments,
                ..
            } => return Err("variant/argument branch requires its parent operation".into()),
            Output::Branch {
                operation:
                    OutputBranch::Enum {
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
            Output::Unary {
                operation: OutputUnary::Convert { conversion },
                child: output,
            } => return self.output_conversion(ty, conversion, output, extra, value, borrowed),
            Output::Unary {
                operation: OutputUnary::Bulk,
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
            Output::Leaf(OutputLeafOp::Discard) => return Ok(out),
            Output::Unary {
                operation: OutputUnary::Named { name },
                child: output,
            } => {
                return self
                    .cx
                    .output_named_view(ty, extra, value, borrowed, wire_name, name, output);
            }
            Output::Unary {
                operation: OutputUnary::Sequence { delivery, mode },
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
                        &Output::Unary {
                            operation: OutputUnary::Bulk,
                            child: Box::new(recipe.clone()),
                        },
                        extra,
                        value,
                        borrowed,
                        wire_name,
                    );
                }
                if !self.state.bulk.get() && *delivery != SequenceDelivery::Callback {
                    if let Output::Unary {
                        operation: OutputUnary::Bulk,
                        child: inner,
                    } = element.as_ref()
                    {
                        return self.output_named(
                            ty,
                            &Output::Unary {
                                operation: OutputUnary::Bulk,
                                child: Box::new(Output::Unary {
                                    operation: OutputUnary::Sequence {
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
            Output::Unary {
                operation: OutputUnary::Result { .. },
                ..
            } => {
                return Err("Result error routing is supported only at the outermost return".into())
            }
            Output::Unary {
                operation: OutputUnary::Optional,
                child: inner,
            } => {
                return self
                    .cx
                    .output_optional(ty, extra, value, borrowed, wire_name, inner);
            }
            Output::Request(never) => match *never {},
            Output::Leaf(OutputLeafOp::Value) => {
                let representation = if self.state.bulk.get() {
                    Representation::Bulk
                } else {
                    Representation::Ordinary
                };
                return self
                    .cx
                    .output_value(ty, extra, value, borrowed, wire_name, representation);
            }
            Output::Leaf(OutputLeafOp::Projected(projection)) => {
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
            Output::Branch {
                operation: OutputBranch::Fields,
                children,
            }
            | Output::Branch {
                operation: OutputBranch::OwnedFields,
                children,
            }
            | Output::Branch {
                operation: OutputBranch::Record { .. },
                children,
                ..
            } => {
                let grouped = matches!(
                    recipe,
                    Output::Branch {
                        operation: OutputBranch::Record { .. },
                        ..
                    }
                );
                let owned = match recipe {
                    Output::Branch {
                        operation: OutputBranch::OwnedFields,
                        ..
                    } => true,
                    Output::Branch {
                        operation: OutputBranch::Record { mode },
                        ..
                    } => mode.map_or(!borrowed, |m| m == ProjectionMode::Owned),
                    _ => false,
                };
                let explicit_copy = matches!(
                    recipe,
                    Output::Branch {
                        operation: OutputBranch::Record {
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
            Output::Unary {
                operation: OutputUnary::Getter { function },
                child: output,
            }
            | Output::Unary {
                operation: OutputUnary::Into { function },
                child: output,
            } => {
                let consuming = matches!(
                    recipe,
                    Output::Unary {
                        operation: OutputUnary::Into { .. },
                        ..
                    }
                );
                return self
                    .cx
                    .output_getter(ty, extra, value, borrowed, function, output, consuming, out);
            }
            Output::Branch {
                operation: OutputBranch::Combine,
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
        parts: &[Output],
        extra: &C::Extra,
        value: TokenStream,
    ) -> Result<Option<CodecOutput<C>>> {
        let owner = parts.iter().position(|part| {
            matches!(part,
            Output::Leaf(OutputLeafOp::Projected(p)) if p.mode == ProjectionMode::Owned && p.fields.is_empty())
                || matches!(part, Output::Leaf(OutputLeafOp::Value)) && self.codec.output_consumes(self.flat, ty, extra)
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
            .any(|part| matches!(part,Output::Leaf(OutputLeafOp::Projected(p)) if p.mode==ProjectionMode::Borrowed))
        {
            return Ok(None);
        }
        let views = parts
            .iter()
            .map(|part| match part {
                Output::Unary {
                    operation: OutputUnary::Getter { function },
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
