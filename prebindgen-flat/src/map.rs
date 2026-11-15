//! Move source declarations between schemas by transforming only their attachments.
use crate::*;

/// Transform data attached to source declarations, preserving their identities and structure.
///
/// Methods run in source-table order. Within a function parameters precede the
/// return; within a type the type data precedes variant and field data. The mapper
/// owns any preparation context it needs. Flat does not interpret the payloads.
pub trait MapAttachments<A: Schema, B: Schema> {
    /// Transform module data.
    fn module(&mut self, data: A::Module) -> Result<B::Module>;
    /// Transform function data. Parameter/result source types remain on the function.
    fn function(
        &mut self,
        name: &syn::Ident,
        path: &syn::Path,
        data: A::Function,
    ) -> Result<B::Function>;
    /// Transform a source parameter's data.
    fn parameter(
        &mut self,
        function: &syn::Ident,
        name: &syn::Ident,
        ty: &Type,
        data: A::Parameter,
    ) -> Result<B::Parameter>;
    /// Transform a source return's data.
    fn returned(&mut self, function: &syn::Ident, ty: &Type, data: A::Return) -> Result<B::Return>;
    /// Transform shared data of a source type.
    fn ty(&mut self, ty: &Type, data: A::Type) -> Result<B::Type>;
    /// Transform source field data, including positional enum members.
    fn field(
        &mut self,
        owner: &Type,
        variant: Option<&syn::Ident>,
        member: &syn::Member,
        ty: &Type,
        data: A::Field,
    ) -> Result<B::Field>;
    /// Transform source enum-variant data.
    fn variant(&mut self, owner: &Type, name: &syn::Ident, data: A::Variant) -> Result<B::Variant>;
}

