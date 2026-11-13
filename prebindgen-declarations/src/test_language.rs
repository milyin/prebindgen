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
                operation: (TestOutputBranch::Arguments).into(),
                children,
            }
        }
    }
    /// Read the source-ordered components of a callable domain.
    pub fn argument_nodes(&self) -> crate::Result<&[Self]> {
        match self {
            Self::None => Ok(&[]),
            Self::Branch {
                operation: crate::Operation::Resolved(TestOutputBranch::Arguments),
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
            operation: (TestInputBranch::Enum { value }).into(),
            children: variants
                .into_iter()
                .map(|children| Self::Branch {
                    operation: (TestInputBranch::Variant).into(),
                    children,
                })
                .collect(),
        }
    }
    /// Read an enum variant's field conversions.
    pub fn variant_fields(&self) -> crate::Result<&[Self]> {
        match self {
            Self::Branch {
                operation: crate::Operation::Resolved(TestInputBranch::Variant),
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
            operation: (TestOutputBranch::Enum { value, mode }).into(),
            children: variants
                .into_iter()
                .map(|children| Self::Branch {
                    operation: (TestOutputBranch::Variant).into(),
                    children,
                })
                .collect(),
        }
    }
    /// Read an enum variant's field conversions.
    pub fn variant_fields(&self) -> crate::Result<&[Self]> {
        match self {
            Self::Branch {
                operation: crate::Operation::Resolved(TestOutputBranch::Variant),
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
            operation: (TestInputUnary::Sequence).into(),
            child: Box::new(element),
        }
    }
}

impl<InRequest, OutRequest> TestOutput<InRequest, OutRequest> {
    /// Attach the lifetime of produced foreign owners to this callable argument.
    pub fn callback_argument(self, ownership: crate::CallbackArgumentOwnership) -> Self {
        Self::Unary {
            operation: (TestOutputUnary::CallbackArgument { ownership }).into(),
            child: Box::new(self),
        }
    }
    /// Read an argument's conversion and ownership contract from its own subtree.
    pub fn callback_argument_parts(
        &self,
    ) -> crate::Result<(crate::CallbackArgumentOwnership, &Self)> {
        match self {
            Self::Unary {
                operation:
                    crate::Operation::Resolved(TestOutputUnary::CallbackArgument { ownership }),
                child,
            } => Ok((*ownership, child)),
            _ => Err("callback argument requires an explicit ownership contract".into()),
        }
    }
}

/// Source-dependent choices accepted only while preparing an input tree.
#[derive(Clone, Debug)]
/// Metadata-only input choices. Explicit conversions use structural node children.
pub enum TestInputRequest {
    /// Partial named enum fields; children are variant branches.
    EnumFields,
    /// One partial variant; the children correspond to these source field names.
    VariantFields {
        variant: String,
        fields: Vec<String>,
    },
    /// Resolve the canonical value representation.
    Value,
    /// Select a local Rust value conversion.
    Conversion(Box<DeclConvert>),
    /// Resolve this occurrence's configured default.
    Auto,
    /// Partial callback metadata; domain/codomain own all conversion children.
    Callback(Box<crate::CallbackRequest>),
    /// Apply the unary child at this source type, including nested occurrences.
    ForType { ty: Box<Type> },
    /// Infer constructor arguments from their defaults.
    Call(String),
}

impl TestInput<TestInputRequest, TestOutputRequest> {
    /// Expand the tag and selected fields. Keys are source variant and field
    /// names (or tuple indices). Unspecified fields inherit boundary defaults.
    pub fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self {
        let overrides = crate::enum_overrides(overrides);
        TestInput::Branch {
            operation: crate::Operation::Request(TestInputRequest::EnumFields),
            children: overrides
                .into_iter()
                .map(|(variant, fields)| {
                    let (fields, children) = fields.into_iter().unzip();
                    TestInput::Branch {
                        operation: crate::Operation::Request(TestInputRequest::VariantFields {
                            variant,
                            fields,
                        }),
                        children,
                    }
                })
                .collect(),
        }
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
        TestInput::Unary {
            operation: crate::Operation::Request(TestInputRequest::ForType { ty: Box::new(ty) }),
            child: Box::new(input),
        }
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
            TestInput::Unary {
                operation: crate::Operation::Request(TestInputRequest::ForType { ty }),
                child: input,
            } => TestInput::for_type(
                *ty,
                TestInput::Unary {
                    operation: (TestInputUnary::Temporary).into(),
                    child: input,
                },
            ),
            other => TestInput::Unary {
                operation: (TestInputUnary::Temporary).into(),
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
            TestInput::Unary {
                operation: crate::Operation::Request(TestInputRequest::ForType { ty }),
                child: input,
            } => TestInput::for_type(*ty, input.add_alternative(arm)),
            TestInput::Branch {
                operation: crate::Operation::Resolved(TestInputBranch::Selector),
                children: mut arms,
            } => {
                arms.push(arm);
                TestInput::Branch {
                    operation: (TestInputBranch::Selector).into(),
                    children: arms,
                }
            }
            other => TestInput::Branch {
                operation: (TestInputBranch::Selector).into(),
                children: vec![other, arm],
            },
        }
    }
}
impl<InRequest, OutRequest> TestInput<InRequest, OutRequest> {
    /// Group this explicit expansion into a bulk carrier.
    pub fn bulk(self) -> Self {
        TestInput::Unary {
            operation: (TestInputUnary::Bulk).into(),
            child: Box::new(self),
        }
    }
    /// Construct a record from field recipes, or unit from an empty list.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration
    /// order. Binding verifies the count against the record.
    pub fn fields(fields: impl IntoIterator<Item = TestInput<InRequest, OutRequest>>) -> Self {
        TestInput::Branch {
            operation: (TestInputBranch::Fields).into(),
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
            operation: (TestInputBranch::Constructor {
                function: name.into(),
            })
            .into(),
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
            operation: (TestInputBranch::Selector).into(),
            children: arms.into_iter().collect(),
        }
    }
}
/// Source-dependent choices accepted only while preparing an output tree.
#[derive(Clone, Debug)]
/// Metadata-only output choices. Explicit conversions use structural node children.
pub enum TestOutputRequest {
    /// Select per-element delivery of the unary child's resolved sequence.
    SequenceCallback { structural: bool },
    /// Partial named enum fields; children are variant branches.
    EnumFields,
    /// One partial variant; children correspond to these source field names.
    VariantFields {
        variant: String,
        fields: Vec<String>,
    },
    /// Resolve the canonical value representation with optional projection policy.
    Value { projection: Option<Projection> },
    /// Select a local Rust value conversion.
    Conversion(Box<DeclConvert>),
    /// Resolve this occurrence's configured default.
    Auto,
    /// Apply the unary child at this source type, including nested occurrences.
    ForType { ty: Box<Type> },
    /// Partial record fields, corresponding to the branch children.
    NamedFields {
        fields: Vec<String>,
        mode: ProjectionMode,
    },
}

impl TestOutput<TestInputRequest, TestOutputRequest> {
    /// Expand an enum's tag and active fields using source variant/member keys.
    pub fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self {
        let overrides = crate::enum_overrides(overrides);
        TestOutput::Branch {
            operation: crate::Operation::Request(TestOutputRequest::EnumFields),
            children: overrides
                .into_iter()
                .map(|(variant, fields)| {
                    let (fields, children) = fields.into_iter().unzip();
                    TestOutput::Branch {
                        operation: crate::Operation::Request(TestOutputRequest::VariantFields {
                            variant,
                            fields,
                        }),
                        children,
                    }
                })
                .collect(),
        }
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
        TestOutput::Unary {
            operation: crate::Operation::Request(TestOutputRequest::ForType { ty: Box::new(ty) }),
            child: Box::new(output),
        }
    }
    /// Append another simultaneous output view.
    pub fn view(self, view: impl Into<Self>) -> Self {
        let view = view.into();
        match self {
            TestOutput::Unary {
                operation: crate::Operation::Request(TestOutputRequest::ForType { ty }),
                child: output,
            } => TestOutput::for_type(*ty, output.view(view)),
            TestOutput::Branch {
                operation: crate::Operation::Resolved(TestOutputBranch::Combine),
                children: mut parts,
            } => {
                parts.push(view);
                TestOutput::Branch {
                    operation: (TestOutputBranch::Combine).into(),
                    children: parts,
                }
            }
            other => TestOutput::Branch {
                operation: (TestOutputBranch::Combine).into(),
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
        self.view(owned_field_view(view))
    }
    /// Deprecated spelling of [`TestOutput::fields_into`].
    #[deprecated(note = "use fields_into instead")]
    pub fn fields_self_into(self, view: Self) -> Self {
        self.fields_into(view)
    }
    /// Override one named source field, preserving defaults for its siblings.
    pub fn field_override(self, name: impl Into<String>, output: Self) -> Self {
        crate::override_output_field(
            self,
            name.into(),
            output,
            |tree| match tree {
                TestOutput::Branch {
                    operation:
                        crate::Operation::Request(TestOutputRequest::NamedFields { fields, .. }),
                    children,
                } => Some((fields, children)),
                _ => None,
            },
            |tree| match tree {
                TestOutput::Unary {
                    operation:
                        crate::Operation::Resolved(
                            TestOutputUnary::Getter { .. } | TestOutputUnary::Into { .. },
                        )
                        | crate::Operation::Request(TestOutputRequest::ForType { .. }),
                    child,
                } => Some(child),
                _ => None,
            },
        )
    }
}
impl<InRequest, OutRequest> TestOutput<InRequest, OutRequest> {
    /// Group this explicit expansion into a bulk carrier.
    pub fn bulk(self) -> Self {
        TestOutput::Unary {
            operation: (TestOutputUnary::Bulk).into(),
            child: Box::new(self),
        }
    }
    /// Observe record fields in declaration order; use an empty list for unit.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration order.
    pub fn fields(fields: impl IntoIterator<Item = TestOutput<InRequest, OutRequest>>) -> Self {
        TestOutput::Branch {
            operation: (TestOutputBranch::Fields).into(),
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
            operation: (TestOutputUnary::Getter {
                function: name.into(),
            })
            .into(),
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
            operation: (TestOutputBranch::Combine).into(),
            children: parts.into_iter().collect(),
        }
    }
}

/// Input expansion declaration specialized for this adapter.
pub type TestExpandParam = crate::DeclExpandParam<
    TestInput<TestInputRequest, TestOutputRequest>,
    TestFunction,
    TestExpansionOptions,
>;
/// Return expansion declaration specialized for this adapter.
pub type TestExpandReturn = crate::DeclExpandReturn<
    TestOutput<TestInputRequest, TestOutputRequest>,
    TestFunction,
    TestExpansionOptions,
>;
/// Named-field declaration specialized for this adapter.
pub type TestFields = crate::DeclFields<TestOutput<TestInputRequest, TestOutputRequest>>;
/// Adapter-specific options applied after the common expansion is built.
#[derive(Clone, Debug, Default)]
pub struct TestExpansionOptions {
    bulk: bool,
}
/// Select this independent adapter's bulk carrier for an expansion.
pub trait TestBulkExpansion: Sized {
    /// Group the expansion in a bulk C-representation carrier.
    fn bulk(self) -> Self;
}
impl TestBulkExpansion for TestExpandParam {
    fn bulk(mut self) -> Self {
        self.extension.get_or_insert_with(Default::default).bulk = true;
        self
    }
}
impl TestBulkExpansion for TestExpandReturn {
    fn bulk(mut self) -> Self {
        self.extension.get_or_insert_with(Default::default).bulk = true;
        self
    }
}
/// Start input expansion using this adapter's node constructors.
pub fn expand_param(ty: Type) -> TestExpandParam {
    crate::DeclExpandParam::new(
        ty,
        |function: TestFunction| TestInput::call(function.reference_name(false)),
        || TestInput::Leaf(TestInputLeafOp::Value),
        |ty, alternatives, extension| {
            let mut input = alternatives.build(TestInput::selector, TestInput::auto, |child| {
                TestInput::Unary {
                    operation: (TestInputUnary::Temporary).into(),
                    child: Box::new(child),
                }
            });
            if extension.is_some_and(|options| options.bulk) {
                input = TestInput::Unary {
                    operation: (TestInputUnary::Bulk).into(),
                    child: Box::new(input),
                };
            }
            TestInput::for_type(ty, input)
        },
    )
}
/// Start return expansion using this adapter's node constructors.
pub fn expand_return(ty: Type) -> TestExpandReturn {
    crate::DeclExpandReturn::new(
        ty,
        |function| {
            crate::getter_view(
                function,
                |name| TestOutput::getter(name, TestOutput::auto()),
                |name, child| TestOutput::Unary {
                    operation: (TestOutputUnary::Named { name }).into(),
                    child: Box::new(child),
                },
            )
        },
        || TestOutput::Leaf(TestOutputLeafOp::Value),
        |policy| TestOutput::Leaf(TestOutputLeafOp::Projected(policy)),
        owned_field_view,
        |ty, views, extension| {
            let mut output = TestOutput::combine(views);
            if extension.is_some_and(|options| options.bulk) {
                output = TestOutput::Unary {
                    operation: (TestOutputUnary::Bulk).into(),
                    child: Box::new(output),
                };
            }
            TestOutput::for_type(ty, output)
        },
    )
}
/// Decompose a source accessor's returned record into named field views.
pub fn fields(function: syn::Ident) -> TestFields {
    crate::DeclFields::new(
        function,
        |output| match output {
            TestOutput::Unary {
                operation: crate::Operation::Request(TestOutputRequest::ForType { ty }),
                child: output,
            } if matches!(output.as_ref(),TestOutput::Branch {operation:crate::Operation::Resolved(TestOutputBranch::Combine),children} if children.is_empty()) => {
                TestOutput::for_type(*ty, TestOutput::Leaf(TestOutputLeafOp::Discard))
            }
            output => output,
        },
        |function, overrides, _extension| {
            let (fields, children) = overrides.into_iter().unzip();
            TestOutput::getter(
                function,
                TestOutput::Branch {
                    operation: crate::Operation::Request(TestOutputRequest::NamedFields {
                        fields,
                        mode: crate::ProjectionMode::Borrowed,
                    }),
                    children,
                },
            )
        },
    )
}

pub type TestFunction = crate::DeclFunction<
    TestInput<TestInputRequest, TestOutputRequest>,
    TestOutput<TestInputRequest, TestOutputRequest>,
>;
pub type TestCallback = crate::DeclCallback<
    TestInput<TestInputRequest, TestOutputRequest>,
    TestOutput<TestInputRequest, TestOutputRequest>,
>;
impl
    crate::CallbackInput<
        TestInput<TestInputRequest, TestOutputRequest>,
        TestOutput<TestInputRequest, TestOutputRequest>,
    > for TestInputRequest
{
    fn callback_input(declaration: TestCallback) -> TestInput<TestInputRequest, TestOutputRequest> {
        crate::callback_input(
            declaration,
            TestInput::auto,
            |children| TestOutput::Branch {
                operation: TestOutputBranch::Arguments.into(),
                children,
            },
            |metadata, domain, codomain| TestInput::Arrow {
                operation: crate::Operation::Request(Self::Callback(Box::new(metadata))),
                domain: Box::new(domain),
                codomain: Box::new(codomain),
            },
            TestInput::for_type,
        )
    }
}
#[derive(Clone)]
/// This environment is discarded after prepared conversions have been installed.
pub struct TestDefaults {
    values: crate::DeclDefaults<
        TestInput<TestInputRequest, TestOutputRequest>,
        TestOutput<TestInputRequest, TestOutputRequest>,
    >,
}
fn input_body(
    input: TestInput<TestInputRequest, TestOutputRequest>,
) -> TestInput<TestInputRequest, TestOutputRequest> {
    match input {
        TestInput::Branch {
            operation: crate::Operation::Resolved(TestInputBranch::Selector),
            children: mut arms,
        } if arms.len() == 1 => arms.remove(0),
        TestInput::Unary {
            operation: crate::Operation::Resolved(TestInputUnary::Bulk),
            child,
        } => TestInput::Unary {
            operation: (TestInputUnary::Bulk).into(),
            child: Box::new(input_body(*child)),
        },
        TestInput::Unary {
            operation: crate::Operation::Resolved(TestInputUnary::Temporary),
            child,
        } => TestInput::Unary {
            operation: (TestInputUnary::Temporary).into(),
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
            operation: crate::Operation::Resolved(TestOutputBranch::Combine),
            children: mut parts,
        } if parts.len() == 1 => parts.remove(0),
        TestOutput::Unary {
            operation: crate::Operation::Resolved(TestOutputUnary::Bulk),
            child,
        } => TestOutput::Unary {
            operation: (TestOutputUnary::Bulk).into(),
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
        let inputs = inputs
            .into_iter()
            .map(|tree| match tree {
                TestInput::Unary {
                    operation: crate::Operation::Request(TestInputRequest::ForType { ty }),
                    child: input,
                } => Ok((*ty, input_body(*input))),
                _ => Err("input defaults require a source type".into()),
            })
            .collect::<Result<Vec<_>>>()?;
        let outputs = outputs
            .into_iter()
            .map(|tree| match tree {
                TestOutput::Unary {
                    operation: crate::Operation::Request(TestOutputRequest::ForType { ty }),
                    child: output,
                } => Ok((*ty, output_body(*output))),
                _ => Err("output defaults require a source type".into()),
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            values: crate::DeclDefaults::new(flat, inputs, outputs, errors)?,
        })
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
                operation: (TestOutputUnary::Result {
                    error: self.error_mapper_for(decl, error).map(str::to_owned),
                })
                .into(),
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
            .or_else(|| self.values.errors.get(&type_key(ty)).map(String::as_str))
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
                operation: crate::Operation::Resolved(TestInputBranch::Variant),
                ..
            } => Err("variant branch requires an enum parent".into()),
            TestInput::Branch {
                operation: crate::Operation::Request(TestInputRequest::EnumFields),
                children,
            } => {
                let entries = crate::enum_children(children, |tree| match tree {
                    TestInput::Branch {
                        operation:
                            crate::Operation::Request(TestInputRequest::VariantFields {
                                variant,
                                fields,
                            }),
                        children,
                    } => Ok((variant.as_str(), fields.as_slice(), children.as_slice())),
                    _ => Err("named enum fields require variant branches".into()),
                })?;
                self.enum_input_recipe(flat, codec, ty, extra, &entries, false, stack)
            }
            TestInput::Branch {
                operation: crate::Operation::Resolved(TestInputBranch::Enum { value }),
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
                operation: crate::Operation::Resolved(TestInputUnary::Convert { conversion }),
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
                operation: crate::Operation::Resolved(TestInputUnary::Temporary),
                child,
            } => Ok(InputNode::Unary {
                operation: (TestInputUnary::Temporary).into(),
                child: Box::new(resolve(
                    &crate::temporary_input_type(ty)?,
                    &input_body((**child).clone()),
                    stack,
                )?),
            }),
            TestInput::Unary {
                operation: crate::Operation::Resolved(TestInputUnary::Bulk),
                child,
            } => Ok(InputNode::Unary {
                operation: (TestInputUnary::Bulk).into(),
                child: Box::new(resolve(ty, &input_body((**child).clone()), stack)?),
            }),
            TestInput::Request(TestInputRequest::Auto) => {
                let key = crate::callback::input_key(flat, ty)?;
                if let Some(tree) = crate::resolve_type_default(
                    &key,
                    &self.values.inputs,
                    stack,
                    crate::Direction::Input,
                    |tree, stack| resolve(ty, tree, stack),
                )? {
                    return Ok(tree);
                }
                if let Some(tree) = crate::resolve_container(
                    ty,
                    true,
                    |inner| resolve(inner, &TestInput::auto(), stack),
                    |child| TestInput::Unary {
                        operation: (TestInputUnary::Optional).into(),
                        child: Box::new(child),
                    },
                    |child| TestInput::Unary {
                        operation: (TestInputUnary::Sequence).into(),
                        child: Box::new(child),
                    },
                )? {
                    return Ok(tree);
                }
                if type_key(ty) == "()" {
                    return Ok(InputNode::None);
                }
                resolve(ty, &TestInput::Leaf(TestInputLeafOp::Value), stack)
            }
            TestInput::Unary {
                operation: crate::Operation::Request(TestInputRequest::ForType { ty: target }),
                child: input,
            } => {
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
                        operation: (TestInputUnary::Optional).into(),
                        child: Box::new(child),
                    },
                    |child| TestInput::Unary {
                        operation: (TestInputUnary::Sequence).into(),
                        child: Box::new(child),
                    },
                    crate::Direction::Input,
                )
            }
            TestInput::Leaf(TestInputLeafOp::Value) => {
                if let Some(conversion) = crate::input_value_conversion(flat, ty)? {
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
                    return resolve(ty, &TestInput::from(TestCallback::new(ty.clone())), stack);
                }
                if let Some(inner) = option_inner(ty) {
                    return Ok(InputNode::Unary {
                        operation: (TestInputUnary::Optional).into(),
                        child: Box::new(resolve(
                            inner,
                            &TestInput::Leaf(TestInputLeafOp::Value),
                            stack,
                        )?),
                    });
                }
                if let Some((element, _)) = sequence_parts(ty) {
                    return Ok(InputNode::Unary {
                        operation: (TestInputUnary::Sequence).into(),
                        child: Box::new(resolve(element, &TestInput::auto(), stack)?),
                    });
                }
                Ok(TestInput::Leaf(TestInputLeafOp::Value))
            }
            TestInput::Arrow {
                operation: crate::Operation::Resolved(callback),
                domain,
                codomain,
            } => {
                let (arguments, result) = crate::resolve_callback_arrow(
                    flat,
                    ty,
                    callback,
                    domain.argument_nodes()?,
                    codomain,
                    |ty, tree| self.output(flat, codec, ty, tree, extra, &mut vec![], true),
                    |ty, tree| resolve(ty, tree, stack),
                )?;
                Ok(InputNode::Arrow {
                    operation: (callback.clone()).into(),
                    domain: Box::new(TestOutput::arguments(arguments)),
                    codomain: Box::new(result),
                })
            }
            TestInput::Arrow {
                operation: crate::Operation::Request(TestInputRequest::Callback(declaration)),
                domain,
                codomain,
            } => {
                let canonical = crate::callback::input_key(flat, ty)?;
                let default = match self.values.inputs.get(&canonical) {
                    Some(TestInput::Arrow {
                        operation: crate::Operation::Request(TestInputRequest::Callback(request)),
                        domain,
                        codomain,
                    }) => Some(crate::CallbackTree {
                        request,
                        arguments: domain.argument_nodes()?,
                        result: codomain.as_ref(),
                    }),
                    _ => None,
                };
                let (operation, arguments, ret) = crate::resolve_callback_tree(
                    flat,
                    ty,
                    crate::CallbackTree {
                        request: declaration,
                        arguments: domain.argument_nodes()?,
                        result: codomain.as_ref(),
                    },
                    default,
                    &self.values.outputs,
                    &TestInput::auto(),
                    &TestOutput::auto(),
                    |tree| matches!(tree, TestInput::Request(TestInputRequest::Auto)),
                    stack,
                    resolve,
                    |ty, tree, ownership| {
                        Ok(self
                            .output(flat, codec, ty, tree, extra, &mut vec![], true)?
                            .callback_argument(ownership))
                    },
                )?;
                Ok(InputNode::Arrow {
                    operation: Box::new(operation).into(),
                    domain: Box::new(TestOutput::arguments(arguments)),
                    codomain: Box::new(ret),
                })
            }

            TestInput::Request(TestInputRequest::Call(function)) => {
                let f = flat.function(function)?;
                resolve(
                    ty,
                    &TestInput::Branch {
                        operation: (TestInputBranch::Constructor {
                            function: function.clone(),
                        })
                        .into(),
                        children: vec![TestInput::auto(); f.params.len()],
                    },
                    stack,
                )
            }
            TestInput::Branch {
                operation: crate::Operation::Resolved(TestInputBranch::Constructor { function }),
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
                operation: crate::Operation::Resolved(TestInputUnary::Optional),
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(InputNode::Unary {
                    operation: (TestInputUnary::Optional).into(),
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            TestInput::Unary {
                operation: crate::Operation::Resolved(TestInputUnary::Sequence),
                child,
            } => {
                let (element, _) =
                    sequence_parts(ty).ok_or("sequence input requires Vec<T>, &[T] or &Vec<T>")?;
                Ok(InputNode::Unary {
                    operation: (TestInputUnary::Sequence).into(),
                    child: Box::new(resolve(element, child, stack)?),
                })
            }
            TestInput::Branch {
                operation: crate::Operation::Resolved(TestInputBranch::Fields),
                children,
            }
            | TestInput::Branch {
                operation: crate::Operation::Resolved(TestInputBranch::Record),
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
                            operation: crate::Operation::Resolved(TestInputBranch::Record),
                            children: _
                        }
                    ) {
                        InputNode::Branch {
                            operation: (TestInputBranch::Record).into(),
                            children,
                        }
                    } else {
                        InputNode::Branch {
                            operation: (TestInputBranch::Fields).into(),
                            children,
                        }
                    },
                )
            }
            TestInput::Branch {
                operation: crate::Operation::Resolved(TestInputBranch::Selector),
                children: arms,
            } => {
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                Ok(InputNode::Branch {
                    operation: (TestInputBranch::Selector).into(),
                    children: arms
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
            TestInput::Request(_)
            | TestInput::Unary {
                operation: crate::Operation::Request(_),
                ..
            }
            | TestInput::Branch {
                operation: crate::Operation::Request(_),
                ..
            }
            | TestInput::Arrow {
                operation: crate::Operation::Request(_),
                ..
            } => Err("pending operation has the wrong structural node shape".into()),
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
                    tree.pending_request(),
                    Some(
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
            TestOutput::Unary {
                operation:
                    crate::Operation::Request(TestOutputRequest::SequenceCallback { structural }),
                child: output,
            } => {
                let _ = structural;
                let _ = resolve(ty, output, stack)?;
                Err("callback return is unsupported by this adapter".into())
            }

            TestOutput::Unary {
                operation:
                    crate::Operation::Resolved(TestOutputUnary::CallbackArgument { ownership }),
                child,
            } => Ok((resolve(ty, child, stack)?).callback_argument(*ownership)),
            TestOutput::None => {
                if type_key(ty) == "()" {
                    Ok(OutputNode::None)
                } else {
                    Err("None output requires unit".into())
                }
            }
            TestOutput::Arrow {
                operation: crate::Operation::Resolved(never),
                ..
            } => match *never {},
            TestOutput::Branch {
                operation:
                    crate::Operation::Resolved(TestOutputBranch::Variant | TestOutputBranch::Arguments),
                ..
            } => Err("variant/argument branch requires its parent operation".into()),
            TestOutput::Branch {
                operation: crate::Operation::Request(TestOutputRequest::EnumFields),
                children,
            } => {
                let entries = crate::enum_children(children, |tree| match tree {
                    TestOutput::Branch {
                        operation:
                            crate::Operation::Request(TestOutputRequest::VariantFields {
                                variant,
                                fields,
                            }),
                        children,
                    } => Ok((variant.as_str(), fields.as_slice(), children.as_slice())),
                    _ => Err("named enum fields require variant branches".into()),
                })?;
                self.enum_output_recipe(
                    flat,
                    codec,
                    crate::output_referent(ty).unwrap_or(ty),
                    extra,
                    &entries,
                    false,
                    stack,
                    getter_result,
                )
            }

            TestOutput::Branch {
                operation: crate::Operation::Resolved(TestOutputBranch::Enum { value, mode }),
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
                operation: crate::Operation::Resolved(TestOutputUnary::Convert { conversion }),
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
                operation: crate::Operation::Resolved(TestOutputUnary::Bulk),
                child,
            } => Ok(OutputNode::Unary {
                operation: (TestOutputUnary::Bulk).into(),
                child: Box::new(resolve(ty, &output_body((**child).clone()), stack)?),
            }),
            TestOutput::Leaf(TestOutputLeafOp::Discard) => {
                Ok(OutputNode::Leaf(TestOutputLeafOp::Discard))
            }
            TestOutput::Unary {
                operation: crate::Operation::Resolved(TestOutputUnary::Named { name }),
                child: output,
            } => Ok(OutputNode::Unary {
                operation: (TestOutputUnary::Named { name: name.clone() }).into(),
                child: Box::new(resolve(ty, output, stack)?),
            }),
            TestOutput::Request(TestOutputRequest::Auto) => {
                let key = type_key(ty);
                if let Some(tree) = crate::resolve_type_default(
                    &key,
                    &self.values.outputs,
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
                if let Some(tree) = crate::resolve_container(
                    ty,
                    true,
                    |inner| resolve(inner, &TestOutput::auto(), stack),
                    |child| TestOutput::Unary {
                        operation: (TestOutputUnary::Optional).into(),
                        child: Box::new(child),
                    },
                    |child| TestOutput::Unary {
                        operation: (TestOutputUnary::Sequence {
                            delivery: SequenceDelivery::Auto,
                            mode: None,
                        })
                        .into(),
                        child: Box::new(child),
                    },
                )? {
                    return Ok(tree);
                }
                if type_key(ty) == "()" {
                    return Ok(OutputNode::None);
                }
                resolve(ty, &TestOutput::Leaf(TestOutputLeafOp::Value), stack)
            }
            TestOutput::Unary {
                operation: crate::Operation::Request(TestOutputRequest::ForType { ty: target }),
                child: output,
            } => {
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
                                operation: (TestOutputUnary::Optional).into(),
                                child: Box::new(child),
                            }
                        }
                    },
                    |child| {
                        if matches!(&child, TestOutput::Leaf(TestOutputLeafOp::Discard)) {
                            child
                        } else {
                            TestOutput::Unary {
                                operation: (TestOutputUnary::Sequence {
                                    delivery: SequenceDelivery::Auto,
                                    mode: None,
                                })
                                .into(),
                                child: Box::new(child),
                            }
                        }
                    },
                    crate::Direction::Output,
                )
            }
            TestOutput::Leaf(TestOutputLeafOp::Value)
            | TestOutput::Leaf(TestOutputLeafOp::Projected(_)) => {
                if let Some(conversion) = crate::output_value_conversion(flat, ty, None)? {
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
                        operation: (TestOutputUnary::Optional).into(),
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
                        operation: (TestOutputUnary::Sequence {
                            delivery: SequenceDelivery::Value,
                            mode,
                        })
                        .into(),
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
                operation: crate::Operation::Resolved(TestOutputUnary::Optional),
                child,
            } => {
                let inner = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                Ok(OutputNode::Unary {
                    operation: (TestOutputUnary::Optional).into(),
                    child: Box::new(resolve(inner, child, stack)?),
                })
            }
            TestOutput::Unary {
                operation: crate::Operation::Resolved(TestOutputUnary::Sequence { delivery, mode }),
                child: element,
            } => {
                let (ty, _) =
                    sequence_parts(ty).ok_or("sequence output requires a collection source")?;
                Ok(OutputNode::Unary {
                    operation: (TestOutputUnary::Sequence {
                        delivery: *delivery,
                        mode: *mode,
                    })
                    .into(),
                    child: Box::new(resolve(ty, element, stack)?),
                })
            }
            TestOutput::Branch {
                operation:
                    crate::Operation::Request(TestOutputRequest::NamedFields { fields, mode }),
                children: recipes,
            } => {
                let overrides = crate::named_children(fields, recipes)?;
                let children = crate::resolve_named_fields(
                    flat,
                    ty,
                    &overrides,
                    &TestOutput::auto(),
                    |ty, tree| resolve(ty, tree, stack),
                )?;
                Ok(if *mode == ProjectionMode::Owned {
                    OutputNode::Branch {
                        operation: (TestOutputBranch::OwnedFields).into(),
                        children,
                    }
                } else {
                    OutputNode::Branch {
                        operation: (TestOutputBranch::Fields).into(),
                        children,
                    }
                })
            }
            TestOutput::Branch {
                operation: crate::Operation::Resolved(TestOutputBranch::Fields),
                children,
            }
            | TestOutput::Branch {
                operation: crate::Operation::Resolved(TestOutputBranch::OwnedFields),
                children,
            }
            | TestOutput::Branch {
                operation: crate::Operation::Resolved(TestOutputBranch::Record { .. }),
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
                        operation: crate::Operation::Resolved(TestOutputBranch::Fields),
                        children: _,
                    } => OutputNode::Branch {
                        operation: (TestOutputBranch::Fields).into(),
                        children,
                    },
                    TestOutput::Branch {
                        operation: crate::Operation::Resolved(TestOutputBranch::OwnedFields),
                        children: _,
                    } => OutputNode::Branch {
                        operation: (TestOutputBranch::OwnedFields).into(),
                        children,
                    },
                    TestOutput::Branch {
                        operation: crate::Operation::Resolved(TestOutputBranch::Record { mode, .. }),
                        ..
                    } => OutputNode::Branch {
                        operation: (TestOutputBranch::Record { mode: *mode }).into(),
                        children,
                    },
                    _ => unreachable!(),
                })
            }
            TestOutput::Unary {
                operation: crate::Operation::Resolved(TestOutputUnary::Getter { function }),
                child: output,
            }
            | TestOutput::Unary {
                operation: crate::Operation::Resolved(TestOutputUnary::Into { function }),
                child: output,
            } => {
                let output = Box::new(crate::resolve_getter(
                    flat,
                    ty,
                    function,
                    matches!(
                        tree,
                        TestOutput::Unary {
                            operation: crate::Operation::Resolved(TestOutputUnary::Into { .. }),
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
                            operation: crate::Operation::Resolved(TestOutputUnary::Into { .. }),
                            ..
                        }
                    ) {
                        OutputNode::Unary {
                            operation: (TestOutputUnary::Into {
                                function: function.clone(),
                            })
                            .into(),
                            child: output,
                        }
                    } else {
                        OutputNode::Unary {
                            operation: (TestOutputUnary::Getter {
                                function: function.clone(),
                            })
                            .into(),
                            child: output,
                        }
                    },
                )
            }
            TestOutput::Branch {
                operation: crate::Operation::Resolved(TestOutputBranch::Combine),
                children: parts,
            } => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                Ok(OutputNode::Branch {
                    operation: (TestOutputBranch::Combine).into(),
                    children: parts
                        .iter()
                        .map(|child| resolve(ty, child, stack))
                        .collect::<Result<_>>()?,
                })
            }
            TestOutput::Unary {
                operation: crate::Operation::Resolved(TestOutputUnary::Result { error }),
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
                    operation: (TestOutputUnary::Result {
                        error: error.clone(),
                    })
                    .into(),
                    child: Box::new(resolve(success, ok, stack)?),
                })
            }
            TestOutput::Request(_)
            | TestOutput::Unary {
                operation: crate::Operation::Request(_),
                ..
            }
            | TestOutput::Branch {
                operation: crate::Operation::Request(_),
                ..
            }
            | TestOutput::Arrow {
                operation: crate::Operation::Request(_),
                ..
            } => Err("pending operation has the wrong structural node shape".into()),
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
            operation: (TestInputUnary::Convert {
                conversion: Box::new(conversion),
            })
            .into(),
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
            operation: (TestOutputUnary::Convert {
                conversion: Box::new(conversion),
            })
            .into(),
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
                operation: (TestInputBranch::Fields).into(),
                children: vec![],
            });
        }
        if let Some(c) = crate::input_value_conversion(flat, ty)? {
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
        if let Some(tree) = crate::resolve_container(
            ty,
            true,
            |inner| self.resolve_input_value(flat, codec, inner, extra, stack),
            |child| TestInput::Unary {
                operation: (TestInputUnary::Optional).into(),
                child: Box::new(child),
            },
            |child| TestInput::Unary {
                operation: (TestInputUnary::Sequence).into(),
                child: Box::new(child),
            },
        )? {
            return Ok(tree);
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
                operation: (TestOutputBranch::Fields).into(),
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
        if let Some(tree) = crate::resolve_container(
            ty,
            true,
            |inner| self.resolve_output_value(flat, codec, inner, extra, stack, getter, policy),
            |child| TestOutput::Unary {
                operation: (TestOutputUnary::Optional).into(),
                child: Box::new(child),
            },
            |child| TestOutput::Unary {
                operation: (TestOutputUnary::Sequence {
                    delivery: crate::SequenceDelivery::Value,
                    mode: policy.map(|p| p.mode),
                })
                .into(),
                child: Box::new(child),
            },
        )? {
            return Ok(tree);
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
            BTreeMap<String, &TestInput<TestInputRequest, TestOutputRequest>>,
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
            BTreeMap<String, &TestOutput<TestInputRequest, TestOutputRequest>>,
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

impl crate::LocalConversion<TestInput<TestInputRequest, TestOutputRequest>> for TestInputRequest {
    fn local_conversion(
        declaration: DeclConvert,
    ) -> TestInput<TestInputRequest, TestOutputRequest> {
        TestInput::for_type(
            declaration.ty.clone(),
            TestInput::Request(Self::Conversion(Box::new(declaration))),
        )
    }
}

impl crate::LocalConversion<TestOutput<TestInputRequest, TestOutputRequest>> for TestOutputRequest {
    fn local_conversion(
        declaration: DeclConvert,
    ) -> TestOutput<TestInputRequest, TestOutputRequest> {
        TestOutput::for_type(
            declaration.ty.clone(),
            TestOutput::Request(Self::Conversion(Box::new(declaration))),
        )
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
macro_rules! test_expand_param {($($ty:tt)+)=>{$crate::test_language::expand_param($crate::declarations::parse_type(stringify!($($ty)+)))};}
#[macro_export]
macro_rules! test_expand_return {($($ty:tt)+)=>{$crate::test_language::expand_return($crate::declarations::parse_type(stringify!($($ty)+)))};}
#[macro_export]
macro_rules! test_fields {
    ($name:ident) => {
        $crate::test_language::fields($crate::ident(stringify!($name)).unwrap())
    };
}
#[macro_export]
macro_rules! test_callback {($($ty:tt)+)=>{$crate::test_language::TestCallback::new($crate::declarations::parse_type(stringify!($($ty)+)))};}

/// Collect source names; all recursive traversal follows structural edges.
pub fn source_items<'a>(
    inputs: &[TestInput<TestInputRequest, TestOutputRequest>],
    outputs: &[TestOutput<TestInputRequest, TestOutputRequest>],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a TestFunction>,
) -> Vec<String> {
    fn input(tree: &TestInput<TestInputRequest, TestOutputRequest>, names: &mut Vec<String>) {
        match tree.pending_request() {
            Some(TestInputRequest::Conversion(declaration)) => {
                crate::conversion_source_items(declaration, names)
            }
            Some(TestInputRequest::Call(name)) => names.push(name.clone()),
            _ => {}
        }
        match tree {
            TestInput::Unary {
                operation: crate::Operation::Resolved(TestInputUnary::Convert { conversion }),
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            TestInput::Branch {
                operation: crate::Operation::Resolved(TestInputBranch::Constructor { function }),
                ..
            } => names.push(function.clone()),
            _ => {}
        }
        crate::input_children(tree, names, &mut input, &mut output);
    }
    fn output(tree: &TestOutput<TestInputRequest, TestOutputRequest>, names: &mut Vec<String>) {
        if let Some(TestOutputRequest::Conversion(declaration)) = tree.pending_request() {
            crate::conversion_source_items(declaration, names)
        }
        match tree {
            TestOutput::Unary {
                operation: crate::Operation::Resolved(TestOutputUnary::Convert { conversion }),
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            TestOutput::Unary {
                operation:
                    crate::Operation::Resolved(
                        TestOutputUnary::Getter { function } | TestOutputUnary::Into { function },
                    ),
                ..
            } => names.push(function.clone()),
            TestOutput::Unary {
                operation: crate::Operation::Resolved(TestOutputUnary::Result { error: Some(name) }),
                ..
            } => names.push(name.clone()),
            _ => {}
        }
        crate::output_children(tree, names, &mut input, &mut output);
    }
    crate::collect_source_items(inputs, outputs, errors, functions, &mut input, &mut output)
}

fn owned_field_view(
    view: TestOutput<TestInputRequest, TestOutputRequest>,
) -> TestOutput<TestInputRequest, TestOutputRequest> {
    let TestOutput::Unary {
        operation: crate::Operation::Resolved(TestOutputUnary::Getter { function }),
        child,
    } = view
    else {
        panic!("fields_into requires a fields! accessor")
    };
    let TestOutput::Branch {
        operation: crate::Operation::Request(TestOutputRequest::NamedFields { fields, .. }),
        children,
    } = *child
    else {
        panic!("fields_into requires named fields")
    };
    TestOutput::Unary {
        operation: TestOutputUnary::Into { function }.into(),
        child: Box::new(TestOutput::Branch {
            operation: crate::Operation::Request(TestOutputRequest::NamedFields {
                fields,
                mode: ProjectionMode::Owned,
            }),
            children,
        }),
    }
}
