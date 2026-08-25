//! Syntax-free plans for converters composed from recipe shapes.
//!
//! A plan keeps Flat [`TypeRef`]s opaque and records only shape operations,
//! wire-side types and child converter contracts. [`RustFunction::render`]
//! receives the writer-owned [`Emit`] after resolution and validation, which is
//! the first point at which the captured Rust types and function bodies are
//! materialized.

use prebindgen_registry::{
    chain::{self, Chain as _},
    flat::{Alternative, TypeRef},
    generation::{GenerationPlan, SiteId},
    recipe::Mode,
    write::RustFunction,
    Emit,
};

use super::{builder::qualify_source_type, *};

/// The callable facts a parent chain needs from a child converter.
#[derive(Clone)]
pub(crate) struct CCall(chain::Call);

impl CCall {
    pub(crate) fn ident(&self) -> &syn::Ident {
        self.0.ident()
    }

    pub(crate) fn fallible(&self) -> bool {
        self.0.fallible()
    }

    pub(crate) fn unsafe_(&self) -> bool {
        self.0.unsafe_()
    }
}

impl chain::Child for CCall {
    fn call(&self) -> &chain::Call {
        &self.0
    }

    fn invoke(&self, value: TokenStream) -> TokenStream {
        let ident = self.ident();
        quote!(#ident(#value))
    }
}

/// A complete legacy function or a chain waiting for final rendering.
#[derive(Clone)]
pub(crate) struct CFunction {
    call: CCall,
    body: CBody,
}

#[derive(Clone)]
enum CBody {
    Complete(syn::ItemFn),
    Custom(CustomPlan),
    InputTerminal(InputTerminalPlan),
    OutputTerminal(OutputTerminalPlan),
    Payload(PayloadPlan),
    Borrow(BorrowPlan),
    SliceInput(SliceInputPlan),
    Marker(MarkerPlan),
    Product(ProductPlan),
    Optional(OptionalPlan),
    Sequence(SequencePlan),
    Choice(ChoicePlan),
    /// The callable contract for an Invoke fragment. The actual helper is
    /// rendered from the callback artifact that consumes frozen callback sites.
    DeferredInvoke,
}

impl CFunction {
    pub(crate) fn complete(function: syn::ItemFn) -> Self {
        let call = CCall(chain::Call::complete(&function));
        Self {
            call,
            body: CBody::Complete(function),
        }
    }

    pub(crate) fn product(plan: ProductPlan) -> Self {
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            plan.fields.iter().any(|field| field.converter.fallible()),
            plan.direction == Direction::Construct,
        ));
        Self {
            call,
            body: CBody::Product(plan),
        }
    }

    pub(crate) fn custom(plan: CustomPlan) -> Self {
        let call = CCall(chain::Call::new(plan.ident.clone(), plan.fallible(), false));
        Self {
            call,
            body: CBody::Custom(plan),
        }
    }

    pub(crate) fn output_terminal(plan: OutputTerminalPlan) -> Self {
        let call = CCall(chain::Call::new(plan.ident.clone(), false, false));
        Self {
            call,
            body: CBody::OutputTerminal(plan),
        }
    }

    pub(crate) fn input_terminal(plan: InputTerminalPlan) -> Self {
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            plan.operation.fallible(),
            plan.operation.unsafe_(),
        ));
        Self {
            call,
            body: CBody::InputTerminal(plan),
        }
    }

    pub(crate) fn payload(plan: PayloadPlan) -> Self {
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            plan.direction == Direction::Construct && !plan.optional,
            plan.direction == Direction::Construct,
        ));
        Self {
            call,
            body: CBody::Payload(plan),
        }
    }

    pub(crate) fn borrow(plan: BorrowPlan) -> Self {
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            plan.operation.fallible(),
            true,
        ));
        Self {
            call,
            body: CBody::Borrow(plan),
        }
    }

    pub(crate) fn slice_input(plan: SliceInputPlan) -> Self {
        let call = CCall(chain::Call::new(plan.ident.clone(), false, false));
        Self {
            call,
            body: CBody::SliceInput(plan),
        }
    }

    pub(crate) fn marker(plan: MarkerPlan) -> Self {
        let call = CCall(chain::Call::new(plan.ident.clone(), false, false));
        Self {
            call,
            body: CBody::Marker(plan),
        }
    }

    pub(crate) fn optional(plan: OptionalPlan) -> Self {
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            plan.converter.fallible(),
            true,
        ));
        Self {
            call,
            body: CBody::Optional(plan),
        }
    }

    pub(crate) fn sequence(plan: SequencePlan) -> Self {
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            plan.child.fallible(),
            plan.child.unsafe_(),
        ));
        Self {
            call,
            body: CBody::Sequence(plan),
        }
    }

    pub(crate) fn choice(plan: ChoicePlan) -> Self {
        let fallible = plan.direction == Direction::Construct
            || plan
                .arms
                .iter()
                .flat_map(|arm| &arm.parts)
                .any(|part| part.child.fallible());
        let call = CCall(chain::Call::new(
            plan.ident.clone(),
            fallible,
            plan.direction == Direction::Construct,
        ));
        Self {
            call,
            body: CBody::Choice(plan),
        }
    }

    pub(crate) fn deferred_invoke(ident: syn::Ident) -> Self {
        Self {
            call: CCall(chain::Call::new(ident, false, true)),
            body: CBody::DeferredInvoke,
        }
    }

    pub(crate) fn is_deferred_invoke(&self) -> bool {
        matches!(self.body, CBody::DeferredInvoke)
    }

    #[cfg(test)]
    pub(crate) fn is_custom(&self) -> bool {
        matches!(self.body, CBody::Custom(_))
    }

    #[cfg(test)]
    pub(crate) fn is_output_terminal(&self) -> bool {
        matches!(self.body, CBody::OutputTerminal(_))
    }

    #[cfg(test)]
    pub(crate) fn is_input_terminal(&self) -> bool {
        matches!(self.body, CBody::InputTerminal(_))
    }

    #[cfg(test)]
    pub(crate) fn is_string_field_terminal(&self) -> bool {
        matches!(
            &self.body,
            CBody::InputTerminal(InputTerminalPlan {
                operation: InputTerminalOperation::StringField,
                ..
            })
        )
    }

    #[cfg(test)]
    pub(crate) fn is_bool_field_terminal(&self) -> bool {
        matches!(
            &self.body,
            CBody::OutputTerminal(OutputTerminalPlan {
                operation: OutputTerminalOperation::BoolField,
                ..
            })
        )
    }

    #[cfg(test)]
    pub(crate) fn is_payload(&self) -> bool {
        matches!(self.body, CBody::Payload(_))
    }

    #[cfg(test)]
    pub(crate) fn borrow_operation(&self) -> Option<&BorrowOperation> {
        match &self.body {
            CBody::Borrow(plan) => Some(&plan.operation),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn slice_input_operation(&self) -> Option<(&TypeRef, bool)> {
        match &self.body {
            CBody::SliceInput(plan) => Some((&plan.element, plan.reinterpret)),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn marker_operation(&self) -> Option<&MarkerOperation> {
        match &self.body {
            CBody::Marker(plan) => Some(&plan.operation),
            _ => None,
        }
    }

    pub(crate) fn call(&self) -> &CCall {
        &self.call
    }
}

impl RustFunction for CFunction {
    fn ident(&self) -> &syn::Ident {
        self.call.ident()
    }

    fn render(&self, emit: &Emit) -> syn::ItemFn {
        match &self.body {
            CBody::Complete(function) => function.clone(),
            CBody::Custom(plan) => plan.render(emit),
            CBody::InputTerminal(plan) => plan.render(emit),
            CBody::OutputTerminal(plan) => plan.render(emit),
            CBody::Payload(plan) => plan.render(emit),
            CBody::Borrow(plan) => plan.render(emit),
            CBody::SliceInput(plan) => plan.render(),
            CBody::Marker(plan) => plan.render(),
            CBody::Product(plan) => plan.render(emit),
            CBody::Choice(plan) => plan.render(emit),
            CBody::Sequence(plan) => plan.render(emit),
            CBody::Optional(plan) => plan.render(emit),
            CBody::DeferredInvoke => {
                unreachable!("a deferred C Invoke helper is rendered by its callback artifact")
            }
        }
    }
}

/// A multi-wire crossing whose ABI lives in its frozen site value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum MarkerOperation {
    Optional,
    Sequence,
    Result,
}

/// A typed replacement for one legacy zero-argument marker converter.
#[derive(Clone)]
pub(crate) struct MarkerPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) operation: MarkerOperation,
    pub(crate) subs: Vec<TypeRef>,
}

