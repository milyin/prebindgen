//! JNI slot converters own Kotlin conversion, locking and rollback behavior.
use super::*;
use crate::group::validate_layout_in;
use prebindgen_declarations::{ComponentPath, ComponentSegment};

use prebindgen_conversions::{Conversion, Wire};

use prebindgen_conversions::{InputConversion, OutputConversion};

/// Native transport selected for an ordinary function result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JniReturnDelivery {
    /// An object-array result packet with status/error framing.
    Packet,
    /// A unit or one-wire native return; errors use the supplied capture.
    Direct,
    /// Success fields delivered through the supplied synchronous capture.
    Capture,
}
/// Function-return reconstruction, including the selected native transport.
/// Custom converters can override kotlin_return or reuse project for the
/// canonical packet-relative projection produced by kotlin.
pub struct KotlinReturnContext<'a> {
    /// Names, classes and cleanup bindings for this wrapper.
    pub kotlin: &'a KotlinContext<'a>,
    /// Explicit native return protocol selected for this call.
    pub delivery: JniReturnDelivery,
    /// Native/capture expressions in successful wire order; empty for packets.
    pub values: &'a [String],
    /// Prepared row projection for a callback-sequence return, if applicable.
    pub sequence: Option<&'a KotlinOutput>,
}
impl KotlinReturnContext<'_> {
    /// Place a canonical Kotlin projection in the selected function-return ABI.
    pub fn project(
        &self,
        outcomes: &prebindgen_conversions::Outcomes<JniOutputWireNode>,
        mut output: KotlinOutput,
    ) -> Result<KotlinOutput> {
        let fields = crate::group::result_packet_fields(&outcomes.success).collect::<Vec<_>>();
        if self.delivery != JniReturnDelivery::Packet && fields.len() != self.values.len() {
            return Err("native return values do not match successful output fields".into());
        }
        remap_output(
            &mut output,
            self.kotlin.result,
            self.kotlin.result,
            1,
            fields.len(),
        );
        if self.delivery != JniReturnDelivery::Packet {
            returns::remap(&mut output, self.kotlin.result, &fields, 1, self.values);
        }
        Ok(output)
    }
}
/// Names and class settings supplied by the Kotlin wrapper writer.
pub struct KotlinContext<'a> {
    /// Root package of this binding module and its bulk runtime.
    pub package: &'a str,
    /// Canonical source declarations and JNI type/field metadata, borrowed from Flat.
    pub source_types: &'a BTreeMap<String, prebindgen_flat::TypeInfo<JniType, JniField>>,
    /// Inherited packed encoding, set by an enclosing input/output Bulk chain while
    /// interpreting its descendants. Start ordinary call/callback boundaries false.
    pub bulk: bool,
    /// Variable holding the native output array.
    pub result: &'a str,
    /// Caught delivery exception to which rollback failures are attached.
    pub failure: &'a str,
    /// Optional mutable nullable Throwable collecting cleanup failures in a finally
    /// block. Without one, cleanup failures are suppressed on `failure`.
    pub cleanup_failure: Option<&'a str>,
    /// Prefix for variables retaining materialized outputs during rollback.
    pub output_prefix: &'a str,
}
impl KotlinContext<'_> {
    /// Reconstruct the element of a sequence-callback result using its declared
    /// JNI layout. Custom element projections can wrap the returned expressions
    /// while preserving its ownership and cleanup code.
    pub fn sequence_element(
        &self,
        outcomes: &prebindgen_conversions::Outcomes<JniOutputWireNode>,
        owners: Option<&str>,
    ) -> Result<KotlinOutput> {
        let (_, element, _) = crate::sequence::return_sequence(&outcomes.success)
            .ok_or("missing sequence return layout")?;
        output_kotlin_scoped(element, self, owners)
    }
    /// Resolve the configured Kotlin class name for a source type.
    pub fn class_name(&self, ty: &syn::Type) -> Result<String> {
        Ok(crate::metadata::source_handle(self.source_types, ty)?
            .1
            .fqn())
    }
    /// Emit the configured native destructor invocation for an unwrapped handle.
    pub fn drop_handle(&self, ty: &syn::Type, value: &str) -> Result<String> {
        let (ty, _) = crate::metadata::source_handle(self.source_types, ty)?;
        let method = crate::handles::drop_method(ty, &mut names::Names::default())?;
        Ok(format!("{}.JNINative.{method}({value})", self.package))
    }
}
/// A prepared local binding, evaluated before collecting handle locks.
#[derive(Clone, Debug)]
pub struct KotlinBinding {
    /// Generated local name.
    pub name: String,
    /// Expression evaluated once to snapshot input collection structure.
    pub expression: String,
    /// Inactive expression when an enclosing optional/alternative suppresses it.
    pub absent: String,
}
/// One handle input, or a homogeneous list of handle inputs, retained before JNI.
/// Its allocation type and access mode let the wrapper reuse lock ordering for
/// alias validation without inspecting the native payload a second time.
#[derive(Clone, Debug)]
pub struct KotlinHandleInput {
    /// Source allocation type, resolved to its generated owning handle class.
    pub ty: syn::Type,
    /// Borrow or ownership requirement for every value in this source.
    pub access: Access,
    /// Kotlin expression selecting a nullable handle or a list of handles.
    pub value: String,
    /// True when value is a homogeneous list, false for an individual handle.
    pub many: bool,
    /// Native argument index for an individual handle; carrier lists use None.
    pub argument: Option<usize>,
    /// Original direct ownership update, replaced with the retained handle.
    /// Carrier ownership remains in its original wire traversal order.
    pub consumed: Option<String>,
}
/// Prepared Kotlin input code; the writer orders these operations across slots.
#[derive(Default)]
pub struct KotlinInput {
    /// Complete handle inputs used for Kotlin alias preflight. Custom input
    /// emission must provide this metadata; None is rejected during generation.
    pub handle_inputs: Option<Vec<KotlinHandleInput>>,
    /// Input snapshots evaluated once before locking; handle addresses stay deferred.
    pub setup: Vec<KotlinBinding>,
    /// Lists of handles to combine with individual locks.
    pub lock_groups: Vec<String>,
    /// Public parameter declarations.
    pub parameters: Vec<String>,
    /// Relative constructor-argument/record-field paths for public parameters,
    /// in the same order as `parameters`. Either empty (no naming hints), or
    /// exactly one path per parameter. Empty paths use positional fallback names.
    pub parameter_paths: Vec<ComponentPath>,
    /// Expressions passed to native wires, in wire order.
    pub arguments: Vec<String>,
    /// Values cached when preparing collection rows, before extracting handle
    /// addresses under locks. Handle fields retain their nullable wrapper objects;
    /// other fields retain their encoded wire values. Nested collections retain
    /// their prepared state. An empty vector falls back to arguments for
    /// resource-free converters; handle-bearing converters must provide this form.
    pub prepared_arguments: Vec<String>,
    /// Handle expressions to lock for the duration of the native call.
    pub locks: Vec<String>,
    /// Statements visiting owned handles in native acquisition order. Use
    /// `__pb_consume(handle)` so the writer can invalidate either all owners
    /// on success or only the consumed prefix and its aliases on failure.
    pub consumed: Vec<String>,
}
impl KotlinInput {
    pub(crate) fn complete_parameter_paths(&mut self) -> Result<()> {
        if !self.parameter_paths.is_empty() && self.parameter_paths.len() != self.parameters.len() {
            return Err("Kotlin parameter naming paths must match public parameters".into());
        }
        self.parameter_paths
            .resize(self.parameters.len(), ComponentPath::default());
        Ok(())
    }
    pub(crate) fn prepare_arguments(&mut self, handles: bool) -> Result<()> {
        if self.prepared_arguments.is_empty() && !self.arguments.is_empty() {
            if handles {
                return Err(
                    "handle-bearing Kotlin input converter must provide prepared_arguments".into(),
                );
            }
            self.prepared_arguments = self.arguments.clone();
        }
        if self.prepared_arguments.len() != self.arguments.len() {
            return Err("Kotlin prepared input values differ from its wire argument count".into());
        }
        Ok(())
    }
}
/// Kotlin result conversion and rollback code owned by the result slot.
#[derive(Default, Clone)]
pub struct KotlinOutput {
    /// Expressions delivered to the caller, in result order.
    pub delivered: Vec<String>,
    /// Public Kotlin result types.
    pub result_types: Vec<String>,
    /// Explicit destination names, aligned with logical results; absent names use positional defaults.
    pub result_names: Vec<Option<String>>,
    /// Declarations retaining partially materialized outputs.
    pub output_locals: String,
    /// Statements materializing outputs before delivery.
    pub materialize: String,
    /// Rollback statements executed if materialization or delivery fails.
    pub cleanup: String,
}
impl KotlinOutput {
    fn checked_names(&self) -> Result<Vec<String>> {
        if self.result_names.len() > self.result_types.len() {
            return Err("more output names than result values".into());
        }
        let mut seen = BTreeSet::new();
        (0..self.result_types.len())
            .map(|i| {
                let name = self
                    .result_names
                    .get(i)
                    .and_then(|name| name.clone())
                    .unwrap_or_else(|| format!("v{i}"));
                validate_name(&name)?;
                if !seen.insert(name.clone()) {
                    return Err(format!("duplicate output parameter name `{name}`"));
                }
                Ok(name)
            })
            .collect()
    }
    /// Render the parameters of a Kotlin build/fold function type, preserving explicit names.
    pub fn callback_type_parameters(&self) -> Result<String> {
        let names = self.checked_names()?;
        Ok(self
            .result_types
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                if self.result_names.get(i).is_some_and(|name| name.is_some()) {
                    format!("{}: {ty}", kotlin_identifier(&names[i]))
                } else {
                    ty.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", "))
    }
    /// Render named arguments for a generated callback interface method.
    pub fn callback_parameter_declarations(&self) -> Result<String> {
        let names = self.checked_names()?;
        Ok(self
            .result_types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("{}: {ty}", kotlin_identifier(&names[i])))
            .collect::<Vec<_>>()
            .join(", "))
    }
}

