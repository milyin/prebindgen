//! Production declaration syntax writing directly into conversion descriptions.
use crate::{
    DeclarationInput, DeclarationOutput, FunctionDecl, InputRequest, OutputRequest, Projection,
};
use crate::{InputBranch, InputUnary, OutputBranch, OutputLeafOp, OutputUnary};
use std::collections::BTreeMap;

/// Input alternatives for a source type, built directly as a [`DeclarationInput`] tree.
#[derive(Clone, Debug)]
pub struct ExpandParamDecl {
    /// Source type targeted by this default or occurrence override.
    pub ty: syn::Type,
    input: DeclarationInput<InputRequest, OutputRequest>,
    bulk: bool,
    reconstruct: bool,
}
impl ExpandParamDecl {
    /// Start an input declaration; add at least one alternative before use.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            input: DeclarationInput::Branch {
                operation: InputBranch::Selector,
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
    pub fn variant(mut self, function: impl Into<FunctionDecl>) -> Self {
        self.input = self.input.variant(function.into().reference_name(false));
        self
    }
    /// Append the type's direct language representation.
    pub fn variant_self(mut self) -> Self {
        self.input = self.input.variant_self();
        self
    }
}
impl From<ExpandParamDecl> for DeclarationInput<InputRequest, OutputRequest> {
    fn from(decl: ExpandParamDecl) -> Self {
        let mut input = decl.input;
        if decl.reconstruct {
            if matches!(&input, DeclarationInput::Branch { operation: InputBranch::Selector, children: arms } if arms.is_empty())
            {
                input = DeclarationInput::auto();
            }
            input = DeclarationInput::Unary {
                operation: InputUnary::Temporary,
                child: Box::new(input),
            };
        }
        if decl.bulk {
            input = DeclarationInput::Unary {
                operation: InputUnary::Bulk,
                child: Box::new(input),
            };
        }
        DeclarationInput::for_type(decl.ty, input)
    }
}

/// Simultaneous views of a source return value, built directly as a [`DeclarationOutput`] tree.
#[derive(Clone, Debug)]
pub struct ExpandReturnDecl {
    /// Source type targeted by this default or occurrence override.
    pub ty: syn::Type,
    output: DeclarationOutput<OutputRequest, InputRequest>,
    bulk: bool,
}
impl ExpandReturnDecl {
    /// Start an output declaration; add views before registering a type default.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            output: DeclarationOutput::Branch {
                operation: OutputBranch::Combine,
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
    pub fn field(mut self, function: impl Into<FunctionDecl>) -> Self {
        let function = function.into();
        let name = function.foreign.clone();
        let getter =
            DeclarationOutput::getter(function.reference_name(true), DeclarationOutput::auto());
        self.output = self.output.view(match name {
            Some(name) => DeclarationOutput::Unary {
                operation: OutputUnary::Named { name },
                child: Box::new(getter),
            },
            None => getter,
        });
        self
    }
    /// Append a getter's named record fields as separate output values.
    pub fn fields(self, fields: FieldsDecl) -> Self {
        self.view(fields)
    }
    /// Append another explicit output view.
    pub fn view(
        mut self,
        output: impl Into<DeclarationOutput<OutputRequest, InputRequest>>,
    ) -> Self {
        self.output = self.output.view(output.into());
        self
    }
    /// Append a consuming accessor followed by owned field decomposition.
    pub fn fields_into(mut self, fields: FieldsDecl) -> Self {
        self.output = self.output.fields_into(fields.into());
        self
    }
    /// Deprecated spelling of [`Self::fields_into`].
    #[deprecated(note = "use fields_into instead")]
    pub fn fields_self_into(self, fields: FieldsDecl) -> Self {
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
impl From<ExpandReturnDecl> for DeclarationOutput<OutputRequest, InputRequest> {
    fn from(decl: ExpandReturnDecl) -> Self {
        DeclarationOutput::for_type(
            decl.ty,
            if decl.bulk {
                DeclarationOutput::Unary {
                    operation: OutputUnary::Bulk,
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
pub struct FieldsDecl {
    function: String,
    overrides: BTreeMap<String, DeclarationOutput<OutputRequest, InputRequest>>,
}
impl FieldsDecl {
    /// Select a getter whose returned record supplies the fields.
    pub fn new(function: syn::Ident) -> Self {
        Self {
            function: function.to_string(),
            overrides: BTreeMap::new(),
        }
    }
    /// Replace one field's conversion. An empty declaration omits the whole field,
    /// including any enclosing Option or sequence representation.
    pub fn field(self, name: impl AsRef<str>, output: ExpandReturnDecl) -> Self {
        self.field_override(name.as_ref().to_owned(), output)
    }
    /// Set an explicit conversion tree for one field, with the same omission
    /// semantics as [`Self::field`] for an empty typed declaration.
    pub fn field_override(
        mut self,
        name: impl Into<String>,
        output: impl Into<DeclarationOutput<OutputRequest, InputRequest>>,
    ) -> Self {
        let output = match output.into() {
            DeclarationOutput::Request(OutputRequest::ForType { ty, output }) if matches!(output.as_ref(),DeclarationOutput::Branch { operation: OutputBranch::Combine, children: parts } if parts.is_empty()) => {
                DeclarationOutput::for_type(*ty, DeclarationOutput::Leaf(OutputLeafOp::Discard))
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
impl From<FieldsDecl> for DeclarationOutput<OutputRequest, InputRequest> {
    fn from(fields: FieldsDecl) -> Self {
        DeclarationOutput::getter(
            fields.function,
            DeclarationOutput::Request(OutputRequest::NamedFields {
                overrides: fields.overrides,
                mode: crate::ProjectionMode::Borrowed,
            }),
        )
    }
}

/// A type-wide source-error message mapping.
#[derive(Clone, Debug)]
pub struct ErrorDecl {
    /// Source error type.
    pub ty: syn::Type,
    /// Source helper taking E or &E and returning String.
    pub mapper: FunctionDecl,
}
impl ErrorDecl {
    /// Select the source helper for this error type.
    pub fn new(ty: syn::Type, mapper: FunctionDecl) -> Self {
        Self { ty, mapper }
    }
}
/// A type-wide input, output, or error-mapping declaration.
/// Generators immediately consume these builders into their conversion trees.
#[derive(Clone, Debug)]
pub enum ExpandDecl {
    /// Input alternatives.
    Param(ExpandParamDecl),
    /// Simultaneous output views.
    Return(ExpandReturnDecl),
    /// A source-error message mapping.
    Error(Box<ErrorDecl>),
}
impl From<ExpandParamDecl> for ExpandDecl {
    fn from(value: ExpandParamDecl) -> Self {
        Self::Param(value)
    }
}
impl From<ExpandReturnDecl> for ExpandDecl {
    fn from(value: ExpandReturnDecl) -> Self {
        Self::Return(value)
    }
}
impl From<ErrorDecl> for ExpandDecl {
    fn from(value: ErrorDecl) -> Self {
        Self::Error(Box::new(value))
    }
}
