//! The JNI adapter's generator.
//!
//! [`Gen`] resolves the declarations against the flat model once (classes,
//! conversions, expansions) and then writes every declared item: the Rust
//! side through `prebindgen-tools`' writers, the Kotlin side as text.
//!
//! How a value crosses is decided in [`codec`]: every type lowers to a list
//! of [`leaf::Leaf`]s, with four conversions between them and the value —
//! Kotlin encode/decode and Rust decode/encode. Output *deliveries* (a result
//! handed to a builder, a callback's arguments, an error handler's fields)
//! are in [`deliver`]; input *selectors* in [`select`].

pub(crate) mod codec;
pub(crate) mod deliver;
pub(crate) mod func;
pub(crate) mod kotlin;
pub(crate) mod leaf;
pub(crate) mod pack;
pub(crate) mod rust;
pub(crate) mod select;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};

use prebindgen_tools::{
    flat::{
        flat::{Function, Type as FlatType},
        Flat,
    },
    names, FnRef, Qualifier, ResolvedConversion, RustFile,
};
use proc_macro2::TokenStream;

use crate::{
    builder::JniGenBuilder,
    decl::{
        ClassDecl, ConstDecl, ExpandDecl, ExpandParamDecl, ExpandReturnDecl, FunctionDecl, Iface,
    },
    Error, Generation,
};

pub(crate) type Res<T> = Result<T, Error>;

pub(crate) fn err<T>(msg: impl Into<String>) -> Res<T> {
    Err(Error(msg.into()))
}

/// A resolved class.
#[derive(Debug)]
pub(crate) struct Class {
    pub rust: String,
    pub kind: ClassKind,
    /// Kotlin package, fully qualified.
    pub pkg: String,
    /// Kotlin simple name.
    pub name: String,
    pub iface: Option<String>,
    pub implements: Vec<String>,
    pub methods: Vec<FunctionDecl>,
    pub constructors: Vec<FunctionDecl>,
}

#[derive(Debug)]
pub(crate) enum ClassKind {
    Ptr { gc: bool },
    Data { packed: bool },
    Enum,
    Sealed { renames: HashMap<String, String> },
}

impl Class {
    pub(crate) fn fqn(&self) -> String {
        format!("{}.{}", self.pkg, self.name)
    }
}

/// A resolved conversion.
pub(crate) struct Conv {
    pub name: String,
    pub resolved: ResolvedConversion,
    pub range: Option<(u128, u128)>,
}

/// Where a function lands in Kotlin.
#[derive(Clone)]
pub(crate) enum Placement {
    Package,
    Method(Rc<Class>),
    Constructor(Rc<Class>),
}

/// A function being bound.
pub(crate) struct Bound {
    pub decl: FunctionDecl,
    pub func: Function,
    pub callee: TokenStream,
    pub placement: Placement,
    /// The Kotlin name.
    pub kt_name: String,
    /// The `external fun` name in the native holder.
    pub ext_name: String,
}

pub(crate) struct Gen<'a> {
    pub b: &'a JniGenBuilder,
    pub flat: &'a Flat,
    pub q: Qualifier<'a>,
    pub classes: HashMap<String, Rc<Class>>,
    pub converts: HashMap<String, Rc<Conv>>,
    pub ret_exp: HashMap<String, ExpandReturnDecl>,
    pub param_exp: HashMap<String, ExpandParamDecl>,
    pub base_pkg: String,
    pub harness: String,
    pub rust: RustFile,
    /// Kotlin declarations by package, in emission order.
    pub kt: BTreeMap<String, Vec<String>>,
    pub externs: Vec<String>,
    pub ext_names: HashSet<String>,
    /// Emitted Kotlin support declarations (callbacks, sinks, builders).
    pub kt_claimed: HashSet<String>,
    /// Interface member headers per class (by Rust name).
    pub iface_members: HashMap<String, Vec<String>>,
    pub report: Vec<String>,
    /// Set while writing a getter whose callee expression is the value.
    pub valued: bool,
}

pub(crate) fn generate(b: &JniGenBuilder, flat: &Flat) -> Res<Generation> {
    let mut g = Gen {
        b,
        flat,
        q: Qualifier::new(flat).with_default_module(b.source_module.clone()),
        classes: HashMap::new(),
        converts: HashMap::new(),
        ret_exp: HashMap::new(),
        param_exp: HashMap::new(),
        base_pkg: b.package_prefix.clone(),
        harness: String::new(),
        rust: RustFile::new(),
        kt: BTreeMap::new(),
        externs: Vec::new(),
        ext_names: HashSet::new(),
        kt_claimed: HashSet::new(),
        iface_members: HashMap::new(),
        report: Vec::new(),
        valued: false,
    };
    g.harness = match &b.harness_hook {
        Some(f) => f("JNINative"),
        None => "JNINative".to_string(),
    };
    prebindgen_tools::check_supported(flat).map_err(Error)?;
    g.resolve()?;
    g.run()?;
    let kotlin = g.kt_files();
    Ok(Generation {
        rust: g.rust,
        kotlin,
        report: g.report.join("\n") + "\n",
    })
}