/// Input conversion with opaque Kotlin-side emission.
/// Also implements the native input contract so both sides describe the same
/// wire tree. `kotlin` is called once per bound parameter during generation.
pub trait KotlinInputConversion:
    for<'a> InputConversion<
    Wire = JniWire,
    Chain = JniInputWireChain,
    Branch = JniInputWireBranch,
    SlotFragment<'a> = prebindgen_conversions::InputFragment<'a, JniContext>,
>
{
    /// Emit destination-side input conversion for this slot.
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput>;
    /// Emit callback-return construction using native bulk carriers already read
    /// by the shared callback boundary. Independent inputs may reuse ordinary
    /// construction; plan-based inputs reuse the cached carrier bindings.
    fn callback_return_fragment(
        &self,
        request: prebindgen_conversions::InputFragment<'_, JniContext>,
        expected: &syn::Type,
        cached_buffers: bool,
    ) -> Result<TokenStream> {
        if cached_buffers
            && !crate::bulk::callback_cache(self.input_node(), request.context)?.is_empty()
        {
            return Err("custom callback-return input must implement callback_return_fragment to reuse the native buffer cache".into());
        }
        self.code_fragment_slot(request, expected)
    }
}
/// Output conversion with opaque Kotlin-side emission.
pub trait KotlinOutputConversion:
    for<'a> OutputConversion<
    Wire = JniWire,
    Chain = JniOutputWireChain,
    Branch = JniOutputWireBranch,
    SlotFragment<'a> = prebindgen_conversions::OutputFragment<'a, JniContext>,
