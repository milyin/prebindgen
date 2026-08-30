use prebindgen_registry::{Conversions, RegistryBuilder};

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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let mut sealed = crate::sealed_class!(Reading);
    if let Some(n) = rename_labeled {
        sealed = sealed.variant(crate::variant!(Labeled).name(n));
    }
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::enum_class!(Priority))
                .class(sealed),
        );

    let dir = unique_test_dir("jnigen_sealed");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
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

    let emit = |item: syn::Item, decl: crate::ClassDecl, tag: &str| {
        let registry =
            crate::test_util::reg_from_items(declare_referenced(vec![(item, loc.clone())]))
                .expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(decl));
        let dir = unique_test_dir(tag);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni.build_with(registry).expect("resolve");
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
        let registry = crate::test_util::reg_from_items(declare_referenced(vec![(
            syn::Item::Enum(syn::parse_quote!(
                pub enum Reading {
                    Missing,
                    Exact(i64),
                }
            )),
            loc.clone(),
        )]))
        .expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::sealed_class!(Reading).variant(crate::variant!(Nope).name("X"))),
            );
        let dir = unique_test_dir("sealed_unknown_variant");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni.build_with(registry).expect("resolve");
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
    let registry = crate::test_util::reg_from_items(declare_referenced(vec![(
        syn::Item::Enum(syn::parse_quote!(
            pub enum Reading {
                Missing,
                Exact(i64),
                Range { low: i64, high: i64 },
            }
        )),
        loc.clone(),
    )]))
    .expect("index items");
    let jni = JniGenBuilder::new()
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
    let gen = jni.build_with(registry).expect("resolve");
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

