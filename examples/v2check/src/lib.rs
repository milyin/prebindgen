//! Compiles what the v2 engine emitted for `docs/v2`'s worked example, and
//! checks it against the specification pages that describe it.
//!
//! Two things happen here that a unit test cannot do. `cargo build` compiles
//! the generated wrappers against the real source module and the real `jni`
//! crate, so an emission that is well-formed text but not valid Rust fails.
//! And `cargo test` calls the generated C entry point, so the wrapper is
//! executed rather than only read.

// Generator findings belong to the generator, not to this file.
#![allow(clippy::all)]

// The module the generated code reaches the source items through — `source::`
// in every generated path, which is `build.rs`'s `source_module`.
pub mod source;

// The two generated files, at the crate root so `source::` resolves from them.
include!(env!("V2CHECK_C"));
include!(env!("V2CHECK_JNI"));

// The adapters `build.rs` generated with, compiled again for the tests that run
// the engine themselves. One copy, two compilations — and this one exercises
// the JNI half, while `build.rs` uses both.
#[cfg(test)]
#[path = "adapters.rs"]
#[allow(dead_code)]
mod adapters;

#[cfg(test)]
mod tests {
    /// The generated C entry point, called the way a C program calls it.
    ///
    /// `Stamp` here is the generated `repr(C)` aggregate, not `source::Stamp`:
    /// same spelling, different type, which is the whole point of the two
    /// declarations.
    #[test]
    fn the_c_wrappers_compute_and_place_their_arguments() {
        let stamp = crate::Stamp {
            secs: 12,
            nanos: 34,
        };
        assert_eq!(crate::stamp_sum(stamp), 46);
        // Addition would survive the two fields arriving in the wrong order;
        // subtraction is what says `secs` reached `secs`.
        let stamp = crate::Stamp {
            secs: 12,
            nanos: 34,
        };
        assert_eq!(crate::stamp_delta(stamp), -22);
        // A wrapper that delivers nothing still runs.
        let stamp = crate::Stamp {
            secs: 12,
            nanos: 34,
        };
        crate::stamp_show(stamp);
    }

    /// Every scalar, through the generated C wrappers, by value.
    ///
    /// Each member is a distinct number, so a wrapper that read one member's
    /// bytes as another's — or read a narrow member at the wrong width — comes
    /// back with the wrong answer rather than merely compiling.
    #[test]
    fn the_c_wrappers_carry_every_scalar() {
        fn scalars(flag: bool) -> crate::Scalars {
            crate::Scalars {
                flag: ::core::mem::MaybeUninit::new(flag),
                tiny: -1,
                small: -2,
                medium: -3,
                large: 4,
                byte: 5,
                word: 6,
                dword: 7,
                qword: u64::MAX,
                single: 8.0,
                double: 9.5,
            }
        }
        assert_eq!(crate::scalars_large(scalars(true)), 4);
        assert_eq!(crate::scalars_qword(scalars(true)), u64::MAX);
        assert_eq!(crate::scalars_double(scalars(true)), 9.5);
        // -1 - 2 - 3 + 5 + 6 + 7 + 8
        assert_eq!(crate::scalars_narrow(scalars(true)), 20);
        // A `bool` crosses as storage: the wrapper normalizes it on the way
        // in and stores it back on the way out.
        for flag in [true, false] {
            let returned = crate::scalars_flag(scalars(flag));
            assert_eq!(unsafe { returned.assume_init() }, !flag);
        }
        // The two scalars whose width is the platform's, which only C carries.
        assert_eq!(crate::size_shift(-2, 10), 8);
    }

    /// Every Rust item the specification's emit pages show is emitted, token
    /// for token.
    ///
    /// The expectation is read out of the pages themselves rather than copied
    /// here, so a chapter and this engine cannot drift apart quietly: if either
    /// changes, this fails.
    #[test]
    fn the_generated_rust_is_what_the_specification_shows() {
        let c = std::fs::read_to_string(env!("V2CHECK_C")).expect("the C file");
        let jni = std::fs::read_to_string(env!("V2CHECK_JNI")).expect("the JNI file");
        for (page, generated) in [
            ("examples/struct/07-emit.c.md", &c),
            ("examples/fn/07-emit.c.md", &c),
            ("examples/fn/07-emit.jni.md", &jni),
        ] {
            let expected = fence(page, "rust");
            let generated = items(generated);
            for item in items(&expected) {
                assert!(
                    generated.contains(&item),
                    "{page} shows an item the engine did not emit:\n{item}\n\ngenerated:\n{}",
                    generated.join("\n")
                );
            }
        }
    }

