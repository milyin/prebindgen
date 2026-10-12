//! The retained JniGen expansion vocabulary translates into tools recipes.

use crate::{Input, Output, Slot};
use prebindgen_flat::{type_key, Flat, Result};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct ExpandParamDecl {
    pub ty: syn::Type,
    variants: Vec<Option<FunctionDecl>>,
}
impl ExpandParamDecl {
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            variants: Vec::new(),
        }
    }
    pub fn variant(mut self, f: FunctionDecl) -> Self {
        self.variants.push(Some(f));
        self
    }
    pub fn variant_self(mut self) -> Self {
        self.variants.push(None);
        self
    }
}
#[derive(Clone)]
pub struct ExpandReturnDecl {
    pub ty: syn::Type,
    fields: Vec<ReturnField>,
}
#[derive(Clone)]
enum ReturnField {
    Identity,
    Getter(Box<FunctionDecl>),
    Fields(FieldsDecl),
}
#[derive(Clone)]
pub struct FieldsDecl {
    function: String,
    overrides: BTreeMap<String, ExpandReturnDecl>,
}
impl FieldsDecl {
    pub fn new(function: syn::Ident) -> Self {
        Self {
            function: function.to_string(),
            overrides: BTreeMap::new(),
        }
    }
    pub fn field(mut self, name: impl AsRef<str>, decl: ExpandReturnDecl) -> Self {
        assert!(
            self.overrides.insert(name.as_ref().into(), decl).is_none(),
            "duplicate field override"
        );
        self
    }
}
impl ExpandReturnDecl {
    pub fn fields(mut self, decl: FieldsDecl) -> Self {
        self.fields.push(ReturnField::Fields(decl));
        self
    }
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            fields: Vec::new(),
        }
    }
    pub fn field(mut self, f: FunctionDecl) -> Self {
        self.fields.push(ReturnField::Getter(Box::new(f)));
        self
    }
    pub fn field_self(mut self) -> Self {
        self.fields.push(ReturnField::Identity);
        self
    }
}
pub enum ExpandDecl {
    Param(ExpandParamDecl),
    Return(ExpandReturnDecl),
}
impl From<ExpandParamDecl> for ExpandDecl {
    fn from(d: ExpandParamDecl) -> Self {
        Self::Param(d)
    }
}
impl From<ExpandReturnDecl> for ExpandDecl {
    fn from(d: ExpandReturnDecl) -> Self {
        Self::Return(d)
    }
}
#[macro_export]
macro_rules! expand_param {($($ty:tt)+)=>{$crate::ExpandParamDecl::new($crate::declarations::parse_type(stringify!($($ty)+)))};}
#[macro_export]
macro_rules! expand_return {($($ty:tt)+)=>{$crate::ExpandReturnDecl::new($crate::declarations::parse_type(stringify!($($ty)+)))};}

