//! Public function parameter names, independent of native wire identities.
use super::*;
use prebindgen_declarations::ComponentSegment;

fn segment_identity(segment: &ComponentSegment) -> String {
    match segment {
        ComponentSegment::Field(name) => format!("field:{name}"),
        ComponentSegment::Argument(name) => format!("argument:{name}"),
        ComponentSegment::Alternative(index) => format!("alternative:{index}"),
        ComponentSegment::View(index) => format!("view:{index}"),
        ComponentSegment::Getter(name) => format!("getter:{name}"),
        ComponentSegment::Selector => "selector".into(),
        ComponentSegment::Named(name) => format!("named:{name}"),
    }
}
/// All declared foreign packages, including declarations retained when the native
/// module is relocated. Those roots remain free references in any function body.
pub(crate) fn package_roots(flat: &Bindings) -> BTreeSet<String> {
    let mut names = vec![flat.metadata.package.clone()];
    for ty in flat.types.values() {
        if let Some(class) = &ty.metadata.class {
            names.push(class.pkg.clone());
        }
        if let Some(enumeration) = &ty.metadata.enumeration {
            names.push(enumeration.class_name.clone());
        }
        if let Some(exception) = &ty.metadata.error_class {
            names.push(exception.clone());
        }
    }
    let nodes = crate::metadata::nodes(flat);
    for wire in prebindgen_flat::wire_definitions(nodes.into_iter().map(|(_, node)| node)) {
        match &wire.kind {
            JniWire::Callback { callback } => names.push(callback.fqn()),
            JniWire::Enum { enum_type, .. } => names.push(enum_type.class_name.clone()),
            _ => {}
        }
    }
    names
        .into_iter()
        .filter_map(|name| name.split('.').next().map(str::to_owned))
        .collect()
}

fn reserved(name: &str, roots: &BTreeSet<String>, locals: &BTreeSet<String>) -> bool {
    [
        "onError",
        "onBindingError",
        "build",
        "fold",
        "initial",
        "returnCapture",
        "nativeResult",
        "result",
        "ownership",
        "handles",
        "remaining",
        "taken",
        "it",
        "run",
        "requireNotNull",
        "require",
        "check",
        "error",
        "kotlin",
        "java",
    ]
    .contains(&name)
        || roots.contains(name)
        || name.starts_with("__pb")
        || ["handle", "nativeArgument", "output", "errorOutput"]
            .iter()
            .any(|prefix| {
                name.strip_prefix(prefix).is_some_and(|rest| {
                    rest.starts_with(|c: char| c.is_ascii_digit())
                        || rest.starts_with("g0")
                        || rest == "owners"
                })
            })
        || locals.contains(name)
}

