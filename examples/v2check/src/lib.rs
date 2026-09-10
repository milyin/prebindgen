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
    fn the_c_wrapper_computes_the_sum() {
        let stamp = crate::Stamp {
            secs: 12,
            nanos: 34,
        };
        assert_eq!(crate::stamp_sum(stamp), 46);
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
    /// specification's emit pages show.
    #[test]
    fn the_generated_kotlin_is_what_the_specification_shows() {
        let kotlin = std::fs::read_to_string(env!("V2CHECK_KOTLIN")).expect("the Kotlin file");
        for page in [
            "examples/struct/07-emit.jni.md",
            "examples/fn/07-emit.jni.md",
        ] {
            for line in fence(page, "kotlin").lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                assert!(
                    kotlin.lines().any(|emitted| emitted.trim() == line),
                    "{page} shows a Kotlin line the writer did not produce: {line}\n\n{kotlin}"
                );
            }
        }
    }

    /// Both requested elements were emitted, and the report says so.
    #[test]
    fn both_targets_report_two_emitted_elements() {
        for target in ["c", "jni"] {
            let report = std::fs::read_to_string(
                std::path::Path::new(env!("V2CHECK_C"))
                    .parent()
                    .unwrap()
                    .join(format!("{target}-report.json")),
            )
            .expect("the report");
            assert!(
                report.matches("\"outcome\": \"emitted\"").count() == 2,
                "{target} report: {report}"
            );
        }
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