>
{
    /// Emit destination-side output conversion and rollback for this slot.
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput>;
    /// Allow the adapter to select direct/capture delivery, or preserve an
    /// explicitly fixed packet protocol. Custom converters default to fixed.
    fn delivery_policy(&self) -> prebindgen_conversions::DeliveryPolicy {
        prebindgen_conversions::DeliveryPolicy::Fixed
    }
    /// Prepare the selected function-return delivery once, before requesting
    /// Rust output fragments. None preserves this converter's fixed protocol.
    fn prepare_return_delivery(
        &self,
        _context: &JniContext,
    ) -> Result<Option<prebindgen_conversions::Delivery>> {
        if self.delivery_policy() == prebindgen_conversions::DeliveryPolicy::AdapterSelected {
            return Err("adapter-selected output must implement prepare_return_delivery".into());
        }
        Ok(None)
    }
    /// Rust wire expressions available inside this converter's Prepare scope
    /// when a retained source callback delivers its arguments.
    fn callback_arguments(
        &self,
        _context: &JniContext,
        _source: &syn::Ident,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        Err("output converter does not implement callback argument delivery".into())
    }
    /// Reconstruct one element delivered through a function's sequence callback.
    /// `owners` is an existing nested-owner journal, when the shared boundary
    /// needs one. Reuse it so partial-row and unread-row cleanup stay coordinated.
    fn kotlin_sequence_element(
        &self,
        context: &KotlinContext<'_>,
        owners: Option<&str>,
    ) -> Result<KotlinOutput> {
        context.sequence_element(self.output_outcomes(), owners)
    }
    /// Reconstruct an ordinary function result in its selected native transport.
    /// Callback-sequence return captures supply their prepared row projection.
    fn kotlin_return(&self, context: &KotlinReturnContext<'_>) -> Result<KotlinOutput> {
        let output = match context.sequence {
            Some(output) => output.clone(),
            None => self.kotlin(context.kotlin)?,
        };
        context.project(self.output_outcomes(), output)
    }
}
/// Prepared JNI input, with a logical schema and a native plan shared by Rust/Kotlin emission.
#[derive(Clone, Debug)]
pub(crate) struct PlanInput {
    pub(crate) rust: JniInputPlan<JniWire>,
}
/// Prepared JNI output, with logical outcomes and a native plan shared by Rust/Kotlin emission.
#[derive(Clone, Debug)]
pub(crate) struct PlanOutput {
    pub(crate) rust: JniOutputPlan<JniWire>,
}
impl Conversion for PlanInput {
    type Wire = JniWire;
    type Chain = JniInputWireChain;
    type Branch = JniInputWireBranch;
    type SlotFragment<'a> = prebindgen_conversions::InputFragment<'a, JniContext>;

    fn code_fragment_slot(
        &self,
        mode: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.rust.code_fragment_slot(mode, expected)
    }
}

impl Conversion for PlanOutput {
    type Wire = JniWire;
    type Chain = JniOutputWireChain;
    type Branch = JniOutputWireBranch;
    type SlotFragment<'a> = prebindgen_conversions::OutputFragment<'a, JniContext>;

    fn code_fragment_slot(
        &self,
        mode: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        let context = match &mode {
            prebindgen_conversions::OutputFragment::Prepare { context, .. }
            | prebindgen_conversions::OutputFragment::Deliver { context, .. }
            | prebindgen_conversions::OutputFragment::Commit { context, .. }
            | prebindgen_conversions::OutputFragment::Result { context, .. } => *context,
        };
        if let Some(delivery) = &context.prepared_delivery {
            return self.rust.emit_with_delivery(mode, delivery);
        }
        self.rust.code_fragment_slot(mode, expected)
    }
}

