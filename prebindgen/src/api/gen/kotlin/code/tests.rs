use super::*;

#[test]
fn reindent_nested_blocks() {
    let flat = "run {\nval __locks = ArrayList<NativeHandle>()\nwithSortedHandleLocks(__locks) {\nval p = x.ptr\nJNINative.call(p)\n}\n}";
    let mut out = String::new();
    Code::raw_reindent(flat).render(0, &mut out);
    assert_eq!(
            out,
            "run {\n    val __locks = ArrayList<NativeHandle>()\n    withSortedHandleLocks(__locks) {\n        val p = x.ptr\n        JNINative.call(p)\n    }\n}\n"
        );
}

#[test]
fn reindent_ignores_braces_in_strings_and_comments() {
    let flat = "val s = \"{ not a brace }\"\n// also { not } counted\nif (x) {\nf()\n}";
    let mut out = String::new();
    Code::raw_reindent(flat).render(0, &mut out);
    assert_eq!(
        out,
        "val s = \"{ not a brace }\"\n// also { not } counted\nif (x) {\n    f()\n}\n"
    );
}

#[test]
fn reindent_single_line_braces_balance() {
    // Balanced one-liners (lambdas) must not change the level.
    let flat = "val __cap = { __je: String? -> __cap_je = __je }\nval after = 1";
    let mut out = String::new();
    Code::raw_reindent(flat).render(0, &mut out);
    assert_eq!(
        out,
        "val __cap = { __je: String? -> __cap_je = __je }\nval after = 1\n"
    );
}

#[test]
fn reindent_try_finally_continuation() {
    let flat = "try {\nf()\n} finally {\ng()\n}";
    let mut out = String::new();
    Code::raw_reindent(flat).render(0, &mut out);
    assert_eq!(out, "try {\n    f()\n} finally {\n    g()\n}\n");
}

#[test]
fn blk_renders_nested() {
    let c = Code::new()
        .line("var x = 0")
        .blk("run {", |c| c.line("x += 1"));
    let mut out = String::new();
    c.render(1, &mut out);
    assert_eq!(out, "    var x = 0\n    run {\n        x += 1\n    }\n");
}

fn reindent_wrapped(flat: &str) -> String {
    let mut out = String::new();
    Code::raw_reindent_wrapped(flat).render(0, &mut out);
    out
}

#[test]
fn wrap_leaves_short_lines_untouched() {
    // Identical to the plain reindenter when nothing exceeds the budget.
    let flat = "run {\nval p = x.ptr\nJNINative.call(p)\n}";
    assert_eq!(
        reindent_wrapped(flat),
        "run {\n    val p = x.ptr\n    JNINative.call(p)\n}\n"
    );
}

#[test]
fn wrap_breaks_long_call_one_arg_per_line() {
    let flat = format!(
        "JNINative.zCall({})",
        (0..12)
            .map(|i| format!("argumentNumber{i}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let out = reindent_wrapped(&flat);
    assert!(out.starts_with("JNINative.zCall(\n"), "{out}");
    assert!(out.contains("    argumentNumber0,\n"), "{out}");
    assert!(out.contains("    argumentNumber11,\n"), "{out}");
    assert!(out.ends_with(")\n"), "{out}");
}

#[test]
fn wrap_breaks_nested_call_and_callback_lambda() {
    // The outer call breaks first; the lambda arg (which starts with `{`)
    // then breaks its params and recursively its `callback(...)` body.
    let flat = "JNINative.zGet(handlePointerValue, selectorParameterValue, anotherParameterValue, { firstLeafValue: ByteArray?, secondLeafValue: Int, thirdLeafValue: Boolean -> callback(firstLeafValue, secondLeafValue, thirdLeafValue, fourthLeafValueGoesHere, fifthLeafValueGoesHere) }, onClose, __cap)";
    let out = reindent_wrapped(flat);
    assert!(out.starts_with("JNINative.zGet(\n"), "{out}");
    // lambda opens on its own arg line, params broken out, then `->`.
    assert!(out.contains("    {\n"), "{out}");
    assert!(
        out.contains("        firstLeafValue: ByteArray?,\n"),
        "{out}"
    );
    assert!(out.contains("        ->\n"), "{out}");
    // nested callback(...) call is itself broken.
    assert!(out.contains("        callback(\n"), "{out}");
    assert!(out.contains("            firstLeafValue,\n"), "{out}");
    assert!(out.contains("    },\n"), "{out}");
    assert!(out.ends_with(")\n"), "{out}");
}

#[test]
fn wrap_skips_control_flow_keyword_paren() {
    // `if (...)` must not be treated as a call (no trailing comma in the
    // condition); the real `onError(...)` call is the one that breaks.
    let flat = "if (someConditionFlagValue) return onError(firstErrorArgumentValue, secondErrorArgumentValue, thirdErrorArgumentValue)";
    let out = reindent_wrapped(flat);
    assert!(
        out.starts_with("if (someConditionFlagValue) return onError(\n"),
        "{out}"
    );
    assert!(!out.contains("someConditionFlagValue,"), "{out}");
    assert!(out.contains("    firstErrorArgumentValue,\n"), "{out}");
    assert!(out.trim_end().ends_with(")"), "{out}");
}

#[test]
fn wrap_breaks_prefixed_capture_lambda() {
    let flat = "val __cap = { firstError: String?, secondError: String?, thirdError: String? -> recordedFailureFlag = true; capturedFirst = firstError }";
    let out = reindent_wrapped(flat);
    assert!(out.starts_with("val __cap = {\n"), "{out}");
    assert!(out.contains("    firstError: String?,\n"), "{out}");
    assert!(out.contains("    ->\n"), "{out}");
    assert!(out.ends_with("}\n"), "{out}");
}

#[test]
fn wrap_does_not_split_generic_or_string_commas() {
    let flat = "JNINative.zRegister(longHandlePointerValue, mapOf<String, Int>(), \"a, b, c is one string literal argument here\")";
    let out = reindent_wrapped(flat);
    // Three arguments only — the commas inside `<String, Int>` and the
    // string literal must not split.
    assert!(out.contains("    mapOf<String, Int>(),\n"), "{out}");
    assert!(
        out.contains("    \"a, b, c is one string literal argument here\",\n"),
        "{out}"
    );
}
