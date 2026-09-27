#!/usr/bin/env python3
"""Fail when rendered rustdoc holds a link that did not resolve.

rustdoc does not always warn about one. An explicit link whose target does not
resolve is kept as a plain URL (`href="prebindgen_flat::Flat"`). A link into a
crate whose docs are not in place yet is dropped, from the text and from every
signature that names one of its types (`flat: &'a Flat` with `Flat` unlinked).
Both show only in the output, so this reads the output.

Usage: check-links.py <doc-dir> <crate>...
"""

import glob
import html
import os
import re
import sys

ITEM_PAGE = re.compile(r"(?:struct|enum|trait|type|union)\.([A-Za-z0-9_]+)\.html$")
# A path kept as a URL.
RAW_PATH = re.compile(r'href="([A-Za-z_][A-Za-z0-9_]*::[^"]*)"')
# `[`Item`]` link text rustdoc left as text.
LITERAL = re.compile(r"\[<code>[^<]*</code>\]")
# A signature, or an item's declaration.
SIGNATURE = re.compile(
    r'<(?:h4|pre) class="(?:code-header|rust item-decl)"[^>]*>(.*?)</(?:h4|pre)>', re.S
)
LINK = re.compile(r"<a [^>]*>.*?</a>", re.S)
# A type name where a signature uses a type: after `:`, `&`, `<`, `(`, `,`,
# `=`, `->` or a keyword, on the same line. An enum variant's name starts its
# own line and is not matched.
TYPE_USE = re.compile(r"(?:[:&<(,=]|->|\bmut|\bimpl|\bdyn|\bfor)[ \t]*([A-Z][A-Za-z0-9_]+)\b")


def pages(doc, crate):
    return glob.glob(os.path.join(doc, crate, "**", "*.html"), recursive=True)


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    doc = sys.argv[1]
    crates = [c.replace("-", "_") for c in sys.argv[2:]]
    problems = []

    # The types the crates document: a signature naming one must link it.
    types = set()
    for c in crates:
        if not os.path.isfile(os.path.join(doc, c, "index.html")):
            problems.append(f"{c}: no docs")
        for f in pages(doc, c):
            m = ITEM_PAGE.match(os.path.basename(f))
            if m and len(m.group(1)) > 2:
                types.add(m.group(1))

    for c in crates:
        for f in pages(doc, c):
            rel = os.path.relpath(f, doc)
            text = open(f, encoding="utf-8").read()
            for path in sorted(set(RAW_PATH.findall(text))):
                problems.append(f"{rel}: the path `{path}` did not resolve")
            for lit in sorted(set(LITERAL.findall(text))):
                problems.append(f"{rel}: link text left unlinked: {html.unescape(lit)}")
            own = ITEM_PAGE.match(os.path.basename(f))
            own = own.group(1) if own else None
            unlinked = set()
            for sig in SIGNATURE.findall(text):
                plain = html.unescape(re.sub(r"<[^>]+>", "", LINK.sub("", sig)))
                plain = re.sub(r"'[a-z_]+\s?", "", plain)  # lifetimes
                unlinked |= set(TYPE_USE.findall(plain)) & types
            for t in sorted(unlinked - {own}):
                problems.append(f"{rel}: a signature names `{t}` without linking it")

    if problems:
        print("check-links: rendered docs hold unresolved links:", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