impl MarkerPlan {
    fn render(&self) -> syn::ItemFn {
        match self.operation {
            MarkerOperation::Optional | MarkerOperation::Sequence | MarkerOperation::Result => {
                render_marker(&self.ident)
            }
        }
    }
}

/// A zero-copy shared-slice input retained without a legacy converter body.
#[derive(Clone)]
pub(crate) struct SliceInputPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) element: TypeRef,
    pub(crate) wire: syn::Type,
    pub(crate) reinterpret: bool,
}

impl SliceInputPlan {
    fn render(&self) -> syn::ItemFn {
        render_marker(&self.ident)
    }
}

fn render_marker(name: &syn::Ident) -> syn::ItemFn {
    syn::parse_quote!(
        #[allow(non_snake_case, dead_code, unused)]
        pub(crate) fn #name() {}
    )
}

/// One source borrow crossing retained without spelling its referent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BorrowOperation {
    StrInput,
    SharedInput,
    MutableInput,
    MutableUninitInput,
    SharedOutput,
}

impl BorrowOperation {
    fn fallible(&self) -> bool {
        !matches!(self, Self::SharedOutput)
    }
}

/// A borrow terminal whose referent is materialized only during final emission.
#[derive(Clone)]
pub(crate) struct BorrowPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source_inner: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Type,
    pub(crate) operation: BorrowOperation,
    pub(crate) null_message: String,
}

impl BorrowPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source_inner = qualify_source_type(
            &emit.spell_ty(&self.source_inner),
            self.source_module.as_ref(),
        );
        let wire = &self.wire;
        let null_message = &self.null_message;

        match self.operation {
            BorrowOperation::StrInput => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name<'a>(
                    v: #wire,
                ) -> ::core::result::Result<&'a str, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#null_message),
                        );
                    }
                    match ::std::ffi::CStr::from_ptr(v).to_str() {
                        ::core::result::Result::Ok(s) => ::core::result::Result::Ok(s),
                        ::core::result::Result::Err(_) => ::core::result::Result::Err(
                            ::std::string::String::from("invalid UTF-8 in str argument"),
                        ),
                    }
                }
            ),
            BorrowOperation::SharedInput => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name<'a>(
                    v: #wire,
                ) -> ::core::result::Result<&'a #source_inner, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#null_message),
                        );
                    }
                    ::core::result::Result::Ok(&*(v as *const #source_inner))
                }
            ),
            BorrowOperation::MutableInput => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name<'a>(
                    v: #wire,
                ) -> ::core::result::Result<&'a mut #source_inner, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#null_message),
                        );
                    }
                    ::core::result::Result::Ok(&mut *(v as *mut #source_inner))
                }
            ),
            BorrowOperation::MutableUninitInput => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name<'a>(
                    v: #wire,
                ) -> ::core::result::Result<
                    &'a mut ::core::mem::MaybeUninit<#source_inner>,
                    ::std::string::String,
                > {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#null_message),
                        );
                    }
                    ::core::result::Result::Ok(
                        &mut *(v as *mut ::core::mem::MaybeUninit<#source_inner>)
                    )
                }
            ),
            BorrowOperation::SharedOutput => syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused)]
                pub(crate) unsafe fn #name(v: &#source_inner) -> #wire {
                    v as *const #source_inner as #wire
                }
            ),
        }
    }
}

/// A tagged-union pointer payload retained without spelling its source types.
#[derive(Clone)]
pub(crate) struct PayloadPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_inner: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Type,
    pub(crate) direction: Direction,
    pub(crate) optional: bool,
    pub(crate) boxed: bool,
    pub(crate) null_message: String,
}

