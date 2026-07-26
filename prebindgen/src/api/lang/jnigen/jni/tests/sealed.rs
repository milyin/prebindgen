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

/// The interface body is a scope with names this emitter already occupies:
/// the `Companion` object holding `fromParts`, and the interface's own name
/// (which the variants use as their supertype). Both are perfectly legal
/// Kotlin identifiers, so no keyword list catches them — they are caught by
/// the same collision check the variants run against each other.
///
/// Verified against the Kotlin compiler: a `Companion` variant is
/// "Conflicting declarations: data class Companion, companion object", and a
/// variant named after the interface is "There's a cycle in the inheritance
/// hierarchy for this type".
#[test]
fn variant_cannot_take_a_name_the_interface_body_already_uses() {
    let loc = myflat_loc();
    // Resolve is where `validate_symbols` runs, so the error surfaces before
    // any artifact writer touches disk.
    let resolve_err = |decl: crate::lang::SealedClassDecl, item: syn::ItemEnum| -> String {
        let registry =
            Registry::<KotlinMeta>::from_items(vec![(syn::Item::Enum(item), loc.clone())])
                .expect("index items");
        let jni = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(decl));
        match registry.resolve(jni) {
            Ok(_) => String::new(),
            Err(e) => e.to_string(),
        }
    };

    let e: syn::ItemEnum = syn::parse_quote!(
        pub enum Reading {
            Missing,
            Exact(i64),
        }
    );

    // A variant taking the interface's own name — its supertype reference
    // would resolve to the variant itself.
    let e2: syn::ItemEnum = syn::parse_quote!(
        pub enum Reading {
            Reading(i64),
            Exact(i64),
        }
    );
    let msg = resolve_err(crate::sealed_class!(Reading), e2);
    assert!(msg.contains("Reading"), "{msg}");
    assert!(msg.contains("supertype"), "{msg}");

    // Same, through a rename — and the check follows the *Kotlin* name, so
    // renaming the INTERFACE moves the collision with it.
    let msg = resolve_err(
        crate::sealed_class!(Reading)
            .name("Measure")
            .variant(crate::variant!(Exact).name("Measure")),
        e.clone(),
    );
    assert!(msg.contains("Measure"), "{msg}");
    assert!(msg.contains("supertype"), "{msg}");

    // …and a variant named `Reading` is then FINE, because the interface is
    // no longer called that.
    let e3: syn::ItemEnum = syn::parse_quote!(
        pub enum Reading {
            Reading(i64),
            Exact(i64),
        }
    );
    let ok = resolve_err(crate::sealed_class!(Reading).name("Measure"), e3);
    assert!(ok.is_empty(), "expected no error, got: {ok}");
}

/// A variant legitimately named `Companion` does **not** oblige the source
/// crate to rename anything: that name is the generator's own default for
/// the companion object holding `fromParts`, not something Kotlin reserves,
/// so the generator moves its companion instead.
///
/// Verified against kotlinc that this is sound: a *named* companion still
/// emits `fromParts` as a real static on the interface class (the reflection
/// probe reported `static-on-interface=true`), so the rename is invisible to
/// `call_static_method` / `GetStaticMethodID`.
#[test]
fn variant_named_companion_moves_the_companion_not_the_variant() {
    let loc = myflat_loc();
    let emit = |decl: crate::lang::SealedClassDecl, item: syn::ItemEnum, tag: &str| -> String {
        let registry =
            Registry::<KotlinMeta>::from_items(vec![(syn::Item::Enum(item), loc.clone())])
                .expect("index items");
        let jni = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(decl));
        let dir = unique_test_dir(tag);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = registry.resolve(jni).expect("resolve");
        gen.write_kotlin(&dir.join("kotlin"))
            .expect("write_kotlin")
            .iter()
            .map(|p| std::fs::read_to_string(p).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    };

    // The variant keeps the name the Rust source gave it…
    let e: syn::ItemEnum = syn::parse_quote!(
        pub enum Reading {
            Companion(i64),
            Exact(i64),
        }
    );
    let kt = emit(crate::sealed_class!(Reading), e, "sealed_companion_variant");
    let c: String = kt.split_whitespace().collect();
    assert!(
        c.contains("publicdataclassCompanion(publicvalv0:Long):Reading"),
        "{kt}"
    );
    // …and OUR companion object steps aside, keeping `fromParts` on it.
    assert!(c.contains("publiccompanionobjectCompanion_{"), "{kt}");
    assert!(c.contains("funfromParts("), "{kt}");
    assert!(c.contains("0->Companion(companion_v0)"), "{kt}");

    // The escape is deterministic and keeps stepping aside if needed.
    let e2: syn::ItemEnum = syn::parse_quote!(
        pub enum Reading {
            Companion(i64),
            Other(i64),
        }
    );
    let kt = emit(
        crate::sealed_class!(Reading).variant(crate::variant!(Other).name("Companion_")),
        e2,
        "sealed_companion_twice",
    );
    let c: String = kt.split_whitespace().collect();
    assert!(c.contains("publiccompanionobjectCompanion__{"), "{kt}");

    // With no such variant the companion stays anonymous — the ordinary
    // emission is untouched.
    let e3: syn::ItemEnum = syn::parse_quote!(
        pub enum Reading {
            Missing,
            Exact(i64),
        }
    );
    let kt = emit(crate::sealed_class!(Reading), e3, "sealed_companion_plain");
    let c: String = kt.split_whitespace().collect();
    assert!(c.contains("publiccompanionobject{"), "{kt}");
}

