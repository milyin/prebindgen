//! The plan: the build script's declarations resolved against the flat
//! model — the setting of every declared type, and the list of elements to
//! write: the declared types, then the callbacks' closure structs, then the
//! functions, each kind in declaration order.
//!
//! Everything a declaration can get wrong is found here: a missing item, a
//! kind that does not match the source, a conversion that does not fit.
//! [`write`](crate::write) then writes the elements one by one.

use std::collections::{HashMap, HashSet};

use prebindgen_flat::{
    flat::{Function, Type as FlatType, TypeKind, TypeRef},
    Flat,
};
use prebindgen_tools::{names, Choices, In, Out, Place, ResolvedConversion, Ways};
use quote::ToTokens;

use crate::{
    builder::{CbindgenBuilder, TypeKind as DeclKind},
    Error,
};

pub(crate) type Res<T> = Result<T, Error>;

pub(crate) fn err<T>(msg: impl Into<String>) -> Res<T> {
    Err(Error(msg.into()))
}

/// How a declared type crosses.
#[allow(clippy::large_enum_variant)] // one per declared type
pub(crate) enum Setting {
    /// A type with a C spelling of its own.
    Type(CType),
    /// A type crossing as its representation.
    Converted(ResolvedConversion),
}

/// A declared type's C spelling.
pub(crate) struct CType {
    pub kind: Kind,
    /// The source type's name.
    pub rust: syn::Ident,
    /// The mirror or opaque struct C sees: `calculator_t`.
    pub c: syn::Ident,
    /// Its destructor: `calculator_drop`.
    pub drop: syn::Ident,
}

#[derive(Clone)]
pub(crate) enum Kind {
    Opaque,
    Error { message: syn::Ident },
    Enum,
    Union,
    Data,
    ReprC { assume_valid: bool },
}

/// One element of the binding.
#[allow(clippy::large_enum_variant)] // one per element
pub(crate) enum Item {
    /// A declared type with a C spelling, by name.
    Type(String),
    /// A callback's closure struct.
    Closure { ty: TypeRef, name: syn::Ident },
    /// An exported function.
    Function {
        func: Function,
        exported: syn::Ident,
        panic: bool,
    },
}

pub(crate) struct Plan<'f> {
    pub flat: &'f Flat,
    pub free_fn: Option<syn::Ident>,
    /// The setting of each declared type, by name.
    pub types: HashMap<String, Setting>,
    /// The ways each declared type crosses: whole, as its C spelling; as its
    /// representation, when converted; and a data struct also as its
    /// fields, which its mirror takes.
    pub ways: Ways<'f, ()>,
    pub inputs: Choices<In>,
    pub outputs: Choices<Out>,
    /// Each callback type's closure struct, by the type's key.
    pub closures: HashMap<String, syn::Ident>,
    pub items: Vec<Item>,
}

impl<'f> Plan<'f> {
    pub(crate) fn new(b: &CbindgenBuilder, flat: &'f Flat) -> Res<Self> {
        prebindgen_tools::check_supported(flat).map_err(Error)?;
        let mut plan = Plan {
            flat,
            free_fn: b.free_fn.as_deref().map(names::ident),
            types: HashMap::new(),
            ways: Ways::new(),
            inputs: Choices::new(),
            outputs: Choices::new(),
            closures: HashMap::new(),
            items: Vec::new(),
        };
        plan.declare_types(b)?;
        for c in &b.conversions {
            let r = c.resolve(flat).map_err(Error)?;
            let name = named(r.target())
                .ok_or_else(|| Error(format!("convert!({}): not a named type", r.target())))?;
            match (r.has_input(), r.has_output()) {
                (true, true) => plan.ways.converted(r.clone()).map(drop),
                (true, false) => plan.ways.converted_in(r.clone()).map(drop),
                _ => plan.ways.converted_out(r.clone()).map(drop),
            }
            .map_err(Error)?;
            if plan
                .types
                .insert(name.clone(), Setting::Converted(r))
                .is_some()
            {
                return err(format!("`{name}` is declared twice"));
            }
        }
        for cb in &b.callbacks {
            let ty = classify(flat, &cb.ty)?;
            plan.plan_closure(&ty, cb.base.clone(), b)?;
        }
        for f in &b.functions {
            let Some(func) = flat.function(&f.name) else {
                return err(format!("`{}` is not a #[prebindgen] function", f.name));
            };
            // A callback parameter nobody declared gets a closure struct
            // named after its arguments.
            for p in &func.params {
                if let TypeKind::Callback { .. } = p.ty.kind() {
                    plan.plan_closure(&p.ty, None, b)?;
                }
            }
            let exported = match &b.mangle_function {
                Some(m) => m(&f.name.to_string()),
                None => f.name.to_string(),
            };
            plan.items.push(Item::Function {
                func: func.clone(),
                exported: names::ident(&exported),
                panic: f.panic,
            });
        }
        plan.warn_undeclared(b);
        Ok(plan)
    }

