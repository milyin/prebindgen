use super::*;

/// Fluent conversion construction for this language.
pub trait CInputExt: Sized {
    /// Describe source record fields in declaration order.
    fn fields(fields: impl IntoIterator<Item = Self>) -> Self;
    /// Construct a source value with explicit argument conversions.
    fn constructor(name: impl Into<String>, args: impl IntoIterator<Item = Self>) -> Self;
    /// Choose between alternative input conversions.
    fn selector(arms: impl IntoIterator<Item = Self>) -> Self;
    /// Describe source variants and their field conversions in source order.
    fn enum_variants(variants: Vec<Vec<Self>>, value: bool) -> Self;
    /// Read the field conversions of one enum variant.
    fn variant_fields(&self) -> crate::Result<&[Self]>;
    /// Convert each element of a source collection.
    fn sequence(element: Self) -> Self;
}
// Both phases share construction and structural inspection of complete operations.
macro_rules! impl_c_input_helpers {
    ($node:ty) => {
        impl CInputExt for $node {
            /// Construct a record from field recipes, or unit from an empty list.
            ///
            /// `fields` holds one recipe per field of the source record, in declaration
            /// order. Binding verifies the count against the record.
            fn fields(fields: impl IntoIterator<Item = Self>) -> Self {
                Self::Branch {
                    operation: CInputBranch::Fields,
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
            fn constructor(name: impl Into<String>, args: impl IntoIterator<Item = Self>) -> Self {
                Self::Branch {
                    operation: CInputBranch::Constructor {
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
            fn selector(arms: impl IntoIterator<Item = Self>) -> Self {
                Self::Branch {
                    operation: CInputBranch::Selector,
                    children: arms.into_iter().collect(),
                }
            }
            /// Describe source variants, each with an ordinary branch of field conversions.
            fn enum_variants(variants: Vec<Vec<Self>>, value: bool) -> Self {
                Self::Branch {
                    operation: CInputBranch::Enum { value },
                    children: variants
                        .into_iter()
                        .map(|children| Self::Branch {
                            operation: CInputBranch::Variant,
                            children,
                        })
                        .collect(),
                }
            }
            /// Read an enum variant's field conversions.
            fn variant_fields(&self) -> crate::Result<&[Self]> {
                match self {
                    Self::Branch {
                        operation: CInputBranch::Variant,
                        children,
                    } => Ok(children),
                    _ => Err("enum children must be variant branches".into()),
                }
            }
            /// Convert each incoming collection element using the supplied subtree.
            fn sequence(element: Self) -> Self {
                Self::Unary {
                    operation: CInputUnary::Sequence,
                    child: Box::new(element),
                }
            }
        }
    };
}
impl_c_input_helpers!(CInput);
impl_c_input_helpers!(CInputConfig);

/// Fluent conversion construction for this language.
pub trait CInputConfigExt: Sized {
    /// Expand source enum fields with local variant/member overrides.
    fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self;
    /// Request the canonical value representation independently of expansion defaults.
    fn value() -> Self;
    /// Resolve the configured default for this source occurrence.
    fn auto() -> Self;
    /// Apply a conversion to the matching source type, including nested occurrences.
    fn for_type(ty: Type, input: Self) -> Self;
    /// Infer a source constructor's arguments from their defaults.
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
impl CInputConfigExt for CInputConfig {
    /// Expand the tag and selected fields. Keys are source variant and field
    /// names (or tuple indices). Unspecified fields inherit boundary defaults.
    fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self {
        let overrides = prebindgen_declarations::enum_overrides(overrides);
        CInputConfig::Branch {
            operation: CInputBranch::ConfigEnumFields(()),
            children: overrides
                .into_iter()
                .map(|(variant, fields)| {
                    let (fields, children) = fields.into_iter().unzip();
                    CInputConfig::Branch {
                        operation: CInputBranch::ConfigVariantFields(CVariantFields {
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
    fn value() -> Self {
        CInputConfig::Leaf(CInputLeafOp::ConfigValue(()))
    }
    /// Request this source type's configured default during preparation.
    fn auto() -> Self {
        CInputConfig::Leaf(CInputLeafOp::ConfigDefault(()))
    }
    /// Associate a conversion with a source type for a default or nested override.
    fn for_type(ty: Type, input: Self) -> Self {
        CInputConfig::Unary {
            operation: CInputUnary::ConfigForType { ty: Box::new(ty) },
            child: Box::new(input),
        }
    }
    /// Infer a source constructor's argument conversions from registered defaults.
    fn call(name: impl Into<String>) -> Self {
        CInputConfig::Leaf(CInputLeafOp::ConfigConstructor(name.into()))
    }
    /// Explicitly reconstruct an owned temporary for a shared-reference input.
    /// Slices construct a Vec, str constructs a String, and sized references
    /// construct their referent. Child recipes keep their ordinary ownership
    /// semantics; this operation authorizes the temporary's allocation/copying.
    fn reconstruct(self) -> Self {
        match self {
            CInputConfig::Unary {
                operation: CInputUnary::ConfigForType { ty },
                child: input,
            } => CInputConfig::for_type(
                *ty,
                CInputConfig::Unary {
                    operation: CInputUnary::Temporary,
                    child: input,
                },
            ),
            other => CInputConfig::Unary {
                operation: CInputUnary::Temporary,
                child: Box::new(other),
            },
        }
    }
    /// Add a constructor alternative to this conversion tree.
    fn variant(self, name: impl Into<String>) -> Self {
        self.add_alternative(CInputConfig::call(name))
    }
    /// Add a direct codec alternative, bypassing this type's expansion default.
    fn variant_self(self) -> Self {
        self.add_alternative(CInputConfig::Leaf(CInputLeafOp::Value))
    }
    fn add_alternative(self, arm: Self) -> Self {
        match self {
            CInputConfig::Unary {
                operation: CInputUnary::ConfigForType { ty },
                child: input,
            } => CInputConfig::for_type(*ty, input.add_alternative(arm)),
            CInputConfig::Branch {
                operation: CInputBranch::Selector,
                children: mut arms,
            } => {
                arms.push(arm);
                CInputConfig::Branch {
                    operation: CInputBranch::Selector,
                    children: arms,
                }
            }
            other => CInputConfig::Branch {
                operation: CInputBranch::Selector,
                children: vec![other, arm],
            },
        }
    }
}
/// Fluent conversion construction for this language.
pub trait COutputExt: Sized {
    /// Describe source record fields in declaration order.
    fn fields(fields: impl IntoIterator<Item = Self>) -> Self;
    /// Observe a source getter and convert its result.
    fn getter(name: impl Into<String>, output: Self) -> Self;
    /// Deliver multiple views of one source value.
    fn combine(parts: impl IntoIterator<Item = Self>) -> Self;
    /// Group callable argument conversions in invocation order.
    fn arguments(children: Vec<Self>) -> Self;
    /// Read the source-ordered conversions of a callable domain.
    fn argument_nodes(&self) -> crate::Result<&[Self]>;
    /// Describe source variants and their field conversions in source order.
    fn enum_variants(variants: Vec<Vec<Self>>, value: bool, mode: Option<ProjectionMode>) -> Self;
    /// Read the field conversions of one enum variant.
    fn variant_fields(&self) -> crate::Result<&[Self]>;
    /// Set the ownership contract of one callable argument.
    fn callback_argument(self, ownership: CallbackArgumentOwnership) -> Self;
    /// Read an argument's conversion and ownership contract.
    fn callback_argument_parts(&self) -> crate::Result<(CallbackArgumentOwnership, &Self)>;
}
// Both phases share construction and structural inspection of complete operations.
macro_rules! impl_c_output_helpers {
    ($node:ty) => {
        impl COutputExt for $node {
            /// Observe record fields in declaration order; use an empty list for unit.
            ///
            /// `fields` holds one recipe per field of the source record, in declaration order.
            fn fields(fields: impl IntoIterator<Item = Self>) -> Self {
                Self::Branch {
                    operation: COutputBranch::Fields,
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
            fn getter(name: impl Into<String>, output: Self) -> Self {
                Self::Unary {
                    operation: COutputUnary::Getter {
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
            fn combine(parts: impl IntoIterator<Item = Self>) -> Self {
                Self::Branch {
                    operation: COutputBranch::Combine,
                    children: parts.into_iter().collect(),
                }
            }
            /// Group a callable's domain in source parameter order.
            fn arguments(children: Vec<Self>) -> Self {
                if children.is_empty() {
                    Self::None
                } else {
                    Self::Branch {
                        operation: COutputBranch::Arguments,
                        children,
                    }
                }
            }
            /// Read the source-ordered components of a callable domain.
            fn argument_nodes(&self) -> crate::Result<&[Self]> {
                match self {
                    Self::None => Ok(&[]),
                    Self::Branch {
                        operation: COutputBranch::Arguments,
                        children,
                    } => Ok(children),
                    _ => Err("callback domain requires an argument branch or None".into()),
                }
            }
            /// Describe source variants, each with an ordinary branch of field conversions.
            fn enum_variants(
                variants: Vec<Vec<Self>>,
                value: bool,
                mode: Option<ProjectionMode>,
            ) -> Self {
                Self::Branch {
                    operation: COutputBranch::Enum { value, mode },
                    children: variants
                        .into_iter()
                        .map(|children| Self::Branch {
                            operation: COutputBranch::Variant,
                            children,
                        })
                        .collect(),
                }
            }
            /// Read an enum variant's field conversions.
            fn variant_fields(&self) -> crate::Result<&[Self]> {
                match self {
                    Self::Branch {
                        operation: COutputBranch::Variant,
                        children,
                    } => Ok(children),
                    _ => Err("enum children must be variant branches".into()),
                }
            }
            /// Attach the lifetime of produced foreign owners to this callable argument.
            fn callback_argument(self, ownership: CallbackArgumentOwnership) -> Self {
                Self::Unary {
                    operation: COutputUnary::CallbackArgument { ownership },
                    child: Box::new(self),
                }
            }
            /// Read an argument's conversion and ownership contract from its own subtree.
            fn callback_argument_parts(&self) -> crate::Result<(CallbackArgumentOwnership, &Self)> {
                match self {
                    Self::Unary {
                        operation: COutputUnary::CallbackArgument { ownership },
                        child,
                    } => Ok((*ownership, child)),
                    _ => Err("callback argument requires an explicit ownership contract".into()),
                }
            }
        }
    };
}
impl_c_output_helpers!(COutput);
impl_c_output_helpers!(COutputConfig);

/// Fluent conversion construction for this language.
pub trait COutputConfigExt: Sized {
    /// Expand source enum fields with local variant/member overrides.
    fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self;
    /// Request the canonical value representation independently of expansion defaults.
    fn value() -> Self;
    /// Resolve the configured default for this source occurrence.
    fn auto() -> Self;
    /// Apply a conversion to the matching source type, including nested occurrences.
    fn for_type(ty: Type, output: Self) -> Self;
    /// Append an explicit output view.
    fn view(self, view: impl Into<Self>) -> Self;
    /// Append a getter view using its default result conversion.
    fn field(self, name: impl Into<String>) -> Self;
    /// Append the direct value representation.
    fn field_self(self) -> Self;
    /// Append an explicit owning copy of a shared observation.
    fn field_self_cloned(self) -> Self;
    /// Append an explicitly owned projection.
    fn field_self_into(self) -> Self;
    /// Append a projection with explicit ownership and field policies.
    fn project_self(self, policy: Projection) -> Self;
    /// Append consuming decomposition of an accessor's record result.
    fn fields_into(self, view: Self) -> Self;
    /// Deprecated spelling of fields_into.
    fn fields_self_into(self, view: Self) -> Self;
    /// Override one named source field's conversion.
    fn field_override(self, name: impl Into<String>, output: Self) -> Self;
}
impl COutputConfigExt for COutputConfig {
    /// Expand an enum's tag and active fields using source variant/member keys.
    fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self {
        let overrides = prebindgen_declarations::enum_overrides(overrides);
        COutputConfig::Branch {
            operation: COutputBranch::ConfigEnumFields(()),
            children: overrides
                .into_iter()
                .map(|(variant, fields)| {
                    let (fields, children) = fields.into_iter().unzip();
                    COutputConfig::Branch {
                        operation: COutputBranch::ConfigVariantFields(CVariantFields {
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
    fn value() -> Self {
        COutputConfig::Leaf(COutputLeafOp::ConfigValue(None))
    }
    /// Request this source type's configured output default during preparation.
    fn auto() -> Self {
        COutputConfig::Leaf(COutputLeafOp::ConfigDefault(()))
    }
    /// Associate a conversion with a source type for a default or nested override.
    fn for_type(ty: Type, output: Self) -> Self {
        COutputConfig::Unary {
            operation: COutputUnary::ConfigForType { ty: Box::new(ty) },
            child: Box::new(output),
        }
    }
    /// Append another simultaneous output view.
    fn view(self, view: impl Into<Self>) -> Self {
        let view = view.into();
        match self {
            COutputConfig::Unary {
                operation: COutputUnary::ConfigForType { ty },
                child: output,
            } => COutputConfig::for_type(*ty, output.view(view)),
            COutputConfig::Branch {
                operation: COutputBranch::Combine,
                children: mut parts,
            } => {
                parts.push(view);
                COutputConfig::Branch {
                    operation: COutputBranch::Combine,
                    children: parts,
                }
            }
            other => COutputConfig::Branch {
                operation: COutputBranch::Combine,
                children: vec![other, view],
            },
        }
    }
    /// Append a source getter whose result uses its registered conversion default.
    fn field(self, name: impl Into<String>) -> Self {
        self.view(COutputConfig::getter(name, COutputConfig::auto()))
    }
    /// Append the direct codec representation, bypassing this type's expansion default.
    fn field_self(self) -> Self {
        self.view(COutputConfig::Leaf(COutputLeafOp::Value))
    }
    /// Append an explicit owning copy of a shared observation.
    fn field_self_cloned(self) -> Self {
        self.project_self(Projection::borrowed())
    }
    /// Append an explicit owned projection.
    fn field_self_into(self) -> Self {
        self.project_self(Projection::owned())
    }
    /// Append a projection with explicit ownership and nested field policies.
    fn project_self(self, policy: Projection) -> Self {
        self.view(COutputConfig::Leaf(COutputLeafOp::Projected(policy)))
    }
    /// Append a consuming accessor followed by owned field decomposition.
    fn fields_into(self, view: Self) -> Self {
        self.view(owned_field_view(view))
    }
    /// Deprecated spelling of [`COutputConfig::fields_into`].
    fn fields_self_into(self, view: Self) -> Self {
        self.fields_into(view)
    }
    /// Override one named source field, preserving defaults for its siblings.
    fn field_override(self, name: impl Into<String>, output: Self) -> Self {
        prebindgen_declarations::override_output_field(
            self,
            name.into(),
            output,
            |tree| match tree {
                COutputConfig::Branch {
                    operation: COutputBranch::ConfigNamedFields(CNamedFields { fields, .. }),
                    children,
                } => Some((fields, children)),
                _ => None,
            },
            |tree| match tree {
                COutputConfig::Unary {
                    operation:
                        COutputUnary::Getter { .. }
                        | COutputUnary::Into { .. }
                        | COutputUnary::ConfigForType { .. },
                    child,
                } => Some(child),
                _ => None,
            },
        )
    }
}
impl prebindgen_declarations::LocalConversion<CInputConfig> for CInputConfigOps {
    fn local_conversion(declaration: DeclConvert) -> CInputConfig {
        CInputConfig::for_type(
            declaration.ty.clone(),
            CInputConfig::Leaf(CInputLeafOp::ConfigConversion(Box::new(declaration))),
        )
    }
}
impl prebindgen_declarations::LocalConversion<COutputConfig> for COutputConfigOps {
    fn local_conversion(declaration: DeclConvert) -> COutputConfig {
        COutputConfig::for_type(
            declaration.ty.clone(),
            COutputConfig::Leaf(COutputLeafOp::ConfigConversion(Box::new(declaration))),
        )
    }
}

pub(super) fn owned_field_view(view: COutputConfig) -> COutputConfig {
    match view {
        COutputConfig::Branch {
            operation: COutputBranch::ConfigNamedFields(CNamedFields { fields, .. }),
            children,
        } => COutputConfig::Branch {
            operation: COutputBranch::ConfigNamedFields(CNamedFields {
                fields,
                mode: ProjectionMode::Owned,
            }),
            children,
        },
        COutputConfig::Unary {
            operation: COutputUnary::Getter { function },
            child,
        } => COutputConfig::Unary {
            operation: COutputUnary::Into { function },
            child: Box::new(owned_field_view(*child)),
        },
        _ => panic!("fields_into requires a fields! declaration"),
    }
}
