//! Generate Rust wrappers and C declarations from conversion slots.
#![warn(missing_docs)]

use prebindgen_conversions::value::Value;
use prebindgen_flat::{Flat, Primitive, Result};
/// The carrier of one C wire.
///
/// This adapter-specific type keeps C and JNI converters from being mixed in
/// one model: their codecs use different carriers and allocation layouts.
#[derive(Clone, Debug)]
pub enum CWire {
    /// A primitive in its C scalar type.
    Scalar(Primitive),
    /// Text or bytes as a pointer/length buffer.
    Buffer,
    /// A typed opaque pointer to a boxed source value.
    Pointer(Box<CPointer>),
}
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
            mutable: value.access != prebindgen_flat::Access::Shared,
        }
    }
}
use prebindgen_declarations::Codec;
use prebindgen_flat::Wire as GenericWire;
mod emission;
pub use emission::CContext;
type InputPlan<W> = prebindgen_flat::InputPlan<W, CContext>;
type OutputPlan<W> = prebindgen_flat::OutputPlan<W, CContext>;
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
    type Extra = ();
    type Wire = CWire;
    type Context = CContext;
    type Input = InputPlan<CWire>;
    type Output = OutputPlan<CWire>;
    fn finish_output(&self, plan: &mut OutputPlan<CWire>, _: &()) -> Result<()> {
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
        let result_type = prebindgen_flat::template_parameter("output_type");
        let out = prebindgen_flat::template_parameter("output_pointer");
        plan.delivery = prebindgen_flat::Delivery {
            statements: quote!(let result = #result_type { #(#values),* }; std::ptr::write(#out, result);),
            result: quote!(()),
        };
        Ok(())
    }
    fn selector(&self, _: &()) -> CWire {
        CWire::Scalar(Primitive::I32)
    }
    fn consumes(&self, ty: &syn::Type, _: &()) -> bool {
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
        let read_buffer = prebindgen_flat::template_parameter("read_buffer");
        let (decode, kind) = match Value::of(ty) {
            Some(Value::Scalar(p)) => (decode_scalar(p, &w), CWire::Scalar(p)),
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
                    |b| CWire::Pointer(Box::new(b.into())),
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
        match Value::of(ty) {
            Some(Value::Scalar(p)) => prebindgen_conversions::value::output(
                ty,
                w,
                v,
                borrowed,
                |v| encode_scalar(p, v),
                CWire::Scalar(p),
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
                let buffer_type = prebindgen_flat::template_parameter("buffer_type");
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
                |b| CWire::Pointer(Box::new(b.into())),
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
    let wide_integer_type = prebindgen_flat::template_parameter("wide_integer_type");
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
/// `0` means success, `-1` means failure, with a thread-local last-error message.
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
                    .map(|(_, ty)| ty.clone())
                    .collect::<Vec<_>>(),
            )?,
            _ => return Err("supply exactly one source or flat model".into()),
        };
        let declarations = prebindgen_declarations::Declarations::new(&flat, self.expansions)?;
        let mut names = std::collections::BTreeSet::new();
        for decl in self.functions {
            if !names.insert(decl.name.clone()) {
                return Err("duplicate C function declaration".into());
            }
            if decl.foreign.is_some() {
                return Err("C function renaming is outside this subset".into());
            }
            declarations.fill(&mut flat, &CCodec, &decl, &())?;
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
        generate(flat, &self.prefix, pointers)
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
fn rust_ty(wire: &Wire) -> TokenStream {
    match &wire.kind {
        CWire::Scalar(p) => match p {
            Primitive::Bool => quote!(u8),
            Primitive::Char => quote!(u32),
            Primitive::I128 | Primitive::U128 => quote!(__Pb128),
            p => {
                let ty = p.rust_type();
                quote!(#ty)
            }
        },
        CWire::Pointer(p) => {
            let ty = &p.ty;
            if !p.mutable {
                quote!(*const #ty)
            } else {
                quote!(*mut #ty)
            }
        }
        CWire::Buffer => quote!(__PbBuffer),
    }
}
fn c_ty(wire: &Wire, prefix: &str, pointers: &[(String, syn::Type)]) -> String {
    match &wire.kind {
        CWire::Scalar(p) => match p {
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
        CWire::Pointer(p) => {
            let (name, _) = pointers
                .iter()
                .find(|(_, ty)| prebindgen_flat::same_type(ty, &p.ty))
                .expect("validated pointer declaration");
            let qualifier = if !p.mutable { "const " } else { "" };
            format!("{qualifier}{prefix}_{name} *")
        }
        CWire::Buffer => format!("{prefix}_buffer"),
    }
}
fn generate<
    I: prebindgen_flat::InputConversion<Wire = CWire, Context = CContext>,
    O: prebindgen_flat::OutputConversion<Wire = CWire, Context = CContext>,
>(
    flat: Flat<I, O>,
    prefix: &str,
    pointers: Vec<(String, syn::Type)>,
) -> Result<Generation> {
    let mut calls = Vec::new();
    let error = format_ident!("{prefix}_last_error");
    let free = format_ident!("{prefix}_free_buffer");
    let mut symbols = std::collections::BTreeSet::from([error.to_string(), free.to_string()]);
    for (name, _) in &pointers {
        if !symbols.insert(format!("{prefix}_drop_{name}")) {
            return Err("duplicate C destructor".into());
        }
    }
    for f in flat.functions.values() {
        if f.ret.slot.is_none() {
            continue;
        }
        let result_type = format_ident!("{prefix}_{}_result", f.name);
        let context = CContext {
            source_result: format_ident!("source_value"),
            output_pointer: format_ident!("output"),
            output_type: syn::parse_quote!(#result_type),
            buffer_type: syn::parse_quote!(__PbBuffer),
            wide_integer_type: syn::parse_quote!(__Pb128),
            read_buffer: syn::parse_quote!(__pb_read),
        };
        let body = prebindgen_flat::emit_call(f, &context)?;
        if !symbols.insert(format!("{prefix}_{}", f.name)) {
            return Err("duplicate C export".into());
        }
        for wire in f
            .inputs()?
            .chain(f.outputs(&context)?.map(|output| output.wire))
        {
            if let CWire::Pointer(pointer) = &wire.kind {
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
        calls.push((f, body, context));
    }
    let mut tokens = quote! {
       #[repr(C)] #[derive(Clone,Copy)] pub struct __Pb128 {pub hi:u64,pub lo:u64}
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
    for (f, body, context) in calls {
        let out = &context.output_pointer;
        let name = format_ident!("{prefix}_{}", f.name);
        let result = format_ident!("{name}_result");
        let mut params = Vec::new();
        let mut cparams = Vec::new();
        for w in f.inputs()? {
            let n = &w.name;
            let ty = rust_ty(w);
            params.push(quote!(#n:#ty));
            cparams.push(format!("{} {n}", c_ty(w, prefix, &pointers)));
        }
        let mut fields = Vec::new();
        header.push_str("typedef struct {\n");
        for (i, output) in f.outputs(&context)?.enumerate() {
            let w = output.wire;
            let field = format_ident!("v{i}");
            let ty = rust_ty(w);
            fields.push(quote!(pub #field:#ty));
            header.push_str(&format!("    {} v{i};\n", c_ty(w, prefix, &pointers)));
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
          match ::prebindgen_runtime::boundary(||{
           #body
          }){Ok(())=>{__pb_error("");0},Err(e)=>{__pb_error(&e);-1}}
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

#[cfg(test)]
mod generation_tests {
    use super::*;

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
        struct CountedOutput(std::rc::Rc<std::cell::Cell<usize>>);
        impl prebindgen_flat::OutputConversion for CountedOutput {
            type Wire = CWire;
            type Context = CContext;
            fn wires(&self) -> &[GenericWire<Self::Wire>] {
                &[]
            }
            fn values(&self, _: &CContext) -> Result<Vec<TokenStream>> {
                Ok(vec![])
            }
            fn preparation(&self, _: &CContext) -> Result<TokenStream> {
                self.0.set(self.0.get() + 1);
                Ok(quote!())
            }
            fn delivery(&self, _: &CContext) -> Result<TokenStream> {
                Ok(quote!())
            }
            fn commit(&self, _: &CContext) -> Result<TokenStream> {
                Ok(quote!())
            }
            fn result(&self, _: &CContext) -> Result<TokenStream> {
                Ok(quote!(()))
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
        flat.function_mut("noop").unwrap().ret.slot = Some(CountedOutput(count.clone()));
        let output = generate(flat, "test", vec![]).unwrap();
        assert!(output.header.contains("test_noop("));
        assert_eq!(count.get(), 1);
    }
}
