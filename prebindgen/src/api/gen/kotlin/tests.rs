//! Golden-string tests: one per Kotlin construct the jnigen back-end emits.

use super::types::ImportSet;
use super::*;

fn body_of(src: &str) -> &str {
    // Strip banner + package + imports + the separating blank line.
    let mut rest = src;
    loop {
        let Some((line, tail)) = rest.split_once('\n') else { break };
        if line.starts_with("//")
            || line.starts_with("package ")
            || line.starts_with("import ")
            || line.is_empty()
        {
            rest = tail;
            // Stop skipping blank lines once the body starts; the single
            // separator blank is consumed by falling through one more loop.
            if line.is_empty() && !tail.starts_with("import ") && !tail.is_empty() {
                break;
            }
        } else {
            break;
        }
    }
    rest
}

#[test]
fn enum_class_with_from_int_companion() {
    let class = KtClass::new(
        ClassKind::Enum(vec![
            KtEnumEntry { name: "RED".into(), args: Some("0".into()) },
            KtEnumEntry { name: "GREEN".into(), args: Some("5".into()) },
            KtEnumEntry { name: "BLUE".into(), args: Some("6".into()) },
        ]),
        "Color",
    )
    .vis(Vis::Public)
    .kdoc("JVM-side surface for the native Rust `Color` enum.")
    .ctor_param(KtCtorParam::new("value", KtType::int()).val().vis(Vis::Public))
    .companion(
        KtClass::companion_object().vis(Vis::Public).member(
            KtFun::new("fromInt")
                .vis(Vis::Public)
                .annotation("JvmStatic")
                .param(KtParam::new("value", KtType::int()))
                .returns(KtType::cls("Color"))
                .expr_body(Code::new().line("entries.first { it.value == value }")),
        ),
    );
    let src = render::render_one(&class.into(), "io.test.jni");
    assert_eq!(
        body_of(&src),
        "\
/** JVM-side surface for the native Rust `Color` enum. */
public enum class Color(public val value: Int) {
    RED(0),
    GREEN(5),
    BLUE(6);

    public companion object {
        @JvmStatic
        public fun fromInt(value: Int): Color = entries.first { it.value == value }
    }
}
"
    );
}

#[test]
fn jvm_inline_value_class() {
    let class = KtClass::new(ClassKind::ValueInline, "ZenohId")
        .vis(Vis::Public)
        .ctor_param(KtCtorParam::new("bytes", KtType::byte_array()).val().vis(Vis::Public));
    let src = render::render_one(&class.into(), "io.test.jni");
    assert_eq!(
        body_of(&src),
        "\
@JvmInline
public value class ZenohId(public val bytes: ByteArray)
"
    );
}

#[test]
fn abstract_class_with_volatile_property_and_supertype() {
    let class = KtClass::new(ClassKind::Abstract, "NativeHandle")
        .vis(Vis::Public)
        .ctor_param(KtCtorParam::new("initialPtr", KtType::long()))
        .supertype(KtType::cls("AutoCloseable"), None)
        .member(
            KtProperty::var("ptr")
                .ty(KtType::long())
                .initializer("initialPtr")
                .vis(Vis::Internal)
                .annotation("Volatile"),
        )
        .member(
            KtFun::new("peek")
                .vis(Vis::Public)
                .returns(KtType::long())
                .expr_body(Code::new().line("ptr")),
        );
    let src = render::render_one(&class.into(), "io.test.jni");
    assert_eq!(
        body_of(&src),
        "\
public abstract class NativeHandle(initialPtr: Long) : AutoCloseable {
    @Volatile internal var ptr: Long = initialPtr

    public fun peek(): Long = ptr
}
"
    );
}

#[test]
fn typed_handle_subclass_with_ctor_args_supertype() {
    let class = KtClass::new(ClassKind::Plain, "ZThing")
        .vis(Vis::Public)
        .ctor_param(KtCtorParam::new("initialPtr", KtType::long()))
        .supertype(KtType::cls("io.test.jni.NativeHandle"), Some("initialPtr"))
        .member(
            KtFun::new("close")
                .annotation("Synchronized")
                .modifier("override")
                .body(Code::new().blk("if (ptr != 0L) {", |c| {
                    c.line("freePtr(ptr)").line("ptr = 0L")
                })),
        )
        .companion(KtClass::companion_object().member(
            KtFun::new("freePtr")
                .annotation("JvmStatic")
                .modifier("external")
                .param(KtParam::new("ptr", KtType::long())),
        ));
    let src = render::render_one(&class.into(), "io.test.jni.thing");
    assert_eq!(
        body_of(&src),
        "\
public class ZThing(initialPtr: Long) : NativeHandle(initialPtr) {
    @Synchronized
    override fun close() {
        if (ptr != 0L) {
            freePtr(ptr)
            ptr = 0L
        }
    }

    companion object {
        @JvmStatic
        external fun freePtr(ptr: Long)
    }
}
"
    );
    // Cross-package supertype produced an import.
    assert!(src.contains("import io.test.jni.NativeHandle"), "{src}");
}