impl PayloadPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = qualify_source_type(&emit.spell_ty(&self.source), self.source_module.as_ref());
        let source_inner = qualify_source_type(
            &emit.spell_ty(&self.source_inner),
            self.source_module.as_ref(),
        );
        let wire = &self.wire;

        match self.direction {
            Direction::Construct => {
                let owned = if self.boxed {
                    quote!(::std::boxed::Box::from_raw(v as *mut #source_inner))
                } else {
                    quote!(*::std::boxed::Box::from_raw(v as *mut #source_inner))
                };
                if self.optional {
                    syn::parse_quote!(
                        #[allow(non_snake_case, unused_variables, dead_code)]
                        pub(crate) unsafe fn #name(v: #wire) -> #source {
                            if v.is_null() {
                                ::core::option::Option::None
                            } else {
                                ::core::option::Option::Some(#owned)
                            }
                        }
                    )
                } else {
                    let null_message = &self.null_message;
                    syn::parse_quote!(
                        #[allow(non_snake_case, unused_variables, dead_code)]
                        pub(crate) unsafe fn #name(
                            v: #wire,
                        ) -> ::core::result::Result<#source, ::std::string::String> {
                            if v.is_null() {
                                return ::core::result::Result::Err(
                                    ::std::string::String::from(#null_message),
                                );
                            }
                            ::core::result::Result::Ok(#owned)
                        }
                    )
                }
            }
            Direction::Deconstruct => {
                let encode = |value: TokenStream| {
                    if self.boxed {
                        quote!(::std::boxed::Box::into_raw(#value) as #wire)
                    } else {
                        quote!(
                            ::std::boxed::Box::into_raw(::std::boxed::Box::new(#value)) as #wire
                        )
                    }
                };
                let body = if self.optional {
                    let some = encode(quote!(__v));
                    quote!(match v {
                        ::core::option::Option::Some(__v) => #some,
                        ::core::option::Option::None => ::core::ptr::null_mut(),
                    })
                } else {
                    encode(quote!(v))
                };
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) fn #name(v: #source) -> #wire {
                        #body
                    }
                )
            }
        }
    }
}

/// One C input terminal operation selected without spelling its source.
#[derive(Clone)]
pub(crate) enum InputTerminalOperation {
    OwnedHandle {
        null_message: String,
    },
    ValueOpaque {
        opaque: syn::Type,
        writeback: ValueOpaqueWriteback,
        null_message: String,
    },
    Enum {
        c_name: syn::Ident,
        variants: Vec<syn::Ident>,
        invalid_message: String,
        size_message: String,
        align_message: String,
    },
    String,
    StringField,
    StrMarker,
    Bool,
    Scalar,
}

/// How a consumed value-opaque C slot is left safely droppable.
#[derive(Clone)]
pub(crate) enum ValueOpaqueWriteback {
    None,
    NullFields(Vec<syn::Ident>),
    Gravestone,
}

impl ValueOpaqueWriteback {
    pub(crate) fn render(&self, slot: &syn::Ident, opaque: &syn::Type) -> TokenStream {
        match self {
            Self::None => TokenStream::new(),
            Self::NullFields(fields) => {
                quote!(#( (*#slot).#fields = ::core::ptr::null_mut(); )*)
            }
            Self::Gravestone => quote!(
                ::core::ptr::write(
                    #slot,
                    <#opaque as ::prebindgen_c_runtime::Gravestone>::gravestone(),
                );
            ),
        }
    }
}

impl InputTerminalOperation {
    fn fallible(&self) -> bool {
        matches!(
            self,
            Self::OwnedHandle { .. } | Self::ValueOpaque { .. } | Self::Enum { .. } | Self::String
        )
    }

    fn unsafe_(&self) -> bool {
        !matches!(self, Self::StrMarker | Self::Scalar)
    }
}

/// A C input terminal retained until the final writer owns [`Emit`].
#[derive(Clone)]
pub(crate) struct InputTerminalPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Type,
    pub(crate) operation: InputTerminalOperation,
}

impl InputTerminalPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = qualify_source_type(&emit.spell_ty(&self.source), self.source_module.as_ref());
        let wire = &self.wire;
        match &self.operation {
            InputTerminalOperation::OwnedHandle { null_message } => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: #wire,
                ) -> ::core::result::Result<#source, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#null_message),
                        );
                    }
                    ::core::result::Result::Ok(
                        *::std::boxed::Box::from_raw(v as *mut #source)
                    )
                }
            ),
            InputTerminalOperation::ValueOpaque {
                opaque,
                writeback,
                null_message,
            } => {
                let writeback = writeback.render(&format_ident!("v"), opaque);
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) unsafe fn #name(
                        v: #wire,
                    ) -> ::core::result::Result<#source, ::std::string::String> {
                        if v.is_null() {
                            return ::core::result::Result::Err(
                                ::std::string::String::from(#null_message),
                            );
                        }
                        let __live = <#opaque as ::prebindgen_c_runtime::Transmute>::into_rust(
                            ::core::ptr::read(v),
                        );
                        #writeback
                        ::core::result::Result::Ok(__live)
                    }
                )
            }
            InputTerminalOperation::Enum {
                c_name,
                variants,
                invalid_message,
                size_message,
                align_message,
            } => {
                let arms = variants.iter().map(|variant| {
                    quote!(
                        if __raw == #c_name::#variant as ::core::ffi::c_int {
                            return ::core::result::Result::Ok(#source::#variant);
                        }
                    )
                });
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) unsafe fn #name(
                        v: #wire,
                    ) -> ::core::result::Result<#source, ::std::string::String> {
                        const _: () = {
                            assert!(
                                ::core::mem::size_of::<#c_name>()
                                    == ::core::mem::size_of::<::core::ffi::c_int>(),
                                #size_message
                            );
                            assert!(
                                ::core::mem::align_of::<#c_name>()
                                    == ::core::mem::align_of::<::core::ffi::c_int>(),
                                #align_message
                            );
                        };
                        let __raw: ::core::ffi::c_int =
                            ::core::ptr::read(v.as_ptr() as *const ::core::ffi::c_int);
                        #(#arms)*
                        ::core::result::Result::Err(::std::format!(#invalid_message, __raw))
                    }
                )
            }
            InputTerminalOperation::String => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: #wire,
                ) -> ::core::result::Result<#source, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from("null pointer passed for String argument"),
                        );
                    }
                    match ::std::ffi::CStr::from_ptr(v).to_str() {
                        ::core::result::Result::Ok(s) => {
                            ::core::result::Result::Ok(s.to_owned())
                        }
                        ::core::result::Result::Err(_) => {
                            ::core::result::Result::Err(
                                ::std::string::String::from("invalid UTF-8 in String argument"),
                            )
                        }
                    }
                }
            ),
            InputTerminalOperation::StringField => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(v: #wire) -> #source {
                    if v.is_null() {
                        ::std::string::String::new()
                    } else {
                        ::std::ffi::CStr::from_ptr(v).to_string_lossy().into_owned()
                    }
                }
            ),
            InputTerminalOperation::StrMarker => syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused_variables)]
                pub(crate) fn #name() {}
            ),
            InputTerminalOperation::Bool => {
                let read = bool_in_expr(quote!(v));
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) unsafe fn #name(v: #wire) -> #source {
                        #read
                    }
                )
            }
            InputTerminalOperation::Scalar => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #wire) -> #source {
                    v
                }
            ),
        }
    }
}

/// One C output terminal operation selected without spelling its source.
#[derive(Clone)]
pub(crate) enum OutputTerminalOperation {
    Unit,
    String,
    BoolField,
    Scalar,
    OwnedHandle {
        c_struct: syn::Ident,
    },
    OpaqueError {
        message_path: syn::Path,
    },
    ValueOpaque,
    Enum {
        c_name: syn::Ident,
        variants: Vec<syn::Ident>,
    },
}