impl KotlinInputConversion for PlanInput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput> {
        crate::sequence::validate_buffered(self.input_node())?;
        validate_layout_in(self.input_node(), context.bulk)?;
        input_kotlin(self.input_node(), context)
    }
    fn callback_return_fragment(
        &self,
        request: prebindgen_conversions::InputFragment<'_, JniContext>,
        expected: &syn::Type,
        cached_buffers: bool,
    ) -> Result<TokenStream> {
        if cached_buffers {
            let code = request
                .context
                .cached_callback_validation(&self.rust.code, self.input_node())?;
            return self.rust.emit_code(request, &code);
        }
        self.code_fragment_slot(request, expected)
    }
}
pub(crate) fn input_leaves(
    wires: &[&Wire<JniWire>],
    context: &KotlinContext<'_>,
    active: &str,
) -> Result<KotlinInput> {
    let conditional = active != "true";
    let mut kt_params = Vec::new();
    let mut kt_args = Vec::new();
    let mut locks = Vec::new();
    let mut handle_inputs = Vec::new();
    let mut prepared_arguments = Vec::new();
    let mut consumed = Vec::new();
    for w in wires {
        let n = &w.name;
        let (kt, arg) = match &w.kind {
            JniWire::Callback { callback } => {
                let adapter = callback.adapter(&n.to_string(), context, conditional)?;
                (
                    format!("{}{}", callback.fqn(), if conditional { "?" } else { "" }),
                    if conditional {
                        format!("if ({active}) {adapter} else null")
                    } else {
                        adapter
                    },
                )
            }
            JniWire::Native128 { primitive: p, .. }
            | JniWire::Scalar {
                primitive: p @ (Primitive::I128 | Primitive::U128),
                ..
            } => {
                let value = if conditional {
                    format!("requireNotNull({n})")
                } else {
                    n.to_string()
                };
                let encoded = format!("{value}.toByteArray()");
                let encoded = if conditional {
                    format!("if ({active}) {encoded} else null")
                } else {
                    encoded
                };
                (
                    format!(
                        "{}{}",
                        scalar::kotlin(*p),
                        if conditional { "?" } else { "" }
                    ),
                    encoded,
                )
            }
            JniWire::Scalar { primitive: p, .. } => scalar::input(*p, &n.to_string()),
            JniWire::Enum { enum_type, .. } => {
                let member = enum_type.value_member();
                (
                    format!(
                        "{}{}",
                        enum_type.class_name,
                        if conditional { "?" } else { "" }
                    ),
                    if conditional {
                        format!("if ({active}) requireNotNull({n}).{member} else 0")
                    } else {
                        format!("{n}.{member}")
                    },
                )
            }
            JniWire::Handle {
                handle, condition, ..
            } => {
                let value = if conditional {
                    format!("requireNotNull({n}) {{ \"null handle\" }}")
                } else {
                    n.to_string()
                };
                locks.push(if condition == "true" {
                    value.clone()
                } else {
                    format!("if ({condition}) {value} else null")
                });
                handle_inputs.push(KotlinHandleInput {
                    ty: handle.ty.clone(),
                    access: handle.access,
                    value: locks.last().unwrap().clone(),
                    many: false,
                    argument: Some(kt_args.len()),
                    consumed: None,
                });
                if handle.access == Access::Move {
                    let consume = format!("__pb_consume({value})");
                    consumed.push(if condition == "true" {
                        consume
                    } else {
                        format!("if ({condition}) {consume}")
                    });
                    handle_inputs.last_mut().unwrap().consumed = consumed.last().cloned();
                }
                let class_name = context.class_name(&handle.ty).map_err(|error| {
                    if handle.access == Access::Exclusive {
                        format!("mutable JNI input `&mut {}` requires a native handle class; a value representation cannot preserve mutation and implicit copy-back is unsupported", type_key(&handle.ty))
                    } else {
                        error
                    }
                })?;
                (
                    format!("{}{}", class_name, if conditional { "?" } else { "" }),
                    if conditional {
                        format!("if ({active}) {value}.raw else 0L")
                    } else {
                        format!("{n}.raw")
                    },
                )
            }
            JniWire::NativeRows { .. } | JniWire::Rows { .. } => {
                return Err("array carrier requires a sequence node".into())
            }
            JniWire::NativeBuffer { text, .. } => (
                format!(
                    "{}{}",
                    if *text { "String" } else { "ByteArray" },
                    if conditional { "?" } else { "" }
                ),
                n.to_string(),
            ),
            _ => (
                if conditional {
                    w.kind.kotlin_type().into()
                } else {
                    w.kind.kotlin_type().trim_end_matches('?').into()
                },
                n.to_string(),
            ),
        };
        prepared_arguments.push(match &w.kind {
            JniWire::Handle { condition, .. } if condition != "true" => {
                format!("if ({condition}) {n} else null")
            }
            JniWire::Handle { .. } => n.to_string(),
            _ => arg.clone(),
        });
        kt_params.push(format!("{n}: {kt}"));
        kt_args.push(arg);
    }
    Ok(KotlinInput {
        handle_inputs: Some(handle_inputs),
        prepared_arguments,
        parameters: kt_params,
        parameter_paths: (0..wires.len())
            .map(|i| {
                ComponentPath(if wires.len() == 1 {
                    vec![]
                } else {
                    vec![ComponentSegment::Named(format!("value{i}"))]
                })
            })
            .collect(),
        arguments: kt_args,
        setup: Vec::new(),
        lock_groups: Vec::new(),
        locks,
        consumed,
    })
}
impl KotlinOutputConversion for PlanOutput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
        validate_layout_in(&self.output_outcomes().success, context.bulk)?;
        output_kotlin(&self.output_outcomes().success, context)
    }
    fn delivery_policy(&self) -> prebindgen_conversions::DeliveryPolicy {
        self.rust.delivery.policy
    }
    fn prepare_return_delivery(
        &self,
        context: &JniContext,
    ) -> Result<Option<prebindgen_conversions::Delivery>> {
        if self.delivery_policy() == prebindgen_conversions::DeliveryPolicy::Fixed {
            return Ok(None);
        }
        Ok(Some(context.output_delivery(
            self.output_outcomes(),
            &self.rust.values,
        )?))
    }
    fn callback_arguments(
        &self,
        context: &JniContext,
        source: &syn::Ident,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        self.rust
            .values
            .iter()
            .map(|(name, value)| {
                prebindgen_conversions::render_output(context, value, source)
                    .map(|value| (name.clone(), value))
            })
            .collect()
    }
}
pub(crate) fn output_leaves(
    wires: &[&Wire<JniWire>],
    context: &KotlinContext<'_>,
    owners: Option<&str>,
) -> Result<KotlinOutput> {
    let result = context.result;
    let suppress_cleanup = record_cleanup_failure(context, "cleanup");
    let output = context.output_prefix;
    let mut delivered = Vec::new();
    let mut result_types = Vec::new();
    for (i, w) in wires.iter().enumerate() {
        let (kt, decode) = match &w.kind {
            JniWire::Callback { .. } => return Err("callback objects are input-only".into()),
            JniWire::Native128 { primitive: p, .. } | JniWire::Scalar { primitive: p, .. } => {
                scalar::output_kotlin(*p, i, result)
            }
            JniWire::Handle { handle, .. } => {
                let c = context.class_name(&handle.ty)?;
                (c.clone(), format!("{c}({result}[{i}] as Long)"))
            }
            JniWire::NativeBuffer { text: true } | JniWire::Text { .. } => {
                ("String".into(), format!("{result}[{i}] as String"))
            }
            JniWire::NativeBuffer { text: false }
            | JniWire::BulkBuffer { .. }
            | JniWire::Bytes { .. } => ("ByteArray".into(), format!("{result}[{i}] as ByteArray")),
            JniWire::NativeRows { .. } | JniWire::Rows { .. } => {
                return Err("array carrier requires a sequence node".into())
            }
            JniWire::Selector { .. } => ("Int".into(), format!("{result}[{i}] as Int")),
            JniWire::Enum { enum_type, .. } => (
                enum_type.class_name.clone(),
                format!(
                    "{}.{}({result}[{i}] as Int)",
                    enum_type.class_name,
                    enum_type.decode_member()
                ),
            ),
        };
        delivered.push(decode);
        result_types.push(kt);
    }
    let mut output_locals = String::new();
    let mut materialize = String::new();
    let mut cleanup = String::new();
    for (i, w) in wires.iter().enumerate() {
        if let JniWire::Handle { handle, .. } = &w.kind {
            let c = context.class_name(&handle.ty)?;
            output_locals.push_str(&format!("        var {output}{i}: {c}? = null\n"));
            let value = if let Some(owners) = owners {
                format!("requireNotNull({owners}).own({})", delivered[i])
            } else {
                delivered[i].clone()
            };
            materialize.push_str(&format!("            {output}{i} = {value}\n"));
            delivered[i] = format!("requireNotNull({output}{i})");
            let drop = context.drop_handle(&handle.ty, &format!("{result}[{i}] as Long"))?;
            let release = if let Some(owners) = owners {
                // Materialization and cleanup visit active handles in the same
                // schema order. Only unwrapped owners still belong to the wire.
                format!("if ({owners}?.unread() != false) {{ {drop} }}")
            } else {
                format!("val owner = {output}{i}; if (owner != null) owner.close() else {drop}")
            };
            cleanup.push_str(&format!("            try {{ {release} }} catch (cleanup: Throwable) {{ {suppress_cleanup} }}\n"));
        }
    }
    Ok(KotlinOutput {
        result_names: vec![None; result_types.len()],
        delivered,
        result_types,
        output_locals,
        materialize,
        cleanup,
    })
}