    fn declare_types(&mut self, b: &CbindgenBuilder) -> Res<()> {
        for d in &b.types {
            let ty = classify(self.flat, &d.ty)?;
            let Some(name) = named(&ty) else {
                return err(format!("`{}` is not a named type", d.ty.to_token_stream()));
            };
            let element = self.flat.declared_type(&name);
            let fits = match (&d.kind, element) {
                (DeclKind::Opaque | DeclKind::OpaqueError { .. }, _) => {
                    name == "String" || element.is_some()
                }
                (DeclKind::Enum, Some(FlatType::Enum(_))) => true,
                (DeclKind::Union, Some(FlatType::Variant(_))) => true,
                (DeclKind::Data | DeclKind::ReprC { .. }, Some(FlatType::Struct(_))) => true,
                _ => false,
            };
            if !fits {
                let what = match d.kind {
                    DeclKind::Enum => "a fieldless enum",
                    DeclKind::Union => "a data-carrying enum",
                    DeclKind::Data | DeclKind::ReprC { .. } => "a struct",
                    _ => "a #[prebindgen] type",
                };
                return err(format!("`{name}` is not {what}"));
            }
            let base = d.base.clone().unwrap_or_else(|| names::snake(&name));
            let c = match &b.mangle_type {
                Some(f) => f(&base),
                None => base.clone(),
            };
            let drop = match &b.mangle_destructor {
                Some(f) => f(&base),
                None => format!("{base}_drop"),
            };
            let kind = match &d.kind {
                DeclKind::Opaque => Kind::Opaque,
                DeclKind::OpaqueError { message } => Kind::Error {
                    message: message.clone(),
                },
                DeclKind::Enum => Kind::Enum,
                DeclKind::Union => Kind::Union,
                DeclKind::Data => Kind::Data,
                DeclKind::ReprC { assume_valid } => Kind::ReprC {
                    assume_valid: *assume_valid,
                },
            };
            let setting = Setting::Type(CType {
                kind,
                rust: names::ident(&name),
                c: names::ident(&c),
                drop: names::ident(&drop),
            });
            if self.types.insert(name.clone(), setting).is_some() {
                return err(format!("`{name}` is declared twice"));
            }
            let whole = self.ways.whole(&ty, ());
            self.inputs.choose(&self.ways, whole).map_err(Error)?;
            self.outputs.choose(&self.ways, whole).map_err(Error)?;
            if let (DeclKind::Data, Some(FlatType::Struct(s))) = (&d.kind, element) {
                // Everywhere whole, except in its own mirror.
                let fields = self.ways.fields(s);
                let mirror = mirror_place(&names::ident(&c));
                self.inputs.choose_at(mirror.clone(), fields);
                self.outputs.choose_at(mirror, fields);
            }
            self.items.push(Item::Type(name));
        }
        Ok(())
    }

    /// Plan the closure struct of a callback type, once.
    fn plan_closure(&mut self, ty: &TypeRef, base: Option<String>, b: &CbindgenBuilder) -> Res<()> {
        let TypeKind::Callback { args } = ty.kind() else {
            return err(format!("`{ty}` is not an `impl Fn(..)` callback"));
        };
        let key = ty.key().as_str().to_string();
        if self.closures.contains_key(&key) {
            return Ok(());
        }
        let bases: Vec<String> = match base {
            Some(b) => vec![b],
            None => args
                .iter()
                .map(|a| names::snake(&names::mangle(a)))
                .collect(),
        };
        let name = names::ident(&match &b.mangle_callback {
            Some(f) => f(&bases),
            None => format!("closure_{}", bases.join("_")),
        });
        self.closures.insert(key, name.clone());
        self.items.push(Item::Closure {
            ty: ty.clone(),
            name,
        });
        Ok(())
    }

    /// Name the source functions no declaration binds.
    fn warn_undeclared(&self, b: &CbindgenBuilder) {
        let mut bound: HashSet<String> = b.functions.iter().map(|f| names::bare(&f.name)).collect();
        bound.extend(b.ignored.iter().map(names::bare));
        for s in self.types.values() {
            match s {
                Setting::Type(CType {
                    kind: Kind::Error { message },
                    ..
                }) => {
                    bound.insert(names::bare(message));
                }
                Setting::Converted(r) => {
                    for f in r.functions() {
                        bound.insert(names::bare(f.name()));
                    }
                }
                _ => {}
            }
        }
        let mut unbound: Vec<String> = self
            .flat
            .functions()
            .map(|f| names::bare(f.name.ident()))
            .filter(|n| !bound.contains(n))
            .collect();
        unbound.sort();
        for n in unbound {
            println!("cargo:warning=prebindgen-c: skipping undeclared function `{n}`");
        }
    }

    /// The setting of a type, if it is declared.
    pub(crate) fn setting(&self, name: &str) -> Option<&Setting> {
        self.types.get(name)
    }

    /// The C spelling of a declared type.
    pub(crate) fn ctype(&self, name: &str) -> Option<&CType> {
        match self.types.get(name) {
            Some(Setting::Type(t)) => Some(t),
            _ => None,
        }
    }

    /// The source element of a declared type.
    pub(crate) fn element(&self, name: &str) -> Res<&'f FlatType> {
        self.flat
            .declared_type(name)
            .ok_or_else(|| Error(format!("`{name}` is not a #[prebindgen] type")))
    }
}

/// Where a declared type's mirror `c` converts it: the one place a data
/// struct crosses as its fields.
pub(crate) fn mirror_place(c: &syn::Ident) -> Place {
    Place::new(names::bare(c))
}

fn classify(flat: &Flat, ty: &syn::Type) -> Res<TypeRef> {
    flat.classify(ty)
        .map_err(|e| Error(format!("`{}`: {e}", ty.to_token_stream())))
}

/// The name a type is declared under: `Payload`, or `String`.
fn named(ty: &TypeRef) -> Option<String> {
    match ty.kind() {
        TypeKind::Named { id, .. } => Some(id.name.clone()),
        TypeKind::String => Some("String".to_string()),
        _ => None,
    }
}

/// The name of a type the C adapter can declare, including a borrowed use.
pub(crate) fn declared_name(ty: &TypeRef) -> &str {
    let core = match ty.kind() {
        TypeKind::Ref { inner, .. } => inner.as_ref(),
        _ => ty,
    };
    match core.kind() {
        TypeKind::Named { id, .. } => &id.name,
        TypeKind::String => "String",
        _ => unreachable!("C declarations name only source types or String"),
    }
}