/// The stored kind's payload merges on reopen, and `.gc_managed()` is
/// sticky-OR: a plain `ptr_class!(T)` reopening a `.gc_managed()` one must not
/// demote the handle. Peer of [`reopened_sealed_class_merges_variant_names`]
/// for the other options-carrying kind — both rules live in
/// `DeclaredKind::merge`, so this pins the order-independence they share.
#[test]
fn reopened_ptr_class_keeps_gc_managed() {
    let loc = myflat_loc();
    let items = || {
        vec![(
            syn::Item::Struct(syn::parse_quote!(
                pub struct Session {
                    pub id: i64,
                }
            )),
            loc.clone(),
        )]
    };
    let gc_managed_of = |first: crate::PtrClassDecl, second: crate::PtrClassDecl| {
        let registry: RegistryBuilder =
            crate::test_util::reg_from_items(declare_referenced(items())).expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(first).class(second));
        drop(registry);
        let key = TypeKey::from_type(&syn::parse_quote!(Session));
        jni.decls
            .types
            .get(&key)
            .expect("declared")
            .opaque()
            .expect("ptr_class")
            .gc_managed
    };

    assert!(gc_managed_of(
        crate::ptr_class!(Session).gc_managed(),
        crate::ptr_class!(Session),
    ));
    assert!(gc_managed_of(
        crate::ptr_class!(Session),
        crate::ptr_class!(Session).gc_managed(),
    ));
    assert!(!gc_managed_of(
        crate::ptr_class!(Session),
        crate::ptr_class!(Session),
    ));
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
    let declare = |first: crate::ClassDecl, second: crate::ClassDecl| {
        let registry: RegistryBuilder =
            crate::test_util::reg_from_items(declare_referenced(items())).expect("index items");
        let _ = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(first).class(second));
        drop(registry);
    };

    // Every conflicting pair among the four declarators is rejected, in both
    // orders — the second declaration is matched against the kind the first
    // one stored, so it does not depend on which came first.
    type MakeDecl = fn() -> crate::ClassDecl;
    let pairs: Vec<(MakeDecl, MakeDecl)> = vec![
        (
            || crate::sealed_class!(Reading).into(),
            || crate::data_class!(Reading).into(),
        ),
        (
            || crate::data_class!(Reading).into(),
            || crate::sealed_class!(Reading).into(),
        ),
        (
            || crate::sealed_class!(Reading).into(),
            || crate::enum_class!(Reading).into(),
        ),
        (
            || crate::enum_class!(Reading).into(),
            || crate::sealed_class!(Reading).into(),
        ),
        (
            || crate::sealed_class!(Reading).into(),
            || crate::ptr_class!(Reading).into(),
        ),
        (
            || crate::ptr_class!(Reading).into(),
            || crate::sealed_class!(Reading).into(),
        ),
        (
            || crate::data_class!(Sample).into(),
            || crate::enum_class!(Sample).into(),
        ),
        (
            || crate::enum_class!(Sample).into(),
            || crate::data_class!(Sample).into(),
        ),
        (
            || crate::ptr_class!(Sample).into(),
            || crate::data_class!(Sample).into(),
        ),
        (
            || crate::data_class!(Sample).into(),
            || crate::ptr_class!(Sample).into(),
        ),
        (
            || crate::ptr_class!(Sample).into(),
            || crate::enum_class!(Sample).into(),
        ),
        (
            || crate::enum_class!(Sample).into(),
            || crate::ptr_class!(Sample).into(),
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
    let resolve_err = |decl: crate::SealedClassDecl, item: syn::ItemEnum| -> String {
        let registry = crate::test_util::reg_from_items(declare_referenced(vec![(
            syn::Item::Enum(item),
            loc.clone(),
        )]))
        .expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(decl));
        match jni.build_with(registry) {
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
    let emit = |decl: crate::SealedClassDecl, item: syn::ItemEnum, tag: &str| -> String {
        let registry = crate::test_util::reg_from_items(declare_referenced(vec![(
            syn::Item::Enum(item),
            loc.clone(),
        )]))
        .expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(decl));
        let dir = unique_test_dir(tag);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni.build_with(registry).expect("resolve");
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
        let registry = crate::test_util::reg_from_items(declare_referenced(vec![(
            syn::Item::Enum(syn::parse_quote!(
                pub enum Reading {
                    Missing,
                    // `Unmapped` is captured by nothing and declared as
                    // nothing, so no converter resolves for it.
                    Exact(Unmapped),
                }
            )),
            loc.clone(),
        )]))
        .expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(crate::package!().class(crate::sealed_class!(Reading)));
        let dir = unique_test_dir("sealed_unmapped_payload");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let gen = jni.build_with(registry).expect("resolve");
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
    let registry = crate::test_util::reg_from_items(declare_referenced(vec![(
        syn::Item::Enum(syn::parse_quote!(
            pub enum Reading {
                Missing,
                Exact(i64),
            }
        )),
        loc.clone(),
    )]))
    .expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!().class(crate::sealed_class!(Reading)));
    let ty: syn::Type = syn::parse_quote!(Reading);
    assert!(matches!(
        jni.decls.type_kind(&registry, &TypeKey::from_type(&ty)),
        crate::jni::classify::TypeKind::Sum
    ));
    let cfg = jni
        .decls
        .types
        .get(&TypeKey::from_type(&ty))
        .expect("declared");
    assert!(cfg.special_decl());
}

/// What the registry holds for a `sealed_class!` sum, in all three parts (#282).
///
/// The `SumTag` selector's `out_ty` is the **sum**, not the `i32` — it names
/// which sum it chooses between. That type is *registered* and *not required*,
/// and those are separate claims a single predicate cannot state:
///
/// * a **cell** says the type entered the pipeline;
/// * a **root** says the binding demands its converter — cleared here, because
///   a sum crosses decomposed and has no whole-value output converter;
/// * an **entry** says one resolved — present INPUT-side (a retained sum plan
///   decodes a whole `JObject`), absent OUTPUT-side, and that asymmetry is the
///   design, not an accident.
///
/// Asserted against a real `Registry` rather than a hand-assembled one, because
/// the claim is about what a *declaration* produces — and that is the limit of
/// what it can claim. It does **not** pin #282: `sealed_class!(Reading)` creates
/// the output cell through `export_type` whether or not the selector registers
/// anything, so this passes against the filtered loop too. The selector's own
/// half is pinned in `core::unfold`'s `sum_return_is_a_fixed_builder_plan`,
/// whose fixture carries the sum as the tag's `out_ty` exactly so it can fail
/// when the leaf stops registering it.
///
/// BLOCKED by the prebindgen-jni crate split: reads `Registry::input_types` /
/// `output_types`, `pub(crate)` fields of `prebindgen_registry::Registry` —
/// reachable when this test lived inside the `prebindgen` crate, not from the
/// separate `prebindgen-jni` crate it moved to. Left in place, not deleted,
/// pending a `prebindgen` accessor for these fields (see the
/// carve-prebindgen-jni report).
#[test]
fn a_sums_registry_cells_are_registered_but_not_required() {
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
                pub fn read_one() -> Reading {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .fun(prebindgen_registry::fun!(read_one)),
        );
    let gen = jni.build_with(registry).expect("resolve");
    let reg = gen.registry();
    let key = TypeKey::from_type(&syn::parse_quote!(Reading));

    let input_root = reg
        .is_root_for_test(Direction::Construct, &key)
        .expect("input cell");
    let output_root = reg
        .is_root_for_test(Direction::Deconstruct, &key)
        .expect("output cell");

    // Registered both ways — the declaration put them there.
    // Required neither way — `boundary_only_types` clears the root, because the
    // sum crosses in pieces.
    assert!(!input_root, "a declared sum crosses decomposed, not whole");
    assert!(!output_root, "a declared sum crosses decomposed, not whole");

    // The asymmetry: Kotlin → Rust decodes a whole `JObject`; Rust → Kotlin is
    // always flattened, so there is nothing to resolve.
    assert!(
        reg.has_entry_for_test(Direction::Construct, &key) == Some(true),
        "the input direction has a whole-object decoder"
    );
    let reading = gen.registry.reading(&key).expect("Reading model type");
    assert!(
        gen.decls
            .in_frag(&reading)
            .expect("Reading whole-object input")
            .is_sum_codec_plan(),
        "the whole-object decoder must retain an unrendered sum plan"
    );
    assert!(
        reg.has_entry_for_test(Direction::Deconstruct, &key) == Some(false),
        "the output direction has none — a sum crosses flattened, always"
    );
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
        let registry = crate::test_util::reg_from_items(declare_referenced(vec![
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
        ]))
        .expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::sealed_class!(Reading))
                    .class(crate::data_class!(Holder))
                    .fun(prebindgen_registry::fun!(holder_new)),
            );
        let dir = unique_test_dir("sealed_vec_field");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let _ = jni
            .build_with(registry)
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
        let registry = crate::test_util::reg_from_items(declare_referenced(vec![
            (syn::Item::Enum(e), loc.clone()),
            (syn::Item::Struct(st), loc.clone()),
            (syn::Item::Fn(f), loc.clone()),
        ]))
        .expect("index items");
        let jni = JniGenBuilder::new()
            .set_package_prefix("io.test.jni")
            .package(
                crate::package!()
                    .class(crate::sealed_class!(Node))
                    .class(crate::data_class!(Holder))
                    .fun(prebindgen_registry::fun!(holder_new)),
            );
        let dir = unique_test_dir(tag);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            jni.build_with(registry)
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

    // `Vec<Node>` — the recipe saying what `Node` is made of reaches `Node`
    // again, and the table refuses it by name before anything is compiled.
    // The same refusal the emitter used to phrase as "variable arity", now
    // stating which crossings close the loop.
    let msg = attempt(quote::quote!(Branch(Vec<Node>)), "rec_vec").expect_err("must fail");
    assert!(msg.contains("reaches its own crossing"), "{msg}");
    assert!(msg.contains("Node"), "{msg}");

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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::enum_class!(Priority))
                .class(crate::sealed_class!(Reading))
                .class(crate::sealed_class!(Lookup))
                .class(crate::ptr_class!(Probe))
                .fun(prebindgen_registry::fun!(read_one))
                .fun(prebindgen_registry::fun!(read_maybe))
                .fun(prebindgen_registry::fun!(read_all))
                .fun(prebindgen_registry::fun!(look_up))
                .fun(prebindgen_registry::fun!(read_each))
                .fun(prebindgen_registry::fun!(read_borrowed))
                .fun(prebindgen_registry::fun!(read_borrowed_maybe)),
        );

    let dir = unique_test_dir(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
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
        kotlin.contains("internal fun interface LookupBuilderRaw<out R>"),
        "the raw twin is what exists:\n{kotlin}"
    );
    // `@get:JvmSynthetic`: the facade getter of a top-level `internal val` is
    // `ACC_PUBLIC`, so without it `ModelKt.get__LookupBuilderRaw().run(0xdead…)`
    // would reconstruct a handle from a forged pointer out of Java.
    assert!(
        kotlin.contains(
            "@get:JvmSynthetic\ninternal val __LookupBuilderRaw: LookupBuilderRaw<Lookup>"
        ),
        "the singleton implements it, hidden from javac:\n{kotlin}"
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

/// The registry-composed Choice converter emits one `match` over the returned
/// value: the live arm invokes its child chains and every other arm
/// intermediate takes the adapter's inactive value. The exported wrapper calls
/// that converter instead of reconstructing the sum walk itself.
#[test]
fn sum_return_emits_one_match_with_wire_defaults() {
    let (rust, _) = sum_returns("jnigen_sum_match");
    let choice = generated_function(&rust, &["v : myflat :: Reading"], &["match v"]);
    let choice_name = choice.sig.ident.to_string();
    let body = choice.block.to_token_stream().to_string();
    let compact: String = body.split_whitespace().collect();
    assert!(
        body.contains("match v"),
        "one match over the value:\n{body}"
    );
    assert!(
        compact.contains("myflat::Reading::Missing=>")
            && compact.contains("myflat::Reading::Range{low:"),
        "arms bind each variant's payload by pattern:\n{body}"
    );
    assert!(
        body.contains("jni :: objects :: JObject :: null ()") || body.contains("JObject::null()"),
        "an inactive object arm is wire-defaulted to null:\n{body}"
    );

    let wrapper_at = rust
        .find("fn Java_io_test_jni_JNINative_readOne")
        .expect("extern");
    let wrapper = &rust[wrapper_at..];
    assert!(
        wrapper.contains(&choice_name),
        "the wrapper delegates to the Choice converter:\n{wrapper}"
    );
}

/// A sum returned **borrowed** (`&E`, `Option<&E>`). `unfold::returns_type`
/// peels the leading `&` and `wire_fixed_returns` records `by_ref`, so the
/// registry Choice chain matches THROUGH the reference rather than moving the
/// value out of the owner. Each live group then clones what it needs, and
/// Kotlin receives an ordinary value with no borrow to track — the borrow never
/// crosses (#161).
#[test]
fn borrowed_sum_return_matches_through_the_reference() {
    let (rust, kotlin) = sum_returns("jnigen_sum_borrowed");

    for extern_fn in ["readBorrowed", "readBorrowedMaybe"] {
        let at = rust
            .find(&format!("fn Java_io_test_jni_JNINative_{extern_fn}"))
            .unwrap_or_else(|| panic!("{extern_fn} extern missing:\n{rust}"));
        let body = &rust[at..at + 4000];
        // The exact borrowed Choice crossing is frozen during planning, so the
        // wrapper delegates rather than rebuilding the match locally.
        assert!(
            body.contains("__jni_out_convert_"),
            "{extern_fn}: delegates to the borrowed Choice chain:\n{body}"
        );
        assert!(
            !body.contains("match &__out"),
            "{extern_fn}: a borrowed return must not take a second reference:\n{body}"
        );
    }

    let rc: String = rust.split_whitespace().collect();
    assert!(
        rc.contains("v:&myflat::Reading") && rc.contains("matchv{") && rc.contains(".clone()"),
        "the retained Choice chain matches through the borrow and clones payloads:\n{rust}"
    );

    // Kotlin sees a plain value / nullable value — a borrowed sum is not a
    // handle, so there is nothing to close and no lifetime to track.
    assert!(
        kotlin.contains(
            "public fun readBorrowed(p: Probe, onError: JniErrorHandler<Reading?>): Reading?"
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
            "public fun readAll(n: Int, onError: JniErrorHandler<List<Reading>?>): List<Reading>?"
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

/// A payload's layers are each carried across, not skipped.
///
/// The raw builder reassembles a variant from wire slots, and between a slot and
/// the property there can be a collection, an `Option`, and the leaf the wrap
/// knows about. Applying the wrap straight to the slot emitted Kotlin that does
/// not compile in all three shapes below (#429) — nulls dropped, and an element
/// conversion applied to the list. Nothing caught it because the Rust half
/// compiles and no test asked for these payloads.
///
/// The fixture has more variants than the defect had shapes, because each
/// answer needs its control: distributing over a collection is right only when
/// its elements convert, a payload with no layer must come out exactly as it
/// did, and the layers occur in **both orders** and at **any depth** —
/// `Vec<Option<T>>`, `Option<Vec<T>>` and `Vec<Vec<Option<T>>>` are all
/// accepted, so the walk over them recurses instead of unrolling.
#[test]
fn a_payload_carries_its_option_and_collection_layers() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Handle {
                    v: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Probe {
                    Count(Option<u64>),
                    Held(Option<Handle>),
                    Many(Vec<Option<u64>>),
                    Owned(Handle),
                    Blob(Vec<u8>),
                    Names(Vec<String>),
                    Values(Option<Vec<Option<u64>>>),
                    Nested(Vec<Vec<Option<u64>>>),
                    Labels(Option<Vec<String>>),
                    Empty,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn probe() -> Probe {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(Handle))
                .class(crate::sealed_class!(Probe))
                .fun(prebindgen_registry::fun!(probe)),
        );

    let dir = unique_test_dir("jnigen_sum_payload_layers");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let kotlin = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin")
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    // `Option<u64>`: JVM null is the absent case, so the conversion is
    // null-safe rather than applied through it.
    assert!(
        kotlin.contains("Probe.Count(count_v0?.toULong())"),
        "an optional scalar payload keeps its null:\n{kotlin}"
    );
    // `Option<Handle>`: the absence rides the handle's own niche — a `Box`
    // pointer is never 0 — so the slot stays the primitive the encoder writes
    // and the wrap tests the sentinel, exactly as a data-class field does.
    assert!(
        kotlin.contains("Probe.Held(if (held_v0 == 0L) null else Handle.fromRawPtr(held_v0))"),
        "an optional handle payload reads its niche:\n{kotlin}"
    );
    // …and the slot it reads that from is a primitive, which is the half the
    // JNI descriptor is built from.
    assert!(
        kotlin.contains("held_v0: Long,"),
        "an optional handle slot is not boxed:\n{kotlin}"
    );
    // `Vec<Option<u64>>`: the list slot is un-inerted, and the element
    // conversion runs per element — the nulls are inside the list, not on it.
    assert!(
        kotlin.contains("Probe.Many(many_v0!!.map { it?.toULong() })"),
        "a collection payload converts element by element:\n{kotlin}"
    );
    // The control: a payload that is neither optional nor a collection is
    // un-inerted and wrapped exactly as before.
    assert!(
        kotlin.contains("Probe.Owned(Handle.fromRawPtr(owned_v0))"),
        "a plain handle payload is unchanged:\n{kotlin}"
    );
    // …and so is a run of values whose elements need no conversion. There is
    // nothing to distribute there, and mapping the identity over one would
    // replace the property's type rather than preserve it: `Vec<u8>` surfaces
    // as a `ByteArray`, and `ByteArray.map { it }` is a `List<Byte>` (#432
    // review).
    assert!(
        kotlin.contains("Probe.Blob(blob_v0!!)"),
        "a byte-array payload is passed straight through:\n{kotlin}"
    );
    // Layers nest, so the walk over them recurses rather than unrolling a fixed
    // two: with `Vec<Vec<Option<u64>>>` the element of the outer run is another
    // run, and the leaf's conversion belongs at the bottom. The inner lambda
    // names its parameter because a nested `it` would shadow the outer one.
    assert!(
        kotlin.contains("Probe.Nested(nested_v0!!.map { it.map { __e1 -> __e1?.toULong() } })"),
        "nested runs distribute at every level:\n{kotlin}"
    );
    // A run under an `Option` is still a run: the layers occur in both orders,
    // and looking for the collection without peeling the `Option` first found
    // none — so the element conversion landed on the list (#432 review).
    assert!(
        kotlin.contains("Probe.Values(values_v0?.map { it?.toULong() })"),
        "an optional run distributes, and keeps its own null:\n{kotlin}"
    );
    // …and the identity guard holds under an `Option` too: nothing to
    // distribute means no `?.map` either.
    assert!(
        kotlin.contains("Probe.Labels(labels_v0)"),
        "an optional run of unconverted elements is passed through:\n{kotlin}"
    );
    assert!(
        kotlin.contains("Probe.Names(names_v0!!)"),
        "a list payload whose elements convert to themselves is too:\n{kotlin}"
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
        kotlin.contains("1 -> Lookup.Found(Probe.fromRawPtr(found_v0))"),
        "the live arm wraps the pointer into its typed handle class:\n{kotlin}"
    );
}

/// The same handle payload as a **data-class field** — the position the
/// return/callback coverage above does not reach, and the one
/// `ReplyStruct { result: ReplyResult, .. }` needs (both `ReplyResult`
/// alternatives carry handles).
///
/// The group slot stays the raw `jlong` and the live arm wraps it into the
/// typed handle class, exactly as in return position — the parent's own
/// `fromParts` inlining the `when`.
///
/// Two consequences of this position, both asserted below so they cannot
/// change silently:
///
/// * The container **is** `AutoCloseable` and cascades, exactly as it does for
///   a plain handle field (#218). It did not, until the field's type stopped
///   deciding who frees the handle: `destructible()` matched only a
///   `Projection`, so the same handle was the container's to close when the
///   field was spelled `Probe` and nobody's when it was spelled `Lookup`. What
///   makes the two spellings agree is that `Lookup` is itself `AutoCloseable`
///   — the `when` over the alternatives lives in the sum, so the container
///   emits the same one-line cascade either way.
/// * A sum field keeps its parent on the fixed-builder path: since #616 the
///   decomposition states it as a tag over one group per alternative, so the
///   value crosses as leaves and no JVM object is built for it. It used to
///   decline, which cost the parent an object — a slower shape, not a broken
///   one — and this test passed either way, since what it pins is the cascade.
#[test]
fn a_data_class_field_may_be_a_sum_carrying_a_handle() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
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
            syn::Item::Struct(syn::parse_quote!(
                pub struct Holder {
                    pub id: i64,
                    pub outcome: Lookup,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn holder_new(id: i64) -> Holder {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(Probe))
                .class(crate::sealed_class!(Lookup))
                .class(crate::data_class!(Holder))
                .fun(prebindgen_registry::fun!(holder_new)),
        );
    let dir = unique_test_dir("jnigen_sum_handle_field");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("gen.rs")).expect("write_rust"))
        .expect("read rust");
    let kotlin = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin")
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    // The tag keeps the `__` marker; a group slot is prefixed with its field
    // by the single-underscore nesting convention (`mode_periodicQueries_period`).
    assert!(
        kotlin.contains("outcome__tag: Int") && kotlin.contains("outcome_found_v0: Long"),
        "the selector plus a raw-pointer group slot, both prefixed by the field:\n{kotlin}"
    );
    assert!(
        kotlin.contains("Lookup.Found(Probe.fromRawPtr(outcome_found_v0))"),
        "the parent's fromParts inlines the `when` and wraps the pointer:\n{kotlin}"
    );
    assert!(
        kotlin.contains("public data class Holder(val id: Long, val outcome: Lookup)"),
        "the field surfaces as the typed sum:\n{kotlin}"
    );
    // The cascade, exactly as a plain handle field gets — the container closes
    // what its field reaches (#218). The `when` is NOT here: `Lookup` closes
    // itself, so the container's body is the same one line it would emit for a
    // handle-typed field.
    let kc: String = kotlin.split_whitespace().collect();
    assert!(
        kotlin.contains(
            "public data class Holder(val id: Long, val outcome: Lookup) : AutoCloseable"
        ),
        "a sum-carried handle is the container's to close, like a plain handle \
         field:\n{kotlin}"
    );
    assert!(
        kc.contains("overridefunclose(){outcome.close()}"),
        "the container closes the field as a whole; the walk into the \
         alternatives lives in `Lookup`:\n{kotlin}"
    );
    // And that walk: the sum is itself `AutoCloseable`, its handle-carrying
    // alternative closes the payload, the others are no-ops.
    assert!(
        kc.contains("publicsealedinterfaceLookup:AutoCloseable"),
        "the sum owns the cascade its containers delegate to:\n{kotlin}"
    );
    assert!(
        kc.contains("dataclassFound(publicvalv0:Probe):Lookup{overridefunclose(){v0.close()}}"),
        "the live alternative closes its payload:\n{kotlin}"
    );
    assert!(
        kc.contains("dataobjectAbsent:Lookup{overridefunclose(){}}"),
        "an alternative holding nothing native overrides with an empty body — \
         Kotlin requires the member, and the emptiness is the statement:\n{kotlin}"
    );
    assert!(
        rust.contains("Lookup::Found") && rust.contains("Lookup::Absent"),
        "Rust matches the field's sum, filling every group's slots:\n{rust}"
    );
}

/// The **third** spelling of the same handle: reached through a nested data
/// class rather than through a sum or directly. Unfiled alongside #218 but the
/// same `_ => None`, and the same argument settles it — swapping a field's type
/// for a struct that holds the handle must not move the free onto the consumer.
///
/// Nothing here is special-cased: `Inner` is `AutoCloseable` because its own
/// field is a handle, and `Outer` cascades into it with the same one-liner it
/// would emit for a handle field. The recursion is one level of
/// [`PlanFieldKind::destructible`], not a second mechanism.
#[test]
fn a_data_class_field_may_be_a_nested_data_class_carrying_a_handle() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Probe {
                    value: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Inner {
                    pub probe: Probe,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Outer {
                    pub id: i64,
                    pub inner: Inner,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn outer_new(id: i64) -> Outer {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(Probe))
                .class(crate::data_class!(Inner))
                .class(crate::data_class!(Outer))
                .fun(prebindgen_registry::fun!(outer_new)),
        );
    let dir = unique_test_dir("jnigen_nested_handle_field");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let kotlin = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin")
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    assert!(
        kc.contains(
            "dataclassInner(valprobe:Probe):AutoCloseable{overridefunclose(){probe.close()}"
        ),
        "the direct handle field cascades, as it always did:\n{kotlin}"
    );
    assert!(
        kc.contains("dataclassOuter(valid:Long,valinner:Inner):AutoCloseable"),
        "…and so does the container that only REACHES the handle:\n{kotlin}"
    );
    assert!(
        kc.contains("overridefunclose(){inner.close()}"),
        "the outer closes the field as a whole — the walk is the inner \
         class's:\n{kotlin}"
    );
}

/// The predicate must not over-fire: a sum whose alternatives own nothing
/// native stays a plain `sealed interface`, with no `AutoCloseable` and no
/// `close()` on its variant classes. Otherwise every sum in every binding would
/// grow a lifecycle it has no use for.
#[test]
fn a_sum_owning_nothing_native_is_not_closeable() {
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
                pub struct Gauge {
                    pub id: i64,
                    pub reading: Reading,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn gauge_new(id: i64) -> Gauge {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .class(crate::data_class!(Gauge))
                .fun(prebindgen_registry::fun!(gauge_new)),
        );
    let dir = unique_test_dir("jnigen_sum_no_handle");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let kotlin = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin")
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    assert!(
        kc.contains("publicsealedinterfaceReading{"),
        "an `i64` payload owns nothing, so the sum takes no lifecycle:\n{kotlin}"
    );
    assert!(
        !kc.contains("interfaceReading:AutoCloseable"),
        "…and does not implement `AutoCloseable`:\n{kotlin}"
    );
    assert!(
        !kc.contains("Exact(publicvalv0:Long):Reading{overridefunclose"),
        "…so its variant classes carry no `close()` either:\n{kotlin}"
    );
    assert!(
        !kc.contains("Gauge(valid:Long,valreading:Reading):AutoCloseable"),
        "and the container holding it stays non-closeable:\n{kotlin}"
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .class(crate::sealed_class!(Lookup))
                .class(crate::ptr_class!(Probe))
                .fun(prebindgen_registry::fun!(read_pair)),
        );
    let dir = unique_test_dir("jnigen_two_sum_cb");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
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
    // Compared whitespace-insensitively: `Lookup` carries a handle, so its
    // reassembly is bound to a local and the call moves inside a `try` (below).
    let kc: String = kotlin.split_whitespace().collect();
    assert!(
        kc.contains("when(tag){0->Reading.Missing;")
            && kotlin.contains(r#"IllegalArgumentException("Reading: invalid tag $tag")"#),
        "{kotlin}"
    );
    assert!(
        kc.contains("when(tag2){0->Lookup.Absent;")
            && kotlin.contains(r#"IllegalArgumentException("Lookup: invalid tag $tag2")"#),
        "the second sum's reassembly must follow its renamed selector, template \
         included:\n{kotlin}"
    );
    // …and exactly the sum that reaches a handle is the proxy's to close after
    // `run` (#218). `Reading`'s payload is an `i64`, so it stays inline in the
    // call and nothing closes it; `Lookup.Found` carries a `Probe`, so it binds
    // to a local closed in a `finally` — the close-unless-taken contract a
    // handle passed DIRECTLY to a callback has always had.
    assert!(
        kc.contains("val__own0=when(tag2)") && kc.contains("finally{__own0.close()}"),
        "a handle reached through a sum arg is closed after `run`, exactly as a \
         handle arg is:\n{kotlin}"
    );
    assert!(
        !kc.contains("__own1"),
        "a sum whose payload owns nothing native is not bound and not closed:\n{kotlin}"
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .class(crate::ptr_class!(Probe))
                .fun(prebindgen_registry::fun!(read_try)),
        );
    let err = jni
        .build_with(registry)
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .fun(prebindgen_registry::fun!(read_try)),
        );
    let err = jni
        .build_with(registry)
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .fun(prebindgen_registry::fun!(read_try)),
        );
    let err = jni
        .build_with(registry)
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .expand(
            prebindgen_registry::expand_return!(Reading)
                .field(prebindgen_registry::fun!(reading_code)),
        )
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .fun(prebindgen_registry::fun!(read_try)),
        );
    jni.build_with(registry)
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .fun(prebindgen_registry::fun!(read_batch)),
        );
    let err = jni
        .build_with(registry)
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

/// A sum named with a **raw** identifier generates, rather than aborting — and
/// specifically through the OUTPUT encoder, which is the site that broke.
///
/// `r#type` is a legal `#[prebindgen]` enum name, and `TypeId::name` stores it
/// as the string `"r#type"`. The sum encoder rebuilds an ident from that name
/// to spell the `match`'s path, and `Ident::new` *panics* on that spelling
/// rather than erroring, so the whole generation aborted (#278 review).
///
/// The sum is a value-form FIELD here, not just a declared type: that is what
/// puts it through `encode_sum_group`. A first version of this test declared
/// the sum and a callback only — the raw name appeared in the generated file
/// via the *input* converter, the assertion passed, and reverting the fix left
/// it passing. A regression test that survives its own regression is worse than
/// none, so this one asserts the output-side `match` path.
#[test]
fn a_raw_named_sum_generates() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum r#type {
                    Missing,
                    Exact(i64),
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZThingStruct {
                    pub reading: r#type,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_to_struct(t: &ZThing) -> ZThingStruct {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_emit(cb: impl Fn(ZThing) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(ZThing))
                .class(crate::sealed_class!(r#type))
                .fun(prebindgen_registry::fun!(z_emit)),
        )
        .expand(
            prebindgen_registry::expand_return!(ZThing)
                .fields(prebindgen_registry::fields!(z_thing_to_struct)),
        );

    let dir = unique_test_dir("jnigen_raw_sum");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write_rust"))
        .expect("read rust");

    // The output-side match, spelled with the raw name — this is what
    // `encode_sum_group` emits, and what `Ident::new` could not produce.
    assert!(
        rust.contains("myflat::r#type::Missing") && rust.contains("myflat::r#type::Exact"),
        "the sum encoder matches the raw-named enum by its real path:\n{rust}"
    );
}

/// A payload-less alternative keeps its own delimiters in the output match.
///
/// The arm builder branched on `variant.fields.first()`, so an alternative with
/// no fields took the `None` arm and was spelled bare — `myflat::Shape::Parens`
/// for `Parens()`, which is **E0533**: a zero-field tuple or struct variant
/// still needs its delimiters in pattern position. An empty alternative has no
/// first field to branch on, exactly as the empty struct in #302 had no field to
/// refuse.
///
/// `Alternative::spell` is the one place those delimiters are chosen — its own
/// doc says "for match patterns and constructors alike, in either direction" —
/// and this is the pattern half of that sentence.
#[test]
fn empty_sum_alternatives_keep_their_own_pattern_delimiters() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Shape {
                    Bare,
                    Parens(),
                    Braces {},
                    Full(i64),
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn make_shape() -> Shape {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::sealed_class!(Shape))
                .fun(prebindgen_registry::fun!(make_shape)),
        );

    let dir = unique_test_dir("jnigen_empty_sum_alternatives");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust = std::fs::read_to_string(gen.write_rust(dir.join("g.rs")).expect("write_rust"))
        .expect("read rust");
    let rc: String = rust.split_whitespace().collect();

    // Each alternative is matched with the delimiters Rust demands for it.
    assert!(
        rc.contains("myflat::Shape::Bare=>"),
        "a unit alternative is matched bare:\n{rust}"
    );
    assert!(
        rc.contains("myflat::Shape::Parens()=>"),
        "an empty TUPLE alternative keeps its parens:\n{rust}"
    );
    assert!(
        rc.contains("myflat::Shape::Braces{}=>"),
        "an empty STRUCT alternative keeps its braces:\n{rust}"
    );
    // And the bare spelling must not appear for the two that cannot take it.
    assert!(
        !rc.contains("myflat::Shape::Parens=>") && !rc.contains("myflat::Shape::Braces=>"),
        "neither empty alternative may be matched bare:\n{rust}"
    );
}

/// The invariant the whole #218 fix rests on: the **two forms** of "does this
/// reach an owned handle" must agree.
///
/// [`PlanFieldKind::destructible`] answers for a classified plan field;
/// [`type_close_strategy`] answers for a bare type, because two callers (the
/// sealed emitter, the callback proxy) hold no plan. They are two independent
/// walks over the same tree, and the doc-comment's "same tree" is a claim, not
/// a construction — `classify_field` classifies a `Sum` *before* consulting
/// `output_entry` while `type_close_strategy` asks `output_entry` first, and
/// only one of them refuses anything.
///
/// A disagreement is a Kotlin compile error in one direction (`field.close()`
/// on a type that is not `AutoCloseable`) and a silent leak in the other — #218
/// reappearing at the seam its own fix introduced. So this asserts equality
/// over every field of every declared type in a set covering all the shapes
/// that can reach a handle: direct, through a sum, through a nested data class,
/// optional, sequence, borrowed, and none of the above.
#[test]
fn a_types_close_answer_matches_its_plans() {
    use crate::jni::struct_plan::{classify_field, type_close_strategy};

    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
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
                    Failed(String),
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Tally {
                    None,
                    Count(i64),
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Inner {
                    pub probe: Probe,
                }
            )),
            loc.clone(),
        ),
        // Every field position at once: a handle direct and optional, a sum
        // that reaches one and a sum that does not, a nested data class that
        // reaches one, and plain scalars that reach nothing.
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Everything {
                    pub id: i64,
                    pub probe: Probe,
                    pub maybe_probe: Option<Probe>,
                    pub outcome: Lookup,
                    pub tally: Tally,
                    pub inner: Inner,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn everything_new(id: i64) -> Everything {
                    unimplemented!()
                }
            )),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::ptr_class!(Probe))
                .class(crate::sealed_class!(Lookup))
                .class(crate::sealed_class!(Tally))
                .class(crate::data_class!(Inner))
                .class(crate::data_class!(Everything))
                .fun(prebindgen_registry::fun!(everything_new)),
        );
    let gen = jni.build_with(registry).expect("resolve");
    let (ext, registry) = (&gen.decls, &gen.registry);

    // Every field of every declared type — struct fields and sum payloads
    // alike, since a payload goes through the same `classify_field`.
    let mut checked = 0usize;
    for ty in registry.flat().types() {
        let (owner, fields): (&syn::Ident, Vec<&prebindgen_registry::flat::Field>) = match ty {
            prebindgen_registry::flat::Type::Struct(st) => (&st.name, st.fields.iter().collect()),
            prebindgen_registry::flat::Type::Variant(sum) => (
                &sum.name,
                sum.alternatives
                    .iter()
                    .flat_map(|a| a.fields.iter())
                    .collect(),
            ),
            _ => continue,
        };
        for field in fields {
            let path = format!("{owner}.{}", field.member().to_token_stream());
            // A field `classify_field` refuses has no plan answer to compare
            // against — the documented, deliberate asymmetry.
            let Some(kind) = classify_field(ext, registry, &field.ty, &path, 0) else {
                continue;
            };
            assert_eq!(
                kind.destructible().is_some(),
                type_close_strategy(ext, registry, &field.ty, 0).is_some(),
                "the two forms disagree about `{path}`: a plan field and its \
                 bare type must reach the same handles",
            );
            checked += 1;
        }
    }
    // The loop asserting nothing would pass vacuously.
    assert!(
        checked >= 10,
        "expected every declared field to be compared, saw {checked}"
    );
    // …and the fixture must actually contain both answers, or agreement is
    // trivial.
    let everything = match registry
        .flat()
        .declared_type("Everything")
        .expect("Everything is declared")
    {
        prebindgen_registry::flat::Type::Struct(st) => st,
        _ => panic!("Everything is a struct"),
    };
    let closes: Vec<String> = everything
        .fields
        .iter()
        .filter(|f| type_close_strategy(ext, registry, &f.ty, 0).is_some())
        .map(|f| f.name.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(
        closes,
        vec!["probe", "maybe_probe", "outcome", "inner"],
        "the fixture must cover reaching a handle four ways AND not reaching one"
    );
}

