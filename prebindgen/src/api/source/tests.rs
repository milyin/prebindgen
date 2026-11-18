use super::*;
use crate::api::{record::RecordKind, utils::jsonl::write_to_jsonl_file};

fn capture_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(CRATE_NAME_FILE), "capture-fixture").unwrap();
    dir
}

fn captured_value(file: &str, line: usize, ty: &str, cfg: Option<&str>) -> Record {
    Record::new(
        RecordKind::Function,
        "value".into(),
        format!("pub fn value() -> {ty} {{}}"),
        SourceLocation {
            file: file.into(),
            line,
            column: 5,
            crate_name: None,
        },
        cfg.map(str::to_owned),
    )
}

#[test]
fn conflicting_capture_records_fail_source_construction() {
    let dir = capture_dir();
    let cfg = Some(r#"feature = "experimental""#);
    let original = captured_value("src/first.rs", 11, "i32", cfg);
    let conflict = captured_value("src/second.rs", 22, "u64", cfg);
    write_to_jsonl_file(dir.path().join("default_1_20.jsonl"), &[original]).unwrap();
    write_to_jsonl_file(dir.path().join("default_2_30.jsonl"), &[conflict]).unwrap();

    let err = std::panic::catch_unwind(|| drop(Source::new(dir.path())))
        .expect_err("Source::new accepted conflicting capture records");
    let message = err.downcast_ref::<String>().expect("conflict diagnostic");
    for context in [
        "Conflicting prebindgen records",
        "`value`",
        "group `default`",
        r#"cfg: feature = "experimental""#,
        "src/first.rs:11:5",
        "src/second.rs:22:5",
        "default_1_20.jsonl",
        "default_2_30.jsonl",
    ] {
        assert!(
            message.contains(context),
            "missing {context:?} in {message}"
        );
    }
}

#[test]
fn identical_capture_records_are_read_once() {
    let dir = capture_dir();
    let record = captured_value("src/source.rs", 4, "i32", None);
    write_to_jsonl_file(dir.path().join("default_1_20.jsonl"), &[&record, &record]).unwrap();
    write_to_jsonl_file(dir.path().join("default_2_30.jsonl"), &[record]).unwrap();

    let source = Source::new(dir.path());
    let locations: Vec<_> = source
        .items_in_groups(&["default"])
        .filter_map(|(item, location)| matches!(item, syn::Item::Fn(_)).then_some(location))
        .collect();
    assert_eq!(
        locations,
        [SourceLocation {
            file: "src/source.rs".into(),
            line: 4,
            column: 5,
            crate_name: Some("capture-fixture".into()),
        }]
    );
}

#[test]
fn capture_identity_keeps_cfg_variants_and_groups_separate() {
    let dir = capture_dir();
    fs::write(dir.path().join(FEATURES_FILE), "selected\n").unwrap();
    let selected = captured_value("src/selected.rs", 4, "i32", Some(r#"feature = "selected""#));
    let disabled = captured_value(
        "src/disabled.rs",
        4,
        "u64",
        Some(r#"not(feature = "selected")"#),
    );
    let alternate = captured_value("src/alternate.rs", 4, "u8", selected.cfg.as_deref());
    write_to_jsonl_file(dir.path().join("default_1_20.jsonl"), &[selected, disabled]).unwrap();
    write_to_jsonl_file(dir.path().join("other_2_30.jsonl"), &[alternate]).unwrap();

    let source = Source::new(dir.path());
    for (group, file) in [
        ("default", "src/selected.rs"),
        ("other", "src/alternate.rs"),
    ] {
        let files: Vec<_> = source
            .items_in_groups(&[group])
            .filter_map(|(item, location)| {
                matches!(item, syn::Item::Fn(_)).then_some(location.file)
            })
            .collect();
        assert_eq!(files, [file]);
    }
}

fn function_origins(source: &Source, group: &str) -> Vec<(String, String)> {
    let mut functions: Vec<_> = source
        .items_in_groups(&[group])
        .filter_map(|(item, loc)| match item {
            syn::Item::Fn(f) => Some((f.sig.ident.to_string(), loc.file)),
            _ => None,
        })
        .collect();
    functions.sort();
    functions
}

#[test]
fn legacy_collision_suffix_loads_records_in_the_first_underscore_group() {
    let dir = capture_dir();
    let first = captured_value("src/api.rs", 4, "i32", None);
    write_to_jsonl_file(dir.path().join("api_200_1200.jsonl"), &[&first]).unwrap();
    let mut second = captured_value("src/api.rs", 8, "u64", None);
    second.name = "other".into();
    second.content = "pub fn other() -> u64 {}".into();
    write_to_jsonl_file(dir.path().join("api_200_1200_3000.jsonl"), &[first, second]).unwrap();

    let source = Source::new(dir.path());
    assert_eq!(
        function_origins(&source, "api"),
        [
            ("other".into(), "src/api.rs".into()),
            ("value".into(), "src/api.rs".into())
        ]
    );
    assert!(function_origins(&source, "api_200").is_empty());
}

#[test]
fn legacy_noncanonical_capture_files_and_empty_groups_load() {
    let dir = capture_dir();
    let manual = captured_value("src/manual.rs", 4, "i32", None);
    write_to_jsonl_file(dir.path().join("api_manual.jsonl"), &[manual]).unwrap();
    let mut numeric = captured_value("src/numeric.rs", 4, "i32", None);
    numeric.name = "numeric".into();
    numeric.content = "pub fn numeric() -> i32 {}".into();
    write_to_jsonl_file(dir.path().join("api_0200_1200.jsonl"), &[numeric]).unwrap();
    fs::write(dir.path().join("notes_cache.jsonl"), "").unwrap();

    let source = Source::new(dir.path());
    assert_eq!(
        function_origins(&source, "api"),
        [
            ("numeric".into(), "src/numeric.rs".into()),
            ("value".into(), "src/manual.rs".into())
        ]
    );
    assert!(function_origins(&source, "notes").is_empty());
}

#[test]
fn mixed_captures_use_each_records_own_group_provenance() {
    let dir = capture_dir();
    let legacy = captured_value("src/api.rs", 4, "i32", None);
    let tagged = |record: &Record, group: &str| {
        let mut value = serde_json::to_value(record).unwrap();
        value["capture_group"] = group.into();
        value.to_string()
    };
    let v2 = captured_value("src/v2.rs", 4, "u64", None);
    // The same file can hold old and new records. The repeat assigned to
    // "api" deduplicates, while the tagged "api_v2" remains separate.
    fs::write(
        dir.path().join("api_200_1200_3000.jsonl"),
        format!(
            "{}\n{}\n{}\n",
            legacy.to_jsonl_string().unwrap(),
            tagged(&legacy, "api"),
            tagged(&v2, "api_v2")
        ),
    )
    .unwrap();

    let source = Source::new(dir.path());
    assert_eq!(
        function_origins(&source, "api"),
        [("value".into(), "src/api.rs".into())]
    );
    assert_eq!(
        function_origins(&source, "api_v2"),
        [("value".into(), "src/v2.rs".into())]
    );
}

#[test]
fn anonymous_const_captures_keep_distinct_guards_and_deduplicate_repeats() {
    let dir = capture_dir();
    let guard = |line, condition| {
        Record::new(
            RecordKind::Const,
            "_".into(),
            format!("const _: () = assert!({condition});"),
            SourceLocation {
                file: "src/guards.rs".into(),
                line,
                column: 5,
                crate_name: None,
            },
            None,
        )
    };
    let first = guard(4, "FIRST");
    let second = guard(8, "SECOND");
    write_to_jsonl_file(
        dir.path().join("default_200_1200.jsonl"),
        &[&first, &second],
    )
    .unwrap();
    write_to_jsonl_file(dir.path().join("default_200_1201.jsonl"), &[first]).unwrap();

    let source = Source::builder(dir.path())
        .enable_feature_filtering(None::<&str>)
        .build();
    let mut guards: Vec<_> = source
        .items_in_groups(&["default"])
        .map(|(item, loc)| {
            let syn::Item::Const(guard) = item else {
                panic!("expected an anonymous const guard");
            };
            let syn::Expr::Macro(assertion) = *guard.expr else {
                panic!("expected an assert! guard");
            };
            assert_eq!(guard.ident, "_");
            assert!(assertion.mac.path.is_ident("assert"));
            (loc.line, assertion.mac.tokens.to_string())
        })
        .collect();
    guards.sort();
    assert_eq!(guards, [(4, "FIRST".into()), (8, "SECOND".into())]);
}