/// Record a cleanup error without replacing the delivery exception or suppressing itself.
pub(crate) fn record_cleanup_failure(context: &KotlinContext<'_>, error: &str) -> String {
    match context.cleanup_failure {
        Some(target) => format!(
            "if ({target} == null) {target} = {error} else if ({target} !== {error}) {target}!!.addSuppressed({error})"
        ),
        None => format!("if ({} !== {error}) {}.addSuppressed({error})", context.failure, context.failure),
    }
}

pub(crate) fn leaf_wires<Chain: group::ChainMetadata, Branch>(
    node: &JniWireNode<Chain, Branch>,
) -> Vec<&Wire<JniWire>> {
    crate::group::kotlin_result_fields(node).collect()
}
pub(crate) fn has_optional<Chain: group::ChainMetadata, Branch: group::BranchMetadata>(
    node: &JniWireNode<Chain, Branch>,
) -> bool {
    match node {
        prebindgen_conversions::WireNode::Leaf(_) => false,
        prebindgen_conversions::WireNode::Chain { meta, child, .. } => {
            !meta.is_scope_boundary() && (meta.optional().is_some() || has_optional(child))
        }
        prebindgen_conversions::WireNode::Branch { meta, children, .. } => {
            meta.record().is_none() && children.iter().any(has_optional)
        }
    }
}
pub(crate) fn lock_list(locks: &[String], groups: &[String]) -> String {
    let mut groups = groups.iter();
    let mut out = if locks.is_empty() {
        groups
            .next()
            .map(|group| format!("({group})"))
            .unwrap_or_else(|| "kotlin.collections.emptyList<Nothing>()".into())
    } else {
        format!("kotlin.collections.listOfNotNull({})", locks.join(", "))
    };
    for group in groups {
        out = format!("({out} + ({group}))");
    }
    out
}