/// Apply one simultaneous mapping only to Kotlin input code. The native plans,
/// output-builder labels, JNI declarations and source conversion trees stay intact.
pub(crate) fn apply(
    source: &[String],
    codes: &mut [KotlinInput],
    roots: &BTreeSet<String>,
) -> Result<()> {
    struct Name {
        input: usize,
        position: usize,
        wire: String,
        public: String,
        identity: Vec<String>,
    }
    if source.len() != codes.len() {
        return Err("Kotlin input conversions do not match source parameters".into());
    }
    let mut pending = Vec::new();
    let mut input_locals = Vec::new();
    let setup_names = codes
        .iter()
        .flat_map(|code| code.setup.iter().map(|b| b.name.clone()))
        .collect::<BTreeSet<_>>();
    for (input, (source, code)) in source.iter().zip(codes.iter_mut()).enumerate() {
        code.complete_parameter_paths()?;
        let mut locals = setup_names.clone();
        let source = source.trim_start_matches("r#");
        let paths = code.parameter_paths.clone();
        if paths.len() != code.parameters.len() {
            return Err(format!(
                "Kotlin parameter names for `{source}` do not match its input representation"
            ));
        }
        locals.extend(code.setup.iter().map(|binding| binding.name.clone()));
        // These snippets can contain callback bridges with their own parameter
        // and local bindings. Avoid capturing those bindings when renaming a
        // callback parameter or an expanded input.
        for text in code
            .arguments
            .iter()
            .chain(&code.prepared_arguments)
            .chain(&code.locks)
            .chain(&code.consumed)
            .chain(&code.lock_groups)
            .chain(code.setup.iter().map(|b| &b.expression))
        {
            locals.extend(identifiers(text));
        }
        for (index, (parameter, mut path)) in code.parameters.iter().zip(paths).enumerate() {
            let wire = parameter
                .split_once(": ")
                .ok_or("invalid Kotlin input parameter")?
                .0
                .to_owned();
            if code.parameters.len() == 1 {
                path.0.clear();
            } else if path.0.is_empty() {
                path.0
                    .push(ComponentSegment::Named(format!("value{index}")));
            }
            let identity = std::iter::once(source.to_owned())
                .chain(path.0.iter().map(segment_identity))
                .collect::<Vec<_>>();
            let parts = std::iter::once(source.to_owned())
                .chain(path.0.iter().map(|segment| match segment {
                    ComponentSegment::Selector => "variant".into(),
                    ComponentSegment::Alternative(index) => format!("variant{index}"),
                    _ => segment.name().trim_start_matches("r#").to_owned(),
                }))
                .collect::<Vec<_>>();
            let public = camel(&parts.join("_"));
            let public = if public.is_empty() {
                "argument".into()
            } else {
                public
            };
            pending.push(Name {
                input,
                position: index,
                wire,
                public,
                identity,
            });
        }
        input_locals.push(locals);
    }
    // All collisions get a source-derived suffix, so reordering parameters does
    // not give an arbitrary first parameter the unsuffixed spelling.
    let mut counts = BTreeMap::<String, usize>::new();
    for name in &pending {
        *counts.entry(name.public.clone()).or_default() += 1;
    }
    let mut identities = BTreeMap::<Vec<String>, usize>::new();
    for name in &pending {
        *identities.entry(name.identity.clone()).or_default() += 1;
    }
    let mut names = names::Names::default();
    let mut claimed = BTreeSet::new();
    let mut mappings = vec![BTreeMap::new(); codes.len()];
    for mut name in pending {
        // Unnamed manual groups can repeat a relative path. Only those ambiguous
        // identities need a positional discriminator; named paths stay stable.
        if identities[&name.identity] > 1 {
            name.identity.push(format!("position:{}", name.position));
        }
        let mut public = name.public;
        if counts[&public] > 1 || reserved(&public, roots, &input_locals[name.input]) {
            let parts = name.identity.iter().map(String::as_str).collect::<Vec<_>>();
            public = names
                .name("param", &public, &parts)?
                .trim_start_matches("param_")
                .to_owned();
        }
        if !claimed.insert(public.clone()) {
            return Err(format!("duplicate public Kotlin parameter `{public}`"));
        }
        if mappings[name.input]
            .insert(name.wire, format!("`{public}`"))
            .is_some()
        {
            return Err("duplicate Kotlin input parameter binding".into());
        }
    }
    for (code, mapping) in codes.iter_mut().zip(&mappings) {
        for parameter in &mut code.parameters {
            let (wire, ty) = parameter
                .split_once(": ")
                .ok_or("invalid Kotlin input parameter")?;
            *parameter = format!(
                "{}: {}",
                mapping
                    .get(wire)
                    .ok_or("missing Kotlin public parameter name")?,
                rewrite(ty, mapping)
            );
        }
        for text in code
            .arguments
            .iter_mut()
            .chain(&mut code.prepared_arguments)
            .chain(&mut code.locks)
            .chain(&mut code.consumed)
            .chain(&mut code.lock_groups)
            .chain(code.setup.iter_mut().map(|b| &mut b.expression))
        {
            *text = rewrite(text, mapping);
        }
    }
    Ok(())
}

// Reserve unqualified identifiers in this input's snippets, including lambda and
// loop bindings and function calls. Member names cannot capture a parameter.
fn identifiers(text: &str) -> BTreeSet<String> {
    let empty = BTreeMap::new();
    let mut lexer = Lexer::new(text, &empty);
    lexer.code(false);
    lexer.words.into_iter().collect()
}

fn rewrite(text: &str, mapping: &BTreeMap<String, String>) -> String {
    let mut lexer = Lexer::new(text, mapping);
    lexer.code(false);
    lexer.out
}

