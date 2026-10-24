//! Generate Rust wrappers and C declarations from conversion slots.
#![warn(missing_docs)]

use prebindgen_conversions::value::Value;
use prebindgen_flat::{Conversion, Flat, Primitive, Result, WireKind};

mod wire;
pub use wire::CWire;
/// Pointee type and mutability of a C boundary pointer.
#[derive(Clone, Debug)]
pub struct CPointer {
    /// Rust pointee type used in the ABI and the opaque C type declaration.
    pub ty: syn::Type,
    /// Whether this carrier is a mutable pointer.
    pub mutable: bool,
}
impl From<prebindgen_conversions::boxed::BoxedWire> for CPointer {
    fn from(value: prebindgen_conversions::boxed::BoxedWire) -> Self {
        Self {
            ty: value.ty,
            mutable: value.access != prebindgen_conversions::Access::Shared,
        }
    }
}
mod enums;
mod errors;
mod group;
mod sequence;
use group::validate_layout;
pub use group::{CGroup, COptional};
use prebindgen_declarations::Codec;
use prebindgen_flat::Wire as GenericWire;
mod emission;
pub use emission::CContext;
type InputPlan<W> = prebindgen_conversions::InputPlan<W, CContext, CGroup>;
type OutputPlan<W> = prebindgen_conversions::OutputPlan<W, CContext, CGroup>;
type GenericBindings<W> = Flat<InputPlan<W>, OutputPlan<W>>;
/// Flat source model with C conversion data for wrapper emission.
pub type Bindings = GenericBindings<CWire>;
type Wire = GenericWire<CWire>;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Conversions and slot glue used by [`Cbindgen`].
/// Prepared code uses explicit emission parameters for the output pointer and
/// result type. The writer supplies these bindings and the error boundary.
/// Use with [`prebindgen_declarations::bind_input`] and [`prebindgen_declarations::bind_output`]
/// to fill [`Bindings`] manually.
///
/// ```
/// use prebindgen_c::{Bindings, Cbindgen, CCodec};
/// use prebindgen_declarations::{bind_input, bind_output, Input, Output};
/// let source = syn::parse_file("pub fn echo(x: i64) -> i64 { x }")?;
/// let mut flat = Bindings::parse(
///     source.items.into_iter().map(|item| (item, Default::default())),
/// )?;
/// bind_input(&mut flat, &CCodec, "echo", "x", &Input::Leaf, &())?;
/// bind_output(&mut flat, &CCodec, "echo", &Output::Leaf, &())?;
/// let generated = Cbindgen::new(flat).build()?;
/// assert!(generated.header.contains("pb_echo("));
/// syn::parse_file(&generated.rust)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct CCodec;
impl Codec for CCodec {
    type Niche = CNiche;
    type Group = CGroup;