pub(crate) fn has_handles<Chain, Branch>(
    node: &JniWireNode<Chain, Branch>,
    owned_only: bool,
) -> bool {
    match node {
        prebindgen_conversions::WireNode::Leaf(wires) => wires.iter().any(|w|matches!(&w.kind,JniWire::Handle{handle,..} if !owned_only || handle.access == Access::Move)),
        prebindgen_conversions::WireNode::Chain { child, .. } => has_handles(child, owned_only),
        prebindgen_conversions::WireNode::Branch { controls, children, .. } => controls.iter().any(|w|matches!(&w.kind,JniWire::Handle{handle,..} if !owned_only || handle.access == Access::Move)) || children.iter().any(|n|has_handles(n,owned_only)),
    }
}

pub(crate) fn input_kotlin(
    node: &JniInputWireNode,
    context: &KotlinContext<'_>,
) -> Result<KotlinInput> {
    input_kotlin_when(node, context, "true")
}
// Availability is local to the exposed value: sequence rows start a new scope,
// while optional payloads and selector arms can be absent. JNI wire types stay
// nullable independently of this public Kotlin contract.
pub(crate) fn input_kotlin_when(
    node: &JniInputWireNode,
    context: &KotlinContext<'_>,
    active: &str,
) -> Result<KotlinInput> {
    let mut out = match node {
        prebindgen_conversions::WireNode::Branch {
            meta: JniInputWireBranch::Enum(group),
            children,
            controls,
        } => crate::sealed::input(children, controls, group, context, active),
        prebindgen_conversions::WireNode::Leaf(wires) => {
            input_leaves(&wires.iter().collect::<Vec<_>>(), context, active)
        }
        prebindgen_conversions::WireNode::Chain {
            meta: JniInputWireChain::Bulk,
            controls,
            child,
        } => crate::bulk::input_kotlin(controls, child, context, active),
        prebindgen_conversions::WireNode::Branch {
            meta: JniInputWireBranch::Alternatives,
            children: elements,
            controls,
            ..
        } => {
            let mut out = input_leaves(&controls.iter().collect::<Vec<_>>(), context, active)?;
            out.parameter_paths = controls
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    ComponentPath(vec![if controls.len() == 1 {
                        ComponentSegment::Selector
                    } else {
                        ComponentSegment::Named(format!("variant{i}"))
                    }])
                })
                .collect();
            for (i, arm) in elements.iter().enumerate() {
                let selector = controls.first().ok_or("alternatives require a selector")?;
                let selected = format!("{} == {i}", selector.name);
                let selected = if active == "true" {
                    selected
                } else {
                    format!("({active}) && ({selected})")
                };
                let mut child = input_kotlin_when(arm, context, &selected)?;
                for path in &mut child.parameter_paths {
                    path.prepend(ComponentSegment::Alternative(i));
                }
                merge_input(&mut out, child)?;
            }
            Ok(out)
        }
        prebindgen_conversions::WireNode::Chain {
            controls,
            child: element,
            meta: JniInputWireChain::Sequence,
            ..
        } => {
            let [carrier] = controls.as_slice() else {
                return Err("sequence requires one carrier control".into());
            };
            crate::sequence::input_list(carrier, element, context, active)
        }
        prebindgen_conversions::WireNode::Chain {
            child: element,
            meta: JniInputWireChain::Optional(metadata),
            ..
        } => optional::input_kotlin(element, metadata, context, active),
        prebindgen_conversions::WireNode::Branch {
            meta: JniInputWireBranch::Record(record),
            children,
            ..
        } => data::input_kotlin(children, record, context, active),
        prebindgen_conversions::WireNode::Branch {
            meta: JniInputWireBranch::Argument(name),
            children,
            ..
        } => {
            let mut out = input_components_when(children, context, active)?;
            for path in &mut out.parameter_paths {
                path.prepend(ComponentSegment::Argument(name.clone()));
            }
            Ok(out)
        }
        prebindgen_conversions::WireNode::Branch {
            meta: JniInputWireBranch::Field(name),
            children,
            ..
        } => {
            let mut out = input_components_when(children, context, active)?;
            for path in &mut out.parameter_paths {
                path.prepend(ComponentSegment::Field(name.to_string()));
            }
            Ok(out)
        }
        prebindgen_conversions::WireNode::Branch { children, .. } => {
            input_components_when(children, context, active)
        }
    }?;
    out.complete_parameter_paths()?;
    Ok(out)
}
fn input_components_when(
    nodes: &[JniInputWireNode],
    context: &KotlinContext<'_>,
    active: &str,
) -> Result<KotlinInput> {
    let mut out = KotlinInput {
        handle_inputs: Some(Vec::new()),
        ..Default::default()
    };
    for node in nodes {
        merge_input(&mut out, input_kotlin_when(node, context, active)?)?;
    }
    Ok(out)
}

