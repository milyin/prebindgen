use super::*;
/// Input expansion declaration specialized for this adapter.
pub type DeclJniExpandParam = prebindgen_declarations::DeclExpandParam<
    JniInput<JniInputRequest, JniOutputRequest>,
    DeclJniFunction,
    JniExpansionOptions,
>;
/// Return expansion declaration specialized for this adapter.
pub type DeclJniExpandReturn = prebindgen_declarations::DeclExpandReturn<
    JniOutput<JniInputRequest, JniOutputRequest>,
    DeclJniFunction,
    JniExpansionOptions,
>;
/// Named-field declaration specialized for this adapter.
pub type DeclJniFields =
    prebindgen_declarations::DeclFields<JniOutput<JniInputRequest, JniOutputRequest>>;
/// Adapter-specific options applied after the common expansion is built.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct JniExpansionOptions {
    /// Pack this expansion using the JNI bulk carrier.
    pub bulk: bool,
}
/// Select the JNI bulk carrier for an input or output expansion.
pub trait JniBulkExpansion: Sized {
    /// Group the expansion in a bulk C-representation carrier.
    /// Constructor, getter, ownership and local conversion rules are retained.
    /// Ordinary output collections combine expansions into one frame. Callback
    /// delivery keeps packing inside each element: bulk `Row` gives one frame
    /// per row of `Vec<Row>`; bulk `Vec<Row>` gives one frame per inner collection
    /// of `Vec<Vec<Row>>`.
    fn bulk(self) -> Self;
}
impl JniBulkExpansion for DeclJniExpandParam {
    fn bulk(mut self) -> Self {
        self.extension.get_or_insert_with(Default::default).bulk = true;
        self
    }
}
impl JniBulkExpansion for DeclJniExpandReturn {
    fn bulk(mut self) -> Self {
        self.extension.get_or_insert_with(Default::default).bulk = true;
        self
    }
}
/// Start input expansion using this adapter's node constructors.
pub fn expand_param(ty: Type) -> DeclJniExpandParam {
    prebindgen_declarations::DeclExpandParam::new(
        ty,
        |function: DeclJniFunction| JniInput::call(function.reference_name(false)),
        || JniInput::Leaf(JniInputLeafOp::Value),
        |ty, alternatives, extension| {
            let mut input = alternatives.build(JniInput::selector, JniInput::auto, |child| {
                JniInput::Unary {
                    operation: (JniInputUnary::Temporary).into(),
                    child: Box::new(child),
                }
            });
            if extension.is_some_and(|options| options.bulk) {
                input = JniInput::Unary {
                    operation: (JniInputUnary::Bulk).into(),
                    child: Box::new(input),
                };
            }
            JniInput::for_type(ty, input)
        },
    )
}
/// Start return expansion using this adapter's node constructors.
pub fn expand_return(ty: Type) -> DeclJniExpandReturn {
    prebindgen_declarations::DeclExpandReturn::new(
        ty,
        |function| {
            prebindgen_declarations::getter_view(
                function,
                |name| JniOutput::getter(name, JniOutput::auto()),
                |name, child| JniOutput::Unary {
                    operation: (JniOutputUnary::Named { name }).into(),
                    child: Box::new(child),
                },
            )
        },
        || JniOutput::Leaf(JniOutputLeafOp::Value),
        |policy| JniOutput::Leaf(JniOutputLeafOp::Projected(policy)),
        super::requests::owned_field_view,
        |ty, views, extension| {
            let mut output = JniOutput::combine(views);
            if extension.is_some_and(|options| options.bulk) {
                output = JniOutput::Unary {
                    operation: (JniOutputUnary::Bulk).into(),
                    child: Box::new(output),
                };
            }
            JniOutput::for_type(ty, output)
        },
    )
}
/// Decompose a source accessor's returned record into named field views.
pub fn fields(function: syn::Ident) -> DeclJniFields {
    prebindgen_declarations::DeclFields::new(
        function,
        |output| match output {
            JniOutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Request(JniOutputRequest::ForType { ty }),
                child: output,
            } if matches!(output.as_ref(),JniOutput::Branch {operation:prebindgen_declarations::Operation::Resolved(JniOutputBranch::Combine),children} if children.is_empty()) => {
                JniOutput::for_type(*ty, JniOutput::Leaf(JniOutputLeafOp::Discard))
            }
            output => output,
        },
        |function, overrides, _extension| {
            let (fields, children) = overrides.into_iter().unzip();
            JniOutput::getter(
                function,
                JniOutput::Branch {
                    operation: prebindgen_declarations::Operation::Request(
                        JniOutputRequest::NamedFields {
                            fields,
                            mode: prebindgen_declarations::ProjectionMode::Borrowed,
                        },
                    ),
                    children,
                },
            )
        },
    )
}
/// Source-error declaration using this adapter's function reference.
pub type DeclJniError = prebindgen_declarations::DeclError<DeclJniFunction>;
/// Registration of this adapter's input/output/error declarations.
pub type DeclJniExpand =
    prebindgen_declarations::DeclExpand<DeclJniExpandParam, DeclJniExpandReturn, DeclJniError>;
/// Callback declaration containing this language's argument and result trees.
/// ```
/// use prebindgen_jni::{callback,expand_param,expand_return,fun};
/// let handler = callback!(impl Fn(Packet) -> Decision)
///     .name("PacketHandler")
///     .expand_param(0, expand_return!(Packet).field(fun!(packet_payload)))
///     .expand_return(expand_param!(Decision).variant(fun!(decision_new)));
/// let _ = fun!(register).expand_param("callback", handler);
/// ```
pub type DeclJniCallback = prebindgen_declarations::DeclCallback<
    JniInput<JniInputRequest, JniOutputRequest>,
    JniOutput<JniInputRequest, JniOutputRequest>,
>;
impl
    prebindgen_declarations::CallbackInput<
        JniInput<JniInputRequest, JniOutputRequest>,
        JniOutput<JniInputRequest, JniOutputRequest>,
    > for JniInputRequest
{
    fn callback_input(declaration: DeclJniCallback) -> JniInput<JniInputRequest, JniOutputRequest> {
        prebindgen_declarations::callback_input(
            declaration,
            JniInput::auto,
            |children| JniOutput::Branch {
                operation: JniOutputBranch::Arguments.into(),
                children,
            },
            |metadata, domain, codomain| JniInput::Arrow {
                operation: prebindgen_declarations::Operation::Request(Self::Callback(Box::new(
                    metadata,
                ))),
                domain: Box::new(domain),
                codomain: Box::new(codomain),
            },
            JniInput::for_type,
        )
    }
}
