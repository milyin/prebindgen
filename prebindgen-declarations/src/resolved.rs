//! The resolved Flat is the complete input to plan compilation.
use crate::*;
use prebindgen_flat::{Function, Param, TypeUse};
use std::collections::BTreeMap;

/// A resolved operation tree, or a converter supplied explicitly by the user.
/// Existing custom converters remain authoritative unless a declaration overrides them.
#[derive(Clone, Debug)]
pub enum ResolvedSlot<T, P> {
    /// Request-free conversion composition.
    Tree(T),
    /// An explicitly supplied converter requiring no declaration compilation.
    Prepared(P),
}

/// Schema with request-free input/output trees and the adapter's source metadata.
pub struct Resolved<C: Codec>(std::marker::PhantomData<C>);
impl<C: Codec> Schema for Resolved<C> {
    type Input = ResolvedSlot<Input, <C::Schema as Schema>::Input>;
    type Output = ResolvedSlot<Output, <C::Schema as Schema>::Output>;
    type Module = ResolvedModule<C>;
    type Function = <C::Schema as Schema>::Function;
    type Type = <C::Schema as Schema>::Type;
    type Field = <C::Schema as Schema>::Field;
    type Variant = <C::Schema as Schema>::Variant;
}
/// Adapter settings and compilation policy owned by the resolved model.
/// No declarations or default-expansion registries are retained.
pub struct ResolvedModule<C: Codec> {
    /// Source module's foreign declarations and generation settings.
    pub source: <C::Schema as Schema>::Module,
    compilation: Option<(C, C::Extra)>,
}
impl<C: Codec> Default for ResolvedModule<C> {
    fn default() -> Self {
        Self {
            source: Default::default(),
            compilation: None,
        }
    }
}

pub(crate) struct FunctionRecipe {
    pub inputs: Vec<Option<Input>>,
    pub output: Option<Output>,
}

impl Declarations {
    /// Resolve function occurrences into an independently compilable Flat.
    ///
    /// Defaults and local overrides are consumed here. The returned model owns
    /// its compiler policy and occurrence settings; drop this Declarations value
    /// before calling [`compile_resolved`] if desired.
    pub fn resolve<C: Codec>(
        &self,
        flat: Flat<C::Schema>,
        codec: C,
        declarations: &[FunctionDecl],
        extra: C::Extra,
    ) -> Result<Flat<Resolved<C>>> {
        let mut recipes = BTreeMap::new();
        for decl in declarations {
            let recipe = self
                .resolve_function(&flat, &codec, decl, &extra)
                .map_err(|e| format!("{}: {e}", decl.name))?;
            if recipes.insert(decl.name.clone(), recipe).is_some() {
                return Err(format!("duplicate function declaration `{}`", decl.name));
            }
        }
        let functions = flat
            .functions
            .into_iter()
            .map(|(name, function)| {
                let recipe = recipes.remove(&name).unwrap_or_else(|| FunctionRecipe {
                    inputs: (0..function.params.len()).map(|_| None).collect(),
                    output: None,
                });
                let params = function
                    .params
                    .into_iter()
                    .zip(recipe.inputs)
                    .map(|(p, tree)| Param {
                        name: p.name,
                        ty: TypeUse {
                            ty: p.ty.ty,
                            slot: tree
                                .map(ResolvedSlot::Tree)
                                .or_else(|| p.ty.slot.map(ResolvedSlot::Prepared)),
                        },
                    })
                    .collect();
                (
                    name,
                    Function {
                        metadata: function.metadata,
                        name: function.name,
                        path: function.path,
                        location: function.location,
                        params,
                        ret: TypeUse {
                            ty: function.ret.ty,
                            slot: recipe
                                .output
                                .map(ResolvedSlot::Tree)
                                .or_else(|| function.ret.slot.map(ResolvedSlot::Prepared)),
                        },
                    },
                )
            })
            .collect();
        Ok(Flat {
            metadata: ResolvedModule {
                source: flat.metadata,
                compilation: Some((codec, extra)),
            },
            functions,
            records: flat.records,
            enums: flat.enums,
            payload_enums: flat.payload_enums,
            types: flat.types,
            guards: flat.guards,
        })
    }
}