/// A `sealed_class` states a recipe saying what it hands out, and it says exactly
/// what the leaf synthesis it will replace says.
///
/// One `jint` naming which alternative is live, then one group of values per
/// alternative laid beside the others — names, types, groups and the pattern
/// binding each is reached through. The fixture carries a unit alternative
/// (which contributes nothing), a positional payload, a two-field payload and a
/// named-field payload, so the slot naming is exercised in every form it takes.
#[test]
fn a_sealed_class_states_what_it_hands_out() {
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
                    Tagged(String, Priority),
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
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::enum_class!(Priority))
                .class(crate::sealed_class!(Reading))
                .fun(prebindgen_registry::fun!(read_one)),
        );
    let gen = jni.build_with(registry).expect("resolve");

    let composed = gen
        .out_lines_for_test("Reading")
        .expect("Reading states a deconstructing parts recipe");
    assert_eq!(
        composed,
        vec![
            "tag: Reading <- tag @[]",
            "exact_v0: i64 <- Exact.v0 @[1]",
            "range_low: i64 <- Range.low @[2]",
            "range_high: i64 <- Range.high @[2]",
            "tagged_v0: String <- Tagged.v0 @[3]",
            "tagged_v1: Priority <- Tagged.v1 @[3]",
        ],
        "the recipe states the tag, then one group per alternative",
    );
    // And it is the same list the synthesis it replaces produces, which is what
    // makes pointing the emitters at the fragment a move rather than a rewrite.
    assert_eq!(
        composed,
        gen.walk_lines_for_test("Reading").expect("the walk"),
        "the recipe and the leaf synthesis disagree",
    );
}

