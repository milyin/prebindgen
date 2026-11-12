//! Declaration compilation operations implemented by a language adapter.
use super::*;

/// Interpret adapter-specific trees into the fixed schema’s parameter/result slots.
/// Adapters select their operation vocabularies and dispatch to concrete helpers.
/// Optional features reject unsupported work or decline a specialization.
/// All operations run during generation; the returned fragments run in the library.
#[allow(clippy::type_complexity)]
pub trait Codec {
    /// Input vocabulary interpreted by this adapter.
    type InputOperations: Operations;
    /// Output vocabulary interpreted by this adapter.
    type OutputOperations: Operations;
    /// Adapter-owned state scoped to one conversion compilation.
    type CompileState: Default;

    /// Interpret one resolved input operation.
    #[allow(clippy::too_many_arguments)]
    fn compile_input_tree(
        &self,
        cx: &Compiler<'_, Self>,
        ty: &Type,
        tree: &ResolvedInput<Self>,
        extra: &Self::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInput<Self>>
    where
        Self: Sized;
    /// Interpret one resolved output operation.
    #[allow(clippy::too_many_arguments)]
    fn compile_output_tree(
        &self,
        cx: &Compiler<'_, Self>,
        ty: &Type,
        tree: &ResolvedOutput<Self>,
        extra: &Self::Extra,
        value: TokenStream,
        borrowed: bool,
        wire: Option<&syn::Ident>,
    ) -> Result<CodecOutput<Self>>
    where
        Self: Sized;
    /// Interpret a function or callback result, including its error channel.
    #[allow(clippy::too_many_arguments)]
    fn compile_output_root(
        &self,
        cx: &Compiler<'_, Self>,
        ty: &Type,
        tree: &ResolvedOutput<Self>,
        extra: &Self::Extra,
        value: TokenStream,
        borrowed: bool,
        reference_return: bool,
    ) -> Result<CodecOutput<Self>>
    where
        Self: Sized,
    {
        if reference_return {
            self.compile_output_return(cx, ty, tree, extra, value, borrowed)
        } else {
            cx.output(ty, tree, extra, value, borrowed)
        }
    }
    /// Interpret a Rust return or getter result with its reference lifetime.
    fn compile_output_return(
        &self,
        cx: &Compiler<'_, Self>,
        ty: &Type,
        tree: &ResolvedOutput<Self>,
        extra: &Self::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<Self>>
    where
        Self: Sized,
    {
        cx.output(ty, tree, extra, value, borrowed)
    }

    /// Fixed converter and declaration metadata types, selected before parsing.
    type Schema: Schema<
        Input: prebindgen_flat::InputConversion + From<CodecInput<Self>>,
        Output: prebindgen_flat::OutputConversion + From<CodecOutput<Self>>,
    >;
    /// Complete description of one foreign wire value.
    type Wire: prebindgen_flat::WireKind + Clone + std::fmt::Debug;
    /// Names supplied by the language's Rust wrapper emitter.
    type Context: prebindgen_conversions::TemplateContext + 'static;
    /// Borrowed occurrence settings used while preparing nested conversions.
    type Extra;
    /// Adapter metadata for a one-child node, without duplicated wires/children.
    type Chain: Clone;
    /// Adapter metadata for groups/alternatives, without duplicated children.
    type Branch: Clone;

    /// Compile standalone callback declarations and typed error outputs stored in Flat.
    /// Called before function plans; implementations consume only resolved model data.
    fn compile_declarations(
        &self,
        _flat: &mut Flat<Self::Schema>,
        _extra: &Self::Extra,
    ) -> Result<()> {
        Ok(())
    }

    /// Decode a leaf in the compiler-selected physical representation and scope.
    fn input_leaf(&self, request: InputLeaf<'_, Self>) -> Result<CodecInput<Self>>;
    /// Prepare the source expression and defer owned output transfers until commit.
    fn output_leaf(&self, request: OutputLeaf<'_, Self>) -> Result<CodecOutput<Self>>;
    /// Whether a leaf moves its source. Must agree with output_leaf and is needed
    /// before ordering observations and moves of the same source value.
    fn output_consumes(&self, flat: &Flat<Self::Schema>, ty: &Type, extra: &Self::Extra) -> bool;
    /// Describe the requested structural group; reject unsupported kinds explicitly.
    fn group(&self, kind: GroupKind<'_>) -> Result<Self::Branch>;
    /// Explicit opt-in to bulk conversion for this direction. JNI dispatch and
    /// reusable sequence helpers check it before selecting bulk transport.
    fn supports_bulk(&self, _direction: Direction) -> bool {
        false
    }
    /// Select a flag or niche, retire inactive offers, and prepare metadata and
    /// inactive output values. Input niches require a test; output niches do not.
    fn optional(&self, _request: Optional<'_, Self>) -> Result<OptionalEncoding<Self>> {
        Err("optional conversion is unsupported by this codec".into())
    }
    /// Create an i32-compatible selector kind, before its source path is prefixed.
    fn selector(&self, _extra: &Self::Extra) -> Result<Self::Wire> {
        Err("selector tags are unsupported by this codec".into())
    }
    /// Validate the already-prefixed selector representation and retire inactive
    /// arm offers. Runtime tag validation remains the shared compiler's policy.
    fn alternatives(&self, _request: Alternatives<'_, Self>) -> Result<Self::Branch> {
        Err("input alternatives are unsupported by this codec".into())
    }
    /// Interpret a selected source variant and its recursively compiled payload.
    fn enum_variants(&self, _request: EnumVariants<'_, Self>) -> Result<EnumEncoding<Self>> {
        Err("payload enums are unsupported by this codec".into())
    }
    /// Wrap an ordinary enum input in the adapter's public value carrier.
    #[allow(clippy::too_many_arguments)]
    fn input_enum(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        plan: CodecInput<Self>,
        _active: &Condition,
        _role: InputRole,
    ) -> Result<CodecInput<Self>> {
        Ok(plan)
    }
    /// Wrap an ordinary enum output in the adapter's public value carrier.
    fn output_enum(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        plan: CodecOutput<Self>,
    ) -> Result<CodecOutput<Self>> {
        Ok(plan)
    }
    /// Choose ordinary or bulk input row transport; preserve row-local lifetimes.
    fn sequence_input(
        &self,
        _request: SequenceInputRequest<'_, Self>,
    ) -> Result<SequenceInput<Self::Wire, Self::Chain>> {
        Err("collections are unsupported by this codec".into())
    }
    /// Choose ordinary/bulk output storage and the explicit element delivery.
    fn sequence_output(
        &self,
        _request: SequenceOutputRequest<'_, Self>,
    ) -> Result<SequenceOutput<Self::Wire, Self::Chain>> {
        Err("collections are unsupported by this codec".into())
    }
    /// Prepare matching source-error wires and propagation. The default preserves
    /// mapped String errors without requiring a complete typed-error implementation.
    fn source_error(&self, request: SourceError<'_, Self>) -> Result<SourceErrorPlan<Self>> {
        prepare_mapped_source_error(self, request)
    }
    /// Route an explicitly mapped String error through this adapter's channel.
    fn mapped_error(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _message: TokenStream,
        _extra: &Self::Extra,
    ) -> Result<TokenStream> {
        Err("Result error routing is unsupported by this codec".into())
    }

    /// Whether foreign tags and array lengths need runtime validation.
    /// Trusted raw-pointer adapters may require valid representations from callers.
    fn validates_inputs(&self) -> bool {
        true
    }

    /// Group a compiled native expansion into its physical bulk input carrier.
    fn bulk_input(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _child: CodecInput<Self>,
        _active: &Condition,
    ) -> Result<CodecInput<Self>> {
        Err("bulk inputs are unsupported by this codec".into())
    }

    /// Group a compiled native expansion into its physical bulk output carrier.
    fn bulk_output(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _child: CodecOutput<Self>,
    ) -> Result<CodecOutput<Self>> {
        Err("bulk outputs are unsupported by this codec".into())
    }

    /// Compile a callback whose two conversion directions are already resolved.
    fn input_arrow(&self, _request: InputArrow<'_, Self>) -> Result<CodecInput<Self>> {
        Err("configured callbacks are unsupported by this codec".into())
    }

    /// Select bulk for automatically materialized output sequences.
    /// Explicit ordinary rows and per-row callbacks bypass this default.
    fn default_output_sequence_is_bulk(
        &self,
        _flat: &Flat<Self::Schema>,
        _extra: &Self::Extra,
    ) -> bool {
        false
    }

    /// Annotate an already compiled foreign-record input; structural expansion is complete.
    fn input_record_metadata(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        _ty: &Type,
        _wire: &syn::Ident,
        _active: &Condition,
        _plan: &mut CodecInput<Self>,
        _extra: &Self::Extra,
    ) -> Result<()> {
        Err("foreign record inputs are unsupported by this codec".into())
    }

    /// Annotate an already compiled foreign-record output with its selected representation.
    fn output_record_metadata(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        _ty: &Type,
        _wire: &syn::Ident,
        _plan: &mut CodecOutput<Self>,
        _extra: &Self::Extra,
    ) -> Result<()> {
        Err("foreign record outputs are unsupported by this codec".into())
    }

    /// Borrow an input collection directly before preparing its element conversions.
    /// Called only for reference sequence occurrences. The resolved `element`
    /// recipe must be compatible with the borrowed storage. Return None to retain
    /// ordinary sequence marshalling, or reject unsupported borrowing explicitly.
    #[allow(clippy::too_many_arguments)]
    fn input_borrowed_sequence(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _var: syn::Ident,
        _element: &ResolvedInput<Self>,
        _active: &Condition,
    ) -> Result<Option<CodecInput<Self>>> {
        Ok(None)
    }

    /// Logical binding-failure payload, independently of how the adapter delivers
    /// it. None means this codec declares no binding-failure channel.
    fn binding_failure_layout(
        &self,
        _extra: &Self::Extra,
    ) -> Result<Option<prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>>> {
        Ok(None)
    }

    /// Validate a prepared input before installing it or embedding it in a sequence.
    /// The default accepts every structural shape. Adapters interpret their own
    /// metadata and enforce semantic rules, such as requiring nonempty alternatives.
    fn validate_input(&self, _plan: &CodecInput<Self>) -> Result<()> {
        Ok(())
    }

    /// Validate named success expressions against the adapter's output representation.
    fn validate_output(&self, plan: &CodecOutput<Self>) -> Result<()> {
        prebindgen_conversions::validate_value_bindings(&plan.values)
    }

    /// Prepend a semantic source path to one prepared occurrence. Override when
    /// node controls carry paths, or element definitions have a separate
    /// path context. Codecs without source-path metadata need no implementation.
    fn prefix_layout(
        &self,
        _layout: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        _segment: ComponentSegment,
    ) {
    }

    /// Complete a composed parameter plan before it is stored in a slot.
    /// Called once per bound parameter, including records and selectors. The
    /// tree is complete when this hook runs. Edits to it or its metadata are
    /// immediately visible to adapter operations; no secondary layout needs refreshing.
    /// The compiler validates the tree before installing the finished slot.
    fn finish_input(&self, _plan: &mut CodecInput<Self>, _extra: &Self::Extra) -> Result<()> {
        Ok(())
    }

    /// Complete a composed result plan, including unit and multi-wire results.
    /// Install call-level delivery here, after all output expressions are known.
    /// The logical schema already includes the composed payload and error channels.
    /// Edits to the tree or its adapter metadata are preserved and validated.
    fn finish_output(&self, _plan: &mut CodecOutput<Self>, _extra: &Self::Extra) -> Result<()> {
        Ok(())
    }

    /// Convert a reference supplied directly by the Rust return value.
    ///
    /// Unlike observing an owned value through a getter, this reference already
    /// has the source function's lifetime contract. Adapters may expose it as a
    /// non-owning foreign pointer. `reference` preserves the referent and mutability;
    /// `value` produces exactly that Rust reference.
    /// This also applies inside optional and successful Result returns and to
    /// getter results whose receiver remains alive. Callback arguments supplied as references
    /// also use this hook: their loan lasts only for the foreign invocation.
    /// The foreign caller must preserve the referenced storage and Rust aliasing
    /// rules. No runtime lifetime tracking is implied.
    ///
    /// Return `None` to use ordinary borrowed leaf conversion (the default).
    fn output_return_reference(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        _reference: &syn::TypeReference,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _value: TokenStream,
    ) -> Result<Option<CodecOutput<Self>>> {
        Ok(None)
    }

    /// Select an address-preserving representation of a borrowed collection.
    ///
    /// Called before element conversion when a sequence is borrowed and neither
    /// its own policy nor an enclosing projection explicitly permits copying.
    /// `value` is a reference to the collection; `element` is its resolved output
    /// recipe. An adapter may return a non-owning view or reject an unsupported
    /// borrow instead of allocating converted rows. `None` retains ordinary
    /// sequence conversion, which may be required by the foreign representation.
    /// `temporary` means the receiver backing this observation is destroyed
    /// after the generated call; a view must not escape that receiver.
    #[allow(clippy::too_many_arguments)]
    fn output_borrowed_sequence(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _value: TokenStream,
        _element: &ResolvedOutput<Self>,
        _delivery: SequenceDelivery,
        _temporary: bool,
    ) -> Result<Option<CodecOutput<Self>>> {
        Ok(None)
    }

    /// Place an owned value in its final foreign allocation before getter views.
    ///
    /// Returns its owning output conversion and a Rust `*mut ty` expression from
    /// which getter receivers can be borrowed. That address must remain valid
    /// across conversion, delivery and ownership commit. Until commit, the plan
    /// must reclaim the allocation on failure. The default keeps ordinary output
    /// conversion and observation-before-move ordering.
    fn output_owner(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _value: TokenStream,
    ) -> Result<Option<(CodecOutput<Self>, TokenStream)>> {
        Ok(None)
    }

    /// Check a getter loan while its owning output is being delivered.
    fn validate_owner_getter(
        &self,
        _flat: &Flat<Self::Schema>,
        _owner: &Type,
        _getter: &str,
        _plan: &CodecOutput<Self>,
    ) -> Result<()> {
        Ok(())
    }

    /// Check whether a getter's converted outputs can outlive its temporary owner.
    /// Owner-backed views are compiled separately and already retain their owner.
    fn validate_getter_output(
        &self,
        _flat: &Flat<Self::Schema>,
        _receiver: &Type,
        _borrowed: bool,
        _ty: &Type,
        _plan: &CodecOutput<Self>,
        _extra: &Self::Extra,
    ) -> Result<()> {
        Ok(())
    }

    /// Explicit projection; the default preserves field-policy and reference
    /// rejection. An adapter override keeps its own existing type-specific rules.
    fn output_projected(
        &self,
        mut request: OutputLeaf<'_, Self>,
        projection: &Projection,
    ) -> Result<CodecOutput<Self>> {
        if !projection.fields.is_empty() {
            return Err("this output converter does not support field projection overrides".into());
        }
        let mut payload = request.ty;
        while let Some(inner) = option_inner(payload) {
            payload = inner;
        }
        if projection.mode == ProjectionMode::Owned && matches!(payload, Type::Reference(_)) {
            return Err("owned projection cannot consume a reference payload".into());
        }
        request.access = if projection.mode == ProjectionMode::Borrowed {
            ValueAccess::Shared
        } else {
            ValueAccess::Owned
        };
        self.output_leaf(request)
    }
}

/// Prepare the ordinary mapped-String error path, also usable by a codec that
/// overrides typed source errors and needs the default for undeclared types.
pub fn prepare_mapped_source_error<C: Codec + ?Sized>(
    codec: &C,
    request: SourceError<'_, C>,
) -> Result<SourceErrorPlan<C>> {
    let SourceError {
        flat,
        ty,
        extra,
        value: _,
        message,
    } = request;
    let message = message.ok_or_else(|| {
        format!(
            "Result error `{}` requires an explicit error_mapper or on_error mapper",
            type_key(ty)
        )
    })?;
    let propagate = codec.mapped_error(flat, ty, message, extra)?;
    let text = syn::parse_quote!(String);
    let output = codec.output_leaf(OutputLeaf::new(
        flat,
        &text,
        extra,
        format_ident!("source_error_message"),
        quote!(source_error_message),
        ValueAccess::Owned,
    ))?;
    Ok(SourceErrorPlan {
        payload: output.outcomes.success,
        propagate,
    })
}

/// Optional frontend for resolving expansion and conversion declarations.
/// Adapters with another configuration language can implement Codec directly.
pub trait DeclarationCodec: Codec {
    /// Whether a type-targeted expansion deliberately omits its payload.
    fn is_discarded_output(&self, _tree: &ResolvedOutput<Self>) -> bool {
        false
    }
    /// Select per-element callback delivery in this adapter's resolved output.
    fn sequence_callback(
        &self,
        _cx: &mut Resolve<'_, Self>,
        _ty: &Type,
        _tree: ResolvedOutput<Self>,
        _structural: bool,
    ) -> Result<ResolvedOutput<Self>>
    where
        Self: Sized,
    {
        Err("callback return is unsupported by this adapter".into())
    }

    /// Interpret one input declaration operation after resolving its children.
    fn resolve_input_leaf(
        &self,
        operation: InputLeafOp,
    ) -> Result<<Self::InputOperations as Operations>::Leaf>;
    /// Interpret one input declaration operation after resolving its children.
    fn resolve_input_unary(
        &self,
        operation: InputUnary,
    ) -> Result<<Self::InputOperations as Operations>::Unary>;
    /// Interpret one input declaration operation after resolving its children.
    fn resolve_input_branch(
        &self,
        operation: InputBranch,
    ) -> Result<<Self::InputOperations as Operations>::Branch>;
    /// Interpret one input declaration operation after resolving its children.
    fn resolve_input_arrow(
        &self,
        operation: Box<CallbackOperation>,
    ) -> Result<<Self::InputOperations as Operations>::Arrow>;
    /// Normalize the adapter tree after all nested requests have been resolved.
    fn finish_input_tree(&self, tree: ResolvedInput<Self>) -> Result<ResolvedInput<Self>> {
        Ok(tree)
    }
    /// Interpret one output declaration operation after resolving its children.
    fn resolve_output_leaf(
        &self,
        operation: OutputLeafOp,
    ) -> Result<<Self::OutputOperations as Operations>::Leaf>;
    /// Interpret one output declaration operation after resolving its children.
    fn resolve_output_unary(
        &self,
        operation: OutputUnary,
    ) -> Result<<Self::OutputOperations as Operations>::Unary>;
    /// Interpret one output declaration operation after resolving its children.
    fn resolve_output_branch(
        &self,
        operation: OutputBranch,
    ) -> Result<<Self::OutputOperations as Operations>::Branch>;
    /// Interpret one output declaration operation after resolving its children.
    fn resolve_output_arrow(
        &self,
        operation: std::convert::Infallible,
    ) -> Result<<Self::OutputOperations as Operations>::Arrow>;
    /// Normalize the adapter tree after all nested requests have been resolved.
    fn finish_output_tree(&self, tree: ResolvedOutput<Self>) -> Result<ResolvedOutput<Self>> {
        Ok(tree)
    }
    /// Source-aware input normalization, called only during tree preparation.
    /// Return an explicit operation tree for a language-specific representation.
    fn expand_input(
        &self,
        _cx: &mut Resolve<'_, Self>,
        _ty: &Type,
        _tree: &Input<InputRequest, OutputRequest>,
    ) -> Result<Option<ResolvedInput<Self>>>
    where
        Self: Sized,
    {
        Ok(None)
    }
    /// Source-aware output normalization, called before conversion compilation.
    fn expand_output(
        &self,
        _cx: &mut Resolve<'_, Self>,
        _ty: &Type,
        _tree: &Output<OutputRequest, InputRequest>,
    ) -> Result<Option<ResolvedOutput<Self>>>
    where
        Self: Sized,
    {
        Ok(None)
    }
    /// Keep a collection occurrence in one native input leaf instead of
    /// decomposing it into element rows. Explicit `InputUnary::Sequence` overrides
    /// this preference; input defaults and whole-collection identities use it.
    fn input_collection_is_leaf(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _role: InputRole,
        _extra: &Self::Extra,
    ) -> bool {
        false
    }
    /// Keep a collection occurrence in one native output leaf. Explicit
    /// `OutputUnary::Sequence` and element-targeted expansions still produce rows.
    fn output_collection_is_leaf(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _extra: &Self::Extra,
    ) -> bool {
        false
    }
}