/// A payload whose **output** converter did not resolve is a generation
/// error naming it — never a Kotlin surface quietly derived from whichever
/// direction happened to resolve. The property type, its nullability and its
/// wire slot are one decision and come from one entry.
#[test]
fn payload_without_output_converter_is_an_error() {
    let loc = myflat_loc();
    let boom = || {
        let registry = Registry::<KotlinMeta>::from_items(vec![(
            syn::Item::Enum(syn::parse_quote!(
                pub enum Reading {
                    Missing,
                    // `Unmapped` is captured by nothing and declared as
                    // nothing, so no converter resolves for it.
                    Exact(Unmapped),
                }
            )),
            loc.clone(),
        )])
        .expect("index items");
        let jni = JniGen::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(crate::sealed_class!(Reading)));
        let dir = unique_test_dir("sealed_unmapped_payload");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = registry.resolve(jni).expect("resolve");
        let _ = gen.write_kotlin(&dir.join("kotlin"));
    };
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(boom)).expect_err("must fail");
    let msg = err
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    // The message locates the offending payload, not just the type.
    assert!(msg.contains("Reading"), "{msg}");
    assert!(msg.contains("Exact.v0"), "{msg}");
    assert!(msg.contains("OUTPUT converter"), "{msg}");
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

/// A `Vec` of tag-gated groups has variable arity, so it cannot ride the
/// fixed-layout `fromParts` bridge — the same reason `Vec<data class>` is
/// rejected. The guard has to peel `Vec` before asking `type_kind`, which
/// answers about a bare ident and would otherwise report `Vec<Reading>` as
/// `Other` and never reach this error.
#[test]
fn vec_of_sum_is_rejected_as_a_struct_field() {
    let loc = myflat_loc();
    let build = |field_ty: syn::Type| {
        let st: syn::ItemStruct = syn::parse_quote!(
            pub struct Holder {
                pub readings: #field_ty,
            }
        );
        let f: syn::ItemFn = syn::parse_quote!(
            pub fn holder_new() -> Holder {
                unimplemented!()
            }
        );
        let registry = Registry::<KotlinMeta>::from_items(vec![
            (
                syn::Item::Enum(syn::parse_quote!(
                    pub enum Reading {
                        Missing,
                        Exact(i64),
                    }
                )),
                loc.clone(),
            ),
            (syn::Item::Struct(st), loc.clone()),
            (syn::Item::Fn(f), loc.clone()),
        ])
        .expect("index items");
        let jni = JniGen::new().set_package_prefix("io.test.jni").package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .class(crate::data_class!(Holder))
                .fun(crate::fun!(holder_new)),
        );
        let dir = unique_test_dir("sealed_vec_field");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let _ = registry
            .resolve(jni)
            .map(|g| g.write_rust(dir.join("g.rs")));
    };

    for ty in [
        syn::parse_quote!(Vec<Reading>),
        syn::parse_quote!(Option<Vec<Reading>>),
    ] {
        let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| build(ty)))
            .expect_err("Vec<sum> must be rejected");
        let msg = err
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(msg.contains("variable arity"), "{msg}");
        assert!(msg.contains("Reading"), "{msg}");
    }
}

