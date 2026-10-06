//! The plan: the build script's declarations resolved against the flat
//! model.
//!
//! It holds two things. The **settings** say how each declared type
//! crosses — a Kotlin class of some kind, or a conversion — and what its
//! default boundary shape is (`expand_param!`, `expand_return!`); the
//! lowering in [`lower`](crate::lower) reads them. The **items** are the
//! elements to write — package by package, each package's classes, then
//! its functions, then its constants — each with every decision about it
//! already made: its Kotlin and extern names, how each parameter crosses
//! (a value or a selector), how the result and the error leave. The
//! Kotlin support interfaces those decisions need — callback, sink, builder,
//! folder and error-handler interfaces — are planned alongside, once each.
//!
//! Everything a declaration can get wrong is found here;
//! [`write`](crate::write) then writes the items one by one.

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use prebindgen_flat::{
    flat::{Function, Param, Type as FlatType, TypeKind, TypeRef},
    Flat,
};
use prebindgen_tools::{names, FnRef, Qualifier, ResolvedConversion};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    builder::JniGenBuilder,
    decl::{
        ClassDecl, ConstDecl, ConstSource, ExpandDecl, ExpandParamDecl, ExpandReturnDecl,
        FunctionDecl, ParamVariant, ReturnField,
    },
    lower::{
        deliver::DParam,
        leaf::{Leaf, LeafTy, Prim},
        select::Selector,
        Dir,
    },
    Error,
};

pub(crate) type Res<T> = Result<T, Error>;

pub(crate) fn err<T>(msg: impl Into<String>) -> Res<T> {
    Err(Error(msg.into()))
}

// ── settings ────────────────────────────────────────────────────────────

/// A Kotlin class a source type is bound to.
#[derive(Debug)]
pub(crate) struct Class {
    /// The source type's name.
    pub rust: String,
    pub kind: ClassKind,
    /// Kotlin package, fully qualified.
    pub pkg: String,
    /// Kotlin simple name.
    pub name: String,
    /// The `…Api` interface the class implements, when declared.
    pub iface: Option<String>,
    pub implements: Vec<String>,
}

#[derive(Debug)]
pub(crate) enum ClassKind {
    Ptr { gc: bool },
    Data,
    Enum,
    Sealed { renames: HashMap<String, String> },
}

impl Class {
    pub(crate) fn fqn(&self) -> String {
        format!("{}.{}", self.pkg, self.name)
    }
}

/// A type crossing as its representation.
pub(crate) struct Conv {
    pub name: String,
    pub resolved: ResolvedConversion,
    pub range: Option<(u128, u128)>,
}

/// How a declared type crosses.
pub(crate) enum Setting {
    Class(Rc<Class>),
    Converted(Rc<Conv>),
}

// ── items ───────────────────────────────────────────────────────────────

/// Where a bound function lands in Kotlin.
#[derive(Clone)]
pub(crate) enum Placement {
    Package,
    Method(Rc<Class>),
    Constructor(Rc<Class>),
}

/// How the extern reaches the value.
pub(crate) enum Callee {
    /// Call this path with the arguments.
    Call(TokenStream),
    /// Evaluate this expression: a constant's value.
    Value(TokenStream),
}

/// How one source parameter crosses.
pub(crate) enum PPlan {
    /// The receiver of a method: `this`.
    Receiver(Param),
    /// A value.
    Value(Param),
    /// Built or passed as chosen by a selector; `true` when split into
    /// typed overloads.
    Selector(Selector, bool),
}

/// How a successful result crosses.
pub(crate) enum RPlan {
    Unit,
    /// One leaf, returned as the extern's own result.
    Direct {
        ty: TypeRef,
        leaf: Leaf,
    },
    /// Several leaves, handed to a sink that assembles the value.
    Sink {
        ty: TypeRef,
        leaves: Vec<Leaf>,
        iface: String,
        kt_sink: String,
    },
    /// Delivered to a caller-supplied builder (`R`), `null` when absent.
    Builder {
        whole: TypeRef,
        exp: ExpandReturnDecl,
        optional: bool,
        iface: String,
        leaves: Vec<Leaf>,
    },
    /// A sequence folded by a caller-supplied folder (`A`), `null` when
    /// absent.
    Fold {
        whole: TypeRef,
        exp: ExpandReturnDecl,
        optional: bool,
        iface: String,
        columns_iface: String,
        params: Vec<DParam>,
        leaves: Vec<Leaf>,
    },
}