/// A `data_class` states a recipe saying what it hands out, and a field that is
/// itself one contributes its own values under the parent's name and chain.
///
/// The recipe declines for the **whole** value rather than per field, because
/// the emitter it feeds does: one field it cannot state sends the whole object
/// down the whole-value `fromParts` path. What is left to decline is structural
/// — a repeated nested class, a field the model does not hold as a named one —
/// and a handle field is not one of those: it is an ordinary leaf whose own
/// output conversion carries it as the `jlong` the receiver adopts, which is
/// what the whole-object encode always emitted for it (#602). So `Wrapped` and
/// `Opaque`-bearing `Guarded` both compose, and both agree with the synthesis
/// they will replace.
#[test]
fn a_data_class_states_what_it_hands_out() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Inner {
                    pub count: i64,
                    pub label: String,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Wrapped {
                    pub id: i64,
                    pub inner: Inner,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Opaque {
                    pub v: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Guarded {
                    pub id: i64,
                    pub held: Opaque,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn wrapped_new() -> Wrapped {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn guarded_new() -> Guarded {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Inner))
                .class(crate::data_class!(Wrapped))
                .class(crate::ptr_class!(Opaque))
                .class(crate::data_class!(Guarded))
                .fun(prebindgen_registry::fun!(wrapped_new))
                .fun(prebindgen_registry::fun!(guarded_new)),
        );
    let gen = jni.build_with(registry).expect("resolve");

    assert_eq!(
        gen.out_lines_for_test("Wrapped")
            .expect("Wrapped states a deconstructing parts recipe"),
        vec![
            "id: i64 <- id @[]",
            "inner__count: i64 <- inner.count @[]",
            "inner__label: String <- inner.label @[]",
        ],
        "a nested data class contributes its own values, under its field's name",
    );
    // Compared as options, because declining is still one of the two answers:
    // a recipe that states nothing and a synthesis that returns nothing are the
    // same claim. No type here makes it any more — the remaining declines are
    // structural — so what this pins is that the two agree either way.
    for short in ["Wrapped", "Inner", "Guarded"] {
        assert_eq!(
            gen.out_lines_for_test(short),
            gen.walk_lines_for_test(short),
            "the recipe and the leaf synthesis disagree on {short}",
        );
    }
    // A handle field is an ordinary leaf: the handle crosses as the `jlong`
    // the receiver adopts, which is what the whole-object encode has always
    // emitted for it. Refusing it here kept such a struct off fixed-builder
    // delivery for no reason the encode shares (#602).
    assert_eq!(
        gen.out_lines_for_test("Guarded")
            .expect("a handle field decomposes like any other leaf"),
        vec!["id: i64 <- id @[]", "held: Opaque <- held @[]"],
    );
}

/// A decomposed return is a **site** asking for its type's `parts` recipe, and it
/// composes to what the expansion plan delivers.
///
/// The registry needed nothing new to express this: `Ask::Recipe` already lets
/// a site name a recipe, and a `parts` fragment already occupies several wires.
/// The two facts stages 3 and 4 established, meeting.
#[test]
fn a_decomposed_return_is_a_site_asking_for_the_parts_row() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Reading {
                    Missing,
                    Exact(i64),
                    Range { low: i64, high: i64 },
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
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .fun(prebindgen_registry::fun!(read_one))
                .fun(prebindgen_registry::fun!(read_maybe)),
        );
    let gen = jni.build_with(registry).expect("resolve");

    let expected = vec![
        "tag: Reading <- tag @[]",
        "exact_v0: i64 <- Exact.v0 @[1]",
        "range_low: i64 <- Range.low @[2]",
        "range_high: i64 <- Range.high @[2]",
    ];
    assert_eq!(
        gen.return_site_lines_for_test("read_one")
            .expect("read_one's return is a site"),
        expected,
        "the site composes the sum's parts",
    );
    // The shape rides the delivery, not the recipe: an `Option<Reading>` return
    // hands out the same values, and the optional is what the builder's
    // nullability says.
    assert_eq!(
        gen.return_site_lines_for_test("read_maybe")
            .expect("read_maybe's return is a site"),
        expected,
        "an optional return decomposes the same type",
    );
    // And it is what the expansion plan delivers, which is what the emitters
    // read today.
    assert_eq!(
        gen.return_site_lines_for_test("read_one"),
        gen.plan_lines_for_test("read_one"),
        "the site and the expansion plan disagree",
    );
}

/// Generate one Optional crossing through the real recipe driver and Rust
/// writer. Keeping the crossing isolated lets a refused composed planner fall
/// back without another function making the whole fixture ambiguous.
fn optional_wrapper_rust(
    ty: syn::Type,
    direction: prebindgen_registry::recipe::Direction,
    case: &str,
) -> Result<String, String> {
    use prebindgen_registry::recipe::Direction;

    let loc = myflat_loc();
    let gen = match direction {
        Direction::Construct => {
            let items: Vec<(syn::Item, SourceLocation)> = vec![(
                syn::Item::Fn(syn::parse_quote!(
                    pub fn cross(v: #ty) {
                        unimplemented!()
                    }
                )),
                loc.clone(),
            )];
            let registry = crate::test_util::reg_from_items(declare_referenced(items))
                .map_err(|e| e.to_string())?;
            JniGenBuilder::new()
                .set_package_prefix("io.test.jni")
                .package(crate::package!().fun(prebindgen_registry::fun!(cross)))
                .build_with(registry)
                .map_err(|e| e.to_string())?
        }
        Direction::Deconstruct => {
            let items: Vec<(syn::Item, SourceLocation)> = vec![
                (
                    syn::Item::Struct(syn::parse_quote!(
                        pub struct Holder {
                            pub reading: #ty,
                        }
                    )),
                    loc.clone(),
                ),
                (
                    syn::Item::Fn(syn::parse_quote!(
                        pub fn z_sample_to_holder(s: &ZSample) -> Holder {
                            unimplemented!()
                        }
                    )),
                    loc.clone(),
                ),
                (
                    syn::Item::Fn(syn::parse_quote!(
                        pub fn z_sample_sub(cb: impl Fn(ZSample) + Send + Sync + 'static) {
                            unimplemented!()
                        }
                    )),
                    loc.clone(),
                ),
            ];
            let registry = crate::test_util::reg_from_items(declare_referenced(items))
                .map_err(|e| e.to_string())?;
            JniGenBuilder::new()
                .set_package_prefix("io.test.jni")
                .package(
                    crate::package!()
                        .class(crate::ptr_class!(ZSample))
                        .fun(prebindgen_registry::fun!(z_sample_sub)),
                )
                .expand(
                    prebindgen_registry::expand_return!(ZSample)
                        .fields(prebindgen_registry::fields!(z_sample_to_holder)),
                )
                .build_with(registry)
                .map_err(|e| e.to_string())?
        }
    };
    let dir = unique_test_dir(case);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = gen
        .write_rust(dir.join("g.rs"))
        .map_err(|e| e.to_string())?;
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

/// Compile and render one Choice crossing through the real recipe driver.
fn choice_wrapper_plan(
    ty: syn::Type,
    direction: prebindgen_registry::recipe::Direction,
) -> Result<(bool, String, String), String> {
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
                pub fn seed(n: i32, sink: impl Fn(Reading) + Send + Sync + 'static) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).map_err(|e| e.to_string())?;
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::sealed_class!(Reading))
                .fun(prebindgen_registry::fun!(seed)),
        )
        .build_with(registry)
        .map_err(|e| e.to_string())?;
    gen.parts_plan_for_test(ty, direction)
}

fn product_wrapper_plan(
    ty: syn::Type,
    direction: prebindgen_registry::recipe::Direction,
) -> Result<(bool, String, String), String> {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Sample {
                    pub label: String,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn seed(v: Sample) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).map_err(|e| e.to_string())?;
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Sample))
                .fun(prebindgen_registry::fun!(seed)),
        )
        .build_with(registry)
        .map_err(|e| e.to_string())?;
    gen.parts_plan_for_test(ty, direction)
}