impl<'a> Gen<'a> {
    pub(crate) fn pkg_fqn(&self, sub: &str) -> String {
        match (self.base_pkg.is_empty(), sub.is_empty()) {
            (_, true) => self.base_pkg.clone(),
            (true, false) => sub.to_string(),
            _ => format!("{}.{sub}", self.base_pkg),
        }
    }

    pub(crate) fn harness_fqn(&self) -> String {
        format!("{}.{}", self.base_pkg, self.harness)
    }

    /// Append a Kotlin declaration to a package.
    pub(crate) fn kt_push(&mut self, pkg: &str, decl: String) {
        self.kt.entry(pkg.to_string()).or_default().push(decl);
    }

    /// Resolve classes, conversions and expansions.
    fn resolve(&mut self) -> Res<()> {
        for p in &self.b.packages {
            let pkg = self.pkg_fqn(&p.name);
            for c in &p.classes {
                let (ty, name_override, kind, iface, methods, ctors, hook) = match c {
                    ClassDecl::Ptr(d) => (
                        &d.ty,
                        d.name.clone(),
                        ClassKind::Ptr { gc: d.gc },
                        &d.iface,
                        d.methods.clone(),
                        d.constructors.clone(),
                        &self.b.ptr_hook,
                    ),
                    ClassDecl::Data(d) => (
                        &d.ty,
                        d.name.clone(),
                        ClassKind::Data { packed: d.packed },
                        &d.iface,
                        d.methods.clone(),
                        d.constructors.clone(),
                        &self.b.data_hook,
                    ),
                    ClassDecl::Enum(d) => (
                        &d.ty,
                        d.name.clone(),
                        ClassKind::Enum,
                        &d.iface,
                        Vec::new(),
                        Vec::new(),
                        &self.b.enum_hook,
                    ),
                    ClassDecl::Sealed(d) => (
                        &d.ty,
                        d.name.clone(),
                        ClassKind::Sealed {
                            renames: d
                                .variants
                                .iter()
                                .filter_map(|v| v.name.clone().map(|n| (v.rust.clone(), n)))
                                .collect(),
                        },
                        &d.iface,
                        Vec::new(),
                        Vec::new(),
                        &self.b.data_hook,
                    ),
                };
                let rust = type_name(ty)?;
                let element = self
                    .flat
                    .declared_type(&rust)
                    .ok_or_else(|| Error(format!("`{rust}` is not a #[prebindgen] type")))?;
                match (&kind, element) {
                    (ClassKind::Ptr { .. }, _) => {}
                    (ClassKind::Data { .. }, FlatType::Struct(_)) => {}
                    (ClassKind::Enum, FlatType::Enum(_)) => {}
                    (ClassKind::Sealed { .. }, FlatType::Variant(_)) => {}
                    _ => {
                        return err(format!(
                            "`{rust}`: the class kind does not match the declared type"
                        ))
                    }
                }
                let name = match (name_override, hook) {
                    (Some(n), _) => n,
                    (None, Some(f)) => f(&pkg, &rust),
                    (None, None) => rust.clone(),
                };
                let iface_name = iface_name(iface, &name);
                let class = Class {
                    rust: rust.clone(),
                    kind,
                    pkg: pkg.clone(),
                    name,
                    iface: iface_name,
                    implements: iface.implements.clone(),
                    methods,
                    constructors: ctors,
                };
                if self.classes.insert(rust.clone(), Rc::new(class)).is_some() {
                    return err(format!("`{rust}` is declared as a class twice"));
                }
            }
        }
        for c in &self.b.converts {
            let resolved = c.conversion.resolve(self.flat).map_err(Error)?;
            let name = match resolved.target.kind() {
                prebindgen_tools::flat::flat::TypeKind::Named { id, .. } => id.name.clone(),
                _ => return err(format!("convert!({}) must name a type", resolved.target)),
            };
            self.converts.insert(
                name.clone(),
                Rc::new(Conv {
                    name,
                    resolved,
                    range: c.range,
                }),
            );
        }
        for e in &self.b.expands {
            match e {
                ExpandDecl::Param(p) => {
                    self.param_exp.insert(type_name(&p.ty)?, p.clone());
                }
                ExpandDecl::Return(r) => {
                    self.ret_exp.insert(type_name(&r.ty)?, r.clone());
                }
            }
        }
        Ok(())
    }

    /// Resolve a function declaration: the model's function, the callee.
    pub(crate) fn resolve_fn(&self, f: &FnRef) -> Res<(Function, TokenStream)> {
        let func = f.resolve(self.flat).map_err(Error)?;
        Ok((func, f.callee(&self.q)))
    }