impl<A: Schema> Flat<A> {
    /// Consume this model, moving its source declarations and transforming each attachment once.
    ///
    /// No source AST is cloned and no consumer has to reconstruct the source
    /// tables. On failure both the remaining model and completed attachments drop
    /// normally; the mapper is borrowed so callers may recover its own state.
    pub fn try_map_attachments<B: Schema>(
        self,
        map: &mut impl MapAttachments<A, B>,
    ) -> Result<Flat<B>> {
        let metadata = map.module(self.metadata)?;
        let functions = self
            .functions
            .into_iter()
            .map(|(key, f)| {
                let metadata = map.function(&f.name, &f.path, f.metadata)?;
                let params = f
                    .params
                    .into_iter()
                    .map(|p| {
                        Ok(Param {
                            ty: TypeUse {
                                slot: map.parameter(&f.name, &p.name, &p.ty.ty, p.ty.slot)?,
                                ty: p.ty.ty,
                            },
                            name: p.name,
                        })
                    })
                    .collect::<Result<_>>()?;
                let ret = TypeUse {
                    slot: map.returned(&f.name, &f.ret.ty, f.ret.slot)?,
                    ty: f.ret.ty,
                };
                Ok((
                    key,
                    Function {
                        metadata,
                        name: f.name,
                        path: f.path,
                        params,
                        ret,
                        location: f.location,
                    },
                ))
            })
            .collect::<Result<_>>()?;
        let types = self
            .types
            .into_iter()
            .map(|(key, t)| {
                let metadata = map.ty(&t.ty, t.metadata)?;
                let definition = match t.definition {
                    TypeDefinition::External => TypeDefinition::External,
                    TypeDefinition::Opaque => TypeDefinition::Opaque,
                    TypeDefinition::Record(r) => TypeDefinition::Record(Record {
                        ty: r.ty,
                        fields: r
                            .fields
                            .into_iter()
                            .map(|f| {
                                Ok(Field {
                                    metadata: map.field(
                                        &t.ty,
                                        None,
                                        &syn::Member::Named(f.name.clone()),
                                        &f.ty,
                                        f.metadata,
                                    )?,
                                    name: f.name,
                                    ty: f.ty,
                                })
                            })
                            .collect::<Result<_>>()?,
                    }),
                    TypeDefinition::Enum(e) => TypeDefinition::Enum(FieldlessEnum {
                        ty: e.ty,
                        variants: e
                            .variants
                            .into_iter()
                            .map(|v| {
                                Ok(EnumVariant {
                                    metadata: map.variant(&t.ty, &v.name, v.metadata)?,
                                    name: v.name,
                                    discriminant: v.discriminant,
                                })
                            })
                            .collect::<Result<_>>()?,
                    }),
                    TypeDefinition::PayloadEnum(e) => TypeDefinition::PayloadEnum(PayloadEnum {
                        ty: e.ty,
                        variants: e
                            .variants
                            .into_iter()
                            .map(|v| {
                                let metadata = map.variant(&t.ty, &v.name, v.metadata)?;
                                let fields = v
                                    .fields
                                    .into_iter()
                                    .map(|f| {
                                        Ok(PayloadField {
                                            metadata: map.field(
                                                &t.ty,
                                                Some(&v.name),
                                                &f.member,
                                                &f.ty,
                                                f.metadata,
                                            )?,
                                            member: f.member,
                                            ty: f.ty,
                                        })
                                    })
                                    .collect::<Result<_>>()?;
                                Ok(PayloadVariant {
                                    metadata,
                                    name: v.name,
                                    style: v.style,
                                    fields,
                                })
                            })
                            .collect::<Result<_>>()?,
                    }),
                };
                Ok((
                    key,
                    TypeInfo {
                        ty: t.ty,
                        metadata,
                        definition,
                    },
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Flat {
            metadata,
            functions,
            types,
            guards: self.guards,
        })
    }
}

impl<A: Schema> Flat<A> {
    /// Change function attachments while moving every source declaration unchanged.
    /// Other node payload types are preserved, so consumers only map their own function data.
    pub fn try_map_function_data<B, F>(self, map: F) -> Result<Flat<B>>
    where
        B: Schema<
            Module = A::Module,
            Type = A::Type,
            Field = A::Field,
            Variant = A::Variant,
            Parameter = A::Parameter,
            Return = A::Return,
        >,
        F: FnMut(&syn::Ident, &syn::Path, A::Function) -> Result<B::Function>,
    {
        struct Functions<F>(F);
        impl<A, B, F> MapAttachments<A, B> for Functions<F>
        where
            A: Schema,
            B: Schema<
                Module = A::Module,
                Type = A::Type,
                Field = A::Field,
                Variant = A::Variant,
                Parameter = A::Parameter,
                Return = A::Return,
            >,
            F: FnMut(&syn::Ident, &syn::Path, A::Function) -> Result<B::Function>,
        {
            fn module(&mut self, d: A::Module) -> Result<B::Module> {
                Ok(d)
            }
            fn function(
                &mut self,
                n: &syn::Ident,
                p: &syn::Path,
                d: A::Function,
            ) -> Result<B::Function> {
                (self.0)(n, p, d)
            }
            fn parameter(
                &mut self,
                _: &syn::Ident,
                _: &syn::Ident,
                _: &Type,
                d: A::Parameter,
            ) -> Result<B::Parameter> {
                Ok(d)
            }
            fn returned(&mut self, _: &syn::Ident, _: &Type, d: A::Return) -> Result<B::Return> {
                Ok(d)
            }
            fn ty(&mut self, _: &Type, d: A::Type) -> Result<B::Type> {
                Ok(d)
            }
            fn field(
                &mut self,
                _: &Type,
                _: Option<&syn::Ident>,
                _: &syn::Member,
                _: &Type,
                d: A::Field,
            ) -> Result<B::Field> {
                Ok(d)
            }
            fn variant(&mut self, _: &Type, _: &syn::Ident, d: A::Variant) -> Result<B::Variant> {
                Ok(d)
            }
        }
        self.try_map_attachments(&mut Functions(map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Owned(String); // deliberately not Clone
    struct Before;
    struct After;
    macro_rules! schema {
        ($name:ident, $data:ty) => {
            impl Schema for $name {
                type Module = $data;
                type Function = $data;
                type Parameter = $data;
                type Return = $data;
                type Type = $data;
                type Field = $data;
                type Variant = $data;
            }
        };
    }
    schema!(Before, Owned);
    schema!(After, String);
    struct Label;
    impl MapAttachments<Before, After> for Label {
        fn module(&mut self, d: Owned) -> Result<String> {
            Ok(d.0)
        }
        fn function(&mut self, n: &syn::Ident, _: &syn::Path, d: Owned) -> Result<String> {
            Ok(format!("function:{n}:{}", d.0))
        }
        fn parameter(
            &mut self,
            f: &syn::Ident,
            n: &syn::Ident,
            _: &Type,
            d: Owned,
        ) -> Result<String> {
            Ok(format!("parameter:{f}.{n}:{}", d.0))
        }
        fn returned(&mut self, f: &syn::Ident, _: &Type, d: Owned) -> Result<String> {
            Ok(format!("return:{f}:{}", d.0))
        }
        fn ty(&mut self, t: &Type, d: Owned) -> Result<String> {
            Ok(format!("type:{}:{}", type_key(t), d.0))
        }
        fn field(
            &mut self,
            _: &Type,
            v: Option<&syn::Ident>,
            m: &syn::Member,
            _: &Type,
            d: Owned,
        ) -> Result<String> {
            Ok(format!(
                "field:{}:{}:{}",
                v.map(ToString::to_string).unwrap_or_default(),
                m.to_token_stream(),
                d.0
            ))
        }
        fn variant(&mut self, _: &Type, n: &syn::Ident, d: Owned) -> Result<String> {
            Ok(format!("variant:{n}:{}", d.0))
        }
    }
    #[test]
    fn transforms_every_source_position_without_clone_or_optional_data() {
        let items=syn::parse_file("pub struct R{pub x:i64}pub enum E{A,B}pub enum P{A(i64),B{r:R}}pub type H=hidden::Handle;pub fn f(r:R)->P{}").unwrap().items;
        let mut flat =
            Flat::<Before>::parse(items.into_iter().map(|i| (i, Default::default()))).unwrap();
        flat.metadata = Owned("module".into());
        flat.function_mut("f").unwrap().params[0].ty.slot = Owned("argument".into());
        let guard_count = flat.guards.len();
        let mapped = flat.try_map_attachments::<After>(&mut Label).unwrap();
        assert_eq!(mapped.metadata, "module");
        assert_eq!(
            mapped.function("f").unwrap().params[0].ty.slot,
            "parameter:f.r:argument"
        );
        assert_eq!(mapped.function("f").unwrap().ret.slot, "return:f:");
        assert_eq!(
            mapped.record(&syn::parse_quote!(crate::R)).unwrap().fields[0].metadata,
            "field::x:"
        );
        assert_eq!(
            mapped.enum_by_key("crate :: E").unwrap().variants[1].metadata,
            "variant:B:"
        );
        let p = mapped.payload_enum(&syn::parse_quote!(crate::P)).unwrap();
        assert_eq!(p.variants[0].fields[0].metadata, "field:A:0:");
        assert_eq!(p.variants[1].fields[0].metadata, "field:B:r:");
        assert!(matches!(
            mapped
                .type_info(&syn::parse_quote!(crate::H))
                .unwrap()
                .definition,
            TypeDefinition::Opaque
        ));
        assert_eq!(mapped.guards.len(), guard_count);
    }
}
