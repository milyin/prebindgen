//! Test adapter syntax used only by compiler conformance tests.
#![allow(missing_docs, unused_imports, unused_variables)]
use crate::*;
use prebindgen_flat::{same_type, type_key, Flat, Result, Schema};
use prebindgen_flat::{Function, Param, TypeUse, ValueConversion};
use proc_macro2::TokenStream;
use std::collections::BTreeMap;
use std::convert::Infallible;
use syn::Type;
/// Input syntax accepted by the optional declaration frontend.
#[derive(Clone, Debug)]
pub struct TestInputOps;
/// Output syntax accepted by the optional declaration frontend.
#[derive(Clone, Debug)]
pub struct TestOutputOps;
impl InputOps for TestInputOps {
    type Leaf = TestInputLeafOp;
    type Unary = TestInputUnary;
    type Branch = TestInputBranch;
    type Arrow = Box<crate::CallbackOperation>;
}
impl OutputOps for TestOutputOps {
    type Leaf = TestOutputLeafOp;
    type Unary = TestOutputUnary;
    type Branch = TestOutputBranch;
    type Arrow = Infallible;
}
/// Input syntax before adapter lowering.
///
/// Preparation uses `TestInput<TestInputRequest, TestOutputRequest>`.
/// Plain `TestInput` uses `Infallible` for both directions, including
/// reverse-direction arrow domains.
pub type TestInput<InRequest = Infallible, OutRequest = Infallible> =
    InputNode<TestInputOps, TestOutputOps, InRequest, OutRequest>;
/// Output declaration tree, with input requests first and output requests second.
pub type TestOutput<InRequest = Infallible, OutRequest = Infallible> =
    OutputNode<TestInputOps, TestOutputOps, InRequest, OutRequest>;

/// Terminal input operations.
#[derive(Clone, Debug)]
pub enum TestInputLeafOp {
    /// Convert the occurrence using its declared value representation.
    Value,
}
/// Single-child input operations.
#[derive(Clone, Debug)]
pub enum TestInputUnary {
    /// Apply an explicitly selected Rust value conversion.
    Convert {
        /// Checked source/representation conversion.
        conversion: Box<prebindgen_flat::ValueConversion>,
    },
    /// Construct a collection from converted elements.
    Sequence,
    /// Construct an owned temporary and lend it for this call.
    Temporary,
    /// Request the adapter’s packed collection/record transport.
    Bulk,
    /// Convert the payload only when present.
    Optional,
}
/// Multiple-child input operations.
#[derive(Clone, Debug)]
pub enum TestInputBranch {
    /// Convert source record fields in declaration order.
    Fields,
    /// Group converted fields into the declared foreign record representation.
    Record,
    /// Select one constructor alternative by its incoming tag.
    Selector,
    /// Construct a value by calling a source function.
    Constructor {
        /// Unqualified source function name.
        function: String,
    },
    /// Construct a source enum from its selected variant branch.
    Enum {
        /// Group the foreign value instead of exposing tag/field arguments.
        value: bool,
    },
    /// Fields of one source enum variant; interpreted by its enum parent.
    Variant,
}
/// Terminal output operations.
#[derive(Clone, Debug)]
pub enum TestOutputLeafOp {
    /// Convert the occurrence using its declared value representation.
    Value,
    /// Omit a value while preserving its ordinary Rust destruction.
    Discard,
    /// Apply an explicit observation or move policy to a terminal value.
    Projected(Projection),
}
/// Single-child output operations.
#[derive(Clone, Debug)]
pub enum TestOutputUnary {
    /// Ownership contract of one callable argument; interpreted by its arrow.
    CallbackArgument {
        /// Whether produced foreign owners expire at the end of invocation.
        ownership: crate::CallbackArgumentOwnership,
    },
    /// Request the adapter’s packed collection/record transport.
    Bulk,
    /// Give one delivered view an explicit foreign field name.
    Named {
        /// Foreign field name.
        name: String,
    },
    /// Apply an explicitly selected Rust value conversion.
    Convert {
        /// Checked source/representation conversion.
        conversion: Box<prebindgen_flat::ValueConversion>,
    },
    /// Convert and deliver each collection element.
    Sequence {
        /// Materialized or per-element callback delivery.
        delivery: SequenceDelivery,
        /// Explicit move/observation policy, or the source occurrence’s policy.
        mode: Option<ProjectionMode>,
    },
    /// Convert success and route a source error through the adapter.
    Result {
        /// Explicit source message mapper, if selected.
        error: Option<String>,
    },
    /// Convert the payload only when present.
    Optional,
    /// Call a consuming source accessor and convert its result.
    Into {
        /// Unqualified source function name.
        function: String,
    },
    /// Observe through a source getter and convert its result.
    Getter {
        /// Unqualified source function name.
        function: String,
    },
}
/// Multiple-child output operations.
#[derive(Clone, Debug)]
pub enum TestOutputBranch {
    /// Convert source record fields in declaration order.
    Fields,
    /// Move source record fields into their conversions.
    OwnedFields,
    /// Deliver a foreign record with explicit field conversions.
    Record {
        /// Explicit move/observation policy, or the source occurrence’s policy.
        mode: Option<ProjectionMode>,
    },
    /// Deliver multiple views, observing before a possible consuming view.
    Combine,
    /// Match a source enum and convert its active variant branch.
    Enum {
        /// Group the foreign value instead of exposing tag/field arguments.
        value: bool,
        /// Explicit move/observation policy, or the source occurrence’s policy.
        mode: Option<ProjectionMode>,
    },
    /// Fields of one source enum variant; interpreted by its enum parent.
    Variant,
    /// A callable domain in source parameter order.
    Arguments,
}

impl<InRequest, OutRequest> TestOutput<InRequest, OutRequest> {
    /// Group a callable's domain in source parameter order.
    pub fn arguments(children: Vec<Self>) -> Self {
        if children.is_empty() {
            Self::None
        } else {
            Self::Branch {
                operation: TestOutputBranch::Arguments,
                children,
            }
        }
    }
    /// Read the source-ordered components of a callable domain.
    pub fn argument_nodes(&self) -> crate::Result<&[Self]> {
        match self {
            Self::None => Ok(&[]),
            Self::Branch {
                operation: TestOutputBranch::Arguments,
                children,
            } => Ok(children),
            _ => Err("callback domain requires an argument branch or None".into()),
        }
    }
}

impl<InRequest, OutRequest> TestInput<InRequest, OutRequest> {
    /// Describe source variants, each with an ordinary branch of field conversions.
    pub fn enum_variants(variants: Vec<Vec<Self>>, value: bool) -> Self {
        Self::Branch {
            operation: TestInputBranch::Enum { value },
            children: variants
                .into_iter()
                .map(|children| Self::Branch {
                    operation: TestInputBranch::Variant,
                    children,
                })
                .collect(),
        }
    }
    /// Read an enum variant's field conversions.
    pub fn variant_fields(&self) -> crate::Result<&[Self]> {
        match self {
            Self::Branch {
                operation: TestInputBranch::Variant,
                children,
            } => Ok(children),
            _ => Err("enum children must be variant branches".into()),
        }
    }
}

impl<InRequest, OutRequest> TestOutput<InRequest, OutRequest> {
    /// Describe source variants, each with an ordinary branch of field conversions.
    pub fn enum_variants(
        variants: Vec<Vec<Self>>,
        value: bool,
        mode: Option<ProjectionMode>,
    ) -> Self {
        Self::Branch {
            operation: TestOutputBranch::Enum { value, mode },
            children: variants
                .into_iter()
                .map(|children| Self::Branch {
                    operation: TestOutputBranch::Variant,
                    children,
                })
                .collect(),
        }
    }
    /// Read an enum variant's field conversions.
    pub fn variant_fields(&self) -> crate::Result<&[Self]> {
        match self {
            Self::Branch {
                operation: TestOutputBranch::Variant,
                children,
            } => Ok(children),
            _ => Err("enum children must be variant branches".into()),
        }
    }
}

impl<InRequest, OutRequest> TestInput<InRequest, OutRequest> {
    /// Convert each incoming collection element using the supplied subtree.
    pub fn sequence(element: Self) -> Self {
        Self::Unary {
            operation: TestInputUnary::Sequence,
            child: Box::new(element),
        }
    }
}

impl<InRequest, OutRequest> TestOutput<InRequest, OutRequest> {
    /// Attach the lifetime of produced foreign owners to this callable argument.
    pub fn callback_argument(self, ownership: crate::CallbackArgumentOwnership) -> Self {
        Self::Unary {
            operation: TestOutputUnary::CallbackArgument { ownership },
            child: Box::new(self),
        }
    }
    /// Read an argument's conversion and ownership contract from its own subtree.
    pub fn callback_argument_parts(
        &self,
    ) -> crate::Result<(crate::CallbackArgumentOwnership, &Self)> {
        match self {
            Self::Unary {
                operation: TestOutputUnary::CallbackArgument { ownership },
                child,
            } => Ok((*ownership, child)),
            _ => Err("callback argument requires an explicit ownership contract".into()),
        }
    }
}

