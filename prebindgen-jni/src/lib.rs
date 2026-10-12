//! JNI/Kotlin adapter over the same prepared conversion slots as C.
//! The retained declaration subset is `package!`, `ptr_class!`, `fun!`,
//! `.name()`, source selection, package prefix and native initialization.
//! Explicit occurrence converters are configured through `.flat(...)`.
use prebindgen_flat::{type_key, Flat, Result};
use prebindgen_tools::{Codec, Prepared, Slot, WireKind};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};

mod decl;
pub use decl::{ExpandDecl, ExpandParamDecl, ExpandReturnDecl};
#[derive(Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub foreign: Option<String>,
    pub(crate) params: BTreeMap<String, ExpandParamDecl>,
    pub(crate) ret: Option<ExpandReturnDecl>,
}
impl FunctionDecl {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            foreign: None,
            params: BTreeMap::new(),
            ret: None,
        }
    }
    pub fn name(mut self, name: &str) -> Self {
        self.foreign = Some(name.into());
        self
    }
    pub fn expand_param(mut self, name: &str, decl: ExpandParamDecl) -> Self {
        self.params.insert(name.into(), decl);
        self
    }
    pub fn expand_return(mut self, decl: ExpandReturnDecl) -> Self {
        self.ret = Some(decl);
        self
    }
}
#[derive(Clone)]
pub struct PtrClassDecl {
    pub ty: syn::Type,
    pub name: Option<String>,
}
impl PtrClassDecl {
    pub fn new(ty: syn::Type) -> Self {
        Self { ty, name: None }
    }
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }
}
#[derive(Clone, Default)]
pub struct PackageDecl {
    pub name: String,
    pub classes: Vec<PtrClassDecl>,
    pub functions: Vec<FunctionDecl>,
}
impl PackageDecl {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    pub fn class(mut self, class: PtrClassDecl) -> Self {
        self.classes.push(class);
        self
    }
    pub fn fun(mut self, f: FunctionDecl) -> Self {
        self.functions.push(f);
        self
    }
}
#[macro_export]
macro_rules! fun {
    ($name:ident) => {
        $crate::FunctionDecl::new(stringify!($name))
    };
}
#[macro_export]
macro_rules! ptr_class {($($ty:tt)+)=>{$crate::PtrClassDecl::new($crate::__type(stringify!($($ty)+)))};}
#[macro_export]
macro_rules! package {
    () => {
        $crate::PackageDecl::new("")
    };
    ($name:expr) => {
        $crate::PackageDecl::new($name)
    };
}
#[doc(hidden)]
pub fn __type(s: &str) -> syn::Type {
    syn::parse_str(s).expect("type syntax")
}

