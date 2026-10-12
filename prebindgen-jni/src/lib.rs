//! JNI/Kotlin adapter over the same prepared conversion slots as C.
//! The retained declaration subset is `package!`, `ptr_class!`, `fun!`,
//! expansion declarations, `.name()`, source selection, package prefix and native initialization.
//! Declaration processing fills occurrence slots before wrappers are generated.
use prebindgen_flat::{type_key, Flat, Result};
mod scalar;
use prebindgen_flat::Primitive;
pub use prebindgen_tools::boxed::BoxedWire as HandleWire;
use prebindgen_tools::{
    Access, Codec, Prepared as GenericPrepared, Slot as GenericSlot, WireKind as GenericWireKind,
};
#[derive(Clone, Debug)]
pub struct Handle;
type Slot = GenericSlot<Handle>;
type Prepared = GenericPrepared<HandleWire>;
type WireKind = GenericWireKind<HandleWire>;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};

pub use prebindgen_tools::{
    expand_param, expand_return, fields, fun, ExpandDecl, ExpandParamDecl, ExpandReturnDecl,
    FieldsDecl, FunctionDecl,
};
#[derive(Clone)]
pub struct PtrClassDecl {
    pub ty: syn::Type,
    pub name: Option<String>,
}
impl PtrClassDecl {
    pub fn new(ty: syn::Type) -> Self {
        Self { ty, name: None }
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
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
    pub fn new(name: impl Into<String>) -> Self {
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

pub struct JniGen {
    pub rust: String,
    pub kotlin: BTreeMap<String, String>,
}
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
    items: Option<Vec<(syn::Item, prebindgen::SourceLocation)>>,
    expansions: Vec<ExpandDecl>,
    prefix: String,
    init: String,
    packages: Vec<PackageDecl>,
}
impl JniGenBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn configure(mut self, f: impl FnOnce(&mut Flat<Slot>) -> Result<()> + 'static) -> Self {
        self.configure.push(Box::new(f));
        self
    }
    pub fn items(
        mut self,
        items: impl IntoIterator<Item = (syn::Item, prebindgen::SourceLocation)>,
    ) -> Self {
        self.items.get_or_insert_with(Vec::new).extend(items);
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
    pub fn set_package_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
    pub fn set_jni_native_init(mut self, init: impl Into<String>) -> Self {
        self.init = init.into();
        self
    }
    pub fn package(mut self, package: PackageDecl) -> Self {
        self.packages.push(package);
        self
    }
    pub fn build(self) -> Result<JniGen> {
        let references = prebindgen_tools::declarations::source_functions(
            &self.expansions,
            self.packages.iter().flat_map(|p| &p.functions),
        );
        let types: Vec<_> = self
            .packages
            .iter()
            .flat_map(|p| p.classes.iter().map(|c| c.ty.clone()))
            .collect();
        let mut flat = match (self.flat, self.source, self.items) {
            (Some(flat), None, None) => flat,
            (None, Some(path), None) => Flat::parse_selected(
                prebindgen::Source::new(path).items_all(),
                &references,
                &types,
            )?,
            (None, None, Some(items)) => Flat::parse_selected(items, &references, &types)?,
            _ => {
                return Err(
                    "supply exactly one source, item stream or configured flat model".into(),
                )
            }
        };
        for configure in self.configure {
            configure(&mut flat)?;
        }
        let defaults = prebindgen_tools::Declarations::new(&flat, self.expansions)?;
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
                if name == "JNINative" || name == "Functions" || name == "NativeHandle" {
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
                defaults.fill(&mut flat, &f, &JniCodec)?;
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
    type Leaf = Handle;
    type Wire = HandleWire;
    fn identity(&self, _: &syn::Type) -> Result<Handle> {
        Ok(Handle)
    }
    fn consumes(&self, _: &Handle) -> bool {
        true
    }
    fn input_foreign(
        &self,
        ty: &syn::Type,
        _: &Handle,
        w: syn::Ident,
        v: syn::Ident,
        active: &prebindgen_tools::Condition,
    ) -> Result<prebindgen_tools::PreparedInput<HandleWire>> {
        prebindgen_tools::boxed::input(
            &prebindgen_tools::boxed::Storage::Wrapped(syn::parse_quote!(
                ::prebindgen_jni_runtime::Allocation
            )),
            ty,
            w,
            v,
            active,
        )
    }
    fn output_foreign(
        &self,
        ty: &syn::Type,
        _: &Handle,
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<prebindgen_tools::PreparedOutput<HandleWire>> {
        prebindgen_tools::boxed::output(
            &prebindgen_tools::boxed::Storage::Wrapped(syn::parse_quote!(
                ::prebindgen_jni_runtime::Allocation
            )),
            ty,
            w,
            v,
            borrowed,
        )
    }
    fn decode_scalar(&self, p: Primitive, w: &syn::Ident) -> TokenStream {
        scalar::decode(p, w)
    }
    fn encode_scalar(&self, p: Primitive, v: TokenStream) -> TokenStream {
        scalar::encode(p, v)
    }
    fn decode_text(&self, w: &syn::Ident, _ty: &syn::Type) -> TokenStream {
        quote!(::prebindgen_jni_runtime::text(&mut env,&#w)?)
    }
    fn decode_bytes(&self, w: &syn::Ident, _ty: &syn::Type) -> TokenStream {
        quote!(::prebindgen_jni_runtime::bytes(&mut env,&#w)?)
    }
}
/// Alias retained for callers of the initial slot prototype.
pub type Generation = JniGen;
impl JniGen {
    /// Relative Rust paths are resolved under Cargo's OUT_DIR, as in the production API.
    pub fn write_rust(&self, path: impl AsRef<std::path::Path>) -> Result<std::path::PathBuf> {
        let path = path.as_ref();
        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?)
                .join(path)
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, &self.rust).map_err(|e| e.to_string())?;
        Ok(path)
    }
    pub fn write_kotlin(&self, root: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
        let mut written = Vec::new();
        for (path, text) in &self.kotlin {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(&path, text).map_err(|e| e.to_string())?;
            written.push(path);
        }
        Ok(written)
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
        WireKind::Scalar(p) => scalar::rust_type(*p),
        WireKind::Foreign(_) => quote!(i64),
        WireKind::Selector => quote!(i32),
        WireKind::Text => quote!(::prebindgen_jni_runtime::jni::objects::JString<'local>),
        WireKind::Bytes => quote!(::prebindgen_jni_runtime::jni::objects::JByteArray<'local>),
    }
}
fn raw_kt(kind: &WireKind) -> &'static str {
    match kind {
        WireKind::Scalar(p) => scalar::raw_kotlin(*p),
        WireKind::Foreign(_) => "Long",
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
) -> Result<JniGen> {
    let mut rust = TokenStream::new();
    let mut files = BTreeMap::new();
    let mut packages: BTreeMap<String, String> = BTreeMap::new();
    let mut native = format!("package {prefix}\n\ninternal object JNINative {{\n");
    if !init.is_empty() {
        native.push_str(&format!("    init {{ {init} }}\n"));
    }
    files.insert(
        format!("{}/NativeHandle.kt", prefix.replace('.', "/")),
        native_handle(prefix),
    );
    for (index, c) in classes.values().enumerate() {
        let method = format!("drop{index}");
        let sym = symbol(prefix, &method);
        let ty = &c.ty;
        rust.extend(quote!{#[doc = "# Safety\nOnly generated Kotlin wrappers may call this native entry point, with their object monitors held and live pointers of the declared types."]
#[no_mangle]pub unsafe extern "system" fn #sym(mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,id:i64){if let Err(e)=::prebindgen_runtime::boundary(||unsafe {::prebindgen_jni_runtime::drop_pointer::<#ty>(id)}){::prebindgen_jni_runtime::fail(&mut env,&e);}}});
        native.push_str(&format!("    @JvmStatic external fun {method}(id: Long)\n"));
        let text=format!("package {}\n\npublic class {} internal constructor(raw: Long) : {prefix}.NativeHandle(raw) {{\n    override fun freeNative(raw: Long) {{ {prefix}.JNINative.{method}(raw) }}\n}}\n",c.pkg,c.name);
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
        let mut validation = TokenStream::new();
        let mut acquire = TokenStream::new();
        let mut resources = Vec::new();
        let mut locks = Vec::new();
        let mut consumed = Vec::new();
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
                    WireKind::Scalar(p) => scalar::input(*p, n),
                    WireKind::Foreign(handle) => {
                        let condition = kotlin_condition(&w.active);
                        locks.push(format!(
                            "if ({condition}) requireNotNull({n}) {{ \"null handle\" }} else null"
                        ));
                        if handle.access == Access::Move {
                            consumed.push(format!("if ({condition}) {n}!!.markConsumed()"));
                        }
                        (
                            format!("{}?", class(&classes, &handle.ty)?.fqn()),
                            format!("{n}?.raw ?: 0L"),
                        )
                    }
                    _ => (raw_kt(&w.kind).into(), n.to_string()),
                };
                kt_params.push(format!("{n}: {kt}"));
                kt_args.push(arg);
            }
            validation.extend(input.validation.clone());
            acquire.extend(input.acquire.clone());
            resources.extend(input.resources.clone());
            body.extend(input.statements.clone());
            args.push(input.argument.clone());
        }
        let has_consumes = !consumed.is_empty();
        let ownership_param = if has_consumes {
            params.push(
                quote!(__ownership: ::prebindgen_jni_runtime::jni::objects::JBooleanArray<'local>),
            );
            raw_params.push("ownership: BooleanArray".into());
            kt_args.push("ownership".into());
            quote!(if env.get_array_length(&__ownership).map_err(|e|e.to_string())? != 1 {return Err("invalid ownership acknowledgement".into());}
                env.set_boolean_array_region(&__ownership, 0, &[1]).map_err(|e|e.to_string())?;)
        } else {
            quote!()
        };
        let slots: usize = usize::from(has_consumes)
            + f.params
                .iter()
                .map(|p| {
                    let Prepared::Input(i) = &p.ty.slot else {
                        unreachable!()
                    };
                    i.wires
                        .iter()
                        .map(|w| match w.kind {
                            WireKind::Foreign(_) => 2,
                            WireKind::Scalar(p) => scalar::parameter_slots(p),
                            _ => 1,
                        })
                        .sum::<usize>()
                })
                .sum::<usize>();
        if slots > 255 {
            return Err(format!(
                "`{source}` needs {slots} JVM parameter slots; the limit is 255"
            ));
        }
        let path = &f.path;
        let output_body = &output.statements;
        let commit = &output.commit;
        let aliases = prebindgen_tools::validate_resources(&resources);
        let count = output.wires.len();
        let mut encode = TokenStream::new();
        let mut delivered = Vec::new();
        let mut result_types = Vec::new();
        for (i, (w, v)) in output.wires.iter().zip(&output.values).enumerate() {
            let (expr, kt, decode) = match &w.kind {
                WireKind::Scalar(p) => scalar::output(*p, v, i),
                WireKind::Foreign(ty) => {
                    let c = class(&classes, &ty.ty)?.fqn();
                    (
                        quote!(::prebindgen_jni_runtime::handle_object(&mut env,(#v) as u64)?),
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
   #[allow(clippy::needless_borrow, clippy::let_unit_value, clippy::unnecessary_cast, clippy::useless_conversion)]
   #[doc = "# Safety\nOnly generated Kotlin wrappers may call this native entry point, with their object monitors held and live pointers of the declared types."]
#[no_mangle]pub unsafe extern "system" fn #sym<'local>(mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,_class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,#(#params),*)->::prebindgen_jni_runtime::jni::sys::jobjectArray{
    match ::prebindgen_runtime::boundary(||{
     #validation #aliases #ownership_param #acquire #body let __result=#path(#(#args),*);#output_body
     let result=::prebindgen_jni_runtime::array(&mut env,#count)?;#encode
     #commit Ok(result.into_raw())
    }){Ok(result)=>result,Err(e)=>{::prebindgen_jni_runtime::fail(&mut env,&e);std::ptr::null_mut()}}
   }
  });
        native.push_str(&format!(
            "    @JvmStatic external fun {method}({}): Array<Any?>\n",
            raw_params.join(", ")
        ));
        let mut output_locals = String::new();
        let mut materialize = String::new();
        let mut cleanup = String::new();
        for (i, w) in output.wires.iter().enumerate() {
            if let WireKind::Foreign(handle) = &w.kind {
                let c = class(&classes, &handle.ty)?.fqn();
                output_locals.push_str(&format!("        var output{i}: {c}? = null\n"));
                materialize.push_str(&format!("            output{i} = {}\n", delivered[i]));
                delivered[i] = format!("output{i}");
                let index = classes
                    .keys()
                    .position(|key| *key == type_key(&handle.ty))
                    .unwrap();
                cleanup.push_str(&format!("            try {{ if (output{i} != null) output{i}.close() else {prefix}.JNINative.drop{index}(result[{i}] as Long) }} catch (cleanup: Throwable) {{ failure.addSuppressed(cleanup) }}\n"));
            }
        }
        let (generic, ret, end) = match delivered.len() {
            0 => ("", "Unit".into(), "Unit".into()),
            1 => ("", result_types[0].clone(), delivered[0].clone()),
            _ => {
                kt_params.push(format!("build: ({}) -> R", result_types.join(", ")));
                (
                    "<R> ",
                    "R".into(),
                    format!("build({})", delivered.join(", ")),
                )
            }
        };
        let end = if cleanup.is_empty() {
            end
        } else {
            format!("{output_locals}        try {{\n{materialize}            {end}\n        }} catch (failure: Throwable) {{\n{cleanup}            throw failure\n        }}")
        };
        let call = format!("{prefix}.JNINative.{method}({})", kt_args.join(", "));
        let call = if has_consumes {
            format!("val ownership = BooleanArray(1)\n        try {{ {call} }} finally {{ if (ownership[0]) {{ {} }} }}", consumed.join("; "))
        } else {
            call
        };
        let native_call = if locks.is_empty() {
            format!("val result = {call}\n        {end}")
        } else {
            format!("val result = {prefix}.withHandleLocks(listOfNotNull({})) {{\n        {call}\n    }}\n{end}", locks.join(", "))
        };
        let text = packages.entry(pkg).or_default();
        text.push_str(&format!(
            "public fun {generic}`{name}`({}): {ret} = run {{\n    {native_call}\n}}\n\n",
            kt_params.join(", ")
        ));
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
    Ok(JniGen {
        rust: prebindgen_tools::rust_source(rust)?,
        kotlin: files,
    })
}

#[cfg(test)]
mod tests;

fn kotlin_condition(condition: &prebindgen_tools::Condition) -> String {
    if condition.0.is_empty() {
        return "true".into();
    }
    condition
        .0
        .iter()
        .map(|(tag, index)| format!("{tag} == {index}"))
        .collect::<Vec<_>>()
        .join(" && ")
}
fn native_handle(prefix: &str) -> String {
    include_str!("native_handle.kt").replace("PACKAGE", prefix)
}