    /// The Kotlin the JNI adapter's own writer produced is what the
    /// specification's emit pages show, in that order and no more than once.
    ///
    /// Order and multiplicity both matter: a writer that emitted the class
    /// twice, or a second `object Bindings` for the second method, would
    /// satisfy a set of lines while producing Kotlin that does not compile.
    #[test]
    fn the_generated_kotlin_is_what_the_specification_shows() {
        let kotlin = std::fs::read_to_string(env!("V2CHECK_KOTLIN")).expect("the Kotlin file");
        let emitted: Vec<&str> = kotlin.lines().map(str::trim).collect();
        for page in [
            "examples/struct/07-emit.jni.md",
            "examples/fn/07-emit.jni.md",
        ] {
            let mut next = 0;
            for line in fence(page, "kotlin").lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let found = emitted[next..]
                    .iter()
                    .position(|emitted| *emitted == line)
                    .unwrap_or_else(|| {
                        panic!(
                            "{page} shows a Kotlin line the writer did not produce in order: \
                             {line}\n\n{kotlin}"
                        )
                    });
                next += found + 1;
            }
        }
        for once in [
            "package example",
            "data class Stamp(val secs: Long, val nanos: Long)",
            "object Bindings {",
        ] {
            assert_eq!(
                emitted.iter().filter(|line| **line == once).count(),
                1,
                "`{once}` must appear exactly once:\n{kotlin}"
            );
        }
        // Both methods live in that one object.
        for method in [
            "external fun sum(stamp: Stamp): Long",
            "external fun delta(stamp: Stamp): Long",
        ] {
            assert_eq!(
                emitted.iter().filter(|line| **line == method).count(),
                1,
                "`{method}` must appear exactly once:\n{kotlin}"
            );
        }
    }

    /// Every requested element was emitted, and the report says so.
    #[test]
    fn both_targets_report_every_generated_element_as_emitted() {
        // Two records and the functions over them: three over `Stamp`, five
        // over `Scalars`. C carries `size_shift` on top of that; the JVM has
        // no type of the platform's width, so it does not.
        for (target, emitted) in [("c", 11), ("jni", 10)] {
            let report = report(target);
            assert_eq!(
                report.matches("\"outcome\": \"emitted\"").count(),
                emitted,
                "{target} report: {report}"
            );
        }
    }

    /// Every declared element the two targets could not generate is reported
    /// as skipped, with the capability that would unblock it.
    ///
    /// All three are declared deliberately: a record whose field has no
    /// carrier, one the model lowers to an opaque declaration, and one with no
    /// fields at all. An adapter that quietly emitted any of them would produce
    /// an empty `repr(C)` aggregate crossing an `extern "C"` boundary, or a
    /// Kotlin data class with no properties — neither of which exists.
    #[test]
    fn what_neither_target_can_carry_is_reported_rather_than_emitted() {
        for (target, element, capability) in [
            ("c", "type:Pair", "unsupported.type.not_a_record"),
            ("c", "type:Marker", "unsupported.c.empty_aggregate"),
            ("c", "fn:marker_value", "unsupported.c.empty_aggregate"),
            ("jni", "type:Reading", "unsupported.jni.carrier"),
            // `usize`/`isize` have no stable JVM type, and the function taking
            // them is declared in both bindings so the report says which one
            // carries them.
            ("jni", "fn:size_shift", "unsupported.jni.carrier"),
            ("jni", "type:Marker", "unsupported.jni.empty_class"),
            ("jni", "fn:marker_value", "unsupported.jni.empty_class"),
        ] {
            let report = report(target);
            let entry = report
                .lines()
                .collect::<Vec<_>>()
                .windows(8)
                .find(|window| window[0].contains(element))
                .map(|window| window.join("\n"))
                .unwrap_or_else(|| panic!("{target} report has no entry for {element}"));
            assert!(
                entry.contains("\"outcome\": \"skipped\"") && entry.contains(capability),
                "{element} should be skipped with {capability}:\n{entry}"
            );
        }
        // Nothing partial reaches the file either: no empty aggregate, no
        // wrapper taking one, no Kotlin class with no properties.
        for file in [
            env!("V2CHECK_C"),
            env!("V2CHECK_JNI"),
            env!("V2CHECK_KOTLIN"),
        ] {
            let generated = std::fs::read_to_string(file).expect("the generated file");
            assert!(
                !generated.contains("Marker"),
                "{file} mentions a record neither target can carry:\n{generated}"
            );
        }
    }

    /// Renaming a Kotlin class moves it in the declaration and in every
    /// signature that mentions it.
    ///
    /// The class name is configuration, so it is read from one table by both;
    /// a signature that spelled the Rust type instead would name a class the
    /// file does not declare.
    #[test]
    fn renaming_a_kotlin_class_moves_every_mention_of_it() {
        use prebindgen_registry_v2::{generate, BindingRequests, DeclaredElement, ElementKind};

        use crate::adapters::{JniClasses, JniPolicy, JniTarget};

        let location = prebindgen::SourceLocation {
            crate_name: Some("v2check".to_string()),
            ..Default::default()
        };
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("source.rs"),
        )
        .expect("read src/source.rs");
        let items: Vec<(syn::Item, prebindgen::SourceLocation)> = syn::parse_file(&text)
            .expect("src/source.rs parses")
            .items
            .into_iter()
            .map(|item| (item, location.clone()))
            .collect();
        let model = prebindgen_flat::flat::Flat::builder()
            .items(items)
            .build()
            .expect("the fixture builds a model");

        let classes = JniClasses::in_package("example").with("Stamp", "Timestamp");
        let mut requests =
            BindingRequests::new("jni", syn::parse_quote!(source), JniPolicy::Scalar);
        let stamp = requests.policy(JniPolicy::DataClass {
            rust: "Stamp".to_string(),
        });
        requests.type_policies.insert("Stamp".to_string(), stamp);
        let sum = requests.policy(JniPolicy::Function {
            placement: "example.Bindings.sum".to_string(),
        });
        requests.output(
            DeclaredElement::new(
                ElementKind::Type,
                "Stamp",
                "example.Timestamp",
                "data_class",
            ),
            stamp,
        );
        requests.output(
            DeclaredElement::new(
                ElementKind::Function,
                "stamp_sum",
                "example.Bindings.sum",
                "function",
            ),
            sum,
        );
        let generation = generate(model, &JniTarget::new(classes), requests, "v2check")
            .expect("the renamed binding plans");
        let kotlin = crate::adapters::write_kotlin(&generation);
        assert!(
            kotlin.contains("data class Timestamp(val secs: Long, val nanos: Long)"),
            "{kotlin}"
        );
        assert!(
            kotlin.contains("external fun sum(stamp: Timestamp): Long"),
            "the signature must name the class, not the Rust type:\n{kotlin}"
        );
    }

    /// One target's report, as JSON.
    fn report(target: &str) -> String {
        std::fs::read_to_string(
            std::path::Path::new(env!("V2CHECK_C"))
                .parent()
                .unwrap()
                .join(format!("{target}-report.json")),
        )
        .expect("the report")
    }

    /// The first fenced block of `language` on a specification page.
    fn fence(page: &str, language: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/v2")
            .join(page);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let opening = format!("```{language}");
        let mut block = String::new();
        let mut inside = false;
        for line in text.lines() {
            if inside {
                if line.trim_end() == "```" {
                    return block;
                }
                block.push_str(line);
                block.push('\n');
            } else if line.trim_end() == opening && block.is_empty() {
                inside = true;
            }
        }
        panic!("{} has no ```{language} block", path.display());
    }

    /// The items of a Rust file, as normalized token text, with imports left
    /// out: which module a name is imported from is the binding crate's
    /// business, not the engine's.
    fn items(text: &str) -> Vec<String> {
        use quote::ToTokens;
        syn::parse_file(text)
            .expect("generated Rust parses")
            .items
            .iter()
            .filter(|item| !matches!(item, syn::Item::Use(_)))
            // A trailing comma before a closing brace is the pretty printer's
            // choice, not a difference in what was generated.
            .map(|item| item.to_token_stream().to_string().replace(" , }", " }"))
            .collect()
    }
}