pub struct JniGen;
impl JniGen {
    pub fn builder() -> JniGenBuilder {
        JniGenBuilder::default()
    }
}
type Configure = Box<dyn FnOnce(&mut Flat<Slot>) -> Result<()>>;
#[derive(Default)]
pub struct JniGenBuilder {
    configure: Vec<Configure>,
    flat: Option<Flat<Slot>>,
    source: Option<std::path::PathBuf>,
    expansions: Vec<ExpandDecl>,
    prefix: String,
    init: String,
    packages: Vec<PackageDecl>,
}
impl JniGenBuilder {
    pub fn configure(mut self, f: impl FnOnce(&mut Flat<Slot>) -> Result<()> + 'static) -> Self {
        self.configure.push(Box::new(f));
        self
    }
    pub fn source(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.source = Some(path.as_ref().to_owned());
        self
    }
    pub fn expand(mut self, decl: impl Into<ExpandDecl>) -> Self {
        self.expansions.push(decl.into());
        self
    }
    pub fn flat(mut self, flat: Flat<Slot>) -> Self {
        self.flat = Some(flat);
        self
    }
    pub fn set_package_prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.into();
        self
    }
    pub fn set_jni_native_init(mut self, init: &str) -> Self {
        self.init = init.into();
        self
    }
    pub fn package(mut self, package: PackageDecl) -> Self {
        self.packages.push(package);
        self
    }
    pub fn build(self) -> Result<Generation> {
        let mut flat = match (self.flat, self.source) {
            (Some(flat), None) => flat,
            (None, Some(path)) => Flat::parse(prebindgen::Source::new(path).items_all())?,
            _ => return Err("supply exactly one source or configured flat model".into()),
        };
        for configure in self.configure {
            configure(&mut flat)?;
        }
        let defaults = decl::Defaults::new(&flat, self.expansions)?;
        validate_package(&self.prefix)?;
        let mut classes = BTreeMap::new();
        let mut functions = Vec::new();
        let mut seen = BTreeSet::new();
        let mut bound = BTreeSet::new();
        for p in self.packages {
            let pkg = if p.name.is_empty() {
                self.prefix.clone()
            } else {
                format!("{}.{}", self.prefix, p.name)
            };
            validate_package(&pkg)?;
            for c in p.classes {
                let ty = flat.resolve_type(&c.ty)?;
                let name = match c.name {
                    Some(n) => n,
                    None => match &c.ty {
                        syn::Type::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
                        _ => return Err("handle class needs a name".into()),
                    },
                };
                validate_name(&name)?;
                if name == "JNINative" || name == "Functions" {
                    return Err("reserved generated class name".into());
                }
                let fqn = format!("{pkg}.{name}");
                if !seen.insert(fqn.clone()) {
                    return Err(format!("duplicate foreign name `{fqn}`"));
                }
                if classes
                    .insert(
                        type_key(&ty),
                        Class {
                            pkg: pkg.clone(),
                            name,
                            ty,
                        },
                    )
                    .is_some()
                {
                    return Err("duplicate handle class for source type".into());
                }
            }
            for f in p.functions {
                defaults.fill(&mut flat, &f)?;
                let name = f.foreign.unwrap_or_else(|| camel(&f.name));
                validate_name(&name)?;
                if !seen.insert(format!("{pkg}.{name}")) {
                    return Err("duplicate foreign function name".into());
                }
                if !bound.insert(f.name.clone()) {
                    return Err("binding one source function twice is outside this subset".into());
                }
                functions.push((f.name, pkg.clone(), name));
            }
        }
        let flat = prebindgen_tools::prepare(flat, &JniCodec)?;
        for (name, f) in &flat.functions {
            if matches!(f.ret.slot, Prepared::Output(_)) && !bound.contains(name) {
                return Err(format!(
                    "configured function `{name}` has no Kotlin declaration"
                ));
            }
        }
        generate(flat, &self.prefix, &self.init, classes, functions)
    }
}
struct Class {
    pkg: String,
    name: String,
    ty: syn::Type,
}
impl Class {
    fn fqn(&self) -> String {
        format!("{}.{}", self.pkg, self.name)
    }
}
struct JniCodec;
impl Codec for JniCodec {
    fn decode_text(&self, w: &syn::Ident) -> TokenStream {
        quote!(::prebindgen_jni_runtime::text(&mut env,&#w)?)
    }
    fn decode_bytes(&self, w: &syn::Ident) -> TokenStream {
        quote!(::prebindgen_jni_runtime::bytes(&mut env,&#w)?)
    }
}
pub struct Generation {
    pub rust: String,
    pub kotlin: BTreeMap<String, String>,
}
impl Generation {
    pub fn write_rust(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        std::fs::write(path, &self.rust).map_err(|e| e.to_string())
    }
    /// The caller owns the destination. Only this generation's files are written.
    pub fn write_kotlin(&self, root: impl AsRef<std::path::Path>) -> Result<()> {
        for (path, text) in &self.kotlin {
            let path = root.as_ref().join(path);
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(path, text).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
fn validate_name(s: &str) -> Result<()> {
    if s.is_empty()
        || s.chars().all(|c| c == '_')
        || [
            "as",
            "break",
            "class",
            "continue",
            "do",
            "else",
            "false",
            "for",
            "fun",
            "if",
            "in",
            "interface",
            "is",
            "null",
            "object",
            "package",
            "return",
            "super",
            "this",
            "throw",
            "true",
            "try",
            "typealias",
            "typeof",
            "val",
            "var",
            "when",
            "while",
        ]
        .contains(&s)
        || !s
            .chars()
            .enumerate()
            .all(|(i, c)| c == '_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()))
    {
        return Err(format!("invalid Kotlin name `{s}`"));
    }
    Ok(())
}
fn validate_package(s: &str) -> Result<()> {
    for p in s.split('.') {
        validate_name(p)?;
    }
    Ok(())
}
fn camel(s: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in s.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}
/// JNI escaping is defined over UTF-16, including underscores in identifiers.
fn escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.encode_utf16() {
        match c {
            46 | 47 => out.push('_'),
            95 => out.push_str("_1"),
            59 => out.push_str("_2"),
            91 => out.push_str("_3"),
            c if (c as u8).is_ascii_alphanumeric() && c < 128 => out.push(char::from(c as u8)),
            c => out.push_str(&format!("_0{c:04x}")),
        }
    }
    out
}
fn symbol(pkg: &str, method: &str) -> syn::Ident {
    format_ident!(
        "Java_{}_{}",
        escape(&format!("{pkg}.JNINative")),
        escape(method)
    )
}
fn raw_rust(kind: &WireKind) -> TokenStream {
    match kind {
        WireKind::Handle(_) => quote!(i64),
        WireKind::Selector => quote!(i32),
        WireKind::Text => quote!(::prebindgen_jni_runtime::jni::objects::JString<'local>),
        WireKind::Bytes => quote!(::prebindgen_jni_runtime::jni::objects::JByteArray<'local>),
    }
}
fn raw_kt(kind: &WireKind) -> &'static str {
    match kind {
        WireKind::Handle(_) => "Long",
        WireKind::Selector => "Int",
        WireKind::Text => "String?",
        WireKind::Bytes => "ByteArray?",
    }
}
fn class<'a>(classes: &'a BTreeMap<String, Class>, ty: &syn::Type) -> Result<&'a Class> {
    classes
        .get(&type_key(ty))
        .ok_or_else(|| format!("no handle class for `{}`", type_key(ty)))
}
fn generate(
    flat: Flat<Prepared>,
    prefix: &str,
    init: &str,
    classes: BTreeMap<String, Class>,
    functions: Vec<(String, String, String)>,
) -> Result<Generation> {
    let mut rust = TokenStream::new();
    let mut files = BTreeMap::new();
    let mut packages: BTreeMap<String, String> = BTreeMap::new();
    let mut native = format!("package {prefix}\n\npublic object JNINative {{\n");
    if !init.is_empty() {
        native.push_str(&format!("    init {{ {init} }}\n"));
    }
    let alive = symbol(prefix, "isAlive");
    rust.extend(quote!{#[no_mangle]pub extern "system" fn #alive(_env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,id:i64)->u8{u8::from(::prebindgen_runtime::is_alive(id as u64))}});
    native.push_str("    @JvmStatic external fun isAlive(id: Long): Boolean\n");
    for (index, c) in classes.values().enumerate() {
        let method = format!("drop{index}");
        let sym = symbol(prefix, &method);
        let ty = &c.ty;
        rust.extend(quote!{#[no_mangle]pub extern "system" fn #sym(mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,id:i64){if let Err(e)=::prebindgen_runtime::boundary(||::prebindgen_runtime::release::<#ty>(id as u64)){::prebindgen_jni_runtime::fail(&mut env,&e);}}});
        native.push_str(&format!("    @JvmStatic external fun {method}(id: Long)\n"));
        let text=format!("package {}\n\npublic class {} internal constructor(internal val raw: Long) : AutoCloseable {{\n    public val isClosed: Boolean get() = !{prefix}.JNINative.isAlive(raw)\n    override fun close() {{ {prefix}.JNINative.{method}(raw) }}\n}}\n",c.pkg,c.name);
        files.insert(format!("{}/{}.kt", c.pkg.replace('.', "/"), c.name), text);
    }
    for (index, (source, pkg, name)) in functions.into_iter().enumerate() {
        let f = flat.function(&source)?;
        let Prepared::Output(output) = &f.ret.slot else {
            return Err(format!("`{source}` has no output converter"));
        };
        let method = format!("call{index}");
        let sym = symbol(prefix, &method);
        let mut params = Vec::new();
        let mut raw_params = Vec::new();
        let mut kt_params = Vec::new();
        let mut kt_args = Vec::new();
        let mut body = TokenStream::new();
        let mut args = Vec::new();
        for p in &f.params {
            let Prepared::Input(input) = &p.ty.slot else {
                unreachable!()
            };
            for w in &input.wires {
                let n = &w.name;
                let rt = raw_rust(&w.kind);
                params.push(quote!(#n:#rt));
                raw_params.push(format!("{n}: {}", raw_kt(&w.kind)));
                let (kt, arg) = match &w.kind {
                    WireKind::Handle(ty) => (
                        format!("{}?", class(&classes, ty)?.fqn()),
                        format!("{n}?.raw ?: 0L"),
                    ),
                    _ => (raw_kt(&w.kind).into(), n.to_string()),
                };
                kt_params.push(format!("{n}: {kt}"));
                kt_args.push(arg);
            }
            body.extend(input.statements.clone());
            args.push(input.argument.clone());
        }
        let slots: usize = f
            .params
            .iter()
            .map(|p| {
                let Prepared::Input(i) = &p.ty.slot else {
                    unreachable!()
                };
                i.wires
                    .iter()
                    .map(|w| {
                        if matches!(w.kind, WireKind::Handle(_)) {
                            2
                        } else {
                            1
                        }
                    })
                    .sum::<usize>()
            })
            .sum();
        if slots > 255 {
            return Err(format!(
                "`{source}` needs {slots} JVM parameter slots; the limit is 255"
            ));
        }
        let path = &f.path;
        let output_body = &output.statements;
        let count = output.wires.len();
        let mut encode = TokenStream::new();
        let mut delivered = Vec::new();
        let mut result_types = Vec::new();
        for (i, (w, v)) in output.wires.iter().zip(&output.values).enumerate() {
            let (expr, kt, decode) = match &w.kind {
                WireKind::Handle(ty) => {
                    let c = class(&classes, ty)?.fqn();
                    (
                        quote!(::prebindgen_jni_runtime::handle_object(&mut env,#v)?),
                        c.clone(),
                        format!("{c}(result[{i}] as Long)"),
                    )
                }
                WireKind::Text => (
                    quote!(::prebindgen_jni_runtime::text_object(&mut env,&#v)?),
                    "String".into(),
                    format!("result[{i}] as String"),
                ),
                WireKind::Bytes => (
                    quote!(::prebindgen_jni_runtime::bytes_object(&mut env,&#v)?),
                    "ByteArray".into(),
                    format!("result[{i}] as ByteArray"),
                ),
                WireKind::Selector => return Err("selector output is unsupported".into()),
            };
            let i = i as i32;
            encode.extend(quote!{let object=#expr;env.set_object_array_element(&result,#i,&object).map_err(|e|e.to_string())?;env.delete_local_ref(object).map_err(|e|e.to_string())?;});
            delivered.push(decode);
            result_types.push(kt);
        }
        rust.extend(quote!{
   #[allow(clippy::needless_borrow, clippy::let_unit_value)]
   #[no_mangle]pub extern "system" fn #sym<'local>(mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,#(#params),*)->::prebindgen_jni_runtime::jni::sys::jobjectArray{
    match ::prebindgen_runtime::boundary(||{
     #body let __result=#path(#(#args),*);let mut __outputs=::prebindgen_runtime::Outputs::default();#output_body
     let result=::prebindgen_jni_runtime::array(&mut env,#count)?;#encode
     __outputs.commit();Ok(result.into_raw())
    }){Ok(result)=>result,Err(e)=>{::prebindgen_jni_runtime::fail(&mut env,&e);std::ptr::null_mut()}}
   }
  });
        native.push_str(&format!(
            "    @JvmStatic external fun {method}({}): Array<Any?>\n",
            raw_params.join(", ")
        ));
        let (generic, ret, end) = match delivered.len() {
            0 => ("", "Unit".into(), "    return Unit\n".into()),
            1 => (
                "",
                result_types[0].clone(),
                format!("    return {}\n", delivered[0]),
            ),
            _ => {
                kt_params.push(format!("build: ({}) -> R", result_types.join(", ")));
                (
                    "<R> ",
                    "R".into(),
                    format!("    return build({})\n", delivered.join(", ")),
                )
            }
        };
        let mut cleanup = String::new();
        for (i, w) in output.wires.iter().enumerate() {
            if let WireKind::Handle(ty) = &w.kind {
                let index = classes.keys().position(|key| *key == type_key(ty)).unwrap();
                cleanup.push_str(&format!(
                    "        {prefix}.JNINative.drop{index}(result[{i}] as Long)\n"
                ));
            }
        }
        let end = if cleanup.is_empty() {
            end
        } else {
            format!("    try {{\n{end}    }} catch (failure: Throwable) {{\n{cleanup}        throw failure\n    }}\n")
        };
        let text = packages.entry(pkg).or_default();
        text.push_str(&format!("public fun {generic}`{name}`({}): {ret} {{\n    val result = {prefix}.JNINative.{method}({})\n{end}}}\n\n",kt_params.join(", "),kt_args.join(", ")));
    }
    native.push_str("}\n");
    files.insert(format!("{}/JNINative.kt", prefix.replace('.', "/")), native);
    for (pkg, body) in packages {
        let path = format!("{}/Functions.kt", pkg.replace('.', "/"));
        if files
            .insert(
                path,
                format!("@file:Suppress(\"UNUSED_VARIABLE\")\npackage {pkg}\n\n{body}"),
            )
            .is_some()
        {
            return Err("Kotlin file collision".into());
        }
    }
    let guards = &flat.guards;
    rust.extend(quote!(#(#guards)*));
    Ok(Generation {
        rust: prebindgen_tools::rust_source(rust)?,
        kotlin: files,
    })
}

#[cfg(test)]
mod tests;
