//! Kotlin enum declarations and checked JNI integer conversions.
use super::*;
use prebindgen_declarations::{Condition, EnumMapping};

/// A Kotlin enum class preserving a fieldless Rust enum's integer discriminants.
#[derive(Clone)]
pub struct EnumClassDecl {
    /// Source enum type, resolved against the selected source.
    pub ty: syn::Type,
    /// Literal Kotlin class name, or the source type's last path segment.
    pub name: Option<String>,
    /// Explicit Kotlin entry-name overrides keyed by source variant name.
    pub variant_names: Vec<(String, String)>,
}
impl EnumClassDecl {
    /// Declare a fieldless enum class from its source type syntax.
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            name: None,
            variant_names: Vec::new(),
        }
    }
    /// Override the Kotlin short class name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Set a Kotlin entry name without changing the source discriminant.
    /// Unknown source names, duplicate overrides and conflicting foreign names
    /// are rejected during build.
    pub fn variant_name(mut self, source: impl Into<String>, name: impl Into<String>) -> Self {
        self.variant_names.push((source.into(), name.into()));
        self
    }
}
#[macro_export]
/// Declare a fieldless Kotlin enum class without requiring the Rust type in scope.
macro_rules! enum_class {($($ty:tt)+)=>{$crate::EnumClassDecl::new($crate::__type(stringify!($($ty)+)))};}