/// A small lexical rewrite, rather than substring replacement: member names,
/// comments and string/character literals are not parameter references. Kotlin
/// string-template expressions are code and must be rewritten as such.
struct Lexer<'a> {
    text: &'a str,
    mapping: &'a BTreeMap<String, String>,
    at: usize,
    out: String,
    words: Vec<String>,
}
impl<'a> Lexer<'a> {
    fn new(text: &'a str, mapping: &'a BTreeMap<String, String>) -> Self {
        Self {
            text,
            mapping,
            at: 0,
            out: String::new(),
            words: vec![],
        }
    }
    fn rest(&self) -> &'a str {
        &self.text[self.at..]
    }
    fn copy(&mut self, count: usize) {
        self.out.push_str(&self.text[self.at..self.at + count]);
        self.at += count;
    }
    fn identifier(&mut self) -> &'a str {
        let start = self.at;
        while self
            .rest()
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            self.at += self.rest().chars().next().unwrap().len_utf8();
        }
        &self.text[start..self.at]
    }
    fn code(&mut self, interpolation: bool) {
        let mut braces = 0;
        let mut member = false;
        while !self.rest().is_empty() {
            if self.rest().starts_with("..") {
                let n = if self.rest().starts_with("..<") { 3 } else { 2 };
                self.copy(n);
                member = false;
                continue;
            }
            if self.rest().starts_with("//") {
                let n = self.rest().find('\n').unwrap_or(self.rest().len());
                self.copy(n);
                continue;
            }
            if self.rest().starts_with("/*") {
                self.copy(2);
                let mut depth = 1;
                while depth != 0 && !self.rest().is_empty() {
                    if self.rest().starts_with("/*") {
                        depth += 1;
                        self.copy(2);
                    } else if self.rest().starts_with("*/") {
                        depth -= 1;
                        self.copy(2);
                    } else {
                        self.copy(self.rest().chars().next().unwrap().len_utf8());
                    }
                }
                continue;
            }
            let c = self.rest().chars().next().unwrap();
            if c == '"' {
                self.string();
                member = false;
                continue;
            }
            if c == '\'' {
                self.copy(1);
                while !self.rest().is_empty() {
                    let c = self.rest().chars().next().unwrap();
                    self.copy(c.len_utf8());
                    if c == '\\' && !self.rest().is_empty() {
                        self.copy(self.rest().chars().next().unwrap().len_utf8());
                    } else if c == '\'' {
                        break;
                    }
                }
                member = false;
                continue;
            }
            if c == '`' {
                let Some(n) = self.rest()[1..].find('`').map(|n| n + 2) else {
                    self.copy(self.rest().len());
                    return;
                };
                let word = &self.rest()[1..n - 1];
                if !member {
                    self.words.push(word.to_owned());
                }
                if !member && self.mapping.contains_key(word) {
                    self.out.push_str(&self.mapping[word]);
                    self.at += n;
                } else {
                    self.copy(n);
                }
                member = false;
                continue;
            }
            if c.is_alphabetic() || c == '_' {
                let word = self.identifier();
                if !member {
                    self.words.push(word.to_owned());
                }
                if !member {
                    self.out
                        .push_str(self.mapping.get(word).map_or(word, String::as_str));
                } else {
                    self.out.push_str(word);
                }
                member = false;
                continue;
            }
            self.copy(c.len_utf8());
            if c == '{' {
                braces += 1;
            }
            if c == '}' {
                if interpolation && braces == 0 {
                    return;
                }
                braces -= 1;
            }
            if !c.is_whitespace() {
                member = c == '.';
            }
        }
    }
    fn string(&mut self) {
        let delimiter = if self.rest().starts_with("\"\"\"") {
            "\"\"\""
        } else {
            "\""
        };
        self.copy(delimiter.len());
        while !self.rest().is_empty() {
            if self.rest().starts_with(delimiter) {
                self.copy(delimiter.len());
                return;
            }
            let c = self.rest().chars().next().unwrap();
            if c == '\\' && delimiter.len() == 1 {
                self.copy(1);
                if !self.rest().is_empty() {
                    self.copy(self.rest().chars().next().unwrap().len_utf8());
                }
            } else if self.rest().starts_with("${") {
                self.copy(2);
                self.code(true);
            } else if c == '$' {
                self.copy(1);
                if self
                    .rest()
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                {
                    let word = self.identifier();
                    if let Some(name) = self.mapping.get(word) {
                        self.out.push_str(&format!("{{{name}}}"));
                    } else {
                        self.out.push_str(word);
                    }
                }
            } else {
                self.copy(c.len_utf8());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builder(source: &str) -> JniGenBuilder {
        JniGen::builder()
            .flat(
                Bindings::parse(
                    syn::parse_file(source)
                        .unwrap()
                        .items
                        .into_iter()
                        .map(|i| (i, Default::default())),
                )
                .unwrap(),
            )
            .set_package_prefix("test")
    }

    #[test]
    fn unnamed_repeated_paths_get_distinct_fallback_names() {
        let path =
            prebindgen_declarations::ComponentPath(vec![ComponentSegment::Named("value0".into())]);
        let mut codes = vec![KotlinInput {
            parameters: vec!["a: Long".into(), "b: Long".into()],
            parameter_paths: vec![path.clone(), path],
            arguments: vec!["a".into(), "b".into()],
            ..Default::default()
        }];
        apply(&["input".into()], &mut codes, &BTreeSet::new()).unwrap();
        assert_ne!(codes[0].parameters[0], codes[0].parameters[1]);
        for parameter in &codes[0].parameters {
            assert!(parameter.starts_with("`inputValue0_"));
        }
    }

    #[test]
    fn relocated_runtime_reserves_foreign_class_and_callback_packages() {
        let source =
            "pub struct Row{pub value:i64}pub fn call(foreign:i64,callback:impl Fn(Row))->Row{}";
        let output = builder(source)
            .set_package_prefix("foreign")
            .configure(|flat| {
                flat.metadata.package = "native".into();
                Ok(())
            })
            .package(package!().class(data_class!(Row)).fun(fun!(call)))
            .build()
            .unwrap();
        let functions = &output.kotlin["foreign/Functions.kt"];
        assert!(functions.contains("`foreign_"), "{functions}");
        assert!(functions.contains("foreign.Row("), "{functions}");
    }

    #[test]
    fn naming_paths_are_absent_or_complete() {
        let mut code = KotlinInput {
            parameters: vec!["a: Long".into(), "b: Long".into()],
            ..Default::default()
        };
        code.complete_parameter_paths().unwrap();
        assert_eq!(code.parameter_paths.len(), 2);
        code.parameter_paths.pop();
        assert!(code.complete_parameter_paths().is_err());
    }

    #[test]
    fn identifiers_cover_all_generated_binding_forms() {
        let words = identifiers("val (`first`, second) = pair; rows.forEachIndexed { index, row -> for (handle in row) { require(handle != null); value.p0_w1 } }");
        for name in ["first", "second", "index", "row", "handle", "require"] {
            assert!(words.contains(name), "{name}");
        }
        assert!(!words.contains("p0_w1"));
    }

    #[test]
    fn direct_parameters_and_callbacks_use_source_names() {
        let output = builder("pub fn send(packet_id:u64, callback:impl Fn(i64))->i64{}")
            .package(package!().fun(fun!(send)))
            .build()
            .unwrap();
        let functions = &output.kotlin["test/Functions.kt"];
        assert!(
            functions.contains("`packetId`: ULong, `callback`: test.i64Callback?"),
            "{functions}"
        );
        assert!(functions.contains("`packetId`.toLong()"));
        assert!(functions.contains("`callback`.run("));
        assert!(output.kotlin["test/JNINative.kt"].contains("p0_w1: Long, p1_callback: Any?"));
    }

    #[test]
    fn constructor_fields_optional_presence_and_bulk_keep_source_paths() {
        let source="pub struct Row{pub number:i64,pub label:String}pub fn row_new(number:i64,label:String)->Row{}pub fn use_row(row:Option<Row>)->i64{}";
        for bulk in [false, true] {
            let expansion = expand_param!(Row).variant(fun!(row_new));
            let expansion = if bulk { expansion.bulk() } else { expansion };
            let output = builder(source)
                .package(package!().fun(fun!(use_row).expand_param("row", expansion)))
                .build()
                .unwrap();
            let functions = &output.kotlin["test/Functions.kt"];
            assert!(functions.contains("`rowPresent`: Boolean"), "{functions}");
            assert!(functions.contains("`rowNumber`: Long"), "{functions}");
            assert!(functions.contains("`rowLabel`: String?"), "{functions}");
        }
    }

    #[test]
    fn record_objects_and_lists_keep_the_outer_parameter_name() {
        let source="pub struct Row{pub number:i64,pub label:String}pub fn rows(records:Vec<Row>,packet:Row)->i64{}";
        let output = builder(source)
            .package(package!().class(data_class!(Row)).fun(fun!(rows)))
            .build()
            .unwrap();
        let functions = &output.kotlin["test/Functions.kt"];
        assert!(
            functions.contains("`records`: kotlin.collections.List<test.Row>"),
            "{functions}"
        );
        assert!(functions.contains("`packet`: test.Row"), "{functions}");
    }

    #[test]
    fn collisions_have_stable_source_derived_suffixes_and_keywords_are_escaped() {
        fn names(source: &str) -> Vec<String> {
            let output = builder(source)
                .package(package!().fun(fun!(call)))
                .build()
                .unwrap();
            let signature = output.kotlin["test/Functions.kt"]
                .lines()
                .find(|s| s.starts_with("public fun"))
                .unwrap();
            signature
                .split_once('(')
                .unwrap()
                .1
                .split(", ")
                .take(5)
                .map(|p| p.split_once(": ").unwrap().0.to_owned())
                .collect()
        }
        let a = names(
            "pub fn call(foo_bar:i64,fooBar:i64,on_error:i64,return_capture:i64,r#when:i64){}",
        );
        let b = names(
            "pub fn call(fooBar:i64,foo_bar:i64,on_error:i64,return_capture:i64,r#when:i64){}",
        );
        assert!(a[0].starts_with("`fooBar_") && a[1].starts_with("`fooBar_"));
        assert_ne!(a[0], a[1]);
        assert_eq!(a[0], b[1]);
        assert_eq!(a[1], b[0]);
        assert!(a[2].starts_with("`onError_"));
        assert!(a[3].starts_with("`returnCapture_"));
        assert_eq!(a[4], "`when`");
    }

    #[test]
    fn lexical_rewrite_preserves_members_literals_comments_and_templates() {
        let mapping = BTreeMap::from([
            ("p0_w1".into(), "`when`".into()),
            ("p1_w1".into(), "`value`".into()),
        ]);
        assert_eq!(
            rewrite("p0_w1.raw + row.p0_w1 + row.`p0_w1` + p1_w1", &mapping),
            "`when`.raw + row.p0_w1 + row.`p0_w1` + `value`"
        );
        assert_eq!(
            rewrite(
                "/* p0_w1 /* p1_w1 */ */ p0_w1 // p1_w1\n\"p0_w1 $p0_w1 ${p1_w1.raw}\"",
                &mapping
            ),
            "/* p0_w1 /* p1_w1 */ */ `when` // p1_w1\n\"p0_w1 ${`when`} ${`value`.raw}\""
        );
        assert_eq!(
            rewrite("\"\"\"p0_w1 $p0_w1\"\"\" + 'x' + `p1_w1`", &mapping),
            "\"\"\"p0_w1 ${`when`}\"\"\" + 'x' + `value`"
        );
        assert_eq!(
            rewrite("0..p0_w1 + 1..<p1_w1", &mapping),
            "0..`when` + 1..<`value`"
        );
        assert_eq!(rewrite("`p0_w1", &mapping), "`p0_w1");
    }

    #[test]
    fn nested_optional_and_source_presence_field_have_distinct_paths() {
        let output=builder("pub struct Row{pub present:bool,pub number:i64}pub fn call(row:Option<Row>,nested:Option<Option<i64>>)->i64{}")
            .package(package!().fun(fun!(call).input("row", Input::Optional(Box::new(Input::Fields(vec![Input::Leaf,Input::Leaf]))))))
            .build().unwrap();
        let functions = &output.kotlin["test/Functions.kt"];
        let signature = functions
            .lines()
            .find(|s| s.starts_with("public fun"))
            .unwrap();
        assert_eq!(signature.matches("`rowPresent_").count(), 2, "{signature}");
        assert!(
            signature.contains("`nestedPresent`: Boolean"),
            "{signature}"
        );
        assert!(signature.contains("`nestedSome`: Long?"), "{signature}");
    }

    #[test]
    fn unrelated_callback_locals_do_not_rename_existing_parameters() {
        for source in [
            "pub type Handle=String;pub fn call(owner:i64){}",
            "pub type Handle=String;pub fn call(owner:i64, callback:impl Fn(Handle)){}",
        ] {
            let output = builder(source)
                .package(package!().class(ptr_class!(Handle)).fun(fun!(call)))
                .build()
                .unwrap();
            assert!(output.kotlin["test/Functions.kt"].contains("`owner`: Long"));
        }
    }
}