pub(crate) fn merge_input(out: &mut KotlinInput, mut child: KotlinInput) -> Result<()> {
    child.complete_parameter_paths()?;
    child.prepare_arguments(
        !child.locks.is_empty() || !child.lock_groups.is_empty() || !child.consumed.is_empty(),
    )?;
    out.parameters.extend(child.parameters);
    out.parameter_paths.extend(child.parameter_paths);
    let argument_offset = out.arguments.len();
    if let Some(handles) = &mut child.handle_inputs {
        for handle in handles {
            if let Some(index) = &mut handle.argument {
                *index += argument_offset;
            }
        }
    }
    out.arguments.extend(child.arguments);
    out.prepared_arguments.extend(child.prepared_arguments);
    out.setup.extend(child.setup);
    if let (Some(out), Some(child)) = (&mut out.handle_inputs, child.handle_inputs) {
        out.extend(child);
    } else {
        out.handle_inputs = None;
    }
    out.lock_groups.extend(child.lock_groups);
    out.locks.extend(child.locks);
    out.consumed.extend(child.consumed);
    Ok(())
}
pub(crate) fn remap_output(
    c: &mut KotlinOutput,
    local: &str,
    result: &str,
    offset: usize,
    count: usize,
) {
    for text in
        c.delivered
            .iter_mut()
            .chain([&mut c.output_locals, &mut c.materialize, &mut c.cleanup])
    {
        for i in (0..count).rev() {
            *text = text.replace(
                &format!("{local}[{i}]"),
                &format!("{result}[{}]", offset + i),
            );
        }
    }
}
pub(crate) fn output_kotlin(
    node: &JniOutputWireNode,
    context: &KotlinContext<'_>,
) -> Result<KotlinOutput> {
    output_kotlin_scoped(node, context, None)
}

/// A collection tree shares one ownership list. Wrappers are registered in wire
/// traversal order, so failure cleanup can skip that prefix of native owners.
pub(crate) fn output_kotlin_scoped(
    node: &JniOutputWireNode,
    context: &KotlinContext<'_>,
    owners: Option<&str>,
) -> Result<KotlinOutput> {
    match node {
        prebindgen_conversions::WireNode::Branch {
            meta: JniOutputWireBranch::Enum(group),
            children,
            controls,
        } => crate::sealed::output(children, controls, group, context, owners),
        prebindgen_conversions::WireNode::Leaf(wires) => {
            output_leaves(&wires.iter().collect::<Vec<_>>(), context, owners)
        }
        prebindgen_conversions::WireNode::Chain {
            meta: JniOutputWireChain::Bulk,
            child,
            ..
        } => crate::bulk::output_kotlin(child, context, owners),
        prebindgen_conversions::WireNode::Chain {
            child: element,
            meta: JniOutputWireChain::Sequence(metadata),
            ..
        } => {
            let name = format!("{}owners", context.output_prefix);
            let root = owners.is_none() && has_handles(element, false);
            let scope = if root { Some(name.as_str()) } else { owners };
            let mut output = if context.bulk {
                if *metadata == JniSequence::Callback {
                    return Err("bulk scopes cannot enclose callback sequence delivery".into());
                }
                crate::bulk::output_list(element, context, scope)?
            } else {
                match *metadata {
                    JniSequence::Buffered => {
                        crate::sequence::output_list(element, context, scope, false)?
                    }
                    JniSequence::TypedBuffered => {
                        crate::sequence::output_list(element, context, scope, true)?
                    }
                    JniSequence::Callback => {
                        return Err(
                            "callback sequence requires function-level callback delivery".into(),
                        )
                    }
                }
            };
            if root {
                output.output_locals = format!(
                    "var {name}: {}.CollectionOwners? = null\n{}",
                    context.package, output.output_locals
                );
                output.materialize = format!(
                    "{name} = {}.CollectionOwners()\n{}\n{name}!!.complete = true\n",
                    context.package, output.materialize
                );
                let close = if let Some(failure) = context.cleanup_failure {
                    format!("{failure} = {name}?.close({failure}) ?: {failure}")
                } else {
                    format!("{name}?.close({})", context.failure)
                };
                output.cleanup = format!(
                    "{close}\nif ({name}?.complete != true) {{\n{}\n}}\n",
                    output.cleanup
                );
            }
            Ok(output)
        }
        prebindgen_conversions::WireNode::Chain {
            child: element,
            meta: JniOutputWireChain::Optional(metadata),
            ..
        } => optional::output_kotlin(element, metadata, context, owners),
        prebindgen_conversions::WireNode::Branch {
            meta: JniOutputWireBranch::Record(record),
            children,
            ..
        } => data::output_kotlin(children, record, context, owners),
        prebindgen_conversions::WireNode::Branch {
            meta: JniOutputWireBranch::Named(name),
            children,
            ..
        } => {
            let mut out = output_components(children, context, owners)?;
            out.result_names = (0..out.result_types.len())
                .map(|i| {
                    Some(if out.result_types.len() == 1 {
                        name.clone()
                    } else {
                        format!("{name}{i}")
                    })
                })
                .collect();
            Ok(out)
        }
        prebindgen_conversions::WireNode::Branch { children, .. } => {
            output_components(children, context, owners)
        }
    }
}
pub(crate) fn output_components(
    nodes: &[JniOutputWireNode],
    context: &KotlinContext<'_>,
    owners: Option<&str>,
) -> Result<KotlinOutput> {
    if nodes
        .iter()
        .all(|n| matches!(n, prebindgen_conversions::WireNode::Leaf(_)))
    {
        let wires = nodes.iter().flat_map(leaf_wires).collect::<Vec<_>>();
        return output_leaves(&wires, context, owners);
    }
    output_components_checked(nodes, context, owners, |_| Ok(()))
}
// Validate or normalize each property while its conversion is emitted once.
pub(crate) fn output_components_checked(
    nodes: &[JniOutputWireNode],
    context: &KotlinContext<'_>,
    owners: Option<&str>,
    mut check: impl FnMut(&mut KotlinOutput) -> Result<()>,
) -> Result<KotlinOutput> {
    let mut out = KotlinOutput::default();
    let mut offset = 0;
    for (index, node) in nodes.iter().enumerate() {
        let local = format!("{}_group{index}", context.result);
        let prefix = format!("{}g{index}_", context.output_prefix);
        let ctx = KotlinContext {
            result: &local,
            output_prefix: &prefix,
            ..*context
        };
        let count = crate::group::kotlin_result_fields(node).len();
        let mut c = output_kotlin_scoped(node, &ctx, owners)?;
        check(&mut c)?;
        remap_output(&mut c, &local, context.result, offset, count);
        c.result_names.resize(c.result_types.len(), None);
        out.result_names.extend(c.result_names);
        out.delivered.extend(c.delivered);
        out.result_types.extend(c.result_types);
        out.output_locals.push_str(&c.output_locals);
        out.materialize.push_str(&c.materialize);
        out.cleanup.push_str(&c.cleanup);
        offset += count;
    }
    Ok(out)
}