#[test]
fn object_with_external_funs() {
    let obj = KtClass::object_("JNINative")
        .vis(Vis::Internal)
        .member(
            KtFun::new("zThingNew")
                .modifier("external")
                .param(KtParam::new("errorSink", KtType::any()))
                .returns(KtType::long()),
        )
        .member(
            KtFun::new("zThingFree")
                .modifier("external")
                .param(KtParam::new("ptr", KtType::long())),
        );
    let src = render::render_one(&obj.into(), "io.test.jni");
    assert_eq!(
        body_of(&src),
        "\
internal object JNINative {
    external fun zThingNew(errorSink: Any): Long

    external fun zThingFree(ptr: Long)
}
"
    );
}

#[test]
fn top_level_fun_with_generics_named_lambda_and_default() {
    let f = KtFun::new("zThingSub")
        .vis(Vis::Public)
        .annotation("Suppress(\"UNCHECKED_CAST\")")
        .generic("R")
        .param(KtParam::new("thing", KtType::cls("io.test.jni.thing.ZThing")))
        .param(
            KtParam::new(
                "onError",
                KtType::lambda(
                    [
                        ("je".to_string(), KtType::string().nullable()),
                        ("message".to_string(), KtType::string()),
                    ],
                    KtType::var_r(),
                ),
            )
            .default("{ __de_je, __de_z0 -> throw ZException(__de_je ?: __de_z0) }"),
        )
        .param(KtParam::new(
            "build",
            KtType::lambda(
                [
                    ("handle".to_string(), KtType::cls("io.test.jni.thing.ZThing")),
                    ("name".to_string(), KtType::string()),
                ],
                KtType::var_r(),
            ),
        ))
        .returns(KtType::var_r())
        .body(
            Code::new()
                .line("var __cap_failed = false")
                .blk("val __ret = run {", |c| {
                    c.line("(JNINative.zThingSub(thing.ptr, build, __cap) as R)")
                })
                .line("if (__cap_failed) return onError(__cap_je, \"\")")
                .line("return __ret"),
        );
    let src = render::render_one(&f.into(), "io.test.jni.thing");
    assert_eq!(
        body_of(&src),
        "\
@Suppress(\"UNCHECKED_CAST\")
public fun <R> zThingSub(
    thing: ZThing,
    onError: (je: String?, message: String) -> R = { __de_je, __de_z0 -> throw ZException(__de_je ?: __de_z0) },
    build: (handle: ZThing, name: String) -> R,
): R {
    var __cap_failed = false
    val __ret = run {
        (JNINative.zThingSub(thing.ptr, build, __cap) as R)
    }
    if (__cap_failed) return onError(__cap_je, \"\")
    return __ret
}
"
    );
}

#[test]
fn unit_return_is_omitted() {
    let f = KtFun::new("doIt")
        .vis(Vis::Public)
        .returns(KtType::unit())
        .body(Code::new().line("work()"));
    let src = render::render_one(&f.into(), "p");
    assert!(src.contains("public fun doIt() {"), "{src}");
    assert!(!src.contains(": Unit"), "{src}");
}

#[test]
fn long_signature_wraps_params_one_per_line() {
    // Short signatures stay on a single line.
    let short = KtFun::new("short")
        .vis(Vis::Public)
        .param(KtParam::new("a", KtType::int()))
        .param(KtParam::new("b", KtType::int()))
        .returns(KtType::int())
        .body(Code::new().line("a + b"));
    let src = render::render_one(&short.into(), "p");
    assert!(src.contains("public fun short(a: Int, b: Int): Int {"), "{src}");

    // A signature wider than the threshold breaks one parameter per line,
    // with a trailing comma and the closing paren at the function indent.
    let long = KtFun::new("zSessionDeclareSubscriber")
        .vis(Vis::Public)
        .param(KtParam::new("session", KtType::cls("ZSession")))
        .param(KtParam::new("keyExprSel", KtType::int()))
        .param(KtParam::new("keyExpr0", KtType::string().nullable()))
        .param(KtParam::new("keyExpr1", KtType::cls("ZKeyExpr").nullable()))
        .param(KtParam::new("onClose", KtType::lambda([], KtType::unit())))
        .returns(KtType::cls("ZSubscriber"))
        .body(Code::new().line("TODO()"));
    let src = render::render_one(&long.into(), "p");
    assert!(
        src.contains(
            "public fun zSessionDeclareSubscriber(\n    \
             session: ZSession,\n    \
             keyExprSel: Int,\n    \
             keyExpr0: String?,\n    \
             keyExpr1: ZKeyExpr?,\n    \
             onClose: () -> Unit,\n\
             ): ZSubscriber {"
        ),
        "{src}"
    );
}

#[test]
fn typealias_renders() {
    let d = KtDecl::TypeAlias {
        vis: Vis::Public,
        name: "OldName".into(),
        target: KtType::cls("io.test.jni.NewName"),
    };
    let src = render::render_one(&d, "io.test.compat");
    assert!(src.contains("public typealias OldName = NewName"), "{src}");
    assert!(src.contains("import io.test.jni.NewName"), "{src}");
}

#[test]
fn import_collision_falls_back_to_fqn() {
    let f = KtFun::new("f")
        .param(KtParam::new("a", KtType::cls("io.a.Same")))
        .param(KtParam::new("b", KtType::cls("io.b.Same")))
        .body(Code::new());
    let src = render::render_one(&f.into(), "p");
    assert!(src.contains("import io.a.Same"), "{src}");
    assert!(!src.contains("import io.b.Same"), "{src}");
    assert!(src.contains("a: Same, b: io.b.Same"), "{src}");
}

#[test]
fn same_package_types_need_no_import() {
    let f = KtFun::new("f")
        .param(KtParam::new("a", KtType::cls("io.p.Local")))
        .body(Code::new());
    let src = render::render_one(&f.into(), "io.p");
    assert!(!src.contains("import io.p.Local"), "{src}");
    assert!(src.contains("a: Local"), "{src}");
}

#[test]
fn from_kotlin_name_covers_metadata_strings() {
    let mut imp = ImportSet::new("p");
    // The shapes the existing jnigen metadata produces.
    for (input, want) in [
        ("Int", "Int"),
        ("String?", "String?"),
        ("ByteArray", "ByteArray"),
        ("io.zenoh.jni.keyexpr.ZKeyExpr", "ZKeyExpr"),
        ("io.zenoh.jni.keyexpr.ZKeyExpr?", "ZKeyExpr?"),
        ("List<io.zenoh.jni.ZZenohId>", "List<ZZenohId>"),
        ("List<ByteArray>", "List<ByteArray>"),
        ("Any?", "Any?"),
        ("R", "R"),
    ] {
        assert_eq!(KtType::from_kotlin_name(input).render(&mut imp), want);
    }
}

#[test]
fn merge_files_groups_by_package_and_rejects_duplicates() {
    let a = KtFile::new("io.p").decl(KtClass::new(ClassKind::Plain, "A").vis(Vis::Public));
    let b = KtFile::new("io.p").decl(KtFun::new("f").body(Code::new()));
    let c = KtFile::new("io.q").decl(KtClass::new(ClassKind::Plain, "C"));
    let merged = merge_files(vec![a, b, c]).expect("merge");
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0].package, "io.p");
    assert_eq!(merged[0].decls.len(), 2);

    let d1 = KtFile::new("io.p").decl(KtClass::new(ClassKind::Plain, "A"));
    let d2 = KtFile::new("io.p").decl(KtClass::new(ClassKind::Plain, "A"));
    assert!(merge_files(vec![d1, d2]).is_err());
}

#[test]
fn merged_file_path_is_flattened() {
    let f = KtFile::new("io.zenoh.jni.bytes");
    let p = file::merged_file_path(std::path::Path::new("/root"), &f, "X");
    assert_eq!(p, std::path::PathBuf::from("/root/io/zenoh/jni/bytes.kt"));
    let empty = KtFile::new("");
    let p2 = file::merged_file_path(std::path::Path::new("/root"), &empty, "NativeHandle");
    assert_eq!(p2, std::path::PathBuf::from("/root/NativeHandle.kt"));
}

#[test]
fn multiline_kdoc() {
    let c = KtClass::new(ClassKind::Plain, "X")
        .kdoc("First line.\n\nSecond paragraph.");
    let src = render::render_one(&c.into(), "p");
    assert!(
        src.contains("/**\n * First line.\n *\n * Second paragraph.\n */\nclass X"),
        "{src}"
    );
}
