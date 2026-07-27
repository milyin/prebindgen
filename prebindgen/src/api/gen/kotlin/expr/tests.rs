use super::{render::render_expr, *};
use crate::api::gen::kotlin::{
    slot::{
        AnnotationSlot, ExprSlot, KtAccessor, KtAnnotation, PropertyValue, StaticAnnotationText,
    },
    types::{ImportSet, KtType},
};

/// Strip Rust comments — both `//` and `/* … */` — while leaving **string
/// literals intact**, so a scan for a forbidden token still sees one written in
/// a message but not one written in prose.
///
/// Char literals are not tracked: `'` is overwhelmingly a lifetime, and a naive
/// char state would swallow code from `'a` to the next quote.
///
/// NOTE: Stage T (#190) carries an identical helper for its module-boundary
/// test. The two branches are independent, so this is a deliberate copy rather
/// than a shared util that would conflict on merge; folding them into one test
/// utility belongs to whichever lands second.
fn code_without_comments(src: &str) -> String {
    #[derive(Clone, Copy, PartialEq)]
    enum St {
        Code,
        Line,
        Block,
        Str,
        StrEsc,
    }
    let mut out = String::with_capacity(src.len());
    let mut st = St::Code;
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        match st {
            St::Code => match (c, chars.peek()) {
                ('/', Some('/')) => {
                    chars.next();
                    st = St::Line;
                }
                ('/', Some('*')) => {
                    chars.next();
                    st = St::Block;
                }
                ('"', _) => {
                    st = St::Str;
                    out.push(c);
                }
                _ => out.push(c),
            },
            St::Line => {
                if c == '\n' {
                    st = St::Code;
                    out.push(c);
                }
            }
            St::Block => {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    st = St::Code;
                } else if c == '\n' {
                    out.push(c);
                }
            }
            St::Str => {
                out.push(c);
                st = match c {
                    '\\' => St::StrEsc,
                    '"' => St::Code,
                    _ => St::Str,
                };
            }
            St::StrEsc => {
                out.push(c);
                st = St::Str;
            }
        }
    }
    out
}

/// Render with a throwaway import set — for assertions that do not care about
/// imports.
fn r(arena: &ExprArena, e: &KtExpr) -> String {
    let mut imports = ImportSet::default();
    render_expr(arena, e, &mut imports)
}

// ── precedence ──────────────────────────────────────────────────────────

/// Precedence is a property of the tree, not of whoever built it. Postfix binds
/// tighter than `as`, which binds tighter than elvis — and a child looser than
/// its position gets parentheses, automatically.
#[test]
fn precedence_parenthesizes_exactly_where_needed() {
    let arena = ExprArena::new();

    // `a ?: b` under a field access needs parens: `(a ?: b).c`.
    let e = KtExpr::name("a").elvis(KtExpr::name("b")).field("c");
    assert_eq!(r(&arena, &e), "(a ?: b).c");

    // …but a field access under an elvis does not: `a.c ?: b`.
    let e = KtExpr::name("a").field("c").elvis(KtExpr::name("b"));
    assert_eq!(r(&arena, &e), "a.c ?: b");

    // `as` under a field access needs parens; a field under `as` does not.
    let e = KtExpr::name("a").cast(KtType::cls("Foo")).field("c");
    assert_eq!(r(&arena, &e), "(a as Foo).c");
    let e = KtExpr::name("a").field("c").cast(KtType::cls("Foo"));
    assert_eq!(r(&arena, &e), "a.c as Foo");

    // Elvis is right-associative: the left operand is parenthesized, the right
    // is not.
    let e = KtExpr::name("a")
        .elvis(KtExpr::name("b"))
        .elvis(KtExpr::name("c"));
    assert_eq!(r(&arena, &e), "(a ?: b) ?: c");
    let e = KtExpr::name("a").elvis(KtExpr::name("b").elvis(KtExpr::name("c")));
    assert_eq!(r(&arena, &e), "a ?: b ?: c");

    // An `as?` operand that is itself an elvis needs parens.
    let e = KtExpr::name("a")
        .elvis(KtExpr::name("b"))
        .safe_cast(KtType::cls("Foo"));
    assert_eq!(r(&arena, &e), "(a ?: b) as? Foo");

    // A call argument is at the loosest position, so it never needs parens.
    let e = KtExpr::free_call("f", [KtExpr::name("a").elvis(KtExpr::name("b"))]);
    assert_eq!(r(&arena, &e), "f(a ?: b)");
}

/// The tag-gated variant access that motivated the whole textual-template
/// design — `(<base>.field as? Reading.Exact)?.v0 ?: 0L` — is one tree, with
/// every parenthesis derived.
#[test]
fn the_tag_gated_variant_access_is_one_tree() {
    let arena = ExprArena::new();
    let e = KtExpr::name("payload")
        .field("field")
        .safe_cast(KtType::cls("io.test.Reading.Exact"))
        .safe_field("v0")
        .elvis(KtExpr::long(0));
    assert_eq!(r(&arena, &e), "(payload.field as? Exact)?.v0 ?: 0L");
}

/// Safe-call chains, and literals escaped by the renderer rather than
/// pre-escaped by a producer.
#[test]
fn safe_calls_and_literal_escaping() {
    let arena = ExprArena::new();
    let e = KtExpr::name("h")
        .safe_field("inner")
        .safe_call("get", [KtExpr::int(3)]);
    assert_eq!(r(&arena, &e), "h?.inner?.get(3)");

    // The raw value goes in; the escaping comes out.
    let e = KtExpr::str_("say \"hi\"\n\tand $x");
    assert_eq!(r(&arena, &e), r#""say \"hi\"\n\tand \$x""#);

    let e = KtExpr::free_call(
        "listOf",
        [KtExpr::null(), KtExpr::bool_(true), KtExpr::long(7)],
    );
    assert_eq!(r(&arena, &e), "listOf(null, true, 7L)");
}

// ── scope-tracked name allocation ───────────────────────────────────────

/// The renderer allocates lambda parameter names with a real scope stack,
/// replacing the hand-numbered `e0`/`e1` convention. Nested lambdas that ask
/// for the same hint get distinct names.
#[test]
fn nested_lambdas_do_not_shadow() {
    let mut arena = ExprArena::new();
    let outer = arena.bind_fresh("e");
    let inner = arena.bind_fresh("e");
    // { e -> f(e, { e2 -> g(e, e2) }) }
    let tree = KtExpr::lambda1(
        [outer],
        KtExpr::free_call("f", [KtExpr::local(outer)]).with_trailing_lambda(KtLambda::expr(
            [inner],
            KtExpr::free_call("g", [KtExpr::local(outer), KtExpr::local(inner)]),
        )),
    );
    assert_eq!(r(&arena, &tree), "{ e -> f(e) { e2 -> g(e, e2) } }");
}

/// A machine-allocated binder may not shadow a **free name** the tree
/// references. Free names are reserved before any binder name is allocated —
/// otherwise a `Fresh` binder hinted `value` would capture a reference to a
/// class or member called `value`.
#[test]
fn a_fresh_binder_never_shadows_a_referenced_free_name() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fresh("value");
    let tree = KtExpr::lambda1(
        [b],
        KtExpr::free_call("f", [KtExpr::local(b), KtExpr::name("value")]),
    );
    let rendered = r(&arena, &tree);
    // The binder had to move aside; the free `value` still reads as itself.
    assert_eq!(rendered, "{ value2 -> f(value2, value) }");
}

