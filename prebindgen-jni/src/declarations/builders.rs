use super::*;
/// Input alternatives for a source type, built directly as a [`JniInput`] tree.
#[derive(Clone, Debug)]
pub struct DeclJniExpandParam {
    /// Source type targeted by this default or occurrence override.
    pub ty: syn::Type,
    input: JniInput<JniInputRequest, JniOutputRequest>,
    bulk: bool,
    reconstruct: bool,
}
impl DeclJniExpandParam {
    /// Start an input declaration; add at least one alternative before use.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            input: JniInput::Branch {
                operation: JniInputBranch::Selector,
                children: vec![],
            },
            bulk: false,
            reconstruct: false,
        }
    }
    /// Group this expansion into a C-representation bulk carrier.
    /// The adapter chooses storage; constructor and local conversion rules are retained.
    pub fn bulk(mut self) -> Self {
        self.bulk = true;
        self
    }
    /// Explicitly construct an owned temporary and borrow it for the source call.
    /// With no alternatives, use the owned value's default input conversion.
    /// Alternatives describe the owned referent, not the reference itself.
    pub fn reconstruct(mut self) -> Self {
        self.reconstruct = true;
        self
    }
    /// Append a constructor reference; its arguments use their type defaults.
    pub fn variant(mut self, function: impl Into<DeclJniFunction>) -> Self {
        self.input = self.input.variant(function.into().reference_name(false));
        self
    }
    /// Append the type's direct language representation.
    pub fn variant_self(mut self) -> Self {
        self.input = self.input.variant_self();
        self
    }
}
impl From<DeclJniExpandParam> for JniInput<JniInputRequest, JniOutputRequest> {
    fn from(decl: DeclJniExpandParam) -> Self {
        let mut input = decl.input;
        if decl.reconstruct {
            if matches!(&input, JniInput::Branch { operation: JniInputBranch::Selector, children: arms } if arms.is_empty())
            {
                input = JniInput::auto();
            }
            input = JniInput::Unary {
                operation: JniInputUnary::Temporary,
                child: Box::new(input),
            };
        }
        if decl.bulk {
            input = JniInput::Unary {
                operation: JniInputUnary::Bulk,
                child: Box::new(input),
            };
        }
        JniInput::for_type(decl.ty, input)
    }
}

/// Simultaneous views of a source return value, built directly as a [`JniOutput`] tree.
#[derive(Clone, Debug)]
pub struct DeclJniExpandReturn {
    /// Source type targeted by this default or occurrence override.
    pub ty: syn::Type,
    output: JniOutput<JniInputRequest, JniOutputRequest>,
    bulk: bool,
}
impl DeclJniExpandReturn {
    /// Start an output declaration; add views before registering a type default.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            output: JniOutput::Branch {
                operation: JniOutputBranch::Combine,
                children: vec![],
            },
            bulk: false,
        }
    }
    /// Group this expansion into a C-representation bulk carrier.
    /// Getter, ownership and local conversion rules are retained. Ordinary
    /// collections combine these expansions into one frame. Callback delivery
    /// keeps the expansion within each callback element instead: bulk `Row`
    /// produces one frame per row of `Vec<Row>`, while bulk `Vec<Row>` produces
    /// one frame per inner collection of `Vec<Vec<Row>>`.
    pub fn bulk(mut self) -> Self {
        self.bulk = true;
        self
    }
    /// Append a getter view. An explicit `.name()` labels the delivered view.
    /// Shared results are observed before another view moves the owner. An
    /// owning opaque copy requires an explicit projection or cloning accessor.
    pub fn field(mut self, function: impl Into<DeclJniFunction>) -> Self {
        let function = function.into();
        let name = function.foreign.clone();
        let getter = JniOutput::getter(function.reference_name(true), JniOutput::auto());
        self.output = self.output.view(match name {
            Some(name) => JniOutput::Unary {
                operation: JniOutputUnary::Named { name },
                child: Box::new(getter),
            },
            None => getter,
        });
        self
    }
    /// Append a getter's named record fields as separate output values.
    pub fn fields(self, fields: DeclJniFields) -> Self {
        self.view(fields)
    }
    /// Append another explicit output view.
    pub fn view(mut self, output: impl Into<JniOutput<JniInputRequest, JniOutputRequest>>) -> Self {
        self.output = self.output.view(output.into());
        self
    }
    /// Append a consuming accessor followed by owned field decomposition.
    pub fn fields_into(mut self, fields: DeclJniFields) -> Self {
        self.output = self.output.fields_into(fields.into());
        self
    }
    /// Deprecated spelling of [`Self::fields_into`].
    #[deprecated(note = "use fields_into instead")]
    pub fn fields_self_into(self, fields: DeclJniFields) -> Self {
        self.fields_into(fields)
    }
    /// Append the language's direct representation, bypassing this type's default.
    pub fn field_self(mut self) -> Self {
        self.output = self.output.field_self();
        self
    }
    /// Append an explicit shared projection. Owning opaque representations clone
    /// the value; record/collection projections permit copying borrowed leaves.
    pub fn field_self_ref(self) -> Self {
        self.project_self(Projection::borrowed())
    }
    /// Append an explicit owned projection.
    pub fn field_self_into(self) -> Self {
        self.project_self(Projection::owned())
    }
    /// Append a projection with explicit ownership and field policies.
    pub fn project_self(mut self, policy: Projection) -> Self {
        self.output = self.output.project_self(policy);
        self
    }
}
impl From<DeclJniExpandReturn> for JniOutput<JniInputRequest, JniOutputRequest> {
    fn from(decl: DeclJniExpandReturn) -> Self {
        JniOutput::for_type(
            decl.ty,
            if decl.bulk {
                JniOutput::Unary {
                    operation: JniOutputUnary::Bulk,
                    child: Box::new(decl.output),
                }
            } else {
                decl.output
            },
        )
    }
}

