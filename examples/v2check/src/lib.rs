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
    /// twice, or a second `object JNINative` for the second method, would
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
            "public data class Stamp(val secs: Long, val nanos: Long)",
            "internal object JNINative {",
        ] {
            assert_eq!(
                emitted.iter().filter(|line| **line == once).count(),
                1,
                "`{once}` must appear exactly once:\n{kotlin}"
            );
        }
        // Every native method lives in that one object, and each has the
        // function a caller uses.
        for method in [
            "external fun stampSum(stamp: Stamp): Long",
            "external fun stampDelta(stamp: Stamp): Long",
            "public fun stampSum(stamp: Stamp): Long = JNINative.stampSum(stamp)",
            "public fun stampDelta(stamp: Stamp): Long = JNINative.stampDelta(stamp)",
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
        for target in ["c", "jni"] {
            let report = report(target);
            // The record, and the three functions over it.
            assert_eq!(
                report.matches("\"outcome\": \"emitted\"").count(),
                4,
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
        use prebindgen_jni::pipeline::Pipeline;

        let location = prebindgen::SourceLocation {
            crate_name: Some("source".to_string()),
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

        let generation = prebindgen_jni::JniGen::builder()
            .items(items)
            .set_package_prefix("example")
            .package(
                prebindgen_jni::package!()
                    .class(prebindgen_jni::data_class!(Stamp).name("Timestamp"))
                    .fun(prebindgen_registry::fun!(stamp_sum)),
            )
            .build_with(Pipeline::V2)
            .expect("the renamed binding plans");
        let dir = std::env::temp_dir().join(format!("v2check-rename-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let written = generation.write_kotlin(&dir).expect("write the Kotlin");
        let kotlin = std::fs::read_to_string(&written[0]).expect("the Kotlin file");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            kotlin.contains("data class Timestamp(val secs: Long, val nanos: Long)"),
            "{kotlin}"
        );
        assert!(
            kotlin.contains("fun stampSum(stamp: Timestamp): Long"),
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