/// Source-dependent choices accepted only while preparing an input tree.
#[derive(Clone, Debug)]
pub enum TestInputRequest {
    /// Expand enum fields, inheriting defaults for fields not explicitly overridden.
    EnumFields(
        std::collections::BTreeMap<
            String,
            std::collections::BTreeMap<String, TestInput<TestInputRequest, TestOutputRequest>>,
        >,
    ),
    /// Resolve only the canonical value form, independently of boundary defaults.
    Value,
    /// Select a local single-value conversion.
    Conversion(Box<DeclConvert>),
    /// Select the configured default for this source occurrence.
    Auto,
    /// Select or override a callback signature's argument and result conversions.
    Callback(Box<TestCallback>),
    /// Apply a type-specific override, including inside optional/sequence sources.
    ForType {
        /// Source type targeted by the override.
        ty: Box<Type>,
        /// The requested conversion for that type.
        input: Box<TestInput<TestInputRequest, TestOutputRequest>>,
    },
    /// Infer a constructor's argument conversions from its source signature.
    Call(String),
}

impl TestInput<TestInputRequest, TestOutputRequest> {
    /// Expand the tag and selected fields. Keys are source variant and field
    /// names (or tuple indices). Unspecified fields inherit boundary defaults.
    pub fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self {
        TestInput::Request(TestInputRequest::EnumFields(crate::enum_overrides(
            overrides,
        )))
    }
    /// Request the canonical value representation, bypassing boundary defaults.
    pub fn value() -> Self {
        TestInput::Request(TestInputRequest::Value)
    }
    /// Request this source type's configured default during preparation.
    pub fn auto() -> Self {
        TestInput::Request(TestInputRequest::Auto)
    }
    /// Associate a conversion with a source type for a default or nested override.
    pub fn for_type(ty: Type, input: Self) -> Self {
        TestInput::Request(TestInputRequest::ForType {
            ty: Box::new(ty),
            input: Box::new(input),
        })
    }
    /// Infer a source constructor's argument conversions from registered defaults.
    pub fn call(name: impl Into<String>) -> Self {
        TestInput::Request(TestInputRequest::Call(name.into()))
    }
    /// Explicitly reconstruct an owned temporary for a shared-reference input.
    /// Slices construct a Vec, str constructs a String, and sized references
    /// construct their referent. Child recipes keep their ordinary ownership
    /// semantics; this operation authorizes the temporary's allocation/copying.
    pub fn reconstruct(self) -> Self {
        match self {
            TestInput::Request(TestInputRequest::ForType { ty, input }) => TestInput::for_type(
                *ty,
                TestInput::Unary {
                    operation: TestInputUnary::Temporary,
                    child: input,
                },
            ),
            other => TestInput::Unary {
                operation: TestInputUnary::Temporary,
                child: Box::new(other),
            },
        }
    }
    /// Add a constructor alternative to this conversion tree.
    pub fn variant(self, name: impl Into<String>) -> Self {
        self.add_alternative(TestInput::call(name))
    }
    /// Add a direct codec alternative, bypassing this type's expansion default.
    pub fn variant_self(self) -> Self {
        self.add_alternative(TestInput::Leaf(TestInputLeafOp::Value))
    }
    fn add_alternative(self, arm: Self) -> Self {
        match self {
            TestInput::Request(TestInputRequest::ForType { ty, input }) => {
                TestInput::for_type(*ty, input.add_alternative(arm))
            }
            TestInput::Branch {
                operation: TestInputBranch::Selector,
                children: mut arms,
            } => {
                arms.push(arm);
                TestInput::Branch {
                    operation: TestInputBranch::Selector,
                    children: arms,
                }
            }
            other => TestInput::Branch {
                operation: TestInputBranch::Selector,
                children: vec![other, arm],
            },
        }
    }
}
impl<InRequest, OutRequest> TestInput<InRequest, OutRequest> {
    /// Group this explicit expansion into a bulk carrier.
    pub fn bulk(self) -> Self {
        TestInput::Unary {
            operation: TestInputUnary::Bulk,
            child: Box::new(self),
        }
    }
    /// Construct a record from field recipes, or unit from an empty list.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration
    /// order. Binding verifies the count against the record.
    pub fn fields(fields: impl IntoIterator<Item = TestInput<InRequest, OutRequest>>) -> Self {
        TestInput::Branch {
            operation: TestInputBranch::Fields,
            children: fields.into_iter().collect(),
        }
    }
    /// Call a named source constructor with the supplied argument recipes.
    ///
    /// Binding checks its result type and argument count. Argument validation and
    /// ownership acquisition precede the source call.
    ///
    /// # Arguments
    ///
    /// * `name` — unqualified name of a source function whose result type is the
    ///   input's type.
    /// * `args` — one recipe per parameter of that function, in order.
    pub fn constructor(
        name: impl Into<String>,
        args: impl IntoIterator<Item = TestInput<InRequest, OutRequest>>,
    ) -> Self {
        TestInput::Branch {
            operation: TestInputBranch::Constructor {
                function: name.into(),
            },
            children: args.into_iter().collect(),
        }
    }
    /// Choose between owned representations of the same input type.
    ///
    /// `arms` holds one recipe per alternative; each must produce the input's
    /// type. The caller passes the zero-based index of the chosen alternative in an
    /// extra wire ahead of all arms' wires.
    ///
    /// Requires at least one arm. Each arm retains its own wires; this does not
    /// merge equivalent wire parameters or generate destination-language overloads.
    pub fn selector(arms: impl IntoIterator<Item = TestInput<InRequest, OutRequest>>) -> Self {
        TestInput::Branch {
            operation: TestInputBranch::Selector,
            children: arms.into_iter().collect(),
        }
    }
}
/// Source-dependent choices accepted only while preparing an output tree.
#[derive(Clone, Debug)]
pub enum TestOutputRequest {
    /// Select per-element delivery after resolving defaults and local expansion.
    SequenceCallback {
        /// Expansion whose outer sequence supplies callback elements.
        output: Box<TestOutput<TestInputRequest, TestOutputRequest>>,
        /// Allow structural decomposition of a default collection identity.
        structural: bool,
    },
    /// Expand enum fields, inheriting boundary defaults for unspecified fields.
    EnumFields(
        std::collections::BTreeMap<
            String,
            std::collections::BTreeMap<String, TestOutput<TestInputRequest, TestOutputRequest>>,
        >,
    ),
    /// Resolve only the canonical value form, independently of boundary defaults.
    Value {
        /// Explicit field ownership/projection policy, if any.
        projection: Option<Projection>,
    },
    /// Select a local single-value conversion.
    Conversion(Box<DeclConvert>),
    /// Select the configured output default for this source occurrence.
    Auto,
    /// Apply a type-specific override, including inside optional/sequence sources.
    ForType {
        /// Source type targeted by the override.
        ty: Box<Type>,
        /// Requested conversion for the matching type.
        output: Box<TestOutput<TestInputRequest, TestOutputRequest>>,
    },
    /// Resolve source record fields by name and fill unspecified conversions.
    NamedFields {
        /// Explicit conversions keyed by source field name.
        overrides:
            std::collections::BTreeMap<String, TestOutput<TestInputRequest, TestOutputRequest>>,
        /// Ownership of field decomposition.
        mode: ProjectionMode,
    },
}