fn sequence_wrapper_plan(
    ty: syn::Type,
    direction: prebindgen_registry::recipe::Direction,
) -> Result<(bool, String, String), String> {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn seed(v: Vec<String>) {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).map_err(|e| e.to_string())?;
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!().fun(prebindgen_registry::fun!(seed)))
        .build_with(registry)
        .map_err(|e| e.to_string())?;
    gen.crossing_plan_for_test(ty, direction)
}

/// Sequence composition has two registry-owned outcomes: an executable list
/// converter for a one-slot element, and a non-rendering shape fragment when a
/// site already folds a multi-slot element on the Kotlin side.
#[test]
fn borrowed_sequences_keep_their_registry_plan_or_specialized_shape() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::parse_quote!(
                pub struct Payload {
                    pub id: i64,
                    pub label: String,
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub fn seed(cb: impl Fn(&[Payload]) + Send + Sync + 'static) {
                    unimplemented!()
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub fn take_labels(labels: &[String]) {
                    unimplemented!()
                }
            ),
            loc,
        ),
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let gen = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Payload))
                .fun(prebindgen_registry::fun!(seed))
                .fun(prebindgen_registry::fun!(take_labels)),
        )
        .build_with(registry)
        .expect("resolve");
    let (specialized, ident, rendered) = gen
        .crossing_plan_for_test(
            syn::parse_quote!(&[Payload]),
            prebindgen_registry::recipe::Direction::Deconstruct,
        )
        .expect("plan");
    assert!(
        specialized && ident.starts_with("__jni_out_convert_"),
        "the flattened callback fold needs only its Sequence shape: {ident}\n{rendered}"
    );

    let (specialized, borrowed_ident, borrowed_rendered) = gen
        .crossing_plan_for_test(
            syn::parse_quote!(&[String]),
            prebindgen_registry::recipe::Direction::Construct,
        )
        .expect("labels plan");
    let compact: String = borrowed_rendered.split_whitespace().collect();
    assert!(
        !specialized
            && borrowed_ident.starts_with("__jni_in_convert_")
            && compact.contains(
                "Result<::std::vec::Vec<::std::string::String>,__JniErr>",
            )
            && compact.contains("letmut__sequence_values")
            && compact.contains("__jni_in_convert_"),
        "the borrowed input must retain the executable registry Sequence plan: {borrowed_ident}\n{borrowed_rendered}"
    );

    let (specialized, owned_ident, owned_rendered) = gen
        .crossing_plan_for_test(
            syn::parse_quote!(Vec<String>),
            prebindgen_registry::recipe::Direction::Construct,
        )
        .expect("owned labels plan");
    assert!(
        !specialized
            && owned_ident == borrowed_ident
            && owned_rendered == borrowed_rendered,
        "owned Vec and borrowed slice inputs must share their produced-carrier converter:\nborrowed {borrowed_ident}: {borrowed_rendered}\nowned {owned_ident}: {owned_rendered}"
    );
}