    fn run(&mut self) -> Res<()> {
        self.rust_prelude();
        self.kotlin_prelude();
        // Classes first: their Kotlin bodies collect members as they go.
        let mut classes: Vec<Rc<Class>> = self.classes.values().cloned().collect();
        classes.sort_by(|a, b| (&a.pkg, &a.name).cmp(&(&b.pkg, &b.name)));
        for c in &classes {
            self.class(c)?;
        }
        for p in &self.b.packages.clone() {
            let pkg = self.pkg_fqn(&p.name);
            for f in &p.funs {
                let (func, callee) = self.resolve_fn(&f.fun)?;
                let camel = names::camel(&names::bare(&func.name));
                let kt_name = match (&f.name, &self.b.fun_hook) {
                    (Some(n), _) => n.clone(),
                    (None, Some(h)) => h(&pkg, &camel),
                    (None, None) => camel,
                };
                let bound = self.bind(f.clone(), func, callee, Placement::Package, kt_name)?;
                let text = self.function(&bound)?;
                self.kt_push(&pkg, text);
            }
            for c in &p.consts {
                self.constant(&pkg, c)?;
            }
        }
        self.check_unbound();
        Ok(())
    }

    /// A bound function with its extern name chosen.
    pub(crate) fn bind(
        &mut self,
        decl: FunctionDecl,
        func: Function,
        callee: TokenStream,
        placement: Placement,
        kt_name: String,
    ) -> Res<Bound> {
        let camel = names::camel(&names::bare(&func.name));
        let base = match &self.b.method_hook {
            Some(h) => h(&self.base_pkg, &self.harness, &camel),
            None => camel,
        };
        let mut ext_name = base.clone();
        let mut i = 2;
        while !self.ext_names.insert(ext_name.clone()) {
            ext_name = format!("{base}{i}");
            i += 1;
        }
        Ok(Bound {
            decl,
            func,
            callee,
            placement,
            kt_name,
            ext_name,
        })
    }

    fn constant(&mut self, pkg: &str, c: &ConstDecl) -> Res<()> {
        let text = self.const_getter(pkg, c)?;
        self.kt_push(pkg, text);
        Ok(())
    }

    /// Warn about marked functions neither bound nor ignored.
    fn check_unbound(&self) {
        let mut bound: HashSet<String> = HashSet::new();
        for p in &self.b.packages {
            for f in &p.funs {
                bound.insert(f.rust_name());
            }
            for c in &p.classes {
                let (m, k): (&[FunctionDecl], &[FunctionDecl]) = match c {
                    ClassDecl::Ptr(d) => (&d.methods, &d.constructors),
                    ClassDecl::Data(d) => (&d.methods, &d.constructors),
                    _ => (&[], &[]),
                };
                for f in m.iter().chain(k) {
                    bound.insert(f.rust_name());
                }
            }
            for c in &p.consts {
                if let crate::decl::ConstSource::Fun(f) = &c.source {
                    bound.insert(f.rust_name());
                }
            }
        }
        for e in &self.b.expands {
            match e {
                ExpandDecl::Param(p) => {
                    for v in &p.variants {
                        if let crate::decl::ParamVariant::Build(f) = v {
                            bound.insert(f.rust_name());
                        }
                    }
                }
                ExpandDecl::Return(r) => {
                    for f in &r.fields {
                        match f {
                            crate::decl::ReturnField::Getter(g) => {
                                bound.insert(g.rust_name());
                            }
                            crate::decl::ReturnField::Form { fun, .. } => {
                                bound.insert(fun.to_string());
                            }
                            crate::decl::ReturnField::Handle => {}
                        }
                    }
                }
            }
        }
        for c in &self.b.converts {
            for v in [&c.conversion.input, &c.conversion.output]
                .into_iter()
                .flatten()
            {
                if let prebindgen_tools::Via::Fn(f) = v {
                    bound.insert(names::bare(f.name()));
                }
            }
        }
        let mut unbound: Vec<String> = self
            .flat
            .functions()
            .map(|f| names::bare(&f.name))
            .filter(|n| !bound.contains(n) && !self.b.ignores.iter().any(|i| (i.0)(n)))
            .collect();
        unbound.sort();
        for n in unbound {
            println!("cargo:warning=prebindgen-jni: skipping undeclared function `{n}`");
        }
    }
}

fn iface_name(iface: &Iface, class: &str) -> Option<String> {
    if !iface.enabled {
        return None;
    }
    Some(iface.name.clone().unwrap_or_else(|| format!("{class}Api")))
}

/// The name a declared type is looked up by.
pub(crate) fn type_name(ty: &syn::Type) -> Res<String> {
    match ty {
        syn::Type::Path(p) if p.qself.is_none() => Ok(p
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default()),
        _ => err(format!(
            "`{}` must name a type",
            quote::ToTokens::to_token_stream(ty)
        )),
    }
}