impl TestOutput<TestInputRequest, TestOutputRequest> {
    /// Expand an enum's tag and active fields using source variant/member keys.
    pub fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self {
        TestOutput::Request(TestOutputRequest::EnumFields(crate::enum_overrides(
            overrides,
        )))
    }
    /// Request the canonical value representation, bypassing boundary defaults.
    pub fn value() -> Self {
        TestOutput::Request(TestOutputRequest::Value { projection: None })
    }
    /// Request this source type's configured output default during preparation.
    pub fn auto() -> Self {
        TestOutput::Request(TestOutputRequest::Auto)
    }
    /// Associate a conversion with a source type for a default or nested override.
    pub fn for_type(ty: Type, output: Self) -> Self {
        TestOutput::Request(TestOutputRequest::ForType {
            ty: Box::new(ty),
            output: Box::new(output),
        })
    }
    /// Append another simultaneous output view.
    pub fn view(self, view: impl Into<Self>) -> Self {
        let view = view.into();
        match self {
            TestOutput::Request(TestOutputRequest::ForType { ty, output }) => {
                TestOutput::for_type(*ty, output.view(view))
            }
            TestOutput::Branch {
                operation: TestOutputBranch::Combine,
                children: mut parts,
            } => {
                parts.push(view);
                TestOutput::Branch {
                    operation: TestOutputBranch::Combine,
                    children: parts,
                }
            }
            other => TestOutput::Branch {
                operation: TestOutputBranch::Combine,
                children: vec![other, view],
            },
        }
    }
    /// Append a source getter whose result uses its registered conversion default.
    pub fn field(self, name: impl Into<String>) -> Self {
        self.view(TestOutput::getter(name, TestOutput::auto()))
    }
    /// Append the direct codec representation, bypassing this type's expansion default.
    pub fn field_self(self) -> Self {
        self.view(TestOutput::Leaf(TestOutputLeafOp::Value))
    }
    /// Append an explicit borrowed projection.
    pub fn field_self_ref(self) -> Self {
        self.project_self(Projection::borrowed())
    }
    /// Append an explicit owned projection.
    pub fn field_self_into(self) -> Self {
        self.project_self(Projection::owned())
    }
    /// Append a projection with explicit ownership and nested field policies.
    pub fn project_self(self, policy: Projection) -> Self {
        self.view(TestOutput::Leaf(TestOutputLeafOp::Projected(policy)))
    }
    /// Append a consuming accessor followed by owned field decomposition.
    pub fn fields_into(self, view: Self) -> Self {
        let TestOutput::Unary {
            operation: TestOutputUnary::Getter { function },
            child: output,
        } = view
        else {
            panic!("fields_into requires a test_fields! accessor")
        };
        let TestOutput::Request(TestOutputRequest::NamedFields { overrides, .. }) = *output else {
            panic!("fields_into requires named fields")
        };
        self.view(TestOutput::Unary {
            operation: TestOutputUnary::Into { function },
            child: Box::new(TestOutput::Request(TestOutputRequest::NamedFields {
                overrides,
                mode: ProjectionMode::Owned,
            })),
        })
    }
    /// Deprecated spelling of [`TestOutput::fields_into`].
    #[deprecated(note = "use fields_into instead")]
    pub fn fields_self_into(self, view: Self) -> Self {
        self.fields_into(view)
    }
    /// Override one named source field, preserving defaults for its siblings.
    pub fn field_override(mut self, name: impl Into<String>, output: Self) -> Self {
        match &mut self {
            TestOutput::Request(TestOutputRequest::NamedFields { overrides, .. }) => {
                assert!(
                    overrides.insert(name.into(), output).is_none(),
                    "duplicate field override"
                );
            }
            TestOutput::Unary {
                operation: TestOutputUnary::Getter { .. },
                child: inner,
                ..
            }
            | TestOutput::Unary {
                operation: TestOutputUnary::Into { .. },
                child: inner,
                ..
            }
            | TestOutput::Request(TestOutputRequest::ForType { output: inner, .. }) => {
                let old = std::mem::replace(inner, Box::new(TestOutput::auto()));
                **inner = old.field_override(name, output);
            }
            _ => panic!("field override requires named field decomposition"),
        }
        self
    }
}
impl<InRequest, OutRequest> TestOutput<InRequest, OutRequest> {
    /// Group this explicit expansion into a bulk carrier.
    pub fn bulk(self) -> Self {
        TestOutput::Unary {
            operation: TestOutputUnary::Bulk,
            child: Box::new(self),
        }
    }
    /// Observe record fields in declaration order; use an empty list for unit.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration order.
    pub fn fields(fields: impl IntoIterator<Item = TestOutput<InRequest, OutRequest>>) -> Self {
        TestOutput::Branch {
            operation: TestOutputBranch::Fields,
            children: fields.into_iter().collect(),
        }
    }
    /// Observe a value through a source getter and convert the getter's result.
    ///
    /// The getter takes `&T` for the current type `T`; adapters retaining a final
    /// output owner may also support `&mut T`. Its result uses the selected
    /// output conversion, including supported reference results.
    ///
    /// # Arguments
    ///
    /// * `name` — unqualified name of the source getter.
    /// * `output` — recipe for the getter's result.
    pub fn getter(name: impl Into<String>, output: TestOutput<InRequest, OutRequest>) -> Self {
        TestOutput::Unary {
            operation: TestOutputUnary::Getter {
                function: name.into(),
            },
            child: Box::new(output),
        }
    }
    /// Produce multiple outputs from one value.
    ///
    /// `parts` holds one recipe per view of the value; their wires appear in this
    /// order. Binding rejects an empty list or more than one consuming part. The
    /// framework uses [`Codec::output_consumes`] to order observations before a move,
    /// unless [`Codec::output_owner`] first retains the final output allocation.
    pub fn combine(parts: impl IntoIterator<Item = TestOutput<InRequest, OutRequest>>) -> Self {
        TestOutput::Branch {
            operation: TestOutputBranch::Combine,
            children: parts.into_iter().collect(),
        }
    }
}

/// Input alternatives for a source type, built directly as a [`TestInput`] tree.
#[derive(Clone, Debug)]
pub struct TestExpandParam {
    /// Source type targeted by this default or occurrence override.
    pub ty: syn::Type,
    input: TestInput<TestInputRequest, TestOutputRequest>,
    bulk: bool,
    reconstruct: bool,
}
impl TestExpandParam {
    /// Start an input declaration; add at least one alternative before use.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            input: TestInput::Branch {
                operation: TestInputBranch::Selector,
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
    pub fn variant(mut self, function: impl Into<TestFunction>) -> Self {
        self.input = self.input.variant(function.into().reference_name(false));
        self
    }
    /// Append the type's direct language representation.
    pub fn variant_self(mut self) -> Self {
        self.input = self.input.variant_self();
        self
    }
}
impl From<TestExpandParam> for TestInput<TestInputRequest, TestOutputRequest> {
    fn from(decl: TestExpandParam) -> Self {
        let mut input = decl.input;
        if decl.reconstruct {
            if matches!(&input, TestInput::Branch { operation: TestInputBranch::Selector, children: arms } if arms.is_empty())
            {
                input = TestInput::auto();
            }
            input = TestInput::Unary {
                operation: TestInputUnary::Temporary,
                child: Box::new(input),
            };
        }
        if decl.bulk {
            input = TestInput::Unary {
                operation: TestInputUnary::Bulk,
                child: Box::new(input),
            };
        }
        TestInput::for_type(decl.ty, input)
    }
}

