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

impl Declarations {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_input_recipe<C: Codec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        extra: &C::Extra,
        entries: &BTreeMap<String, BTreeMap<String, Input<InputRequest, OutputRequest>>>,
        value: bool,
        stack: &mut Vec<String>,
    ) -> Result<Input> {
        let source = flat.payload_enum(ty)?;
        validate_overrides(source, entries)?;
        let default = if value { Input::value() } else { Input::auto() };
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
        Ok(Input::enum_variants(variants, value))
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_output_recipe<C: Codec>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        extra: &C::Extra,
        entries: &BTreeMap<String, BTreeMap<String, Output<OutputRequest, InputRequest>>>,
        value: bool,
        stack: &mut Vec<String>,
        getter: bool,
    ) -> Result<Output> {
        let source = flat.payload_enum(ty)?;
        validate_overrides(source, entries)?;
        let default = if value {
            Output::value()
        } else {
            Output::auto()
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
        Ok(Output::enum_variants(variants, value, None))
    }
}

impl<C: Codec> Compiler<'_, C> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_input(
        &self,
        ty: &Type,
        recipes: &[Input],
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
            let recipes = recipes.variant_fields()?;
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
    pub(crate) fn enum_output(
        &self,
        ty: &Type,
        recipes: &[Output],
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
            let recipes = recipes.variant_fields()?;
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