/// Every composed planner refuses a wrapper it cannot peel and put back, and
/// refuses to read one it does not own.
///
/// Optional goes through public binding resolution and `write_rust`. Product
/// and Choice ask their declared `parts` rows, while Sequence asks its default
/// row; each then renders the JFunction the actual planner retained. Choice
/// needs the explicit row because sealed-class inputs intentionally retain a
/// whole-JObject compatibility row.
///
/// Both directions exercise a supported `Box` first, proving that the composed
/// Optional route is available rather than merely asserting every wrapper is
/// refused. Their unsupported twins then exercise each half of the gate:
///
/// * `&Box<Option<String>>` going out: reading peels with `*v`, so a chain
///   composed over a shared borrow moves the `Option` out of the `Box` — E0507
///   in the consumer's crate, not here.
/// * `Cow<'_, Option<String>>` coming in: `Cow` has no build operation, so
///   planning would succeed and `JSource::build` would hit its `expect` while
///   `write_rust` renders — a panic in the generator, after every check has
///   passed.
///
/// Product, Sequence and Optional have no legacy JNI representation for the
/// refused input spellings, so resolution names the unsupported type. Choice
/// does have one: its refused crossings must retain a non-emitting marker, while an
/// owned `Box<Reading>` renders the chain converter.
#[test]
fn a_composed_chain_refuses_a_wrapper_it_cannot_peel() {
    use prebindgen_registry::recipe::Direction;

    let output = optional_wrapper_rust(
        syn::parse_quote!(Box<Option<String>>),
        Direction::Deconstruct,
        "jnigen_chain_wrapper_output",
    )
    .expect("an owned Box output composes");
    let output_compact: String = output.split_whitespace().collect();
    assert!(
        output_compact.contains("fn__jni_out_convert_"),
        "the output uses the composed Optional converter:\n{output}"
    );
    assert!(
        output_compact.contains("match*v{"),
        "the owned wrapper is peeled inside that converter:\n{output}"
    );

    let borrowed = optional_wrapper_rust(
        syn::parse_quote!(&'static Box<Option<String>>),
        Direction::Deconstruct,
        "jnigen_chain_wrapper_borrowed_output",
    );
    let borrowed = borrowed.expect_err("a shared Box output must refuse composition");
    assert!(
        borrowed.contains("unresolved prebindgen output type")
            && borrowed.contains("Box < Option < String > >"),
        "the refusal names the unsupported borrowed output: {borrowed}"
    );

    let input = optional_wrapper_rust(
        syn::parse_quote!(Box<Option<String>>),
        Direction::Construct,
        "jnigen_chain_wrapper_input",
    )
    .expect("an owned Box input composes");
    let input_compact: String = input.split_whitespace().collect();
    assert!(
        input_compact.contains("fn__jni_in_convert_"),
        "the input uses the composed Optional converter:\n{input}"
    );
    assert!(
        input_compact.contains("Box::new({ifv.is_null()"),
        "the canonical Optional is put back in its owned wrapper:\n{input}"
    );

    let cow = optional_wrapper_rust(
        syn::parse_quote!(Cow<'static, Option<String>>),
        Direction::Construct,
        "jnigen_chain_wrapper_cow_input",
    );
    let cow = cow.expect_err("a Cow input must refuse composition");
    assert!(
        cow.contains("unresolved prebindgen input type")
            && cow.contains("Cow < 'static , Option < String > >"),
        "the refusal names the unsupported Cow input: {cow}"
    );

    let (legacy, ident, rendered) =
        product_wrapper_plan(syn::parse_quote!(Box<Sample>), Direction::Construct)
            .expect("an owned Box Product composes");
    assert!(
        !legacy
            && ident.starts_with("__jni_in_convert_")
            && rendered
                .replace(' ', "")
                .contains("Box::new(myflat::Sample{"),
        "the Product parts row must retain its wrapper-aware plan: {ident}\n{rendered}"
    );
    let (legacy, ident, _) = product_wrapper_plan(
        syn::parse_quote!(Cow<'static, Sample>),
        Direction::Construct,
    )
    .expect("a Cow Product keeps its compatibility fragment");
    assert!(
        legacy && ident.starts_with("__jni_in_convert_"),
        "the Cow Product must retain the legacy parts fragment, got {ident}"
    );

    let (legacy, ident, rendered) =
        sequence_wrapper_plan(syn::parse_quote!(Box<Vec<String>>), Direction::Construct)
            .expect("an owned Box Sequence composes");
    assert!(
        !legacy
            && ident.starts_with("__jni_in_convert_")
            && rendered
                .replace(' ', "")
                .contains("Box::new({let__sequence_list="),
        "the Sequence default row must retain its wrapper-aware plan: {ident}\n{rendered}"
    );
    let refused = sequence_wrapper_plan(
        syn::parse_quote!(Cow<'static, Vec<String>>),
        Direction::Construct,
    )
    .expect_err("a Cow Sequence must refuse composition");
    assert!(
        refused.contains("no JNI representation for this run"),
        "the Sequence refusal names its missing compatibility path: {refused}"
    );

    let (legacy, ident, rendered) =
        choice_wrapper_plan(syn::parse_quote!(Box<Reading>), Direction::Deconstruct)
            .expect("an owned Box Choice composes");
    let rendered_compact: String = rendered.split_whitespace().collect();
    assert!(!legacy, "the owned Choice must select its composed plan");
    assert!(
        ident.starts_with("__jni_out_convert_") && rendered_compact.contains("match*v{"),
        "the owned Choice renders its wrapper-aware converter: {ident}\n{rendered}"
    );

    for (label, ty, direction) in [
        (
            "borrowed",
            syn::parse_quote!(&'static Box<Reading>),
            Direction::Deconstruct,
        ),
        (
            "cow",
            syn::parse_quote!(Cow<'static, Reading>),
            Direction::Construct,
        ),
    ] {
        let (legacy, ident, _) = choice_wrapper_plan(ty, direction)
            .unwrap_or_else(|error| panic!("the {label} Choice keeps its fallback: {error}"));
        assert!(
            legacy && ident.starts_with("__jni_"),
            "the {label} Choice must retain the legacy parts fragment, got {ident}"
        );
    }
}
