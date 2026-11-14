use super::*;
/// Input expansion declaration specialized for this adapter.
pub type DeclJniExpandParam =
    prebindgen_declarations::DeclExpandParam<JniInputConfig, DeclJniFunction, JniExpansionOptions>;
/// Return expansion declaration specialized for this adapter.
pub type DeclJniExpandReturn = prebindgen_declarations::DeclExpandReturn<
    JniOutputConfig,
    DeclJniFunction,
    JniExpansionOptions,
>;
/// Named-field declaration specialized for this adapter.
pub type DeclJniFields = prebindgen_declarations::DeclFields<JniOutputConfig>;
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
        |function: DeclJniFunction| JniInputConfig::call(function.reference_name(false)),
        || JniInputConfig::Leaf(JniInputLeafOp::Value),
        |ty, alternatives, extension| {
            let mut input =
                alternatives.build(JniInputConfig::selector, JniInputConfig::auto, |child| {
                    JniInputConfig::Unary {
                        operation: JniInputUnary::Temporary,
                        child: Box::new(child),
                    }
                });
            if extension.is_some_and(|options| options.bulk) {
                input = JniInputConfig::Unary {
                    operation: JniInputUnary::Bulk,
                    child: Box::new(input),
                };
            }
            JniInputConfig::for_type(ty, input)
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
                |name| JniOutputConfig::getter(name, JniOutputConfig::auto()),
                |name, child| JniOutputConfig::Unary {
                    operation: JniOutputUnary::Named { name },
                    child: Box::new(child),
                },
            )
        },
        || JniOutputConfig::Leaf(JniOutputLeafOp::Value),
        |policy| JniOutputConfig::Leaf(JniOutputLeafOp::Projected(policy)),
        super::configuration::owned_field_view,
        |ty, views, extension| {
            let mut output = JniOutputConfig::combine(views);
            if extension.is_some_and(|options| options.bulk) {
                output = JniOutputConfig::Unary {
                    operation: JniOutputUnary::Bulk,
                    child: Box::new(output),
                };
            }
            JniOutputConfig::for_type(ty, output)
        },
    )
}
/// Decompose a source accessor's returned record into named field views.
pub fn fields(function: syn::Ident) -> DeclJniFields {
    prebindgen_declarations::DeclFields::new(
        function,
        |output| match output {
            JniOutputConfig::Unary {
                operation: JniOutputUnary::ConfigForType { ty },
                child: output,
            } if matches!(output.as_ref(),JniOutputConfig::Branch {operation:JniOutputBranch::Combine,children} if children.is_empty()) => {
                JniOutputConfig::for_type(*ty, JniOutputConfig::Leaf(JniOutputLeafOp::Discard))
            }
            output => output,
        },
        |function, overrides, _extension| {
            let (fields, children) = overrides.into_iter().unzip();
            JniOutputConfig::getter(
                function,
                JniOutputConfig::Branch {
                    operation: JniOutputBranch::ConfigNamedFields(JniNamedFields {
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
pub type DeclJniCallback = prebindgen_declarations::DeclCallback<JniInputConfig, JniOutputConfig>;
impl prebindgen_declarations::CallbackInput<JniInputConfig, JniOutputConfig> for JniInputConfigOps {
    fn callback_input(declaration: DeclJniCallback) -> JniInputConfig {
        prebindgen_declarations::callback_input(
            declaration,
            JniInputConfig::auto,
            |children| JniOutputConfig::Branch {
                operation: JniOutputBranch::Arguments,
                children,
            },
            |metadata, domain, codomain| JniInputConfig::Arrow {
                operation: JniInputArrow::ConfigCallback(Box::new(metadata)),
                domain: Box::new(domain),
                codomain: Box::new(codomain),
            },
            JniInputConfig::for_type,
        )
    }
}
