use super::*;

/// Emit the Kotlin surface for a `sealed_class!`-declared sum, optionally
/// with a per-variant rename, and return the single generated file's text.
fn sealed_kotlin(rename_labeled: Option<&str>) -> String {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Priority {
                    Low = 0,
                    High = 1,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Enum(syn::parse_quote!(
                /// A sensor reading.
                pub enum Reading {
                    /// Nothing read.
                    Missing,
                    Exact(i64),
                    Range {
                        low: i64,
                        high: i64,
                    },
                    Labeled(String, Priority),
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let mut sealed = crate::sealed_class!(Reading);
    if let Some(n) = rename_labeled {
        sealed = sealed.variant(crate::variant!(Labeled).name(n));
    }
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::enum_class!(Priority))
            .class(sealed),
    );

    let dir = unique_test_dir("jnigen_sealed");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The sealed surface: a `sealed interface` whose variant classes are nested
/// inside it, a payload-less variant as a `data object`, tuple payloads as
/// `v0`/`v1`, named payloads keeping their camelCased names, and a
/// `fromParts` companion that picks the live group by tag.
#[test]
fn sealed_class_kotlin_surface() {
    let kt = sealed_kotlin(None);
    let c: String = kt.split_whitespace().collect();

    assert!(c.contains("publicsealedinterfaceReading{"), "{kt}");
    // The payload-less alternative is a singleton `data object`.
    assert!(c.contains("publicdataobjectMissing:Reading"), "{kt}");
    // Tuple payloads surface as `v0`, `v1`; named payloads keep their names.
    assert!(
        c.contains("publicdataclassExact(publicvalv0:Long):Reading"),
        "{kt}"
    );
    assert!(
        c.contains("publicdataclassRange(publicvallow:Long,publicvalhigh:Long):Reading"),
        "{kt}"
    );
    // A declared `enum_class` payload surfaces as that enum's Kotlin type.
    assert!(
        c.contains("publicdataclassLabeled(publicvalv0:String,publicvalv1:Priority):Reading"),
        "{kt}"
    );

    // `fromParts(tag, …every group's slots side by side…)`.
    assert!(c.contains("funfromParts("), "{kt}");
    assert!(c.contains("tag:Int,"), "{kt}");
    assert!(c.contains("exact_v0:Long,"), "{kt}");
    assert!(c.contains("range_low:Long,"), "{kt}");
    assert!(c.contains("range_high:Long,"), "{kt}");
    assert!(c.contains("labeled_v0:String,"), "{kt}");
    assert!(c.contains("labeled_v1:Priority,"), "{kt}");
    // Tags are declaration order, and an out-of-range tag is an error rather
    // than a variant.
    assert!(c.contains("0->Missing"), "{kt}");
    assert!(c.contains("1->Exact(exact_v0)"), "{kt}");
    assert!(c.contains("2->Range(range_low,range_high)"), "{kt}");
    assert!(c.contains("3->Labeled(labeled_v0,labeled_v1)"), "{kt}");
    assert!(
        c.contains("else->throwIllegalArgumentException(\"Reading:invalidtag$tag\")"),
        "{kt}"
    );
    // The enum class beside it is untouched — sums are a new path, not a
    // replacement.
    assert!(c.contains("publicenumclassPriority"), "{kt}");
}

/// `variant!(V).name(...)` renames the variant class **and** its `fromParts`
/// slots, so the emitted surface stays self-consistent.
#[test]
fn variant_rename_carries_to_slots() {
    let kt = sealed_kotlin(Some("Tagged"));
    let c: String = kt.split_whitespace().collect();
    assert!(c.contains("publicdataclassTagged("), "{kt}");
    assert!(!c.contains("dataclassLabeled("), "{kt}");
    assert!(c.contains("tagged_v0:String,"), "{kt}");
    assert!(c.contains("tagged_v1:Priority,"), "{kt}");
    assert!(c.contains("3->Tagged(tagged_v0,tagged_v1)"), "{kt}");
    // The tag numbering is the enum's declaration order, unaffected by names.
    assert!(c.contains("0->Missing"), "{kt}");
}

/// Kdoc from the Rust item and from each variant carries into the Kotlin
/// surface.
#[test]
fn sealed_class_carries_docs() {
    let kt = sealed_kotlin(None);
    assert!(kt.contains("A sensor reading."), "{kt}");
    assert!(kt.contains("Nothing read."), "{kt}");
    assert!(
        kt.contains("exactly one alternative is live"),
        "the framework kdoc line is missing:\n{kt}"
    );
}

/// The two enum declarators are shape-exclusive: a fieldless enum handed to
/// `sealed_class!` is an error naming `enum_class!`, and a payload enum
/// handed to `enum_class!` is an error naming `sealed_class!`. Neither
/// silently upgrades.
#[test]
fn declarators_do_not_accept_each_others_shape() {
    let loc = myflat_loc();
    let unit_enum: syn::Item = syn::Item::Enum(syn::parse_quote!(
        pub enum Priority {
            Low = 0,
            High = 1,
        }
    ));
    let payload_enum: syn::Item = syn::Item::Enum(syn::parse_quote!(
        pub enum Reading {
            Missing,
            Exact(i64),
        }
    ));

    let emit = |item: syn::Item, decl: crate::lang::ClassDecl, tag: &str| {
        let registry =
            Registry::<KotlinMeta>::from_items(vec![(item, loc.clone())]).expect("index items");
        let jni = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(decl));
        let dir = unique_test_dir(tag);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = registry.resolve(jni).expect("resolve");
        let _ = gen.write_kotlin(&dir.join("kotlin"));
    };

    // Fieldless enum → `sealed_class!` is rejected.
    let unit = unit_enum.clone();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        emit(unit, crate::sealed_class!(Priority).into(), "sealed_unit");
    }))
    .is_err());

    // Payload enum → `enum_class!` is rejected.
    let payload = payload_enum.clone();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        emit(payload, crate::enum_class!(Reading).into(), "enum_payload");
    }))
    .is_err());
}