pub struct Declarations {
    inputs: BTreeMap<String, ExpandParamDecl>,
    outputs: BTreeMap<String, ExpandReturnDecl>,
}
impl Declarations {
    pub fn new(flat: &Flat<Slot>, decls: Vec<ExpandDecl>) -> Result<Self> {
        let mut out = Self {
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        };
        for d in decls {
            match d {
                ExpandDecl::Param(d) => {
                    let key = type_key(&flat.resolve_type(&d.ty)?);
                    if out.inputs.insert(key, d).is_some() {
                        return Err("duplicate input expansion".into());
                    }
                }
                ExpandDecl::Return(d) => {
                    let key = type_key(&flat.resolve_type(&d.ty)?);
                    if out.outputs.insert(key, d).is_some() {
                        return Err("duplicate output expansion".into());
                    }
                }
            }
        }
        Ok(out)
    }
    pub fn fill(&self, flat: &mut Flat<Slot>, decl: &FunctionDecl) -> Result<()> {
        let source = flat.function(&decl.name)?.clone();
        for name in decl.params.keys() {
            if !source.params.iter().any(|p| p.name == name.as_str()) {
                return Err(format!("`{}`: unknown parameter `{name}`", decl.name));
            }
        }
        let mut inputs = Vec::new();
        for p in &source.params {
            let explicit = decl.params.get(&p.name.to_string());
            inputs.push(if explicit.is_some() || matches!(p.ty.slot, Slot::Empty) {
                Some(self.input(flat, &p.ty.ty, explicit, &mut Vec::new())?)
            } else {
                None
            });
        }
        let output = if decl.ret.is_some() || matches!(source.ret.slot, Slot::Empty) {
            Some(self.output(flat, &source.ret.ty, decl.ret.as_ref(), &mut Vec::new())?)
        } else {
            None
        };
        let dest = flat.function_mut(&decl.name)?;
        for (p, input) in dest.params.iter_mut().zip(inputs) {
            if let Some(input) = input {
                p.ty.slot = Slot::Input(input);
            }
        }
        if let Some(output) = output {
            dest.ret.slot = Slot::Output(output);
        }
        Ok(())
    }
    fn input(
        &self,
        flat: &Flat<Slot>,
        ty: &syn::Type,
        explicit: Option<&ExpandParamDecl>,
        stack: &mut Vec<String>,
    ) -> Result<Input> {
        let key = type_key(ty);
        let d = explicit.or_else(|| self.inputs.get(&key));
        if let Some(d) = d {
            if type_key(&flat.resolve_type(&d.ty)?) != key {
                return Err("input expansion type does not match occurrence".into());
            }
            if stack.contains(&key) {
                return Err(format!("recursive input expansion `{key}`"));
            }
            stack.push(key);
            let mut arms = Vec::new();
            for variant in &d.variants {
                arms.push(match variant {
                    None => Input::Handle,
                    Some(f) => {
                        if !f.params.is_empty() || f.ret.is_some() || f.foreign.is_some() {
                            return Err("constructor references only support their source name in this subset".into());
                        }
                        let func = flat.function(&f.name)?;
                        let args = func.params.iter()
                            .map(|p| self.input(flat, &p.ty.ty, None, stack))
                            .collect::<Result<Vec<_>>>()?;
                        Input::constructor(&f.name, args)
                    }
                });
            }
            stack.pop();
            return match arms.len() {
                0 => Err("empty input expansion".into()),
                1 => Ok(arms.remove(0)),
                _ => Ok(Input::selector(arms)),
            };
        }
        let key = match ty {
            syn::Type::Reference(r) => type_key(&r.elem),
            _ => key,
        };
        Ok(match key.as_str() {
            "String" | "str" => Input::Text,
            "Vec < u8 >" | "[u8]" => Input::Bytes,
            "()" => Input::fields([]),
            _ => Input::Handle,
        })
    }
    fn output(
        &self,
        flat: &Flat<Slot>,
        ty: &syn::Type,
        explicit: Option<&ExpandReturnDecl>,
        stack: &mut Vec<String>,
    ) -> Result<Output> {
        let key = type_key(ty);
        let d = explicit.or_else(|| self.outputs.get(&key));
        if let Some(d) = d {
            if type_key(&flat.resolve_type(&d.ty)?) != key {
                return Err("output expansion type does not match occurrence".into());
            }
            if stack.contains(&key) {
                return Err(format!("recursive output expansion `{key}`"));
            }
            stack.push(key);
            let mut parts = Vec::new();
            for field in &d.fields {
                parts.push(match field {
                    ReturnField::Identity => Output::Handle,
                    ReturnField::Getter(f) => {
                        if !f.params.is_empty() || f.ret.is_some() || f.foreign.is_some() {
                            return Err(
                                "getter references only support their source name in this subset"
                                    .into(),
                            );
                        }
                        let func = flat.function(&f.name)?;
                        Output::getter(&f.name, self.output(flat, &func.ret.ty, None, stack)?)
                    }
                    ReturnField::Fields(d) => {
                        let f = flat.function(&d.function)?;
                        let record = flat.record(&f.ret.ty)?;
                        for name in d.overrides.keys() {
                            if !record.fields.iter().any(|p| p.name == name.as_str()) {
                                return Err(format!("unknown output field `{name}`"));
                            }
                        }
                        let children = record
                            .fields
                            .iter()
                            .map(|p| {
                                self.output(
                                    flat,
                                    &p.ty.ty,
                                    d.overrides.get(&p.name.to_string()),
                                    stack,
                                )
                                .map(clone_field_handles)
                            })
                            .collect::<Result<Vec<_>>>()?;
                        Output::getter(&d.function, Output::fields(children))
                    }
                });
            }
            stack.pop();
            return match parts.len() {
                0 => Err("empty output expansion is outside this subset".into()),
                1 => Ok(parts.remove(0)),
                _ => Ok(Output::combine(parts)),
            };
        }
        Ok(match key.as_str() {
            "String" => Output::Text,
            "Vec < u8 >" => Output::Bytes,
            "()" => Output::fields([]),
            _ => Output::Handle,
        })
    }
}

// A value-form field is borrowed. Identity delivery therefore needs an independent handle.
fn clone_field_handles(output: Output) -> Output {
    match output {
        Output::Handle => Output::ClonedHandle,
        Output::Combine(parts) => Output::combine(parts.into_iter().map(clone_field_handles)),
        other => other,
    }
}
#[doc(hidden)]
pub fn parse_type(s: &str) -> syn::Type {
    syn::parse_str(s).expect("type syntax")
}
#[doc(hidden)]
pub fn parse_ident(s: &str) -> syn::Ident {
    syn::parse_str(s).expect("identifier syntax")
}
#[macro_export]
macro_rules! fun {
    ($name:ident) => {
        $crate::FunctionDecl::new($crate::declarations::parse_ident(stringify!($name)))
    };
}
#[macro_export]
macro_rules! fields {
    ($name:ident) => {
        $crate::FieldsDecl::new($crate::declarations::parse_ident(stringify!($name)))
    };
}

#[derive(Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub foreign: Option<String>,
    pub params: BTreeMap<String, ExpandParamDecl>,
    pub ret: Option<ExpandReturnDecl>,
}
impl FunctionDecl {
    pub fn new(name: syn::Ident) -> Self {
        Self {
            name: name.to_string(),
            foreign: None,
            params: BTreeMap::new(),
            ret: None,
        }
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.foreign = Some(name.into());
        self
    }
    pub fn expand_param(mut self, name: impl AsRef<str>, decl: ExpandParamDecl) -> Self {
        assert!(
            self.params.insert(name.as_ref().into(), decl).is_none(),
            "duplicate parameter override"
        );
        self
    }
    pub fn expand_return(mut self, decl: ExpandReturnDecl) -> Self {
        self.ret = Some(decl);
        self
    }
}
