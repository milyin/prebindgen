#!/usr/bin/env python3
"""Rust source lines per crate, split so adapter growth cannot hide in tests.

#613's size gate is "prebindgen-c and prebindgen-jni production Rust line
counts both decrease", which is only checkable if everyone counts the same way.
This is that way. Run it from anywhere:

    python3 examples/line-report.py                              # every crate
    python3 examples/line-report.py prebindgen-c prebindgen-jni
    python3 examples/line-report.py --self-test                  # check the classifier

Three numbers per crate, and every line of every `src/**/*.rs` file is in
exactly one of them:

  test files  a file under a `tests/` directory, or named `tests.rs` or
              `test_util.rs`. Wholly test support.
  test items  `#[cfg(test)]` items inside the remaining files, counted from the
              attribute to the item's real end. Test support that would
              otherwise read as production, which is the number that stops a
              move into an inline test module from looking like a deletion.
  production  what is left, and what the gate is about.

Only a bare `#[cfg(test)]` counts. An item behind
`#[cfg(any(test, feature = "testing"))]` ships to other crates under that
feature, so it is production by this rule.

An item's end is found by tracking delimiter depth, not by matching a closing
line: `#[cfg(test)] mod tests;` ends on its own line, a gated `fn` ends at its
`}`, and a gated multiline call ends at the `);` several lines down. Counting
the last of those by indentation swallowed the production statements after it
(#614 review). Strings, chars and comments are skipped so a `"{"` inside a
literal cannot open a block.
"""

from __future__ import annotations

import sys
from pathlib import Path

CRATES = [
    "prebindgen",
    "prebindgen-flat",
    "prebindgen-registry",
    "prebindgen-c",
    "prebindgen-jni",
    "prebindgen-c-runtime",
    "prebindgen-jni-runtime",
    "prebindgen-proc-macro",
]

OPEN = "([{"
CLOSE = ")]}"


def scan(text: str) -> tuple[int, int]:
    """Return (production lines, test-item lines) for one production file."""
    lines = text.splitlines()
    production = 0
    items = 0
    index = 0
    while index < len(lines):
        if lines[index].strip() != "#[cfg(test)]":
            production += 1
            index += 1
            continue
        # The attribute and the whole item that follows it.
        items += 1
        index += 1
        depth = 0
        in_body = False
        while index < len(lines):
            items += 1
            depth, in_body, ended = consume(lines[index], depth, in_body)
            index += 1
            if ended:
                break
        else:
            raise SystemExit("a #[cfg(test)] item runs to the end of the file")
    return production, items


def consume(line: str, depth: int, in_body: bool) -> tuple[int, bool, bool]:
    """Track one line's delimiters. Returns (depth, in_body, item ended here).

    An item ends at the `}` closing the brace body it opened at depth 0, or at
    a `;` at depth 0 when it never opened one. Both halves matter: `pub(crate)
    fn f()` balances two paren pairs before its body starts, so "depth returned
    to zero" alone ends the item on its own first line.
    """
    position = 0
    while position < len(line):
        char = line[position]
        if char == "/" and line[position : position + 2] == "//":
            break
        if char in ('"', "'"):
            position = skip_literal(line, position)
            continue
        if char == "{" and depth == 0:
            in_body = True
            depth = 1
        elif char in OPEN:
            depth += 1
        elif char in CLOSE:
            depth -= 1
            if in_body and depth == 0:
                return 0, in_body, True
        elif char == ";" and depth == 0 and not in_body:
            return 0, in_body, True
        position += 1
    return depth, in_body, False


def skip_literal(line: str, position: int) -> int:
    """Index just past the literal starting at `position`, or past a lifetime."""
    quote = line[position]
    if quote == "'":
        # A lifetime (`'a`) is not a literal: it has no closing quote.
        closing = line.find("'", position + 1)
        if closing == -1 or closing > position + 3:
            return position + 1
    position += 1
    while position < len(line):
        if line[position] == "\\":
            position += 2
            continue
        if line[position] == quote:
            return position + 1
        position += 1
    return position


def classify(path: Path) -> bool:
    """Whether this source is wholly test support."""
    return path.name in ("tests.rs", "test_util.rs") or "tests" in path.parts


def report(root: Path, crates: list[str]) -> None:
    header = f"{'crate':<24}{'production':>12}{'test items':>12}{'test files':>12}{'total':>12}"
    print(header)
    totals = [0, 0, 0]
    for crate in crates:
        src = root / crate / "src"
        if not src.is_dir():
            continue
        production = items = test_files = 0
        for path in sorted(src.rglob("*.rs")):
            text = path.read_text()
            if classify(path.relative_to(src)):
                test_files += len(text.splitlines())
                continue
            file_production, file_items = scan(text)
            production += file_production
            items += file_items
        total = production + items + test_files
        print(f"{crate:<24}{production:>12}{items:>12}{test_files:>12}{total:>12}")
        totals = [totals[0] + production, totals[1] + items, totals[2] + test_files]
    print(f"{'TOTAL':<24}{totals[0]:>12}{totals[1]:>12}{totals[2]:>12}{sum(totals):>12}")


FIXTURE = '''\
fn production_one() {
    let brace = "{";
    let life: &'static str = "}";
}

#[cfg(test)]
pub(crate) fn gated_fn() -> bool {
    if true { false } else { true }
}

#[cfg(test)]
mod tests;

impl Thing {
    #[cfg(test)]
    fn gated_method(&self) -> bool {
        self.0
    }

    fn production_two(&self) {
        // A gated MULTILINE call whose item ends in `);`, not in a `}` at the
        // attribute's indentation. Counting it by indentation swallowed
        // everything to the next closing brace — the production lines below.
        #[cfg(test)]
        check(
            self.first(),
            "jni",
        );
        self.second();
        self.third();
    }
}

#[cfg(any(test, feature = "testing"))]
pub fn shipped_under_a_feature() -> bool {
    true
}
'''


def self_test() -> None:
    production, items = scan(FIXTURE)
    total = len(FIXTURE.splitlines())
    assert production + items == total, "every line is counted once"
    # Four gated items: `gated_fn` (4 lines), `mod tests;` (2),
    # `gated_method` (4), and the multiline call (5).
    #
    # 15 is also the regression #614's review asked for. Ending the call's item
    # at a `}` with the attribute's indentation — the rule this replaced —
    # would run it to `production_two`'s closing brace and count
    # `self.second();` and `self.third();` as test support, giving 17.
    assert items == 15, f"gated items are 15 lines, got {items}"
    assert production == total - 15, "the statements after the gated call stay production"
    print("self-test ok: every line counted once, gated items 15, nothing swallowed")


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    arguments = sys.argv[1:]
    if arguments == ["--self-test"]:
        self_test()
        return
    report(root, arguments or CRATES)


if __name__ == "__main__":
    main()