/// How an `Err` crosses.
#[allow(clippy::large_enum_variant)] // one per bound function
pub(crate) enum EPlan {
    None,
    /// Displayed into the binding-error channel.
    Binding,
    /// Delivered to a typed error handler.
    Domain {
        ty: TypeRef,
        handler: String,
        capture: String,
        raw_iface: String,
        params: Vec<DParam>,
        leaves: Vec<Leaf>,
    },
}

/// One bound function, every decision made.
pub(crate) struct Binding {
    pub func: Function,
    pub callee: Callee,
    /// The Kotlin name.
    pub kt_name: String,
    /// The `external fun` name in the native holder.
    pub ext_name: String,
    pub placement: Placement,
    pub params: Vec<PPlan>,
    pub ret: RPlan,
    pub err: EPlan,
}

/// One element of the binding.
pub(crate) enum Item {
    Class {
        class: Rc<Class>,
        methods: Vec<Binding>,
        ctors: Vec<Binding>,
    },
    Function {
        pkg: String,
        binding: Binding,
    },
    Constant {
        pkg: String,
        name: String,
        binding: Binding,
    },
}

/// A Kotlin interface the bindings share, written once.
pub(crate) enum Support {
    /// A callback type's user-facing and raw interfaces.
    Callback(TypeRef),
    /// The interface Rust calls with a multi-leaf value, and the singleton
    /// that assembles it.
    Sink { ty: TypeRef, base: String },
    Builder {
        pkg: String,
        name: String,
        params: Vec<DParam>,
        leaves: Vec<Leaf>,
    },
    Folder {
        pkg: String,
        name: String,
        columns: String,
        params: Vec<DParam>,
        leaves: Vec<Leaf>,
    },
    ErrorHandler {
        pkg: String,
        name: String,
        raw: String,
        capture: String,
        params: Vec<DParam>,
        leaves: Vec<Leaf>,
    },
}

pub(crate) struct Plan<'f> {
    pub flat: &'f Flat,
    pub q: Qualifier<'f>,
    /// The base Kotlin package.
    pub base_pkg: String,
    /// The native-method holder's simple name.
    pub harness: String,
    pub native_init: Option<String>,
    pub handle_locks: bool,
    /// How each declared type crosses, by name.
    pub types: HashMap<String, Setting>,
    /// Each type's default output expansion.
    pub ret_exp: HashMap<String, ExpandReturnDecl>,
    /// Each type's default input expansion.
    pub param_exp: HashMap<String, ExpandParamDecl>,
    pub items: Vec<Item>,
    pub supports: Vec<Support>,
}

impl<'f> Plan<'f> {
    pub(crate) fn new(b: &JniGenBuilder, flat: &'f Flat) -> Res<Self> {
        prebindgen_tools::check_supported(flat).map_err(Error)?;
        let mut plan = Plan {
            flat,
            q: Qualifier::new(flat).with_default_module(b.source_module.clone()),
            base_pkg: b.package_prefix.clone(),
            harness: match &b.harness_hook {
                Some(f) => f("JNINative"),
                None => "JNINative".to_string(),
            },
            native_init: b.native_init.clone(),
            handle_locks: b.handle_locks,
            types: HashMap::new(),
            ret_exp: HashMap::new(),
            param_exp: HashMap::new(),
            items: Vec::new(),
            supports: Vec::new(),
        };
        plan.declare_types(b)?;
        plan.check_value_forms(b)?;
        let mut planner = Planner {
            plan: &plan,
            b,
            ext_names: HashSet::new(),
            supports: Vec::new(),
            seen: HashSet::new(),
            accessors: accessors(b),
        };
        let items = planner.items()?;
        let supports = planner.supports;
        plan.items = items;
        plan.supports = supports;
        plan.warn_unbound(b);
        Ok(plan)
    }

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

    /// The class a type is bound to.
    pub(crate) fn class(&self, name: &str) -> Option<&Rc<Class>> {
        match self.types.get(name) {
            Some(Setting::Class(c)) => Some(c),
            _ => None,
        }
    }