/// A C output terminal retained until the final writer owns `Emit`.
#[derive(Clone)]
pub(crate) struct OutputTerminalPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Type,
    pub(crate) operation: OutputTerminalOperation,
}

impl OutputTerminalPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = qualify_source_type(&emit.spell_ty(&self.source), self.source_module.as_ref());
        let wire = &self.wire;
        match &self.operation {
            OutputTerminalOperation::Unit => syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused_variables)]
                pub(crate) fn #name(v: #source) {}
            ),
            OutputTerminalOperation::String => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #source) -> #wire {
                    __cbg_alloc_cstr(v)
                }
            ),
            OutputTerminalOperation::BoolField => {
                let wrap = bool_out_expr(quote!(v));
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) fn #name(v: #source) -> #wire {
                        #wrap
                    }
                )
            }
            OutputTerminalOperation::Scalar => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #source) -> #wire {
                    v
                }
            ),
            OutputTerminalOperation::OwnedHandle { c_struct } => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #source) -> #wire {
                    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut #c_struct
                }
            ),
            OutputTerminalOperation::OpaqueError { message_path } => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #source) -> #wire {
                    __cbg_alloc_cstr(#message_path(&v))
                }
            ),
            OutputTerminalOperation::ValueOpaque => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #source) -> #wire {
                    <#wire as ::prebindgen_c_runtime::Transmute>::from_rust(v)
                }
            ),
            OutputTerminalOperation::Enum { c_name, variants } => {
                let arms = variants
                    .iter()
                    .map(|variant| quote!(#source::#variant => #c_name::#variant,));
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) fn #name(v: #source) -> #wire {
                        match v { #(#arms)* }
                    }
                )
            }
        }
    }
}

/// The operation behind one canonical scalar `convert!` declaration.
///
/// The callable path and wire type belong to the adapter and are safe to freeze
/// during planning. The Rust value type remains a [`TypeRef`] in
/// [`CustomPlan`] until final rendering.
#[derive(Clone)]
pub(crate) enum CustomOperation {
    Function {
        path: syn::Path,
        by_ref: bool,
        fallible: bool,
    },
    Trait {
        fallible: bool,
    },
}

impl CustomOperation {
    fn fallible(&self) -> bool {
        match self {
            Self::Function { fallible, .. } | Self::Trait { fallible } => *fallible,
        }
    }

    fn expression(&self, direction: Direction, source: &syn::Type, wire: &syn::Type) -> syn::Expr {
        match self {
            Self::Function { path, by_ref, .. } => {
                if *by_ref {
                    syn::parse_quote!(#path(&v))
                } else {
                    syn::parse_quote!(#path(v))
                }
            }
            Self::Trait { fallible: true } => match direction {
                Direction::Construct => syn::parse_quote!(
                    <#wire as ::core::convert::TryInto<#source>>::try_into(v)
                ),
                Direction::Deconstruct => syn::parse_quote!(
                    <#source as ::core::convert::TryInto<#wire>>::try_into(v)
                ),
            },
            Self::Trait { fallible: false } => match direction {
                Direction::Construct => syn::parse_quote!(
                    <#wire as ::core::convert::Into<#source>>::into(v)
                ),
                Direction::Deconstruct => syn::parse_quote!(
                    <#source as ::core::convert::Into<#wire>>::into(v)
                ),
            },
        }
    }
}

/// One canonical scalar conversion, frozen before source spelling is allowed.
#[derive(Clone)]
pub(crate) struct CustomPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Type,
    pub(crate) direction: Direction,
    pub(crate) operation: CustomOperation,
    pub(crate) valid: Option<syn::Expr>,
    pub(crate) invalid_message: String,
}

impl CustomPlan {
    pub(crate) fn fallible(&self) -> bool {
        self.valid.is_some() || self.operation.fallible()
    }

    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let name = &self.ident;
        let source = qualify_source_type(&emit.spell_ty(&self.source), self.source_module.as_ref());
        let wire = &self.wire;
        let conversion = self.operation.expression(self.direction, &source, wire);
        let conversion_fallible = self.operation.fallible();

        match self.direction {
            Direction::Construct if self.fallible() => {
                let valid = self
                    .valid
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| syn::parse_quote!(true));
                let message = &self.invalid_message;
                let converted = if conversion_fallible {
                    quote!((#conversion).map_err(|e| e.to_string()))
                } else {
                    quote!(::core::result::Result::Ok(#conversion))
                };
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) fn #name(v: #wire)
                        -> ::core::result::Result<#source, ::std::string::String>
                    {
                        if !(#valid) {
                            return ::core::result::Result::Err(
                                ::std::string::String::from(#message)
                            );
                        }
                        #converted
                    }
                )
            }
            Direction::Construct => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #wire) -> #source {
                    #conversion
                }
            ),
            Direction::Deconstruct if self.fallible() => {
                let valid = self
                    .valid
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| syn::parse_quote!(true));
                let message = &self.invalid_message;
                let repr_expr = if conversion_fallible {
                    quote!((#conversion).map_err(|error| error.to_string())?)
                } else {
                    quote!(#conversion)
                };
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) fn #name(v: #source)
                        -> ::core::result::Result<#wire, ::std::string::String>
                    {
                        let __repr: #wire = #repr_expr;
                        if !(#valid) {
                            return ::core::result::Result::Err(
                                ::std::string::String::from(#message)
                            );
                        }
                        ::core::result::Result::Ok(__repr)
                    }
                )
            }
            Direction::Deconstruct => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #source) -> #wire {
                    #conversion
                }
            ),
        }
    }
}

/// One callback argument tied to the frozen deconstruction site that supplies
/// both its C ABI leaves and its encoder.
#[derive(Clone)]
pub(crate) struct CallbackArgument {
    pub(crate) site: SiteId,
    pub(crate) value: crate::compile::CValue,
    pub(crate) zero_copy_element: Option<syn::Type>,
    pub(crate) takeable: bool,
}

impl CallbackArgument {
    fn direct_wire(&self) -> Option<&syn::Type> {
        self.value.direct().map(|(wire, _)| wire)
    }