/// **`Fixed` spellings survive rendering byte-identically.** A function or
/// constructor parameter name is Kotlin's named-argument surface, callable from
/// user code — renaming one would silently break every `foo(bar = …)` call
/// site. So a `Fresh` binder moves aside for a `Fixed` one, never the reverse.
#[test]
fn fixed_spellings_are_preserved_byte_identically() {
    let mut arena = ExprArena::new();
    let fixed = arena.bind_fixed(KtName::expect("payload"));
    let fresh = arena.bind_fresh("payload");
    let tree = KtExpr::lambda1(
        [fixed, fresh],
        KtExpr::free_call("f", [KtExpr::local(fixed), KtExpr::local(fresh)]),
    );
    let rendered = r(&arena, &tree);
    assert!(
        rendered.starts_with("{ payload, payload2 ->"),
        "the Fixed name must be kept verbatim: {rendered}"
    );
    assert_eq!(rendered, "{ payload, payload2 -> f(payload, payload2) }");
}

/// A local `val` is a binder like any other, and its value is rendered before
/// it enters scope — `val x = x` refers to the outer `x`.
#[test]
fn locals_are_binders_and_bind_after_their_value() {
    let mut arena = ExprArena::new();
    let outer = arena.bind_fixed(KtName::expect("x"));
    let local = arena.bind_fresh("x");
    let tree = KtExpr::lambda(
        [outer],
        vec![
            KtStmt::Let {
                binder: local,
                mutable: false,
                value: KtExpr::local(outer).field("inner"),
            },
            KtStmt::Expr(KtExpr::local(local)),
        ],
    );
    assert_eq!(r(&arena, &tree), "{ x -> val x2 = x.inner; x2 }");
}

// ── import collection ───────────────────────────────────────────────────

/// A rendered unit reports the imports it needs, **from the tree** — replacing
/// the hand registration alongside raw text, where the two could drift.
#[test]
fn imports_are_collected_from_the_tree() {
    let arena = ExprArena::new();
    let tree = KtExpr::name("io.zenoh.jni.Session")
        .call("open", [KtExpr::name("io.zenoh.Config").field("DEFAULT")])
        .cast(KtType::cls("io.zenoh.jni.Handle"));

    let mut imports = ImportSet::default();
    let rendered = render_expr(&arena, &tree, &mut imports);
    // Qualified names render short…
    assert_eq!(rendered, "Session.open(Config.DEFAULT) as Handle");
    // …and register their FQN.
    let collected = imports.import_lines();
    for want in [
        "io.zenoh.jni.Session",
        "io.zenoh.Config",
        "io.zenoh.jni.Handle",
    ] {
        assert!(
            collected.iter().any(|i| i.contains(want)),
            "missing {want} in {collected:?}"
        );
    }
}

/// `free_names` is what both the import set and the renderer's reservation are
/// built from, so it has to see every name-bearing position.
#[test]
fn free_names_sees_every_name_position() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fresh("it");
    let tree = KtExpr::lambda1(
        [b],
        KtExpr::name("Cls")
            .field("member")
            .call("method", [KtExpr::name("Arg")]),
    );
    let names: Vec<String> = free_names(&tree)
        .into_iter()
        .map(|n| n.as_str().to_string())
        .collect();
    for want in ["Cls", "member", "method", "Arg"] {
        assert!(
            names.contains(&want.to_string()),
            "missing {want} in {names:?}"
        );
    }
    // A binder is NOT a free name — that is the whole `Local` / `Name` split.
    assert!(!names.contains(&"it".to_string()));
}

// ── hole filling ────────────────────────────────────────────────────────

/// Hole-filling is a tree operation. `kt_access_prefix` + base +
/// `kt_access_tail` was simulating exactly this, with the base in the middle
/// and three strings that each had to remember the other two.
#[test]
fn fill_hole_replaces_the_base_in_place() {
    let arena = ExprArena::new();
    let template = KtExpr::Hole
        .field("field")
        .safe_cast(KtType::cls("io.test.Reading.Exact"))
        .safe_field("v0")
        .elvis(KtExpr::long(0));
    assert!(has_hole(&template));

    for (base, want) in [
        (
            KtExpr::name("payload"),
            "(payload.field as? Exact)?.v0 ?: 0L",
        ),
        (KtExpr::this(), "(this.field as? Exact)?.v0 ?: 0L"),
    ] {
        let filled = fill_hole(&arena, &template, &base);
        assert!(!has_hole(&filled));
        assert_eq!(r(&arena, &filled), want);
    }

    // A base that is itself compound gets its own parentheses from precedence,
    // not from the template author remembering to add them.
    let filled = fill_hole(
        &arena,
        &template,
        &KtExpr::name("a").elvis(KtExpr::name("b")),
    );
    assert_eq!(r(&arena, &filled), "((a ?: b).field as? Exact)?.v0 ?: 0L");
}

/// An unfilled hole is a generator bug, so it fails loudly rather than
/// rendering something plausible.
#[test]
#[should_panic(expected = "KtExpr::Hole reached the renderer")]
fn an_unfilled_hole_panics_rather_than_rendering() {
    let arena = ExprArena::new();
    let _ = r(&arena, &KtExpr::Hole.field("x"));
}

// ── substitution and capture ────────────────────────────────────────────

/// Substituting an expression that contains a `Local` into a lambda binding a
/// **same-printed** name must not capture it.
///
/// This is `replace_ident`'s defect stated as a test: textually, both are `e`.
/// Structurally they are different `BindingId`s, and the renderer — which
/// allocates the printed names — keeps them apart.
#[test]
fn substituting_a_local_into_a_same_printed_scope_does_not_capture() {
    let mut arena = ExprArena::new();
    let outer = arena.bind_fresh("e");
    let inner = arena.bind_fresh("e");
    let target = arena.bind_fresh("slot");

    // The expression being substituted refers to the OUTER `e`.
    let value = KtExpr::free_call("use", [KtExpr::local(outer)]);
    // …and is inserted under a lambda that binds another binder also hinted `e`.
    let template = KtExpr::lambda1([inner], KtExpr::local(target));
    let substituted = substitute(&arena, &template, target, &value);
    let whole = KtExpr::lambda1([outer], substituted);

    let rendered = r(&arena, &whole);
    // The inner binder was renamed, so `use` still sees the outer one.
    assert_eq!(rendered, "{ e -> { e2 -> use(e) } }");
}

