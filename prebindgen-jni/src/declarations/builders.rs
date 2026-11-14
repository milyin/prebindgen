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
        |function: DeclJniFunction| {
            prebindgen_declarations::constructor_input(
                function,
                JniInputConfig::call,
                |function, parameters| {
                    let (parameters, children) = parameters.into_iter().unzip();
                    JniInputConfig::Branch {
                        operation: JniInputBranch::ConfigConstructorArguments(
                            JniConstructorArguments {
                                function,
                                parameters,
                            },
                        ),
                        children,
                    }
                },
            )
        },
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
                |name, result| {
                    JniOutputConfig::getter(name, result.unwrap_or_else(JniOutputConfig::auto))
                },
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
/// Observe named source-record fields, optionally through an accessor.
pub fn fields(function: impl Into<Option<syn::Ident>>) -> DeclJniFields {
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
            let output = JniOutputConfig::Branch {
                operation: JniOutputBranch::ConfigNamedFields(JniNamedFields {
                    fields,
                    mode: prebindgen_declarations::ProjectionMode::Borrowed,
                }),
                children,
            };
            match function {
                Some(function) => JniOutputConfig::getter(function, output),
                None => output,
            }
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

/// Named input field conversions; unspecified fields inherit their input defaults.
pub type DeclJniParamFields = prebindgen_declarations::DeclFields<JniInputConfig>;
/// Start source-record or enum-variant input field declarations.
pub fn param_fields() -> DeclJniParamFields {
    prebindgen_declarations::DeclFields::new(
        None,
        |input| input,
        |_function, overrides, _extension| {
            let (fields, children) = overrides.into_iter().unzip();
            JniInputConfig::Branch {
                operation: JniInputBranch::ConfigNamedFields(JniInputFields {
                    fields,
                    record: false,
                }),
                children,
            }
        },
    )
}

/// Input representations supported by the JNI declaration frontend.
/// All choices lower into the same source resolver and converter compiler.
/// ```
/// use prebindgen_jni::{callback,expand_param,fun,param_fields,prelude::*};
/// let rows=expand_param!(Vec<Row>).element(expand_param!(Row).variant_record(
///     param_fields!().field("observed",expand_param!(Handle).variant(fun!(copy_handle))))).bulk();
/// let _=fun!(accept).expand_param("rows",rows.clone());
/// let _=callback!(impl Fn()->Vec<Row>).expand_return(rows);
/// ```
pub trait JniInputExpansion: Sized {
    /// Resolve this occurrence using its configured type default.
    fn default(self) -> Self;
    /// Receive a declared Kotlin data class, explicitly constructing its owned fields.
    /// Owned handle fields are consumed. Copying requires an explicit constructor helper.
    fn variant_record(self, fields: DeclJniParamFields) -> Self;
    /// Convert a collection using this occurrence-local element declaration.
    fn element(self, input: impl Into<JniInputConfig>) -> Self;
    /// Convert an Option payload with an occurrence-local declaration.
    fn optional(self, input: impl Into<JniInputConfig>) -> Self;
    /// Expand an enum into tag and field arguments, inheriting unspecified field defaults.
    fn enum_fields<'a>(
        self,
        variants: impl IntoIterator<Item = (&'a str, DeclJniParamFields)>,
    ) -> Self;
    /// Receive a declared Kotlin sealed value with named variant-field overrides.
    fn enum_value<'a>(
        self,
        variants: impl IntoIterator<Item = (&'a str, DeclJniParamFields)>,
    ) -> Self;
}
impl JniInputExpansion for DeclJniExpandParam {
    fn default(self) -> Self {
        self.variant_with(JniInputConfig::auto())
    }
    fn variant_record(self, fields: DeclJniParamFields) -> Self {
        let (fields, children) = fields.into_overrides().into_iter().unzip();
        self.variant_with(JniInputConfig::Branch {
            operation: JniInputBranch::ConfigNamedFields(JniInputFields {
                fields,
                record: true,
            }),
            children,
        })
    }
    fn element(self, input: impl Into<JniInputConfig>) -> Self {
        self.variant_with(JniInputConfig::sequence(input.into()))
    }
    fn optional(self, input: impl Into<JniInputConfig>) -> Self {
        self.variant_with(JniInputConfig::Unary {
            operation: JniInputUnary::Optional,
            child: Box::new(input.into()),
        })
    }
    fn enum_fields<'a>(
        self,
        variants: impl IntoIterator<Item = (&'a str, DeclJniParamFields)>,
    ) -> Self {
        self.variant_with(input_enum_fields(variants, false))
    }
    fn enum_value<'a>(
        self,
        variants: impl IntoIterator<Item = (&'a str, DeclJniParamFields)>,
    ) -> Self {
        self.variant_with(input_enum_fields(variants, true))
    }
}
fn input_enum_fields<'a>(
    variants: impl IntoIterator<Item = (&'a str, DeclJniParamFields)>,
    value: bool,
) -> JniInputConfig {
    let overrides = prebindgen_declarations::enum_overrides(
        variants
            .into_iter()
            .map(|(name, fields)| (name, fields.into_overrides())),
    );
    JniInputConfig::Branch {
        operation: JniInputBranch::ConfigEnumFields(value),
        children: overrides
            .into_iter()
            .map(|(variant, fields)| {
                let (fields, children) = fields.into_iter().unzip();
                JniInputConfig::Branch {
                    operation: JniInputBranch::ConfigVariantFields(JniVariantFields {
                        variant,
                        fields,
                    }),
                    children,
                }
            })
            .collect(),
    }
}

/// Output representations supported by the JNI declaration frontend.
/// ```
/// use prebindgen_jni::{expand_return,fields,fun,prelude::*};
/// let _=expand_return!(Owner).field(fun!(child)
///     .expand_return(expand_return!(Child).field_self_cloned()));
/// let _=expand_return!(Row).fields_into(fields!()
///     .field("count",expand_return!(i64).field_self()));
/// let _=expand_return!(Vec<Row>).element(expand_return!(Row).field_self());
/// let _=expand_return!(Vec<Row>).element_objects(expand_return!(Row).field_self());
/// ```
pub trait JniOutputExpansion: Sized {
    /// Resolve this occurrence using its configured type default.
    fn default(self) -> Self;
    /// Convert each returned element, retaining the configured default collection transport.
    /// Apply bulk to this declaration or its element to select a packed level.
    fn element(self, output: impl Into<JniOutputConfig>) -> Self;
    /// Return an ordinary Kotlin object collection with the selected element conversion.
    /// This explicitly opts out of automatic bulk packing at this collection level.
    fn element_objects(self, output: impl Into<JniOutputConfig>) -> Self;
    /// Return an Option payload with an occurrence-local declaration.
    fn optional(self, output: impl Into<JniOutputConfig>) -> Self;
    /// Expand an enum into tag and field results, inheriting unspecified field defaults.
    fn enum_fields<'a>(self, variants: impl IntoIterator<Item = (&'a str, DeclJniFields)>) -> Self;
    /// Return a declared Kotlin sealed value with named variant-field overrides.
    fn enum_value<'a>(self, variants: impl IntoIterator<Item = (&'a str, DeclJniFields)>) -> Self;
}
impl JniOutputExpansion for DeclJniExpandReturn {
    fn default(self) -> Self {
        self.view(JniOutputConfig::auto())
    }
    fn element(self, output: impl Into<JniOutputConfig>) -> Self {
        self.view(JniOutputConfig::Unary {
            operation: JniOutputUnary::Sequence {
                delivery: SequenceDelivery::Auto,
                mode: None,
            },
            child: Box::new(output.into()),
        })
    }
    fn element_objects(self, output: impl Into<JniOutputConfig>) -> Self {
        self.view(JniOutputConfig::Unary {
            operation: JniOutputUnary::Sequence {
                delivery: SequenceDelivery::Value,
                mode: None,
            },
            child: Box::new(output.into()),
        })
    }
    fn optional(self, output: impl Into<JniOutputConfig>) -> Self {
        self.view(JniOutputConfig::Unary {
            operation: JniOutputUnary::Optional,
            child: Box::new(output.into()),
        })
    }
    fn enum_fields<'a>(self, variants: impl IntoIterator<Item = (&'a str, DeclJniFields)>) -> Self {
        self.view(output_enum_fields(variants, false))
    }
    fn enum_value<'a>(self, variants: impl IntoIterator<Item = (&'a str, DeclJniFields)>) -> Self {
        self.view(output_enum_fields(variants, true))
    }
}
fn output_enum_fields<'a>(
    variants: impl IntoIterator<Item = (&'a str, DeclJniFields)>,
    value: bool,
) -> JniOutputConfig {
    let overrides = prebindgen_declarations::enum_overrides(
        variants
            .into_iter()
            .map(|(name, fields)| (name, fields.into_overrides())),
    );
    JniOutputConfig::Branch {
        operation: JniOutputBranch::ConfigEnumFields(value),
        children: overrides
            .into_iter()
            .map(|(variant, fields)| {
                let (fields, children) = fields.into_iter().unzip();
                JniOutputConfig::Branch {
                    operation: JniOutputBranch::ConfigVariantFields(JniVariantFields {
                        variant,
                        fields,
                    }),
                    children,
                }
            })
            .collect(),
    }
}
