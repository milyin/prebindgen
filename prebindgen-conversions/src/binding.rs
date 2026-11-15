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
/// The consumer chooses `S`: a declaration pipeline can track [`BindingState`],
/// while an already prepared export can hold [`PreparedCall`] directly.
#[derive(Clone, Debug)]
pub struct FunctionBinding<M, S> {
    /// Generated wrapper identity.
    pub name: syn::Ident,
    /// Adapter placement, names and error associations.
    pub metadata: M,
    /// Consumer-selected call data or preparation state for this export.
    pub state: S,
}
impl<M, C, R, I, O> FunctionBinding<M, BindingState<C, R, PreparedCall<I, O>>> {
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
impl<M, I, O> FunctionBinding<M, PreparedCall<I, O>> {
    /// Borrow this prepared export alongside its authoritative source declaration.
    /// Only the argument count needs checking; no preparation state is involved.
    pub fn prepared<'a, SP, SR, SF>(
        &'a self,
        source: &'a Function<SP, SR, SF>,
    ) -> Result<BoundFunction<'a, I, O, M>> {
        self.state.view(source, &self.name, &self.metadata)
    }
}
impl<C, R, I: Conversion, O: Conversion, M> FunctionConversions<I, O>
    for Function<(), (), Vec<FunctionBinding<M, BindingState<C, R, PreparedCall<I, O>>>>>
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

#[cfg(test)]
mod tests {
    use super::*;
    use prebindgen_flat::Flat;

    #[test]
    fn directly_prepared_export_borrows_source_and_converters_without_a_lifecycle() {
        struct Converter(u8); // deliberately not Clone
        struct Metadata(&'static str);
        let items = syn::parse_file("pub fn send(value:i64)->i64{}pub fn pair(a:i64,b:i64){}")
            .unwrap()
            .items;
        let flat = Flat::<()>::parse(items.into_iter().map(|i| (i, Default::default()))).unwrap();
        let binding = FunctionBinding {
            name: syn::parse_quote!(send_native),
            metadata: Metadata("native export"),
            state: PreparedCall {
                inputs: vec![Converter(1)],
                output: Converter(2),
            },
        };
        let source = flat.function("send").unwrap();
        let view = binding.prepared(source).unwrap();
        assert_eq!(view.metadata.0, "native export");
        assert_eq!(view.params[0].ty.slot.0, 1);
        assert_eq!(view.ret.slot.0, 2);
        assert!(std::ptr::eq(view.path, &source.path));
        assert!(std::ptr::eq(view.metadata, &binding.metadata));
        assert!(std::ptr::eq(view.params[0].name, &source.params[0].name));
        assert!(std::ptr::eq(
            view.params[0].ty.slot,
            &binding.state.inputs[0]
        ));
        assert!(std::ptr::eq(view.ret.slot, &binding.state.output));
        let error = binding
            .prepared(flat.function("pair").unwrap())
            .err()
            .unwrap();
        assert!(error.contains("send_native"));
        assert!(error.contains("input count does not match source `pair`"));
    }
}