/// Substituting an expression containing a **free `KtName`** into a scope whose
/// binder would print that name must not capture it either — the position
/// `BindingId`s do not cover, closed by reserving free names first.
#[test]
fn substituting_a_free_name_into_a_colliding_scope_does_not_capture() {
    let mut arena = ExprArena::new();
    let inner = arena.bind_fresh("config");
    let target = arena.bind_fresh("slot");

    let value = KtExpr::free_call("use", [KtExpr::name("config")]);
    let template = KtExpr::lambda1([inner], KtExpr::local(target));
    let whole = substitute(&arena, &template, target, &value);

    // The binder moved aside; the free `config` still reads as itself.
    assert_eq!(r(&arena, &whole), "{ config2 -> use(config) }");
}

/// **Grafting two independently-built trees that allocated colliding
/// `BindingId`s must alpha-remap rather than capture.**
///
/// Both arenas below allocate `BindingId(0)`. Merging them naively would fuse
/// two unrelated binders — structural capture that scope-aware *rendering*
/// cannot detect, because at render time the two are indistinguishable.
#[test]
fn grafting_colliding_arenas_alpha_remaps() {
    let mut host = ExprArena::new();
    let host_b = host.bind_fresh("v");
    assert_eq!(host_b.index(), 0);

    let mut guest = ExprArena::new();
    let guest_b = guest.bind_fresh("v");
    // The collision the remap exists for.
    assert_eq!(guest_b.index(), 0);
    let guest_tree = KtExpr::lambda1([guest_b], KtExpr::free_call("g", [KtExpr::local(guest_b)]));

    let grafted = host.graft(&guest, &guest_tree);
    // The guest's binder got a fresh id in the host arena.
    let KtExpr::Lambda(l) = &grafted else {
        panic!("expected a lambda");
    };
    let params = &l.params;
    assert_ne!(params[0], host_b, "the graft must not reuse the host's id");
    assert_eq!(host.len(), 2);

    // And the whole thing renders with two distinct names.
    let whole = KtExpr::lambda1(
        [host_b],
        KtExpr::free_call("f", [KtExpr::local(host_b)]).with_trailing_lambda(match grafted {
            KtExpr::Lambda(l) => l,
            other => panic!("expected a lambda, got {other:?}"),
        }),
    );
    assert_eq!(r(&host, &whole), "{ v -> f(v) { v2 -> g(v2) } }");
}

/// A grafted tree referring to a binder it does not itself introduce is a
/// dangling reference — it was built against a scope that is not coming with
/// it — and is rejected rather than silently rebound to whatever id it lands on.
#[test]
#[should_panic(expected = "refers to a binder the grafted tree does not introduce")]
fn grafting_a_dangling_local_is_rejected() {
    let mut host = ExprArena::new();
    let mut guest = ExprArena::new();
    let stray = guest.bind_fresh("x");
    // `stray` is referenced but never bound inside the grafted expression.
    let _ = host.graft(&guest, &KtExpr::local(stray));
}

/// A `Local` nothing introduces is caught at render time too, rather than
/// producing a plausible-looking name.
#[test]
#[should_panic(expected = "is not in scope")]
fn an_unbound_local_is_rejected_by_the_renderer() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fresh("x");
    let _ = r(&arena, &KtExpr::local(b));
}

// ── `when` ──────────────────────────────────────────────────────────────

#[test]
fn when_arms_render_with_patterns() {
    let mut arena = ExprArena::new();
    let subj = arena.bind_fixed(KtName::expect("v"));
    let tree = KtExpr::lambda1(
        [subj],
        KtExpr::When {
            subject: Box::new(KtExpr::local(subj)),
            arms: vec![
                (
                    KtPattern::Is(KtType::cls("io.test.Reading.Exact")),
                    KtExpr::long(1),
                ),
                (KtPattern::Value(KtExpr::null()), KtExpr::long(0)),
                (KtPattern::Else, KtExpr::long(-1)),
            ],
        },
    );
    assert_eq!(
        r(&arena, &tree),
        "{ v -> when (v) { is Exact -> 1L; null -> 0L; else -> -1L } }"
    );
}

// ── name validation ─────────────────────────────────────────────────────

/// A malformed identifier is rejected **at construction**, not discovered by
/// Gradle. Without this, `Name("a.b() ?: c")` would render as arbitrary source
/// and make #199's "no string-built expressions" exit unfalsifiable.
#[test]
fn kt_name_rejects_anything_that_is_not_an_identifier_path() {
    for bad in [
        "",
        "a.b()",
        "a ?: b",
        "1abc",
        "a..b",
        "a.",
        "has space",
        "a-b",
        "\"quoted\"",
    ] {
        assert!(KtName::new(bad).is_err(), "`{bad}` should be rejected");
    }
    for good in ["a", "_x", "Foo", "io.zenoh.jni.Session", "a1_b2"] {
        assert!(KtName::new(good).is_ok(), "`{good}` should be accepted");
    }
}

/// Only `Raw` accepts free-form expression text. Every other variant is
/// structured — which is what makes counting `Raw`'s construction sites a
/// meaningful progress metric for #199.
#[test]
fn no_variant_other_than_raw_accepts_free_form_text() {
    // The constructors that take a string all route it through `KtName`
    // validation or `KtLiteral` escaping…
    assert!(std::panic::catch_unwind(|| KtExpr::name("a ?: b")).is_err());
    let arena = ExprArena::new();
    // …and a literal is data, not source: it comes back quoted.
    assert_eq!(r(&arena, &KtExpr::str_("a ?: b")), "\"a ?: b\"");
    // `Raw` is the one hole, and it is deliberately conspicuous.
    assert_eq!(r(&arena, &KtExpr::Raw("a ?: b".into())), "a ?: b");
}

// ── declaration parameters are binders ──────────────────────────────────

/// A typed function body can reference its **own parameters** through `Local`.
///
/// Every slot renders in a fresh scope, so without the enclosing declaration's
/// binders a `Local(param)` would be unbound and the renderer would panic.
/// Reaching for `Name("initialPtr")` instead is not an option — it would put a
/// binder back into the free-name set and restore exactly the textual capture
/// `BindingId` exists to remove.
#[test]
fn a_typed_body_can_reference_its_own_parameters() {
    let fun = crate::api::gen::kotlin::KtFun::new("open")
        .param(crate::api::gen::kotlin::KtParam::new(
            "initialPtr",
            KtType::long(),
        ))
        .param(crate::api::gen::kotlin::KtParam::new(
            "config",
            KtType::string(),
        ))
        .typed_body(ExprArena::new(), |_arena, params| {
            vec![KtStmt::Expr(KtExpr::free_call(
                "nativeOpen",
                [KtExpr::local(params[0]), KtExpr::local(params[1])],
            ))]
        });

    // The binders were recorded on the parameters…
    assert!(fun.params.iter().all(|p| p.binder.is_some()));
    // …and the body renders them by their declared names, byte-identically:
    // a parameter name is Kotlin's named-argument surface.
    let crate::api::gen::kotlin::KtBody::Block(slot) = &fun.body else {
        panic!("expected a typed block body");
    };
    let mut imports = ImportSet::default();
    let mut out = String::new();
    slot.render_lines(&mut imports).render(0, &mut out);
    assert_eq!(out.trim(), "nativeOpen(initialPtr, config)");
}