/// `variant!(V)` naming a variant the enum does not have is a declaration
/// mistake, not a silent no-op.
#[test]
fn unknown_variant_is_an_error() {
    let loc = myflat_loc();
    let boom = || {
        let registry = Registry::<KotlinMeta>::from_items(vec![(
            syn::Item::Enum(syn::parse_quote!(
                pub enum Reading {
                    Missing,
                    Exact(i64),
                }
            )),
            loc.clone(),
        )])
        .expect("index items");
        let jni = JniGen::new().set_package_prefix("io.test.jni").package(
            crate::package!()
                .class(crate::sealed_class!(Reading).variant(crate::variant!(Nope).name("X"))),
        );
        let dir = unique_test_dir("sealed_unknown_variant");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = registry.resolve(jni).expect("resolve");
        let _ = gen.write_kotlin(&dir.join("kotlin"));
    };
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(boom)).is_err());
}

/// Reopening `sealed_class!(E)` **merges**, like every other class kind: a
/// second declaration adding one `.variant(...)` must not drop the renames
/// the first one set.
#[test]
fn reopened_sealed_class_merges_variant_names() {
    let loc = myflat_loc();
    let registry = Registry::<KotlinMeta>::from_items(vec![(
        syn::Item::Enum(syn::parse_quote!(
            pub enum Reading {
                Missing,
                Exact(i64),
                Range { low: i64, high: i64 },
            }
        )),
        loc.clone(),
    )])
    .expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!().class(
                crate::sealed_class!(Reading).variant(crate::variant!(Missing).name("None_")),
            ),
        )
        // Reopened in a second `.package(...)` context, adding one more
        // rename — the first one must survive.
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading).variant(crate::variant!(Exact).name("One"))),
        );

    let dir = unique_test_dir("sealed_reopen");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let kt: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let c: String = kt.split_whitespace().collect();

    assert!(c.contains("publicdataobjectNone_:Reading"), "{kt}");
    assert!(
        c.contains("publicdataclassOne(publicvalv0:Long):Reading"),
        "{kt}"
    );
    // The undeclared variant keeps its Rust ident.
    assert!(c.contains("publicdataclassRange("), "{kt}");
    assert!(c.contains("0->None_"), "{kt}");
    assert!(c.contains("1->One(one_v0)"), "{kt}");
}