/// A sum that reaches its own type must fail deterministically, never run
/// away. Rust's sizedness rules make an unindirected cycle impossible to
/// declare, so the reachable shapes are the indirected ones — each of which
/// must produce a clear outcome rather than a stack overflow.
#[test]
fn recursive_sum_shapes_fail_deterministically() {
    let loc = myflat_loc();
    let attempt = |variant: proc_macro2::TokenStream, tag: &str| -> Result<(), String> {
        let e: syn::ItemEnum = syn::parse_quote!(
            pub enum Node {
                Leaf(i64),
                #variant
            }
        );
        let st: syn::ItemStruct = syn::parse_quote!(
            pub struct Holder {
                pub node: Node,
            }
        );
        let f: syn::ItemFn = syn::parse_quote!(
            pub fn holder_new() -> Holder {
                unimplemented!()
            }
        );
        let registry = Registry::<KotlinMeta>::from_items(vec![
            (syn::Item::Enum(e), loc.clone()),
            (syn::Item::Struct(st), loc.clone()),
            (syn::Item::Fn(f), loc.clone()),
        ])
        .expect("index items");
        let jni = JniGen::new().set_package_prefix("io.test.jni").package(
            crate::package!()
                .class(crate::sealed_class!(Node))
                .class(crate::data_class!(Holder))
                .fun(crate::fun!(holder_new)),
        );
        let dir = unique_test_dir(tag);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            registry
                .resolve(jni)
                .map(|g| g.write_rust(dir.join("g.rs")))
                .map(|_| ())
                .map_err(|e| e.to_string())
        }));
        match outcome {
            Ok(r) => r,
            Err(p) => Err(p
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "panic".to_string())),
        }
    };

    // `Vec<Node>` — variable arity, rejected with the intended message.
    let msg = attempt(quote::quote!(Branch(Vec<Node>)), "rec_vec").expect_err("must fail");
    assert!(msg.contains("variable arity"), "{msg}");

    // `Box<Node>` — not a bare ident, so it never classifies as a sum; it
    // fails as an unresolvable payload rather than recursing.
    let msg = attempt(quote::quote!(Branch(Box<Node>)), "rec_box").expect_err("must fail");
    assert!(
        !msg.contains("too deep"),
        "expected a resolution failure, not the depth guard: {msg}"
    );
}