/// A `Fresh` binder introduced inside a typed body moves aside for the
/// enclosing **parameter**, never the reverse — the parameter's spelling is
/// public API.
#[test]
fn a_body_local_does_not_shadow_an_enclosing_parameter() {
    let fun = crate::api::gen::kotlin::KtFun::new("f")
        .param(crate::api::gen::kotlin::KtParam::new(
            "value",
            KtType::long(),
        ))
        .typed_body(ExprArena::new(), |arena, params| {
            let local = arena.bind_fresh("value");
            vec![
                KtStmt::Let {
                    binder: local,
                    mutable: false,
                    value: KtExpr::local(params[0]),
                },
                KtStmt::Expr(KtExpr::local(local)),
            ]
        });
    let crate::api::gen::kotlin::KtBody::Block(slot) = &fun.body else {
        panic!("expected a typed block body");
    };
    let mut imports = ImportSet::default();
    let mut out = String::new();
    slot.render_lines(&mut imports).render(0, &mut out);
    assert_eq!(out.trim(), "val value2 = value\nvalue2");
}

/// The same scope wiring on an arbitrary slot, which is what a supertype
/// constructor argument (`NativeHandle(initialPtr)`) or a setter body needs.
#[test]
fn any_slot_can_be_rendered_with_enclosing_binders() {
    let mut arena = ExprArena::new();
    let param = arena.bind_fixed(KtName::expect("initialPtr"));
    let slot: ExprSlot<Vec<KtExpr>> =
        ExprSlot::ast_in_scope(arena, vec![param], vec![KtExpr::local(param)]);
    let mut imports = ImportSet::default();
    assert_eq!(slot.render_args(&mut imports), "initialPtr");
}

// ── keywords and the literal domain ─────────────────────────────────────

/// Kotlin **hard keywords** are not identifiers. `KtName::new("when")` used to
/// succeed and the renderer emitted uncompilable Kotlin.
///
/// Rejected rather than backtick-escaped: silently rewriting a name would make
/// the emitted spelling differ from the one asked for, which for a `Fixed`
/// binder — a named-argument surface — is exactly what must not happen.
#[test]
fn kt_name_rejects_kotlin_hard_keywords() {
    for kw in [
        "when",
        "class",
        "object",
        "val",
        "is",
        "in",
        "fun",
        "typealias",
    ] {
        assert!(KtName::new(kw).is_err(), "`{kw}` is a hard keyword");
        // …in any path segment, too.
        assert!(
            KtName::new(format!("io.zenoh.{kw}")).is_err(),
            "`{kw}` segment"
        );
    }
    // Soft / modifier keywords ARE legal identifiers and must stay accepted.
    for ok in ["data", "value", "by", "where", "sealed", "inline", "it"] {
        assert!(KtName::new(ok).is_ok(), "`{ok}` is only a soft keyword");
    }
    // `this` is a hard keyword AND a legitimate expression — so it is a node,
    // not a name.
    let arena = ExprArena::new();
    assert_eq!(r(&arena, &KtExpr::this().field("x")), "this.x");
}

/// A `Fresh` hint can easily be a keyword — hints come from field and leaf
/// names — so the allocator sidesteps them the same way it sidesteps a taken
/// name.
#[test]
fn a_fresh_binder_never_renders_as_a_hard_keyword() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fresh("when");
    let tree = KtExpr::lambda1([b], KtExpr::local(b));
    assert_eq!(r(&arena, &tree), "{ when2 -> when2 }");
}

/// Literals must cover the **whole** domain they can represent.
///
/// `-2147483648` and `-9223372036854775808` do not round-trip: Kotlin parses
/// them as unary minus applied to a positive literal one past the type's
/// maximum, and rejects them as out of range. Rust prints non-finite doubles as
/// `NaN` / `inf` / `-inf`, none of which Kotlin accepts.
#[test]
fn literals_cover_their_whole_value_domain() {
    let arena = ExprArena::new();
    for (e, want) in [
        (KtExpr::int(i32::MIN), "Int.MIN_VALUE"),
        (KtExpr::int(i32::MAX), "2147483647"),
        (KtExpr::int(-1), "-1"),
        (KtExpr::long(i64::MIN), "Long.MIN_VALUE"),
        (KtExpr::long(i64::MAX), "9223372036854775807L"),
        (KtExpr::long(-1), "-1L"),
        (KtExpr::Literal(KtLiteral::Double(f64::NAN)), "Double.NaN"),
        (
            KtExpr::Literal(KtLiteral::Double(f64::INFINITY)),
            "Double.POSITIVE_INFINITY",
        ),
        (
            KtExpr::Literal(KtLiteral::Double(f64::NEG_INFINITY)),
            "Double.NEGATIVE_INFINITY",
        ),
        (KtExpr::Literal(KtLiteral::Double(1.0)), "1.0"),
        (KtExpr::Literal(KtLiteral::Double(-0.5)), "-0.5"),
    ] {
        assert_eq!(r(&arena, &e), want);
    }
}

// ── one BindingId is one binding site ───────────────────────────────────

/// Reintroducing the same `BindingId` is rejected.
///
/// `lambda1([b], lambda1([b], local(b)))` used to render `{ x -> { x2 -> x2 } }`:
/// one structural identity with two binding sites, where `Local(b)` picks its
/// referent purely by nesting depth because `lookup` walks frames
/// innermost-first. It also silently breaks `substitute`, which rewrites *every*
/// `Local(b)`.
#[test]
#[should_panic(expected = "is introduced twice in one tree")]
fn nested_reintroduction_of_a_binder_is_rejected() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fresh("x");
    let tree = KtExpr::lambda1([b], KtExpr::lambda1([b], KtExpr::local(b)));
    let _ = r(&arena, &tree);
}

/// The same identity twice in **one** parameter list.
#[test]
#[should_panic(expected = "is introduced twice in one tree")]
fn same_frame_reintroduction_of_a_binder_is_rejected() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fresh("x");
    let tree = KtExpr::lambda1([b, b], KtExpr::local(b));
    let _ = r(&arena, &tree);
}