    fn binding_failure_layout(
        &self,
        _: &(),
    ) -> Result<Option<Vec<prebindgen_flat::WireNode<Self::Wire, Self::Group>>>> {
        Ok(Some(vec![prebindgen_flat::WireNode::Wire(
            prebindgen_flat::Wire {
                name: syn::parse_quote!(message),
                kind: CWire::Buffer,
            },
        )]))
    }
    fn optional_metadata(
        &self,
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<CNiche>,
        controls: Vec<prebindgen_flat::Wire<CWire>>,
    ) -> CGroup {
        CGroup::Optional(COptional {
            controls,
            presence,
            discriminator,
            niche,
        })
    }
    fn alternatives_metadata(&self, selector: prebindgen_flat::Wire<CWire>) -> CGroup {
        CGroup::Alternatives(selector)
    }
    fn field_group(&self, name: syn::Ident) -> CGroup {
        CGroup::Field(name)
    }
    fn fields_group(&self) -> CGroup {
        CGroup::Fields
    }
    type Extra = ();
    type Wire = CWire;
    type Context = CContext;
    type Input = InputPlan<CWire>;
    type Output = OutputPlan<CWire>;
    fn finish_input(&self, plan: &mut InputPlan<CWire>, _: &()) -> Result<()> {
        validate_layout(&plan.nodes)?;
        if plan
            .resources
            .iter()
            .any(|r| r.access == prebindgen_conversions::Access::Move)
        {
            let accepted = prebindgen_conversions::template_parameter("acknowledgement");
            plan.acknowledgement = Some(quote!(#accepted = true;));
        }
        Ok(())
    }
    fn finish_output(&self, plan: &mut OutputPlan<CWire>, _: &()) -> Result<()> {
        validate_layout(&plan.nodes)?;
        plan.commit
            .extend(prebindgen_conversions::template_parameter(
                "output_committed",
            ));
        let mut values: Vec<_> = crate::group::result_packet(&plan.nodes, &plan.values)?
            .into_iter()
            .enumerate()
            .map(|(i, (_, value))| {
                let field = format_ident!("v{i}");
                quote!(#field: #value)
            })
            .collect();
        if values.is_empty() {
            values.push(quote!(unused: 0));
        }
        let result_type = prebindgen_conversions::template_parameter("output_type");
        let out = prebindgen_conversions::template_parameter("output_pointer");
        plan.delivery = prebindgen_conversions::Delivery {
            statements: quote!(let result = #result_type { #(#values),* }; std::ptr::write(#out, result);),
            result: quote!(()),
        };
        Ok(())
    }
    fn sequence_input(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        index: syn::Ident,
        element: &[prebindgen_flat::WireNode<CWire, CGroup>],
        _: &prebindgen_declarations::Condition,
    ) -> Result<prebindgen_declarations::SequenceInput<CGroup>> {
        sequence::input(wire, length, index, element)
    }
    fn sequence_output(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        _: syn::Ident,
        element: &[prebindgen_flat::WireNode<CWire, CGroup>],
        values: &[(syn::Ident, TokenStream)],
    ) -> Result<prebindgen_declarations::SequenceOutput<CGroup>> {
        sequence::output(wire, length, element, values)
    }
    fn result_error(&self, _: &syn::Type, message: TokenStream, _: &()) -> Result<TokenStream> {
        let pointer = prebindgen_conversions::template_parameter("error_pointer");
        let reported = prebindgen_conversions::template_parameter("error_reported");
        Ok(
            quote!({if !#pointer.is_null(){unsafe {__pb_report_message(#message,#pointer)};}#reported=true;String::new()}),
        )
    }
    fn niche(&self, wire: &CWire) -> Option<CNiche> {
        wire.available_niche()
    }
    fn set_niche(&self, wire: &mut CWire, niche: Option<CNiche>) {
        wire.set_niche(niche)
            .expect("C codec supplied an incompatible niche");
    }
    fn is_present(&self, wire: &CWire, niche: &CNiche, value: TokenStream) -> Result<TokenStream> {
        wire.validate_niche(niche)?;
        Ok(niche.present(value))
    }
    fn remaining_niche(&self, wire: &CWire, niche: &CNiche) -> Option<CNiche> {
        wire.remaining_niche(niche)
    }
    fn absent_value(&self, wire: &CWire, niche: Option<&CNiche>, _: bool) -> Result<TokenStream> {
        if let Some(niche) = niche {
            wire.validate_niche(niche)?;
        }
        let buffer_type = prebindgen_conversions::template_parameter("buffer_type");
        let wide = prebindgen_conversions::template_parameter("wide_integer_type");
        Ok(match wire {
            CWire::Pointer { .. } => quote!(std::ptr::null_mut()),
            CWire::Buffer => quote!(#buffer_type {data:std::ptr::null(),len:0}),
            CWire::Rows => quote!(__PbRows {
                data: std::ptr::null(),
                len: 0
            }),
            CWire::Scalar {
                primitive: Primitive::I128 | Primitive::U128,
                ..
            } => quote!(#wide {hi:0,lo:0}),
            CWire::Scalar {
                primitive: Primitive::F32 | Primitive::F64,
                ..
            } => quote!(0.0),
            CWire::Scalar { .. } | CWire::Enum { .. } => niche
                .map(|n| n.integer_value())
                .unwrap_or_else(|| quote!(0)),
        })
    }
    fn selector(&self, _: &()) -> CWire {
        CWire::scalar(Primitive::I32)
    }
    fn consumes(&self, ty: &syn::Type, _: &()) -> bool {
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            return self.consumes(inner, &());
        }
        Value::of(ty).is_none()
    }
    fn input_leaf(
        &self,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: syn::Ident,
        active: &prebindgen_declarations::Condition,
    ) -> Result<InputPlan<CWire>> {
        if prebindgen_flat::sequence_parts(ty).is_some() {
            return Err("collections require a Sequence recipe, not an opaque leaf; use an alias for opaque storage".into());
        }
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            let tag = quote::format_ident!("{w}_present");
            let child = self.input_leaf(
                inner,
                &(),
                quote::format_ident!("{w}_some"),
                quote::format_ident!("{v}_some"),
                &active.arm(tag.clone(), 1),
            )?;
            return prebindgen_declarations::optional_input(self, &(), tag, v, active, child);
        }
        let read_buffer = prebindgen_conversions::template_parameter("read_buffer");
        let (decode, kind) = match Value::of(ty) {
            Some(Value::Scalar(p)) => (decode_scalar(p, &w), CWire::scalar(p)),
            // Text and bytes borrow the caller's buffer; owned targets copy it.
            Some(Value::Text) => (
                quote!(std::str::from_utf8(unsafe {#read_buffer(#w)?}).map_err(|_|"invalid UTF-8")?),
                CWire::Buffer,
            ),
            Some(Value::Bytes) => (quote!(unsafe {#read_buffer(#w)?}), CWire::Buffer),
            None => {
                return prebindgen_conversions::boxed::input(
                    &prebindgen_conversions::boxed::Storage::Direct,
                    ty,
                    w,
                    v,
                    &active.rust(),
                    |b| CWire::pointer(b.into()),
                )
            }
        };
        let mut plan =
            prebindgen_conversions::value::input(ty, w, v, &active.rust(), decode, kind)?;
        plan.fallible = !matches!(Value::of(ty), Some(Value::Scalar(p)) if p != Primitive::Char);
        Ok(plan)
    }
    fn output_leaf(
        &self,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<CWire>> {
        if prebindgen_flat::sequence_parts(ty).is_some() {
            return Err("collections require a Sequence recipe, not an opaque leaf; use an alias for opaque storage".into());
        }
        if let syn::Type::Reference(reference) = ty {
            if reference.mutability.is_some() {
                return Err("mutable-reference outputs are unsupported".into());
            }
            match prebindgen_flat::type_key(&reference.elem).as_str() {
                "str" => {
                    return self.output_leaf(
                        &syn::parse_quote!(String),
                        &(),
                        w,
                        quote!((#v).to_string()),
                        false,
                    )
                }
                "[u8]" => {
                    return self.output_leaf(
                        &syn::parse_quote!(Vec<u8>),
                        &(),
                        w,
                        quote!((#v).to_vec()),
                        false,
                    )
                }
                _ => {}
            }
            let value = if borrowed { quote!(*#v) } else { v };
            return self.output_leaf(&reference.elem, &(), w, value, true);
        }
        if let Some(inner) = prebindgen_declarations::option_inner(ty) {
            let tag = quote::format_ident!("{w}_present");
            let payload = quote::format_ident!("{w}_payload");
            let child = self.output_leaf(
                inner,
                &(),
                quote::format_ident!("{w}_some"),
                quote!(#payload),
                borrowed,
            )?;
            return prebindgen_declarations::optional_output(
                self,
                &(),
                tag,
                payload,
                v,
                borrowed,
                child,
            );
        }
        match Value::of(ty) {
            Some(Value::Scalar(p)) => {
                let mut plan = prebindgen_conversions::value::output(
                    ty,
                    w,
                    v,
                    borrowed,
                    |v| encode_scalar(p, v),
                    CWire::scalar(p),
                )?;
                plan.fallible = false;
                Ok(plan)
            }
            Some(_) => {
                // The buffer stays owned by Rust until every output is delivered.
                let mut out = prebindgen_conversions::value::output(
                    ty,
                    w.clone(),
                    v,
                    borrowed,
                    |v| quote!(::core::convert::Into::<Vec<u8>>::into(#v).into_boxed_slice()),
                    CWire::Buffer,
                )?;
                out.fallible = false;
                let buffer_type = prebindgen_conversions::template_parameter("buffer_type");
                out.values = vec![(
                    w.clone(),
                    quote!(#buffer_type {data: #w.as_ptr(), len: #w.len()}),
                )];
                out.commit = quote!(let _ = Box::into_raw(#w););
                Ok(out)
            }
            None => prebindgen_conversions::boxed::output(
                &prebindgen_conversions::boxed::Storage::Direct,
                ty,
                w,
                v,
                borrowed,
                |b| CWire::pointer(b.into()),
            ),
        }
    }

    fn optional_niche(
        &self,
        payload: &[prebindgen_flat::WireNode<Self::Wire, Self::Group>],
    ) -> Option<prebindgen_declarations::NicheReservation<Self::Niche>> {
        crate::group::optional_niche(payload)
    }
    fn optional_presence(
        &self,
        payload: &[prebindgen_flat::WireNode<Self::Wire, Self::Group>],
        reservation: &prebindgen_declarations::NicheReservation<Self::Niche>,
    ) -> Result<TokenStream> {
        crate::group::optional_presence(payload, reservation)
    }
    fn absent_output_values(
        &self,
        payload: &[prebindgen_flat::WireNode<Self::Wire, Self::Group>],
        reservation: Option<&prebindgen_declarations::NicheReservation<Self::Niche>>,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        crate::group::absent_output_values(payload, reservation)
    }
    fn retire_niches(
        &self,
        payload: &mut Vec<prebindgen_flat::WireNode<Self::Wire, Self::Group>>,
        reservation: Option<&prebindgen_declarations::NicheReservation<Self::Niche>>,
    ) -> Result<()> {
        crate::group::retire_niches(payload, reservation)
    }
    fn validate_input(&self, plan: &prebindgen_declarations::CodecInput<Self>) -> Result<()> {
        crate::group::validate_layout(&plan.nodes)
    }
    fn validate_output(&self, plan: &prebindgen_declarations::CodecOutput<Self>) -> Result<()> {
        crate::group::validate_layout(&plan.nodes)?;
        crate::group::result_packet(&plan.nodes, &plan.values)?;
        Ok(())
    }
}
fn decode_scalar(p: Primitive, w: &syn::Ident) -> TokenStream {
    let ty = p.rust_type();
    match p {
        Primitive::Bool => quote!(#w != 0),
        Primitive::Char => quote!(char::from_u32(#w).ok_or("invalid Unicode scalar")?),
        Primitive::I128 | Primitive::U128 => {
            quote!(((u128::from(#w.hi) << 64) | u128::from(#w.lo)) as #ty)
        }
        _ => quote!(#w),
    }
}
fn encode_scalar(p: Primitive, v: TokenStream) -> TokenStream {
    let wide_integer_type = prebindgen_conversions::template_parameter("wide_integer_type");
    match p {
        Primitive::Bool => quote!(u8::from(#v)),
        Primitive::Char => quote!((#v) as u32),
        Primitive::I128 | Primitive::U128 => {
            quote!({ let bits = (#v) as u128; #wide_integer_type { hi: (bits >> 64) as u64, lo: bits as u64 } })
        }
        _ => v,
    }
}
/// Configure C bindings, using typed opaque pointers for foreign values.
///
/// Start with captured source via [`Self::builder`] and [`Self::source`], or a
/// flat model via [`Self::new`]. With captured source, declare each
/// exported function. In either case, declare each opaque pointee type.
/// Conversion declarations use [`prebindgen_declarations::Declarations`]. When reading
/// captured source, add the source crate as both a build dependency and a normal
/// dependency: generation reads its captures, and generated code calls its functions.
///
/// ```
/// use prebindgen_c::{Bindings, Cbindgen};
/// use prebindgen_declarations::fun;
/// let source = syn::parse_file("pub fn add(a: i64, b: i64) -> i64 { a + b }")?;
/// let flat = Bindings::parse(
///     source.items.into_iter().map(|item| (item, Default::default())),
/// )?;
/// let generated = Cbindgen::new(flat)
///     .prefix("demo")
///     .function(fun!(add))
///     .build()?;
/// assert!(generated.header.contains("demo_add("));
/// syn::parse_file(&generated.rust)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Non-Result calls return their first boundary value directly and write any
/// additional values through output pointers. Unit returns void. Fallible
/// conversions require explicit [`Self::panic`] and abort on failure; source
/// panics also abort. Destructors return void. Result calls report failures
/// through their explicit error output.
/// Owned pointer inputs transfer ownership after all active inputs validate;
/// later source failures do not restore ownership. References borrow for the
/// call and require the caller to obey Rust aliasing and lifetime rules.
/// Result calls use an optional explicit error output, with owned messages or a
/// declared error record. A success wire with an available niche is returned
/// directly; otherwise the return is bool and success values use output pointers.
/// Unit success needs no output pointer. Consuming Result calls additionally
/// require a bool ownership output, indicating transfer even on failure.
/// Returned pointers, buffers and errors require their matching destructors.
pub struct Cbindgen {
    flat: Option<Bindings>,
    source: Option<std::path::PathBuf>,
    expansions: Vec<prebindgen_declarations::ExpandDecl>,
    functions: Vec<prebindgen_declarations::FunctionDecl>,
    prefix: String,
    pointers: Vec<(String, syn::Type)>,
    enums: Vec<(String, syn::Type)>,
    errors: Vec<(String, syn::Type)>,
    panics: std::collections::BTreeSet<String>,
}
impl Cbindgen {
    /// Create an unconfigured builder with the symbol prefix `pb`.
    pub fn builder() -> Self {
        Self {
            flat: None,
            source: None,
            expansions: Vec::new(),
            functions: Vec::new(),
            prefix: "pb".into(),
            pointers: Vec::new(),
            enums: Vec::new(),
            errors: Vec::new(),
            panics: Default::default(),
        }
    }
    /// Read captures from a source crate's `PREBINDGEN_OUT_DIR`, passed as `path`.
    ///
    /// Only declared functions, referenced constructors/getters and required types
    /// are selected. Automatic type discovery follows unqualified source names
    /// (see [`Flat::parse_selected`]). Types referenced only by qualified paths
    /// can be selected through [`Self::opaque_ptr`] using their unqualified names.
    /// Supply exactly one source directory or flat model.
    pub fn source(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.source = Some(path.as_ref().to_owned());
        self
    }
    /// Register a type-wide input or output expansion, built with `expand_param!`
    /// or `expand_return!`.
    ///
    /// Per-function overrides replace this default; duplicate defaults are rejected
    /// by [`Self::build`]. Registration order does not affect type lookup.
    pub fn expand(mut self, decl: impl Into<prebindgen_declarations::ExpandDecl>) -> Self {
        self.expansions.push(decl.into());
        self
    }
    /// Select caller-owned C-string errors (`char **error`) for Result returns
    /// using `ty`, with a default source error-message mapper.
    /// The helper takes exactly E or &E and returns String. A function's
    /// `on_error(...)` overrides this default. Duplicate resolved types are rejected.
    pub fn error_mapper(
        mut self,
        ty: syn::Type,
        mapper: prebindgen_declarations::FunctionDecl,
    ) -> Self {
        self.expansions
            .push(prebindgen_declarations::ErrorDecl::new(ty, mapper).into());
        self
    }

    /// Deliver this declared record error by value through an optional C
    /// error output pointer. Its fields use ordinary output declarations. The
    /// source type must implement `From<String>` for binding failures.
    pub fn error(mut self, name: &str, ty: syn::Type) -> Self {
        self.errors.push((name.into(), ty));
        self
    }
    /// Deliver an opaque source error as a caller-owned C string through
    /// `char **error`. The mapper takes E or &E and returns String.
    pub fn opaque_error(
        self,
        ty: syn::Type,
        mapper: prebindgen_declarations::FunctionDecl,
    ) -> Self {
        self.error_mapper(ty, mapper)
    }

    /// Export a function and build its occurrence converters from declarations.
    ///
    /// `decl` names the function, normally with `fun!`, and carries its overrides.
    ///
    /// Function renaming is currently unsupported; exports use the symbol prefix
    /// and original source name.
    pub fn function(mut self, decl: prebindgen_declarations::FunctionDecl) -> Self {
        self.functions.push(decl);
        self
    }

    /// Allow binding conversion errors to abort the last declared non-Result
    /// function. Required for fallible conversions without a Result error channel.
    /// Result functions continue to report errors through their explicit output.
    /// Source panics always abort non-Result calls at the C boundary.
    ///
    /// # Panics
    /// Panics if no function has been declared.
    pub fn panic(mut self) -> Self {
        let name = self
            .functions
            .last()
            .expect("panic() must follow function(...)")
            .name
            .clone();
        self.panics.insert(name);
        self
    }

    /// Start from a flat model with input and output conversion slots and prefix `pb`.
    ///
    /// `flat` is the parsed source, with any converters already attached.
    ///
    /// Attach converters with [`prebindgen_declarations::bind_input`] and
    /// [`prebindgen_declarations::bind_output`] with [`CCodec`], or fill defaults with
    /// [`Self::function`].
    /// Every fully bound function in the supplied model is exported.
    pub fn new(flat: Bindings) -> Self {
        Self {
            flat: Some(flat),
            source: None,
            expansions: Vec::new(),
            functions: Vec::new(),
            prefix: "pb".into(),
            pointers: Vec::new(),
            enums: Vec::new(),
            errors: Vec::new(),
            panics: Default::default(),
        }
    }
    /// Set the prefix for exported functions, C types and destructors.
    ///
    /// Defaults to `pb`. The prefix is validated as an identifier during build.
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.into();
        self
    }
    /// Declare an opaque C pointee and its typed destructor for a source type.
    ///
    /// Every leaf type other than a primitive, text or bytes needs a matching
    /// declaration. Owned values use `Box<T>` directly; they have no `Send`
    /// requirement. This declaration alone does not export constructors.
    ///
    /// # Arguments
    ///
    /// * `name` — the C name, without the prefix: the header declares the type
    ///   `{prefix}_{name}` and the destructor `{prefix}_drop_{name}`.
    /// * `ty` — the source type behind the pointer, as written in the source crate.
    pub fn opaque_ptr(mut self, name: &str, ty: syn::Type) -> Self {
        self.pointers.push((name.into(), ty));
        self
    }
    /// Bind a fieldless source enum as a named `int32_t` C type and constants.
    /// Source integer discriminants are preserved and checked on input. Register
    /// the type explicitly; `name` is the unprefixed C typedef/constant prefix.
    pub fn enum_type(mut self, name: &str, ty: syn::Type) -> Self {
        self.enums.push((name.into(), ty));
        self
    }
    /// Resolve declarations, build converters and generate both output texts.
    ///
    /// Does not write files. Returns an error for conflicting sources, invalid or
    /// duplicate declarations, unsupported conversions, or missing opaque pointees.
    /// Rust layout/trait requirements are checked when the generated Rust compiles.
    pub fn build(mut self) -> Result<Generation> {
        prebindgen_declarations::ident(&self.prefix)?;
        let mut flat = match (self.flat, self.source) {
            (Some(flat), None) => flat,
            (None, Some(path)) => Flat::parse_selected(
                prebindgen::Source::new(path).items_all(),
                &prebindgen_declarations::declarations::source_functions(
                    &self.expansions,
                    &self.functions,
                ),
                &self
                    .pointers
                    .iter()
                    .chain(&self.enums)
                    .chain(&self.errors)
                    .map(|(_, ty)| ty.clone())
                    .chain(self.expansions.iter().filter_map(|d| match d {
                        prebindgen_declarations::ExpandDecl::Error(d) => Some(d.ty.clone()),
                        _ => None,
                    }))
                    .collect::<Vec<_>>(),
            )?,
            _ => return Err("supply exactly one source or flat model".into()),
        };
        let mut enum_map = std::collections::BTreeMap::new();
        let mut enum_names = std::collections::BTreeSet::new();
        for (name, ty) in self.enums {
            prebindgen_declarations::ident(&name)?;
            if !enum_names.insert(name.clone()) {
                return Err("duplicate C enum name".into());
            }
            let ty = flat.resolve_type(&ty)?;
            let key = prebindgen_flat::type_key(&ty);
            let source = flat
                .enums
                .get(&key)
                .ok_or("enum_type requires a fieldless enum declaration")?;
            let mapping = prebindgen_declarations::EnumMapping::new(source)?;
            for (variant, _) in &mapping.variants {
                prebindgen_declarations::ident(&format!(
                    "v_{}",
                    variant.to_string().trim_start_matches("r#")
                ))?;
            }
            flat.guards.extend(mapping.guards());
            if enum_map.insert(key, (name, mapping)).is_some() {
                return Err("duplicate C enum source type".into());
            }
        }
        let mut error_map = std::collections::BTreeMap::new();
        let mut error_names = std::collections::BTreeSet::new();
        for (name, ty) in &self.errors {
            prebindgen_declarations::ident(name)?;
            if !error_names.insert(name.clone())
                || error_map
                    .insert(
                        prebindgen_flat::type_key(&flat.resolve_type(ty)?),
                        name.clone(),
                    )
                    .is_some()
            {
                return Err("duplicate C error declaration".into());
            }
        }
        for expansion in &self.expansions {
            if let prebindgen_declarations::ExpandDecl::Error(error) = expansion {
                if error_map
                    .contains_key(&prebindgen_flat::type_key(&flat.resolve_type(&error.ty)?))
                {
                    return Err("structured C errors cannot also select a message mapper".into());
                }
            }
        }
        let mut codec = enums::EnumCodec {
            error_layouts: Default::default(),
            callback_wire: None,
            enums: enum_map,
            errors: error_map,
        };
        let declarations = prebindgen_declarations::Declarations::new(&flat, self.expansions)?;
        let errors = errors::prepare(&mut flat, &codec, &declarations, self.errors)
            .map_err(|e| format!("C error preparation: {e}"))?;
        for error in &errors {
            codec.error_layouts.insert(
                prebindgen_flat::type_key(&error.ty),
                match error.plan.nodes.as_slice() {
                    [prebindgen_flat::WireNode::Outcomes { success, .. }] => success.as_slice(),
                    nodes => nodes,
                }
                .to_vec(),
            );
        }
        let mut callbacks = std::collections::BTreeSet::new();
        let mut names = std::collections::BTreeSet::new();
        for decl in self.functions {
            if !names.insert(decl.name.clone()) {
                return Err("duplicate C function declaration".into());
            }
            if decl.foreign.is_some() {
                return Err("C function renaming is outside this subset".into());
            }
            declarations
                .fill(&mut flat, &codec, &decl, &())
                .map_err(|e| format!("{}: {e}", decl.name))?;
            if decl.callback_return.is_some() {
                let name = format!("{}_each", decl.name);
                let wire = sequence::callback_target(
                    flat.function(&decl.name)?.output_conversion()?.wire_nodes(),
                )?;
                let variant = declarations.callback_variant(&mut flat, &decl, &name)?;
                codec.callback_wire = Some(wire);
                declarations.fill(&mut flat, &codec, &variant, &())?;
                codec.callback_wire = None;
                if !sequence::callback_shape(
                    &flat.function(&name)?.ret.slot.as_ref().unwrap().nodes,
                ) {
                    return Err("callback_return cannot discard additional return values".into());
                }
                if self.panics.contains(&decl.name) {
                    self.panics.insert(name.clone());
                }
                callbacks.insert(name);
                if decl.callback_return == Some(prebindgen_declarations::CallbackReturn::Only) {
                    let original = flat.functions.get_mut(&decl.name).unwrap();
                    original.ret.slot = None;
                    for param in &mut original.params {
                        param.ty.slot = None;
                    }
                }
            }
        }
        let pointers = self
            .pointers
            .into_iter()
            .map(|(name, ty)| {
                prebindgen_declarations::ident(&name)?;
                Ok((name, flat.resolve_type(&ty)?))
            })
            .collect::<Result<Vec<_>>>()?;
        prebindgen_flat::validate_bindings(&flat)?;
        for f in flat.functions.values().filter(|f| f.ret.slot.is_some()) {
            let is_result = prebindgen_declarations::result_parts(&f.ret.ty).is_some();
            if !is_result
                && !self.panics.contains(&f.name.to_string())
                && (f
                    .params
                    .iter()
                    .filter_map(|p| p.ty.slot.as_ref())
                    .any(|p| p.fallible || !p.resources.is_empty())
                    || f.ret.slot.as_ref().is_some_and(|p| p.fallible))
            {
                let name = f.name.to_string();
                let declaration = if callbacks.contains(&name) {
                    name.strip_suffix("_each").unwrap()
                } else {
                    name.as_str()
                };
                return Err(format!("C function `{}` has a fallible binding conversion without Result; declare .panic() after .function(fun!({declaration})) to allow aborting", f.name));
            }
        }
        generate(
            flat,
            &self.prefix,
            pointers,
            codec.enums.into_values().collect(),
            errors,
            callbacks,
        )
    }
}
/// Generated Rust bindings and their corresponding C header.
pub struct Generation {
    /// Formatted Rust source to include in a binding crate with `prebindgen-runtime`
    /// and `konst` (used by captured source feature guards).
    pub rust: String,
    /// C declarations matching the generated Rust ABI, with C++ linkage guards.
    pub header: String,
}
impl Generation {
    /// Write Rust source to exactly `path`, replacing any existing file.
    ///
    /// Relative paths use the current directory, not `OUT_DIR`. Parent directories
    /// must already exist. Returns filesystem errors as messages.
    pub fn write_rust(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        std::fs::write(path, &self.rust).map_err(|e| e.to_string())
    }
    /// Write the C header to exactly `path`, replacing any existing file.
    ///
    /// Uses the same path and error rules as [`Self::write_rust`].
    pub fn write_header(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        std::fs::write(path, &self.header).map_err(|e| e.to_string())
    }
}
fn c_ty(wire: &Wire, prefix: &str, pointers: &[(String, syn::Type)], context: &CContext) -> String {
    match &wire.kind {
        CWire::Enum { name, .. } => format!("{prefix}_{name}"),
        CWire::Scalar { primitive: p, .. } => match p {
            Primitive::Bool | Primitive::U8 => "uint8_t".into(),
            Primitive::Char | Primitive::U32 => "uint32_t".into(),
            Primitive::I8 => "int8_t".into(),
            Primitive::I16 => "int16_t".into(),
            Primitive::I32 => "int32_t".into(),
            Primitive::I64 => "int64_t".into(),
            Primitive::U16 => "uint16_t".into(),
            Primitive::U64 => "uint64_t".into(),
            Primitive::Isize => "intptr_t".into(),
            Primitive::Usize => "uintptr_t".into(),
            Primitive::F32 => "float".into(),
            Primitive::F64 => "double".into(),
            Primitive::I128 | Primitive::U128 => format!("{prefix}_int128"),
        },
        CWire::Pointer { pointer: p, .. } => {
            let (name, _) = pointers
                .iter()
                .find(|(_, ty)| prebindgen_flat::same_type(ty, &p.ty))
                .expect("validated pointer declaration");
            let qualifier = if !p.mutable { "const " } else { "" };
            format!("{qualifier}{prefix}_{name} *")
        }
        CWire::Buffer => format!("{prefix}_buffer"),
        CWire::Rows => format!(
            "{prefix}_{}_rows",
            context.row_types[&wire.name.to_string()]
                .to_string()
                .trim_start_matches("__PbRow_")
        ),
    }
}
fn validate_wire_declaration(
    wire: &Wire,
    pointers: &[(String, syn::Type)],
    enums: &[(String, prebindgen_declarations::EnumMapping)],
) -> Result<()> {
    if let CWire::Enum { name, mapping, .. } = &wire.kind {
        if !enums.iter().any(|(declared, m)| {
            declared == name
                && prebindgen_flat::same_type(&m.ty, &mapping.ty)
                && m.variants
                    .iter()
                    .map(|(n, v)| (n.to_string(), v))
                    .eq(mapping.variants.iter().map(|(n, v)| (n.to_string(), v)))
        }) {
            return Err("declare a matching C enum type for this wire".into());
        }
    }
    if let CWire::Pointer { pointer, .. } = &wire.kind {
        if !pointers
            .iter()
            .any(|(_, ty)| prebindgen_flat::same_type(ty, &pointer.ty))
        {
            return Err(format!(
                "declare a C opaque pointer for `{}`",
                prebindgen_flat::type_key(&pointer.ty)
            ));
        }
    }
    Ok(())
}

fn generate<
    I: for<'a> prebindgen_flat::Conversion<
        Wire = CWire,
        Group = CGroup,
        SlotFragment<'a> = prebindgen_conversions::InputFragment<'a, CContext>,
    >,
    O: for<'a> prebindgen_flat::Conversion<
        Wire = CWire,
        Group = CGroup,
        SlotFragment<'a> = prebindgen_conversions::OutputFragment<'a, CContext>,
    >,
>(
    flat: Flat<I, O>,
    prefix: &str,
    pointers: Vec<(String, syn::Type)>,
    enums: Vec<(String, prebindgen_declarations::EnumMapping)>,
    errors: Vec<errors::ErrorBinding>,
    callbacks: std::collections::BTreeSet<String>,
) -> Result<Generation> {
    let mut calls = Vec::new();
    let free = format_ident!("{prefix}_free_buffer");
    let free_error = format_ident!("{prefix}_free_error");
    let mut symbols = std::collections::BTreeSet::from([
        free.to_string(),
        free_error.to_string(),
        format!("{prefix}_buffer"),
        format!("{prefix}_int128"),
        format!("{prefix}_H"),
    ]);
    for (name, mapping) in &enums {
        if !symbols.insert(format!("{prefix}_{name}")) {
            return Err("duplicate C enum symbol".into());
        }
        for (variant, _) in &mapping.variants {
            let variant = variant.to_string();
            if !symbols.insert(format!(
                "{prefix}_{name}_{}",
                variant.trim_start_matches("r#")
            )) {
                return Err("duplicate C enum constant".into());
            }
        }
    }
    for (name, _) in &pointers {
        if !symbols.insert(format!("{prefix}_{name}")) {
            return Err("C enum and pointer names collide".into());
        }
        if !symbols.insert(format!("{prefix}_drop_{name}")) {
            return Err("duplicate C destructor".into());
        }
    }
    for f in flat.functions.values() {
        if f.ret.slot.is_none() {
            continue;
        }
        for slot in f.params.iter().filter_map(|p| p.ty.slot.as_ref()) {
            prebindgen_flat::validate_wire_nodes(slot.wire_nodes().iter())?;
            validate_layout(slot.wire_nodes())?;
        }
        if let Some(slot) = f.ret.slot.as_ref() {
            prebindgen_flat::validate_wire_nodes(slot.wire_nodes().iter())?;
            validate_layout(slot.wire_nodes())?;
        }
        let result_type = format_ident!("{prefix}_{}_result", f.name);
        if !symbols.insert(result_type.to_string()) {
            return Err("duplicate C result type symbol".into());
        }
        let is_result = prebindgen_declarations::result_parts(&f.ret.ty).is_some();
        let (has_source_error, has_binding_failure) = match f.output_conversion()?.wire_nodes() {
            [prebindgen_flat::WireNode::Outcomes {
                source_error,
                binding_failure,
                ..
            }] => (source_error.is_some(), binding_failure.is_some()),
            _ => (false, false),
        };
        if is_result != has_source_error {
            return Err("C source-error schema must match the declared Result return".into());
        }
        // Recoverable Result wrappers require a diagnostic channel. Other C
        // wrappers retain main's abort policy and may omit it in manual schemas.
        if is_result && !has_binding_failure {
            return Err("recoverable C delivery requires a binding-failure schema".into());
        }
        let context = CContext {
            row_types: {
                let mut schemas = Vec::new();
                for slot in f.input_conversions()? {
                    sequence::schemas(slot.wire_nodes(), &mut schemas);
                }
                sequence::schemas(f.output_conversion()?.wire_nodes(), &mut schemas);
                schemas
                    .into_iter()
                    .map(|(wire, _)| {
                        (
                            wire.name.to_string(),
                            format_ident!("__PbRow_{}_{}", f.name, wire.name),
                        )
                    })
                    .collect()
            },
            acknowledgement: if is_result {
                format_ident!("ownership_accepted")
            } else {
                format_ident!("_ownership_accepted")
            },
            output_pointer: format_ident!("output"),
            output_type: syn::parse_quote!(#result_type),
            error_pointer: format_ident!("error"),
            error_reported: format_ident!("error_reported"),
            output_committed: if prebindgen_declarations::result_parts(&f.ret.ty).is_some() {
                quote!(success_committed = true;)
            } else {
                TokenStream::new()
            },
            buffer_type: syn::parse_quote!(__PbBuffer),
            wide_integer_type: syn::parse_quote!(__Pb128),
            read_buffer: syn::parse_quote!(__pb_read),
        };
        let source = format_ident!("source_value");
        let body = prebindgen_conversions::staged::emit_call(
            f,
            &context,
            &source,
            &syn::parse_quote!(()),
            &crate::group::function_parameters(f)?
                .map(|w| w.name.clone())
                .collect::<Vec<_>>(),
        )?;
        if !symbols.insert(format!("{prefix}_{}", f.name)) {
            return Err("duplicate C export".into());
        }
        let mut all_wires = crate::group::function_parameters(f)?
            .chain(crate::group::result_packet_fields(
                f.output_conversion()?.wire_nodes(),
            ))
            .collect::<Vec<_>>();
        let mut schemas = Vec::new();
        for slot in f.input_conversions()? {
            sequence::schemas(slot.wire_nodes(), &mut schemas);
        }
        sequence::schemas(f.output_conversion()?.wire_nodes(), &mut schemas);
        for (_, element) in schemas {
            all_wires.extend(crate::group::row_fields(element));
        }
        for wire in all_wires {
            validate_wire_declaration(wire, &pointers, &enums)?;
        }
        calls.push((f, body, context));
    }
    let mut tokens = quote! {
       #[repr(C)] #[derive(Clone,Copy)] pub struct __Pb128 {pub hi:u64,pub lo:u64}
       #[repr(C)] #[derive(Clone,Copy)] pub struct __PbRows {pub data:*const ::core::ffi::c_void,pub len:usize}
       #[repr(C)] #[derive(Clone,Copy)] pub struct __PbBuffer {pub data:*const u8,pub len:usize}
       unsafe fn __pb_report_message(message:String,error:*mut *mut std::ffi::c_char){
           if !error.is_null(){std::ptr::write(error,std::ffi::CString::new(message.replace('\0',"?")).unwrap().into_raw());}
       }
       #[doc="# Safety\nThe pointer must be null or an unfreed error string returned by this library."]
       #[no_mangle]pub unsafe extern "C" fn #free_error(error:*mut std::ffi::c_char){if !error.is_null(){drop(std::ffi::CString::from_raw(error));}}
       // Input pointers are borrowed only during the call; output buffers are owned.
       unsafe fn __pb_read<'a>(b:__PbBuffer)->Result<&'a [u8],String>{
        if b.len==0{return Ok(&[]);}
        if b.data.is_null()||b.len>isize::MAX as usize{return Err("invalid buffer".into());}
        Ok(std::slice::from_raw_parts(b.data,b.len))
       }
       #[doc = "# Safety
    The buffer must have been returned by this library and not already freed."]
    #[no_mangle] pub unsafe extern "C" fn #free(b:__PbBuffer){if !b.data.is_null(){drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(b.data as *mut u8,b.len)));}}
      };
    let mut header=format!("#ifndef {prefix}_H\n#define {prefix}_H\n#include <stdint.h>\n#include <stddef.h>\n#include <stdbool.h>\n#ifdef __cplusplus\nextern \"C\" {{\n#endif\ntypedef struct {{ uint64_t hi; uint64_t lo; }} {prefix}_int128;\n typedef struct {{ const uint8_t *data; size_t len; }} {prefix}_buffer;\nvoid {free}({prefix}_buffer b);\nvoid {free_error}(char *error);\n");
    for (name, mapping) in &enums {
        header.push_str(&format!("typedef int32_t {prefix}_{name};\n"));
        for (variant, value) in &mapping.variants {
            let variant = variant.to_string();
            let value = if *value == i32::MIN {
                "INT32_MIN".into()
            } else {
                value.to_string()
            };
            header.push_str(&format!(
                "#define {prefix}_{name}_{} (({prefix}_{name})({value}))\n",
                variant.trim_start_matches("r#")
            ));
        }
    }
    header.push_str("/* Field paths below start at a source parameter or return and list their ABI values.\n * Optional groups below name their discriminator. Rules apply only in active\n * selector arms and present enclosing groups. Absent payload wires are ignored. */\n");
    for (name, ty) in &pointers {
        let drop = format_ident!("{prefix}_drop_{name}");
        tokens.extend(quote!{
            #[doc = "# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call."]
            #[no_mangle] pub unsafe extern "C" fn #drop(ptr:*mut #ty){
                if let Err(error) = ::prebindgen_runtime::boundary(||{if !ptr.is_null(){drop(Box::from_raw(ptr));}Ok(())}) { panic!("C destructor failed: {}", error); }
            }
        });
        header.push_str(&format!(
            "typedef struct {prefix}_{name} {prefix}_{name};\nvoid {drop}({prefix}_{name} *ptr);\n"
        ));
    }
    for error in &errors {
        validate_layout(&error.plan.nodes)?;
        let mut schemas = Vec::new();
        sequence::schemas(&error.plan.nodes, &mut schemas);
        for wire in crate::group::result_packet_fields(&error.plan.nodes).chain(
            schemas
                .iter()
                .flat_map(|(_, element)| crate::group::row_fields(element)),
        ) {
            validate_wire_declaration(wire, &pointers, &enums)?;
        }
    }
    errors::emit(
        &errors,
        prefix,
        &pointers,
        &mut symbols,
        &mut tokens,
        &mut header,
    )?;
    header.push_str("/* Non-Result calls return their first value directly; additional values use output pointers.\n * Binding failures require explicit .panic() and abort. Destructors return void. */\n");
    for (f, body, context) in calls {
        let mut schemas = Vec::new();
        for slot in f.input_conversions()? {
            sequence::schemas(slot.wire_nodes(), &mut schemas);
        }
        let mut output_schemas = Vec::new();
        sequence::schemas(f.output_conversion()?.wire_nodes(), &mut output_schemas);
        let output_carriers = output_schemas
            .iter()
            .map(|(wire, _)| wire.name.to_string())
            .collect::<std::collections::BTreeSet<_>>();
        schemas.extend(output_schemas);
        sequence::emit_schemas(
            schemas,
            &output_carriers,
            &context,
            prefix,
            &pointers,
            &mut symbols,
            &mut tokens,
            &mut header,
        )?;
        let out = &context.output_pointer;
        let accepted = &context.acknowledgement;
        let name = format_ident!("{prefix}_{}", f.name);
        let result = format_ident!("{name}_result");
        let inputs = crate::group::function_parameters(f)?.collect::<Vec<_>>();
        let outputs = crate::group::result_packet_fields(f.output_conversion()?.wire_nodes())
            .collect::<Vec<_>>();
        let mut next_option_label = 0;
        for param in &f.params {
            if let Some(slot) = &param.ty.slot {
                header.push_str(&describe_groups(
                    slot.wire_nodes(),
                    &inputs,
                    false,
                    &mut next_option_label,
                    &param.name.to_string(),
                ));
            }
        }
        if prebindgen_declarations::result_parts(&f.ret.ty).is_none() {
            if let Some(slot) = &f.ret.slot {
                let mut comments = describe_groups(
                    slot.wire_nodes(),
                    &outputs,
                    true,
                    &mut next_option_label,
                    "return",
                );
                for i in (0..outputs.len()).rev() {
                    comments = comments.replace(
                        &format!("output->v{i}"),
                        &if i == 0 {
                            "return value".to_owned()
                        } else {
                            format!("*out{i}")
                        },
                    );
                }
                header.push_str(&comments);
            }
        }
        let mut params = Vec::new();
        let mut cparams = Vec::new();
        for w in crate::group::function_parameters(f)? {
            let n = &w.name;
            let ty = w.kind.rust_type();
            params.push(quote!(#n:#ty));
            cparams.push(format!("{} {n}", c_ty(w, prefix, &pointers, &context)));
        }
        let body = if callbacks.contains(&f.name.to_string()) {
            let (_, element) = sequence::return_sequence(f.output_conversion()?.wire_nodes())
                .ok_or("missing callback element schema")?;
            let types = crate::group::callback_parameters(element)
                .map(|wire| wire.rust_type())
                .collect::<Vec<_>>();
            params.push(quote!(__pb_callback:Option<unsafe extern "C" fn(*mut ::core::ffi::c_void,#(#types),*)>));
            params.push(quote!(__pb_callback_context:*mut ::core::ffi::c_void));
            let mut fields = vec!["void *context".to_string()];
            fields.extend(
                crate::group::callback_parameters(element)
                    .enumerate()
                    .map(|(i, wire)| format!("{} v{i}", c_ty(wire, prefix, &pointers, &context))),
            );
            cparams.push(format!("void (*callback)({})", fields.join(", ")));
            cparams.push("void *context".into());
            header.push_str("/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */\n");
            quote!(if __pb_callback.is_none(){return Err("null sequence callback".into());}#body)
        } else {
            body
        };
        if let Some((_, error_ty)) = prebindgen_declarations::result_parts(&f.ret.ty) {
            let fields = outputs
                .iter()
                .enumerate()
                .map(|(i, wire)| {
                    let field = format_ident!("v{i}");
                    let ty = wire.rust_type();
                    quote!(pub #field:#ty)
                })
                .collect::<Vec<_>>();
            let fields = if fields.is_empty() {
                vec![quote!(pub unused:u8)]
            } else {
                fields
            };
            tokens.extend(quote!(#[repr(C)]pub struct #result {#(#fields),*}));
            let comments = describe_groups(
                f.ret.slot.as_ref().unwrap().wire_nodes(),
                &outputs,
                true,
                &mut next_option_label,
                "return",
            );
            let consumes=f.params.iter().filter_map(|p|p.ty.slot.as_ref()).any(|slot| !matches!(slot.code_fragment_slot(prebindgen_conversions::InputFragment::Acknowledge {context:&context},&syn::parse_quote!(())),Ok(code) if code.to_string()==quote!(()).to_string()));
            errors::result_function(
                &f.name,
                error_ty,
                &outputs,
                f.output_conversion()?.wire_nodes(),
                params,
                cparams,
                &body,
                &context,
                prefix,
                &pointers,
                &errors,
                consumes,
                &comments,
                &mut tokens,
                &mut header,
            )?;
            continue;
        }
        let mut fields = Vec::new();
        let mut writes = TokenStream::new();
        let mut validate_outputs = TokenStream::new();
        for (i, wire) in outputs.iter().enumerate() {
            let field = format_ident!("v{i}");
            let ty = wire.rust_type();
            fields.push(quote!(pub #field:#ty));
            if i > 0 {
                let pointer = format_ident!("out{i}");
                params.push(quote!(#pointer:*mut #ty));
                cparams.push(format!(
                    "{} *{pointer}",
                    c_ty(wire, prefix, &pointers, &context)
                ));
                validate_outputs.extend(
                    quote!(if #pointer.is_null(){return Err("null success output".into());}),
                );
                writes.extend(quote!(std::ptr::write(#pointer, value.#field);));
            }
        }
        if fields.is_empty() {
            fields.push(quote!(pub unused:u8));
        }
        let (return_type, c_return, returned) = match outputs.first() {
            Some(wire) => {
                let ty = wire.rust_type();
                (
                    quote!(->#ty),
                    c_ty(wire, prefix, &pointers, &context),
                    quote!(value.v0),
                )
            }
            None => (quote!(), "void".into(), quote!()),
        };
        let value = if outputs.is_empty() {
            quote!()
        } else {
            quote!(let value=storage.assume_init();)
        };
        header.push_str(&format!(
            "{c_return} {name}({});\n",
            if cparams.is_empty() {
                "void".into()
            } else {
                cparams.join(", ")
            }
        ));
        tokens.extend(quote! {
         #[repr(C)] pub struct #result {#(#fields),*}
         #[doc = "# Safety\nAdditional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call."]
         #[allow(clippy::needless_borrow,clippy::let_unit_value,clippy::unnecessary_cast,clippy::useless_conversion)]
         #[no_mangle] pub unsafe extern "C" fn #name(#(#params),*) #return_type {
            #[allow(unused_mut)] let mut #accepted=false;
            let mut storage=std::mem::MaybeUninit::<#result>::uninit();
            let #out=storage.as_mut_ptr();
            if let Err(error)=::prebindgen_runtime::boundary(||{#validate_outputs #body}) {panic!("C binding failed: {}",error);}
            #value
            #writes
            #returned
         }
        });
    }
    let guards = &flat.guards;
    tokens.extend(quote!(#(#guards)*));
    header.push_str("#ifdef __cplusplus\n}\n#endif\n#endif\n");
    Ok(Generation {
        rust: prebindgen_flat::rust_source(tokens)?,
        header,
    })
}

fn describe_groups(
    layout: &[prebindgen_flat::WireNode<CWire, CGroup>],
    wires: &[&Wire],
    output: bool,
    next_label: &mut usize,
    root: &str,
) -> String {
    fn visit(
        layout: &[prebindgen_flat::WireNode<CWire, CGroup>],
        names: &std::collections::BTreeMap<String, String>,
        parents: &mut Vec<String>,
        out: &mut String,
        next_label: &mut usize,
        field_path: &mut Vec<String>,
    ) {
        for node in layout {
            if let prebindgen_flat::WireNode::Alternatives { alternatives, .. } = node {
                for arm in alternatives {
                    visit(arm, names, parents, out, next_label, field_path);
                }
            }
            if let prebindgen_flat::WireNode::Sequence(c) = node {
                let carrier = c
                    .metadata
                    .as_ref()
                    .and_then(CGroup::sequence_wire)
                    .expect("validated sequence carrier");
                let element = &c.element;
                let path = field_path.join(".");
                if matches!(carrier.kind, CWire::Rows) {
                    out.push_str(&format!("/* Collection {path}: {} carries rows of {} ABI fields; element-owned fields are released separately from row storage. */\n",carrier.name,crate::group::row_fields(element).len()));
                } else {
                    out.push_str(&format!("/* Callback sequence {path}: {} reports the delivered element count; the visitor receives {} fields per element. */\n",carrier.name,crate::group::row_fields(element).len()));
                }
                let row_names = crate::group::row_fields(element)
                    .enumerate()
                    .map(|(i, w)| (w.name.to_string(), format!("row.v{i}")))
                    .collect();
                let original = field_path.last().cloned();
                if let Some(last) = field_path.last_mut() {
                    last.push_str("[]");
                }
                visit(element, &row_names, parents, out, next_label, field_path);
                if let (Some(last), Some(original)) = (field_path.last_mut(), original) {
                    *last = original;
                }
            }
            if let prebindgen_flat::WireNode::Optional(c) = node {
                if let Some(CGroup::Optional(g)) = &c.metadata {
                    let label = format!("option{}", *next_label);
                    *next_label += 1;
                    let discriminator = &names[&g.discriminator.to_string()];
                    let rule=match &g.niche {
                        None=>format!("{discriminator}: 0 = None, 1 = Some; other active flags are errors"),
                        Some(CNiche::Null)=>format!("{discriminator} == NULL means None; non-null means Some"),
                        Some(CNiche::Integer(n))=>format!("{discriminator} == {n} means None; other values encode Some and must be valid payloads"),
                    };
                    let parent = if parents.is_empty() {
                        String::new()
                    } else {
                        format!("; only when {} is present", parents.join(" and "))
                    };
                    out.push_str(&format!("/* Option {}: {rule}{parent}. */\n", label));
                    parents.push(label);
                    visit(&c.element, names, parents, out, next_label, field_path);
                    parents.pop();
                }
            }
            if let prebindgen_flat::WireNode::Outcomes { success, .. } = node {
                visit(success, names, parents, out, next_label, field_path);
            }
            if let prebindgen_flat::WireNode::Group { metadata, children } = node {
                if let CGroup::Field(name) = metadata {
                    field_path.push(name.to_string().trim_start_matches("r#").into());
                    let values = crate::group::record_fields(children)
                        .map(|w| names[&w.name.to_string()].clone())
                        .collect::<Vec<_>>();
                    let values = if values.is_empty() {
                        "no boundary values".into()
                    } else {
                        values.join(", ")
                    };
                    out.push_str(&format!(
                        "/* Field {}: {values}. */\n",
                        field_path.join(".")
                    ));
                    visit(children, names, parents, out, next_label, field_path);
                    field_path.pop();
                } else {
                    visit(children, names, parents, out, next_label, field_path);
                }
            }
        }
    }
    let names = wires
        .iter()
        .enumerate()
        .map(|(i, w)| {
            (
                w.name.to_string(),
                if output {
                    format!("output->v{i}")
                } else {
                    w.name.to_string()
                },
            )
        })
        .collect();
    let mut out = String::new();
    visit(
        layout,
        &names,
        &mut Vec::new(),
        &mut out,
        next_label,
        &mut vec![root.trim_start_matches("r#").into()],
    );
    out
}

#[cfg(test)]
mod generation_tests {
    use super::*;

    fn error_model(source: &str) -> Bindings {
        let source = syn::parse_file(source).unwrap();
        Bindings::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap()
    }
    #[test]
    fn logical_outcomes_and_sequence_fields_are_independent_of_c_transport() {
        use prebindgen_declarations::{fun, Declarations};
        use prebindgen_flat::Conversion;
        let mut flat=error_model("pub struct Failure {pub code:i64,pub info:Option<String>} pub fn f(v:Option<Vec<i64>>)->Result<Option<Vec<Option<i64>>>,Failure>{}");
        let ty = flat.resolve_type(&syn::parse_quote!(Failure)).unwrap();
        let mut codec = enums::EnumCodec {
            callback_wire: None,
            enums: Default::default(),
            errors: std::collections::BTreeMap::from([(
                prebindgen_flat::type_key(&ty),
                "failure".into(),
            )]),
            error_layouts: Default::default(),
        };
        let declarations = Declarations::new(&flat, vec![]).unwrap();
        let bindings = errors::prepare(
            &mut flat,
            &codec,
            &declarations,
            vec![("failure".into(), ty.clone())],
        )
        .unwrap();
        codec.error_layouts.insert(
            prebindgen_flat::type_key(&ty),
            match bindings[0].plan.nodes.as_slice() {
                [prebindgen_flat::WireNode::Outcomes { success, .. }] => success.as_slice(),
                nodes => nodes,
            }
            .to_vec(),
        );
        let decl = fun!(f).callback_return();
        declarations.fill(&mut flat, &codec, &decl, &()).unwrap();
        let normal = flat.function("f").unwrap();
        assert_eq!(normal.input_wire_definitions().unwrap().count(), 1);
        assert_eq!(
            crate::group::function_parameters(normal).unwrap().count(),
            2
        );
        let output = normal.output_conversion().unwrap();
        let [prebindgen_flat::WireNode::Outcomes {
            success,
            source_error: Some(errors),
            binding_failure: Some(binding_failure),
        }] = output.wire_nodes()
        else {
            panic!("all three completion channels must be declared")
        };

        assert_eq!(prebindgen_flat::wire_definitions(success.iter()).count(), 1);

        assert_eq!(prebindgen_flat::wire_definitions(errors.iter()).count(), 2);
        assert_eq!(
            crate::group::result_packet_fields(&bindings[0].plan.nodes).count(),
            3
        );
        assert_eq!(
            prebindgen_flat::wire_definitions(binding_failure.iter()).count(),
            1
        );
        assert_ne!(
            prebindgen_flat::wire_definitions(output.wire_nodes().iter()).count(),
            output.values.len()
        );
        assert!(matches!(
            prebindgen_flat::wire_definitions(success.iter())
                .next()
                .unwrap()
                .kind,
            CWire::Scalar {
                primitive: Primitive::I64,
                ..
            }
        ));
        let shape = |layout: &[prebindgen_flat::WireNode<CWire, CGroup>]| {
            prebindgen_flat::wire_definitions_with_paths(layout.iter())
                .map(|e| (e.path, prebindgen_flat::type_key(&e.wire.rust_type())))
                .collect::<Vec<_>>()
        };
        let expected = shape(output.wire_nodes());
        let wire = sequence::callback_target(
            flat.function("f")
                .unwrap()
                .output_conversion()
                .unwrap()
                .wire_nodes(),
        )
        .unwrap();
        let variant = declarations
            .callback_variant(&mut flat, &decl, "f_each")
            .unwrap();
        codec.callback_wire = Some(wire);
        declarations.fill(&mut flat, &codec, &variant, &()).unwrap();
        let callback = flat
            .function("f_each")
            .unwrap()
            .output_conversion()
            .unwrap();
        assert_eq!(shape(callback.wire_nodes()), expected);
        assert!(matches!(
            crate::group::result_packet_fields(callback.wire_nodes())
                .nth(1)
                .unwrap()
                .kind,
            CWire::Scalar {
                primitive: Primitive::Usize,
                ..
            }
        ));
    }
    #[test]
    fn callback_return_rejects_unrelated_output_values() {
        use prebindgen_declarations::{expand_return, fun};
        let source=syn::parse_file("pub fn values()->Option<Vec<i64>>{} pub fn number(v:&Option<Vec<i64>>)->i64{} pub fn view(v:&Option<Vec<i64>>)->Vec<i64>{}").unwrap();
        let flat = Bindings::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap();
        let result = Cbindgen::new(flat)
            .function(
                fun!(values).callback_return().expand_return(
                    expand_return!(Option<Vec<i64>>)
                        .field(fun!(number))
                        .field(fun!(view)),
                ),
            )
            .panic()
            .build();
        assert!(result.err().unwrap().contains("additional return"));
    }

    #[test]
    fn callback_returns_preserve_buffers_and_require_explicit_abort_policy() {
        let flat = || {
            let source = syn::parse_file("pub fn values()->Vec<i64>{}").unwrap();
            Bindings::parse(
                source
                    .items
                    .into_iter()
                    .map(|item| (item, Default::default())),
            )
            .unwrap()
        };
        let builder = || {
            Cbindgen::new(flat()).function(prebindgen_declarations::fun!(values).callback_return())
        };
        assert!(builder().build().err().unwrap().contains(".panic()"));
        let output = builder().panic().build().unwrap();
        assert!(output.header.contains("test_values") || output.header.contains("pb_values("));
        assert!(output.header.contains("pb_values_each("));
        assert!(output
            .header
            .contains("void (*callback)(void *context, int64_t v0)"));
        syn::parse_file(&output.rust).unwrap();
    }

    #[test]
    fn structured_error_registration_is_explicit_and_conflicts_are_rejected() {
        use prebindgen_declarations::fun;
        let source="pub struct Error {pub code:i64} pub fn message(error:&Error)->String{} pub fn f()->Result<i64,Error>{}";
        assert!(Cbindgen::new(error_model(source))
            .function(fun!(f))
            .build()
            .err()
            .unwrap()
            .contains("explicit error_mapper"));
        assert!(Cbindgen::new(error_model(source))
            .error("error", syn::parse_quote!(Error))
            .error("other", syn::parse_quote!(Error))
            .function(fun!(f))
            .build()
            .is_err());
        assert!(Cbindgen::new(error_model(source))
            .error("error", syn::parse_quote!(Error))
            .error_mapper(syn::parse_quote!(Error), fun!(message))
            .function(fun!(f))
            .build()
            .err()
            .unwrap()
            .contains("cannot also select"));
        let generated = Cbindgen::new(error_model(source))
            .error("error", syn::parse_quote!(Error))
            .function(fun!(f))
            .build()
            .unwrap();
        assert!(generated
            .header
            .contains("bool pb_f(int64_t *out0, pb_error *error)"));
        assert!(!generated.header.contains("pb_f_result *"));
        assert!(generated.header.contains("int64_t code;"));
    }
    #[test]
    fn error_field_names_follow_wire_subtrees_and_avoid_c_keywords() {
        use prebindgen_declarations::fun;
        let generated = Cbindgen::new(error_model(
            "pub struct Error {pub empty:(),pub value:Option<i64>} pub fn f()->Result<(),Error>{}",
        ))
        .error("error", syn::parse_quote!(Error))
        .function(fun!(f))
        .build()
        .unwrap();
        assert!(generated
            .header
            .contains("Field error.empty: no boundary values"));
        assert!(generated
            .header
            .contains("Field error.value: error->v0, error->v1"));
        let generated = Cbindgen::new(error_model(
            "pub struct Error {pub char:String,pub v0:i64} pub fn f()->Result<(),Error>{}",
        ))
        .error("error", syn::parse_quote!(Error))
        .function(fun!(f))
        .build()
        .unwrap();
        assert!(generated.header.contains("pb_buffer v0;"));
        assert!(generated.header.contains("int64_t v1;"));
        assert!(!generated.header.contains("pb_buffer char;"));
    }

    #[test]
    fn unit_and_optional_handle_results_use_the_expected_explicit_outputs() {
        use prebindgen_declarations::fun;
        let source="pub type Handle=String; pub struct Error {pub code:i64} pub fn unit()->Result<(),Error>{} pub fn make()->Result<Handle,Error>{} pub fn maybe()->Result<Option<Handle>,Error>{}";
        let generated = Cbindgen::new(error_model(source))
            .opaque_ptr("handle", syn::parse_quote!(Handle))
            .error("error", syn::parse_quote!(Error))
            .function(fun!(unit))
            .function(fun!(make))
            .function(fun!(maybe))
            .build()
            .unwrap();
        assert!(generated.header.contains("bool pb_unit(pb_error *error)"));
        assert!(generated
            .header
            .contains("pb_handle * pb_make(pb_error *error)"));
        assert!(generated
            .header
            .contains("bool pb_maybe(pb_handle * *out0, pb_error *error)"));
        assert!(generated.header.contains("NULL means failure"));
    }

    #[test]
    fn multiple_collection_observations_follow_the_actual_borrow() {
        use prebindgen_declarations::{bind_output, Output};
        let seq = || Output::Sequence {
            element: Box::new(Output::Leaf),
            mode: None,
        };
        for (source, recipe) in [
            ("pub fn f()->&Vec<i64> {}", Output::combine([seq(), seq()])),
            (
                "pub struct Record { pub values: Vec<i64> } pub fn f()->Record {}",
                Output::fields([Output::combine([seq(), seq()])]),
            ),
        ] {
            let source = syn::parse_file(source).unwrap();
            let mut flat =
                Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
            bind_output(&mut flat, &CCodec, "f", &recipe, &()).unwrap();
            assert_eq!(
                crate::group::result_packet_fields(
                    &flat.function("f").unwrap().ret.slot.as_ref().unwrap().nodes
                )
                .len(),
                2
            );
            Cbindgen::new(flat).build().unwrap();
        }
    }

    #[test]
    fn option_comment_labels_are_unique_across_parameters_and_result() {
        use prebindgen_declarations::{bind_input, bind_output, Input, Output};
        let source = syn::parse_file("pub fn echo(a: Option<Option<char>>, b: Option<Option<char>>) -> Option<Option<char>> { a }").unwrap();
        let mut flat =
            Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
        let input = Input::Optional(Box::new(Input::Optional(Box::new(Input::Leaf))));
        for param in ["a", "b"] {
            bind_input(&mut flat, &CCodec, "echo", param, &input, &()).unwrap();
        }
        let output = Output::Optional(Box::new(Output::Optional(Box::new(Output::Leaf))));
        bind_output(&mut flat, &CCodec, "echo", &output, &()).unwrap();
        let header = Cbindgen::new(flat)
            .function(prebindgen_declarations::fun!(echo))
            .panic()
            .build()
            .unwrap()
            .header;
        for i in 0..6 {
            assert_eq!(header.matches(&format!("Option option{i}:")).count(), 1);
        }
        for parent in [0, 2, 4] {
            assert!(header.contains(&format!("only when option{parent} is present")));
        }
    }

    #[test]
    fn header_describes_shared_nested_empty_and_multiwire_fields() {
        use prebindgen_declarations::{bind_output, Output};
        let source = syn::parse_file("pub struct Empty {} pub struct Pair { pub x: i32, pub y: i32 } pub struct Record { pub empty: Empty, pub pair: Pair, pub maybe: Option<i32> } pub fn make() -> Record { unreachable!() }").unwrap();
        let mut flat =
            Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
        let recipe = Output::Fields(vec![
            Output::Fields(vec![]),
            Output::Fields(vec![Output::Leaf, Output::Leaf]),
            Output::Optional(Box::new(Output::Leaf)),
        ]);
        bind_output(&mut flat, &CCodec, "make", &recipe, &()).unwrap();
        let header = Cbindgen::new(flat).build().unwrap().header;
        assert!(header.contains("Field return.empty: no boundary values."));
        assert!(header.contains("Field return.pair: return value, *out1."));
        assert!(header.contains("Field return.pair.x: return value."));
        assert!(header.contains("Field return.pair.y: *out1."));
        assert!(header.contains("Field return.maybe: *out2, *out3."));
    }

    #[test]
    fn character_niches_exhaust_at_the_carrier_limit() {
        let wire: CWire = CWire::scalar(Primitive::Char);
        assert!(CCodec
            .remaining_niche(&wire, &CNiche::Integer(u32::MAX as i64))
            .is_none());
        assert!(matches!(
            CCodec.remaining_niche(&wire, &CNiche::Integer(0x110000)),
            Some(CNiche::Integer(0x110001))
        ));
    }

    #[test]
    fn shared_compiler_suppresses_niches_in_selector_arms() {
        use prebindgen_declarations::{bind_input, Input};
        for (payload, arm) in [
            ("char", Input::Leaf),
            ("Option<char>", Input::Optional(Box::new(Input::Leaf))),
        ] {
            let source = syn::parse_file(&format!("pub fn echo(value: {payload}) {{}}")).unwrap();
            let mut flat =
                Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
            let selector = Input::Selector(vec![arm.clone(), arm.clone()]);
            bind_input(&mut flat, &CCodec, "echo", "value", &selector, &()).unwrap();
            assert_eq!(
                crate::group::function_parameters(flat.function("echo").unwrap())
                    .unwrap()
                    .len(),
                3
            );
            // Check suppression before an outer option can clear the arm niches.
            for wire in crate::group::function_parameters(flat.function("echo").unwrap())
                .unwrap()
                .skip(1)
            {
                assert!(matches!(
                    wire.kind,
                    CWire::Scalar {
                        primitive: Primitive::Char,
                        ..
                    }
                ));
                assert!(CCodec.niche(&wire.kind).is_none());
            }
            let source =
                syn::parse_file(&format!("pub fn echo(value: Option<{payload}>) {{}}")).unwrap();
            let mut flat =
                Bindings::parse(source.items.into_iter().map(|i| (i, Default::default()))).unwrap();
            let recipe = Input::Optional(Box::new(Input::Selector(vec![arm.clone(), arm])));
            bind_input(&mut flat, &CCodec, "echo", "value", &recipe, &()).unwrap();
            let plan = flat.function("echo").unwrap().params[0]
                .ty
                .slot
                .as_ref()
                .unwrap();
            // Presence flag + selector + two char carriers. Even a remaining
            // niche of an optional arm cannot describe the outer option.
            assert_eq!(
                crate::group::function_parameters(flat.function("echo").unwrap())
                    .unwrap()
                    .len(),
                4
            );
            let prebindgen_flat::WireNode::Optional(container) = &plan.nodes.as_slice()[0] else {
                panic!()
            };
            let Some(CGroup::Optional(optional)) = &container.metadata else {
                panic!()
            };
            assert!(optional.niche.is_none());
            assert_eq!(
                optional.discriminator,
                crate::group::function_parameters(flat.function("echo").unwrap())
                    .unwrap()
                    .next()
                    .unwrap()
                    .name
            );
        }
    }

    #[test]
    fn rejects_invalid_pointer_names_during_preparation() {
        let source = syn::parse_file("pub type Handle = String;").unwrap();
        let flat = Bindings::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap();
        let result = Cbindgen::new(flat)
            .opaque_ptr("invalid-name", syn::parse_quote!(Handle))
            .build();
        assert_eq!(result.err().unwrap(), "invalid identifier `invalid-name`");
    }

    #[test]
    fn recoverable_c_results_require_a_declared_binding_failure_channel() {
        use prebindgen_declarations::{fun, Declarations};
        use prebindgen_flat::WireNode;
        let mut flat = error_model("pub fn message(e:i64)->String{} pub fn f()->Result<i64,i64>{}");
        let declarations = Declarations::new(&flat, vec![]).unwrap();
        declarations
            .fill(&mut flat, &CCodec, &fun!(f).on_error(fun!(message)), &())
            .unwrap();
        let output = flat.function_mut("f").unwrap().ret.slot.as_mut().unwrap();
        let [WireNode::Outcomes {
            binding_failure, ..
        }] = output.nodes.as_mut_slice()
        else {
            panic!()
        };
        *binding_failure = None;
        let error = generate(flat, "test", vec![], vec![], vec![], Default::default())
            .err()
            .unwrap();
        assert!(error.contains("binding-failure schema"));
    }

    #[test]
    fn emits_custom_slot_code_without_building_a_plan() {
        struct CountedOutput(
            std::rc::Rc<std::cell::Cell<usize>>,
            Vec<prebindgen_flat::WireNode<CWire, CGroup>>,
        );
        impl prebindgen_flat::Conversion for CountedOutput {
            type Wire = CWire;
            type Group = CGroup;
            type SlotFragment<'a> = prebindgen_conversions::OutputFragment<'a, CContext>;

            fn wire_nodes(&self) -> &[prebindgen_flat::WireNode<CWire, CGroup>] {
                &self.1
            }
            fn code_fragment_slot(
                &self,
                mode: Self::SlotFragment<'_>,
                _: &syn::Type,
            ) -> Result<TokenStream> {
                use prebindgen_conversions::OutputFragment;
                Ok(match mode {
                    OutputFragment::Prepare { next, .. } => {
                        self.0.set(self.0.get() + 1);
                        quote!({ #next })
                    }
                    OutputFragment::Deliver { next, .. } => quote!({ #next }),
                    OutputFragment::Commit { .. } | OutputFragment::Result { .. } => quote!(()),
                })
            }
        }
        let count = std::rc::Rc::new(std::cell::Cell::new(0));
        let source = syn::parse_file("pub fn noop() {}").unwrap();
        let mut flat = Flat::<InputPlan<CWire>, CountedOutput>::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap();
        flat.function_mut("noop").unwrap().ret.slot =
            Some(CountedOutput(count.clone(), Default::default()));
        let output = generate(flat, "test", vec![], vec![], vec![], Default::default()).unwrap();
        assert!(output.header.contains("test_noop("));
        assert_eq!(count.get(), 1);
    }
}

/// A wire value excluded from the representation of a present value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CNiche {
    /// Null address (pointer or integer-encoded handle).
    Null,
    /// An invalid integer representation, within the wire's integer range.
    Integer(i64),
}
impl CNiche {
    /// Expression recognizing the present case without decoding the payload.
    pub(crate) fn present(&self, value: TokenStream) -> TokenStream {
        match self {
            Self::Null => quote!((#value as usize) != 0),
            Self::Integer(n) => {
                let n = syn::parse_str::<syn::Expr>(&n.to_string()).unwrap();
                quote!(#value != #n)
            }
        }
    }
    /// Absence expression for a numeric wire. Pointer codecs supply a null pointer.
    pub(crate) fn integer_value(&self) -> TokenStream {
        match self {
            Self::Null => quote!(0),
            Self::Integer(n) => syn::parse_str::<TokenStream>(&n.to_string()).unwrap(),
        }
    }
}

#[cfg(test)]
mod abort_policy_tests {
    use super::*;
    use prebindgen_declarations::{expand_param, fun};
    fn model(source: &str) -> Bindings {
        Bindings::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap()
    }
    #[test]
    fn direct_values_and_void_do_not_need_an_error_policy() {
        let generated = Cbindgen::new(model("pub fn identity(x:i64)->i64{} pub fn unit(){}"))
            .function(fun!(identity))
            .function(fun!(unit))
            .build()
            .unwrap();
        assert!(generated.header.contains("int64_t pb_identity(int64_t"));
        assert!(generated.header.contains("void pb_unit(void)"));
        assert!(!generated.header.contains("_result"));
        assert!(!generated.header.contains("last_error"));
        assert!(!generated.rust.contains("thread_local"));
        syn::parse_file(&generated.rust).unwrap();
    }
    #[test]
    fn fallible_leaves_and_option_flags_require_explicit_abort_policy() {
        for ty in ["char", "&str", "&[u8]", "Option<i64>"] {
            let source = format!("pub fn f(value:{ty}){{}}");
            let error = Cbindgen::new(model(&source))
                .function(fun!(f))
                .build()
                .err()
                .unwrap();
            assert!(error.contains(".panic()"), "{error}");
            Cbindgen::new(model(&source))
                .function(fun!(f))
                .panic()
                .build()
                .unwrap();
        }
    }
    #[test]
    fn constructor_alternatives_retain_their_own_validation_policy() {
        let source="pub struct Number {pub value:i64} pub fn new(value:i64)->Number{} pub fn other(value:i64)->Number{} pub fn consume(value:Number){}";
        let bind = || {
            Cbindgen::new(model(source)).function(
                fun!(consume).expand_param(
                    "value",
                    expand_param!(Number)
                        .variant(fun!(new))
                        .variant(fun!(other)),
                ),
            )
        };
        assert!(bind().build().err().unwrap().contains(".panic()"));
        bind().panic().build().unwrap();
    }
    #[test]
    fn manually_bound_fallible_output_requires_the_same_policy() {
        let mut flat = model("pub fn f()->i64{}");
        prebindgen_declarations::bind_output(
            &mut flat,
            &CCodec,
            "f",
            &prebindgen_declarations::Output::Leaf,
            &(),
        )
        .unwrap();
        flat.function_mut("f")
            .unwrap()
            .ret
            .slot
            .as_mut()
            .unwrap()
            .fallible = true;
        assert!(Cbindgen::new(flat.clone())
            .build()
            .err()
            .unwrap()
            .contains(".panic()"));
        Cbindgen::new(flat)
            .function(fun!(f))
            .panic()
            .build()
            .unwrap();
    }
}