/// Simultaneous views of a source return value, built directly as a [`TestOutput`] tree.
#[derive(Clone, Debug)]
pub struct TestExpandReturn {
    /// Source type targeted by this default or occurrence override.
    pub ty: syn::Type,
    output: TestOutput<TestInputRequest, TestOutputRequest>,
    bulk: bool,
}
impl TestExpandReturn {
    /// Start an output declaration; add views before registering a type default.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            output: TestOutput::Branch {
                operation: TestOutputBranch::Combine,
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
    pub fn field(mut self, function: impl Into<TestFunction>) -> Self {
        let function = function.into();
        let name = function.foreign.clone();
        let getter = TestOutput::getter(function.reference_name(true), TestOutput::auto());
        self.output = self.output.view(match name {
            Some(name) => TestOutput::Unary {
                operation: TestOutputUnary::Named { name },
                child: Box::new(getter),
            },
            None => getter,
        });
        self
    }
    /// Append a getter's named record fields as separate output values.
    pub fn fields(self, fields: TestFields) -> Self {
        self.view(fields)
    }
    /// Append another explicit output view.
    pub fn view(
        mut self,
        output: impl Into<TestOutput<TestInputRequest, TestOutputRequest>>,
    ) -> Self {
        self.output = self.output.view(output.into());
        self
    }
    /// Append a consuming accessor followed by owned field decomposition.
    pub fn fields_into(mut self, fields: TestFields) -> Self {
        self.output = self.output.fields_into(fields.into());
        self
    }
    /// Deprecated spelling of [`Self::fields_into`].
    #[deprecated(note = "use fields_into instead")]
    pub fn fields_self_into(self, fields: TestFields) -> Self {
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
impl From<TestExpandReturn> for TestOutput<TestInputRequest, TestOutputRequest> {
    fn from(decl: TestExpandReturn) -> Self {
        TestOutput::for_type(
            decl.ty,
            if decl.bulk {
                TestOutput::Unary {
                    operation: TestOutputUnary::Bulk,
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
pub struct TestFields {
    function: String,
    overrides: BTreeMap<String, TestOutput<TestInputRequest, TestOutputRequest>>,
}
impl TestFields {
    /// Select a getter whose returned record supplies the fields.
    pub fn new(function: syn::Ident) -> Self {
        Self {
            function: function.to_string(),
            overrides: BTreeMap::new(),
        }
    }
    /// Replace one field's conversion. An empty declaration omits the whole field,
    /// including any enclosing Option or sequence representation.
    pub fn field(self, name: impl AsRef<str>, output: TestExpandReturn) -> Self {
        self.field_override(name.as_ref().to_owned(), output)
    }
    /// Set an explicit conversion tree for one field, with the same omission
    /// semantics as [`Self::field`] for an empty typed declaration.
    pub fn field_override(
        mut self,
        name: impl Into<String>,
        output: impl Into<TestOutput<TestInputRequest, TestOutputRequest>>,
    ) -> Self {
        let output = match output.into() {
            TestOutput::Request(TestOutputRequest::ForType { ty, output }) if matches!(output.as_ref(),TestOutput::Branch { operation: TestOutputBranch::Combine, children: parts } if parts.is_empty()) => {
                TestOutput::for_type(*ty, TestOutput::Leaf(TestOutputLeafOp::Discard))
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
impl From<TestFields> for TestOutput<TestInputRequest, TestOutputRequest> {
    fn from(fields: TestFields) -> Self {
        TestOutput::getter(
            fields.function,
            TestOutput::Request(TestOutputRequest::NamedFields {
                overrides: fields.overrides,
                mode: crate::ProjectionMode::Borrowed,
            }),
        )
    }
}

pub type TestFunction = crate::DeclFunction<
    TestInput<TestInputRequest, TestOutputRequest>,
    TestOutput<TestInputRequest, TestOutputRequest>,
>;
pub type TestCallback = crate::DeclCallback<
    TestInput<TestInputRequest, TestOutputRequest>,
    TestOutput<TestInputRequest, TestOutputRequest>,
>;
impl From<TestCallback> for TestInputRequest {
    fn from(decl: TestCallback) -> Self {
        Self::ForType {
            ty: Box::new(decl.ty.clone()),
            input: Box::new(TestInput::Request(Self::Callback(Box::new(decl)))),
        }
    }
}
#[derive(Clone)]
/// This environment is discarded after prepared conversions have been installed.
pub struct TestDefaults {
    inputs: BTreeMap<String, TestInput<TestInputRequest, TestOutputRequest>>,
    outputs: BTreeMap<String, TestOutput<TestInputRequest, TestOutputRequest>>,
    errors: BTreeMap<String, String>,
}
fn input_body(
    input: TestInput<TestInputRequest, TestOutputRequest>,
) -> TestInput<TestInputRequest, TestOutputRequest> {
    match input {
        TestInput::Branch {
            operation: TestInputBranch::Selector,
            children: mut arms,
        } if arms.len() == 1 => arms.remove(0),
        TestInput::Unary {
            operation: TestInputUnary::Bulk,
            child,
        } => TestInput::Unary {
            operation: TestInputUnary::Bulk,
            child: Box::new(input_body(*child)),
        },
        TestInput::Unary {
            operation: TestInputUnary::Temporary,
            child,
        } => TestInput::Unary {
            operation: TestInputUnary::Temporary,
            child: Box::new(input_body(*child)),
        },
        other => other,
    }
}
fn output_body(
    output: TestOutput<TestInputRequest, TestOutputRequest>,
) -> TestOutput<TestInputRequest, TestOutputRequest> {
    match output {
        TestOutput::Branch {
            operation: TestOutputBranch::Combine,
            children: mut parts,
        } if parts.len() == 1 => parts.remove(0),
        TestOutput::Unary {
            operation: TestOutputUnary::Bulk,
            child,
        } => TestOutput::Unary {
            operation: TestOutputUnary::Bulk,
            child: Box::new(output_body(*child)),
        },
        other => other,
    }
}
impl TestDefaults {
    /// Register typed Input/Output defaults and explicit error-message functions.
    /// Typed trees use Input::for_type / Output::for_type. Production declaration
    /// builders also convert directly into these trees with `.into()`.
    pub fn new<M: prebindgen_flat::Schema>(
        flat: &Flat<M>,
        inputs: Vec<TestInput<TestInputRequest, TestOutputRequest>>,
        outputs: Vec<TestOutput<TestInputRequest, TestOutputRequest>>,
        errors: Vec<(syn::Type, String)>,
    ) -> Result<Self> {
        let mut out = Self {
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            errors: BTreeMap::new(),
        };
        for input in inputs {
            let TestInput::Request(TestInputRequest::ForType { ty, input }) = input else {
                return Err("input defaults require a source type".into());
            };
            let key = type_key(&crate::callback::input_type(flat, &ty)?);
            if out.inputs.insert(key, input_body(*input)).is_some() {
                return Err("duplicate input expansion".into());
            }
        }
        for output in outputs {
            let TestOutput::Request(TestOutputRequest::ForType { ty, output }) = output else {
                return Err("output defaults require a source type".into());
            };
            let key = type_key(&flat.resolve_type(&ty)?);
            if out.outputs.insert(key, output_body(*output)).is_some() {
                return Err("duplicate output expansion".into());
            }
        }
        for (ty, function) in errors {
            let key = type_key(&flat.resolve_type(&ty)?);
            if out.errors.insert(key, function).is_some() {
                return Err("duplicate error mapper for source type".into());
            }
        }
        Ok(out)
    }

    /// Resolve source-dependent operations, compile them, and atomically install one binding.
    /// Existing converters are preserved unless explicitly replaced.
    pub fn fill<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &mut Flat<C::Schema>,
        codec: &C,
        decl: &TestFunction,
        extra: &C::Extra,
    ) -> Result<()> {
        let recipe = self
            .resolve_function(flat, codec, decl, extra)
            .map_err(|e| format!("{}: {e}", decl.name))?;
        codec.compile_declarations(flat, extra)?;
        crate::compile_function(flat, codec, &decl.name, recipe, extra)
            .map_err(|e| format!("{}: {e}", decl.name))
    }
    pub(crate) fn resolve_function<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        decl: &TestFunction,
        extra: &C::Extra,
    ) -> Result<crate::FunctionRecipe<ResolvedInput<C>, ResolvedOutput<C>>> {
        crate::resolve_function(
            flat,
            decl,
            |ty, explicit| {
                self.resolve_input(
                    flat,
                    codec,
                    ty,
                    explicit.unwrap_or(&TestInput::auto()),
                    extra,
                )
            },
            |ty| self.function_output(flat, codec, ty, decl, extra),
        )
    }
    fn function_output<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        decl: &TestFunction,
        extra: &C::Extra,
    ) -> Result<crate::ResolvedOutput<C>> {
        let tree = if let Some((_, error)) = result_parts(ty).filter(|_| !decl.result_value) {
            TestOutput::Unary {
                operation: TestOutputUnary::Result {
                    error: self.error_mapper_for(decl, error).map(str::to_owned),
                },
                child: Box::new(decl.ret.clone().unwrap_or_else(TestOutput::auto)),
            }
        } else {
            decl.ret.clone().unwrap_or_else(TestOutput::auto)
        };
        self.resolve_output(flat, codec, ty, &tree, extra)
    }
    /// Select a function-specific or type-wide explicit error message mapper.
    pub fn error_mapper_for<'a>(
        &'a self,
        decl: &'a TestFunction,
        ty: &syn::Type,
    ) -> Option<&'a str> {
        decl.error
            .as_deref()
            .or_else(|| self.errors.get(&type_key(ty)).map(String::as_str))
    }

    /// Resolve every input operation against canonical source declarations before compilation.
    pub fn resolve_input<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &TestInput<TestInputRequest, TestOutputRequest>,
        extra: &C::Extra,
    ) -> Result<crate::ResolvedInput<C>> {
        self.input(flat, codec, ty, tree, extra, &mut vec![])
    }
    pub(crate) fn input<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &TestInput<TestInputRequest, TestOutputRequest>,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<crate::ResolvedInput<C>> {
        let resolve = |ty: &syn::Type,
                       tree: &TestInput<TestInputRequest, TestOutputRequest>,
                       stack: &mut Vec<String>| {
            self.input(flat, codec, ty, tree, extra, stack)
        };
        match tree {
            TestInput::None => {
                if type_key(ty) == "()" {
                    Ok(InputNode::None)
                } else {
                    Err("None input requires unit".into())
                }
            }
            TestInput::Branch {
                operation: TestInputBranch::Variant,
                ..
            } => Err("variant branch requires an enum parent".into()),
            TestInput::Request(TestInputRequest::EnumFields(entries)) => {
                self.enum_input_recipe(flat, codec, ty, extra, entries, false, stack)
            }
            TestInput::Branch {
                operation: TestInputBranch::Enum { value },
                children: variants,
            } => {
                let variants = crate::resolve_enum_variants(
                    flat,
                    ty,
                    variants,
                    |tree| tree.variant_fields(),
                    |ty, recipe| resolve(ty, recipe, stack),
                )?;
                Ok(TestInput::enum_variants(variants, *value))
            }
            TestInput::Request(TestInputRequest::Value) => {
                self.resolve_input_value(flat, codec, ty, extra, stack)
            }
            TestInput::Request(TestInputRequest::Conversion(decl)) => {
                decl.validate()?;
                let source = decl
                    .input
                    .as_ref()
                    .ok_or("local input conversion has no input source")?;
                let conversion = source.resolve(flat, ty, true)?;
                self.resolve_input_conversion(
                    flat,
                    codec,
                    ty,
                    conversion,
                    &TestInput::value(),
                    extra,
                    stack,
                )
            }
            TestInput::Unary {
                operation: TestInputUnary::Convert { conversion },
                child: input,
            } => self.resolve_input_conversion(
                flat,
                codec,
                ty,
                (**conversion).clone(),
                input,
                extra,
                stack,
            ),
            TestInput::Unary {
                operation: TestInputUnary::Temporary,
                child,
            } => Ok(InputNode::Unary {
                operation: TestInputUnary::Temporary,
                child: Box::new(resolve(
                    &crate::temporary_input_type(ty)?,
                    &input_body((**child).clone()),
                    stack,
                )?),
            }),
            TestInput::Unary {
                operation: TestInputUnary::Bulk,
                child,
            } => Ok(InputNode::Unary {
                operation: TestInputUnary::Bulk,
                child: Box::new(resolve(ty, &input_body((**child).clone()), stack)?),
            }),
            TestInput::Request(TestInputRequest::Auto) => {
                let key = crate::callback::input_key(flat, ty)?;
                if let Some(tree) = crate::resolve_type_default(
                    &key,
                    &self.inputs,
                    stack,
                    crate::Direction::Input,
                    |tree, stack| resolve(ty, tree, stack),
                )? {
                    return Ok(tree);
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(InputNode::Unary {
                        operation: TestInputUnary::Optional,
                        child: Box::new(resolve(inner, &TestInput::auto(), stack)?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(InputNode::Unary {
                        operation: TestInputUnary::Sequence,
                        child: Box::new(resolve(element, &TestInput::auto(), stack)?),
                    });
                }
                if type_key(ty) == "()" {
                    return Ok(InputNode::None);
                }
                resolve(ty, &TestInput::Leaf(TestInputLeafOp::Value), stack)
            }
            TestInput::Request(TestInputRequest::ForType { ty: target, input }) => {
                let target = crate::callback::input_type(flat, target)?;
                let matched = crate::callback::input_key(flat, ty)? == type_key(&target);
                crate::resolve_typed_expansion(
                    ty,
                    matched,
                    &input_body((**input).clone()),
                    tree,
                    stack,
                    resolve,
                    |child| TestInput::Unary {
                        operation: TestInputUnary::Optional,
                        child: Box::new(child),
                    },
                    |child| TestInput::Unary {
                        operation: TestInputUnary::Sequence,
                        child: Box::new(child),
                    },
                    crate::Direction::Input,
                )
            }
            TestInput::Leaf(TestInputLeafOp::Value) => {
                if flat
                    .type_info(ty)
                    .ok()
                    .is_some_and(|i| i.conversion.input.is_none() && i.conversion.output.is_some())
                {
                    return Err(format!(
                        "`{}` has no canonical input conversion",
                        type_key(ty)
                    ));
                }
                if let Some(conversion) = flat
                    .type_info(ty)
                    .ok()
                    .and_then(|i| i.conversion.input.clone())
                {
                    return self.resolve_input_conversion(
                        flat,
                        codec,
                        ty,
                        conversion,
                        &TestInput::value(),
                        extra,
                        stack,
                    );
                }

                if prebindgen_flat::callback_signature(ty)?.is_some() {
                    return resolve(
                        ty,
                        &TestInput::Request(TestInputRequest::Callback(Box::new(
                            crate::TestCallback::new(ty.clone()),
                        ))),
                        stack,
                    );
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(InputNode::Unary {
                        operation: TestInputUnary::Optional,
                        child: Box::new(resolve(
                            inner,
                            &TestInput::Leaf(TestInputLeafOp::Value),
                            stack,
                        )?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(InputNode::Unary {
                        operation: TestInputUnary::Sequence,
                        child: Box::new(resolve(element, &TestInput::auto(), stack)?),
                    });
                }
                Ok(TestInput::Leaf(TestInputLeafOp::Value))
            }
            TestInput::Arrow {
                operation: callback,
                domain,
                codomain,
            } => {
                callback.validate_for(flat, ty)?;
                let signature = prebindgen_flat::callback_signature(ty)?
                    .ok_or("arrow requires a callable source")?;
                let arguments = domain.argument_nodes()?;
                if arguments.len() != signature.arguments.len() {
                    return Err("wrong arrow domain arity".into());
                }
                Ok(InputNode::Arrow {
                    operation: callback.clone(),
                    domain: Box::new(TestOutput::arguments(
                        arguments
                            .iter()
                            .zip(&signature.arguments)
                            .map(|(tree, ty)| {
                                self.output(flat, codec, ty, tree, extra, &mut vec![], true)
                            })
                            .collect::<Result<Vec<_>>>()?,
                    )),
                    codomain: Box::new(resolve(&signature.return_type, codomain, stack)?),
                })
            }
            TestInput::Request(TestInputRequest::Callback(declaration)) => {
                let canonical = crate::callback::input_key(flat, ty)?;
                let default = match self.inputs.get(&canonical) {
                    Some(TestInput::Request(TestInputRequest::Callback(default))) => {
                        Some(default.as_ref())
                    }
                    _ => None,
                };
                let (operation, arguments, ret) = crate::resolve_callback(
                    flat,
                    ty,
                    declaration,
                    default,
                    &self.outputs,
                    &TestInput::auto(),
                    &TestOutput::auto(),
                    stack,
                    resolve,
                    |ty, tree, ownership| {
                        Ok(self
                            .output(flat, codec, ty, tree, extra, &mut vec![], true)?
                            .callback_argument(ownership))
                    },
                )?;
                Ok(InputNode::Arrow {
                    operation: Box::new(operation),
                    domain: Box::new(TestOutput::arguments(arguments)),
                    codomain: Box::new(ret),
                })
            }
            TestInput::Request(TestInputRequest::Call(function)) => {
                let f = flat.function(function)?;
                resolve(
                    ty,
                    &TestInput::Branch {
                        operation: TestInputBranch::Constructor {
                            function: function.clone(),
                        },
                        children: vec![TestInput::auto(); f.params.len()],
                    },
                    stack,
                )
            }
            TestInput::Branch {
                operation: TestInputBranch::Constructor { function },
                children: args,
            } => crate::resolve_constructor(
                flat,
                ty,
                function,
                args,
                |ty, child| resolve(ty, child, stack),
                |function| TestInputBranch::Constructor { function },
            ),
            TestInput::Unary {
                operation: TestInputUnary::Optional,
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(InputNode::Unary {
                    operation: TestInputUnary::Optional,
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            TestInput::Unary {
                operation: TestInputUnary::Sequence,
                child,
            } => {
                let (element, _) =
                    sequence_parts(ty).ok_or("sequence input requires Vec<T>, &[T] or &Vec<T>")?;
                Ok(InputNode::Unary {
                    operation: TestInputUnary::Sequence,
                    child: Box::new(resolve(element, child, stack)?),
                })
            }
            TestInput::Branch {
                operation: TestInputBranch::Fields,
                children,
            }
            | TestInput::Branch {
                operation: TestInputBranch::Record,
                children,
            } => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(InputNode::None);
                }
                let children = crate::resolve_record_fields(flat, ty, children, |ty, child| {
                    resolve(ty, child, stack)
                })?;
                Ok(
                    if matches!(
                        tree,
                        TestInput::Branch {
                            operation: TestInputBranch::Record,
                            children: _
                        }
                    ) {
                        InputNode::Branch {
                            operation: TestInputBranch::Record,
                            children,
                        }
                    } else {
                        InputNode::Branch {
                            operation: TestInputBranch::Fields,
                            children,
                        }
                    },
                )
            }
            TestInput::Branch {
                operation: TestInputBranch::Selector,
                children: arms,
            } => {
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                Ok(InputNode::Branch {
                    operation: TestInputBranch::Selector,
                    children: arms
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
        }
    }
    /// Resolve every output operation, source reference and ownership choice before compilation.
    pub fn resolve_output<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &TestOutput<TestInputRequest, TestOutputRequest>,
        extra: &C::Extra,
    ) -> Result<crate::ResolvedOutput<C>> {
        self.output(flat, codec, ty, tree, extra, &mut vec![], false)
    }
    #[allow(clippy::too_many_arguments)] // Getter defaults differ from the existing plain-reference ABI.
    pub(crate) fn output<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &syn::Type,
        tree: &TestOutput<TestInputRequest, TestOutputRequest>,
        extra: &C::Extra,
        stack: &mut Vec<String>,
        getter_result: bool,
    ) -> Result<crate::ResolvedOutput<C>> {
        let resolve = |ty: &syn::Type,
                       tree: &TestOutput<TestInputRequest, TestOutputRequest>,
                       stack: &mut Vec<String>| {
            self.output(flat, codec, ty, tree, extra, stack, getter_result)
        };
        if let Some(referent) = crate::output_referent(ty) {
            if (getter_result || !matches!(tree, TestOutput::Leaf(TestOutputLeafOp::Value)))
                && !matches!(
                    tree,
                    TestOutput::Request(
                        TestOutputRequest::Auto
                            | TestOutputRequest::ForType { .. }
                            | TestOutputRequest::SequenceCallback { .. }
                    )
                )
            {
                return resolve(referent, tree, stack);
            }
        }
        match tree {
            TestOutput::Request(TestOutputRequest::SequenceCallback { output, structural }) => {
                let _ = structural;
                let _ = resolve(ty, output, stack)?;
                Err("callback return is unsupported by this adapter".into())
            }

            TestOutput::Unary {
                operation: TestOutputUnary::CallbackArgument { ownership },
                child,
            } => Ok((resolve(ty, child, stack)?).callback_argument(*ownership)),
            TestOutput::None => {
                if type_key(ty) == "()" {
                    Ok(OutputNode::None)
                } else {
                    Err("None output requires unit".into())
                }
            }
            TestOutput::Arrow { operation, .. } => match *operation {},
            TestOutput::Branch {
                operation: TestOutputBranch::Variant | TestOutputBranch::Arguments,
                ..
            } => Err("variant/argument branch requires its parent operation".into()),
            TestOutput::Request(TestOutputRequest::EnumFields(entries)) => self.enum_output_recipe(
                flat,
                codec,
                crate::output_referent(ty).unwrap_or(ty),
                extra,
                entries,
                false,
                stack,
                getter_result,
            ),
            TestOutput::Branch {
                operation: TestOutputBranch::Enum { value, mode },
                children: variants,
            } => {
                let variants = crate::resolve_enum_variants(
                    flat,
                    ty,
                    variants,
                    |tree| tree.variant_fields(),
                    |ty, recipe| resolve(ty, recipe, stack),
                )?;
                Ok(TestOutput::enum_variants(variants, *value, *mode))
            }
            TestOutput::Request(TestOutputRequest::Value { projection }) => self
                .resolve_output_value(
                    flat,
                    codec,
                    ty,
                    extra,
                    stack,
                    getter_result,
                    projection.as_ref(),
                ),
            TestOutput::Request(TestOutputRequest::Conversion(decl)) => {
                decl.validate()?;
                let source = decl
                    .output
                    .as_ref()
                    .ok_or("local output conversion has no output source")?;
                let conversion = source.resolve(flat, ty, false)?;
                self.resolve_output_conversion(
                    flat,
                    codec,
                    ty,
                    conversion,
                    &TestOutput::value(),
                    extra,
                    stack,
                )
            }
            TestOutput::Unary {
                operation: TestOutputUnary::Convert { conversion },
                child: output,
            } => self.resolve_output_conversion(
                flat,
                codec,
                ty,
                (**conversion).clone(),
                output,
                extra,
                stack,
            ),
            TestOutput::Unary {
                operation: TestOutputUnary::Bulk,
                child,
            } => Ok(OutputNode::Unary {
                operation: TestOutputUnary::Bulk,
                child: Box::new(resolve(ty, &output_body((**child).clone()), stack)?),
            }),
            TestOutput::Leaf(TestOutputLeafOp::Discard) => {
                Ok(OutputNode::Leaf(TestOutputLeafOp::Discard))
            }
            TestOutput::Unary {
                operation: TestOutputUnary::Named { name },
                child: output,
            } => Ok(OutputNode::Unary {
                operation: TestOutputUnary::Named { name: name.clone() },
                child: Box::new(resolve(ty, output, stack)?),
            }),
            TestOutput::Request(TestOutputRequest::Auto) => {
                let key = type_key(ty);
                if let Some(tree) = crate::resolve_type_default(
                    &key,
                    &self.outputs,
                    stack,
                    crate::Direction::Output,
                    |tree, stack| resolve(ty, tree, stack),
                )? {
                    return Ok(tree);
                }
                if getter_result {
                    if let Some(referent) = crate::output_referent(ty) {
                        return resolve(referent, tree, stack);
                    }
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(OutputNode::Unary {
                        operation: TestOutputUnary::Optional,
                        child: Box::new(resolve(inner, &TestOutput::auto(), stack)?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(OutputNode::Unary {
                        operation: TestOutputUnary::Sequence {
                            delivery: SequenceDelivery::Auto,
                            mode: None,
                        },
                        child: Box::new(resolve(element, &TestOutput::auto(), stack)?),
                    });
                }
                if type_key(ty) == "()" {
                    return Ok(OutputNode::None);
                }
                resolve(ty, &TestOutput::Leaf(TestOutputLeafOp::Value), stack)
            }
            TestOutput::Request(TestOutputRequest::ForType { ty: target, output }) => {
                let target = flat.resolve_type(target)?;
                let matched = same_type(ty, &target);
                crate::resolve_typed_expansion(
                    ty,
                    matched,
                    &output_body((**output).clone()),
                    tree,
                    stack,
                    resolve,
                    |child| {
                        if matches!(&child, TestOutput::Leaf(TestOutputLeafOp::Discard)) {
                            child
                        } else {
                            TestOutput::Unary {
                                operation: TestOutputUnary::Optional,
                                child: Box::new(child),
                            }
                        }
                    },
                    |child| {
                        if matches!(&child, TestOutput::Leaf(TestOutputLeafOp::Discard)) {
                            child
                        } else {
                            TestOutput::Unary {
                                operation: TestOutputUnary::Sequence {
                                    delivery: SequenceDelivery::Auto,
                                    mode: None,
                                },
                                child: Box::new(child),
                            }
                        }
                    },
                    crate::Direction::Output,
                )
            }
            TestOutput::Leaf(TestOutputLeafOp::Value)
            | TestOutput::Leaf(TestOutputLeafOp::Projected(_)) => {
                let conversion_type = crate::output_referent(ty).unwrap_or(ty);
                if flat
                    .type_info(conversion_type)
                    .ok()
                    .is_some_and(|i| i.conversion.output.is_none() && i.conversion.input.is_some())
                {
                    return Err(format!(
                        "`{}` has no canonical output conversion",
                        type_key(ty)
                    ));
                }
                let converted = flat
                    .type_info(ty)
                    .ok()
                    .and_then(|i| i.conversion.output.clone())
                    .or_else(|| {
                        crate::output_referent(ty)
                            .and_then(|t| flat.type_info(t).ok())
                            .and_then(|i| i.conversion.output.clone())
                    });
                if let Some(conversion) = converted {
                    if let TestOutput::Leaf(TestOutputLeafOp::Projected(policy)) = tree {
                        return self.resolve_output_value(
                            flat,
                            codec,
                            ty,
                            extra,
                            stack,
                            getter_result,
                            Some(policy),
                        );
                    }
                    return self.resolve_output_conversion(
                        flat,
                        codec,
                        ty,
                        conversion,
                        &TestOutput::value(),
                        extra,
                        stack,
                    );
                }

                if let Some(inner) = option_inner(ty) {
                    return Ok(OutputNode::Unary {
                        operation: TestOutputUnary::Optional,
                        child: Box::new(resolve(inner, tree, stack)?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    let mode = if let TestOutput::Leaf(TestOutputLeafOp::Projected(p)) = tree {
                        if !p.fields.is_empty() {
                            return Err(
                                "collection field projections belong on the element type".into()
                            );
                        }
                        Some(p.mode)
                    } else {
                        None
                    };
                    return Ok(OutputNode::Unary {
                        operation: TestOutputUnary::Sequence {
                            delivery: SequenceDelivery::Value,
                            mode,
                        },
                        child: Box::new(resolve(element, &TestOutput::auto(), stack)?),
                    });
                }
                Ok(match tree {
                    TestOutput::Leaf(TestOutputLeafOp::Value) => {
                        TestOutput::Leaf(TestOutputLeafOp::Value)
                    }
                    TestOutput::Leaf(TestOutputLeafOp::Projected(p)) => {
                        TestOutput::Leaf(TestOutputLeafOp::Projected(p.clone()))
                    }
                    _ => unreachable!(),
                })
            }
            TestOutput::Unary {
                operation: TestOutputUnary::Optional,
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(OutputNode::Unary {
                    operation: TestOutputUnary::Optional,
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            TestOutput::Unary {
                operation: TestOutputUnary::Sequence { delivery, mode },
                child: element,
            } => {
                let (ty, _) =
                    sequence_parts(ty).ok_or("sequence output requires a collection source")?;
                Ok(OutputNode::Unary {
                    operation: TestOutputUnary::Sequence {
                        delivery: *delivery,
                        mode: *mode,
                    },
                    child: Box::new(resolve(ty, element, stack)?),
                })
            }
            TestOutput::Request(TestOutputRequest::NamedFields { overrides, mode }) => {
                let children = crate::resolve_named_fields(
                    flat,
                    ty,
                    overrides,
                    &TestOutput::auto(),
                    |ty, tree| resolve(ty, tree, stack),
                )?;
                Ok(if *mode == ProjectionMode::Owned {
                    OutputNode::Branch {
                        operation: TestOutputBranch::OwnedFields,
                        children,
                    }
                } else {
                    OutputNode::Branch {
                        operation: TestOutputBranch::Fields,
                        children,
                    }
                })
            }
            TestOutput::Branch {
                operation: TestOutputBranch::Fields,
                children,
            }
            | TestOutput::Branch {
                operation: TestOutputBranch::OwnedFields,
                children,
            }
            | TestOutput::Branch {
                operation: TestOutputBranch::Record { .. },
                children,
                ..
            } => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(OutputNode::None);
                }
                let children = crate::resolve_record_fields(flat, ty, children, |ty, child| {
                    resolve(ty, child, stack)
                })?;
                Ok(match tree {
                    TestOutput::Branch {
                        operation: TestOutputBranch::Fields,
                        children: _,
                    } => OutputNode::Branch {
                        operation: TestOutputBranch::Fields,
                        children,
                    },
                    TestOutput::Branch {
                        operation: TestOutputBranch::OwnedFields,
                        children: _,
                    } => OutputNode::Branch {
                        operation: TestOutputBranch::OwnedFields,
                        children,
                    },
                    TestOutput::Branch {
                        operation: TestOutputBranch::Record { mode, .. },
                        ..
                    } => OutputNode::Branch {
                        operation: TestOutputBranch::Record { mode: *mode },
                        children,
                    },
                    _ => unreachable!(),
                })
            }
            TestOutput::Unary {
                operation: TestOutputUnary::Getter { function },
                child: output,
            }
            | TestOutput::Unary {
                operation: TestOutputUnary::Into { function },
                child: output,
            } => {
                let output = Box::new(crate::resolve_getter(
                    flat,
                    ty,
                    function,
                    matches!(
                        tree,
                        TestOutput::Unary {
                            operation: TestOutputUnary::Into { .. },
                            ..
                        }
                    ),
                    output,
                    |ty, output| self.output(flat, codec, ty, output, extra, stack, true),
                )?);
                Ok(
                    if matches!(
                        tree,
                        TestOutput::Unary {
                            operation: TestOutputUnary::Into { .. },
                            ..
                        }
                    ) {
                        OutputNode::Unary {
                            operation: TestOutputUnary::Into {
                                function: function.clone(),
                            },
                            child: output,
                        }
                    } else {
                        OutputNode::Unary {
                            operation: TestOutputUnary::Getter {
                                function: function.clone(),
                            },
                            child: output,
                        }
                    },
                )
            }
            TestOutput::Branch {
                operation: TestOutputBranch::Combine,
                children: parts,
            } => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                Ok(OutputNode::Branch {
                    operation: TestOutputBranch::Combine,
                    children: parts
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
            TestOutput::Unary {
                operation: TestOutputUnary::Result { error },
                child: ok,
            } => {
                let (success, failure) =
                    result_parts(ty).ok_or("Result recipe requires Result<T, E>")?;
                if let Some(name) = error {
                    let f = flat.function(name)?;
                    let valid = f.params.len() == 1
                        && (same_type(&f.params[0].ty.ty, failure)
                            || matches!(&f.params[0].ty.ty,syn::Type::Reference(r) if r.mutability.is_none()&&same_type(&r.elem,failure)));
                    if !valid || type_key(&f.ret.ty) != "String" {
                        return Err(
                            "Result error mapper must take exactly E or &E and return String"
                                .into(),
                        );
                    }
                }
                Ok(OutputNode::Unary {
                    operation: TestOutputUnary::Result {
                        error: error.clone(),
                    },
                    child: Box::new(resolve(success, ok, stack)?),
                })
            }
        }
    }
}

impl TestDefaults {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_input_conversion<
        C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>,
    >(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        conversion: ValueConversion,
        child: &TestInput<TestInputRequest, TestOutputRequest>,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<crate::ResolvedInput<C>> {
        let child = crate::resolve_conversion(
            ty,
            &conversion,
            child,
            stack,
            crate::Direction::Input,
            |ty, child, stack| self.input(flat, codec, ty, child, extra, stack),
        )?;
        Ok(TestInput::Unary {
            operation: TestInputUnary::Convert {
                conversion: Box::new(conversion),
            },
            child: Box::new(child),
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_output_conversion<
        C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>,
    >(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        conversion: ValueConversion,
        child: &TestOutput<TestInputRequest, TestOutputRequest>,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<crate::ResolvedOutput<C>> {
        let child = crate::resolve_conversion(
            ty,
            &conversion,
            child,
            stack,
            crate::Direction::Output,
            |ty, child, stack| self.output(flat, codec, ty, child, extra, stack, true),
        )?;
        Ok(TestOutput::Unary {
            operation: TestOutputUnary::Convert {
                conversion: Box::new(conversion),
            },
            child: Box::new(child),
        })
    }
    pub(crate) fn resolve_input_value<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        extra: &C::Extra,
        stack: &mut Vec<String>,
    ) -> Result<crate::ResolvedInput<C>> {
        if type_key(ty) == "()" {
            return Ok(InputNode::Branch {
                operation: TestInputBranch::Fields,
                children: vec![],
            });
        }
        if let Some(c) = flat
            .type_info(ty)
            .ok()
            .and_then(|i| i.conversion.input.clone())
        {
            return self.resolve_input_conversion(
                flat,
                codec,
                ty,
                c,
                &TestInput::value(),
                extra,
                stack,
            );
        }
        if let Some(inner) = prebindgen_flat::option_inner(ty) {
            return Ok(InputNode::Unary {
                operation: TestInputUnary::Optional,
                child: Box::new(self.resolve_input_value(flat, codec, inner, extra, stack)?),
            });
        }
        if let Some((inner, _)) = prebindgen_flat::sequence_parts(ty) {
            return Ok(InputNode::Unary {
                operation: TestInputUnary::Sequence,
                child: Box::new(self.resolve_input_value(flat, codec, inner, extra, stack)?),
            });
        }
        self.input(
            flat,
            codec,
            ty,
            &TestInput::Leaf(TestInputLeafOp::Value),
            extra,
            stack,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_output_value<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        extra: &C::Extra,
        stack: &mut Vec<String>,
        getter: bool,
        policy: Option<&crate::Projection>,
    ) -> Result<crate::ResolvedOutput<C>> {
        if type_key(ty) == "()" {
            return Ok(OutputNode::Branch {
                operation: TestOutputBranch::Fields,
                children: vec![],
            });
        }
        if let Some(c) = crate::output_value_conversion(flat, ty, policy)? {
            return self.resolve_output_conversion(
                flat,
                codec,
                ty,
                c,
                &TestOutput::value(),
                extra,
                stack,
            );
        }
        if let Some(inner) = prebindgen_flat::option_inner(ty) {
            return Ok(OutputNode::Unary {
                operation: TestOutputUnary::Optional,
                child: Box::new(
                    self.resolve_output_value(flat, codec, inner, extra, stack, getter, policy)?,
                ),
            });
        }
        if let Some((inner, _)) = prebindgen_flat::sequence_parts(ty) {
            return Ok(OutputNode::Unary {
                operation: TestOutputUnary::Sequence {
                    delivery: crate::SequenceDelivery::Value,
                    mode: policy.map(|p| p.mode),
                },
                child: Box::new(
                    self.resolve_output_value(flat, codec, inner, extra, stack, getter, policy)?,
                ),
            });
        }
        self.output(
            flat,
            codec,
            ty,
            &policy.map_or(TestOutput::Leaf(TestOutputLeafOp::Value), |p| {
                TestOutput::Leaf(TestOutputLeafOp::Projected(p.clone()))
            }),
            extra,
            stack,
            getter,
        )
    }
}
impl TestDefaults {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_input_recipe<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        extra: &C::Extra,
        entries: &BTreeMap<
            String,
            BTreeMap<String, TestInput<TestInputRequest, TestOutputRequest>>,
        >,
        value: bool,
        stack: &mut Vec<String>,
    ) -> Result<crate::ResolvedInput<C>> {
        let default = if value {
            TestInput::value()
        } else {
            TestInput::auto()
        };
        let variants = crate::resolve_enum_fields(flat, ty, entries, &default, |ty, recipe| {
            self.input(flat, codec, ty, recipe, extra, stack)
        })?;
        Ok(TestInput::enum_variants(variants, value))
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn enum_output_recipe<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: &Flat<C::Schema>,
        codec: &C,
        ty: &Type,
        extra: &C::Extra,
        entries: &BTreeMap<
            String,
            BTreeMap<String, TestOutput<TestInputRequest, TestOutputRequest>>,
        >,
        value: bool,
        stack: &mut Vec<String>,
        getter: bool,
    ) -> Result<crate::ResolvedOutput<C>> {
        let default = if value {
            TestOutput::value()
        } else {
            TestOutput::auto()
        };
        let variants = crate::resolve_enum_fields(flat, ty, entries, &default, |ty, recipe| {
            self.output(flat, codec, ty, recipe, extra, stack, getter)
        })?;
        Ok(TestOutput::enum_variants(variants, value, None))
    }
}
impl TestDefaults {
    /// Resolve function occurrences into an independently compilable Flat.
    ///
    /// Defaults and local overrides are consumed here. The returned model owns
    /// its compiler policy and occurrence settings; drop this TestDefaults value
    /// before calling [`compile_resolved`] if desired.
    pub fn resolve<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
        &self,
        flat: Flat<C::Schema>,
        codec: C,
        declarations: &[TestFunction],
        extra: C::Extra,
    ) -> Result<Flat<ResolvedSchema<C>>> {
        crate::resolve_bindings(
            flat,
            codec,
            declarations,
            |flat, codec, decl, extra| self.resolve_function(flat, codec, decl, extra),
            extra,
        )
    }
}

impl From<DeclConvert> for TestInputRequest {
    fn from(decl: DeclConvert) -> Self {
        Self::ForType {
            ty: Box::new(decl.ty.clone()),
            input: Box::new(TestInput::Request(Self::Conversion(Box::new(decl)))),
        }
    }
}

impl From<DeclConvert> for TestOutputRequest {
    fn from(decl: DeclConvert) -> Self {
        Self::ForType {
            ty: Box::new(decl.ty.clone()),
            output: Box::new(TestOutput::Request(Self::Conversion(Box::new(decl)))),
        }
    }
}
pub fn test_bind_input<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
    flat: &mut Flat<C::Schema>,
    codec: &C,
    function: &str,
    parameter: &str,
    recipe: &TestInput<TestInputRequest, TestOutputRequest>,
    extra: &C::Extra,
) -> Result<()> {
    let f = flat.function(function)?;
    let index = f
        .params
        .iter()
        .position(|p| p.name == parameter)
        .ok_or_else(|| format!("`{function}`: unknown parameter `{parameter}`"))?;
    let defaults = TestDefaults::new(flat, vec![], vec![], vec![])?;
    let recipe = defaults.resolve_input(flat, codec, &f.params[index].ty.ty, recipe, extra)?;
    let conversion = compile_input(flat, codec, function, parameter, &recipe, extra)?;
    flat.function_mut(function)?.params[index].ty.slot = Some(conversion);
    Ok(())
}

/// Build and attach a converter to a function result, replacing any existing one.
///
/// Uses the same construction and failure rules as [`bind_input`]. Output code
/// uses [`prebindgen_conversions::source_result_expr`] for the source call's result local.
/// The recipe is checked for compatible types and signatures, and for repeated
/// consumption of the same value. Unit results use `TestOutput::fields([])`.
///
/// # Arguments
///
/// * `flat` — the model holding the function; the result's slot is replaced.
/// * `codec` — the adapter's leaf conversions.
/// * `function` — unqualified name of the source function.
/// * `recipe` — preparation tree describing how to deliver the result as boundary outputs.
///   Adapter-resolved trees go directly to [`compile_output`].
/// * `extra` — occurrence data passed to every codec call for this conversion.
///
/// Output binding rejects input recipes:
/// ```compile_fail
/// use prebindgen_declarations::{bind_output, TestResolvePolicy, TestInput};
/// use prebindgen_flat::Flat;
/// fn wrong<C: Codec<InOps=TestInputOps,OutOps=TestOutputOps>>(flat: &mut Flat<C::Schema>, codec: &C, extra: &C::Extra) {
///     bind_output(flat, codec, "foo", &TestInput::value(), extra).unwrap();
/// }
/// ```
pub fn test_bind_output<C: Codec<InOps = TestInputOps, OutOps = TestOutputOps>>(
    flat: &mut Flat<C::Schema>,
    codec: &C,
    function: &str,
    recipe: &TestOutput<TestInputRequest, TestOutputRequest>,
    extra: &C::Extra,
) -> Result<()> {
    let defaults = TestDefaults::new(flat, vec![], vec![], vec![])?;
    let recipe =
        defaults.resolve_output(flat, codec, &flat.function(function)?.ret.ty, recipe, extra)?;
    let conversion = compile_output(flat, codec, function, &recipe, extra)?;
    flat.function_mut(function)?.ret.slot = Some(conversion);
    Ok(())
}

#[macro_export]
macro_rules! test_fun {
    ($name:ident) => {
        $crate::test_language::TestFunction::new($crate::ident(stringify!($name)).unwrap())
    };
}
#[macro_export]
macro_rules! test_expand_param {($($ty:tt)+)=>{$crate::test_language::TestExpandParam::new($crate::declarations::parse_type(stringify!($($ty)+)))};}
#[macro_export]
macro_rules! test_expand_return {($($ty:tt)+)=>{$crate::test_language::TestExpandReturn::new($crate::declarations::parse_type(stringify!($($ty)+)))};}
#[macro_export]
macro_rules! test_fields {
    ($name:ident) => {
        $crate::test_language::TestFields::new($crate::ident(stringify!($name)).unwrap())
    };
}
#[macro_export]
macro_rules! test_callback {($($ty:tt)+)=>{$crate::test_language::TestCallback::new($crate::declarations::parse_type(stringify!($($ty)+)))};}

/// Collect source helper and representation-type names needed by `Flat::parse_selected`.
pub fn source_items<'a>(
    inputs: &[TestInput<TestInputRequest, TestOutputRequest>],
    outputs: &[TestOutput<TestInputRequest, TestOutputRequest>],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a TestFunction>,
) -> Vec<String> {
    fn input(tree: &TestInput<TestInputRequest, TestOutputRequest>, names: &mut Vec<String>) {
        match tree {
            TestInput::Request(TestInputRequest::EnumFields(variants)) => {
                for child in variants.values().flat_map(|v| v.values()) {
                    input(child, names)
                }
            }
            TestInput::Request(TestInputRequest::Conversion(d)) => {
                crate::conversion_source_items(d, names)
            }
            TestInput::Unary {
                operation: TestInputUnary::Convert { conversion },
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            TestInput::Request(TestInputRequest::Callback(declaration)) => {
                for tree in declaration.params.values() {
                    output(tree, names)
                }
                if let Some(tree) = &declaration.ret {
                    input(tree, names)
                }
            }
            TestInput::Request(TestInputRequest::Call(name)) => names.push(name.clone()),
            TestInput::Branch {
                operation: TestInputBranch::Constructor { function },
                ..
            } => names.push(function.clone()),
            TestInput::Request(TestInputRequest::ForType { input: child, .. }) => {
                input(child, names)
            }
            _ => {}
        }
        crate::input_children(tree, names, &mut input, &mut output);
    }
    fn output(tree: &TestOutput<TestInputRequest, TestOutputRequest>, names: &mut Vec<String>) {
        match tree {
            TestOutput::Request(TestOutputRequest::SequenceCallback { output: child, .. })
            | TestOutput::Request(TestOutputRequest::ForType { output: child, .. }) => {
                output(child, names)
            }
            TestOutput::Request(TestOutputRequest::EnumFields(variants)) => {
                for child in variants.values().flat_map(|v| v.values()) {
                    output(child, names)
                }
            }
            TestOutput::Request(TestOutputRequest::Conversion(d)) => {
                crate::conversion_source_items(d, names)
            }
            TestOutput::Unary {
                operation: TestOutputUnary::Convert { conversion },
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            TestOutput::Unary {
                operation: TestOutputUnary::Getter { function },
                ..
            }
            | TestOutput::Unary {
                operation: TestOutputUnary::Into { function },
                ..
            } => names.push(function.clone()),
            TestOutput::Unary {
                operation: TestOutputUnary::Result { error: Some(name) },
                ..
            } => names.push(name.clone()),
            TestOutput::Request(TestOutputRequest::NamedFields { overrides, .. }) => {
                for child in overrides.values() {
                    output(child, names)
                }
            }
            _ => {}
        }
        crate::output_children(tree, names, &mut input, &mut output);
    }
    crate::collect_source_items(inputs, outputs, errors, functions, &mut input, &mut output)
}