/// Build a binding whose declared functions return a sum in every position —
/// bare, `Option`, `Vec`, and as a callback argument — plus one whose variant
/// payload is an opaque handle. Returns `(generated Rust, generated Kotlin)`.
fn sum_returns(tag: &str) -> (String, String) {
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
                pub enum Reading {
                    Missing,
                    Exact(i64),
                    Range { low: i64, high: i64 },
                    Labeled(String, Priority),
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Probe {
                    value: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Lookup {
                    Absent,
                    Found(Probe),
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_one(which: i32) -> Reading {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_maybe(which: i32) -> Option<Reading> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_all(n: i32) -> Vec<Reading> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn look_up(n: i64) -> Lookup {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_each(n: i32, sink: impl Fn(Reading) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_borrowed(p: &Probe) -> &Reading {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_borrowed_maybe(p: &Probe) -> Option<&Reading> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::enum_class!(Priority))
            .class(crate::sealed_class!(Reading))
            .class(crate::sealed_class!(Lookup))
            .class(crate::ptr_class!(Probe))
            .fun(crate::fun!(read_one))
            .fun(crate::fun!(read_maybe))
            .fun(crate::fun!(read_all))
            .fun(crate::fun!(look_up))
            .fun(crate::fun!(read_each))
            .fun(crate::fun!(read_borrowed))
            .fun(crate::fun!(read_borrowed_maybe)),
    );

    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let kotlin = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin")
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    (rust, kotlin)
}

/// A sum in RETURN position crosses as its synthesized tag plus one leaf group
/// per variant, laid side by side — the same wire layout a sum-typed struct
/// field gets, but reassembled by a hoisted builder singleton because there is
/// no parent `fromParts` to ride.
///
/// The builder's parameters are **wire** types (the enum payload is its `Int`
/// discriminant), and every object-shaped group slot is nullable: an inert
/// group is wire-defaulted to JVM null, which a non-null Kotlin parameter would
/// reject in its intrinsic null check before any generated code ran.
#[test]
fn sum_return_builds_through_a_wire_shaped_singleton() {
    let (_, kotlin) = sum_returns("jnigen_sum_return");

    assert!(
        kotlin.contains(
            "public fun run(\n        tag: Int,\n        exact_v0: Long,\n        \
             range_low: Long,\n        range_high: Long,\n        labeled_v0: String?,\n        \
             labeled_v1: Int,\n    ): R"
        ),
        "builder must take the tag plus every group's WIRE slots, inert object \
         slots nullable:\n{kotlin}"
    );
    assert!(
        kotlin.contains(
            "when (tag) { 0 -> Reading.Missing; 1 -> Reading.Exact(exact_v0); \
             2 -> Reading.Range(range_low, range_high); \
             3 -> Reading.Labeled(labeled_v0!!, Priority.fromInt(labeled_v1)); \
             else -> throw IllegalArgumentException(\"Reading: invalid tag $tag\") }"
        ),
        "the singleton picks the live group by tag, re-asserts the inert-nullable \
         slot in its own arm, and rebuilds the enum payload from its \
         discriminant:\n{kotlin}"
    );
    // An out-of-range tag is an error, never a variant.
    assert!(kotlin.contains("Reading: invalid tag $tag"), "{kotlin}");
}

/// A **fixed** builder is implemented only by its hoisted singleton, so the
/// typed `fun interface` and the `asRaw` proxy that would adapt to it are dead
/// public API — and for a sum with a handle payload they are actively wrong:
/// `asRaw` would wrap every group's slot as if all were live, turning an inert
/// group's `0L` sentinel into a handle to nothing. Neither is emitted (#160);
/// the raw twin the singleton implements is.
///
/// Uses the handle-payload fixture because that is the shape with a raw twin
/// at all — when a builder's leaves need no wrapping, `raw_name() == name` and
/// the single interface IS the one JNI calls, so there is nothing dead to drop.
#[test]
fn a_fixed_builder_emits_no_dead_typed_twin() {
    let (_, kotlin) = sum_returns("jnigen_sum_no_dead_twin");

    // The twin the singleton implements and JNI calls.
    assert!(
        kotlin.contains("public fun interface LookupBuilderRaw<out R>"),
        "the raw twin is what exists:\n{kotlin}"
    );
    assert!(
        kotlin.contains("internal val __LookupBuilderRaw: LookupBuilderRaw<Lookup>"),
        "the singleton implements it:\n{kotlin}"
    );
    // The dead surface: the typed declaration and the proxy adapting to it.
    assert!(
        !kotlin.contains("public fun interface LookupBuilder<out R>"),
        "no typed twin for a fixed builder:\n{kotlin}"
    );
    assert!(
        !kotlin.contains("LookupBuilder<R>.asRaw()"),
        "and so no proxy that would wrap an inert group's sentinel:\n{kotlin}"
    );

    // A CALLBACK interface is the opposite case and keeps both: the user
    // implements the typed one, and `asRaw` is how the raw leaves reach it.
    assert!(
        kotlin.contains("public fun interface ReadingCallback"),
        "a callback keeps its typed interface — the user implements it:\n{kotlin}"
    );
    assert!(
        kotlin.contains("ReadingCallback.asRaw()"),
        "and keeps the proxy that feeds it:\n{kotlin}"
    );
}

/// The sealed interface's own `fromParts` is the Kotlin-facing convenience
/// stage C emits, NOT the wire target: its parameters are the variants'
/// property types and its object slots are non-null. The return path therefore
/// reassembles through its own singleton and leaves this factory alone —
/// asserted here so the two do not silently converge.
#[test]
fn sum_from_parts_stays_the_property_typed_convenience() {
    let (_, kotlin) = sum_returns("jnigen_sum_fromparts");
    assert!(
        kotlin.contains("labeled_v0: String,\n            labeled_v1: Priority,"),
        "`fromParts` keeps property types and non-null object slots:\n{kotlin}"
    );
}

/// The Rust side emits ONE `match` over the returned value: the live arm
/// converts its own group's payloads and every other slot takes the wire
/// default. No leaf is an independent expression — that is what a product
/// decomposition does, and a sum is not one.
#[test]
fn sum_return_emits_one_match_with_wire_defaults() {
    let (rust, _) = sum_returns("jnigen_sum_match");
    let at = rust
        .find("fn Java_io_test_jni_JNINative_readOne")
        .expect("extern");
    let body = &rust[at..at + 4000];
    assert!(
        body.contains("match &__out"),
        "one match over the value:\n{body}"
    );
    assert!(
        body.contains("myflat::Reading::Missing =>")
            && body.contains("myflat::Reading::Range { low"),
        "arms bind each variant's payload by pattern:\n{body}"
    );
    // The payload-less arm assigns the tag and defaults every group slot.
    assert!(
        body.contains("jni :: objects :: JObject :: null ()") || body.contains("JObject::null()"),
        "an inert object slot is wire-defaulted to null:\n{body}"
    );
}

/// A sum returned **borrowed** (`&E`, `Option<&E>`). `unfold::returns_type`
/// peels the leading `&` and `wire_fixed_returns` records `by_ref`, so the
/// encoder matches THROUGH the reference rather than moving the value out of
/// the owner. Each live group then clones what it needs, and Kotlin receives an
/// ordinary value with no borrow to track — the borrow never crosses (#161).
#[test]
fn borrowed_sum_return_matches_through_the_reference() {
    let (rust, kotlin) = sum_returns("jnigen_sum_borrowed");

    for extern_fn in ["readBorrowed", "readBorrowedMaybe"] {
        let at = rust
            .find(&format!("fn Java_io_test_jni_JNINative_{extern_fn}"))
            .unwrap_or_else(|| panic!("{extern_fn} extern missing:\n{rust}"));
        let body = &rust[at..at + 4000];
        // The value is borrowed from its owner, so the encoder matches the
        // binding DIRECTLY — `by_ref` is already reflected in `__out`'s type.
        // Asserted exactly: a bare `contains("match __out")` would also accept
        // `match __out2`, and accepting `match &__out` as an alternative would
        // let a regression to double-referencing the borrow pass silently.
        assert!(
            body.contains("match __out {"),
            "{extern_fn}: one match over the borrowed value:\n{body}"
        );
        assert!(
            !body.contains("match &__out"),
            "{extern_fn}: a borrowed return must not take a second reference:\n{body}"
        );
        assert!(
            body.contains("myflat::Reading::Missing =>"),
            "{extern_fn}: arms bind each variant through the reference:\n{body}"
        );
        // A payload reached through a borrow cannot be moved out, so the live
        // group clones it.
        assert!(
            body.contains(".clone()"),
            "{extern_fn}: a borrowed group's payload is cloned, not moved:\n{body}"
        );
    }

    // Kotlin sees a plain value / nullable value — a borrowed sum is not a
    // handle, so there is nothing to close and no lifetime to track.
    assert!(
        kotlin.contains(
            "public fun readBorrowed(p: Probe, onError: JniErrorHandler<Reading>): Reading"
        ),
        "a borrowed sum arrives as an ordinary value:\n{kotlin}"
    );
    assert!(
        kotlin.contains(
            "public fun readBorrowedMaybe(p: Probe, onError: JniErrorHandler<Reading?>): Reading?"
        ),
        "and the optional layer only nulls the whole result:\n{kotlin}"
    );
}

/// `Option` and `Vec` layers ride the existing shape fold, so a sum needs
/// nothing new for them: the optional nulls the whole result and the vector
/// folds element by element, each element rebuilt by the same `when`.
#[test]
fn sum_return_composes_with_option_and_vec() {
    let (_, kotlin) = sum_returns("jnigen_sum_layers");
    assert!(
        kotlin.contains(
            "public fun readMaybe(which: Int, onError: JniErrorHandler<Reading?>): Reading?"
        ),
        "Option<sum> return is a nullable sum:\n{kotlin}"
    );
    assert!(
        kotlin.contains(
            "public fun readAll(n: Int, onError: JniErrorHandler<List<Reading>>): List<Reading>"
        ) && kotlin.contains("__ReadingFolderRawHolder.instance"),
        "Vec<sum> folds through a hoisted appender singleton:\n{kotlin}"
    );
    // Callback argument: the user callback receives the whole reassembled sum
    // while the raw twin still carries the decoupled slots.
    assert!(
        kotlin.contains(
            "public fun interface ReadingCallback {\n    public fun run(reading: Reading)\n}"
        ),
        "the user callback sees the whole sum:\n{kotlin}"
    );
}

/// A variant payload may be an opaque **handle**: it rides its raw `jlong`
/// like any other handle leaf, so its slot stays a primitive (an inert group
/// leaves the `0L` sentinel, never a fabricated handle object).
#[test]
fn sum_return_group_can_own_a_handle() {
    let (_, kotlin) = sum_returns("jnigen_sum_handle");
    assert!(
        kotlin.contains("public fun run(tag: Int, found_v0: Long): R"),
        "a handle payload's group slot is the raw pointer:\n{kotlin}"
    );
    assert!(
        kotlin.contains("1 -> Lookup.Found(Probe(found_v0))"),
        "the live arm wraps the pointer into its typed handle class:\n{kotlin}"
    );
}

/// TWO sums in one callback signature: each contributes its own selector, so
/// the signature-wide dedup renames the second to `tag2` — and the reassembly
/// expressions must follow it, including the `$tag` Kotlin string template in
/// the invalid-tag message. That is why a group's reassembly is stored with
/// positional placeholders and filled at render time rather than captured by
/// name when the group is described.
#[test]
fn two_sum_callback_args_keep_their_own_selectors() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
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
                pub struct Probe {
                    value: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Lookup {
                    Absent,
                    Found(Probe),
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_pair(f: impl Fn(Reading, Lookup) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::sealed_class!(Reading))
            .class(crate::sealed_class!(Lookup))
            .class(crate::ptr_class!(Probe))
            .fun(crate::fun!(read_pair)),
    );
    let dir = unique_test_dir("jnigen_two_sum_cb");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = registry.resolve(jni).expect("resolve");
    let kotlin = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin")
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    // The user callback still sees two whole values.
    assert!(
        kotlin.contains("public fun run(reading: Reading, lookup: Lookup)"),
        "{kotlin}"
    );
    // The raw twin carries both selectors, the second deduped.
    assert!(
        kotlin.contains("tag: Int,") && kotlin.contains("tag2: Int,"),
        "each sum contributes its own selector:\n{kotlin}"
    );
    // Each `when` reads ITS OWN selector — in the dispatch and in the message.
    assert!(
        kotlin.contains("when (tag) { 0 -> Reading.Missing;")
            && kotlin.contains(r#"IllegalArgumentException("Reading: invalid tag $tag")"#),
        "{kotlin}"
    );
    assert!(
        kotlin.contains("when (tag2) { 0 -> Lookup.Absent;")
            && kotlin.contains(r#"IllegalArgumentException("Lookup: invalid tag $tag2")"#),
        "the second sum's reassembly must follow its renamed selector, template \
         included:\n{kotlin}"
    );
}

/// A sum in the **success position of a fallible return** has no lowering, and
/// says so. `Result` returns deliberately keep their whole-value converter (so
/// a fallible factory still hands back a handle), and a sum has no whole-value
/// converter to keep — the decomposition lives on the builder-callback lane the
/// `Result` path does not use.
#[test]
fn sum_in_result_ok_position_is_rejected_with_its_reason() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
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
                pub struct Probe {
                    value: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_try(n: i64) -> Result<Reading, Probe> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::sealed_class!(Reading))
            .class(crate::ptr_class!(Probe))
            .fun(crate::fun!(read_try)),
    );
    let err = registry
        .resolve(jni)
        .expect_err("must be rejected")
        .to_string();
    assert!(
        err.contains("read_try") && err.contains("success position of a fallible return"),
        "the error must name the function and the unsupported position: {err}"
    );
    assert!(
        err.contains("Return `Reading` directly"),
        "…and say what to write instead: {err}"
    );
}

/// A sum in the **error** position of a `Result`, with nothing declared to
/// decompose it. Unlike the other two rejected positions this one RESOLVES —
/// it takes the generic undecomposed-`E` path and routes the `Err` to the
/// plain binding-error channel as `e.to_string()`. Kotlin would receive a
/// `String` from a hierarchy the author explicitly declared, and the generated
/// crate would quietly require `E: Display`, failing downstream in generated
/// code. Emitting something misleading is worse than not emitting.
#[test]
fn undeclared_sum_in_result_error_position_is_rejected() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
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
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_try(n: i64) -> Result<i64, Reading> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::sealed_class!(Reading))
            .fun(crate::fun!(read_try)),
    );
    let err = registry
        .resolve(jni)
        .expect_err("must be rejected")
        .to_string();
    assert!(
        err.contains("read_try") && err.contains("Result<_, Reading>"),
        "the error must name the function and the position: {err}"
    );
    assert!(
        err.contains("e.to_string()") && err.contains("Display"),
        "…and both consequences of the silent path: {err}"
    );
    assert!(
        err.contains("expand_return!(Reading)"),
        "…and what to write instead: {err}"
    );
}

/// A WRAPPED error type (`Result<_, Option<Sum>>`) is where the diagnostic's
/// two type spellings diverge, so it pins which is which: the `Display` bound
/// and the `expand_return!` key are the WHOLE error type (that is what
/// `__e.to_string()` runs on, and what the deconstructor auto-apply matches),
/// while only the "is declared `sealed_class!`" clause names the peeled sum.
/// Getting these backwards would hand the author advice that cannot work.
#[test]
fn the_diagnostic_names_the_whole_error_type_where_it_must() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
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
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_try(n: i64) -> Result<i64, Option<Reading>> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::sealed_class!(Reading))
            .fun(crate::fun!(read_try)),
    );
    let err = registry
        .resolve(jni)
        .expect_err("must be rejected")
        .to_string();
    let compact: String = err.split_whitespace().collect();

    // The whole `E` — the position, the `Display` bound, and the fix.
    assert!(compact.contains("`Result<_,Option<Reading>>`"), "{err}");
    assert!(compact.contains("`Option<Reading>:Display`"), "{err}");
    assert!(compact.contains("expand_return!(Option<Reading>)"), "{err}");
    // …and the peeled sum only where the declaration lives.
    assert!(
        compact.contains("`Reading`isdeclared`sealed_class!`"),
        "{err}"
    );
}

