use super::*;

fn render_as_raw(spec: IfaceSpec) -> String {
    kt::KtFile::new(&spec.package)
        .decl(spec.to_as_raw_fun())
        .render()
}

#[test]
fn as_raw_adapter_is_multiline_even_when_short() {
    let spec = IfaceSpec {
        package: "io.test".to_string(),
        name: "ThingCallback".to_string(),
        type_params: vec![],
        params: vec![IfaceParam {
            name: "handle".to_string(),
            typed: kt::KtType::cls("io.test.Thing"),
            raw: kt::KtType::long(),
            wrap: WrapKind::Handle("io.test.Thing".to_string()),
        }],
        ret: kt::KtType::unit(),
        descr: "(J)V".to_string(),
        typed_groups: Vec::new(),
    };

    let src = render_as_raw(spec);
    assert!(
        src.contains(
            "public fun ThingCallback.asRaw(): ThingCallbackRaw =\n    \
                 ThingCallbackRaw {\n        \
                 handle ->\n        \
                 run(\n            \
                 Thing(handle)\n        \
                 )\n    \
                 }"
        ),
        "{src}"
    );
}

#[test]
fn as_raw_adapter_breaks_wide_lambda_params_and_run_args() {
    let spec = IfaceSpec {
        package: "io.test".to_string(),
        name: "ReplyCallback".to_string(),
        type_params: vec![],
        params: vec![
            IfaceParam {
                name: "replierZid".to_string(),
                typed: kt::KtType::cls("io.test.ZenohId").nullable(),
                raw: kt::KtType::byte_array().nullable(),
                wrap: WrapKind::Blob("io.test.ZenohId".to_string()),
            },
            IfaceParam::same("replierEid".to_string(), kt::KtType::int()),
            IfaceParam::same("isOk".to_string(), kt::KtType::boolean()),
            IfaceParam {
                name: "sample__keyExpr".to_string(),
                typed: kt::KtType::cls("io.test.KeyExpr").nullable(),
                raw: kt::KtType::long().nullable(),
                wrap: WrapKind::Handle("io.test.KeyExpr".to_string()),
            },
            IfaceParam {
                name: "sample__payload".to_string(),
                typed: kt::KtType::cls("io.test.ZBytes").nullable(),
                raw: kt::KtType::long().nullable(),
                wrap: WrapKind::Handle("io.test.ZBytes".to_string()),
            },
        ],
        ret: kt::KtType::unit(),
        descr: "([BIZLjava/lang/Long;Ljava/lang/Long;)V".to_string(),
        typed_groups: Vec::new(),
    };

    let src = render_as_raw(spec);
    assert!(
        src.contains("public fun ReplyCallback.asRaw(): ReplyCallbackRaw =\n"),
        "{src}"
    );
    assert!(src.contains("    ReplyCallbackRaw {\n"), "{src}");
    assert!(src.contains("        replierZid,\n"), "{src}");
    assert!(src.contains("        sample__payload ->\n"), "{src}");
    assert!(src.contains("        run(\n"), "{src}");
    assert!(
        src.contains("            replierZid?.let { ZenohId(it) },\n"),
        "{src}"
    );
    assert!(
        src.contains("            sample__payload?.let { ZBytes(it) }\n"),
        "{src}"
    );
}
