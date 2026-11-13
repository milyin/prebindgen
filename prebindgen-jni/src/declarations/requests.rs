use super::*;
#[derive(Clone, Debug)]
/// Source-dependent input choices resolved before compilation.
pub enum JniInputRequest {
    /// Expand enum fields, inheriting defaults for fields not explicitly overridden.
    EnumFields(
        std::collections::BTreeMap<
            String,
            std::collections::BTreeMap<String, JniInput<JniInputRequest, JniOutputRequest>>,
        >,
    ),
    /// Resolve only the canonical value form, independently of boundary defaults.
    Value,
    /// Select a local single-value conversion.
    Conversion(Box<DeclConvert>),
    /// Select the configured default for this source occurrence.
    Auto,
    /// Select or override a callback signature's argument and result conversions.
    Callback(Box<DeclJniCallback>),
    /// Apply a type-specific override, including inside optional/sequence sources.
    ForType {
        /// Source type targeted by the override.
        ty: Box<Type>,
        /// The requested conversion for that type.
        input: Box<JniInput<JniInputRequest, JniOutputRequest>>,
    },
    /// Infer a constructor's argument conversions from its source signature.
    Call(String),
}
#[derive(Clone, Debug)]
/// Source-dependent output choices resolved before compilation.
pub enum JniOutputRequest {
    /// Select per-element delivery after resolving defaults and local expansion.
    SequenceCallback {
        /// Expansion whose outer sequence supplies callback elements.
        output: Box<JniOutput<JniInputRequest, JniOutputRequest>>,
        /// Allow structural decomposition of a default collection identity.
        structural: bool,
    },
    /// Expand enum fields, inheriting boundary defaults for unspecified fields.
    EnumFields(
        std::collections::BTreeMap<
            String,
            std::collections::BTreeMap<String, JniOutput<JniInputRequest, JniOutputRequest>>,
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
        output: Box<JniOutput<JniInputRequest, JniOutputRequest>>,
    },
    /// Resolve source record fields by name and fill unspecified conversions.
    NamedFields {
        /// Explicit conversions keyed by source field name.
        overrides: std::collections::BTreeMap<String, JniOutput<JniInputRequest, JniOutputRequest>>,
        /// Ownership of field decomposition.
        mode: ProjectionMode,
    },
}
/// Fluent conversion construction for this language.
pub trait JniInputExt<InRequest, OutRequest>: Sized {
    /// Group this explicit expansion into a bulk carrier.
    fn bulk(self) -> Self;
    /// Describe source record fields in declaration order.
    fn fields(fields: impl IntoIterator<Item = JniInput<InRequest, OutRequest>>) -> Self;
    /// Construct a source value with explicit argument conversions.
    fn constructor(
        name: impl Into<String>,
        args: impl IntoIterator<Item = JniInput<InRequest, OutRequest>>,
    ) -> Self;
    /// Choose between owned representations of the same input type.
    ///
    /// `arms` holds one recipe per alternative; each must produce the input's
    /// type. The caller passes the zero-based index of the chosen alternative in an
    /// extra wire ahead of all arms' wires.
    ///
    /// Requires at least one arm. Each arm retains its own wires; this does not
    /// merge equivalent wire parameters or generate destination-language overloads.
    fn selector(arms: impl IntoIterator<Item = JniInput<InRequest, OutRequest>>) -> Self;
    /// Describe source variants and their field conversions in source order.
    fn enum_variants(variants: Vec<Vec<Self>>, value: bool) -> Self;
    /// Read the field conversions of one enum variant.
    fn variant_fields(&self) -> crate::Result<&[Self]>;
    /// Convert each element of a source collection.
    fn sequence(element: Self) -> Self;
}
impl<InRequest, OutRequest> JniInputExt<InRequest, OutRequest> for JniInput<InRequest, OutRequest> {
    /// Group this explicit expansion into a bulk carrier.
    fn bulk(self) -> Self {
        JniInput::Unary {
            operation: JniInputUnary::Bulk,
            child: Box::new(self),
        }
    }
    /// Construct a record from field recipes, or unit from an empty list.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration
    /// order. Binding verifies the count against the record.
    fn fields(fields: impl IntoIterator<Item = JniInput<InRequest, OutRequest>>) -> Self {
        JniInput::Branch {
            operation: JniInputBranch::Fields,
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
    fn constructor(
        name: impl Into<String>,
        args: impl IntoIterator<Item = JniInput<InRequest, OutRequest>>,
    ) -> Self {
        JniInput::Branch {
            operation: JniInputBranch::Constructor {
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
    fn selector(arms: impl IntoIterator<Item = JniInput<InRequest, OutRequest>>) -> Self {
        JniInput::Branch {
            operation: JniInputBranch::Selector,
            children: arms.into_iter().collect(),
        }
    }

    /// Describe source variants, each with an ordinary branch of field conversions.
    fn enum_variants(variants: Vec<Vec<Self>>, value: bool) -> Self {
        Self::Branch {
            operation: JniInputBranch::Enum { value },
            children: variants
                .into_iter()
                .map(|children| Self::Branch {
                    operation: JniInputBranch::Variant,
                    children,
                })
                .collect(),
        }
    }
    /// Read an enum variant's field conversions.
    fn variant_fields(&self) -> crate::Result<&[Self]> {
        match self {
            Self::Branch {
                operation: JniInputBranch::Variant,
                children,
            } => Ok(children),
            _ => Err("enum children must be variant branches".into()),
        }
    }

    /// Convert each incoming collection element using the supplied subtree.
    fn sequence(element: Self) -> Self {
        Self::Unary {
            operation: JniInputUnary::Sequence,
            child: Box::new(element),
        }
    }
}

/// Fluent conversion construction for this language.
pub trait JniInputRequests: Sized {
    /// Expand the tag and selected fields. Keys are source variant and field
    /// names (or tuple indices). Unspecified fields inherit boundary defaults.
    fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self;
    /// Request the canonical value representation independently of expansion defaults.
    fn value() -> Self;
    /// Resolve the configured default for this source occurrence.
    fn auto() -> Self;
    /// Apply a conversion to the matching source type, including nested occurrences.
    fn for_type(ty: Type, input: Self) -> Self;
    /// Infer a source constructor's argument conversions from registered defaults.
    fn call(name: impl Into<String>) -> Self;
    /// Explicitly construct owned backing storage for a borrowed input.
    fn reconstruct(self) -> Self;
    /// Append a constructor alternative.
    fn variant(self, name: impl Into<String>) -> Self;
    /// Append an explicit input alternative.
    fn add_alternative(self, arm: Self) -> Self;
    /// Append the direct value representation as an input alternative.
    fn variant_self(self) -> Self;
}
impl JniInputRequests for JniInput<JniInputRequest, JniOutputRequest> {
    /// Expand the tag and selected fields. Keys are source variant and field
    /// names (or tuple indices). Unspecified fields inherit boundary defaults.
    fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self {
        JniInput::Request(JniInputRequest::EnumFields(
            prebindgen_declarations::enum_overrides(overrides),
        ))
    }
    /// Request the canonical value representation, bypassing boundary defaults.
    fn value() -> Self {
        JniInput::Request(JniInputRequest::Value)
    }
    /// Request this source type's configured default during preparation.
    fn auto() -> Self {
        JniInput::Request(JniInputRequest::Auto)
    }
    /// Associate a conversion with a source type for a default or nested override.
    fn for_type(ty: Type, input: Self) -> Self {
        JniInput::Request(JniInputRequest::ForType {
            ty: Box::new(ty),
            input: Box::new(input),
        })
    }
    /// Infer a source constructor's argument conversions from registered defaults.
    fn call(name: impl Into<String>) -> Self {
        JniInput::Request(JniInputRequest::Call(name.into()))
    }
    /// Explicitly reconstruct an owned temporary for a shared-reference input.
    /// Slices construct a Vec, str constructs a String, and sized references
    /// construct their referent. Child recipes keep their ordinary ownership
    /// semantics; this operation authorizes the temporary's allocation/copying.
    fn reconstruct(self) -> Self {
        match self {
            JniInput::Request(JniInputRequest::ForType { ty, input }) => JniInput::for_type(
                *ty,
                JniInput::Unary {
                    operation: JniInputUnary::Temporary,
                    child: input,
                },
            ),
            other => JniInput::Unary {
                operation: JniInputUnary::Temporary,
                child: Box::new(other),
            },
        }
    }
    /// Add a constructor alternative to this conversion tree.
    fn variant(self, name: impl Into<String>) -> Self {
        self.add_alternative(JniInput::call(name))
    }
    /// Add a direct codec alternative, bypassing this type's expansion default.
    fn variant_self(self) -> Self {
        self.add_alternative(JniInput::Leaf(JniInputLeafOp::Value))
    }
    fn add_alternative(self, arm: Self) -> Self {
        match self {
            JniInput::Request(JniInputRequest::ForType { ty, input }) => {
                JniInput::for_type(*ty, input.add_alternative(arm))
            }
            JniInput::Branch {
                operation: JniInputBranch::Selector,
                children: mut arms,
            } => {
                arms.push(arm);
                JniInput::Branch {
                    operation: JniInputBranch::Selector,
                    children: arms,
                }
            }
            other => JniInput::Branch {
                operation: JniInputBranch::Selector,
                children: vec![other, arm],
            },
        }
    }
}

/// Fluent conversion construction for this language.
pub trait JniOutputExt<InRequest, OutRequest>: Sized {
    /// Group this explicit expansion into a bulk carrier.
    fn bulk(self) -> Self;
    /// Describe source record fields in declaration order.
    fn fields(fields: impl IntoIterator<Item = JniOutput<InRequest, OutRequest>>) -> Self;
    /// Observe a source getter and convert its result.
    fn getter(name: impl Into<String>, output: JniOutput<InRequest, OutRequest>) -> Self;
    /// Deliver multiple views of one source value.
    fn combine(parts: impl IntoIterator<Item = JniOutput<InRequest, OutRequest>>) -> Self;
    /// Group callable argument conversions in invocation order.
    fn arguments(children: Vec<Self>) -> Self;
    /// Read the source-ordered conversions of a callable domain.
    fn argument_nodes(&self) -> crate::Result<&[Self]>;
    /// Describe source variants, each with an ordinary branch of field conversions.
    fn enum_variants(variants: Vec<Vec<Self>>, value: bool, mode: Option<ProjectionMode>) -> Self;
    /// Read the field conversions of one enum variant.
    fn variant_fields(&self) -> crate::Result<&[Self]>;
    /// Set the ownership contract of one callable argument.
    fn callback_argument(self, ownership: CallbackArgumentOwnership) -> Self;
    /// Read an argument's conversion and ownership contract.
    fn callback_argument_parts(&self) -> crate::Result<(CallbackArgumentOwnership, &Self)>;
}
impl<InRequest, OutRequest> JniOutputExt<InRequest, OutRequest>
    for JniOutput<InRequest, OutRequest>
{
    /// Group this explicit expansion into a bulk carrier.
    fn bulk(self) -> Self {
        JniOutput::Unary {
            operation: JniOutputUnary::Bulk,
            child: Box::new(self),
        }
    }
    /// Observe record fields in declaration order; use an empty list for unit.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration order.
    fn fields(fields: impl IntoIterator<Item = JniOutput<InRequest, OutRequest>>) -> Self {
        JniOutput::Branch {
            operation: JniOutputBranch::Fields,
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
    fn getter(name: impl Into<String>, output: JniOutput<InRequest, OutRequest>) -> Self {
        JniOutput::Unary {
            operation: JniOutputUnary::Getter {
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
    fn combine(parts: impl IntoIterator<Item = JniOutput<InRequest, OutRequest>>) -> Self {
        JniOutput::Branch {
            operation: JniOutputBranch::Combine,
            children: parts.into_iter().collect(),
        }
    }

    /// Group a callable's domain in source parameter order.
    fn arguments(children: Vec<Self>) -> Self {
        if children.is_empty() {
            Self::None
        } else {
            Self::Branch {
                operation: JniOutputBranch::Arguments,
                children,
            }
        }
    }
    /// Read the source-ordered components of a callable domain.
    fn argument_nodes(&self) -> crate::Result<&[Self]> {
        match self {
            Self::None => Ok(&[]),
            Self::Branch {
                operation: JniOutputBranch::Arguments,
                children,
            } => Ok(children),
            _ => Err("callback domain requires an argument branch or None".into()),
        }
    }

    /// Describe source variants, each with an ordinary branch of field conversions.
    fn enum_variants(variants: Vec<Vec<Self>>, value: bool, mode: Option<ProjectionMode>) -> Self {
        Self::Branch {
            operation: JniOutputBranch::Enum { value, mode },
            children: variants
                .into_iter()
                .map(|children| Self::Branch {
                    operation: JniOutputBranch::Variant,
                    children,
                })
                .collect(),
        }
    }
    /// Read an enum variant's field conversions.
    fn variant_fields(&self) -> crate::Result<&[Self]> {
        match self {
            Self::Branch {
                operation: JniOutputBranch::Variant,
                children,
            } => Ok(children),
            _ => Err("enum children must be variant branches".into()),
        }
    }

    /// Attach the lifetime of produced foreign owners to this callable argument.
    fn callback_argument(self, ownership: CallbackArgumentOwnership) -> Self {
        Self::Unary {
            operation: JniOutputUnary::CallbackArgument { ownership },
            child: Box::new(self),
        }
    }
    /// Read an argument's conversion and ownership contract from its own subtree.
    fn callback_argument_parts(&self) -> crate::Result<(CallbackArgumentOwnership, &Self)> {
        match self {
            Self::Unary {
                operation: JniOutputUnary::CallbackArgument { ownership },
                child,
            } => Ok((*ownership, child)),
            _ => Err("callback argument requires an explicit ownership contract".into()),
        }
    }
}

/// Fluent conversion construction for this language.
pub trait JniOutputRequests: Sized {
    /// Expand the tag and selected fields. Keys are source variant and field
    /// names (or tuple indices). Unspecified fields inherit boundary defaults.
    fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self;
    /// Request the canonical value representation independently of expansion defaults.
    fn value() -> Self;
    /// Resolve the configured default for this source occurrence.
    fn auto() -> Self;
    /// Apply a conversion to the matching source type, including nested occurrences.
    fn for_type(ty: Type, output: Self) -> Self;
    /// Append another simultaneous output view.
    fn view(self, view: impl Into<Self>) -> Self;
    /// Append a getter view using its default result conversion.
    fn field(self, name: impl Into<String>) -> Self;
    /// Append the direct value representation.
    fn field_self(self) -> Self;
    /// Append an explicitly borrowed projection.
    fn field_self_ref(self) -> Self;
    /// Append an explicitly owned projection.
    fn field_self_into(self) -> Self;
    /// Append a projection with explicit ownership and field policies.
    fn project_self(self, policy: Projection) -> Self;
    /// Append a consuming accessor followed by owned field decomposition.
    fn fields_into(self, view: Self) -> Self;
    /// Deprecated spelling of fields_into.
    fn fields_self_into(self, view: Self) -> Self;
    /// Override one named source field's conversion.
    fn field_override(self, name: impl Into<String>, output: Self) -> Self;
}
impl JniOutputRequests for JniOutput<JniInputRequest, JniOutputRequest> {
    /// Expand an enum's tag and active fields using source variant/member keys.
    fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self {
        JniOutput::Request(JniOutputRequest::EnumFields(
            prebindgen_declarations::enum_overrides(overrides),
        ))
    }
    /// Request the canonical value representation, bypassing boundary defaults.
    fn value() -> Self {
        JniOutput::Request(JniOutputRequest::Value { projection: None })
    }
    /// Request this source type's configured output default during preparation.
    fn auto() -> Self {
        JniOutput::Request(JniOutputRequest::Auto)
    }
    /// Associate a conversion with a source type for a default or nested override.
    fn for_type(ty: Type, output: Self) -> Self {
        JniOutput::Request(JniOutputRequest::ForType {
            ty: Box::new(ty),
            output: Box::new(output),
        })
    }
    /// Append another simultaneous output view.
    fn view(self, view: impl Into<Self>) -> Self {
        let view = view.into();
        match self {
            JniOutput::Request(JniOutputRequest::ForType { ty, output }) => {
                JniOutput::for_type(*ty, output.view(view))
            }
            JniOutput::Branch {
                operation: JniOutputBranch::Combine,
                children: mut parts,
            } => {
                parts.push(view);
                JniOutput::Branch {
                    operation: JniOutputBranch::Combine,
                    children: parts,
                }
            }
            other => JniOutput::Branch {
                operation: JniOutputBranch::Combine,
                children: vec![other, view],
            },
        }
    }
    /// Append a source getter whose result uses its registered conversion default.
    fn field(self, name: impl Into<String>) -> Self {
        self.view(JniOutput::getter(name, JniOutput::auto()))
    }
    /// Append the direct codec representation, bypassing this type's expansion default.
    fn field_self(self) -> Self {
        self.view(JniOutput::Leaf(JniOutputLeafOp::Value))
    }
    /// Append an explicit borrowed projection.
    fn field_self_ref(self) -> Self {
        self.project_self(Projection::borrowed())
    }
    /// Append an explicit owned projection.
    fn field_self_into(self) -> Self {
        self.project_self(Projection::owned())
    }
    /// Append a projection with explicit ownership and nested field policies.
    fn project_self(self, policy: Projection) -> Self {
        self.view(JniOutput::Leaf(JniOutputLeafOp::Projected(policy)))
    }
    /// Append a consuming accessor followed by owned field decomposition.
    fn fields_into(self, view: Self) -> Self {
        let JniOutput::Unary {
            operation: JniOutputUnary::Getter { function },
            child: output,
        } = view
        else {
            panic!("fields_into requires a fields! accessor")
        };
        let JniOutput::Request(JniOutputRequest::NamedFields { overrides, .. }) = *output else {
            panic!("fields_into requires named fields")
        };
        self.view(JniOutput::Unary {
            operation: JniOutputUnary::Into { function },
            child: Box::new(JniOutput::Request(JniOutputRequest::NamedFields {
                overrides,
                mode: ProjectionMode::Owned,
            })),
        })
    }
    /// Deprecated spelling of [`JniOutput::fields_into`].
    fn fields_self_into(self, view: Self) -> Self {
        self.fields_into(view)
    }
    /// Override one named source field, preserving defaults for its siblings.
    fn field_override(mut self, name: impl Into<String>, output: Self) -> Self {
        match &mut self {
            JniOutput::Request(JniOutputRequest::NamedFields { overrides, .. }) => {
                assert!(
                    overrides.insert(name.into(), output).is_none(),
                    "duplicate field override"
                );
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Getter { .. },
                child: inner,
                ..
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Into { .. },
                child: inner,
                ..
            }
            | JniOutput::Request(JniOutputRequest::ForType { output: inner, .. }) => {
                let old = std::mem::replace(inner, Box::new(JniOutput::auto()));
                **inner = old.field_override(name, output);
            }
            _ => panic!("field override requires named field decomposition"),
        }
        self
    }
}

impl From<DeclConvert> for JniInputRequest {
    fn from(decl: DeclConvert) -> Self {
        Self::ForType {
            ty: Box::new(decl.ty.clone()),
            input: Box::new(JniInput::Request(Self::Conversion(Box::new(decl)))),
        }
    }
}
impl From<DeclConvert> for JniOutputRequest {
    fn from(decl: DeclConvert) -> Self {
        Self::ForType {
            ty: Box::new(decl.ty.clone()),
            output: Box::new(JniOutput::Request(Self::Conversion(Box::new(decl)))),
        }
    }
}
