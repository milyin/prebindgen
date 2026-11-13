use super::*;
/// Input alternatives for a source type, built directly as a [`CInput`] tree.
#[derive(Clone, Debug)]
pub struct DeclCExpandParam {
    /// Source type targeted by this default or occurrence override.
    pub ty: syn::Type,
    input: CInput<CInputRequest, COutputRequest>,
    reconstruct: bool,
}
impl DeclCExpandParam {
    /// Start an input declaration; add at least one alternative before use.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            input: CInput::Branch {
                operation: CInputBranch::Selector,
                children: vec![],
            },
            reconstruct: false,
        }
    }
    /// Explicitly construct an owned temporary and borrow it for the source call.
    /// With no alternatives, use the owned value's default input conversion.
    /// Alternatives describe the owned referent, not the reference itself.
    pub fn reconstruct(mut self) -> Self {
        self.reconstruct = true;
        self
    }
    /// Append a constructor reference; its arguments use their type defaults.
    pub fn variant(mut self, function: impl Into<DeclCFunction>) -> Self {
        self.input = self.input.variant(function.into().reference_name(false));
        self
    }
    /// Append the type's direct language representation.
    pub fn variant_self(mut self) -> Self {
        self.input = self.input.variant_self();
        self
    }
}
impl From<DeclCExpandParam> for CInput<CInputRequest, COutputRequest> {
    fn from(decl: DeclCExpandParam) -> Self {
        let mut input = decl.input;
        if decl.reconstruct {
            if matches ! (& input , CInput :: Branch { operation : CInputBranch :: Selector , children : arms } if arms . is_empty ())
            {
                input = CInput::auto();
            }
            input = CInput::Unary {
                operation: CInputUnary::Temporary,
                child: Box::new(input),
            };
        }
        CInput::for_type(decl.ty, input)
    }
}
/// Simultaneous views of a source return value, built directly as a [`COutput`] tree.
#[derive(Clone, Debug)]
pub struct DeclCExpandReturn {
    /// Source type targeted by this default or occurrence override.
    pub ty: syn::Type,
    output: COutput<CInputRequest, COutputRequest>,
}
impl DeclCExpandReturn {
    /// Start an output declaration; add views before registering a type default.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            output: COutput::Branch {
                operation: COutputBranch::Combine,
                children: vec![],
            },
        }
    }
    /// Append a getter view. An explicit `.name()` labels the delivered view.
    /// Shared results are observed before another view moves the owner. An
    /// owning opaque copy requires an explicit projection or cloning accessor.
    pub fn field(mut self, function: impl Into<DeclCFunction>) -> Self {
        let function = function.into();
        let name = function.foreign.clone();
        let getter = COutput::getter(function.reference_name(true), COutput::auto());
        self.output = self.output.view(match name {
            Some(name) => COutput::Unary {
                operation: COutputUnary::Named { name },
                child: Box::new(getter),
            },
            None => getter,
        });
        self
    }
    /// Append a getter's named record fields as separate output values.
    pub fn fields(self, fields: DeclCFields) -> Self {
        self.view(fields)
    }
    /// Append another explicit output view.
    pub fn view(mut self, output: impl Into<COutput<CInputRequest, COutputRequest>>) -> Self {
        self.output = self.output.view(output.into());
        self
    }
    /// Append a consuming accessor followed by owned field decomposition.
    pub fn fields_into(mut self, fields: DeclCFields) -> Self {
        self.output = self.output.fields_into(fields.into());
        self
    }
    /// Deprecated spelling of [`Self::fields_into`].
    #[deprecated(note = "use fields_into instead")]
    pub fn fields_self_into(self, fields: DeclCFields) -> Self {
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
impl From<DeclCExpandReturn> for COutput<CInputRequest, COutputRequest> {
    fn from(decl: DeclCExpandReturn) -> Self {
        COutput::for_type(decl.ty, decl.output)
    }
}
/// A source accessor followed by named field decomposition.
/// Field adjustments edit the accessor's output tree directly.
#[derive(Clone, Debug)]
pub struct DeclCFields {
    function: String,
    overrides: BTreeMap<String, COutput<CInputRequest, COutputRequest>>,
}
impl DeclCFields {
    /// Select a getter whose returned record supplies the fields.
    pub fn new(function: syn::Ident) -> Self {
        Self {
            function: function.to_string(),
            overrides: BTreeMap::new(),
        }
    }
    /// Replace one field's conversion. An empty declaration omits the whole field,
    /// including any enclosing Option or sequence representation.
    pub fn field(self, name: impl AsRef<str>, output: DeclCExpandReturn) -> Self {
        self.field_override(name.as_ref().to_owned(), output)
    }
    /// Set an explicit conversion tree for one field, with the same omission
    /// semantics as [`Self::field`] for an empty typed declaration.
    pub fn field_override(
        mut self,
        name: impl Into<String>,
        output: impl Into<COutput<CInputRequest, COutputRequest>>,
    ) -> Self {
        let output = match output.into() {
            COutput::Request(COutputRequest::ForType { ty, output }) if matches ! (output . as_ref () , COutput :: Branch { operation : COutputBranch :: Combine , children : parts } if parts . is_empty ()) => {
                COutput::for_type(*ty, COutput::Leaf(COutputLeafOp::Discard))
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
impl From<DeclCFields> for COutput<CInputRequest, COutputRequest> {
    fn from(fields: DeclCFields) -> Self {
        COutput::getter(
            fields.function,
            COutput::Request(COutputRequest::NamedFields {
                overrides: fields.overrides,
                mode: prebindgen_declarations::ProjectionMode::Borrowed,
            }),
        )
    }
}
/// A type-wide source-error message mapping.
#[derive(Clone, Debug)]
pub struct DeclCError {
    /// Source error type.
    pub ty: syn::Type,
    /// Source helper taking E or &E and returning String.
    pub mapper: DeclCFunction,
}
impl DeclCError {
    /// Select the source helper for this error type.
    pub fn new(ty: syn::Type, mapper: DeclCFunction) -> Self {
        Self { ty, mapper }
    }
}
/// A type-wide input, output, or error-mapping declaration.
/// Generators immediately consume these builders into their conversion trees.
#[derive(Clone, Debug)]
pub enum DeclCExpand {
    /// Input alternatives.
    Param(DeclCExpandParam),
    /// Simultaneous output views.
    Return(DeclCExpandReturn),
    /// A source-error message mapping.
    Error(Box<DeclCError>),
}
impl From<DeclCExpandParam> for DeclCExpand {
    fn from(value: DeclCExpandParam) -> Self {
        Self::Param(value)
    }
}
impl From<DeclCExpandReturn> for DeclCExpand {
    fn from(value: DeclCExpandReturn) -> Self {
        Self::Return(value)
    }
}
impl From<DeclCError> for DeclCExpand {
    fn from(value: DeclCError) -> Self {
        Self::Error(Box::new(value))
    }
}
/// Callback declaration containing this language's argument and result trees.
pub type DeclCCallback = prebindgen_declarations::DeclCallback<
    CInput<CInputRequest, COutputRequest>,
    COutput<CInputRequest, COutputRequest>,
>;
impl From<DeclCCallback> for CInputRequest {
    fn from(decl: DeclCCallback) -> Self {
        Self::ForType {
            ty: Box::new(decl.ty.clone()),
            input: Box::new(CInput::Request(Self::Callback(Box::new(decl)))),
        }
    }
}
