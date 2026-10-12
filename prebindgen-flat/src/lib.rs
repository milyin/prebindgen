//! The small flat source model. Types retain Rust syntax and an occurrence slot.
use prebindgen::SourceLocation;
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};
use syn::{visit_mut::VisitMut, Type};

mod primitive;
pub use primitive::Primitive;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug)]
pub struct TypeUse<S> {
    pub ty: Type,
    pub slot: S,
}
#[derive(Clone, Debug)]
pub struct Param<S> {
    pub name: syn::Ident,
    pub ty: TypeUse<S>,
}
#[derive(Clone, Debug)]
pub struct Function<S> {
    pub name: syn::Ident,
    pub path: syn::Path,
    pub params: Vec<Param<S>>,
    pub ret: TypeUse<S>,
    pub location: SourceLocation,
}
#[derive(Clone, Debug)]
pub struct Record<S> {
    pub ty: Type,
    pub fields: Vec<Param<S>>,
}
#[derive(Clone, Debug)]
pub struct Flat<S> {
    pub functions: BTreeMap<String, Function<S>>,
    pub records: BTreeMap<String, Record<S>>,
    pub types: BTreeMap<String, Type>,
    pub guards: Vec<syn::ItemConst>,
}

impl<S: Default> Flat<S> {
    /// Select the source declarations needed by this binding before validating
    /// the supported flat subset. Unbound source items are not language features.
    pub fn parse_selected(
        items: impl IntoIterator<Item = (syn::Item, SourceLocation)>,
        functions: &[String],
        types: &[Type],
    ) -> Result<Self> {
        let items: Vec<_> = items.into_iter().collect();
        fn name(item: &syn::Item) -> Option<String> {
            match item {
                syn::Item::Fn(i) => Some(i.sig.ident.to_string()),
                syn::Item::Struct(i) => Some(i.ident.to_string()),
                syn::Item::Enum(i) => Some(i.ident.to_string()),
                syn::Item::Type(i) => Some(i.ident.to_string()),
                _ => None,
            }
        }
        struct References<'a>(&'a mut BTreeSet<String>);
        impl VisitMut for References<'_> {
            fn visit_type_path_mut(&mut self, path: &mut syn::TypePath) {
                if path.qself.is_none() && path.path.segments.len() == 1 {
                    self.0.insert(path.path.segments[0].ident.to_string());
                }
                syn::visit_mut::visit_type_path_mut(self, path);
            }
        }
        let mut needed: BTreeSet<_> = functions.iter().cloned().collect();
        for ty in types {
            References(&mut needed).visit_type_mut(&mut ty.clone());
        }
        loop {
            let old = needed.len();
            for (item, _) in &items {
                if !name(item).is_some_and(|n| needed.contains(&n)) {
                    continue;
                }
                let mut visitor = References(&mut needed);
                match item {
                    syn::Item::Fn(f) => visitor.visit_signature_mut(&mut f.sig.clone()),
                    syn::Item::Struct(s) => visitor.visit_fields_mut(&mut s.fields.clone()),
                    // Aliases are opaque: their implementation is not part of the boundary.
                    _ => {}
                }
            }
            if needed.len() == old {
                break;
            }
        }
        Self::parse(items.into_iter().filter(|(item, _)| {
            matches!(item, syn::Item::Const(c) if c.ident == "_")
                || name(item).is_some_and(|n| needed.contains(&n))
        }))
    }

    /// Read captured items; only free functions, named records and opaque aliases
    /// belong to this first subset. Unknown forms fail here, not during emission.
    pub fn parse(items: impl IntoIterator<Item = (syn::Item, SourceLocation)>) -> Result<Self> {
        let items: Vec<_> = items.into_iter().collect();
        let mut names = BTreeSet::new();
        let mut types = BTreeMap::new();
        for (item, loc) in &items {
            let visibility = match item {
                syn::Item::Fn(f) => Some(&f.vis),
                syn::Item::Struct(s) => Some(&s.vis),
                syn::Item::Type(t) => Some(&t.vis),
                _ => None,
            };
            if visibility.is_some_and(|v| !matches!(v, syn::Visibility::Public(_))) {
                return Err("flat declarations must be public".into());
            }
            let id = match item {
                syn::Item::Fn(f) => &f.sig.ident,
                syn::Item::Struct(s) => &s.ident,
                syn::Item::Type(t) => &t.ident,
                syn::Item::Const(c) if c.ident == "_" => continue,
                _ => return Err(format!("unsupported flat item: {}", item.to_token_stream())),
            };
            if !names.insert(id.to_string()) {
                return Err(format!("duplicate flat name `{id}`"));
            }
            if !matches!(item, syn::Item::Fn(_)) {
                let module: syn::Path = syn::parse_str(
                    &loc.crate_name
                        .as_deref()
                        .unwrap_or("crate")
                        .replace('-', "_"),
                )
                .map_err(|e| e.to_string())?;
                types.insert(id.to_string(), syn::parse_quote!(#module::#id));
            }
        }
        let mut flat = Self {
            functions: BTreeMap::new(),
            records: BTreeMap::new(),
            types,
            guards: Vec::new(),
        };
        for (item, location) in items {
            match item {
                syn::Item::Fn(f) => {
                    if !f.sig.generics.params.is_empty()
                        || f.sig.generics.where_clause.is_some()
                        || f.sig.asyncness.is_some()
                        || f.sig.unsafety.is_some()
                        || f.sig.variadic.is_some()
                        || f.sig.abi.is_some()
                    {
                        return Err(format!(
                            "`{}`: only safe, synchronous, nongeneric free functions are supported",
                            f.sig.ident
                        ));
                    }
                    let module: syn::Path = syn::parse_str(
                        &location
                            .crate_name
                            .as_deref()
                            .unwrap_or("crate")
                            .replace('-', "_"),
                    )
                    .map_err(|e| e.to_string())?;
                    let name = f.sig.ident;
                    let mut params = Vec::new();
                    for arg in f.sig.inputs {
                        let syn::FnArg::Typed(p) = arg else {
                            return Err("receivers are not flat parameters".into());
                        };
                        let syn::Pat::Ident(id) = *p.pat else {
                            return Err("a parameter must have a name".into());
                        };
                        if id.subpat.is_some() || id.by_ref.is_some() {
                            return Err("parameter patterns are unsupported".into());
                        }
                        params.push(Param {
                            name: id.ident,
                            ty: flat.type_use(*p.ty)?,
                        });
                    }
                    let ret = match f.sig.output {
                        syn::ReturnType::Default => syn::parse_quote!(()),
                        syn::ReturnType::Type(_, t) => *t,
                    };
                    let ret = flat.type_use(ret)?;
                    flat.functions.insert(
                        name.to_string(),
                        Function {
                            path: syn::parse_quote!(#module::#name),
                            name,
                            params,
                            ret,
                            location,
                        },
                    );
                }
                syn::Item::Struct(s) => {
                    if !s.generics.params.is_empty() || s.generics.where_clause.is_some() {
                        return Err("generic records are unsupported".into());
                    }
                    let syn::Fields::Named(fields) = s.fields else {
                        return Err(
                            "only named records are supported; use an alias for an opaque type"
                                .into(),
                        );
                    };
                    let ty = flat.types[&s.ident.to_string()].clone();
                    let mut out = Vec::new();
                    for field in fields.named {
                        if !matches!(field.vis, syn::Visibility::Public(_)) {
                            return Err("record fields must be public".into());
                        }
                        out.push(Param {
                            name: field.ident.unwrap(),
                            ty: flat.type_use(field.ty)?,
                        });
                    }
                    flat.records
                        .insert(type_key(&ty), Record { ty, fields: out });
                }
                syn::Item::Type(t) => {
                    if !t.generics.params.is_empty() || t.generics.where_clause.is_some() {
                        return Err("generic aliases are unsupported".into());
                    }
                    // The alias's contents remain opaque, as in the flat source contract.
                }
                syn::Item::Const(c) => flat.guards.push(c),
                _ => unreachable!(),
            }
        }
        Ok(flat)
    }
    fn type_use(&self, ty: Type) -> Result<TypeUse<S>> {
        Ok(TypeUse {
            ty: self.resolve_type(&ty)?,
            slot: S::default(),
        })
    }
}

impl<S> Flat<S> {
    pub fn function(&self, name: &str) -> Result<&Function<S>> {
        self.functions
            .get(name)
            .ok_or_else(|| format!("unknown function `{name}`"))
    }
    pub fn function_mut(&mut self, name: &str) -> Result<&mut Function<S>> {
        self.functions
            .get_mut(name)
            .ok_or_else(|| format!("unknown function `{name}`"))
    }
    pub fn record(&self, ty: &Type) -> Result<&Record<S>> {
        self.records
            .get(&type_key(ty))
            .ok_or_else(|| format!("`{}` is not a record", type_key(ty)))
    }
    /// Qualify declared names, and validate just the supported grammar.
    pub fn resolve_type(&self, ty: &Type) -> Result<Type> {
        let mut ty = ty.clone();
        struct Names<'a>(&'a BTreeMap<String, Type>);
        impl VisitMut for Names<'_> {
            fn visit_type_mut(&mut self, ty: &mut Type) {
                if let Type::Path(p) = ty {
                    if p.qself.is_none() && p.path.segments.len() == 1 {
                        let seg = &p.path.segments[0];
                        if matches!(seg.arguments, syn::PathArguments::None) {
                            if let Some(named) = self.0.get(&seg.ident.to_string()) {
                                *ty = named.clone();
                                return;
                            }
                        }
                    }
                }
                syn::visit_mut::visit_type_mut(self, ty);
            }
        }
        Names(&self.types).visit_type_mut(&mut ty);
        self.validate_type(&ty)?;
        Ok(ty)
    }
    fn validate_type(&self, ty: &Type) -> Result<()> {
        if Primitive::from_type(ty).is_some() || self.types.values().any(|t| same_type(t, ty)) {
            return Ok(());
        }
        match ty {
            Type::Tuple(t) if t.elems.is_empty() => Ok(()),
            Type::Reference(r) if r.mutability.is_none() && r.lifetime.is_none() => {
                self.validate_type(&r.elem)
            }
            Type::Slice(s) if type_key(&s.elem) == "u8" => Ok(()),
            Type::Path(_)
                if matches!(
                    type_key(ty).as_str(),
                    "String" | "str" | "u8" | "Vec < u8 >"
                ) =>
            {
                Ok(())
            }
            _ => Err(format!("unsupported source type `{}`", type_key(ty))),
        }
    }
    /// Change only slot payloads; source structure and origins are retained.
    pub fn map_slots<T>(self, mut f: impl FnMut(&Type, S) -> Result<T>) -> Result<Flat<T>> {
        fn param<S, T>(p: Param<S>, f: &mut impl FnMut(&Type, S) -> Result<T>) -> Result<Param<T>> {
            let slot = f(&p.ty.ty, p.ty.slot)?;
            Ok(Param {
                name: p.name,
                ty: TypeUse { ty: p.ty.ty, slot },
            })
        }
        let mut functions = BTreeMap::new();
        for (name, fun) in self.functions {
            let params = fun
                .params
                .into_iter()
                .map(|p| param(p, &mut f))
                .collect::<Result<_>>()?;
            let slot = f(&fun.ret.ty, fun.ret.slot)?;
            functions.insert(
                name,
                Function {
                    name: fun.name,
                    path: fun.path,
                    params,
                    ret: TypeUse {
                        ty: fun.ret.ty,
                        slot,
                    },
                    location: fun.location,
                },
            );
        }
        let mut records = BTreeMap::new();
        for (name, r) in self.records {
            records.insert(
                name,
                Record {
                    ty: r.ty,
                    fields: r
                        .fields
                        .into_iter()
                        .map(|p| param(p, &mut f))
                        .collect::<Result<_>>()?,
                },
            );
        }
        Ok(Flat {
            functions,
            records,
            types: self.types,
            guards: self.guards,
        })
    }
}

pub fn type_key(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}
pub fn same_type(a: &Type, b: &Type) -> bool {
    type_key(a) == type_key(b)
}

#[cfg(test)]
mod tests;