/// And in two sequential `Let`s, which is the statement-position form.
#[test]
#[should_panic(expected = "is introduced twice in one tree")]
fn repeated_let_of_one_binder_is_rejected() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fresh("x");
    let tree = KtExpr::lambda(
        [],
        vec![
            KtStmt::Let {
                binder: b,
                mutable: false,
                value: KtExpr::int(1),
            },
            KtStmt::Let {
                binder: b,
                mutable: false,
                value: KtExpr::int(2),
            },
            KtStmt::Expr(KtExpr::local(b)),
        ],
    );
    let _ = r(&arena, &tree);
}

/// **Sibling** scopes too, even though they never overlap and `lookup` would
/// stay unambiguous: the identity is still duplicated, so `substitute` would
/// rewrite both occurrences.
#[test]
#[should_panic(expected = "is introduced twice in one tree")]
fn sibling_reintroduction_of_a_binder_is_rejected() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fresh("x");
    let tree = KtExpr::free_call(
        "f",
        [
            KtExpr::lambda1([b], KtExpr::local(b)),
            KtExpr::lambda1([b], KtExpr::local(b)),
        ],
    );
    let _ = r(&arena, &tree);
}

/// Two *distinct* binders that merely share a hint are fine — that is the
/// allocator's job, not an identity error.
#[test]
fn two_distinct_binders_sharing_a_hint_are_fine() {
    let mut arena = ExprArena::new();
    let a = arena.bind_fresh("x");
    let b = arena.bind_fresh("x");
    let tree = KtExpr::lambda1([a], KtExpr::lambda1([b], KtExpr::local(a)));
    assert_eq!(r(&arena, &tree), "{ x -> { x2 -> x } }");
}

/// The uniqueness check must hold **across a vector-valued slot**, not only
/// within one expression.
///
/// `render_args` used to render each element through its own `Scope`, so every
/// element started with an empty `introduced` set and two arguments could bind
/// the same `BindingId` — the exact sibling duplication the whole-render check
/// exists to reject, arriving through the one position where sibling lambdas
/// are most natural. Reachable from enum-entry and supertype-constructor
/// argument slots.
#[test]
#[should_panic(expected = "is introduced twice in one tree")]
fn sibling_reintroduction_across_a_vector_slot_is_rejected() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fresh("x");
    let slot: ExprSlot<Vec<KtExpr>> = ExprSlot::ast(
        arena,
        vec![
            KtExpr::lambda1([b], KtExpr::local(b)),
            KtExpr::lambda1([b], KtExpr::local(b)),
        ],
    );
    let mut imports = ImportSet::default();
    let _ = slot.render_args(&mut imports);
}

/// The elements of a vector slot share **one name scope**, so a free name
/// referenced in *any* element is reserved against binders allocated in the
/// others — they render into the same argument list, where a binder that took
/// the free name's spelling would capture it.
///
/// Note what this does *not* require: the two sibling binders may share a
/// spelling. Their scopes do not overlap — each lambda's frame pops before the
/// next pushes — so `config2` twice is correct Kotlin and gratuitous numbering
/// would be noise. Only the *free* name is off limits to both.
#[test]
fn a_vector_slot_shares_one_name_scope() {
    let mut arena = ExprArena::new();
    let a = arena.bind_fresh("config");
    let b = arena.bind_fresh("config");
    let slot: ExprSlot<Vec<KtExpr>> = ExprSlot::ast(
        arena,
        vec![
            KtExpr::lambda1([a], KtExpr::local(a)),
            KtExpr::lambda1([b], KtExpr::local(b)),
            KtExpr::name("config"),
        ],
    );
    let mut imports = ImportSet::default();
    assert_eq!(
        slot.render_args(&mut imports),
        "{ config2 -> config2 }, { config2 -> config2 }, config"
    );

    // The reservation is what matters, and it comes from the *whole* vector:
    // the free `config` appears only in the last element, yet both binders in
    // the earlier ones avoid it.
    let mut arena = ExprArena::new();
    let only = arena.bind_fresh("config");
    let slot: ExprSlot<Vec<KtExpr>> = ExprSlot::ast(
        arena,
        vec![
            KtExpr::lambda1([only], KtExpr::local(only)),
            KtExpr::name("config"),
        ],
    );
    assert_eq!(
        slot.render_args(&mut imports),
        "{ config2 -> config2 }, config"
    );
}

// ── trailing lambdas ────────────────────────────────────────────────────

/// Kotlin's trailing-argument syntax accepts only a lambda, and the field is
/// typed as one — so `consume() 1` is not constructible rather than rejected at
/// render time. `with_trailing_lambda` takes a [`KtLambda`], and direct variant
/// construction cannot bypass it either, because the field itself is
/// `Option<Box<KtLambda>>`.
#[test]
fn a_trailing_lambda_is_structurally_a_lambda() {
    let mut arena = ExprArena::new();
    let e = arena.bind_fresh("e");
    let call = KtExpr::free_call("consume", [KtExpr::int(1)])
        .with_trailing_lambda(KtLambda::expr([e], KtExpr::local(e)));
    assert_eq!(r(&arena, &call), "consume(1) { e -> e }");

    // A receiver-qualified call takes one too.
    let mut arena = ExprArena::new();
    let it = arena.bind_fresh("it");
    let call = KtExpr::name("list")
        .call("map", [])
        .with_trailing_lambda(KtLambda::expr([it], KtExpr::local(it).field("id")));
    assert_eq!(r(&arena, &call), "list.map() { it -> it.id }");
}

// ── free-name reservations vs. binder scope ─────────────────────────────

/// A **`Fixed` binder sharing a free name's spelling must not consume the free
/// name's reservation** when its frame pops.
///
/// The reservations made up front are permanent for the whole render; a
/// binder's claim lasts until its frame pops. With a set they were the same
/// thing, so a `Fixed` binder inserted a no-op and then *removed the
/// reservation* — after which a later `Fresh` binder could allocate that
/// spelling and capture the free reference.
#[test]
fn a_fixed_binder_does_not_consume_a_free_name_reservation() {
    let mut arena = ExprArena::new();
    let fixed = arena.bind_fixed(KtName::expect("config"));
    let fresh = arena.bind_fresh("config");
    // `{ config -> config }` — the Fixed binder's frame opens and closes —
    // then a Fresh binder asks for the same hint, while the tree still
    // references the free `io.example.config`.
    let tree = KtExpr::free_call(
        "f",
        [
            KtExpr::lambda1([fixed], KtExpr::local(fixed)),
            KtExpr::lambda1([fresh], KtExpr::local(fresh)),
            KtExpr::name("io.example.config"),
        ],
    );
    let mut imports = ImportSet::default();
    let rendered = render_expr(&arena, &tree, &mut imports);
    // The Fresh binder had to move aside: the reservation survived the Fixed
    // binder's pop.
    assert_eq!(
        rendered,
        "f({ config -> config }, { config2 -> config2 }, config)"
    );
}