/// A type gets exactly **one** class declarator. Two different ones would
/// emit two Kotlin declarations for the same FQN, so the second is a hard
/// error — symmetrically, whichever order they come in.
#[test]
fn a_type_gets_one_class_declarator() {
    let loc = myflat_loc();
    let items = || {
        vec![
            (
                syn::Item::Enum(syn::parse_quote!(
                    pub enum Reading {
                        Missing,
                        Exact(i64),
                    }
                )),
                loc.clone(),
            ),
            (
                syn::Item::Struct(syn::parse_quote!(
                    pub struct Sample {
                        pub id: i64,
                    }
                )),
                loc.clone(),
            ),
        ]
    };
    let declare = |first: crate::lang::ClassDecl, second: crate::lang::ClassDecl| {
        let registry = Registry::<KotlinMeta>::from_items(items()).expect("index items");
        let _ = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(first).class(second));
        drop(registry);
    };

    // Every conflicting pair among the five declarators is rejected, in both
    // orders — the check runs before any registration, so it does not depend
    // on which came first.
    type MakeDecl = fn() -> crate::lang::ClassDecl;
    let pairs: Vec<(MakeDecl, MakeDecl)> = vec![
        (
            || crate::sealed_class!(Reading).into(),
            || crate::value_class!(Reading).into(),
        ),
        (
            || crate::value_class!(Reading).into(),
            || crate::sealed_class!(Reading).into(),
        ),
        (
            || crate::sealed_class!(Reading).into(),
            || crate::enum_class!(Reading).into(),
        ),
        (
            || crate::sealed_class!(Reading).into(),
            || crate::ptr_class!(Reading).into(),
        ),
        (
            || crate::data_class!(Sample).into(),
            || crate::value_class!(Sample).into(),
        ),
        (
            || crate::value_class!(Sample).into(),
            || crate::data_class!(Sample).into(),
        ),
        (
            || crate::ptr_class!(Sample).into(),
            || crate::data_class!(Sample).into(),
        ),
    ];
    for (a, b) in pairs {
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| declare(a(), b()))).is_err(),
            "a conflicting declarator pair was accepted"
        );
    }

    // Reopening the SAME declarator stays legal (options merge).
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        declare(
            crate::sealed_class!(Reading).into(),
            crate::sealed_class!(Reading).into(),
        );
    }))
    .is_ok());
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        declare(
            crate::data_class!(Sample).into(),
            crate::data_class!(Sample).into(),
        );
    }))
    .is_ok());
}

/// A sum is `TypeKind::Sum`, not a data struct — it has its own emitter and
/// must never be picked up by the data-class flattener.
#[test]
fn sum_is_its_own_type_kind() {
    let loc = myflat_loc();
    let registry = Registry::<KotlinMeta>::from_items(vec![(
        syn::Item::Enum(syn::parse_quote!(
            pub enum Reading {
                Missing,
                Exact(i64),
            }
        )),
        loc.clone(),
    )])
    .expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!().class(crate::sealed_class!(Reading)));
    let ty: syn::Type = syn::parse_quote!(Reading);
    assert!(matches!(
        jni.type_kind(&registry, &ty),
        crate::api::lang::jnigen::jni::classify::TypeKind::Sum
    ));
    let cfg = jni.types.get(&TypeKey::from_type(&ty)).expect("declared");
    assert!(cfg.special_decl());
}
