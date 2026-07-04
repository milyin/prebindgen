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
        Self::reindent_inner(text, false)
    }

    /// Like [`Self::raw_reindent`], but additionally breaks any line that would
    /// exceed [`super::render::MAX_SIGNATURE_WIDTH`] across multiple lines,
    /// one call argument (or lambda parameter) per line — the body-side analogue
    /// of the width-aware *signature* layout in [`super::render`]. The brace
    /// delta that drives nesting is still computed from the original (un-broken)
    /// line, so the wrapping never disturbs the surrounding block structure.
    pub fn raw_reindent_wrapped(text: &str) -> Self {
        Self::reindent_inner(text, true)
    }

    fn reindent_inner(text: &str, wrap: bool) -> Self {
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
            if wrap {
                wrap_line(line, level, &mut out);
            } else {
                push_line(&mut out, level, line);
            }
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

/// Width budget for breaking a body line, matching the signature layout.
const MAX_LINE_WIDTH: usize = super::render::MAX_SIGNATURE_WIDTH;

/// Push `text` as a `Line` carrying `level` 4-space indents (the same absolute
/// indentation [`Code::reindent_inner`] bakes in for non-wrapped lines).
fn push_line(out: &mut Code, level: usize, text: &str) {
    let mut s = String::with_capacity(level * 4 + text.len());
    for _ in 0..level {
        s.push_str("    ");
    }
    s.push_str(text);
    out.items.push(Item::Line(s));
}

fn fits(line: &str, level: usize) -> bool {
    level * 4 + line.len() <= MAX_LINE_WIDTH
}

fn is_ident_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Is the `(` at byte `open` a *call* paren — preceded by an identifier whose
/// word is not a control-flow keyword (those can't take a trailing comma)?
fn is_call_paren(line: &str, open: usize) -> bool {
    let b = line.as_bytes();
    if open == 0 || !is_ident_byte(b[open - 1]) {
        return false;
    }
    let mut j = open;
    while j > 0 && is_ident_byte(b[j - 1]) {
        j -= 1;
    }
    !matches!(
        &line[j..open],
        "if" | "while" | "for" | "when" | "catch" | "synchronized"
    )
}

/// Byte index of the `-` of the first top-level `->` in `line[start..end)`
/// (bracket depth 0, ignoring string/char literals), or `None`.
fn find_arrow(line: &str, start: usize, end: usize) -> Option<usize> {
    let b = line.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut in_char = false;
    let mut i = start;
    while i < end {
        let c = b[i];
        if in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_char {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'\'' {
                in_char = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => in_str = true,
            b'\'' => in_char = true,
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => depth -= 1,
            b'-' if depth == 0 && i + 1 < end && b[i + 1] == b'>' => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Split `s` at top-level commas, tracking `(){}[]` and generic `<>` depth and
/// ignoring string/char literals (an arrow `->` is not a generic closer).
fn split_top_commas(s: &str) -> Vec<&str> {
    let b = s.as_bytes();
    let mut depth = 0i32;
    let mut angle = 0i32;
    let mut in_str = false;
    let mut in_char = false;
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_char {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'\'' {
                in_char = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => in_str = true,
            b'\'' => in_char = true,
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => break,
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => depth -= 1,
            b'<' if i > 0 && is_ident_byte(b[i - 1]) => angle += 1,
            b'>' if !(i > 0 && b[i - 1] == b'-') && angle > 0 => angle -= 1,
            b',' if depth == 0 && angle == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&s[start..]);
    parts
}

/// The outermost breakable construct on a line.
enum Construct {
    /// A call: byte index of `(` and its matching `)`.
    Call { open: usize, close: usize },
    /// A lambda: byte index of `{`, its matching `}`, and the `-` of its `->`.
    Lambda {
        open: usize,
        close: usize,
        arrow: Option<usize>,
    },
}

/// Find the depth-0 construct with the largest span (ties keep the leftmost):
/// a call `ident(…)` or a lambda `{ … -> … }`. Only the *outermost* construct
/// is returned — nested calls/lambdas are reached by recursing into its parts.
fn find_break(line: &str) -> Option<Construct> {
    let b = line.as_bytes();
    let mut stack: Vec<(u8, usize)> = Vec::new();
    let mut in_str = false;
    let mut in_char = false;
    let mut best: Option<Construct> = None;
    let mut best_span = 0usize;
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_char {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'\'' {
                in_char = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => in_str = true,
            b'\'' => in_char = true,
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => break,
            b'(' | b'{' | b'[' => stack.push((c, i)),
            b')' | b'}' | b']' => {
                if let Some((open_c, open_i)) = stack.pop() {
                    if stack.is_empty() {
                        // The candidate construct, if this depth-0 close yields one.
                        let cand = if open_c == b'(' && c == b')' {
                            (is_call_paren(line, open_i) && !line[open_i + 1..i].trim().is_empty())
                                .then_some(Construct::Call {
                                    open: open_i,
                                    close: i,
                                })
                        } else if open_c == b'{' && c == b'}' {
                            Some(Construct::Lambda {
                                open: open_i,
                                close: i,
                                arrow: find_arrow(line, open_i + 1, i),
                            })
                        } else {
                            None
                        };
                        if let Some(cand) = cand {
                            let span = i - open_i;
                            if best.is_none() || span > best_span {
                                best_span = span;
                                best = Some(cand);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    best
}

/// Emit `line` at `level`, breaking it one argument/parameter per line when it
/// exceeds the width budget. Recurses into the broken-out parts so nested calls
/// and the inline callback lambda are formatted too.
fn wrap_line(line: &str, level: usize, out: &mut Code) {
    if fits(line, level) {
        push_line(out, level, line);
        return;
    }
    match find_break(line) {
        Some(Construct::Call { open, close }) => {
            push_line(out, level, &line[..=open]);
            for arg in split_top_commas(&line[open + 1..close]) {
                let arg = arg.trim();
                if arg.is_empty() {
                    continue;
                }
                wrap_line(&format!("{arg},"), level + 1, out);
            }
            push_line(out, level, &line[close..]);
        }
        Some(Construct::Lambda { open, close, arrow }) => {
            push_line(out, level, &format!("{}{{", &line[..open]));
            let inner = &line[open + 1..close];
            match arrow {
                Some(arr) => {
                    let arr_in = arr - (open + 1);
                    for p in split_top_commas(inner[..arr_in].trim()) {
                        let p = p.trim();
                        if p.is_empty() {
                            continue;
                        }
                        push_line(out, level + 1, &format!("{p},"));
                    }
                    push_line(out, level + 1, "->");
                    wrap_line(inner[arr_in + 2..].trim(), level + 1, out);
                }
                None => wrap_line(inner.trim(), level + 1, out),
            }
            push_line(out, level, &format!("}}{}", &line[close + 1..]));
        }
        None => push_line(out, level, line),
    }
}

#[cfg(test)]
mod tests;
