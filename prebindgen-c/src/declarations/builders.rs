use super::*;
/// Input expansion declaration specialized for this adapter.
pub type DeclCExpandParam = prebindgen_declarations::DeclExpandParam<
    CInput<CInputRequest, COutputRequest>,
    DeclCFunction,
    std::convert::Infallible,
>;
/// Return expansion declaration specialized for this adapter.
/// Field-decomposition methods require a fields declaration:
/// ```compile_fail
/// use prebindgen_c::{expand_return, COutput};
/// let _ = expand_return!(Row).fields(COutput::None);
/// ```
/// ```compile_fail
/// use prebindgen_c::{expand_return, COutput};
/// let _ = expand_return!(Row).fields_into(COutput::None);
/// ```
pub type DeclCExpandReturn = prebindgen_declarations::DeclExpandReturn<
    COutput<CInputRequest, COutputRequest>,
    DeclCFunction,
    std::convert::Infallible,
>;
/// Named-field declaration specialized for this adapter.
pub type DeclCFields = prebindgen_declarations::DeclFields<COutput<CInputRequest, COutputRequest>>;
/// Start input expansion using this adapter's node constructors.
pub fn expand_param(ty: Type) -> DeclCExpandParam {
    prebindgen_declarations::DeclExpandParam::new(
        ty,
        |function: DeclCFunction| CInput::call(function.reference_name(false)),
        || CInput::Leaf(CInputLeafOp::Value),
        |ty, alternatives, extension| {
            let input = alternatives.build(CInput::selector, CInput::auto, |child| CInput::Unary {
                operation: CInputUnary::Temporary,
                child: Box::new(child),
            });
            let _ = extension;
            CInput::for_type(ty, input)
        },
    )
}
/// Start return expansion using this adapter's node constructors.
pub fn expand_return(ty: Type) -> DeclCExpandReturn {
    prebindgen_declarations::DeclExpandReturn::new(
        ty,
        |function| {
            prebindgen_declarations::getter_view(
                function,
                |name| COutput::getter(name, COutput::auto()),
                |name, child| COutput::Unary {
                    operation: COutputUnary::Named { name },
                    child: Box::new(child),
                },
            )
        },
        || COutput::Leaf(COutputLeafOp::Value),
        |policy| COutput::Leaf(COutputLeafOp::Projected(policy)),
        super::requests::owned_field_view,
        |ty, views, extension| {
            let output = COutput::combine(views);
            let _ = extension;
            COutput::for_type(ty, output)
        },
    )
}
/// Decompose a source accessor's returned record into named field views.
pub fn fields(function: syn::Ident) -> DeclCFields {
    prebindgen_declarations::DeclFields::new(
        function,
        |output| match output {
            COutput::Unary {
                operation: COutputUnary::ForType { ty },
                child: output,
            } if matches!(output.as_ref(),COutput::Branch {operation:COutputBranch::Combine,children} if children.is_empty()) => {
                COutput::for_type(*ty, COutput::Leaf(COutputLeafOp::Discard))
            }
            output => output,
        },
        |function, overrides, _extension| {
            let (fields, children) = overrides.into_iter().unzip();
            COutput::getter(
                function,
                COutput::Branch {
                    operation: COutputBranch::NamedFields {
                        fields,
                        mode: prebindgen_declarations::ProjectionMode::Borrowed,
                    },
                    children,
                },
            )
        },
    )
}
/// Source-error declaration using this adapter's function reference.
pub type DeclCError = prebindgen_declarations::DeclError<DeclCFunction>;
/// Registration of this adapter's input/output/error declarations.
pub type DeclCExpand =
    prebindgen_declarations::DeclExpand<DeclCExpandParam, DeclCExpandReturn, DeclCError>;
/// Callback declaration containing this language's argument and result trees.
pub type DeclCCallback = prebindgen_declarations::DeclCallback<
    CInput<CInputRequest, COutputRequest>,
    COutput<CInputRequest, COutputRequest>,
>;
impl
    prebindgen_declarations::CallbackInput<
        CInput<CInputRequest, COutputRequest>,
        COutput<CInputRequest, COutputRequest>,
    > for CInputRequest
{
    fn callback_input(declaration: DeclCCallback) -> CInput<CInputRequest, COutputRequest> {
        prebindgen_declarations::callback_input(
            declaration,
            CInput::auto,
            |children| COutput::Branch {
                operation: COutputBranch::Arguments,
                children,
            },
            |metadata, domain, codomain| CInput::Arrow {
                operation: CInputArrow::Callback(Box::new(metadata)),
                domain: Box::new(domain),
                codomain: Box::new(codomain),
            },
            CInput::for_type,
        )
    }
}