    pub(crate) fn wires(&self) -> Vec<syn::Type> {
        if let Some(element) = &self.zero_copy_element {
            return vec![syn::parse_quote!(*const #element), syn::parse_quote!(usize)];
        }
        if self.takeable {
            let wire = self
                .direct_wire()
                .expect("a takeable callback argument must have one direct wire");
            return vec![syn::parse_quote!(*mut #wire)];
        }
        let fields = self.value.fields();
        if self.direct_wire().is_some() {
            return fields.into_iter().map(|field| field.wire).collect();
        }
        fields
            .into_iter()
            .map(|field| {
                let wire = field.wire;
                syn::parse_quote!(::core::mem::MaybeUninit<#wire>)
            })
            .collect()
    }

    fn invoke_part(&self, index: usize) -> InvokePart {
        let source = invoke_argument_name(index);
        if let Some(element) = &self.zero_copy_element {
            return InvokePart {
                source: source.clone(),
                prepare: TokenStream::new(),
                arguments: vec![
                    quote!(#source.as_ptr() as *const #element),
                    quote!(#source.len()),
                ],
                cleanup: TokenStream::new(),
            };
        }

        let mut prepare = TokenStream::new();
        let mut arguments = Vec::new();
        let mut cleanup = TokenStream::new();
        if self.takeable {
            let (wire, converter) = self
                .value
                .direct()
                .expect("a takeable callback argument must have one direct wire");
            let target = format_ident!("__w{index}");
            let conv = converter.ident();
            let converted = if converter.fallible() {
                quote!(match #conv(#source) {
                    ::core::result::Result::Ok(__v) => __v,
                    ::core::result::Result::Err(__e) => {
                        ::core::panic!("cbindgen: callback argument conversion failed: {}", __e)
                    }
                })
            } else {
                quote!(#conv(#source))
            };
            prepare.extend(quote!(let mut #target = #converted;));
            arguments.push(quote!(&mut #target as *mut #wire));
            cleanup.extend(quote!(
                let _ = <#wire as ::prebindgen_c_runtime::Transmute>::into_rust(#target);
            ));
        } else if let Some((_, converter)) = self.value.direct() {
            let target = format_ident!("__w{index}");
            let conv = converter.ident();
            let converted = if converter.fallible() {
                quote!(match #conv(#source) {
                    ::core::result::Result::Ok(__v) => __v,
                    ::core::result::Result::Err(__e) => {
                        ::core::panic!("cbindgen: callback argument conversion failed: {}", __e)
                    }
                })
            } else {
                quote!(#conv(#source))
            };
            prepare.extend(quote!(let #target = #converted;));
            arguments.push(quote!(#target));
        } else {
            let fields = self.value.fields();
            let mut targets = Vec::new();
            for (field_index, field) in fields.iter().enumerate() {
                let target = if fields.len() == 1 {
                    format_ident!("__w{index}")
                } else {
                    format_ident!("__w{index}_{field_index}")
                };
                let wire = &field.wire;
                prepare
                    .extend(quote!(let mut #target = ::core::mem::MaybeUninit::<#wire>::zeroed();));
                targets.push(quote!(*#target.as_mut_ptr()));
                arguments.push(quote!(#target));
            }
            prepare.extend(
                self.value
                    .encode(quote!(#source), &targets, &ErrRoute::Panic),
            );
        }
        InvokePart {
            source,
            prepare,
            arguments,
            cleanup,
        }
    }
}

/// A callback declaration and its Invoke helper, frozen as one final artifact.
#[derive(Clone)]
pub(crate) struct CallbackArtifact {
    pub(crate) c_struct: syn::Ident,
    pub(crate) invoke: InvokePlan,
    pub(crate) arguments: Vec<CallbackArgument>,
}

impl CallbackArtifact {
    pub(crate) fn new(
        c_struct: syn::Ident,
        ident: syn::Ident,
        source: TypeRef,
        source_module: Option<syn::Path>,
        arguments: Vec<TypeRef>,
        planned: Vec<CallbackArgument>,
    ) -> Self {
        let wire = syn::parse_quote!(#c_struct);
        let parts = planned
            .iter()
            .enumerate()
            .map(|(index, argument)| argument.invoke_part(index))
            .collect();
        Self {
            c_struct,
            invoke: InvokePlan {
                ident,
                source,
                source_module,
                wire,
                arguments,
                parts,
            },
            arguments: planned,
        }
    }

    pub(crate) fn signature(&self) -> Vec<String> {
        self.arguments
            .iter()
            .flat_map(CallbackArgument::wires)
            .map(|wire| TypeKey::from_type(&wire).as_str().to_owned())
            .collect()
    }

    pub(crate) fn render(&self, emit: &Emit) -> Vec<syn::Item> {
        let c_struct = &self.c_struct;
        let arg_wires: Vec<syn::Type> = self
            .arguments
            .iter()
            .flat_map(CallbackArgument::wires)
            .collect();
        let declaration = syn::parse_quote!(
            #[repr(C)]
            #[allow(non_camel_case_types)]
            pub struct #c_struct {
                pub context: *mut ::core::ffi::c_void,
                pub call: ::core::option::Option<
                    unsafe extern "C" fn(#(#arg_wires,)* *mut ::core::ffi::c_void),
                >,
                pub drop: ::core::option::Option<
                    unsafe extern "C" fn(*mut ::core::ffi::c_void),
                >,
            }
        );
        vec![declaration, syn::Item::Fn(self.invoke.render(emit))]
    }
}

/// Adapter-owned final artifacts stored and ordered by the registry plan.
pub(crate) enum CArtifact {
    Callback(CallbackArtifact),
    OpaqueHandle(OpaqueHandleArtifact),
    ValueOpaque(ValueOpaqueArtifact),
}

impl CArtifact {
    pub(crate) fn render(&self, emit: &Emit) -> Vec<syn::Item> {
        match self {
            Self::Callback(callback) => callback.render(emit),
            Self::OpaqueHandle(handle) => handle.render(emit),
            Self::ValueOpaque(value) => value.render(emit),
        }
    }
}

/// Render one class of C artifacts from the frozen registry plan.
///
/// The deliberately narrow signature is the artifact-rendering boundary: final
/// emission can spell retained source types through Emit, but it cannot
/// reopen the registry or the adapter's mutable compilation cache.
pub(crate) fn render_artifacts(
    generation: &GenerationPlan<crate::compile::CRepresentation>,
    kind: &str,
    emit: &Emit,
) -> Vec<syn::Item> {
    generation
        .artifacts()
        .filter(|artifact| artifact.id().kind() == kind)
        .flat_map(|artifact| artifact.payload().render(emit))
        .collect()
}

/// One opaque C handle declaration and its typed destructor.
pub(crate) struct OpaqueHandleArtifact {
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) c_struct: syn::Ident,
    pub(crate) drop_ident: syn::Ident,
}

impl OpaqueHandleArtifact {
    fn render(&self, emit: &Emit) -> Vec<syn::Item> {
        let source = qualify_source_type(&emit.spell_ty(&self.source), self.source_module.as_ref());
        let c_struct = &self.c_struct;
        let drop_ident = &self.drop_ident;
        vec![
            syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub struct #c_struct {
                    _private: [u8; 0],
                }
            ),
            syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub unsafe extern "C" fn #drop_ident(this_: *mut #c_struct) {
                    if !this_.is_null() {
                        drop(::std::boxed::Box::from_raw(this_ as *mut #source));
                    }
                }
            ),
        ]
    }
}

/// A generated visible mirror and whether it needs a full gravestone.
pub(crate) struct ValueOpaqueMirror {
    pub(crate) ident: syn::Ident,
    pub(crate) fields: Vec<(syn::Ident, syn::Type)>,
    pub(crate) gravestone: bool,
}

/// The optional public move helper for a takeable value-opaque type.
pub(crate) struct ValueOpaqueTake {
    pub(crate) ident: syn::Ident,
    pub(crate) writeback: ValueOpaqueWriteback,
}

/// One value-opaque declaration family retained until final source spelling.
pub(crate) struct ValueOpaqueArtifact {
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) opaque: syn::Type,
    pub(crate) mirror: Option<ValueOpaqueMirror>,
    pub(crate) drop_ident: syn::Ident,
    pub(crate) take: Option<ValueOpaqueTake>,
}

impl ValueOpaqueArtifact {
    fn render(&self, emit: &Emit) -> Vec<syn::Item> {
        let source = qualify_source_type(&emit.spell_ty(&self.source), self.source_module.as_ref());
        let opaque = &self.opaque;
        let mut items = Vec::new();
        if let Some(mirror) = &self.mirror {
            let ident = &mirror.ident;
            let fields = mirror
                .fields
                .iter()
                .map(|(name, wire)| quote!(pub #name: #wire));
            items.push(syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub struct #ident {
                    #(#fields,)*
                }
            ));
            if mirror.gravestone {
                items.push(syn::parse_quote!(
                    impl ::prebindgen_c_runtime::Gravestone for #ident {
                        #[inline]
                        fn rust_gravestone() -> #source {
                            <#source as ::core::default::Default>::default()
                        }
                    }
                ));
            }
        }
        items.push(syn::parse_quote!(
            const _: () = {
                assert!(
                    ::core::mem::size_of::<#source>() == ::core::mem::size_of::<#opaque>(),
                    "value_opaque: Rust type and opaque counterpart differ in size"
                );
                assert!(
                    ::core::mem::align_of::<#source>() == ::core::mem::align_of::<#opaque>(),
                    "value_opaque: Rust type and opaque counterpart differ in alignment"
                );
            };
        ));
        items.push(syn::parse_quote!(
            impl ::prebindgen_c_runtime::Transmute for #opaque {
                type Rust = #source;
                #[inline]
                fn from_rust(value: Self::Rust) -> Self {
                    let __v = ::core::mem::ManuallyDrop::new(value);
                    unsafe {
                        ::core::ptr::read(&*__v as *const Self::Rust as *const Self)
                    }
                }
                #[inline]
                fn into_rust(self) -> Self::Rust {
                    let __v = ::core::mem::ManuallyDrop::new(self);
                    unsafe {
                        ::core::ptr::read(&*__v as *const Self as *const Self::Rust)
                    }
                }
                #[inline]
                fn as_rust(&self) -> &Self::Rust {
                    unsafe { &*(self as *const Self as *const Self::Rust) }
                }
                #[inline]
                fn as_rust_mut(&mut self) -> &mut Self::Rust {
                    unsafe { &mut *(self as *mut Self as *mut Self::Rust) }
                }
            }
        ));
        let drop_ident = &self.drop_ident;
        items.push(syn::parse_quote!(
            #[no_mangle]
            #[allow(non_snake_case, unused_variables)]
            pub unsafe extern "C" fn #drop_ident(this_: *mut #opaque) {
                if !this_.is_null() {
                    ::core::ptr::drop_in_place(
                        <#opaque as ::prebindgen_c_runtime::Transmute>::as_rust_mut(&mut *this_),
                    );
                }
            }
        ));
        if let Some(take) = &self.take {
            let take_ident = &take.ident;
            let src = format_ident!("src");
            let writeback = take.writeback.render(&src, opaque);
            items.push(syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub unsafe extern "C" fn #take_ident(
                    dst: *mut #opaque,
                    src: *mut #opaque,
                ) {
                    if dst.is_null() || src.is_null() {
                        return;
                    }
                    ::core::ptr::write(dst, ::core::ptr::read(src));
                    #writeback
                }
            ));
        }
        items
    }
}

/// One C callback argument's already-resolved wire delivery.
#[derive(Clone)]
pub(crate) struct InvokePart {
    pub(crate) source: syn::Ident,
    pub(crate) prepare: TokenStream,
    pub(crate) arguments: Vec<TokenStream>,
    pub(crate) cleanup: TokenStream,
}

impl chain::InvokePart for InvokePart {
    fn render(&self, value: &syn::Ident, index: usize, _emit: &Emit) -> chain::RenderedInvokePart {
        assert_eq!(value, &invoke_argument_name(index));
        assert_eq!(value, &self.source);
        chain::RenderedInvokePart {
            prepare: self.prepare.clone(),
            arguments: self.arguments.clone(),
            cleanup: self.cleanup.clone(),
        }
    }
}

#[derive(Clone)]
struct CInvokeBridge {
    wire: syn::Type,
}

pub(crate) fn invoke_argument_name(index: usize) -> syn::Ident {
    format_ident!("__a{}", index)
}

impl chain::InvokeBridge for CInvokeBridge {
    fn intermediate(&self) -> syn::Type {
        self.wire.clone()
    }

    fn value_name(&self) -> syn::Ident {
        format_ident!("c")
    }

    fn argument_name(&self, index: usize) -> syn::Ident {
        invoke_argument_name(index)
    }

    fn capture(&self, value: TokenStream, closure: TokenStream) -> TokenStream {
        quote!({
            struct __Ctx {
                context: *mut ::core::ffi::c_void,
                drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
            }
            unsafe impl ::core::marker::Send for __Ctx {}
            unsafe impl ::core::marker::Sync for __Ctx {}
            impl ::core::ops::Drop for __Ctx {
                fn drop(&mut self) {
                    if let ::core::option::Option::Some(__d) = self.drop {
                        unsafe { __d(self.context) }
                    }
                }
            }
            let __call = #value.call;
            let __ctx = ::std::sync::Arc::new(__Ctx {
                context: #value.context,
                drop: #value.drop,
            });
            #closure
        })
    }

    fn invoke(&self, arguments: &[TokenStream]) -> TokenStream {
        quote!(unsafe { __f(#(#arguments,)* __ctx.context) })
    }

    fn surround(
        &self,
        prepare: TokenStream,
        invoke: TokenStream,
        cleanup: TokenStream,
    ) -> TokenStream {
        // Encode inside the NULL-call guard. A closure whose `call` is NULL
        // receives nothing; encoding before the guard would leak every
        // allocation produced for an undelivered value (for example a
        // `Vec`/`Cow` array or `String` buffer). Invoke owns phase order, but
        // this bridge must keep the whole prepare/invoke/cleanup sequence
        // inside the target-specific guard (#428 review).
        quote!({
            if let ::core::option::Option::Some(__f) = __call {
                #prepare
                #invoke
                #cleanup
            }
        })
    }

    fn fallible(&self) -> bool {
        false
    }
}

/// A callback chain. Its `impl Fn(..)` spelling remains opaque until render.
#[derive(Clone)]
pub(crate) struct InvokePlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Type,
    pub(crate) arguments: Vec<TypeRef>,
    pub(crate) parts: Vec<InvokePart>,
}

impl InvokePlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let plan = chain::Invoke {
            source: self.source.clone(),
            arguments: self.arguments.clone(),
            source_policy: CSource {
                module: self.source_module.clone(),
            },
            bridge: CInvokeBridge {
                wire: self.wire.clone(),
            },
            parts: self.parts.clone(),
        };
        let rendered = plan.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let syn::Expr::Block(body) = &rendered.body else {
            unreachable!("the C Invoke bridge captures its callable in a block")
        };
        let block = &body.block;
        syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(c: #intermediate) -> #source #block
        )
    }
}

/// One field step in a product converter chain.
#[derive(Clone)]
pub(crate) struct ProductField {
    pub(crate) name: syn::Ident,
    pub(crate) converter: CCall,
    pub(crate) mode: Mode,
    pub(crate) hold_uninit: bool,
}

#[derive(Clone)]
struct CSource {
    module: Option<syn::Path>,
}

impl chain::Source for CSource {
    fn spell(&self, source: &TypeRef, emit: &Emit) -> syn::Type {
        qualify_source_type(&emit.spell_ty(source), self.module.as_ref())
    }
}

#[derive(Clone)]
struct CProductBridge {
    wire: syn::Type,
}

impl chain::ProductBridge for CProductBridge {
    fn intermediate(&self) -> syn::Type {
        self.wire.clone()
    }

    fn part(&self, value: TokenStream, _index: usize, name: &syn::Ident) -> TokenStream {
        quote!(#value.#name)
    }

    fn build(&self, parts: &[(syn::Ident, TokenStream)]) -> TokenStream {
        let wire = &self.wire;
        let names = parts.iter().map(|(name, _)| name);
        let values = parts.iter().map(|(_, value)| value);
        quote!(#wire { #(#names: #values),* })
    }
}

/// A product chain. Source syntax stays behind `source` until `render`.
#[derive(Clone)]
pub(crate) struct ProductPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Type,
    pub(crate) direction: Direction,
    pub(crate) fields: Vec<ProductField>,
}

impl ProductPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let chain = chain::Product {
            source: self.source.clone(),
            direction: self.direction,
            source_policy: CSource {
                module: self.source_module.clone(),
            },
            bridge: CProductBridge {
                wire: self.wire.clone(),
            },
            parts: self
                .fields
                .iter()
                .map(|field| chain::ProductPart {
                    name: field.name.clone(),
                    child: field.converter.clone(),
                    mode: field.mode,
                    hold_uninit: field.hold_uninit,
                })
                .collect(),
        };
        let rendered = chain.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let body = &rendered.body;

        match (self.direction, rendered.fallible) {
            (Direction::Construct, true) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: #intermediate,
                ) -> ::core::result::Result<#source, ::std::string::String> {
                    ::core::result::Result::Ok(#body)
                }
            ),
            (Direction::Construct, false) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(v: #intermediate) -> #source {
                    #body
                }
            ),
            (Direction::Deconstruct, true) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(
                    v: #source,
                ) -> ::core::result::Result<#intermediate, ::std::string::String> {
                    ::core::result::Result::Ok(#body)
                }
            ),
            (Direction::Deconstruct, false) => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #source) -> #intermediate {
                    #body
                }
            ),
        }
    }
}

/// How an inbound optional distinguishes absence from a present child.
#[derive(Clone)]
pub(crate) enum OptionalRepr {
    Niche { absent: syn::Expr },
    Nullable { read_direct: bool },
}

#[derive(Clone)]
struct COptionalBridge {
    wire: syn::Type,
    repr: OptionalRepr,
}

impl chain::OptionalBridge for COptionalBridge {
    fn intermediate(&self) -> syn::Type {
        self.wire.clone()
    }

    fn is_absent(&self) -> TokenStream {
        match &self.repr {
            OptionalRepr::Niche { absent } => quote!(#absent),
            OptionalRepr::Nullable { .. } => quote!((v).is_null()),
        }
    }

    fn present(&self, value: TokenStream) -> TokenStream {
        match self.repr {
            OptionalRepr::Niche { .. } | OptionalRepr::Nullable { read_direct: true } => value,
            OptionalRepr::Nullable { read_direct: false } => quote!(::core::ptr::read(#value)),
        }
    }

    fn build_absent(&self) -> TokenStream {
        unreachable!("optional bridge operation does not match its planned direction")
    }

    fn build_present(&self, _child: TokenStream) -> TokenStream {
        unreachable!("optional bridge operation does not match its planned direction")
    }
}

/// An optional chain. Its source `Option<T>` remains opaque until rendering.
#[derive(Clone)]
pub(crate) struct OptionalPlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Type,
    pub(crate) converter: CCall,
    pub(crate) repr: OptionalRepr,
    pub(crate) borrowed: bool,
}

impl OptionalPlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let chain = chain::Optional {
            source: self.source.clone(),
            direction: Direction::Construct,
            source_policy: CSource {
                module: self.source_module.clone(),
            },
            bridge: COptionalBridge {
                wire: self.wire.clone(),
                repr: self.repr.clone(),
            },
            child: self.converter.clone(),
        };
        let rendered = chain.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let body = &rendered.body;
        let lifetime = self.borrowed.then(|| quote!(<'a>));

        if rendered.fallible {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name #lifetime(v: #intermediate)
                    -> ::core::result::Result<#source, ::std::string::String>
                {
                    ::core::result::Result::Ok(#body)
                }
            )
        } else {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name #lifetime(v: #intermediate) -> #source {
                    #body
                }
            )
        }
    }
}

/// A Vec of one child wire for one registry-owned Sequence loop.
#[derive(Clone)]
struct CSequenceBridge {
    child: syn::Type,
}

impl chain::SequenceBridge for CSequenceBridge {
    fn intermediate(&self) -> syn::Type {
        let child = &self.child;
        syn::parse_quote!(::std::vec::Vec<#child>)
    }

    fn begin(&self, _value: TokenStream) -> TokenStream {
        unreachable!("C Sequence input is represented by the specialized slice path")
    }

    fn next(&self) -> TokenStream {
        unreachable!("C Sequence input is represented by the specialized slice path")
    }

    fn begin_output(&self, source: TokenStream) -> TokenStream {
        let child = &self.child;
        quote!(
            let mut __sequence_output: ::std::vec::Vec<#child> =
                ::std::vec::Vec::with_capacity((#source).len());
        )
    }

    fn push(&self, value: TokenStream) -> TokenStream {
        quote!(__sequence_output.push(#value);)
    }

    fn finish(&self) -> TokenStream {
        quote!(__sequence_output)
    }

    fn fallible(&self) -> bool {
        false
    }
}

/// One C Vec-output converter composed by the registry.
#[derive(Clone)]
pub(crate) struct SequencePlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) element: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) child_wire: syn::Type,
    pub(crate) child: CCall,
}

impl SequencePlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let composed = chain::Sequence {
            source: self.source.clone(),
            element: self.element.clone(),
            direction: Direction::Deconstruct,
            source_policy: CSource {
                module: self.source_module.clone(),
            },
            bridge: CSequenceBridge {
                child: self.child_wire.clone(),
            },
            child: self.child.clone(),
        };
        let rendered = composed.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let body = &rendered.body;
        let unsafe_ = self.child.unsafe_().then(|| quote!(unsafe));
        if rendered.fallible {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                #[inline(always)]
                pub(crate) #unsafe_ fn #name(v: #source)
                    -> ::core::result::Result<#intermediate, ::std::string::String>
                {
                    ::core::result::Result::Ok(#body)
                }
            )
        } else {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                #[inline(always)]
                pub(crate) #unsafe_ fn #name(v: #source) -> #intermediate {
                    #body
                }
            )
        }
    }
}
#[derive(Clone)]
struct CChoiceBridge {
    wire: syn::Ident,
    alternatives: Vec<Alternative>,
}

impl chain::ChoiceBridge for CChoiceBridge {
    fn intermediate(&self) -> syn::Type {
        let wire = &self.wire;
        syn::parse_quote!(::core::mem::MaybeUninit<#wire>)
    }

    fn tag(&self, value: TokenStream) -> TokenStream {
        let wire = &self.wire;
        let bounds_msg = format!(
            "`{wire}`: a #[repr(C)] enum with payload variants must be at least as large as its C `int` discriminant"
        );
        quote!({
            const _: () = {
                assert!(
                    ::core::mem::size_of::<#wire>()
                        >= ::core::mem::size_of::<::core::ffi::c_int>(),
                    #bounds_msg
                );
            };
            unsafe {
                ::core::ptr::read((#value).as_ptr() as *const ::core::ffi::c_int)
            }
        })
    }

    fn prepare(&self, value: TokenStream) -> TokenStream {
        quote!(unsafe { (#value).assume_init() })
    }

    fn arm(&self, emit: &Emit, value: TokenStream, index: usize) -> TokenStream {
        let wire = &self.wire;
        let alternative = &self.alternatives[index];
        let variant = &alternative.name;
        let bindings: Vec<_> = alternative
            .fields
            .iter()
            .enumerate()
            .map(|(index, _)| format_ident!("__wire_part{index}"))
            .collect();
        let bound: Vec<_> = alternative
            .fields
            .iter()
            .zip(&bindings)
            .map(|(field, binding)| field.bind(binding))
            .collect();
        let pattern = emit.shape_alternative(alternative, quote!(#wire::#variant), &bound);
        quote!({
            match #value {
                #pattern => (#(#bindings,)*),
                _ => unreachable!("validated Choice tag selected a different arm"),
            }
        })
    }

    fn build(&self, emit: &Emit, active: usize, value: TokenStream) -> TokenStream {
        let wire = &self.wire;
        let alternative = &self.alternatives[active];
        let variant = &alternative.name;
        if alternative.fields.is_empty() {
            return emit.shape_alternative(alternative, quote!(#wire::#variant), &[]);
        }
        let fields: Vec<_> = alternative
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let index = syn::Index::from(index);
                field.bind(&quote!(__built_arm.#index))
            })
            .collect();
        let built = emit.shape_alternative(alternative, quote!(#wire::#variant), &fields);
        quote!({
            let __built_arm = #value;
            #built
        })
    }

    fn finish(&self, value: TokenStream) -> TokenStream {
        quote!(::core::mem::MaybeUninit::new(#value))
    }

    fn invalid_tag(&self, tag: TokenStream) -> TokenStream {
        let wire = &self.wire;
        let variants = self.alternatives.len();
        quote!(::std::format!(
            "invalid tag {} for `{}` (expected 0..{})",
            #tag,
            stringify!(#wire),
            #variants,
        ))
    }
}

/// A tagged-union Choice chain. Raw C storage is represented by its bridge;
/// source syntax remains behind `source` until `render`.
#[derive(Clone)]
pub(crate) struct ChoicePlan {
    pub(crate) ident: syn::Ident,
    pub(crate) source: TypeRef,
    pub(crate) source_module: Option<syn::Path>,
    pub(crate) wire: syn::Ident,
    pub(crate) direction: Direction,
    pub(crate) arms: Vec<chain::ChoiceArm<chain::TupleProduct, CCall>>,
}

impl ChoicePlan {
    fn render(&self, emit: &Emit) -> syn::ItemFn {
        let composed = chain::Choice {
            source: self.source.clone(),
            direction: self.direction,
            source_policy: CSource {
                module: self.source_module.clone(),
            },
            bridge: CChoiceBridge {
                wire: self.wire.clone(),
                alternatives: self
                    .arms
                    .iter()
                    .map(|arm| arm.alternative.clone())
                    .collect(),
            },
            arms: self.arms.clone(),
        };
        let rendered = composed.render(emit);
        let name = &self.ident;
        let source = &rendered.source;
        let intermediate = &rendered.intermediate;
        let body = &rendered.body;

        match self.direction {
            Direction::Construct => syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: #intermediate,
                ) -> ::core::result::Result<#source, ::std::string::String> {
                    ::core::result::Result::Ok(#body)
                }
            ),
            Direction::Deconstruct => {
                assert!(
                    !rendered.fallible,
                    "fallible C Choice output must be refused while planning"
                );
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) fn #name(v: #source) -> #intermediate {
                        #body
                    }
                )
            }
        }
    }
}
