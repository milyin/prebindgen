//! Generate Rust wrappers and C declarations from prepared occurrence slots.
#![warn(missing_docs)]
use prebindgen_flat::{Flat, Primitive, Result};
/// Pointee type and access mode used by this adapter's foreign wires.
pub use prebindgen_tools::boxed::BoxedWire as PointerWire;
use prebindgen_tools::{Bindings as GenericBindings, Codec, Wire as GenericWire, WireKind};
/// Flat source model with C pointer conversion data for wrapper emission.
pub type Bindings = GenericBindings<Option<PointerWire>>;
type Wire = GenericWire<Option<PointerWire>>;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Leaf conversions used by [`Cbindgen`].
/// Use with [`prebindgen_tools::bind_input`] and [`prebindgen_tools::bind_output`]
/// to fill [`Bindings`] manually.
///
/// ```
/// use prebindgen_c::{Bindings, Cbindgen, CCodec};
/// use prebindgen_tools::{bind_input, bind_output, Input, Output};
/// let source = syn::parse_file("pub fn echo(x: i64) -> i64 { x }")?;
/// let mut flat = Bindings::parse(
///     source.items.into_iter().map(|item| (item, Default::default())),
/// )?;
/// bind_input(&mut flat, &CCodec, "echo", "x", &Input::Scalar, &())?;
/// bind_output(&mut flat, &CCodec, "echo", &Output::Scalar, &())?;
/// let generated = Cbindgen::new(flat).build()?;
/// assert!(generated.header.contains("pb_echo("));
/// syn::parse_file(&generated.rust)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct CCodec;
impl Codec for CCodec {
    type Extra = ();
    type Wire = Option<PointerWire>;
    fn wire_extra(&self, _: &syn::Type, _: &WireKind, _: &()) -> Self::Wire {
        None
    }
    fn consumes(&self, _: &syn::Type, _: &()) -> bool {
        true
    }
    fn input_foreign(
        &self,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: syn::Ident,
        active: &prebindgen_tools::Condition,
    ) -> Result<prebindgen_tools::InputConversion<Option<PointerWire>>> {
        prebindgen_tools::boxed::input(
            &prebindgen_tools::boxed::Storage::Direct,
            ty,
            w,
            v,
            active,
            Some,
        )
    }
    fn output_foreign(
        &self,
        ty: &syn::Type,
        _: &(),
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<prebindgen_tools::OutputConversion<Option<PointerWire>>> {
        prebindgen_tools::boxed::output(
            &prebindgen_tools::boxed::Storage::Direct,
            ty,
            w,
            v,
            borrowed,
            Some,
        )
    }
    fn decode_scalar(&self, p: Primitive, w: &syn::Ident, _: &()) -> TokenStream {
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
    fn encode_scalar(&self, p: Primitive, v: TokenStream, _: &()) -> TokenStream {
        match p {
            Primitive::Bool => quote!(u8::from(#v)),
            Primitive::Char => quote!((#v) as u32),
            Primitive::I128 | Primitive::U128 => {
                quote!({ let bits = (#v) as u128; __Pb128 { hi: (bits >> 64) as u64, lo: bits as u64 } })
            }
            _ => v,
        }
    }
    fn decode_text(&self, w: &syn::Ident, _ty: &syn::Type, _: &()) -> TokenStream {
        if matches!(_ty, syn::Type::Reference(r) if prebindgen_flat::type_key(&r.elem) == "str") {
            quote!(std::str::from_utf8(unsafe {__pb_read(#w)?}).map_err(|_|"invalid UTF-8")?)
        } else {
            quote!(std::str::from_utf8(unsafe {__pb_read(#w)?}).map_err(|_|"invalid UTF-8")?.to_owned())
        }
    }
    fn decode_bytes(&self, w: &syn::Ident, _ty: &syn::Type, _: &()) -> TokenStream {
        if matches!(_ty, syn::Type::Reference(r) if prebindgen_flat::type_key(&r.elem) == "[u8]") {
            quote!(unsafe {__pb_read(#w)?})
        } else {
            quote!(unsafe {__pb_read(#w)?}.to_vec())
        }
    }
}
/// Configure C bindings, using typed opaque pointers for foreign values.
///
/// Start with captured source via [`Self::builder`] and [`Self::source`], or a
/// flat model via [`Self::new`]. With captured source, declare each
/// exported function. In either case, declare each opaque pointee type.
/// Conversion declarations use [`prebindgen_tools::Declarations`]. When reading
/// captured source, add the source crate as both a build dependency and a normal
/// dependency: generation reads its captures, and generated code calls its functions.
///
/// ```
/// use prebindgen_c::{Bindings, Cbindgen};
/// use prebindgen_tools::fun;
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
    expansions: Vec<prebindgen_tools::ExpandDecl>,
    functions: Vec<prebindgen_tools::FunctionDecl>,
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
    /// Read captures from a source crate's `PREBINDGEN_OUT_DIR`.
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
    /// Register a type-wide input or output expansion.
    ///
    /// Per-function overrides replace this default; duplicate defaults are rejected
    /// by [`Self::build`]. Registration order does not affect type lookup.
    pub fn expand(mut self, decl: impl Into<prebindgen_tools::ExpandDecl>) -> Self {
        self.expansions.push(decl.into());
        self
    }
    /// Export a function and build its occurrence converters from declarations.
    ///
    /// Function renaming is currently unsupported; exports use the symbol prefix
    /// and original source name.
    pub fn function(mut self, decl: prebindgen_tools::FunctionDecl) -> Self {
        self.functions.push(decl);
        self
    }

    /// Start from a flat model with input and output conversion slots and prefix `pb`.
    ///
    /// Attach converters with [`prebindgen_tools::bind_input`] and
    /// [`prebindgen_tools::bind_output`], or fill defaults with [`Self::function`].
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
    /// Produces `{prefix}_{name}` and `{prefix}_drop_{name}`. Every foreign wire
    /// needs a matching declaration. Owned values use `Box<T>` directly; they have
    /// no `Send` requirement. This declaration alone does not export constructors.
    pub fn opaque_ptr(mut self, name: &str, ty: syn::Type) -> Self {
        self.pointers.push((name.into(), ty));
        self
    }
    /// Resolve declarations, prepare conversions and generate both output texts.
    ///
    /// Does not write files. Returns an error for conflicting sources, invalid or
    /// duplicate declarations, unsupported conversions, or missing opaque pointees.
    /// Rust layout/trait requirements are checked when the generated Rust compiles.
    pub fn build(self) -> Result<Generation> {
        prebindgen_tools::ident(&self.prefix)?;
        let mut flat = match (self.flat, self.source) {
            (Some(flat), None) => flat,
            (None, Some(path)) => Flat::parse_selected(
                prebindgen::Source::new(path).items_all(),
                &prebindgen_tools::declarations::source_functions(
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
        let declarations = prebindgen_tools::Declarations::new(&flat, self.expansions)?;
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
            .map(|(n, t)| Ok((n, flat.resolve_type(&t)?)))
            .collect::<Result<Vec<_>>>()?;
        prebindgen_tools::validate_bindings(&flat)?;
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
        WireKind::Scalar(p) => match p {
            Primitive::Bool => quote!(u8),
            Primitive::Char => quote!(u32),
            Primitive::I128 | Primitive::U128 => quote!(__Pb128),
            p => {
                let ty = p.rust_type();
                quote!(#ty)
            }
        },
        WireKind::Foreign => {
            let p = wire.extra.as_ref().expect("validated C pointer metadata");
            let ty = &p.ty;
            if p.access == prebindgen_tools::Access::Shared {
                quote!(*const #ty)
            } else {
                quote!(*mut #ty)
            }
        }
        WireKind::Selector => quote!(i32),
        _ => quote!(__PbBuffer),
    }
}
fn c_ty(wire: &Wire, prefix: &str, pointers: &[(String, syn::Type)]) -> String {
    match &wire.kind {
        WireKind::Scalar(p) => match p {
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
        WireKind::Foreign => {
            let p = wire.extra.as_ref().expect("validated C pointer metadata");
            let (name, _) = pointers
                .iter()
                .find(|(_, ty)| prebindgen_flat::same_type(ty, &p.ty))
                .expect("validated pointer declaration");
            let qualifier = if p.access == prebindgen_tools::Access::Shared {
                "const "
            } else {
                ""
            };
            format!("{qualifier}{prefix}_{name} *")
        }
        WireKind::Selector => "int32_t".into(),
        _ => format!("{prefix}_buffer"),
    }
}
fn generate(
    flat: Bindings,
    prefix: &str,
    pointers: Vec<(String, syn::Type)>,
) -> Result<Generation> {
    let error = format_ident!("{prefix}_last_error");
    let free = format_ident!("{prefix}_free_buffer");
    let mut symbols = std::collections::BTreeSet::from([error.to_string(), free.to_string()]);
    for (name, _) in &pointers {
        if !symbols.insert(format!("{prefix}_drop_{name}")) {
            return Err("duplicate C destructor".into());
        }
    }
    for f in flat.functions.values() {
        let Some(out) = &f.ret.slot else {
            continue;
        };
        if !symbols.insert(format!("{prefix}_{}", f.name)) {
            return Err("duplicate C export".into());
        }
        let inputs = f.params.iter().flat_map(|p| match &p.ty.slot {
            Some(i) => i.wires.iter(),
            _ => unreachable!(),
        });
        for wire in inputs.chain(out.wires.iter()) {
            if matches!(wire.kind, WireKind::Foreign) {
                let pointer = wire
                    .extra
                    .as_ref()
                    .ok_or("C foreign wire needs pointer metadata")?;
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
        prebindgen_tools::ident(name)?;
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
    for f in flat.functions.values() {
        let Some(output) = &f.ret.slot else {
            continue;
        };
        let name = format_ident!("{prefix}_{}", f.name);
        let result = format_ident!("{name}_result");
        let mut params = Vec::new();
        let mut cparams = Vec::new();
        let mut body = TokenStream::new();
        let mut validation = TokenStream::new();
        let mut acquire = TokenStream::new();
        let mut resources = Vec::new();
        let mut args = Vec::new();
        for p in &f.params {
            let Some(input) = &p.ty.slot else {
                unreachable!()
            };
            for w in &input.wires {
                let n = &w.name;
                let ty = rust_ty(w);
                params.push(quote!(#n:#ty));
                cparams.push(format!("{} {n}", c_ty(w, prefix, &pointers)));
            }
            validation.extend(input.validation.clone());
            acquire.extend(input.acquire.clone());
            resources.extend(input.resources.clone());
            body.extend(input.statements.clone());
            args.push(input.argument.clone());
        }
        let mut fields = Vec::new();
        let mut values = Vec::new();
        let mut delivery = TokenStream::new();
        let mut buffer_commit = TokenStream::new();
        header.push_str("typedef struct {\n");
        for (i, (w, v)) in output.wires.iter().zip(&output.values).enumerate() {
            let field = format_ident!("v{i}");
            let ty = rust_ty(w);
            fields.push(quote!(pub #field:#ty));
            let value = match &w.kind {
                WireKind::Text | WireKind::Bytes => {
                    let buffer = format_ident!("buffer{i}");
                    let bytes = if matches!(w.kind, WireKind::Text) {
                        quote!(#v.into_bytes())
                    } else {
                        v.clone()
                    };
                    delivery.extend(quote!(let #buffer = (#bytes).into_boxed_slice();));
                    buffer_commit.extend(quote!(let _ = Box::into_raw(#buffer);));
                    quote!(__PbBuffer {data: #buffer.as_ptr(), len: #buffer.len()})
                }
                _ => v.clone(),
            };
            values.push(quote!(#field:#value));
            header.push_str(&format!("    {} v{i};\n", c_ty(w, prefix, &pointers)));
        }
        if fields.is_empty() {
            fields.push(quote!(pub unused:u8));
            values.push(quote!(unused:0));
            header.push_str("    uint8_t unused;\n");
        }
        header.push_str(&format!("}} {result};\n"));
        cparams.push(format!("{result} *out"));
        header.push_str(&format!("int32_t {name}({});\n", cparams.join(", ")));
        let call = &f.path;
        let output_body = &output.statements;
        let commit = &output.commit;
        let aliases = prebindgen_tools::validate_resources(&resources);
        tokens.extend(quote! {
         #[repr(C)] pub struct #result {#(#fields),*}
         #[doc = "# Safety
The output pointer must be writable. Active input buffers must be readable for their stated lengths. Opaque pointers must be live allocations of the declared type from this library. Consuming inputs become invalid at ownership transfer; borrows remain valid and must obey Rust aliasing rules for the duration of the call."]
   #[allow(clippy::needless_borrow, clippy::let_unit_value, clippy::unnecessary_cast)]
   #[no_mangle] pub unsafe extern "C" fn #name(#(#params,)*out:*mut #result)->i32{
          if out.is_null(){__pb_error("null result pointer");return -1;}
          match ::prebindgen_runtime::boundary(||{
           #validation #aliases #acquire #body
           let __result=#call(#(#args),*);
           #output_body
           #delivery
           let result=#result{#(#values),*};
           std::ptr::write(out,result);
           #commit #buffer_commit Ok(())
          }){Ok(())=>{__pb_error("");0},Err(e)=>{__pb_error(&e);-1}}
         }
        });
    }
    let guards = &flat.guards;
    tokens.extend(quote!(#(#guards)*));
    header.push_str("#ifdef __cplusplus\n}\n#endif\n#endif\n");
    Ok(Generation {
        rust: prebindgen_tools::rust_source(tokens)?,
        header,
    })
}
