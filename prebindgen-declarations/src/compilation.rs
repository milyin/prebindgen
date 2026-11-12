//! Reusable concrete conversion operations. Adapters choose when to call them.
use super::*;
use prebindgen_flat::{ValueConversion, ValueSource};
use std::collections::BTreeMap;
#[allow(missing_docs, clippy::too_many_arguments)]
impl<C: Codec> Compiler<'_, C> {
    pub fn with_temporary_receiver<T>(
        &self,
        temporary: bool,
        prepare: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let previous = self.temporary_receiver.get();
        self.temporary_receiver.set(previous || temporary);
        let result = prepare();
        self.temporary_receiver.set(previous);
        result
    }
    pub fn with_copy_permission<T>(
        &self,
        explicit: bool,
        prepare: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let previous = self
            .copy_borrowed
            .replace(self.copy_borrowed.get() || explicit);
        let result = prepare();
        self.copy_borrowed.set(previous);
        result
    }
    pub fn name(&self) -> syn::Ident {
        let next = self.next.get();
        let n = format_ident!("{}_w{next}", self.prefix);
        self.next.set(next + 1);
        n
    }
    pub fn prefix(
        &self,
        wires: &mut prebindgen_flat::WireNode<C::Wire, C::Chain, C::Branch>,
        segment: ComponentSegment,
    ) {
        self.codec.prefix_layout(wires, segment);
    }
    pub fn empty_output(&self) -> Result<CodecOutput<C>> {
        let out = OutputPlan {
            fallible: false,
            context: Default::default(),
            delivery: Default::default(),
            commit: TokenStream::new(),
            outcomes: prebindgen_flat::Outcomes {
                success: WireNode::Branch {
                    controls: vec![],
                    children: vec![],
                    meta: self.codec.group(GroupKind::Values)?,
                },
                source_error: None,
                binding_failure: None,
            },
            statements: TokenStream::new(),
            values: Vec::new(),
        };
        Ok(out)
    }
    pub fn input_unit(&self, ty: &Type) -> Result<CodecInput<C>> {
        {
            if type_key(ty) != "()" {
                return Err("None input requires unit".into());
            }
            Ok(InputPlan::new(
                WireNode::Branch {
                    controls: vec![],
                    children: vec![],
                    meta: self.codec.group(GroupKind::Fields)?,
                },
                quote!(),
                quote!(()),
            ))
        }
    }
    pub fn input_temporary(
        &self,
        ty: &Type,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
        inner: &ResolvedInput<C>,
    ) -> Result<CodecInput<C>> {
        {
            let owned = temporary_input_type(ty)?;
            let mut child = self.input_named(&owned, inner, extra, active, names)?;
            let storage = self.name();
            let target = format_ident!("{storage}_owned");
            let mut backing = std::mem::take(&mut child.storage);
            backing . extend (quote!(# [allow (unused_mut , unused_assignments)] let mut # storage : Option <# owned >= None ;)) ;
            let build = child.bind(&target, quote!(# storage = Some (# target) ;))?;
            let Type::Reference(reference) = ty else {
                unreachable!()
            };
            let elem = &reference.elem;
            let value = if matches!(elem.as_ref(), Type::Slice(_))
                || matches!(elem . as_ref () , Type :: Path (p) if p . path . is_ident ("str"))
            {
                quote!(:: core :: convert :: AsRef ::<# elem >:: as_ref (# storage . as_ref () . expect ("active temporary")))
            } else {
                quote!(# storage . as_ref () . expect ("active temporary"))
            };
            let mut plan = InputPlan::new(
                WireNode::Branch {
                    controls: vec![],
                    children: vec![child.node],
                    meta: self.codec.group(GroupKind::Values)?,
                },
                quote!(# build ;),
                value,
            );
            plan.storage = backing;
            plan.fallible = child.fallible;
            Ok(plan)
        }
    }
    pub fn input_optional(
        &self,
        ty: &Type,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
        var: syn::Ident,
        inner: &ResolvedInput<C>,
    ) -> Result<CodecInput<C>> {
        {
            let ty = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
            let tag = names
                .map(|(wire, _)| format_ident!("{wire}_present"))
                .unwrap_or_else(|| self.name());
            let child = if let Some((wire, var)) = names {
                self.input_named(
                    ty,
                    inner,
                    extra,
                    &active.arm(tag.clone(), 1),
                    Some((&format_ident!("{wire}_some"), &format_ident!("{var}_some"))),
                )?
            } else {
                self.input(ty, inner, extra, &active.arm(tag.clone(), 1))?
            };
            optional_input(self.codec, extra, tag, var, active, child)
        }
    }
    pub fn input_fields(
        &self,
        ty: &Type,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
        var: syn::Ident,
        children: &[ResolvedInput<C>],
        grouped: bool,
    ) -> Result<CodecInput<C>> {
        {
            if type_key(ty) == "()" && children.is_empty() {
                return Ok(InputPlan::new(
                    WireNode::Branch {
                        controls: vec![],
                        children: vec![],
                        meta: self.codec.group(GroupKind::Fields)?,
                    },
                    quote!(),
                    quote!(()),
                ));
            }
            let r = self.flat.record(ty)?;
            if r.fields.len() != children.len() {
                return Err(format!(
                    "`{}`: expected {} field converters, got {}",
                    type_key(ty),
                    r.fields.len(),
                    children.len()
                ));
            }
            let wire =
                grouped.then(|| names.map(|(w, _)| w.clone()).unwrap_or_else(|| self.name()));
            let mut plan = record_input(
                self.codec,
                ty,
                var.clone(),
                r.fields
                    .iter()
                    .zip(children)
                    .enumerate()
                    .map(|(index, (field, child))| {
                        let plan = if let Some(wire) = &wire {
                            self.input_named(
                                &field.ty,
                                child,
                                extra,
                                active,
                                Some((
                                    &format_ident!("{wire}_f{index}"),
                                    &format_ident!("{var}_f{index}"),
                                )),
                            )?
                        } else {
                            self.input(&field.ty, child, extra, active)?
                        };
                        Ok((field.name.clone(), plan))
                    }),
            )?;
            if let Some(wire) = wire {
                self.codec
                    .input_record_metadata(self.flat, ty, &wire, active, &mut plan, extra)?;
            }
            Ok(plan)
        }
    }
    pub fn input_constructor(
        &self,
        ty: &Type,
        extra: &C::Extra,
        active: &Condition,
        var: syn::Ident,
        function: &String,
        args: &[ResolvedInput<C>],
    ) -> Result<CodecInput<C>> {
        {
            let f = self.flat.function(function)?;
            if !same_type(&f.ret.ty, ty) {
                return Err(format!(
                    "constructor `{function}` returns `{}`, expected `{}`",
                    type_key(&f.ret.ty),
                    type_key(ty)
                ));
            }
            if f.params.len() != args.len() {
                return Err(format!("constructor `{function}`: wrong argument count"));
            }
            let mut plans = Vec::new();
            let mut call_args = Vec::new();
            let mut wires = Vec::new();
            let mut storage = TokenStream::new();
            let mut fallible = false;
            for (i, (param, child)) in f.params.iter().zip(args).enumerate() {
                let mut p = self.input(&param.ty.ty, child, extra, active)?;
                self.prefix(
                    &mut p.node,
                    ComponentSegment::Argument(param.name.to_string()),
                );
                fallible |= p.fallible;
                storage.extend(std::mem::take(&mut p.storage));
                let target = format_ident!("{var}_argument_{i}");
                call_args.push(target.clone());
                plans.push((target, p));
            }
            let path = &f.path;
            let mut plan = InputPlan::new(
                WireNode::default(),
                quote!(let # var :# ty =# path (# (# call_args) ,*) ;),
                quote!(# var),
            );
            for (target, p) in plans.iter().rev() {
                plan.code = p.bind(target, plan.code)?;
            }
            for (_, p) in plans {
                wires.push(p.node);
            }
            plan.node = WireNode::Branch {
                controls: vec![],
                children: wires,
                meta: self.codec.group(GroupKind::Values)?,
            };
            plan.storage = storage;
            plan.fallible = fallible;
            Ok(plan)
        }
    }
    pub fn input_selector(
        &self,
        ty: &Type,
        extra: &C::Extra,
        active: &Condition,
        var: syn::Ident,
        arms: &[ResolvedInput<C>],
    ) -> Result<CodecInput<C>> {
        {
            let mut fallible = self.codec.validates_inputs();
            let mut storage = TokenStream::new();
            let mut validation = TokenStream::new();
            if arms.is_empty() {
                return Err("an input selector needs at least one alternative".into());
            }
            if matches!(ty, Type::Reference(_)) {
                return Err("selector results must be owned in this subset".into());
            }
            let tag = self.name();
            let mut selector_layout = prebindgen_flat::WireNode::Leaf(vec![Wire {
                name: tag.clone(),
                kind: self.codec.selector(extra)?,
            }]);
            self.prefix(&mut selector_layout, ComponentSegment::Selector);
            let WireNode::Leaf(mut selector_values) = selector_layout else {
                return Err("selector prefixing must retain its value".into());
            };
            if selector_values.len() != 1 {
                return Err("selector prefixing must retain its value".into());
            }
            let selector = selector_values.remove(0);
            let mut alternatives = Vec::new();
            let count = arms.len() as i32;
            let invalid = active.and(quote!(! (0 ..# count) . contains (&# tag)));
            if self.codec.validates_inputs() {
                validation
                    .extend(quote!(if # invalid { return Err ("invalid selector" . into ()) ; }));
            }
            let mut cases = Vec::new();
            for (i, arm) in arms.iter().enumerate() {
                let mut p = self.input(ty, arm, extra, &active.arm(tag.clone(), i as i32))?;
                self.prefix(&mut p.node, ComponentSegment::Alternative(i));
                fallible |= p.fallible;
                storage.extend(std::mem::take(&mut p.storage));
                let target = format_ident!("{var}_alternative_{i}");
                let body = p.bind(&target, quote!(# target))?;
                alternatives.push(p.node);
                let i = i as i32;
                cases.push(quote!(# i =># body));
            }
            let metadata = self.codec.alternatives(Alternatives {
                selector: &selector,
                arms: &mut alternatives,
            })?;
            let node = WireNode::Branch {
                controls: vec![selector],
                meta: metadata,
                children: alternatives,
            };
            let invalid = if self.codec.validates_inputs() {
                quote!(unreachable!("validated selector"))
            } else {
                quote!(unsafe { ::core::hint::unreachable_unchecked() })
            };
            let mut plan = InputPlan::new(
                node,
                quote!(# validation let # var :# ty = match # tag { # (# cases ,) * _ =># invalid } ;),
                quote!(# var),
            );
            plan.storage = storage;
            plan.fallible = fallible;
            Ok(plan)
        }
    }
    pub fn output_unit(&self, ty: &Type, mut out: CodecOutput<C>) -> Result<CodecOutput<C>> {
        {
            if type_key(ty) != "()" {
                return Err("None output requires unit".into());
            }
            out.outcomes.success = WireNode::Branch {
                controls: vec![],
                children: vec![],
                meta: self.codec.group(GroupKind::Fields)?,
            };
            Ok(out)
        }
    }
    pub fn output_named_view(
        &self,
        ty: &Type,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire_name: Option<&syn::Ident>,
        name: &str,
        output: &ResolvedOutput<C>,
    ) -> Result<CodecOutput<C>> {
        {
            let mut plan = self.output_named(ty, output, extra, value, borrowed, wire_name)?;
            plan.outcomes.success = WireNode::Branch {
                meta: self.codec.group(GroupKind::NamedOutput(name))?,
                controls: vec![],
                children: vec![plan.outcomes.success],
            };
            Ok(plan)
        }
    }
    pub fn output_optional(
        &self,
        ty: &Type,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire_name: Option<&syn::Ident>,
        inner: &ResolvedOutput<C>,
    ) -> Result<CodecOutput<C>> {
        {
            let ty = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
            let tag = wire_name
                .map(|w| format_ident!("{w}_present"))
                .unwrap_or_else(|| self.name());
            let payload = wire_name
                .map(|w| format_ident!("{w}_payload"))
                .unwrap_or_else(|| self.name());
            let child_wire = wire_name.map(|w| format_ident!("{w}_some"));
            let child = self.output_named(
                ty,
                inner,
                extra,
                quote!(# payload),
                borrowed,
                child_wire.as_ref(),
            )?;
            optional_output(self.codec, extra, tag, payload, value, borrowed, child)
        }
    }
    pub fn output_value(
        &self,
        ty: &Type,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire_name: Option<&syn::Ident>,
        representation: Representation,
    ) -> Result<CodecOutput<C>> {
        {
            let request = OutputLeaf {
                flat: self.flat,
                ty,
                extra,
                wire: wire_name.cloned().unwrap_or_else(|| self.name()),
                value,
                access: if borrowed {
                    ValueAccess::Shared
                } else {
                    ValueAccess::Owned
                },
                representation,
            };
            if borrowed && self.copy_borrowed.get() {
                self.codec
                    .output_projected(request, &Projection::borrowed())
            } else {
                self.codec.output_leaf(request)
            }
        }
    }
    pub fn output_projection(
        &self,
        ty: &Type,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire_name: Option<&syn::Ident>,
        projection: &Projection,
        representation: Representation,
    ) -> Result<CodecOutput<C>> {
        {
            if borrowed && projection.mode == ProjectionMode::Owned {
                return Err("owned projection cannot consume a shared-reference occurrence".into());
            }
            let value = if !borrowed && projection.mode == ProjectionMode::Borrowed {
                quote!(& (# value))
            } else {
                value
            };
            self.codec.output_projected(
                OutputLeaf {
                    flat: self.flat,
                    ty,
                    extra,
                    wire: wire_name.cloned().unwrap_or_else(|| self.name()),
                    value,
                    access: if projection.mode == ProjectionMode::Borrowed {
                        ValueAccess::Shared
                    } else {
                        ValueAccess::Owned
                    },
                    representation,
                },
                projection,
            )
        }
    }
    pub fn output_fields(
        &self,
        ty: &Type,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire_name: Option<&syn::Ident>,
        children: &[ResolvedOutput<C>],
        grouped: bool,
        owned: bool,
        explicit_copy: bool,
        mut out: CodecOutput<C>,
    ) -> Result<CodecOutput<C>> {
        {
            let wire = grouped.then(|| wire_name.cloned().unwrap_or_else(|| self.name()));
            if owned && borrowed {
                return Err("owned fields cannot consume a shared-reference occurrence".into());
            }
            if type_key(ty) == "()" && children.is_empty() {
                out.outcomes.success = WireNode::Branch {
                    controls: vec![],
                    meta: self.codec.group(GroupKind::Fields)?,
                    children: vec![],
                };
                return Ok(out);
            }
            let r = self.flat.record(ty)?;
            if r.fields.len() != children.len() {
                return Err("wrong output field count".into());
            }
            let convert = |index: usize, value| {
                let child_wire = wire.as_ref().map(|w| format_ident!("{w}_f{index}"));
                self.output_named(
                    &r.fields[index].ty,
                    &children[index],
                    extra,
                    value,
                    !owned,
                    child_wire.as_ref(),
                )
            };
            let var = wire
                .as_ref()
                .map(|w| format_ident!("{w}_record"))
                .unwrap_or_else(|| self.name());
            let mut plan = self.with_copy_permission(explicit_copy, || {
                self.with_temporary_receiver(!owned && !borrowed, || {
                    if owned {
                        record_output_owned(
                            self.codec,
                            var,
                            value,
                            r.fields.iter().map(|f| f.name.clone()),
                            convert,
                        )
                    } else {
                        record_output(
                            self.codec,
                            var,
                            value,
                            r.fields.iter().map(|f| f.name.clone()),
                            convert,
                        )
                    }
                })
            })?;
            if let Some(wire) = wire {
                self.codec
                    .output_record_metadata(self.flat, ty, &wire, &mut plan, extra)?;
            }
            Ok(plan)
        }
    }
    pub fn output_getter(
        &self,
        ty: &Type,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        function: &String,
        output: &ResolvedOutput<C>,
        consuming: bool,
        mut out: CodecOutput<C>,
    ) -> Result<CodecOutput<C>> {
        {
            if consuming && borrowed {
                return Err(
                    "consuming accessor cannot consume a shared-reference occurrence".into(),
                );
            }
            let f = self.flat.function(function)?;
            let expected: Type = if consuming {
                ty.clone()
            } else {
                syn::parse_quote!(&# ty)
            };
            if f.params.len() != 1 || !same_type(&f.params[0].ty.ty, &expected) {
                return Err(format!(
                    "getter `{function}` must take exactly `{}`",
                    type_key(&expected)
                ));
            }
            if matches!(& f . ret . ty , Type :: Reference (r) if r . mutability . is_some ()) {
                return Err("mutable-reference getter results are unsupported".into());
            }
            let v = self.name();
            let path = &f.path;
            let arg = if borrowed || consuming {
                value
            } else {
                quote!(&# value)
            };
            out.statements = quote!(let # v =# path (# arg) ;);
            let mut child = self.output_return(&f.ret.ty, output, extra, quote!(# v), false)?;
            self.codec.validate_getter_output(
                self.flat,
                ty,
                borrowed && !self.temporary_receiver.get(),
                &f.ret.ty,
                &child,
                extra,
            )?;
            self.prefix(
                &mut child.outcomes.success,
                ComponentSegment::Getter(function.clone()),
            );
            append(&mut out, child)?;
            Ok(out)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn input_conversion(
        &self,
        ty: &Type,
        conversion: &ValueConversion,
        input: &ResolvedInput<C>,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInput<C>> {
        crate::convert::validate_conversion(self.flat, ty, conversion, true)?;
        let target = names.map(|(_, v)| v.clone()).unwrap_or_else(|| self.name());
        let repr = format_ident!("{target}_repr");
        let decoded = format_ident!("{repr}_decoded");
        let child_names = names.map(|(w, _)| (w, &decoded));
        let mut child = self.input_named(
            &conversion.representation,
            input,
            extra,
            active,
            child_names,
        )?;
        let (expr, fallible) = match &conversion.source {
            ValueSource::Function(name) => {
                let f = self.flat.function(name)?;
                let path = &f.path;
                (
                    quote!(#path(#repr)),
                    prebindgen_flat::result_parts(&f.ret.ty).is_some() && !same_type(&f.ret.ty, ty),
                )
            }
            ValueSource::Into { fallible } => {
                let r = &conversion.representation;
                if *fallible {
                    (
                        quote!(<#r as ::core::convert::TryInto<#ty>>::try_into(#repr)),
                        true,
                    )
                } else {
                    (
                        quote!(<#r as ::core::convert::Into<#ty>>::into(#repr)),
                        false,
                    )
                }
            }
        };
        let expr = if fallible {
            quote!((#expr).map_err(|error|::std::string::ToString::to_string(&error))?)
        } else {
            expr
        };
        let rest: CodecInput<C> = prebindgen_conversions::InputPlan::new(
            child.node.clone(),
            quote!(let #target:#ty=#expr;),
            quote!(#target),
        );
        let storage = std::mem::take(&mut child.storage);
        let code = child.bind(&repr, rest.code)?;
        Ok(prebindgen_conversions::InputPlan {
            fallible: child.fallible || fallible,
            node: child.node,
            storage,
            code,
            context: Default::default(),
        })
    }
    pub fn output_conversion(
        &self,
        ty: &Type,
        conversion: &ValueConversion,
        output: &ResolvedOutput<C>,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<C>> {
        crate::convert::validate_conversion(self.flat, ty, conversion, false)?;
        let borrowed = borrowed || matches!(ty, Type::Reference(_));
        let source = self.name();
        let repr = self.name();
        let (expr, fallible) = match &conversion.source {
            ValueSource::Function(name) => {
                let f = self.flat.function(name)?;
                let path = &f.path;
                let shared =
                    matches!(&f.params[0].ty.ty,Type::Reference(r) if r.mutability.is_none());
                if !shared && borrowed {
                    return Err(format!("consuming conversion `{name}` cannot convert a borrowed source; use a shared-reference conversion"));
                }
                let argument = if shared && !borrowed {
                    quote!(&#source)
                } else {
                    quote!(#source)
                };
                (
                    quote!(#path(#argument)),
                    prebindgen_flat::result_parts(&f.ret.ty).is_some()
                        && !same_type(&f.ret.ty, &conversion.representation),
                )
            }
            ValueSource::Into { fallible } => {
                if borrowed {
                    return Err("consuming trait conversion cannot convert a borrowed source; use a shared-reference helper".into());
                }
                let r = &conversion.representation;
                if *fallible {
                    (
                        quote!(<#ty as ::core::convert::TryInto<#r>>::try_into(#source)),
                        true,
                    )
                } else {
                    (
                        quote!(<#ty as ::core::convert::Into<#r>>::into(#source)),
                        false,
                    )
                }
            }
        };
        let expr = if fallible {
            quote!((#expr).map_err(|error|::std::string::ToString::to_string(&error))?)
        } else {
            expr
        };
        let mut child = self.output_return(
            &conversion.representation,
            output,
            extra,
            quote!(#repr),
            false,
        )?;
        self.codec.validate_getter_output(
            self.flat,
            ty,
            borrowed && !self.temporary_receiver.get(),
            &conversion.representation,
            &child,
            extra,
        )?;
        let prepared = child.statements;
        child.statements = quote!(let #source=#value;let #repr=#expr;#prepared);
        child.fallible |= fallible;
        Ok(child)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn enum_input(
        &self,
        ty: &Type,
        recipes: &[&[ResolvedInput<C>]],
        value: bool,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInput<C>> {
        if matches!(ty, Type::Reference(_)) {
            return Err("enum-field input constructs an owned value; select reconstruct() to borrow a temporary".into());
        }
        let source = self.flat.payload_enum(ty)?;
        if source.variants.len() != recipes.len() {
            return Err("wrong enum variant count".into());
        }
        i32::try_from(recipes.len()).map_err(|_| "too many enum variants")?;
        let root = names.map(|(w, _)| w.clone()).unwrap_or_else(|| self.name());
        let var = names.map(|(_, v)| v.clone()).unwrap_or_else(|| self.name());
        let tag = format_ident!("{root}_tag");
        let mut selector = Wire {
            name: tag.clone(),
            kind: self.codec.selector(extra)?,
        };
        let mut arms = vec![];
        let mut cases = vec![];
        let mut storage = TokenStream::new();
        let mut fallible = self.codec.validates_inputs();
        for (index, (arm, recipes)) in source.variants.iter().zip(recipes).enumerate() {
            let recipes = *recipes;
            if arm.fields.len() != recipes.len() {
                return Err("wrong enum field count".into());
            }
            let mut plans = vec![];
            let mut arguments = vec![];
            let mut children = vec![];
            let condition = active.arm(tag.clone(), index as i32);
            for (field_index, (field, recipe)) in arm.fields.iter().zip(recipes).enumerate() {
                let wire = format_ident!("{root}_a{index}_f{field_index}");
                let target = format_ident!("{var}_a{index}_f{field_index}");
                let converted = format_ident!("{target}_value");
                let previous = self.input_role.get();
                if value {
                    self.input_role.set(InputRole::EnumField);
                }
                let result = self.input_named(
                    &field.ty,
                    recipe,
                    extra,
                    &condition,
                    Some((&wire, &converted)),
                );
                self.input_role.set(previous);
                let mut child = result
                    .map_err(|e| format!("{}::{}.{}: {e}", type_key(ty), arm.name, field.key()))?;
                let name = format_ident!("{}", field.property());
                if arm.fields.len() != 1 || field.key() != "0" {
                    self.prefix(&mut child.node, ComponentSegment::Field(field.property()));
                }
                child.node = WireNode::Branch {
                    controls: vec![],
                    children: vec![child.node],
                    meta: self.codec.group(GroupKind::Field(&name))?,
                };
                storage.extend(std::mem::take(&mut child.storage));
                fallible |= child.fallible;
                arguments.push(quote!(#target));
                children.push(child.node.clone());
                plans.push((target, child));
            }
            let constructed = source.spell(index, &arguments)?;
            let mut body = quote!(#constructed);
            for (target, child) in plans.iter().rev() {
                body = child.bind(target, body)?;
            }
            let index = index as i32;
            cases.push(quote!(#index=>{#body}));
            let mut arm_node = WireNode::Branch {
                controls: vec![],
                children,
                meta: self.codec.group(GroupKind::Fields)?,
            };
            self.prefix(
                &mut arm_node,
                ComponentSegment::Named(arm.name.to_string().trim_start_matches("r#").into()),
            );
            arms.push(arm_node);
        }
        let encoding = self.codec.enum_variants(EnumVariants {
            flat: self.flat,
            ty,
            extra,
            selector: &mut selector,
            arms: &mut arms,
            form: if value {
                EnumForm::Value
            } else {
                EnumForm::Fields
            },
            direction: Direction::Input,
        })?;
        let invalid = if self.codec.validates_inputs() {
            quote!(return Err("invalid enum tag".into()))
        } else {
            quote!(unsafe { ::core::hint::unreachable_unchecked() })
        };
        let mut plan = InputPlan::new(
            WireNode::Branch {
                controls: vec![selector],
                children: arms,
                meta: encoding.metadata,
            },
            quote!(let #var:#ty=match #tag {#(#cases,)* _=>#invalid};),
            quote!(#var),
        );
        plan.storage = storage;
        plan.fallible = fallible;
        if value {
            self.codec.input_enum(
                self.flat,
                ty,
                extra,
                root,
                plan,
                active,
                self.input_role.get(),
            )
        } else {
            Ok(plan)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn enum_output(
        &self,
        ty: &Type,
        recipes: &[&[ResolvedOutput<C>]],
        grouped: bool,
        mode: Option<ProjectionMode>,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire: Option<&syn::Ident>,
    ) -> Result<CodecOutput<C>> {
        let owned = mode.map_or(!borrowed, |m| m == ProjectionMode::Owned);
        if owned && borrowed {
            return Err("owned enum fields cannot consume a shared-reference occurrence".into());
        }
        let temporary_observation = !owned && !borrowed;
        let observed = format_ident!(
            "__enum_observed_{}",
            wire.cloned().unwrap_or_else(|| self.name())
        );
        let prepare = if !owned && !borrowed {
            quote!(let #observed=#value;)
        } else {
            TokenStream::new()
        };
        let value = if !owned && !borrowed {
            quote!(&#observed)
        } else {
            value
        };
        let borrowed = !owned;
        let source = self.flat.payload_enum(ty)?;
        if source.variants.len() != recipes.len() {
            return Err("wrong enum variant count".into());
        }
        i32::try_from(recipes.len()).map_err(|_| "too many enum variants")?;
        let root = wire.cloned().unwrap_or_else(|| self.name());
        let tag = format_ident!("{root}_tag");
        let mut selector = Wire {
            name: tag.clone(),
            kind: self.codec.selector(extra)?,
        };
        let mut plans = vec![];
        let mut patterns = vec![];
        for (index, (arm, recipes)) in source.variants.iter().zip(recipes).enumerate() {
            let recipes = *recipes;
            if arm.fields.len() != recipes.len() {
                return Err("wrong enum field count".into());
            }
            let bindings = (0..arm.fields.len())
                .map(|i| format_ident!("__{root}_a{index}_b{i}"))
                .collect::<Vec<_>>();
            patterns.push(source.spell(
                index,
                &bindings.iter().map(|b| quote!(#b)).collect::<Vec<_>>(),
            )?);
            let mut child = OutputPlan {
                fallible: false,
                context: Default::default(),
                delivery: Default::default(),
                commit: TokenStream::new(),
                statements: TokenStream::new(),
                values: vec![],
                outcomes: prebindgen_flat::Outcomes {
                    success: WireNode::Branch {
                        controls: vec![],
                        children: vec![],
                        meta: self.codec.group(GroupKind::Fields)?,
                    },
                    source_error: None,
                    binding_failure: None,
                },
            };
            for (field_index, (field, recipe)) in arm.fields.iter().zip(recipes).enumerate() {
                let binding = &bindings[field_index];
                let wire = format_ident!("{root}_a{index}_f{field_index}");
                let mut field_plan = self
                    .with_copy_permission(mode == Some(ProjectionMode::Borrowed), || {
                        self.with_temporary_receiver(temporary_observation, || {
                            self.output_named(
                                &field.ty,
                                recipe,
                                extra,
                                quote!(#binding),
                                borrowed,
                                Some(&wire),
                            )
                        })
                    })
                    .map_err(|e| format!("{}::{}.{}: {e}", type_key(ty), arm.name, field.key()))?;
                let name = format_ident!("{}", field.property());
                if arm.fields.len() != 1 || field.key() != "0" {
                    self.prefix(
                        &mut field_plan.outcomes.success,
                        ComponentSegment::Field(field.property()),
                    );
                }
                field_plan.outcomes.success = WireNode::Branch {
                    controls: vec![],
                    children: vec![field_plan.outcomes.success],
                    meta: self.codec.group(GroupKind::Field(&name))?,
                };
                append(&mut child, field_plan)?;
            }
            self.prefix(
                &mut child.outcomes.success,
                ComponentSegment::Named(arm.name.to_string().trim_start_matches("r#").into()),
            );
            plans.push(child);
        }
        let mut arms = plans
            .iter_mut()
            .map(|p| std::mem::take(&mut p.outcomes.success))
            .collect::<Vec<_>>();
        let encoding = self.codec.enum_variants(EnumVariants {
            flat: self.flat,
            ty,
            extra,
            selector: &mut selector,
            arms: &mut arms,
            form: if grouped {
                EnumForm::Value
            } else {
                EnumForm::Fields
            },
            direction: Direction::Output,
        })?;
        if encoding.inactive.len() != plans.len() {
            return Err("missing inactive enum arm values".into());
        }
        let mut defaults = vec![];
        for (child, absent) in plans.iter().zip(encoding.inactive) {
            let mut absent = absent
                .into_iter()
                .map(|(n, v)| (n.to_string(), v))
                .collect::<BTreeMap<_, _>>();
            defaults.push(
                child
                    .values
                    .iter()
                    .map(|(n, _)| {
                        absent
                            .remove(&n.to_string())
                            .ok_or_else(|| format!("missing inactive enum value `{n}`"))
                    })
                    .collect::<Result<Vec<_>>>()?,
            );
            if !absent.is_empty() {
                return Err("unexpected inactive enum output".into());
            }
        }
        // One statically typed active-owner guard, with no boxing or per-arm
        // Option storage. A failed later conversion drops the selected guard.
        let guard_type = format_ident!("__PbEnumCommit_{root}");
        let guard = format_ident!("__{root}_commit");
        let cases = (0..plans.len())
            .map(|i| format_ident!("A{i}"))
            .collect::<Vec<_>>();
        let parameters = (0..plans.len())
            .map(|i| format_ident!("F{i}"))
            .collect::<Vec<_>>();
        let root_ref = &root;
        let locals = plans
            .iter()
            .enumerate()
            .flat_map(|(i, p)| {
                (0..p.values.len()).map(move |j| format_ident!("__{root_ref}_a{i}_value{j}"))
            })
            .collect::<Vec<_>>();
        let values = std::iter::once((tag.clone(), quote!(#tag)))
            .chain(
                plans
                    .iter()
                    .flat_map(|p| p.values.iter().map(|(n, _)| n.clone()))
                    .zip(locals.iter().map(|n| quote!(#n))),
            )
            .collect();
        let mut bodies = vec![];
        let mut fallible = false;
        for (index, (plan, pattern)) in plans.iter().zip(patterns).enumerate() {
            let tag = index as i32;
            let body = &plan.statements;
            let commit = &plan.commit;
            let case = &cases[index];
            let fields = plans
                .iter()
                .enumerate()
                .flat_map(|(i, p)| {
                    if i == index {
                        p.values.iter().map(|(_, v)| v.clone()).collect::<Vec<_>>()
                    } else {
                        defaults[i].clone()
                    }
                })
                .collect::<Vec<_>>();
            bodies.push(quote!(#pattern => { #body let values=(#(#fields,)*); (#tag,values,#guard_type::#case(move ||{#commit})) }));
            fallible |= plan.fallible;
        }
        let statements = quote! {
            #prepare
            #[allow(non_camel_case_types)] enum #guard_type<#(#parameters),*> { #(#cases(#parameters)),* }
            let (#tag,(#(#locals,)*),#guard)=match #value {#(#bodies,)*};
        };
        let commit = quote!(match #guard {#(#guard_type::#cases(action)=>action(),)*});
        let plan = OutputPlan {
            fallible,
            context: Default::default(),
            delivery: Default::default(),
            statements,
            commit,
            values,
            outcomes: prebindgen_flat::Outcomes {
                success: WireNode::Branch {
                    controls: vec![selector],
                    children: arms,
                    meta: encoding.metadata,
                },
                source_error: None,
                binding_failure: None,
            },
        };
        if grouped {
            self.codec.output_enum(self.flat, ty, extra, root, plan)
        } else {
            Ok(plan)
        }
    }
}

#[allow(missing_docs, clippy::too_many_arguments)]
impl<C: Codec> Compiler<'_, C> {
    pub fn output_result(
        &self,
        ty: &Type,
        ok: &ResolvedOutput<C>,
        error: &Option<String>,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        reference_return: bool,
    ) -> Result<CodecOutput<C>> {
        if borrowed {
            return Err("Result error routing requires an owned Result occurrence".into());
        }
        let (success_ty, error_ty) =
            result_parts(ty).ok_or("Result recipe requires Result<T, E>")?;
        let payload = self.name();
        let failure = self.name();
        let mapped = if let Some(error) = error {
            let mapper = self.flat.function(error)?;
            let mapper_arg = mapper.params.first().map(|p| &p.ty.ty);
            let shared = matches!(mapper_arg , Some (Type :: Reference (r)) if r . mutability . is_none () && same_type (& r . elem , error_ty));
            if mapper.params.len() != 1
                || !(mapper_arg.is_some_and(|ty| same_type(ty, error_ty)) || shared)
                || type_key(&mapper.ret.ty) != "String"
            {
                return Err(
                    "Result error mapper must take exactly E or &E and return String".into(),
                );
            }
            let path = &mapper.path;
            let argument = if shared {
                quote!(&# failure)
            } else {
                quote!(# failure)
            };
            Some(quote!(# path (# argument)))
        } else {
            None
        };
        let error = self.codec.source_error(SourceError {
            flat: self.flat,
            ty: error_ty,
            value: quote!(# failure),
            message: mapped,
            extra,
        })?;
        let message = error.propagate;
        let mut child = if reference_return {
            self.output_return(success_ty, ok, extra, quote!(# payload), false)?
        } else {
            self.output(success_ty, ok, extra, quote!(# payload), false)?
        };
        let prepare = child.statements;
        child.statements = quote! { let # payload = match # value { :: core :: result :: Result :: Ok (value) => value , :: core :: result :: Result :: Err (# failure) => return Err (# message) , } ; # prepare };
        child.outcomes.source_error = Some(error.payload);
        Ok(child)
    }
    pub fn input_sequence(
        &self,
        ty: &Type,
        element_recipe: &ResolvedInput<C>,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
        var: syn::Ident,
        bulk: bool,
    ) -> Result<CodecInput<C>> {
        {
            if matches!(ty, Type::Reference(_)) {
                let next = self.next.get();
                let wire = names
                    .map(|(wire, _)| wire.clone())
                    .unwrap_or_else(|| self.name());
                if let Some(plan) = self.codec.input_borrowed_sequence(
                    self.flat,
                    ty,
                    extra,
                    wire,
                    var.clone(),
                    element_recipe,
                    active,
                )? {
                    return Ok(plan);
                }
                self.next.set(next);
            }
            let (element, _) =
                sequence_parts(ty).ok_or("sequence input requires Vec<T>, &[T] or &Vec<T>")?;
            let wire = self.name();
            let length = self.name();
            let index = self.name();
            let child = self.input(element, element_recipe, extra, &Condition::default());
            let child = child?;
            sequence::input(
                self.codec, ty, extra, wire, var, length, index, active, child, bulk,
            )
        }
    }
    pub fn output_sequence(
        &self,
        ty: &Type,
        element: &ResolvedOutput<C>,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        delivery: &SequenceDelivery,
        mode: &Option<ProjectionMode>,
        bulk: bool,
    ) -> Result<CodecOutput<C>> {
        {
            let (element_type, _) =
                sequence_parts(ty).ok_or("sequence output requires a collection")?;
            let explicit_copy = *mode == Some(ProjectionMode::Borrowed);
            let mode = mode.unwrap_or_else(|| {
                if borrowed || !sequence_parts(ty).unwrap().1.is_owned() {
                    ProjectionMode::Borrowed
                } else {
                    ProjectionMode::Owned
                }
            });
            let wire = self.name();
            if mode == ProjectionMode::Borrowed && !explicit_copy && !self.copy_borrowed.get() {
                if let Some(plan) = self.codec.output_borrowed_sequence(
                    self.flat,
                    ty,
                    extra,
                    wire.clone(),
                    value.clone(),
                    element,
                    *delivery,
                    self.temporary_receiver.get(),
                )? {
                    return Ok(plan);
                }
            }
            let payload = self.name();
            let length = self.name();
            let index = self.name();
            let child = self.with_copy_permission(explicit_copy, || {
                self.with_temporary_receiver(
                    mode == ProjectionMode::Borrowed
                        && !borrowed
                        && sequence_parts(ty).unwrap().1.is_owned(),
                    || {
                        self.output(
                            element_type,
                            element,
                            extra,
                            quote!(# payload),
                            mode == ProjectionMode::Borrowed,
                        )
                    },
                )
            });
            let child = child?;
            sequence::output(
                self.codec,
                ty,
                extra,
                wire,
                payload,
                length,
                index,
                value,
                borrowed,
                mode,
                child,
                if *delivery == SequenceDelivery::Auto {
                    SequenceDelivery::Value
                } else {
                    *delivery
                },
                bulk,
            )
        }
    }
    pub fn output_owner_getters(
        &self,
        ty: &Type,
        parts: &[Option<(&str, &ResolvedOutput<C>)>],
        owner: usize,
        extra: &C::Extra,
        value: TokenStream,
    ) -> Result<Option<CodecOutput<C>>> {
        {
            let next = self.next.get();
            let wire = self.name();
            let Some((mut owned, pointer)) =
                self.codec.output_owner(self.flat, ty, extra, wire, value)?
            else {
                self.next.set(next);
                return Ok(None);
            };
            if parts
                .iter()
                .enumerate()
                .any(|(i, part)| i != owner && part.is_none())
            {
                return Err("an owner-backed output combination supports only the owning identity and getter views".into());
            }
            let prepare_owner = std::mem::take(&mut owned.statements);
            let mut result = OutputPlan {
                fallible: false,
                context: Default::default(),
                delivery: Default::default(),
                commit: TokenStream::new(),
                statements: TokenStream::new(),
                values: vec![],
                outcomes: prebindgen_flat::Outcomes {
                    success: WireNode::Branch {
                        controls: vec![],
                        children: vec![],
                        meta: self.codec.group(GroupKind::Values)?,
                    },
                    source_error: None,
                    binding_failure: None,
                },
            };
            let mut owner_plan = Some(owned);
            for (index, part) in parts.iter().enumerate() {
                let mut plan = if index == owner {
                    owner_plan.take().unwrap()
                } else {
                    let (function, output) = part.expect("validated getter");
                    let f = self.flat.function(function)?;
                    if f.params.len() != 1 {
                        return Err(format!(
                            "getter `{function}` must borrow its returned owner"
                        ));
                    }
                    let receiver = &f.params[0].ty.ty;
                    let mutable_receiver =
                        matches!(receiver , Type :: Reference (r) if r . mutability . is_some ());
                    let mutable = mutable_receiver || mutable_reference(&f.ret.ty);
                    if mutable && parts.len() != 2 {
                        return Err ("a mutable owner getter must be the only getter view; simultaneous owner borrows could invalidate earlier views" . into ()) ;
                    }
                    let expected: Type = if mutable_receiver {
                        syn::parse_quote!(& mut # ty)
                    } else {
                        syn::parse_quote!(&# ty)
                    };
                    if !same_type(receiver, &expected) {
                        return Err(format!(
                            "getter `{function}` must borrow its returned owner"
                        ));
                    }
                    let source = self.name();
                    let path = &f.path;
                    let argument = if mutable_receiver {
                        quote!(unsafe { & mut * (# pointer) })
                    } else {
                        quote!(unsafe { &* (# pointer) })
                    };
                    let mut plan =
                        self.output_return(&f.ret.ty, output, extra, quote!(# source), false)?;
                    self.codec
                        .validate_owner_getter(self.flat, ty, function, &plan)?;
                    plan.statements = {
                        let body = plan.statements;
                        quote!(let # source = # path (# argument) ; # body)
                    };
                    self.prefix(
                        &mut plan.outcomes.success,
                        ComponentSegment::Getter(function.to_owned()),
                    );
                    plan
                };
                self.prefix(&mut plan.outcomes.success, ComponentSegment::View(index));
                append(&mut result, plan)?;
            }
            result.statements = {
                let views = result.statements;
                quote!(# prepare_owner # views)
            };
            Ok(Some(result))
        }
    }
}

#[allow(missing_docs)]
impl<C: Codec> Compiler<'_, C> {
    pub fn output_reference_views(
        &self,
        ty: &Type,
        parts: &[ResolvedOutput<C>],
        extra: &C::Extra,
        value: TokenStream,
    ) -> Result<CodecOutput<C>> {
        if parts.is_empty() {
            return Err("an output combination needs at least one part".into());
        }
        if mutable_reference(ty) && parts.len() > 1 {
            return Err("a mutable reference cannot supply multiple output views".into());
        }
        let mut result = OutputPlan {
            fallible: false,
            context: Default::default(),
            delivery: Default::default(),
            commit: TokenStream::new(),
            statements: TokenStream::new(),
            values: vec![],
            outcomes: prebindgen_flat::Outcomes {
                success: WireNode::Branch {
                    controls: vec![],
                    children: vec![],
                    meta: self.codec.group(GroupKind::Values)?,
                },
                source_error: None,
                binding_failure: None,
            },
        };
        for (index, part) in parts.iter().enumerate() {
            let mut child = self.output_return(ty, part, extra, value.clone(), false)?;
            self.prefix(&mut child.outcomes.success, ComponentSegment::View(index));
            append(&mut result, child)?;
        }
        Ok(result)
    }
}