/// While a binder printed `config` is live, a **qualified** free name that
/// would shorten to `config` stays fully qualified.
///
/// Shortening it would silently turn the qualified reference into the
/// parameter — capture arriving through the import set rather than the scope.
#[test]
fn a_qualified_free_name_shadowed_by_a_binder_stays_qualified() {
    let mut arena = ExprArena::new();
    let fixed = arena.bind_fixed(KtName::expect("config"));
    let tree = KtExpr::lambda1(
        [fixed],
        KtExpr::free_call(
            "f",
            [KtExpr::local(fixed), KtExpr::name("io.example.config")],
        ),
    );
    let mut imports = ImportSet::default();
    assert_eq!(
        render_expr(&arena, &tree, &mut imports),
        "{ config -> f(config, io.example.config) }"
    );
}

/// A **bare** free name shadowed by a live binder is rejected: the two are the
/// same token in the output, and the only fix would be renaming a `Fixed`
/// binder that callers name in arguments.
///
/// A `Fresh` binder can never reach this — the allocator avoids every reserved
/// free name — so it is exactly the `Fixed` case.
#[test]
#[should_panic(expected = "is shadowed by a binder printed the same way")]
fn a_bare_free_name_shadowed_by_a_fixed_binder_is_rejected() {
    let mut arena = ExprArena::new();
    let fixed = arena.bind_fixed(KtName::expect("config"));
    let tree = KtExpr::lambda1(
        [fixed],
        KtExpr::free_call("f", [KtExpr::local(fixed), KtExpr::name("config")]),
    );
    let mut imports = ImportSet::default();
    let _ = render_expr(&arena, &tree, &mut imports);
}

// ── cross-arena composition ─────────────────────────────────────────────

/// `fill_hole` and `substitute` clone trees, so a tree built in **another**
/// arena would bring its `Local`s along — and because indices start at zero per
/// arena, `Local(index 0)` would resolve against the host's binder 0. That is
/// the structural capture `graft` exists to prevent, arriving through the
/// composition functions instead.
///
/// `BindingId` now carries its arena, so the mismatch is detected at the seam
/// rather than silently rendering the wrong binder.
#[test]
#[should_panic(expected = "fill_hole(value)")]
fn filling_a_hole_with_a_foreign_tree_is_rejected() {
    let mut host = ExprArena::new();
    let host_b = host.bind_fresh("v");
    let mut guest = ExprArena::new();
    let guest_b = guest.bind_fresh("v");
    // Both are index 0 — the collision that used to resolve silently.
    assert_eq!(host_b.index(), guest_b.index());

    let template = KtExpr::lambda1([host_b], KtExpr::Hole);
    let _ = fill_hole(&host, &template, &KtExpr::local(guest_b));
}

#[test]
#[should_panic(expected = "substitute(value)")]
fn substituting_a_foreign_tree_is_rejected() {
    let mut host = ExprArena::new();
    let target = host.bind_fresh("slot");
    let mut guest = ExprArena::new();
    let guest_b = guest.bind_fresh("v");
    let _ = substitute(
        &host,
        &KtExpr::local(target),
        target,
        &KtExpr::local(guest_b),
    );
}

/// The supported way across is `graft`, which alpha-remaps — and after it the
/// composition is accepted, because the tree now belongs to the host arena.
#[test]
fn grafting_first_makes_cross_arena_composition_legal() {
    let mut host = ExprArena::new();
    let host_b = host.bind_fresh("v");
    let mut guest = ExprArena::new();
    let guest_b = guest.bind_fresh("v");
    let guest_tree = KtExpr::lambda1([guest_b], KtExpr::local(guest_b));

    let grafted = host.graft(&guest, &guest_tree);
    let template = KtExpr::lambda1([host_b], KtExpr::free_call("f", [KtExpr::Hole]));
    let filled = fill_hole(&host, &template, &grafted);
    assert_eq!(r(&host, &filled), "{ v -> f({ v2 -> v2 }) }");
}

/// A binder looked up in the wrong arena is a named panic, not an index into
/// someone else's `Vec`.
#[test]
#[should_panic(expected = "looked up in arena")]
fn a_foreign_binder_lookup_is_rejected() {
    let host = ExprArena::new();
    let mut guest = ExprArena::new();
    let guest_b = guest.bind_fresh("v");
    let _ = host.binder(guest_b);
}

/// `Ast::in_scope` permits the same mismatched arena/id pairing, so it is
/// guarded at render time by the binder lookup above.
#[test]
#[should_panic(expected = "looked up in arena")]
fn a_slot_scope_from_another_arena_is_rejected() {
    let host = ExprArena::new();
    let mut guest = ExprArena::new();
    let foreign = guest.bind_fresh("v");
    let slot: ExprSlot<KtExpr> =
        ExprSlot::ast_in_scope(host, vec![foreign], KtExpr::local(foreign));
    let mut imports = ImportSet::default();
    let _ = slot.render_inline(&mut imports);
}

// ── the slot bridges ────────────────────────────────────────────────────

/// **No expression position can hold a legacy and a typed value
/// simultaneously.** `ExprSlot` is a sum, so the renderer never chooses between
/// two authorities for one fact — the defect #187 exists to remove, which
/// introducing the typed fields *beside* the legacy ones would have recreated
/// inside its own migration.
#[test]
fn expr_slot_is_exclusive_by_type() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fixed(KtName::expect("x"));
    let ast: ExprSlot<KtExpr> = ExprSlot::ast(arena.clone(), KtExpr::local(b));
    assert!(!ast.is_legacy());

    let legacy: ExprSlot<KtExpr> = ExprSlot::legacy(crate::api::gen::kotlin::Code::new().line("x"));
    assert!(legacy.is_legacy());

    // Both render to the same text through one entry point, so migrating a
    // position is a local change.
    let mut imports = ImportSet::default();
    // A `Fixed` binder at the root has no enclosing scope, so the AST arm is
    // exercised through a lambda that introduces it.
    let wrapped: ExprSlot<KtExpr> =
        ExprSlot::ast(arena.clone(), KtExpr::lambda1([b], KtExpr::local(b)));
    assert_eq!(wrapped.render_inline(&mut imports), "{ x -> x }");
    assert_eq!(legacy.render_inline(&mut imports), "x");
}

/// `PropertyValue` makes the initializer/delegate exclusion **structural**. It
/// used to be two `Option<String>` fields, a doc comment saying "mutually
/// exclusive", and a `debug_assert` that only fires in debug builds — a product
/// where a sum belongs, i.e. the #180 pattern sitting in the Kotlin model.
#[test]
fn property_value_makes_the_exclusion_structural() {
    let d = PropertyValue::default();
    assert!(matches!(d, PropertyValue::None));
    // There is no constructible value carrying both; the type has one slot.
    let init = PropertyValue::Initializer(ExprSlot::legacy(
        crate::api::gen::kotlin::Code::new().line("1"),
    ));
    assert!(matches!(init, PropertyValue::Initializer(_)));
}

