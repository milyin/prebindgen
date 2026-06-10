//! [`Code`] — an indentation-aware builder for raw statement/expression
//! bodies. Declarations are modeled ([`super::model`]); their *bodies* are
//! raw Kotlin text structured through this builder, so nesting and
//! indentation are correct by construction without modeling every statement.

/// One body element: a raw line at the current level, or a nested block.
#[derive(Clone, Debug)]
enum Item {
    /// A single raw line (no leading indentation; may be empty).
    Line(String),
    /// `opener` + indented children + `closer` (`}` / `})` / `} finally {`-style
    /// continuations are expressed as sibling blocks).
    Block {
        opener: String,
        children: Code,
        closer: String,
    },
}

/// A sequence of raw lines and nested blocks plus the imports its text
/// references. Rendered with 4-space indentation per nesting level.
#[derive(Clone, Debug, Default)]
pub struct Code {
    items: Vec<Item>,
    /// FQNs referenced only inside this raw text (forwarded to the file's
    /// `ImportSet` at render time).
    imports: Vec<String>,
}

impl Code {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Append one raw line.
    pub fn line(mut self, s: impl Into<String>) -> Self {
        self.items.push(Item::Line(s.into()));
        self
    }

    /// Append a multi-line string: each line lands at the current level.
    pub fn lines(mut self, text: &str) -> Self {
        for l in text.lines() {
            self.items.push(Item::Line(l.to_string()));
        }
        self
    }

    /// Append a nested block: `opener` at the current level, children one
    /// level deeper, then `}`.
    pub fn blk(self, opener: impl Into<String>, f: impl FnOnce(Code) -> Code) -> Self {
        self.blk_with(opener, "}", f)
    }

    /// [`Self::blk`] with an explicit closer (`})`, `},`, …).
    pub fn blk_with(
        mut self,
        opener: impl Into<String>,
        closer: impl Into<String>,
        f: impl FnOnce(Code) -> Code,
    ) -> Self {
        self.items.push(Item::Block {
            opener: opener.into(),
            children: f(Code::new()),
            closer: closer.into(),
        });
        self
    }

    /// Append another `Code`'s items (same level) and imports.
    pub fn push(mut self, other: Code) -> Self {
        self.items.extend(other.items);
        self.imports.extend(other.imports);
        self
    }

    /// Register an FQN referenced only inside this raw text.
    pub fn import(mut self, fqn: impl Into<String>) -> Self {
        self.imports.push(fqn.into());
        self
    }

    /// Build from a flat multi-line blob by recomputing nesting from brace
    /// balance — for migrating bodies composed as unindented strings. Tracks
    /// Kotlin string literals (incl. escapes) and `//` comments so braces
    /// inside them don't count. A line's level drops first by its leading
    /// closers (`}`, `})` …), then the rest of its delta applies to the
    /// following lines.
    pub fn raw_reindent(text: &str) -> Self {
        let mut out = Code::new();
        let mut level: usize = 0;
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() {
                out.items.push(Item::Line(String::new()));
                continue;
            }
            let (leading_close, delta) = brace_profile(line);
            level = level.saturating_sub(leading_close);
            let mut s = String::new();
            for _ in 0..level {
                s.push_str("    ");
            }
            s.push_str(line);
            out.items.push(Item::Line(s));
            // Remaining delta after the leading closers were applied.
            let net = delta + leading_close as i64;
            if net > 0 {
                level += net as usize;
            } else {
                level = level.saturating_sub((-net) as usize);
            }
        }
        // The reindenter produced absolute indentation; mark items as
        // pre-indented by wrapping: we emit them at the caller's level via
        // render(), which prepends the base indent — exactly what we want.
        out
    }

    pub(crate) fn collect_imports(&self, sink: &mut Vec<String>) {
        sink.extend(self.imports.iter().cloned());
        for it in &self.items {
            if let Item::Block { children, .. } = it {
                children.collect_imports(sink);
            }
        }
    }

    /// Render with `level` leading 4-space indents per line.
    pub fn render(&self, level: usize, out: &mut String) {
        for it in &self.items {
            match it {
                Item::Line(l) => {
                    if l.is_empty() {
                        out.push('\n');
                    } else {
                        for _ in 0..level {
                            out.push_str("    ");
                        }
                        out.push_str(l);
                        out.push('\n');
                    }
                }
                Item::Block {
                    opener,
                    children,
                    closer,
                } => {
                    for _ in 0..level {
                        out.push_str("    ");
                    }
                    out.push_str(opener);
                    out.push('\n');
                    children.render(level + 1, out);
                    for _ in 0..level {
                        out.push_str("    ");
                    }
                    out.push_str(closer);
                    out.push('\n');
                }
            }
        }
    }
}

/// `(leading_closers, total_brace_delta)` of one trimmed line, ignoring
/// braces inside string literals and `//` comments. `leading_closers` counts
/// the `}` characters before any opener/content (so `}` and `})` and `} }`
/// prefixes dedent the line itself); `total_brace_delta` is opens − closes
/// over the whole line.
fn brace_profile(line: &str) -> (usize, i64) {
    let mut leading_close = 0usize;
    let mut seen_content = false;
    let mut delta: i64 = 0;
    let mut chars = line.chars().peekable();
    let mut in_str = false;
    let mut in_char = false;
    while let Some(c) = chars.next() {
        if in_str {
            match c {
                '\\' => {
                    let _ = chars.next();
                }
                '"' => in_str = false,
                _ => {}
            }
            continue;
        }
        if in_char {
            match c {
                '\\' => {
                    let _ = chars.next();
                }
                '\'' => in_char = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                seen_content = true;
            }
            '\'' => {
                in_char = true;
                seen_content = true;
            }
            '/' if chars.peek() == Some(&'/') => break,
            '{' => {
                delta += 1;
                seen_content = true;
            }
            '}' => {
                delta -= 1;
                if !seen_content {
                    leading_close += 1;
                }
            }
            c if c.is_whitespace() || c == ')' || c == ',' || c == ';' => {
                // closers like `})` / `},` keep counting as leading
            }
            _ => seen_content = true,
        }
    }
    (leading_close, delta)
}

#[cfg(test)]
mod tests {
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
}