/// The counterpart: a sum error type WITH a type-level deconstructor is the
/// supported shape and must keep resolving — the diagnostic above must not
/// become a blanket ban on sums in the error position.
#[test]
fn declared_sum_in_result_error_position_resolves() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
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
            syn::Item::Fn(syn::parse_quote!(
                pub fn reading_code(v: &Reading) -> i64 {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_try(n: i64) -> Result<i64, Reading> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new()
        .set_package_prefix("io.test.jni")
        .expand(crate::expand_return!(Reading).field(crate::fun!(reading_code)))
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .fun(crate::fun!(read_try)),
        );
    registry
        .resolve(jni)
        .expect("a declared error deconstructor is the supported shape");
}

/// A **slice of sums** as a callback argument has no lowering, and says so.
///
/// Folding a sequence of tag-gated groups into the foreign list needs the
/// element folder that a `Vec<E>` *return* provides; without one the shape
/// would resolve to nothing and surface as "`E` has no output converter",
/// which names the sum rather than the position — misleading, because a sum
/// has no whole-value converter by design. The declaration is rejected up
/// front instead.
#[test]
fn slice_of_sum_callback_arg_is_rejected_with_its_reason() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
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
            syn::Item::Fn(syn::parse_quote!(
                pub fn read_batch(f: impl Fn(&[Reading]) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry = Registry::<KotlinMeta>::from_items(items).expect("index items");
    let jni = JniGen::new().set_package_prefix("io.test.jni").package(
        crate::package!()
            .class(crate::sealed_class!(Reading))
            .fun(crate::fun!(read_batch)),
    );
    let err = registry
        .resolve(jni)
        .expect_err("must be rejected")
        .to_string();
    assert!(
        err.contains("read_batch") && err.contains("slice of a sealed_class"),
        "the error must name the function and the unsupported position: {err}"
    );
    assert!(
        err.contains("impl Fn(Reading)") && err.contains("Vec<Reading>"),
        "…and point at the two shapes that do work: {err}"
    );
}