/// Accessors are **declarations containing bodies**, not expressions — so they
/// get a structured model rather than an `ExprSlot<KtExpr>`.
#[test]
fn accessors_are_structured_declarations() {
    let mut arena = ExprArena::new();
    let b = arena.bind_fixed(KtName::expect("v"));
    let acc = KtAccessor::get_expr(arena.clone(), KtExpr::lambda1([b], KtExpr::local(b)));
    assert!(!acc.is_legacy());
    let mut imports = ImportSet::default();
    let mut out = String::new();
    acc.render_lines(&mut imports).render(0, &mut out);
    assert_eq!(out.trim(), "get() = { v -> v }");
}

/// An **expression** accessor keeps its scope, like the block arm and every
/// other typed slot. A getter referencing a constructor parameter would
/// otherwise be unbound.
#[test]
fn an_expression_accessor_can_reference_its_enclosing_binders() {
    let mut arena = ExprArena::new();
    let ctor_param = arena.bind_fixed(KtName::expect("initialPtr"));
    let acc = KtAccessor::Get(crate::api::gen::kotlin::AccessorTree::Expr(
        crate::api::gen::kotlin::slot::Ast::in_scope(
            arena,
            vec![ctor_param],
            KtExpr::local(ctor_param).field("value"),
        ),
    ));
    let mut imports = ImportSet::default();
    let mut out = String::new();
    acc.render_lines(&mut imports).render(0, &mut out);
    assert_eq!(out.trim(), "get() = initialPtr.value");
}

/// A setter's value parameter is a **binder**, so the header and the body are
/// spelled from one allocation.
///
/// The header used to be a hardcoded `set(value)`: a binder spelled `incoming`
/// rendered `set(value) = incoming`, and a `Fresh` binder hinted `value` that
/// the allocator moved aside left the signature still saying `value`.
#[test]
fn a_setter_parameter_is_spelled_from_the_same_allocation_as_its_body() {
    // A differently-spelled binder reaches the header.
    let acc = KtAccessor::set_expr(ExprArena::new(), "incoming", |p| {
        KtExpr::free_call("store", [KtExpr::local(p)])
    });
    let mut imports = ImportSet::default();
    let mut out = String::new();
    acc.render_lines(&mut imports).render(0, &mut out);
    assert_eq!(out.trim(), "set(incoming) = store(incoming)");

    // …and one the allocator has to move aside moves in BOTH places, never
    // just the body.
    let acc = KtAccessor::set_expr(ExprArena::new(), "value", |p| {
        // A free `value` in the tree forces the binder to be renamed.
        KtExpr::free_call("store", [KtExpr::local(p), KtExpr::name("value")])
    });
    let mut out = String::new();
    acc.render_lines(&mut imports).render(0, &mut out);
    assert_eq!(out.trim(), "set(value2) = store(value2, value)");
}

/// A typed `Set` **cannot** be built without its value parameter: the variant
/// carries the binder, so `param: None` is not a representable state.
///
/// The old shape fell back to the text `value` whenever `param` was absent, so
/// a `Set` with a body containing `KtExpr::name("value")` rendered
/// `set(value) = value` — a free name captured by an implicit binder with no
/// `BindingId` at all.
#[test]
fn a_typed_setter_always_carries_its_parameter() {
    // The only way to build one binds the parameter.
    let acc = KtAccessor::set_expr(ExprArena::new(), "value", KtExpr::local);
    assert!(matches!(acc, KtAccessor::Set { .. }));
    let mut imports = ImportSet::default();
    let mut out = String::new();
    acc.render_lines(&mut imports).render(0, &mut out);
    assert_eq!(out.trim(), "set(value) = value");

    // And a hand-built `Set` whose parameter is not in its body's scope fails
    // fast rather than falling back to a textual header.
    let mut arena = ExprArena::new();
    let param = arena.bind_fresh("incoming");
    let stray = KtAccessor::Set {
        param,
        body: crate::api::gen::kotlin::AccessorTree::Expr(crate::api::gen::kotlin::slot::Ast::new(
            arena,
            KtExpr::int(0),
        )),
    };
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut imports = ImportSet::default();
        stray.render_lines(&mut imports)
    }));
    assert!(
        err.is_err(),
        "a parameter outside its body's scope must be rejected"
    );
}

/// A **clone** of an arena cannot allocate.
///
/// Cloning keeps the `ArenaId` — trees already built against it still refer to
/// it — so two clones both allocating would mint identical
/// `BindingId { arena, index }` values for different binders, and the
/// provenance check would wave the collision straight through. Sealing the
/// clone closes that by construction; the model's own clones never bind
/// afterwards, so nothing legitimate is lost.
#[test]
#[should_panic(expected = "cannot allocate")]
fn a_cloned_arena_cannot_allocate() {
    let mut original = ExprArena::new();
    let _ = original.bind_fresh("v");
    let mut copy = original.clone();
    let _ = copy.bind_fresh("v");
}

/// The clone is still fully usable for everything except allocation — reading,
/// rendering, and being grafted *from*.
#[test]
fn a_cloned_arena_still_renders_and_grafts() {
    let mut original = ExprArena::new();
    let b = original.bind_fresh("v");
    let tree = KtExpr::lambda1([b], KtExpr::local(b));

    let copy = original.clone();
    assert_eq!(r(&copy, &tree), "{ v -> v }");

    let mut host = ExprArena::new();
    let grafted = host.graft(&copy, &tree);
    assert_eq!(r(&host, &grafted), "{ v -> v }");
}

/// Kotlin reads a bare `_` as an ignoring placeholder, not an identifier — it
/// is not referenceable, so it may be neither a `KtName` nor a rendered binder.
#[test]
fn underscore_is_not_an_identifier() {
    for bad in ["_", "__", "___"] {
        assert!(KtName::new(bad).is_err(), "`{bad}` is a placeholder");
    }
    // …but an underscore *within* a name is ordinary.
    for ok in ["_x", "x_", "a_b", "_1"] {
        assert!(KtName::new(ok).is_ok(), "`{ok}` should be accepted");
    }
    // A hint that would render as the placeholder falls back instead.
    let mut arena = ExprArena::new();
    let b = arena.bind_fresh("_");
    assert_eq!(
        r(&arena, &KtExpr::lambda1([b], KtExpr::local(b))),
        "{ tmp -> tmp }"
    );
}

