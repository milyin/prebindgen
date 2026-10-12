//! C adapter for prepared occurrence slots. No recursive type conversion lives here.
use prebindgen_flat::{Flat, Primitive, Result};
use prebindgen_tools::{Codec, Prepared, Slot, WireKind};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

struct CCodec;
impl Codec for CCodec {
    fn decode_scalar(&self, p: Primitive, w: &syn::Ident) -> TokenStream {
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
    fn encode_scalar(&self, p: Primitive, v: TokenStream) -> TokenStream {
        match p {
            Primitive::Bool => quote!(u8::from(#v)),
            Primitive::Char => quote!((#v) as u32),
            Primitive::I128 | Primitive::U128 => {
                quote!({ let bits = (#v) as u128; __Pb128 { hi: (bits >> 64) as u64, lo: bits as u64 } })
            }
            _ => v,
        }
    }
    fn decode_text(&self, w: &syn::Ident) -> TokenStream {
        quote!(String::from_utf8(__pb_read(#w)?).map_err(|_|"invalid UTF-8")?)
    }
    fn decode_bytes(&self, w: &syn::Ident) -> TokenStream {
        quote!(__pb_read(#w)?)
    }
}
/// The caller pins conversion recipes on the flat model before building.
pub struct Cbindgen {
    flat: Option<Flat<Slot>>,
    source: Option<std::path::PathBuf>,
    expansions: Vec<prebindgen_tools::ExpandDecl>,
    functions: Vec<prebindgen_tools::FunctionDecl>,
    prefix: String,
    handles: Vec<(String, syn::Type)>,
}
impl Cbindgen {
    pub fn builder() -> Self {
        Self {
            flat: None,
            source: None,
            expansions: Vec::new(),
            functions: Vec::new(),
            prefix: "pb".into(),
            handles: Vec::new(),
        }
    }
    pub fn source(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.source = Some(path.as_ref().to_owned());
        self
    }
    pub fn expand(mut self, decl: impl Into<prebindgen_tools::ExpandDecl>) -> Self {
        self.expansions.push(decl.into());
        self
    }
    pub fn function(mut self, decl: prebindgen_tools::FunctionDecl) -> Self {
        self.functions.push(decl);
        self
    }

    pub fn new(flat: Flat<Slot>) -> Self {
        Self {
            flat: Some(flat),
            source: None,
            expansions: Vec::new(),
            functions: Vec::new(),
            prefix: "pb".into(),
            handles: Vec::new(),
        }
    }
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.into();
        self
    }
    pub fn handle(mut self, name: &str, ty: syn::Type) -> Self {
        self.handles.push((name.into(), ty));
        self
    }
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
                    .handles
                    .iter()
                    .map(|(_, ty)| ty.clone())
                    .collect::<Vec<_>>(),
            )?,
            _ => return Err("supply exactly one source or configured flat model".into()),
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
            declarations.fill(&mut flat, &decl)?;
        }
        let handles = self
            .handles
            .into_iter()
            .map(|(n, t)| Ok((n, flat.resolve_type(&t)?)))
            .collect::<Result<Vec<_>>>()?;
        let flat = prebindgen_tools::prepare(flat, &CCodec)?;
        generate(flat, &self.prefix, handles)
    }
}
pub struct Generation {
    pub rust: String,
    pub header: String,
}
impl Generation {
    pub fn write_rust(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        std::fs::write(path, &self.rust).map_err(|e| e.to_string())
    }
    pub fn write_header(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        std::fs::write(path, &self.header).map_err(|e| e.to_string())
    }
}
fn rust_ty(kind: &WireKind) -> TokenStream {
    match kind {
        WireKind::Scalar(p) => match p {
            Primitive::Bool => quote!(u8),
            Primitive::Char => quote!(u32),
            Primitive::I128 | Primitive::U128 => quote!(__Pb128),
            p => {
                let ty = p.rust_type();
                quote!(#ty)
            }
        },
        WireKind::Handle(_) => quote!(u64),
        WireKind::Selector => quote!(i32),
        _ => quote!(__PbBuffer),
    }
}
fn c_ty(kind: &WireKind, prefix: &str) -> String {
    match kind {
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
        WireKind::Handle(_) => "uint64_t".into(),
        WireKind::Selector => "int32_t".into(),
        _ => format!("{prefix}_buffer"),
    }
}
fn generate(
    flat: Flat<Prepared>,
    prefix: &str,
    handles: Vec<(String, syn::Type)>,
) -> Result<Generation> {
    let error = format_ident!("{prefix}_last_error");
    let free = format_ident!("{prefix}_free_buffer");
    let mut symbols = std::collections::BTreeSet::from([error.to_string(), free.to_string()]);
    for (name, _) in &handles {
        if !symbols.insert(format!("{prefix}_drop_{name}")) {
            return Err("duplicate C destructor".into());
        }
    }
    for f in flat.functions.values() {
        let Prepared::Output(out) = &f.ret.slot else {
            continue;
        };
        if !symbols.insert(format!("{prefix}_{}", f.name)) {
            return Err("duplicate C export".into());
        }
        let inputs = f.params.iter().flat_map(|p| match &p.ty.slot {
            Prepared::Input(i) => i.wires.iter(),
            _ => unreachable!(),
        });
        for wire in inputs.chain(out.wires.iter()) {
            if let WireKind::Handle(ty) = &wire.kind {
                if !handles
                    .iter()
                    .any(|(_, declared)| prebindgen_flat::same_type(declared, ty))
                {
                    return Err(format!(
                        "declare a C handle destructor for `{}`",
                        prebindgen_flat::type_key(ty)
                    ));
                }
            }
        }
    }
    let mut tokens = quote! {
       #[repr(C)] #[derive(Clone,Copy)] pub struct __Pb128 {pub hi:u64,pub lo:u64}
       #[repr(C)] #[derive(Clone,Copy)] pub struct __PbBuffer {pub data:*const u8,pub len:usize}
       std::thread_local! {static __PB_ERROR:std::cell::RefCell<std::ffi::CString>=std::cell::RefCell::new(std::ffi::CString::default());}
       fn __pb_error(e:&str){__PB_ERROR.with(|s|*s.borrow_mut()=std::ffi::CString::new(e.replace('\0',"?")).unwrap());}
       #[no_mangle] pub extern "C" fn #error()->*const std::ffi::c_char {__PB_ERROR.with(|s|s.borrow().as_ptr())}
       // Input pointers are borrowed only during the call; output buffers are owned.
       unsafe fn __pb_read(b:__PbBuffer)->Result<Vec<u8>,String>{
        if b.len==0{return Ok(Vec::new());}
        if b.data.is_null()||b.len>isize::MAX as usize{return Err("invalid buffer".into());}
        Ok(std::slice::from_raw_parts(b.data,b.len).to_vec())
       }
       fn __pb_buffer(v:Vec<u8>)->__PbBuffer {let b=v.into_boxed_slice();let len=b.len();let data=Box::into_raw(b) as *mut u8;__PbBuffer{data,len}}
       #[doc = "# Safety
    The buffer must have been returned by this library and not already freed."]
    #[no_mangle] pub unsafe extern "C" fn #free(b:__PbBuffer){if !b.data.is_null(){drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(b.data as *mut u8,b.len)));}}
      };
    let mut header=format!("#ifndef {prefix}_H\n#define {prefix}_H\n#include <stdint.h>\n#include <stddef.h>\n#ifdef __cplusplus\nextern \"C\" {{\n#endif\ntypedef struct {{ uint64_t hi; uint64_t lo; }} {prefix}_int128;\n typedef struct {{ const uint8_t *data; size_t len; }} {prefix}_buffer;\nconst char *{error}(void);\nvoid {free}({prefix}_buffer b);\n");
    for (name, ty) in handles {
        prebindgen_tools::ident(&name)?;
        let drop = format_ident!("{prefix}_drop_{name}");
        tokens.extend(quote!{#[no_mangle] pub extern "C" fn #drop(id:u64)->i32{match ::prebindgen_runtime::boundary(||::prebindgen_runtime::release::<#ty>(id)){Ok(())=>{__pb_error("");0},Err(e)=>{__pb_error(&e);-1}}}});
        header.push_str(&format!("int32_t {drop}(uint64_t id);\n"));
    }
    for f in flat.functions.values() {
        let Prepared::Output(output) = &f.ret.slot else {
            continue;
        };
        let name = format_ident!("{prefix}_{}", f.name);
        let result = format_ident!("{name}_result");
        let mut params = Vec::new();
        let mut cparams = Vec::new();
        let mut body = TokenStream::new();
        let mut args = Vec::new();
        for p in &f.params {
            let Prepared::Input(input) = &p.ty.slot else {
                unreachable!()
            };
            for w in &input.wires {
                let n = &w.name;
                let ty = rust_ty(&w.kind);
                params.push(quote!(#n:#ty));
                cparams.push(format!("{} {n}", c_ty(&w.kind, prefix)));
            }
            body.extend(input.statements.clone());
            args.push(input.argument.clone());
        }
        let mut fields = Vec::new();
        let mut values = Vec::new();
        header.push_str("typedef struct {\n");
        for (i, (w, v)) in output.wires.iter().zip(&output.values).enumerate() {
            let field = format_ident!("v{i}");
            let ty = rust_ty(&w.kind);
            fields.push(quote!(pub #field:#ty));
            let value = match &w.kind {
                WireKind::Text => quote!(__pb_buffer(#v.into_bytes())),
                WireKind::Bytes => quote!(__pb_buffer(#v)),
                _ => v.clone(),
            };
            values.push(quote!(#field:#value));
            header.push_str(&format!("    {} v{i};\n", c_ty(&w.kind, prefix)));
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
        tokens.extend(quote! {
         #[repr(C)] pub struct #result {#(#fields),*}
         #[doc = "# Safety
The output pointer must be writable. Each active input buffer must be readable for its stated length."]
   #[allow(clippy::needless_borrow, clippy::let_unit_value, clippy::unnecessary_cast)]
   #[no_mangle] pub unsafe extern "C" fn #name(#(#params,)*out:*mut #result)->i32{
          if out.is_null(){__pb_error("null result pointer");return -1;}
          match ::prebindgen_runtime::boundary(||{
           #body
           let __result=#call(#(#args),*);
           let mut __outputs=::prebindgen_runtime::Outputs::default();
           #output_body
           let result=#result{#(#values),*};
           std::ptr::write(out,result);
           __outputs.commit();Ok(())
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