/// A JNI enum's checked integer mapping and Kotlin class identity.
#[derive(Clone, Debug)]
pub struct JniEnum {
    /// Source variants and their signed 32-bit representations.
    pub mapping: EnumMapping,
    /// Fully qualified generated Kotlin enum class name.
    pub class_name: String,
    /// Explicit Kotlin variant-name overrides; source names are otherwise retained.
    pub variant_names: BTreeMap<String, String>,
}
impl JniEnum {
    pub(crate) fn from_class<I, O, M: prebindgen_flat::Metadata>(
        flat: &Flat<I, O, M>,
        class: &Class,
        key: &str,
    ) -> Result<Self> {
        let source = flat
            .enums
            .get(key)
            .ok_or("enum_class requires a fieldless source enum")?;
        let mut variant_names = BTreeMap::new();
        for (source, name) in &class.variant_names {
            if variant_names.insert(source.clone(), name.clone()).is_some() {
                return Err("duplicate enum variant override".into());
            }
        }
        let result = Self {
            mapping: EnumMapping::new(source)?,
            class_name: class.fqn(),
            variant_names,
        };
        result.validate()?;
        Ok(result)
    }
    pub(crate) fn variant_name(&self, source: &syn::Ident) -> String {
        let name = source.to_string();
        let source = name.trim_start_matches("r#");
        self.variant_names
            .get(source)
            .cloned()
            .unwrap_or_else(|| source.into())
    }
    pub(crate) fn validate(&self) -> Result<()> {
        self.mapping.validate()?;
        validate_package(&self.class_name)?;
        for source in self.variant_names.keys() {
            if !self
                .mapping
                .variants
                .iter()
                .any(|(n, _)| n.to_string().trim_start_matches("r#") == source)
            {
                return Err(format!("unknown enum variant override `{source}`"));
            }
        }
        let mut names = BTreeSet::new();
        for (source, _) in &self.mapping.variants {
            let name = self.variant_name(source);
            prebindgen_declarations::ident(&format!("v_{name}"))?;
            if ["name", "ordinal", "entries", "kotlin"].contains(&name.as_str())
                || name.chars().all(|c| c == '_')
            {
                return Err(format!(
                    "Kotlin enum entry `{name}` is reserved; use variant_name to rename it"
                ));
            }
            if !names.insert(name) {
                return Err("duplicate Kotlin enum entry name".into());
            }
        }
        Ok(())
    }
    fn member(&self, initial: &str) -> String {
        let mut name = initial.to_string();
        while self
            .mapping
            .variants
            .iter()
            .any(|(v, _)| self.variant_name(v) == name)
        {
            name.insert(0, '_');
        }
        name
    }
    pub(crate) fn value_member(&self) -> String {
        self.member("wireValue")
    }
    pub(crate) fn decode_member(&self) -> String {
        self.member("fromWire")
    }
    pub(crate) fn wire(self: &std::rc::Rc<Self>) -> JniWire {
        JniWire::Enum {
            enum_type: self.clone(),
            next_integer_niche: self.mapping.initial_niche(),
        }
    }
    pub(crate) fn input(
        self: &std::rc::Rc<Self>,
        ty: &syn::Type,
        w: syn::Ident,
        v: syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<JniWire>> {
        prebindgen_declarations::enum_input(&self.mapping, ty, w, v, active, self.wire())
    }
    pub(crate) fn output(
        self: &std::rc::Rc<Self>,
        w: syn::Ident,
        v: TokenStream,
    ) -> OutputPlan<JniWire> {
        prebindgen_declarations::enum_output(&self.mapping, w, v, self.wire())
    }
    pub(crate) fn render(&self, class: &Class) -> Result<String> {
        let mut entries = Vec::new();
        let mut cases = Vec::new();
        for (variant, value) in &self.mapping.variants {
            let name = self.variant_name(variant);
            prebindgen_declarations::ident(&format!("v_{name}"))?;
            entries.push(format!("    `{name}`({value})"));
            cases.push(format!("            {value} -> `{name}`"));
        }
        let member = self.value_member();
        let decode = self.decode_member();
        let companion = self.member("Companion");
        let parameter = self.member("value");
        Ok(format!("package {}\n\npublic enum class {}(public val {member}: kotlin.Int) {{\n{};\n\n    public companion object {companion} {{\n        public fun {decode}({parameter}: kotlin.Int): {} = when ({parameter}) {{\n{}\n            else -> throw kotlin.IllegalArgumentException(\"invalid enum discriminant: ${parameter}\")\n        }}\n    }}\n}}\n",class.pkg,class.name,entries.join(",\n"),class.name,cases.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn flat(source: &str) -> Bindings {
        Flat::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap()
    }
    #[test]
    fn enum_classes_are_order_independent_and_use_discriminants_in_data_fields() {
        let generated = JniGen::builder().flat(flat("pub enum E { A=-2, B, C=7 } pub struct Record { pub value: E, pub maybe: Option<E> } pub fn echo(x:Record)->Record {}"))
            .package(package!("records").class(data_class!(Record)).fun(fun!(echo)))
            .package(package!("codes").class(enum_class!(E).name("Code")))
            .set_package_prefix("test").build().unwrap();
        let enumeration = &generated.kotlin["test/codes/Code.kt"];
        assert!(enumeration.contains("`B`(-1)"));
        assert!(enumeration.contains("`C`(7)"));
        assert!(enumeration.contains("else -> throw kotlin.IllegalArgumentException"));
        let record = &generated.kotlin["test/records/Record.kt"];
        assert!(record.contains("test.codes.Code"));
        assert!(record.contains("test.codes.Code?"));
        syn::parse_file(&generated.rust).unwrap();
    }
    #[test]
    fn enum_class_aliases_bad_discriminants_and_duplicate_bindings_fail() {
        let fail = |source: &str| {
            JniGen::builder()
                .flat(flat(source))
                .set_package_prefix("test")
                .package(package!().class(enum_class!(E)))
                .build()
                .err()
                .unwrap()
        };
        assert!(fail("pub type E = hidden::E;").contains("requires a fieldless source enum"));
        assert!(fail("pub enum E { A = 2147483648 }").contains("does not fit"));
        assert!(fail("pub enum E { A = CONST }").contains("integer literals"));
        assert!(JniGen::builder()
            .flat(flat("pub enum E { A, B }"))
            .set_package_prefix("test")
            .package(package!().class(enum_class!(E)).class(ptr_class!(E)))
            .build()
            .err()
            .unwrap()
            .contains("duplicate"));
    }
    #[test]
    fn enum_wire_niches_validate_domain_skip_variants_and_keep_consumed_state() {
        let source = flat("pub enum E { A = -2, B, C = 0, D = 7 }");
        let e = JniEnum {
            mapping: EnumMapping::new(source.enums.values().next().unwrap()).unwrap(),
            class_name: "test.E".into(),
            variant_names: BTreeMap::new(),
        };
        let mut wire = std::rc::Rc::new(e).wire();
        assert_eq!(wire.available_niche(), Some(JniNiche::Integer(-3)));
        assert_eq!(
            optional::remaining(&wire, &JniNiche::Integer(8)),
            Some(JniNiche::Integer(6))
        );
        for bad in [
            JniNiche::Null,
            JniNiche::Integer(-1),
            JniNiche::Integer(i64::MAX),
        ] {
            assert!(wire.set_niche(Some(bad)).is_err());
            assert_eq!(wire.available_niche(), Some(JniNiche::Integer(-3)));
        }
        wire.clear_niche();
        assert!(wire.available_niche().is_none());
        wire.validate_niche(&JniNiche::Integer(-3)).unwrap();
        wire.set_niche(Some(JniNiche::Integer(i32::MIN as i64)))
            .unwrap();
        assert!(optional::remaining(&wire, &JniNiche::Integer(i32::MIN as i64)).is_none());
    }
    #[test]
    fn generated_infrastructure_avoids_source_variant_names() {
        let generated = JniGen::builder()
            .flat(flat("pub enum E { Companion=1, wireValue=2, fromWire=3 }"))
            .set_package_prefix("test")
            .package(package!().class(enum_class!(E)))
            .build()
            .unwrap();
        let kt = &generated.kotlin["test/E.kt"];
        assert!(kt.contains("public val _wireValue"));
        assert!(kt.contains("companion object _Companion"));
        assert!(kt.contains("fun _fromWire"));
    }
}

#[cfg(test)]
mod naming_tests {
    use super::*;
    fn build(decl: EnumClassDecl) -> Result<JniGen> {
        let items = syn::parse_file("pub enum E { name=1, ordinal=2, entries=3 }")
            .unwrap()
            .items;
        JniGen::builder()
            .items(items.into_iter().map(|i| (i, Default::default())))
            .set_package_prefix("test")
            .package(package!().class(decl))
            .build()
    }
    #[test]
    fn reserved_entry_names_require_explicit_unique_overrides() {
        assert!(build(enum_class!(E)).err().unwrap().contains("reserved"));
        let valid = || {
            enum_class!(E)
                .variant_name("name", "Name")
                .variant_name("ordinal", "Ordinal")
                .variant_name("entries", "Entries")
        };
        assert!(build(valid()).is_ok());
        assert!(build(valid().variant_name("unknown", "Other"))
            .err()
            .unwrap()
            .contains("unknown"));
        assert!(build(valid().variant_name("name", "Other"))
            .err()
            .unwrap()
            .contains("duplicate"));
        assert!(build(
            enum_class!(E)
                .variant_name("name", "Same")
                .variant_name("ordinal", "Same")
                .variant_name("entries", "Entries")
        )
        .err()
        .unwrap()
        .contains("duplicate Kotlin enum"));
    }
}
