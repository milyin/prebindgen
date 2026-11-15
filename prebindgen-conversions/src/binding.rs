//! Prepared foreign callables attached to a source function by a language adapter.
use crate::{Conversion, FunctionConversions, InputConversion, Wire};
use prebindgen_flat::{Function, Result};

/// Conversions and export data for one foreign use of a source function.
/// The source function retains the only Rust signature and callable path.
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