/// Annotation arguments are expressions, and the typed form renders them
/// through the same escaping the rest of the tier uses.
#[test]
fn annotations_carry_expression_arguments() {
    let ast = AnnotationSlot::Ast(
        KtAnnotation::new(KtName::expect("Suppress")).arg(KtExpr::str_("UNCHECKED_CAST")),
    );
    let mut imports = ImportSet::default();
    assert_eq!(ast.render(&mut imports), "Suppress(\"UNCHECKED_CAST\")");

    // The legacy arm renders verbatim, so wrapping existing producers moved no
    // output.
    let legacy = AnnotationSlot::Legacy(StaticAnnotationText::from_legacy_string("JvmStatic"));
    assert_eq!(legacy.render(&mut imports), "JvmStatic");
    assert!(legacy.is_legacy());
    assert!(legacy.renders_as("JvmStatic"));
}

/// `StaticAnnotationText` built through the macro carries **literal origin as a
/// compile-time property**: `macro_rules!`'s `literal` fragment cannot match
/// `String::leak(…)`, which is why narrowing the payload to `&'static str`
/// would not have been enough.
#[test]
fn annotation_text_from_a_literal() {
    let t = crate::api::gen::kotlin::slot::kt_annotation_text!("JvmField");
    assert_eq!(t.as_str(), "JvmField");
}

/// `from_legacy_string` is the one remaining hole in that guarantee, and it
/// exists only so 5A could change the field type without migrating call sites.
///
/// Its caller count is pinned here so it cannot grow unnoticed: #199 drives it
/// to zero and deletes the function, the same enumerate-then-delete contract
/// `KtExpr::Raw` is under.
#[test]
fn legacy_annotation_bridge_has_exactly_one_caller() {
    let model = include_str!("../model.rs");
    let calls = model
        .matches("StaticAnnotationText::from_legacy_string(")
        .count();
    assert_eq!(
        calls, 1,
        "the legacy annotation bridge should have exactly one caller (the `legacy_annotation` \
         helper in model.rs); every new one is a #199 work item"
    );
    // …and nothing outside the Kotlin model reaches for it.
    for other in [
        include_str!("../render.rs"),
        include_str!("../file.rs"),
        include_str!("../code.rs"),
    ] {
        assert!(!other.contains("from_legacy_string"));
    }
}

/// Every `.rs` file in the crate, for an audit that cannot be outrun by adding
/// a module.
///
/// The constructors below are crate-visible, so a hand-listed set of files is
/// not an audit — a direct call from `expr.rs`, `file.rs` or any JniGen module
/// would simply not be looked at. This walks `src/` instead.
fn crate_sources() -> Vec<(String, String)> {
    // The auditing file is skipped: it necessarily *spells* the constructors it
    // searches for, so counting itself would make the audit self-defeating.
    const SELF: &str = "kotlin/expr/tests.rs";
    fn walk(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        for entry in std::fs::read_dir(dir).expect("readable source dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let shown = path.display().to_string();
                if shown.replace('\\', "/").ends_with(SELF) {
                    continue;
                }
                let text = std::fs::read_to_string(&path).expect("readable source");
                out.push((shown, code_without_comments(&text)));
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    walk(&root, &mut out);
    assert!(out.len() > 50, "the walk should find the whole crate");
    out
}

/// `StaticAnnotationText`'s literal-origin guarantee is **audited, not typed**:
/// `__from_literal` accepts any `&'static str`, leaked included, so the macro —
/// not the type — is where the guarantee lives, and only for its own callers.
///
/// The audit is therefore the whole point, and it runs over **every file in the
/// crate**, which is the scope the constructors' visibility actually has. #199
/// drives both to zero and deletes them.
#[test]
fn static_annotation_text_constructors_are_pinned_crate_wide() {
    let mut from_literal: Vec<&str> = Vec::new();
    let mut legacy: Vec<&str> = Vec::new();
    for (path, code) in crate_sources() {
        for _ in 0..code
            .matches("StaticAnnotationText::__from_literal(")
            .count()
        {
            from_literal.push(Box::leak(path.clone().into_boxed_str()));
        }
        for _ in 0..code
            .matches("StaticAnnotationText::from_legacy_string(")
            .count()
        {
            legacy.push(Box::leak(path.clone().into_boxed_str()));
        }
    }
    let ends_with = |v: &[&str], suffix: &str| v.iter().filter(|p| p.ends_with(suffix)).count();

    // `__from_literal`: the macro body, plus the one injected `JvmInline` the
    // value-class renderer adds.
    assert_eq!(
        from_literal.len(),
        2,
        "unexpected `__from_literal` call sites: {from_literal:?} — every new one is a #199 work \
         item, and the constructor is crate-visible so this list is the only thing bounding it"
    );
    assert_eq!(ends_with(&from_literal, "kotlin/slot.rs"), 1);
    assert_eq!(ends_with(&from_literal, "kotlin/render.rs"), 1);

    // `from_legacy_string`: the single `legacy_annotation` helper in the model.
    assert_eq!(
        legacy.len(),
        1,
        "unexpected `from_legacy_string` call sites: {legacy:?}"
    );
    assert_eq!(ends_with(&legacy, "kotlin/model.rs"), 1);
}

/// `KtExpr::Raw` exists only if migration needs it, and **enumerating its
/// producers is the mechanical check** behind #199's global exit.
///
/// Scanned **crate-wide**, not over a hand-listed set of files: `KtExpr` is
/// re-exported for JniGen and visible throughout it, so a
/// `kt::KtExpr::Raw(...)` in `api/lang/jnigen/…` would pass a `gen/kotlin`-only
/// audit and the asserted exit would drift silently. Same visibility-boundary
/// problem `crate_sources` already solves for `StaticAnnotationText`.
///
/// The AST's own definition, traversal and renderer necessarily *name* the
/// variant — in a declaration and in `match` arms — so those three files are
/// the allow-list. Every other file in the crate must not mention it. Today
/// that set is empty, so the variant is reachable from tests only and #199 can
/// delete it outright.
#[test]
fn ktexpr_raw_has_no_producers_crate_wide() {
    const ALLOWED: [&str; 3] = [
        "kotlin/expr.rs",
        "kotlin/expr/render.rs",
        "kotlin/expr/tests.rs",
    ];
    let mut offenders: Vec<String> = Vec::new();
    let mut allowed_seen = 0usize;
    for (path, code) in crate_sources() {
        let normalized = path.replace('\\', "/");
        if ALLOWED.iter().any(|a| normalized.ends_with(a)) {
            allowed_seen += 1;
            continue;
        }
        if code.contains("KtExpr::Raw") {
            offenders.push(normalized);
        }
    }
    // `crate_sources` skips the auditing file itself, so two of the three
    // allow-listed files are actually walked.
    assert_eq!(
        allowed_seen, 2,
        "the allow-list should name files the walk actually visits"
    );
    assert!(
        offenders.is_empty(),
        "KtExpr::Raw is referenced outside the AST itself: {offenders:?} — every site is a #199 \
         work item, and the variant cannot be deleted while any remain"
    );
}
