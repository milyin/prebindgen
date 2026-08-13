//! Does the emitted Rust become a C header a caller can use?
//!
//! rustc accepting the file is the wrong finish line for the C target. What a C
//! consumer gets is a **header**, produced from that file by `cbindgen`, and the
//! two can disagree: an `extern "C"` signature rustc is perfectly happy with can
//! be one cbindgen skips, renders as an opaque it never defines, or cannot name
//! at all. A cell whose function is missing from the header is a cell no C
//! program can call.
//!
//! # The receipt
//!
//! Not "cbindgen returned `Ok`" — it returns `Ok` for a header that declares
//! nothing. The receipt is that **the wrapper this cell exists to export is
//! declared in the output**, which is the weakest claim a C caller actually
//! depends on.
//!
//! The pinned cbindgen version is the one `examples/example-cbindgen` pins, so
//! this judges the same header that example produces rather than a different
//! tool's opinion of the same Rust.

use std::panic::AssertUnwindSafe;

/// What became of one cell's header.
pub enum Header {
    /// cbindgen declared the exported wrapper.
    Declared,
    /// cbindgen ran and the wrapper is not in the output.
    Missing,
    /// cbindgen refused the file, or panicked on it.
    Failed(String),
}

impl Header {
    pub fn is_ok(&self) -> bool {
        matches!(self, Header::Declared)
    }

    /// What to print when it is not `Declared`.
    pub fn detail(&self) -> Option<String> {
        match self {
            Header::Declared => None,
            Header::Missing => {
                Some("cbindgen produced a header that does not declare the wrapper".to_string())
            }
            Header::Failed(why) => Some(why.clone()),
        }
    }
}

/// Run cbindgen over one cell's emitted Rust.
///
/// `exported` is the symbol a C caller would link against — the wrapper's name
/// after the binding's own mangling, since that is what ends up in the header.
pub fn generate(emitted: &str, exported: &str) -> Header {
    let dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => return Header::Failed(format!("creating a temporary directory: {e}")),
    };
    let src = dir.path().join("generated.rs");
    if let Err(e) = std::fs::write(&src, emitted) {
        return Header::Failed(format!("writing the emitted Rust: {e}"));
    }

    // cbindgen parses source it was not given a crate for, and is entitled to
    // give up loudly. A panic here is an answer about the cell, exactly as it is
    // for the generators themselves.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
        cbindgen::Builder::new()
            .with_src(&src)
            .with_language(cbindgen::Language::C)
            .generate()
            .map(|bindings| {
                let mut header = Vec::new();
                bindings.write(&mut header);
                String::from_utf8_lossy(&header).into_owned()
            })
    }));
    std::panic::set_hook(previous);

    match outcome {
        Ok(Ok(header)) => {
            if declares(&header, exported) {
                Header::Declared
            } else {
                Header::Missing
            }
        }
        Ok(Err(e)) => Header::Failed(format!("cbindgen: {e:?}")),
        Err(payload) => Header::Failed(format!("cbindgen panicked: {}", panic_text(payload))),
    }
}

/// Whether the header declares `exported` as a function.
///
/// The name followed by an open parenthesis, not merely the name: cbindgen
/// carries doc comments through, so a header that only *mentions* the wrapper
/// would otherwise count as declaring it — a receipt that passes on prose is
/// not a receipt.
fn declares(header: &str, exported: &str) -> bool {
    header.match_indices(exported).any(|(at, _)| {
        let before_is_boundary = header[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_');
        let after = header[at + exported.len()..].trim_start();
        before_is_boundary && after.starts_with('(')
    })
}

fn panic_text(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "a non-string payload".to_string()
    }
}