/// Compile a resolved Flat without any external configuration or default registry.
/// Source metadata and custom converters are moved into the result unchanged.
pub fn compile_resolved<C: Codec>(flat: Flat<Resolved<C>>) -> Result<Flat<C::Schema>> {
    let (codec, extra) = flat
        .metadata
        .compilation
        .ok_or("resolved Flat has no compilation policy")?;
    let mut recipes = Vec::new();
    let functions = flat
        .functions
        .into_iter()
        .map(|(name, function)| {
            let mut inputs = Vec::with_capacity(function.params.len());
            let params = function
                .params
                .into_iter()
                .map(|p| {
                    let (tree, slot) = separate(p.ty.slot);
                    inputs.push(tree);
                    Param {
                        name: p.name,
                        ty: TypeUse { ty: p.ty.ty, slot },
                    }
                })
                .collect();
            let (output, slot) = separate(function.ret.slot);
            recipes.push((name.clone(), FunctionRecipe { inputs, output }));
            (
                name,
                Function {
                    metadata: function.metadata,
                    name: function.name,
                    path: function.path,
                    location: function.location,
                    params,
                    ret: TypeUse {
                        ty: function.ret.ty,
                        slot,
                    },
                },
            )
        })
        .collect();
    let mut result = Flat {
        metadata: flat.metadata.source,
        functions,
        records: flat.records,
        enums: flat.enums,
        payload_enums: flat.payload_enums,
        types: flat.types,
        guards: flat.guards,
    };
    codec.compile_declarations(&mut result, &extra)?;
    for (name, recipe) in recipes {
        compile_function(&mut result, &codec, &name, recipe, &extra)
            .map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(result)
}
fn separate<T, P>(slot: Option<ResolvedSlot<T, P>>) -> (Option<T>, Option<P>) {
    match slot {
        Some(ResolvedSlot::Tree(tree)) => (Some(tree), None),
        Some(ResolvedSlot::Prepared(plan)) => (None, Some(plan)),
        None => (None, None),
    }
}

pub(crate) fn compile_function<C: Codec>(
    flat: &mut Flat<C::Schema>,
    codec: &C,
    name: &str,
    recipe: FunctionRecipe,
    extra: &C::Extra,
) -> Result<()> {
    let source = flat.function(name)?;
    let mut inputs = Vec::with_capacity(source.params.len());
    for (index, (p, tree)) in source.params.iter().zip(recipe.inputs).enumerate() {
        let plan = if let Some(tree) = tree {
            let compiler = Compiler::new(flat, codec, format!("p{index}"));
            let wire = format_ident!("p{index}_callback");
            let value = format_ident!("p{index}_callback_value");
            let names = matches!(tree, Input::Arrow { .. }).then_some((&wire, &value));
            let plan =
                compiler.input_named(&p.ty.ty, &tree, extra, &Condition::default(), names)?;
            Some(plan)
        } else {
            None
        };
        inputs.push(plan);
    }
    let mut output = if let Some(tree) = recipe.output {
        let plan = Compiler::new(flat, codec, "ret".into()).output_root(
            &source.ret.ty,
            &tree,
            extra,
            prebindgen_conversions::source_result_expr(),
            false,
            true,
        )?;
        Some(plan)
    } else {
        None
    };
    for input in inputs.iter_mut().flatten() {
        codec.finish_input(input, extra)?;
        codec.validate_input(input)?;
    }
    if let Some(output) = &mut output {
        finalize_output_schema(codec, output, extra)?;
        codec.finish_output(output, extra)?;
        codec.validate_output(output)?;
    }
    let dest = flat.function_mut(name)?;
    for (p, plan) in dest.params.iter_mut().zip(inputs) {
        if let Some(plan) = plan {
            p.ty.slot = Some(plan.into());
        }
    }
    if let Some(plan) = output {
        dest.ret.slot = Some(plan.into());
    }
    Ok(())
}
