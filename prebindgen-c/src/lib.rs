//! Generate Rust wrappers and C declarations from conversion slots.
#![warn(missing_docs)]

use prebindgen_conversions::staged::StagedFunction;
use prebindgen_conversions::value::Value;
use prebindgen_flat::{Flat, Primitive, Result, WireKind};

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
    fn optional_group(
        &self,
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<CNiche>,
    ) -> CGroup {
        CGroup::Optional(COptional {
            presence,
            discriminator,
            niche,
        })
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
        validate_layout(&plan.layout)?;
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
        validate_layout(&plan.layout)?;
        let mut values: Vec<_> = plan
            .values
            .iter()
            .enumerate()
            .map(|(i, value)| {
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
        element: &prebindgen_flat::WireLayout<CWire, CGroup>,
        _: &prebindgen_declarations::Condition,
    ) -> Result<prebindgen_declarations::SequenceInput<CWire>> {
        sequence::input(wire, length, index, element)
    }
    fn sequence_output(
        &self,
        _: &(),
        wire: syn::Ident,
        length: syn::Ident,
        _: syn::Ident,
        element: &prebindgen_flat::WireLayout<CWire, CGroup>,
        values: &[TokenStream],
    ) -> Result<prebindgen_declarations::SequenceOutput<CWire>> {
        sequence::output(wire, length, element, values)
    }
    fn result_error(&self, _: &syn::Type, message: TokenStream, _: &()) -> Result<TokenStream> {
        Ok(message)
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
        prebindgen_conversions::value::input(ty, w, v, &active.rust(), decode, kind)
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
            Some(Value::Scalar(p)) => prebindgen_conversions::value::output(
                ty,
                w,
                v,
                borrowed,
                |v| encode_scalar(p, v),
                CWire::scalar(p),
            ),
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
                let buffer_type = prebindgen_conversions::template_parameter("buffer_type");
                out.values = vec![quote!(#buffer_type {data: #w.as_ptr(), len: #w.len()})];
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
/// Generated calls return a status and write results through an output pointer;
/// `0` means success. On failure, `-1` means ownership has not been accepted;
/// `-2` means active owned inputs have been consumed. Both set a thread-local
/// last-error message and leave success-output storage unchanged.
/// Owned pointer inputs transfer ownership after all active inputs validate;
/// later source failures do not restore ownership. References borrow for the
/// call and require the caller to obey Rust aliasing and lifetime rules.
/// Returned pointers and buffers must be freed with their generated destructors.
pub struct Cbindgen {
    flat: Option<Bindings>,
    source: Option<std::path::PathBuf>,
    expansions: Vec<prebindgen_declarations::ExpandDecl>,
    functions: Vec<prebindgen_declarations::FunctionDecl>,
    prefix: String,
    pointers: Vec<(String, syn::Type)>,
    enums: Vec<(String, syn::Type)>,
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
    /// Select a default source error-message mapper for Result returns using `ty`.
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
    pub fn build(self) -> Result<Generation> {
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
        let codec = enums::EnumCodec { enums: enum_map };
        let declarations = prebindgen_declarations::Declarations::new(&flat, self.expansions)?;
        let mut names = std::collections::BTreeSet::new();
        for decl in self.functions {
            if !names.insert(decl.name.clone()) {
                return Err("duplicate C function declaration".into());
            }
            if decl.foreign.is_some() {
                return Err("C function renaming is outside this subset".into());
            }
            declarations.fill(&mut flat, &codec, &decl, &())?;
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
        generate(
            flat,
            &self.prefix,
            pointers,
            codec.enums.into_values().collect(),
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
        WireFragment<'a> = prebindgen_conversions::OutputWireFragment<'a, CContext>,
    >,
>(
    flat: Flat<I, O>,
    prefix: &str,
    pointers: Vec<(String, syn::Type)>,
    enums: Vec<(String, prebindgen_declarations::EnumMapping)>,
) -> Result<Generation> {
    let mut calls = Vec::new();
    let error = format_ident!("{prefix}_last_error");
    let free = format_ident!("{prefix}_free_buffer");
    let mut symbols = std::collections::BTreeSet::from([
        error.to_string(),
        free.to_string(),
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
            validate_layout(slot.layout())?;
        }
        if let Some(slot) = f.ret.slot.as_ref() {
            validate_layout(slot.layout())?;
        }
        let result_type = format_ident!("{prefix}_{}_result", f.name);
        if !symbols.insert(result_type.to_string()) {
            return Err("duplicate C result type symbol".into());
        }
        let context = CContext {
            acknowledgement: format_ident!("ownership_accepted"),
            row_types: {
                let mut schemas = Vec::new();
                for slot in f.input_conversions()? {
                    sequence::schemas(slot.layout(), &mut schemas);
                }
                sequence::schemas(f.output_conversion()?.layout(), &mut schemas);
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
            output_pointer: format_ident!("output"),
            output_type: syn::parse_quote!(#result_type),
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
        )?;
        if !symbols.insert(format!("{prefix}_{}", f.name)) {
            return Err("duplicate C export".into());
        }
        let mut all_wires = f
            .inputs()?
            .chain(f.outputs(&context, &source)?.map(|o| o.wire))
            .collect::<Vec<_>>();
        let mut schemas = Vec::new();
        for slot in f.input_conversions()? {
            sequence::schemas(slot.layout(), &mut schemas);
        }
        sequence::schemas(f.output_conversion()?.layout(), &mut schemas);
        for (_, element) in schemas {
            all_wires.extend(element.iter());
        }
        for wire in all_wires {
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
                    .any(|(_, declared)| prebindgen_flat::same_type(declared, &pointer.ty))
                {
                    return Err(format!(
                        "declare a C opaque pointer for `{}`",
                        prebindgen_flat::type_key(&pointer.ty)
                    ));
                }
            }
        }
        calls.push((f, body, context, source));
    }
    let mut tokens = quote! {
       #[repr(C)] #[derive(Clone,Copy)] pub struct __Pb128 {pub hi:u64,pub lo:u64}
       #[repr(C)] #[derive(Clone,Copy)] pub struct __PbRows {pub data:*const ::core::ffi::c_void,pub len:usize}
       #[repr(C)] #[derive(Clone,Copy)] pub struct __PbBuffer {pub data:*const u8,pub len:usize}
       std::thread_local! {static __PB_ERROR:std::cell::RefCell<Option<std::ffi::CString>>=const {std::cell::RefCell::new(None)};}
       fn __pb_error(e:&str){__PB_ERROR.with(|s|*s.borrow_mut()=if e.is_empty(){None}else{Some(std::ffi::CString::new(e.replace('\0',"?")).unwrap())});}
       #[no_mangle] pub extern "C" fn #error()->*const std::ffi::c_char {__PB_ERROR.with(|s|s.borrow().as_ref().map_or(c"".as_ptr(),|e| e.as_ptr()))}
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
    let mut header=format!("#ifndef {prefix}_H\n#define {prefix}_H\n#include <stdint.h>\n#include <stddef.h>\n#ifdef __cplusplus\nextern \"C\" {{\n#endif\ntypedef struct {{ uint64_t hi; uint64_t lo; }} {prefix}_int128;\n typedef struct {{ const uint8_t *data; size_t len; }} {prefix}_buffer;\nconst char *{error}(void);\nvoid {free}({prefix}_buffer b);\n");
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
            #[no_mangle] pub unsafe extern "C" fn #drop(ptr:*mut #ty)->i32{
                match ::prebindgen_runtime::boundary(||{if !ptr.is_null(){drop(Box::from_raw(ptr));}Ok(())}){Ok(())=>{__pb_error("");0},Err(e)=>{__pb_error(&e);-1}}
            }
        });
        header.push_str(&format!("typedef struct {prefix}_{name} {prefix}_{name};\nint32_t {drop}({prefix}_{name} *ptr);\n"));
    }
    header.push_str("/* Exported source-function status: 0 = success; -1 = failure before ownership acceptance; -2 = failure after ownership acceptance.\n * On -1 all owned inputs remain valid; on -2 active owned inputs have been consumed.\n * Failure leaves success-output storage unchanged. Read the error message before another library call. */\n");
    for (f, body, context, source) in calls {
        let mut schemas = Vec::new();
        for slot in f.input_conversions()? {
            sequence::schemas(slot.layout(), &mut schemas);
        }
        let mut output_schemas = Vec::new();
        sequence::schemas(f.output_conversion()?.layout(), &mut output_schemas);
        let output_carriers = output_schemas
            .iter()
            .map(|(wire, _)| wire.name.to_string())
            .collect::<std::collections::BTreeSet<_>>();
        schemas.extend(output_schemas);
        for (carrier, element) in schemas {
            let row = &context.row_types[&carrier.name.to_string()];
            let base = row.to_string();
            let base = base.trim_start_matches("__PbRow_");
            let c_row = format!("{prefix}_{base}_row");
            let c_rows = format!("{prefix}_{base}_rows");
            let free_rows = format_ident!("{prefix}_{base}_rows_free");
            let is_output = output_carriers.contains(&carrier.name.to_string());
            let mut row_symbols = vec![c_row.clone(), c_rows.clone()];
            if is_output {
                row_symbols.push(free_rows.to_string());
            }
            for symbol in row_symbols {
                if !symbols.insert(symbol) {
                    return Err("duplicate C collection symbol".into());
                }
            }
            let fields = element
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    let name = format_ident!("v{i}");
                    let ty = w.rust_type();
                    quote!(pub #name:#ty)
                })
                .collect::<Vec<_>>();
            tokens.extend(quote!(#[repr(C)] #[derive(Clone,Copy)] pub struct #row {#(#fields),*}));
            header.push_str("typedef struct {\n");
            for (i, w) in element.iter().enumerate() {
                header.push_str(&format!(
                    "    {} v{i};\n",
                    c_ty(w, prefix, &pointers, &context)
                ));
            }
            header.push_str(&format!(
                "}} {c_row};\ntypedef struct {{ {c_row} *data; size_t len; }} {c_rows};\n"
            ));
            if is_output {
                header.push_str(&format!("/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */\nvoid {free_rows}({c_rows} rows);\n"));
                tokens.extend(quote! {
                    #[doc="# Safety\nThe rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields."]
                    #[no_mangle]pub unsafe extern "C" fn #free_rows(rows:__PbRows){if rows.len!=0{drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(rows.data as *mut #row,rows.len)));}}
                });
            }
        }
        let out = &context.output_pointer;
        let accepted = &context.acknowledgement;
        let name = format_ident!("{prefix}_{}", f.name);
        let result = format_ident!("{name}_result");
        let inputs = f.inputs()?.collect::<Vec<_>>();
        let outputs = f
            .outputs(&context, &source)?
            .map(|o| o.wire)
            .collect::<Vec<_>>();
        let mut next_option_label = 0;
        for param in &f.params {
            if let Some(slot) = &param.ty.slot {
                header.push_str(&describe_groups(
                    slot.layout(),
                    &inputs,
                    false,
                    &mut next_option_label,
                    &param.name.to_string(),
                ));
            }
        }
        if let Some(slot) = &f.ret.slot {
            header.push_str(&describe_groups(
                slot.layout(),
                &outputs,
                true,
                &mut next_option_label,
                "return",
            ));
        }
        let mut params = Vec::new();
        let mut cparams = Vec::new();
        for w in f.inputs()? {
            let n = &w.name;
            let ty = w.kind.rust_type();
            params.push(quote!(#n:#ty));
            cparams.push(format!("{} {n}", c_ty(w, prefix, &pointers, &context)));
        }
        let mut fields = Vec::new();
        header.push_str("typedef struct {\n");
        for (i, output) in f.outputs(&context, &source)?.enumerate() {
            let w = output.wire;
            let field = format_ident!("v{i}");
            let ty = w.kind.rust_type();
            fields.push(quote!(pub #field:#ty));
            header.push_str(&format!(
                "    {} v{i};\n",
                c_ty(w, prefix, &pointers, &context)
            ));
        }
        if fields.is_empty() {
            fields.push(quote!(pub unused:u8));
            header.push_str("    uint8_t unused;\n");
        }
        header.push_str(&format!("}} {result};\n"));
        cparams.push(format!("{result} *{out}"));
        header.push_str(&format!("int32_t {name}({});\n", cparams.join(", ")));
        tokens.extend(quote! {
         #[repr(C)] pub struct #result {#(#fields),*}
         #[doc = "# Safety
The output pointer must be writable. Active input buffers must be readable for their stated lengths. Opaque pointers must be live allocations of the declared type from this library. Consuming inputs become invalid at ownership transfer; borrows remain valid and must obey Rust aliasing rules for the duration of the call."]
   #[allow(clippy::needless_borrow, clippy::let_unit_value, clippy::unnecessary_cast, clippy::useless_conversion)]
   #[no_mangle] pub unsafe extern "C" fn #name(#(#params,)* #out:*mut #result)->i32{
          if #out.is_null(){__pb_error("null result pointer");return -1;}
          #[allow(unused_mut)] let mut #accepted = false;
          match ::prebindgen_runtime::boundary(||{
           #body
          }){Ok(())=>{__pb_error("");0},Err(e)=>{__pb_error(&e);if #accepted {-2} else {-1}}}
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
    layout: &prebindgen_flat::WireLayout<CWire, CGroup>,
    wires: &[&Wire],
    output: bool,
    next_label: &mut usize,
    root: &str,
) -> String {
    fn visit(
        layout: &prebindgen_flat::WireLayout<CWire, CGroup>,
        names: &std::collections::BTreeMap<String, String>,
        parents: &mut Vec<String>,
        out: &mut String,
        next_label: &mut usize,
        field_path: &mut Vec<String>,
    ) {
        for node in layout.nodes() {
            if let prebindgen_flat::WireNode::Sequence { carrier, element } = node {
                let path = field_path.join(".");
                out.push_str(&format!("/* Collection {path}: {} carries rows of {} ABI fields; element-owned fields are released separately from row storage. */\n",carrier.name,element.len()));
                let row_names = element
                    .iter()
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
            if let prebindgen_flat::WireNode::Group { metadata, children } = node {
                if let CGroup::Optional(g) = metadata {
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
                    out.push_str(&format!("/* Option group {}: {rule}{parent}. */\n", label));
                    parents.push(label);
                    visit(children, names, parents, out, next_label, field_path);
                    parents.pop();
                } else if let CGroup::Field(name) = metadata {
                    field_path.push(name.to_string().trim_start_matches("r#").into());
                    let values = children
                        .iter()
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
                flat.function("f")
                    .unwrap()
                    .ret
                    .slot
                    .as_ref()
                    .unwrap()
                    .layout
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
        let header = Cbindgen::new(flat).build().unwrap().header;
        for i in 0..6 {
            assert_eq!(
                header.matches(&format!("Option group option{i}:")).count(),
                1
            );
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
        assert!(header.contains("Field return.pair: output->v0, output->v1."));
        assert!(header.contains("Field return.pair.x: output->v0."));
        assert!(header.contains("Field return.pair.y: output->v1."));
        assert!(header.contains("Field return.maybe: output->v2, output->v3."));
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
            let plan = flat.function("echo").unwrap().params[0]
                .ty
                .slot
                .as_ref()
                .unwrap();
            assert_eq!(plan.layout.len(), 3);
            // Check suppression before an outer option can clear the arm niches.
            for wire in plan.layout.iter().skip(1) {
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
            assert_eq!(plan.layout.len(), 4);
            let prebindgen_flat::WireNode::Group {
                metadata: group, ..
            } = &plan.layout.nodes()[0]
            else {
                panic!()
            };
            let CGroup::Optional(optional) = group else {
                panic!()
            };
            assert!(optional.niche.is_none());
            assert_eq!(optional.discriminator, plan.layout[0].name);
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
    fn emits_custom_slot_code_without_building_a_plan() {
        struct CountedOutput(
            std::rc::Rc<std::cell::Cell<usize>>,
            prebindgen_flat::WireLayout<CWire, CGroup>,
        );
        impl prebindgen_flat::Conversion for CountedOutput {
            type Wire = CWire;
            type Group = CGroup;
            type SlotFragment<'a> = prebindgen_conversions::OutputFragment<'a, CContext>;
            type WireFragment<'a> = prebindgen_conversions::OutputWireFragment<'a, CContext>;
            fn layout(&self) -> &prebindgen_flat::WireLayout<Self::Wire, Self::Group> {
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
            fn code_fragment_wire(
                &self,
                pos: usize,
                _: Self::WireFragment<'_>,
                _: &syn::Type,
            ) -> Result<TokenStream> {
                Err(format!("invalid output wire position {pos}"))
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
        let output = generate(flat, "test", vec![], vec![]).unwrap();
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
