//! The small flat source model. Types retain Rust syntax and an occurrence slot.
//! Named objects must have nonzero Rust size, including opaque aliases. Their
//! layout is checked by the emitted compile-time guards, independently of the
//! adapter or conversion chosen. Unit remains available for absence of a value.
#![warn(missing_docs)]
use prebindgen::SourceLocation;
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};
use syn::{visit_mut::VisitMut, Type};

mod primitive;
pub use primitive::Primitive;

/// A model or generation result with a diagnostic suitable for a build script.
pub type Result<T> = std::result::Result<T, String>;

/// One occurrence of a Rust type, paired with stage-specific data.
///
/// Two parameters of the same type have independent slots; the slot is not a
/// type-table entry.
#[derive(Clone, Debug)]
pub struct TypeUse<S> {
    /// Qualified source type; independent of its chosen conversion.
    pub ty: Type,
    /// Data for this occurrence, such as a recipe or prepared conversion.
    pub slot: S,
}
/// A named function parameter or record field, in source declaration order.
#[derive(Clone, Debug)]
pub struct Param<S> {
    /// Source parameter or field name.
    pub name: syn::Ident,
    /// Its source type and occurrence slot.
    pub ty: TypeUse<S>,
}
/// A validated free function and its independently configurable type occurrences.
#[derive(Clone, Debug)]
pub struct Function<S> {
    /// Unqualified name used by binding declarations.
    pub name: syn::Ident,
    /// Qualified path used to call the source function from generated Rust.
    pub path: syn::Path,
    /// Parameters in call order.
    pub params: Vec<Param<S>>,
    /// Result occurrence; an omitted Rust return type becomes `()`.
    pub ret: TypeUse<S>,
    /// Origin recorded by source capture, for diagnostics and qualification.
    pub location: SourceLocation,
}
/// A named-field record available for field construction and projection.
#[derive(Clone, Debug)]
pub struct Record<S> {
    /// Qualified record type.
    pub ty: Type,
    /// Public fields in declaration order.
    pub fields: Vec<Param<S>>,
}
/// Source structure with a slot at every parameter, result, and record field.
///
/// Use [`Flat::parse`] or [`Flat::parse_selected`] to validate source syntax and
/// qualify declared names. Aliases stay opaque. Named types must have nonzero
/// Rust size; generated code must emit [`Self::guards`] to enforce that rule.
///
/// ```
/// use prebindgen_flat::{Flat, type_key};
/// let items = syn::parse_file("pub fn add(x: i64, y: i64) -> i64 { x + y }")?.items;
/// let flat = Flat::<()>::parse(items.into_iter().map(|item| (item, Default::default())))?;
/// let mut flat = flat.map_slots(|ty, ()| Ok(type_key(ty)))?;
/// flat.function_mut("add")?.params[0].ty.slot = "first input".into();
/// assert_eq!(flat.function("add")?.params[1].ty.slot, "i64");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct Flat<S> {
    /// Free functions indexed by unqualified source name.
    pub functions: BTreeMap<String, Function<S>>,
    /// Records indexed by [`type_key`] of their qualified type.
    pub records: BTreeMap<String, Record<S>>,
    /// Declared type names mapped to qualified Rust types, including opaque aliases.
    pub types: BTreeMap<String, Type>,
    /// Constants every adapter must emit: captured feature guards and generated
    /// nonzero-size assertions. Layout assertions run when the bindings compile.
    pub guards: Vec<syn::ItemConst>,
}

impl<S: Default> Flat<S> {
    /// Select the source declarations needed by this binding before validating
    /// the supported flat subset. Unbound source items are not language features.
    ///
    /// Selection follows function signatures and record fields transitively, but
    /// does not inspect opaque alias implementations. Unnamed constant guards are
    /// retained. Selected items are validated by [`Self::parse`].
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
    /// belong to this subset. Rejects duplicate names, non-public declarations,
    /// unsupported signatures and types, and inaccessible record fields.
    ///
    /// All slots start at `S::default()`. Unqualified declared types and callable
    /// paths are qualified with the captured crate name (or `crate` if absent).
    /// Opaque alias implementations are not inspected. Rust layout and trait
    /// requirements are checked later when the generated code compiles.
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
        // Rust, not source syntax, determines layout (especially for opaque
        // aliases). Keep this flat-model invariant in the guards all adapters emit.
        for (name, ty) in &flat.types {
            let message = format!("flat object `{name}` must have nonzero size");
            flat.guards.push(syn::parse_quote! {
                const _: () = assert!(::core::mem::size_of::<#ty>() != 0, #message);
            });
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
    /// Look up a free function by its unqualified source name.
    ///
    /// Returns an error if it was not included in this model.
    pub fn function(&self, name: &str) -> Result<&Function<S>> {
        self.functions
            .get(name)
            .ok_or_else(|| format!("unknown function `{name}`"))
    }
    /// Look up a function for occurrence-slot configuration.
    ///
    /// Returns an error for an unknown source name; changing slots does not change
    /// the original Rust parameter or result types.
    pub fn function_mut(&mut self, name: &str) -> Result<&mut Function<S>> {
        self.functions
            .get_mut(name)
            .ok_or_else(|| format!("unknown function `{name}`"))
    }
    /// Look up a named record by its qualified type syntax.
    ///
    /// Use [`Self::resolve_type`] first for an unqualified declaration name. Opaque
    /// aliases have no record entry and return an error.
    pub fn record(&self, ty: &Type) -> Result<&Record<S>> {
        self.records
            .get(&type_key(ty))
            .ok_or_else(|| format!("`{}` is not a record", type_key(ty)))
    }
    /// Qualify declared names and validate the supported source-type grammar.
    ///
    /// Accepts primitives, declared types, unit, strings, byte vectors/slices and
    /// references without explicit lifetimes. A successful result does not mean
    /// every adapter or recipe supports the type, and aliases remain opaque.
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
            Type::Reference(r) if r.lifetime.is_none() => self.validate_type(&r.elem),
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
    /// Transform every occurrence slot, preserving source structure and guards.
    ///
    /// Calls `f` with the source type and old slot for each function parameter,
    /// result, and record field. Stops on its first error. This is a stage
    /// transformation, not dependency-ordered traversal.
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

/// Return the token spelling used to index a type.
///
/// This is syntactic identity, not Rust type equivalence or alias resolution.
pub fn type_key(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}
/// Compare token spellings; this does not resolve aliases or normalize types.
pub fn same_type(a: &Type, b: &Type) -> bool {
    type_key(a) == type_key(b)
}

#[cfg(test)]
mod tests;
