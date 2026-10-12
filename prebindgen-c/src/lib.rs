//! C adapter for prepared occurrence slots. No recursive type conversion lives here.
use prebindgen_flat::{Flat, Primitive, Result};
pub use prebindgen_tools::boxed::BoxedWire as PointerWire;
use prebindgen_tools::{
    Codec, Prepared as GenericPrepared, Slot as GenericSlot, WireKind as GenericWireKind,
};
#[derive(Clone, Debug)]
pub struct OpaquePointer;
type Slot = GenericSlot<OpaquePointer>;
type Prepared = GenericPrepared<PointerWire>;
type WireKind = GenericWireKind<PointerWire>;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

struct CCodec;
impl Codec for CCodec {
    type Leaf = OpaquePointer;
    type Wire = PointerWire;
    fn identity(&self, _: &syn::Type) -> Result<OpaquePointer> {
        Ok(OpaquePointer)
    }
    fn consumes(&self, _: &OpaquePointer) -> bool {
        true
    }
    fn input_foreign(
        &self,
        ty: &syn::Type,
        _: &OpaquePointer,
        w: syn::Ident,
        v: syn::Ident,
        active: &prebindgen_tools::Condition,
    ) -> Result<prebindgen_tools::PreparedInput<PointerWire>> {
        prebindgen_tools::boxed::input(&prebindgen_tools::boxed::Storage::Direct, ty, w, v, active)
    }
    fn output_foreign(
        &self,
        ty: &syn::Type,
        _: &OpaquePointer,
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<prebindgen_tools::PreparedOutput<PointerWire>> {
        prebindgen_tools::boxed::output(
            &prebindgen_tools::boxed::Storage::Direct,
            ty,
            w,
            v,
            borrowed,
        )
    }
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
    fn decode_text(&self, w: &syn::Ident, _ty: &syn::Type) -> TokenStream {
        if matches!(_ty, syn::Type::Reference(r) if prebindgen_flat::type_key(&r.elem) == "str") {
            quote!(std::str::from_utf8(unsafe {__pb_read(#w)?}).map_err(|_|"invalid UTF-8")?)
        } else {
            quote!(std::str::from_utf8(unsafe {__pb_read(#w)?}).map_err(|_|"invalid UTF-8")?.to_owned())
        }
    }
    fn decode_bytes(&self, w: &syn::Ident, _ty: &syn::Type) -> TokenStream {
        if matches!(_ty, syn::Type::Reference(r) if prebindgen_flat::type_key(&r.elem) == "[u8]") {
            quote!(unsafe {__pb_read(#w)?})
        } else {
            quote!(unsafe {__pb_read(#w)?}.to_vec())
        }
    }
}
/// C declarations resolve domain types to typed opaque pointers.
pub struct Cbindgen {
    flat: Option<Flat<Slot>>,
    source: Option<std::path::PathBuf>,
    expansions: Vec<prebindgen_tools::ExpandDecl>,
    functions: Vec<prebindgen_tools::FunctionDecl>,
    prefix: String,
    pointers: Vec<(String, syn::Type)>,
}
impl Cbindgen {
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
            pointers: Vec::new(),
        }
    }
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.into();
        self
    }
    pub fn opaque_ptr(mut self, name: &str, ty: syn::Type) -> Self {
        self.pointers.push((name.into(), ty));
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
                    .pointers
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
            declarations.fill(&mut flat, &decl, &CCodec)?;
        }
        let pointers = self
            .pointers
            .into_iter()
            .map(|(n, t)| Ok((n, flat.resolve_type(&t)?)))
            .collect::<Result<Vec<_>>>()?;
        let flat = prebindgen_tools::prepare(flat, &CCodec)?;
        generate(flat, &self.prefix, pointers)
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
        WireKind::Foreign(p) => {
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
fn c_ty(kind: &WireKind, prefix: &str, pointers: &[(String, syn::Type)]) -> String {
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
        WireKind::Foreign(p) => {
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
    flat: Flat<Prepared>,
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
            if let WireKind::Foreign(pointer) = &wire.kind {
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
        let Prepared::Output(output) = &f.ret.slot else {
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
            let Prepared::Input(input) = &p.ty.slot else {
                unreachable!()
            };
            for w in &input.wires {
                let n = &w.name;
                let ty = rust_ty(&w.kind);
                params.push(quote!(#n:#ty));
                cparams.push(format!("{} {n}", c_ty(&w.kind, prefix, &pointers)));
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
            let ty = rust_ty(&w.kind);
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
            header.push_str(&format!("    {} v{i};\n", c_ty(&w.kind, prefix, &pointers)));
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