impl prebindgen_conversions::InputConversion for PlanInput {
    fn input_node(&self) -> &JniInputWireNode {
        &self.rust.node
    }
}
impl prebindgen_conversions::OutputConversion for PlanOutput {
    fn output_outcomes(&self) -> &prebindgen_conversions::Outcomes<JniOutputWireNode> {
        &self.rust.outcomes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_emit_private_conditions_and_use_supplied_output_bindings() {
        let flat = JniGen::builder()
            .items([(
                syn::parse_quote!(
                    pub struct Counter {}
                ),
                Default::default(),
            )])
            .set_package_prefix("example")
            .package(package!().class(ptr_class!(Counter)))
            .configured()
            .unwrap();
        let context = KotlinContext {
            package: "example",
            source_types: &flat.types,
            bulk: false,
            result: "native_values",
            failure: "delivery_error",
            cleanup_failure: None,
            output_prefix: "wrapped_",
        };
        // This condition is arbitrary Kotlin code, not a declaration selector path.
        let mut plan = JniCodec::raw()
            .input_leaf(prebindgen_declarations::InputLeaf::new(
                &JniBindings::parse([]).unwrap(),
                &syn::parse_quote!(crate::Counter),
                &(),
                syn::parse_quote!(incoming),
                syn::parse_quote!(value),
                &prebindgen_declarations::Condition::default(),
            ))
            .unwrap();
        let JniWire::Handle { condition, .. } =
            &mut prebindgen_conversions::wire_definitions_mut(std::iter::once(&mut plan.node))
                .next()
                .expect("logical wire index out of range")
                .kind
        else {
            panic!()
        };
        *condition = "isEnabled() && count > 3".into();
        JniCodec::raw().finish_input(&mut plan, &()).unwrap();
        let input = JniInputConverter::from(plan);
        // A handle and an ordinary i64 expose identical transport information.
        let scalar = JniWire::scalar(Primitive::I64);
        assert_eq!(
            prebindgen_flat::type_key(
                &prebindgen_conversions::wire_definitions(std::iter::once(input.input_node()))
                    .next()
                    .unwrap()
                    .kind
                    .rust_type()
            ),
            prebindgen_flat::type_key(&scalar.rust_type())
        );
        assert_eq!(
            prebindgen_conversions::wire_definitions(std::iter::once(input.input_node()))
                .next()
                .unwrap()
                .kind
                .kotlin_type(),
            scalar.kotlin_type()
        );
        let emitted = input.kotlin(&context).unwrap();
        assert_eq!(
            emitted.locks,
            ["if (isEnabled() && count > 3) incoming else null"]
        );
        assert_eq!(
            emitted.consumed,
            ["if (isEnabled() && count > 3) __pb_consume(incoming)"]
        );

        let plan = JniCodec::raw()
            .output_leaf(prebindgen_declarations::OutputLeaf::new(
                &JniBindings::parse([]).unwrap(),
                &syn::parse_quote!(crate::Counter),
                &(),
                syn::parse_quote!(wire),
                quote!(returned),
                prebindgen_declarations::ValueAccess::Owned,
            ))
            .unwrap();
        let output = JniOutputConverter::from(plan).kotlin(&context).unwrap();
        assert_eq!(output.delivered, ["requireNotNull(wrapped_0)"]);
        assert!(output
            .materialize
            .contains("example.Counter(native_values[0] as Long)"));
        assert!(output.cleanup.contains("example.JNINative.drop_Counter_"));
        assert!(output.cleanup.contains("(native_values[0] as Long)"));
        assert!(output
            .cleanup
            .contains("delivery_error.addSuppressed(cleanup)"));
    }
}
