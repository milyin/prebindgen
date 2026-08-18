use kotlin_codegen::{KtFile, KtType};

use super::*;

fn render_as_raw(spec: IfaceSpec) -> String {
    KtFile::new(&spec.package)
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
            typed: KtType::cls("io.test.Thing"),
            raw: KtType::long(),
            wrap: WrapKind::Handle {
                fqn: "io.test.Thing".to_string(),
                niche_sentinel: None,
            },
        }],
        ret: KtType::unit(),
        descr: "(J)V".to_string(),
        typed_groups: Vec::new(),
        kdoc: None,
    };

    let src = render_as_raw(spec);
    assert!(
        src.contains(
            "@JvmSynthetic\ninternal fun ThingCallback.asRaw(): ThingCallbackRaw =\n    \
                 ThingCallbackRaw {\n        \
                 handle ->\n        \
                 run(\n            \
                 Thing.fromRawPtr(handle)\n        \
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
                typed: KtType::cls("io.test.ZenohId").nullable(),
                raw: KtType::long().nullable(),
                wrap: WrapKind::Handle {
                    fqn: "io.test.ZenohId".to_string(),
                    niche_sentinel: None,
                },
            },
            IfaceParam::same("replierEid".to_string(), KtType::int()),
            IfaceParam::same("isOk".to_string(), KtType::boolean()),
            IfaceParam {
                name: "sample__keyExpr".to_string(),
                typed: KtType::cls("io.test.KeyExpr").nullable(),
                raw: KtType::long().nullable(),
                wrap: WrapKind::Handle {
                    fqn: "io.test.KeyExpr".to_string(),
                    niche_sentinel: None,
                },
            },
            IfaceParam {
                name: "sample__payload".to_string(),
                typed: KtType::cls("io.test.ZBytes").nullable(),
                raw: KtType::long().nullable(),
                wrap: WrapKind::Handle {
                    fqn: "io.test.ZBytes".to_string(),
                    niche_sentinel: None,
                },
            },
        ],
        ret: KtType::unit(),
        descr: "(Ljava/lang/Long;IZLjava/lang/Long;Ljava/lang/Long;)V".to_string(),
        typed_groups: Vec::new(),
        kdoc: None,
    };

    let src = render_as_raw(spec);
    assert!(
        src.contains("@JvmSynthetic\ninternal fun ReplyCallback.asRaw(): ReplyCallbackRaw =\n"),
        "{src}"
    );
    assert!(src.contains("    ReplyCallbackRaw {\n"), "{src}");
    assert!(src.contains("        replierZid,\n"), "{src}");
    assert!(src.contains("        sample__payload ->\n"), "{src}");
    assert!(src.contains("        run(\n"), "{src}");
    assert!(
        src.contains("            replierZid?.let { ZenohId.fromRawPtr(it) },\n"),
        "{src}"
    );
    assert!(
        src.contains("            sample__payload?.let { ZBytes.fromRawPtr(it) }\n"),
        "{src}"
    );
}
