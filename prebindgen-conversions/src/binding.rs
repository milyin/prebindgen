//! Prepared foreign callables attached to a source function by a language adapter.
use crate::{Conversion, FunctionConversions, InputConversion, Wire};
use prebindgen_flat::{Function, Result};

/// Conversions and export data for one foreign use of a source function.
/// The source function retains the only Rust signature and callable path.
#[derive(Clone, Debug)]
pub struct PreparedCall<I, O> {
    /// One conversion per source argument, in declaration order.
    pub inputs: Vec<I>,
    /// Return conversion, including unit results.
    pub output: O,
}
impl<I, O> PreparedCall<I, O> {
    /// Borrow this export together with its original source signature.
    pub fn view<'a, P, R, F, M>(
        &'a self,
        source: &'a Function<P, R, F>,
        name: &'a syn::Ident,
        metadata: &'a M,
    ) -> Result<BoundFunction<'a, I, O, M>> {
        if self.inputs.len() != source.params.len() {
            return Err(format!(
                "binding `{}`: input count does not match source `{}`",
                name, source.name
            ));
        }
        Ok(BoundFunction {
            name,
            path: &source.path,
            metadata,
            params: source
                .params
                .iter()
                .zip(&self.inputs)
                .map(|(p, input)| BoundParam {
                    name: &p.name,
                    ty: BoundTypeUse {
                        ty: &p.ty.ty,
                        slot: input,
                    },
                })
                .collect(),
            ret: BoundTypeUse {
                ty: &source.ret.ty,
                slot: &self.output,
            },
        })
    }
}
/// Read-only projection of one prepared export onto its source signature.
/// All source facts and conversions are borrowed from their authoritative owners.
pub struct BoundFunction<'a, I, O, M> {
    /// Generated export identity.
    pub name: &'a syn::Ident,
    /// Original Rust callable path.
    pub path: &'a syn::Path,
    /// Adapter-owned export metadata.
    pub metadata: &'a M,
    /// Source-ordered borrowed argument positions.
    pub params: Vec<BoundParam<'a, I>>,
    /// Borrowed Rust return position and its prepared conversion.
    pub ret: BoundTypeUse<'a, O>,
}
/// A borrowed source argument paired with this export's conversion.
pub struct BoundParam<'a, I> {
    /// Original Rust argument name.
    pub name: &'a syn::Ident,
    /// Original Rust type and selected conversion.
    pub ty: BoundTypeUse<'a, I>,
}
/// A borrowed Rust type paired with a prepared conversion.
pub struct BoundTypeUse<'a, T> {
    /// Original Rust type.
    pub ty: &'a syn::Type,
    /// The selected prepared conversion.
    pub slot: &'a T,
}
impl<I: Conversion, O: Conversion, M> FunctionConversions<I, O> for BoundFunction<'_, I, O, M> {
    fn source_path(&self) -> &syn::Path {
        self.path
    }
    fn input_conversions(&self) -> Result<Vec<&I>> {
        Ok(self.params.iter().map(|p| p.ty.slot).collect())
    }
    fn output_conversion(&self) -> Result<&O> {
        Ok(self.ret.slot)
    }
    fn input_wire_definitions<'a>(&'a self) -> Result<impl Iterator<Item = &'a Wire<I::Wire>>>
    where
        I: InputConversion,
        I::Wire: 'a,
    {
        Ok(self
            .input_conversions()?
            .into_iter()
            .flat_map(|i| crate::wire_definitions(std::iter::once(i.input_node()))))
    }
}

/// Preparation stage of one exported call, owned by the binding library.
#[derive(Clone, Debug)]
pub enum BindingState<C, R, P> {
    /// User configuration awaiting source-dependent resolution.
    Configured(C),
    /// Explicit conversion operations awaiting compilation.
    Resolved(R),
    /// Complete call conversions ready for emission.
    Prepared(P),
}
/// One exported use of a source callable, without a duplicate Rust signature.
#[derive(Clone, Debug)]
pub struct FunctionBinding<M, C, R, P> {
    /// Generated wrapper identity.
    pub name: syn::Ident,
    /// Adapter placement, names and error associations.
    pub metadata: M,
    /// Current preparation data for this export.
    pub state: BindingState<C, R, P>,
}
impl<M, C, R, I, O> FunctionBinding<M, C, R, PreparedCall<I, O>> {
    /// Mutably access the compiled conversions of an already prepared export.
    pub fn call_mut(&mut self) -> Result<&mut PreparedCall<I, O>> {
        match &mut self.state {
            BindingState::Prepared(call) => Ok(call),
            _ => Err(format!("binding `{}` is not prepared", self.name)),
        }
    }
    /// Read this export's compiled argument/result conversions.
    pub fn call(&self) -> Result<&PreparedCall<I, O>> {
        match &self.state {
            BindingState::Prepared(call) => Ok(call),
            _ => Err(format!("binding `{}` is not prepared", self.name)),
        }
    }
    /// Borrow this export alongside its authoritative source declaration.
    pub fn prepared<'a, SP, SR, SF>(
        &'a self,
        source: &'a Function<SP, SR, SF>,
    ) -> Result<BoundFunction<'a, I, O, M>> {
        self.call()?.view(source, &self.name, &self.metadata)
    }
}
impl<C, R, I: Conversion, O: Conversion, M> FunctionConversions<I, O>
    for Function<(), (), Vec<FunctionBinding<M, C, R, PreparedCall<I, O>>>>
{
    fn source_path(&self) -> &syn::Path {
        &self.path
    }
    fn input_conversions(&self) -> Result<Vec<&I>> {
        if self.metadata.len() != 1 {
            return Err(format!(
                "source `{}` has {} bindings; select an export explicitly",
                self.name,
                self.metadata.len()
            ));
        }
        let call = self.metadata[0].call()?;
        if call.inputs.len() != self.params.len() {
            return Err("prepared input count does not match source parameters".into());
        }
        Ok(call.inputs.iter().collect())
    }
    fn output_conversion(&self) -> Result<&O> {
        if self.metadata.len() != 1 {
            return Err(format!(
                "source `{}` has {} bindings; select an export explicitly",
                self.name,
                self.metadata.len()
            ));
        }
        Ok(&self.metadata[0].call()?.output)
    }
    fn input_wire_definitions<'a>(&'a self) -> Result<impl Iterator<Item = &'a Wire<I::Wire>>>
    where
        I: InputConversion,
        I::Wire: 'a,
    {
        Ok(self
            .input_conversions()?
            .into_iter()
            .flat_map(|i| crate::wire_definitions(std::iter::once(i.input_node()))))
    }
}