    fn declare_types(&mut self, b: &JniGenBuilder) -> Res<()> {
        for p in &b.packages {
            let pkg = self.pkg_fqn(&p.name);
            for c in &p.classes {
                let (ty, name, kind, iface) = match c {
                    ClassDecl::Ptr(d) => (&d.ty, &d.name, ClassKind::Ptr { gc: d.gc }, &d.iface),
                    ClassDecl::Data(d) => (&d.ty, &d.name, ClassKind::Data, &d.iface),
                    ClassDecl::Enum(d) => (&d.ty, &d.name, ClassKind::Enum, &d.iface),
                    ClassDecl::Sealed(d) => (
                        &d.ty,
                        &d.name,
                        ClassKind::Sealed {
                            renames: d
                                .variants
                                .iter()
                                .filter_map(|v| v.name.clone().map(|n| (v.rust.clone(), n)))
                                .collect(),
                        },
                        &d.iface,
                    ),
                };
                let rust = type_name(ty)?;
                let element = self
                    .flat
                    .declared_type(&rust)
                    .ok_or_else(|| Error(format!("`{rust}` is not a #[prebindgen] type")))?;
                let fits = matches!(
                    (&kind, element),
                    (ClassKind::Ptr { .. }, _)
                        | (ClassKind::Data, FlatType::Struct(_))
                        | (ClassKind::Enum, FlatType::Enum(_))
                        | (ClassKind::Sealed { .. }, FlatType::Variant(_))
                );
                if !fits {
                    return err(format!(
                        "`{rust}`: the class kind does not match the declared type"
                    ));
                }
                let hook = match kind {
                    ClassKind::Ptr { .. } => &b.ptr_hook,
                    ClassKind::Data | ClassKind::Sealed { .. } => &b.data_hook,
                    ClassKind::Enum => &b.enum_hook,
                };
                let name = match (name, hook) {
                    (Some(n), _) => n.clone(),
                    (None, Some(f)) => f(&pkg, &rust),
                    (None, None) => rust.clone(),
                };
                let iface_name = match (iface.enabled, &iface.name, &b.iface_hook) {
                    (false, _, _) => None,
                    (true, Some(n), _) => Some(n.clone()),
                    (true, None, Some(f)) => Some(f(&pkg, &name)),
                    (true, None, None) => Some(format!("{name}Api")),
                };
                if iface_name.as_deref() == Some(name.as_str()) {
                    return err(format!(
                        "`{rust}`: its interface cannot share the class name `{name}`"
                    ));
                }
                let class = Class {
                    rust: rust.clone(),
                    kind,
                    pkg: pkg.clone(),
                    name,
                    iface: iface_name,
                    implements: iface.implements.clone(),
                };
                if self
                    .types
                    .insert(rust.clone(), Setting::Class(Rc::new(class)))
                    .is_some()
                {
                    return err(format!("`{rust}` is declared as a class twice"));
                }
            }
        }
        for c in &b.converts {
            let resolved = c.conversion.resolve(self.flat).map_err(Error)?;
            let name = match resolved.target().kind() {
                TypeKind::Named { id, .. } => id.name.clone(),
                _ => return err(format!("convert!({}) must name a type", resolved.target())),
            };
            let conv = Conv {
                name: name.clone(),
                resolved,
                range: c.range,
            };
            if self
                .types
                .insert(name.clone(), Setting::Converted(Rc::new(conv)))
                .is_some()
            {
                return err(format!("`{name}` is declared twice"));
            }
        }
        for e in &b.expands {
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

    /// Every `fields!(f)` field override and rename names a field of the
    /// struct `f` returns, once.
    fn check_value_forms(&self, b: &JniGenBuilder) -> Res<()> {
        let mut decls: Vec<&ExpandReturnDecl> = b
            .expands
            .iter()
            .filter_map(|e| match e {
                ExpandDecl::Return(r) => Some(r),
                ExpandDecl::Param(_) => None,
            })
            .collect();
        for p in &b.packages {
            decls.extend(p.funs.iter().filter_map(|f| f.ret.as_ref()));
            for c in &p.classes {
                let (methods, ctors): (&[FunctionDecl], &[FunctionDecl]) = match c {
                    ClassDecl::Ptr(d) => (&d.methods, &d.constructors),
                    ClassDecl::Data(d) => (&d.methods, &d.constructors),
                    _ => (&[], &[]),
                };
                decls.extend(methods.iter().chain(ctors).filter_map(|f| f.ret.as_ref()));
            }
        }
        while let Some(d) = decls.pop() {
            for field in &d.fields {
                let ReturnField::Form { form, .. } = field else {
                    continue;
                };
                let fields: Vec<String> = match self.flat.function(&form.fun).map(|f| f.ret.kind())
                {
                    Some(TypeKind::Named { id, .. }) => match self.flat.declared_type(&id.name) {
                        Some(FlatType::Struct(s)) => {
                            s.fields.iter().map(crate::lower::field_seg).collect()
                        }
                        _ => continue,
                    },
                    _ => continue,
                };
                let keys = form
                    .overrides
                    .iter()
                    .map(|(n, _)| ("field", n))
                    .chain(form.names.iter().map(|(n, _)| ("name", n)));
                let mut seen = HashSet::new();
                for (what, n) in keys {
                    if !fields.contains(n) {
                        return err(format!(
                            "fields!({}).{what}(\"{n}\", ..): the struct has no field `{n}` (it has {})",
                            form.fun,
                            fields.join(", ")
                        ));
                    }
                    if !seen.insert((what, n)) {
                        return err(format!(
                            "fields!({}).{what}(\"{n}\", ..): declared twice",
                            form.fun
                        ));
                    }
                }
                decls.extend(form.overrides.iter().map(|(_, d)| d));
            }
        }
        Ok(())
    }

    /// Name the source functions no declaration binds.
    fn warn_unbound(&self, b: &JniGenBuilder) {
        let mut bound: HashSet<String> = HashSet::new();
        let mut add = |f: &FunctionDecl| {
            bound.insert(f.rust_name());
        };
        for p in &b.packages {
            p.funs.iter().for_each(&mut add);
            for c in &p.classes {
                if let ClassDecl::Ptr(d) = c {
                    d.methods.iter().chain(&d.constructors).for_each(&mut add);
                }
                if let ClassDecl::Data(d) = c {
                    d.methods.iter().chain(&d.constructors).for_each(&mut add);
                }
            }
            for c in &p.consts {
                if let ConstSource::Fun(f) = &c.source {
                    add(f);
                }
            }
        }
        for p in &b.packages {
            for f in &p.funs {
                for (_, d) in &f.params {
                    param_exp_fns(d, &mut bound);
                }
                if let Some(r) = &f.ret {
                    ret_exp_fns(r, &mut bound);
                }
            }
        }
        for e in &b.expands {
            match e {
                ExpandDecl::Param(p) => param_exp_fns(p, &mut bound),
                ExpandDecl::Return(r) => ret_exp_fns(r, &mut bound),
            }
        }
        for c in &b.converts {
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
            .filter(|n| !bound.contains(n) && !b.ignores.iter().any(|i| (i.0)(n)))
            .collect();
        unbound.sort();
        for n in unbound {
            println!("cargo:warning=prebindgen-jni: skipping undeclared function `{n}`");
        }
    }
}

/// The functions every output expansion — type-level or a function's own —
/// reads a field through.
fn accessors(b: &JniGenBuilder) -> HashSet<String> {
    let mut out = HashSet::new();
    for e in &b.expands {
        if let ExpandDecl::Return(r) = e {
            ret_exp_fns(r, &mut out);
        }
    }
    let decls = b.packages.iter().flat_map(|p| {
        let members = p.classes.iter().flat_map(|c| match c {
            ClassDecl::Ptr(d) => d.methods.iter().chain(&d.constructors).collect::<Vec<_>>(),
            ClassDecl::Data(d) => d.methods.iter().chain(&d.constructors).collect(),
            _ => Vec::new(),
        });
        p.funs.iter().chain(members)
    });
    for f in decls {
        if let Some(r) = &f.ret {
            ret_exp_fns(r, &mut out);
        }
    }
    out
}

fn param_exp_fns(d: &ExpandParamDecl, bound: &mut HashSet<String>) {
    for v in &d.variants {
        if let ParamVariant::Build(f) = v {
            bound.insert(f.rust_name());
        }
    }
}

fn ret_exp_fns(d: &ExpandReturnDecl, bound: &mut HashSet<String>) {
    for f in &d.fields {
        match f {
            ReturnField::Getter(g) => {
                bound.insert(g.rust_name());
            }
            ReturnField::Form { form, .. } => {
                bound.insert(form.fun.to_string());
                for (_, d) in &form.overrides {
                    ret_exp_fns(d, bound);
                }
            }
            ReturnField::Handle => {}
        }
    }
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
        _ => err(format!("`{}` must name a type", ty.to_token_stream())),
    }
}

// ── planning the items ──────────────────────────────────────────────────

/// Plans the items against the settings.
struct Planner<'p, 'f> {
    plan: &'p Plan<'f>,
    b: &'p JniGenBuilder,
    /// Extern names taken so far.
    ext_names: HashSet<String>,
    supports: Vec<Support>,
    /// Support interfaces planned so far, by name.
    seen: HashSet<String>,
    /// Functions an output expansion reads a field through.
    accessors: HashSet<String>,
}

impl Planner<'_, '_> {
    fn items(&mut self) -> Res<Vec<Item>> {
        let mut items = Vec::new();
        for p in &self.b.packages {
            let pkg = self.plan.pkg_fqn(&p.name);
            for c in &p.classes {
                let rust = type_name(match c {
                    ClassDecl::Ptr(d) => &d.ty,
                    ClassDecl::Data(d) => &d.ty,
                    ClassDecl::Enum(d) => &d.ty,
                    ClassDecl::Sealed(d) => &d.ty,
                })?;
                let class = self.plan.class(&rust).expect("a declared class").clone();
                let (methods, ctors): (&[FunctionDecl], &[FunctionDecl]) = match c {
                    ClassDecl::Ptr(d) => (&d.methods, &d.constructors),
                    ClassDecl::Data(d) => (&d.methods, &d.constructors),
                    _ => (&[], &[]),
                };
                let mut bind_all = |fs: &[FunctionDecl], ctor: bool| -> Res<Vec<Binding>> {
                    fs.iter()
                        .map(|f| {
                            let placement = if ctor {
                                Placement::Constructor(class.clone())
                            } else {
                                Placement::Method(class.clone())
                            };
                            let (func, callee) = self.resolve_fn(&f.fun)?;
                            let camel = names::camel(&names::bare(&func.name));
                            let kt_name = match (&f.name, &self.b.method_hook) {
                                (Some(n), _) => n.clone(),
                                (None, Some(h)) => h(&class.pkg, &class.name, &camel),
                                (None, None) => camel,
                            };
                            self.bind(f, func, Callee::Call(callee), placement, kt_name)
                        })
                        .collect()
                };
                let methods = bind_all(methods, false)?;
                let ctors = bind_all(ctors, true)?;
                items.push(Item::Class {
                    class,
                    methods,
                    ctors,
                });
            }
            for f in &p.funs {
                let (func, callee) = self.resolve_fn(&f.fun)?;
                let camel = names::camel(&names::bare(&func.name));
                let kt_name = match (&f.name, &self.b.fun_hook) {
                    (Some(n), _) => n.clone(),
                    (None, Some(h)) => h(&pkg, &camel),
                    (None, None) => camel,
                };
                let binding =
                    self.bind(f, func, Callee::Call(callee), Placement::Package, kt_name)?;
                items.push(Item::Function {
                    pkg: pkg.clone(),
                    binding,
                });
            }
            for c in &p.consts {
                items.push(self.constant(&pkg, c)?);
            }
        }
        Ok(items)
    }

    fn resolve_fn(&self, f: &FnRef) -> Res<(Function, TokenStream)> {
        let func = f.resolve(self.plan.flat).map_err(Error)?;
        Ok((func, f.callee(&self.plan.q)))
    }

    /// A top-level `val`: a private getter extern and the `val` reading it.
    fn constant(&mut self, pkg: &str, c: &ConstDecl) -> Res<Item> {
        let plan = self.plan;
        let vname = names::bare(&c.name);
        let getter = format!("constGet{}", names::pascal(&vname.to_lowercase()));
        let classify = |ty: &syn::Type| {
            plan.flat
                .classify(ty)
                .map_err(|e| Error(format!("constant `{vname}`: {e}")))
        };
        let synthetic = |t: TypeRef| Function::synthetic_getter(names::ident(&getter), t);
        let (func, value, kt_name) = match &c.source {
            ConstSource::Const => {
                let k = plan
                    .flat
                    .constant(&c.name)
                    .ok_or_else(|| Error(format!("`{vname}` is not a #[prebindgen] const")))?;
                let path = plan.q.path(&c.name);
                (synthetic(k.ty.clone()), quote!(#path), getter.clone())
            }
            ConstSource::Fun(fd) => {
                let (f, callee) = self.resolve_fn(&fd.fun)?;
                let n = names::camel(&names::bare(&f.name));
                (f, quote!(#callee()), n)
            }
            ConstSource::With(ty, path) => {
                (synthetic(classify(ty)?), quote!(#path()), getter.clone())
            }
            ConstSource::Expr(ty, expr) => {
                // The expression sees every source crate's items.
                let mut modules: Vec<String> = plan
                    .flat
                    .elements()
                    .filter_map(|e| e.location().crate_name.clone())
                    .map(|c| c.replace('-', "_"))
                    .collect();
                modules.sort();
                modules.dedup();
                let uses = modules.iter().map(|m| {
                    let m: syn::Path = syn::parse_str(m).expect("a crate name");
                    quote!(#[allow(unused_imports)] use #m::*;)
                });
                (
                    synthetic(classify(ty)?),
                    quote!({ #(#uses)* #expr }),
                    getter.clone(),
                )
            }
        };
        let decl = FunctionDecl::new(syn::parse_quote!(__const));
        let binding = self.bind(
            &decl,
            func,
            Callee::Value(value),
            Placement::Package,
            kt_name,
        )?;
        Ok(Item::Constant {
            pkg: pkg.to_string(),
            name: c.val_name.clone().unwrap_or(vname),
            binding,
        })
    }

    /// Decide everything about one bound function.
    fn bind(
        &mut self,
        decl: &FunctionDecl,
        func: Function,
        callee: Callee,
        placement: Placement,
        kt_name: String,
    ) -> Res<Binding> {
        let camel = names::camel(&names::bare(&func.name));
        let base = match &self.b.method_hook {
            Some(h) => h(&self.plan.base_pkg, &self.plan.harness, &camel),
            None => camel,
        };
        let mut ext_name = base.clone();
        let mut i = 2;
        while !self.ext_names.insert(ext_name.clone()) {
            ext_name = format!("{base}{i}");
            i += 1;
        }
        let params = self.params(decl, &func, &placement)?;
        let (ok, err_ty) = match func.ret.kind() {
            TypeKind::Fallible { ok, err } => ((**ok).clone(), Some((**err).clone())),
            _ => (func.ret.clone(), None),
        };
        let ret = self.ret(decl, &func, &placement, &ok, err_ty.is_some())?;
        let err = match err_ty {
            None => EPlan::None,
            Some(e) => self.err(&e)?,
        };
        Ok(Binding {
            func,
            callee,
            kt_name,
            ext_name,
            placement,
            params,
            ret,
            err,
        })
    }

    fn params(
        &mut self,
        decl: &FunctionDecl,
        f: &Function,
        placement: &Placement,
    ) -> Res<Vec<PPlan>> {
        let plan = self.plan;
        let receiver = matches!(placement, Placement::Method(_));
        if receiver && f.params.is_empty() {
            return err(format!("method `{}` has no receiver parameter", f.name));
        }
        let mut params = Vec::new();
        for (i, p) in f.params.iter().enumerate() {
            if receiver && i == 0 {
                params.push(PPlan::Receiver(p.clone()));
                continue;
            }
            let pname = names::bare(&p.name);
            let explicit = decl
                .params
                .iter()
                .find(|(n, _)| *n == pname)
                .map(|(_, d)| d);
            match plan.selector(p, explicit)? {
                Some(s) => {
                    let split = decl.splits.contains(&pname) && !s.is_direct();
                    params.push(PPlan::Selector(s, split));
                }
                None => {
                    if let TypeKind::Callback { args } = p.ty.kind() {
                        let leaves: Vec<Leaf> = plan
                            .callback_args(args)?
                            .into_iter()
                            .flat_map(|d| d.leaves)
                            .collect();
                        check_slots(&format!("the callback `{}`", p.ty), &leaves, 1)?;
                        self.support(plan.callback_fqn(&p.ty)?, || {
                            Ok(Support::Callback(p.ty.clone()))
                        })?;
                    }
                    params.push(PPlan::Value(p.clone()));
                }
            }
        }
        for s in &decl.splits {
            let named = params.iter().any(
                |p| matches!(p, PPlan::Selector(sel, _) if names::bare(&sel.param.name) == *s),
            );
            if !named {
                return err(format!(
                    "`{}`: `.split_on_param(\"{s}\")` names no selector parameter",
                    f.name
                ));
            }
        }
        // The receiver and up to three sinks join the parameters' leaves.
        let mut leaves = Vec::new();
        for p in &params {
            leaves.extend(plan.param_leaves(p)?);
        }
        check_slots(
            &format!("the native method of `{}`", f.name),
            &leaves,
            1 + 3,
        )?;
        Ok(params)
    }

    /// How the success value leaves. A type's own output expansion applies
    /// to a function returning the type, except to a function returning
    /// `Result` (a fallible factory keeps its handle) and to an accessor
    /// some expansion reads a field through (it is the field).
    fn ret(
        &mut self,
        decl: &FunctionDecl,
        f: &Function,
        placement: &Placement,
        ty: &TypeRef,
        fallible: bool,
    ) -> Res<RPlan> {
        let plan = self.plan;
        if let TypeKind::Unit = ty.kind() {
            return Ok(RPlan::Unit);
        }
        let explicit = decl.ret.as_ref();
        // A constructor returns its own class as itself, never expanded.
        if let Placement::Constructor(c) = placement {
            let own = matches!(ty.kind(), TypeKind::Named { id, .. } if id.name == c.rust);
            if explicit.is_none() && own {
                let leaf = plan.leaves(ty, Dir::Out)?.remove(0);
                return Ok(RPlan::Direct {
                    ty: ty.clone(),
                    leaf,
                });
            }
        }
        // The layers a builder or folder absorbs: `Box`, `Option`, `Vec`.
        let mut core = ty;
        let mut optional = false;
        let mut seq = false;
        loop {
            match core.kind() {
                TypeKind::Boxed(t) => core = t,
                TypeKind::Optional(t) if !optional && !seq => {
                    optional = true;
                    core = t
                }
                TypeKind::Vec(t) if !seq => {
                    seq = true;
                    core = t
                }
                _ => break,
            }
        }
        let bare = core.borrow_target().unwrap_or(core);
        let expansion = match explicit {
            None if fallible || self.accessors.contains(&names::bare(&f.name)) => None,
            _ => plan.expansion(bare, explicit),
        };
        if let Some(e) = expansion.cloned() {
            let TypeKind::Named { id, .. } = bare.kind() else {
                return err(format!("`{ty}`: an output expansion needs a named type"));
            };
            let tname = id.name.clone();
            // A function's own expansion gets its own interface.
            let suffix = if explicit.is_some() {
                names::pascal(&names::bare(&f.name))
            } else {
                String::new()
            };
            let d = plan.deliver(core, quote!(__x), "r", "", Some(&e), &[], false, 1)?;
            // The interfaces live beside the type's class.
            let base = &plan
                .class(&tname)
                .map_or(plan.base_pkg.clone(), |c| c.pkg.clone());
            if seq {
                // The folder's columns upcall: a count, then one array per leaf.
                let mut columns = vec![Leaf::new(LeafTy::Prim(Prim::I))];
                columns.extend(d.leaves.iter().map(Leaf::column));
                check_slots(&format!("the folder of `{ty}`"), &columns, 1)?;
                let iface = format!("{base}.{tname}{suffix}Folder");
                let columns_iface = format!("{base}.{tname}{suffix}FolderColumns");
                self.support(iface.clone(), || {
                    Ok(Support::Folder {
                        pkg: base.clone(),
                        name: simple(&iface),
                        columns: simple(&columns_iface),
                        params: d.params.clone(),
                        leaves: d.leaves.clone(),
                    })
                })?;
                return Ok(RPlan::Fold {
                    whole: ty.clone(),
                    exp: e,
                    optional,
                    iface,
                    columns_iface,
                    params: d.params,
                    leaves: d.leaves,
                });
            }
            check_slots(&format!("the builder of `{ty}`"), &d.leaves, 1)?;
            let iface = format!("{base}.{tname}{suffix}Builder");
            self.support(iface.clone(), || {
                Ok(Support::Builder {
                    pkg: base.clone(),
                    name: simple(&iface),
                    params: d.params.clone(),
                    leaves: d.leaves.clone(),
                })
            })?;
            return Ok(RPlan::Builder {
                whole: ty.clone(),
                exp: e,
                optional,
                iface,
                leaves: d.leaves,
            });
        }
        // A typed value: one leaf returns directly, several go to a sink.
        let mut leaves = plan.leaves(ty, Dir::Out)?;
        if leaves.len() == 1 {
            return Ok(RPlan::Direct {
                ty: ty.clone(),
                leaf: leaves.remove(0),
            });
        }
        check_slots(&format!("the result sink of `{ty}`"), &leaves, 1)?;
        let base = names::mangle(ty);
        let iface = format!("{}.__Sink_{base}", plan.base_pkg);
        let kt_sink = format!("{}.__sink_{base}", plan.base_pkg);
        self.support(iface.clone(), || {
            Ok(Support::Sink {
                ty: ty.clone(),
                base: base.clone(),
            })
        })?;
        Ok(RPlan::Sink {
            ty: ty.clone(),
            leaves,
            iface,
            kt_sink,
        })
    }

    fn err(&mut self, e: &TypeRef) -> Res<EPlan> {
        let plan = self.plan;
        let TypeKind::Named { id, .. } = e.kind() else {
            return Ok(EPlan::Binding);
        };
        if plan.expansion(e, None).is_none() {
            return Ok(EPlan::Binding);
        }
        // An error type with no class is Rust-side only: its handler lives
        // in the base package.
        let pkg = match plan.class(&id.name) {
            Some(class) => class.pkg.clone(),
            None => plan.base_pkg.clone(),
        };
        let handler = format!("{pkg}.{}Handler", id.name);
        let raw_iface = format!("{handler}Raw");
        let capture = format!("{handler}Capture");
        let d = plan.deliver(e, quote!(__e), "e", "", None, &[], false, 1)?;
        check_slots(&format!("the error handler of `{e}`"), &d.leaves, 1)?;
        self.support(handler.clone(), || {
            Ok(Support::ErrorHandler {
                pkg: pkg.clone(),
                name: simple(&handler),
                raw: simple(&raw_iface),
                capture: simple(&capture),
                params: d.params.clone(),
                leaves: d.leaves.clone(),
            })
        })?;
        Ok(EPlan::Domain {
            ty: e.clone(),
            handler,
            capture,
            raw_iface,
            params: d.params,
            leaves: d.leaves,
        })
    }

    /// Plan the support interface `fqn` unless it already is.
    fn support(&mut self, fqn: String, make: impl FnOnce() -> Res<Support>) -> Res<()> {
        if self.seen.insert(fqn) {
            self.supports.push(make()?);
        }
        Ok(())
    }
}

fn simple(fqn: &str) -> String {
    fqn.rsplit('.').next().unwrap_or(fqn).to_string()
}

impl Plan<'_> {
    /// The raw leaves of the extern for one parameter.
    pub(crate) fn param_leaves(&self, p: &PPlan) -> Res<Vec<Leaf>> {
        match p {
            PPlan::Receiver(fp) | PPlan::Value(fp) => Ok(self
                .leaves(&fp.ty, Dir::In)?
                .into_iter()
                .map(|l| l.under(&names::bare(&fp.name)))
                .collect()),
            PPlan::Selector(s, _) => self.selector_leaves(s),
        }
    }
}

/// Refuse a JVM method whose arguments would take more than 255 slots, the
/// JVM's limit (JVMS §4.3.3): a `long` or `double` takes two, any other
/// argument one. `extra` counts the slots beyond `leaves` — the receiver,
/// and for a native method the sinks.
fn check_slots(what: &str, leaves: &[Leaf], extra: usize) -> Res<()> {
    let slots = extra + leaves.iter().map(Leaf::slots).sum::<usize>();
    if slots > 255 {
        return err(format!(
            "{what} would take {slots} JVM argument slots, more than the JVM's limit of 255: \
             split the value into smaller parameters or fields"
        ));
    }
    Ok(())
}
