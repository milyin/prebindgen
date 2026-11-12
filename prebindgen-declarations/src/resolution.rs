//! Declaration resolution preserves adapter-specific nested operations.
use crate::*;
/// Source-aware context for choosing an adapter representation.
/// Nested resolution uses the same defaults, local policies and recursion stack.
pub struct DeclResolve<'a, C: DeclCodec> {
    pub(crate) declarations: &'a DeclDefaults,
    /// Source declarations and adapter metadata.
    pub flat: &'a Flat<C::Schema>,
    pub(crate) codec: &'a C,
    /// Occurrence settings supplied to nested conversions.
    pub extra: &'a C::Extra,
    pub(crate) stack: &'a mut Vec<String>,
    pub(crate) getter: bool,
}
impl<C: DeclCodec> DeclResolve<'_, C> {
    /// Resolve a nested parameter or callback result.
    pub fn input(
        &mut self,
        ty: &Type,
        tree: &DeclInput<DeclInputRequest, DeclOutputRequest>,
    ) -> Result<ResolvedInput<C>> {
        self.declarations
            .input(self.flat, self.codec, ty, tree, self.extra, self.stack)
    }
    /// Resolve a nested result, getter or callback argument.
    pub fn output(
        &mut self,
        ty: &Type,
        tree: &DeclOutput<DeclInputRequest, DeclOutputRequest>,
    ) -> Result<ResolvedOutput<C>> {
        self.declarations.output(
            self.flat,
            self.codec,
            ty,
            tree,
            self.extra,
            self.stack,
            self.getter,
        )
    }
}
pub(crate) fn resolve_output_arguments<C: DeclCodec>(
    codec: &C,
    children: Vec<ResolvedOutput<C>>,
) -> Result<ResolvedOutput<C>> {
    Ok(if children.is_empty() {
        OutputNode::None
    } else {
        OutputNode::Branch {
            operation: codec.resolve_output_branch(DeclOutputBranch::Arguments)?,
            children,
        }
    })
}
pub(crate) fn resolve_callback_argument<C: DeclCodec>(
    codec: &C,
    child: ResolvedOutput<C>,
    ownership: CallbackArgumentOwnership,
) -> Result<ResolvedOutput<C>> {
    Ok(OutputNode::Unary {
        operation: codec.resolve_output_unary(DeclOutputUnary::CallbackArgument { ownership })?,
        child: Box::new(child),
    })
}
pub(crate) fn resolve_input_enum<C: DeclCodec>(
    codec: &C,
    variants: Vec<Vec<ResolvedInput<C>>>,
    value: bool,
) -> Result<ResolvedInput<C>> {
    Ok(InputNode::Branch {
        operation: codec.resolve_input_branch(DeclInputBranch::Enum { value })?,
        children: variants
            .into_iter()
            .map(|children| {
                Ok(InputNode::Branch {
                    operation: codec.resolve_input_branch(DeclInputBranch::Variant)?,
                    children,
                })
            })
            .collect::<Result<_>>()?,
    })
}
pub(crate) fn resolve_output_enum<C: DeclCodec>(
    codec: &C,
    variants: Vec<Vec<ResolvedOutput<C>>>,
    value: bool,
    mode: Option<ProjectionMode>,
) -> Result<ResolvedOutput<C>> {
    Ok(OutputNode::Branch {
        operation: codec.resolve_output_branch(DeclOutputBranch::Enum { value, mode })?,
        children: variants
            .into_iter()
            .map(|children| {
                Ok(OutputNode::Branch {
                    operation: codec.resolve_output_branch(DeclOutputBranch::Variant)?,
                    children,
                })
            })
            .collect::<Result<_>>()?,
    })
}
