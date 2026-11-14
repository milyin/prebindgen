use super::*;
/// Input expansion declaration specialized for this adapter.
pub type DeclCExpandParam =
    prebindgen_declarations::DeclExpandParam<CInputConfig, DeclCFunction, std::convert::Infallible>;
/// Return expansion declaration specialized for this adapter.
/// Field-decomposition methods require a fields declaration:
/// ```compile_fail
/// use prebindgen_c::{expand_return, COutputConfig};
/// let _ = expand_return!(Row).fields(COutputConfig::None);
/// ```
/// ```compile_fail
/// use prebindgen_c::{expand_return, COutputConfig};
/// let _ = expand_return!(Row).fields_into(COutputConfig::None);
/// ```
pub type DeclCExpandReturn = prebindgen_declarations::DeclExpandReturn<
    COutputConfig,
    DeclCFunction,
    std::convert::Infallible,
>;
/// Named-field declaration specialized for this adapter.
pub type DeclCFields = prebindgen_declarations::DeclFields<COutputConfig>;
/// Start input expansion using this adapter's node constructors.
pub fn expand_param(ty: Type) -> DeclCExpandParam {
    prebindgen_declarations::DeclExpandParam::new(
        ty,
        |function: DeclCFunction| CInputConfig::call(function.reference_name(false)),
        || CInputConfig::Leaf(CInputLeafOp::Value),
        |ty, alternatives, extension| {
            let input = alternatives.build(CInputConfig::selector, CInputConfig::auto, |child| {
                CInputConfig::Unary {
                    operation: CInputUnary::Temporary,
                    child: Box::new(child),
                }
            });
            let _ = extension;
            CInputConfig::for_type(ty, input)
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
                |name| COutputConfig::getter(name, COutputConfig::auto()),
                |name, child| COutputConfig::Unary {
                    operation: COutputUnary::Named { name },
                    child: Box::new(child),
                },
            )
        },
        || COutputConfig::Leaf(COutputLeafOp::Value),
        |policy| COutputConfig::Leaf(COutputLeafOp::Projected(policy)),
        super::configuration::owned_field_view,
        |ty, views, extension| {
            let output = COutputConfig::combine(views);
            let _ = extension;
            COutputConfig::for_type(ty, output)
        },
    )
}
/// Decompose a source accessor's returned record into named field views.
pub fn fields(function: syn::Ident) -> DeclCFields {
    prebindgen_declarations::DeclFields::new(
        function,
        |output| match output {
            COutputConfig::Unary {
                operation: COutputUnary::ConfigForType { ty },
                child: output,
            } if matches!(output.as_ref(),COutputConfig::Branch {operation:COutputBranch::Combine,children} if children.is_empty()) => {
                COutputConfig::for_type(*ty, COutputConfig::Leaf(COutputLeafOp::Discard))
            }
            output => output,
        },
        |function, overrides, _extension| {
            let (fields, children) = overrides.into_iter().unzip();
            COutputConfig::getter(
                function,
                COutputConfig::Branch {
                    operation: COutputBranch::ConfigNamedFields(CNamedFields {
                        fields,
                        mode: prebindgen_declarations::ProjectionMode::Borrowed,
                    }),
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
pub type DeclCCallback = prebindgen_declarations::DeclCallback<CInputConfig, COutputConfig>;
impl prebindgen_declarations::CallbackInput<CInputConfig, COutputConfig> for CInputConfigOps {
    fn callback_input(declaration: DeclCCallback) -> CInputConfig {
        prebindgen_declarations::callback_input(
            declaration,
            CInputConfig::auto,
            |children| COutputConfig::Branch {
                operation: COutputBranch::Arguments,
                children,
            },
            |metadata, domain, codomain| CInputConfig::Arrow {
                operation: CInputArrow::ConfigCallback(Box::new(metadata)),
                domain: Box::new(domain),
                codomain: Box::new(codomain),
            },
            CInputConfig::for_type,
        )
    }
}
