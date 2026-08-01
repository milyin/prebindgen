//! The describe/built split is only worth having if something checks it.

/// A built `JniGen` has no route back into being described.
///
/// The sibling of `a_built_registry_exposes_no_mutation`, and it exists for the
/// same reason at one remove. That test guards `Registry`, which a build script
/// used to build itself — the `RegistryBuilder` → `Registry` split *was* the
/// enforcement, and it lived in the caller's hands. Once
/// `JniGen::builder().source(..).build()` moved both phases inside the generator,
/// the guarantee became this crate's to keep, and there was nothing keeping it:
/// `JniGen` held its `JniGenBuilder` whole, mutators and all.
///
/// So [`Declarations`](crate::lang::Declarations) exists, and what this checks is
/// that it stays what it is: the type a built binding keeps, with **no `&mut self`
/// method at all** — not "the obvious ones were removed".
///
/// Reads the source with **all** whitespace stripped, which makes a multi-line
/// signature indistinguishable from a one-line one. That is not defensive
/// styling: `Registry::supply` survived two commits claiming it was gone because
/// the check for it was a single-line grep and its signature spanned lines.
///
/// `builder.rs` and `config.rs` are skipped, exactly as the registry test skips
/// `declare.rs` — `JniGenBuilder` is *meant* to be mutable, and that is the whole
/// point of it being a different type.
#[test]
fn a_built_jnigen_exposes_no_mutation() {
    let mut offenders: Vec<String> = Vec::new();

    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src/api/lang/jnigen/jni");
    let mut dirs = vec![std::path::PathBuf::from(root)];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).expect("jnigen module dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                // `tests/` is this file's own home, and a fixture may hold a
                // builder mutably on purpose.
                if path.file_name().is_some_and(|n| n != "tests") {
                    dirs.push(path);
                }
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let name = path
                .file_name()
                .expect("file name")
                .to_string_lossy()
                .to_string();
            if name == "builder.rs" || name == "config.rs" {
                continue;
            }
            let src = std::fs::read_to_string(&path).expect("read source");
            let bare: String = src.chars().filter(|c| !c.is_whitespace()).collect();

            // Walk the `impl` blocks, and only complain inside the two types a
            // built binding keeps. An `impl JniGenBuilder` in one of these files
            // is the describing half and is allowed to mutate.
            let mut rest = bare.as_str();
            while let Some(at) = rest.find("impl") {
                rest = &rest[at + "impl".len()..];
                let sealed = rest.starts_with("Declarations{")
                    || rest.starts_with("JniGen{")
                    || rest.starts_with("PrebindgenforDeclarations{");
                // The block's own text, up to the next `impl` — good enough,
                // because a nested `impl` would start its own scan anyway.
                let end = rest.find("impl").unwrap_or(rest.len());
                if sealed {
                    let block = &rest[..end];
                    let mut scan = block;
                    while let Some(f) = scan.find("fn") {
                        scan = &scan[f + "fn".len()..];
                        let Some(open) = scan.find('(') else { break };
                        if scan[open..].starts_with("(&mutself") {
                            offenders.push(format!("{name}: fn {}(&mut self …)", &scan[..open]));
                        }
                    }
                }
                rest = &rest[end..];
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "a built `JniGen` and its `Declarations` must be read-only; found mutation: {offenders:#?}"
    );
}

/// The two `RefCell`s on `Declarations` are memos, and are named here so a third
/// one has to be argued for rather than merely added.
///
/// Interior mutability is not a hole in the split: `iface_specs` and `fn_plans`
/// cache work derived from `(declarations, registry)`, both of which are frozen by
/// the time anything reads them. Caching a pure function of frozen inputs is not
/// re-declaring anything. A `RefCell` holding *declaration state* would be, and
/// that is what this catches.
#[test]
fn declarations_interior_mutability_is_only_the_two_memos() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/api/lang/jnigen/jni/mod.rs"
    ))
    .expect("read mod.rs");

    let decls = {
        let start = src.find("pub struct Declarations {").expect("Declarations");
        let rest = &src[start..];
        let end = rest.find("\n}\n").expect("struct end");
        &rest[..end]
    };

    // Whitespace-stripped, for the reason the sibling test gives: `iface_specs`
    // wraps onto a second line, so a line-based scan sees a bare `RefCell<..>`
    // with no field name attached — which is how this check first "passed" on a
    // field it could not identify.
    let bare: String = decls.chars().filter(|c| !c.is_whitespace()).collect();

    let mut named: Vec<String> = Vec::new();
    let mut rest = bare.as_str();
    while let Some(at) = rest.find("RefCell") {
        // The field name is the ident before the `:` that introduces this type.
        let decl_start = rest[..at].rfind(',').map(|i| i + 1).unwrap_or(0);
        let field = &rest[decl_start..at];
        named.push(field.trim_end_matches(|c: char| c != ':').to_string());
        rest = &rest[at + "RefCell".len()..];
    }

    assert_eq!(
        named.len(),
        2,
        "expected exactly the two memo fields, found: {named:#?}"
    );
    assert!(
        named.iter().any(|f| f.contains("iface_specs:"))
            && named.iter().any(|f| f.contains("fn_plans:")),
        "the two memos must be `iface_specs` and `fn_plans`, found: {named:#?}"
    );
}