/// A source accessor followed by named field decomposition.
/// Field adjustments edit the accessor's output tree directly.
#[derive(Clone, Debug)]
pub struct DeclJniFields {
    function: String,
    overrides: BTreeMap<String, JniOutput<JniInputRequest, JniOutputRequest>>,
}
impl DeclJniFields {
    /// Select a getter whose returned record supplies the fields.
    pub fn new(function: syn::Ident) -> Self {
        Self {
            function: function.to_string(),
            overrides: BTreeMap::new(),
        }
    }
    /// Replace one field's conversion. An empty declaration omits the whole field,
    /// including any enclosing Option or sequence representation.
    pub fn field(self, name: impl AsRef<str>, output: DeclJniExpandReturn) -> Self {
        self.field_override(name.as_ref().to_owned(), output)
    }
    /// Set an explicit conversion tree for one field, with the same omission
    /// semantics as [`Self::field`] for an empty typed declaration.
    pub fn field_override(
        mut self,
        name: impl Into<String>,
        output: impl Into<JniOutput<JniInputRequest, JniOutputRequest>>,
    ) -> Self {
        let output = match output.into() {
            JniOutput::Request(JniOutputRequest::ForType { ty, output }) if matches!(output.as_ref(),JniOutput::Branch { operation: JniOutputBranch::Combine, children: parts } if parts.is_empty()) => {
                JniOutput::for_type(*ty, JniOutput::Leaf(JniOutputLeafOp::Discard))
            }
            output => output,
        };
        assert!(
            self.overrides.insert(name.into(), output).is_none(),
            "duplicate field override"
        );
        self
    }
}
impl From<DeclJniFields> for JniOutput<JniInputRequest, JniOutputRequest> {
    fn from(fields: DeclJniFields) -> Self {
        JniOutput::getter(
            fields.function,
            JniOutput::Request(JniOutputRequest::NamedFields {
                overrides: fields.overrides,
                mode: prebindgen_declarations::ProjectionMode::Borrowed,
            }),
        )
    }
}

/// A type-wide source-error message mapping.
#[derive(Clone, Debug)]
pub struct DeclJniError {
    /// Source error type.
    pub ty: syn::Type,
    /// Source helper taking E or &E and returning String.
    pub mapper: DeclJniFunction,
}
impl DeclJniError {
    /// Select the source helper for this error type.
    pub fn new(ty: syn::Type, mapper: DeclJniFunction) -> Self {
        Self { ty, mapper }
    }
}
/// A type-wide input, output, or error-mapping declaration.
/// Generators immediately consume these builders into their conversion trees.
#[derive(Clone, Debug)]
pub enum DeclJniExpand {
    /// Input alternatives.
    Param(DeclJniExpandParam),
    /// Simultaneous output views.
    Return(DeclJniExpandReturn),
    /// A source-error message mapping.
    Error(Box<DeclJniError>),
}
impl From<DeclJniExpandParam> for DeclJniExpand {
    fn from(value: DeclJniExpandParam) -> Self {
        Self::Param(value)
    }
}
impl From<DeclJniExpandReturn> for DeclJniExpand {
    fn from(value: DeclJniExpandReturn) -> Self {
        Self::Return(value)
    }
}
impl From<DeclJniError> for DeclJniExpand {
    fn from(value: DeclJniError) -> Self {
        Self::Error(Box::new(value))
    }
}

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
impl From<DeclJniCallback> for JniInputRequest {
    fn from(decl: DeclJniCallback) -> Self {
        Self::ForType {
            ty: Box::new(decl.ty.clone()),
            input: Box::new(JniInput::Request(Self::Callback(Box::new(decl)))),
        }
    }
}
