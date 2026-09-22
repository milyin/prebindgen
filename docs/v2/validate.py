#!/usr/bin/env python3
"""Check the V2 specification's structure: identities, link ids, indexes, links.

Uses only the standard library. Run: python3 docs/v2/validate.py
"""
import argparse
import json
import re
import sys
from pathlib import Path

ROOT_PAGES = {
    "README.md": "root",
    "FORMAT.md": "format",
    "source.md": "fixture",
    "implementation.md": "implementation",
    "concepts.md": "concepts",
    "extensions.md": "extensions",
}
# Pages the vocabulary rule does not read: the vocabulary itself, and the page
# about the document's format.
VOCABULARY_EXEMPT = {"concepts.md", "FORMAT.md"}
REQUIRED_SECTIONS = ("Input", "Result", "Checks")
META = re.compile(r"^<!-- spec: (\{.*\}) -->$")
DEFINITION = re.compile(r"^\[([A-Za-z0-9_]+)\]:\s*(\S+)\s*$", re.M)
REFERENCE = re.compile(r"\[[^\]^]*?\]\[([A-Za-z0-9_]+)\]")
INLINE = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
HEADING = re.compile(r"^(#{1,6})\s+(.+?)\s*$", re.M)


class InvalidSpec(ValueError):
    pass


class Page:
    def __init__(self, root, relative):
        self.relative = relative
        self.path = root / relative
        self.raw = self.path.read_text()
        self.text = strip_fences(self.raw, relative)
        self.meta = read_meta(self.raw, relative)
        self.definitions = {}
        for label, target in DEFINITION.findall(self.text):
            if label in self.definitions:
                fail(f"{relative}: duplicate link definition [{label}]")
            self.definitions[label] = target
        self.references = REFERENCE.findall(DEFINITION.sub("", self.text))
        self.anchors = anchors(self.text)

    def kind(self):
        return self.meta["kind"]


def fail(message):
    raise InvalidSpec(message)


def strip_fences(text, where):
    """Replace fenced code with a marker: it is content, but not links or headings."""
    kept, fence = [], None
    # One line out per line in, so a reported line number is the file's.
    for line in text.splitlines():
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker:
            run = marker.group(1)
            if fence is None:
                fence = run
                kept.append("{code}")
            elif run[0] == fence[0] and len(run) >= len(fence):
                fence = None
                kept.append("")
            continue
        kept.append(line if fence is None else "")
    if fence is not None:
        fail(f"{where}: unclosed code fence")
    return "\n".join(kept)


def read_meta(raw, where):
    first = raw.splitlines()[0] if raw.strip() else ""
    match = META.match(first)
    if not match:
        fail(f"{where}: missing '<!-- spec: {{...}} -->' metadata on the first line")
    try:
        meta = json.loads(match.group(1))
    except json.JSONDecodeError as error:
        fail(f"{where}: invalid metadata JSON ({error})")
    if not isinstance(meta, dict) or "kind" not in meta:
        fail(f"{where}: metadata must be an object with a 'kind'")
    return meta


def anchors(text):
    found, counts = set(), {}
    for _, label in HEADING.findall(text):
        base = re.sub(r"[^\w\- ]", "", label.strip().lower()).replace(" ", "-")
        seen = counts.get(base, 0)
        found.add(base if seen == 0 else f"{base}-{seen}")
        counts[base] = seen + 1
    return found


def section(page, heading):
    matches = list(re.finditer(r"^## " + re.escape(heading) + r"\s*$", page.text, re.M))
    if len(matches) != 1:
        fail(f"{page.relative}: expected exactly one '## {heading}' section")
    tail = page.text[matches[0].end():]
    end = re.search(r"^#{1,2} ", tail, re.M)
    body = tail[: end.start()] if end else tail
    if not body.strip():
        fail(f"{page.relative}: empty '## {heading}' section")
    return body


def header(page):
    """The navigation block above the page title: backlinks and neighbours."""
    title = re.search(r"^# .*$", page.text, re.M)
    if title is None:
        fail(f"{page.relative}: missing a title")
    return page.text[: title.start()]


def ordered_ids(text, allowed):
    """Ids of `allowed` in order of first use, as [label] references."""
    seen = []
    for label in REFERENCE.findall(text):
        if label in allowed and label not in seen:
            seen.append(label)
    return seen


def read_manifest(root):
    manifest = json.loads((root / "manifest.json").read_text())
    languages = [entry["id"] for entry in manifest["languages"]]
    stages = manifest["stages"]
    examples = manifest["examples"]
    stage_by_id = {stage["id"]: stage for stage in stages}
    example_by_id = {example["id"]: example for example in examples}
    for cell in manifest["cells"]:
        if cell["stage"] not in stage_by_id or cell["example"] not in example_by_id:
            fail(f"manifest: cell {cell} names an unknown stage or example")
        declared = cell["languages"]
        if stage_by_id[cell["stage"]]["language_dependent"]:
            if declared != languages:
                fail(f"manifest: {cell['example']} at {cell['stage']} must declare "
                     f"every language {languages}, not {declared}")
        elif declared:
            fail(f"manifest: {cell['example']} at {cell['stage']} is language "
                 "independent but declares languages")
    return manifest, languages, stages, examples, stage_by_id, example_by_id


def build_index(manifest, stage_by_id, example_by_id):
    """Map every link id to its canonical path, and every path to its identity."""
    ids, identity = {}, {}
    for example in manifest["examples"]:
        path = f"examples/{example['dir']}/README.md"
        ids[example["id"]] = path
        identity[path] = {"kind": "example", "example": example["id"]}
    for cell in manifest["cells"]:
        example = example_by_id[cell["example"]]
        stage = stage_by_id[cell["stage"]]
        base = f"examples/{example['dir']}/{stage['id']}"
        cell_id = f"{example['id']}_{stage['slug']}"
        ids[cell_id] = f"{base}.md"
        identity[f"{base}.md"] = {
            "kind": "cell", "example": example["id"], "stage": stage["id"]}
        for language in cell["languages"]:
            ids[f"{cell_id}_{language}"] = f"{base}.{language}.md"
            identity[f"{base}.{language}.md"] = {
                "kind": "variant", "example": example["id"],
                "stage": stage["id"], "language": language}
    for stage in manifest["stages"]:
        path = f"stages/{stage['id']}.md"
        identity[path] = {"kind": "stage", "stage": stage["id"]}
    for path, kind in ROOT_PAGES.items():
        identity[path] = {"kind": kind}
    return ids, identity


def check_files(root, identity):
    present = {str(path.relative_to(root)) for path in root.rglob("*.md")}
    for missing in sorted(set(identity) - present):
        fail(f"{missing}: declared in the manifest but missing")
    for extra in sorted(present - set(identity)):
        fail(f"{extra}: not declared in the manifest")


def check_identity(page, identity):
    expected = identity[page.relative]
    if page.meta != expected:
        fail(f"{page.relative}: metadata {page.meta} does not match its canonical "
             f"identity {expected}")


def check_definitions(page, ids, root):
    for label, target in page.definitions.items():
        if label not in ids:
            fail(f"{page.relative}: [{label}] is not a declared link id")
        canonical = (root / ids[label]).resolve()
        actual = (page.path.parent / target).resolve()
        if canonical != actual:
            fail(f"{page.relative}: [{label}] points at {target}, expected "
                 f"{Path(ids[label]).name} at {ids[label]}")
    for label in page.references:
        if label not in page.definitions:
            fail(f"{page.relative}: [{label}] is used but not defined on this page")
    for label in page.definitions:
        if label not in page.references:
            fail(f"{page.relative}: [{label}] is defined but never used")


def check_inline_links(page, root):
    for target in INLINE.findall(page.text):
        if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
            continue
        file, _, anchor = target.partition("#")
        destination = (page.path.parent / file).resolve() if file else page.path.resolve()
        try:
            relative = destination.relative_to(root.resolve())
        except ValueError:
            relative = None
        if relative is None:
            continue  # A link out of the documentation tree, such as into the crates.
        if str(relative).startswith("examples/"):
            fail(f"{page.relative}: inline link to {target}; links into examples/ "
                 "must use a reference-style link id")
        if not destination.exists():
            fail(f"{page.relative}: link to {target} does not exist")
        yield destination, anchor, target


def check_anchors(page, pages, root, links):
    for destination, anchor, target in links:
        if not anchor:
            continue
        relative = str(destination.resolve().relative_to(root.resolve()))
        other = pages.get(relative)
        if other is None:
            continue
        if anchor not in other.anchors:
            fail(f"{page.relative}: link to {target} has no matching heading")


def blank(match):
    """Replace a match with spaces, keeping every offset and every newline."""
    return re.sub(r"[^\n]", " ", match.group(0))


# A code span opens with a complete run of backticks and closes with a complete
# run of the same length; a shorter run inside is content, so ``a ` b`` is one
# span, and an unmatched run is text.
CODE_SPAN = re.compile(r"(?<!`)(`+)(?!`)(?:(?!(?<!`)\1(?!`))[\s\S])+?(?<!`)\1(?!`)")
# The one HTML construct the document writes is a table, and a `<code>` cell
# in it is code the way a backtick span is.
HTML_CODE = re.compile(r"<code>[\s\S]*?</code>")
# A backslash-escaped punctuation character has no Markdown meaning: \` opens
# no code span, \[ no link, \* no emphasis.
ESCAPE = re.compile(r"\\[\\`*_\[\]()#]")
STRONG = r"(?:\*\*|__)"
REFERENCE_TEXT = re.compile(r"\[([^\]^]*?)\]\[[A-Za-z0-9_]+\]", re.S)
INLINE_SPAN = re.compile(r"\[([^\]]*)\]\(([^)]+)\)", re.S)


def prose(text):
    """The page as a reader reads it, offsets preserved: headings, link
    definitions, metadata and code spans blanked out."""
    out = []
    for line in text.split("\n"):
        if HEADING.match(line) or DEFINITION.match(line) or META.match(line):
            out.append(" " * len(line))
        else:
            out.append(line)
    text = ESCAPE.sub(blank, "\n".join(out))
    return HTML_CODE.sub(blank, CODE_SPAN.sub(blank, text))


def section_of(text, anchor):
    """The text under the heading whose anchor this is, up to the next heading
    of the same or a higher level — a subsection belongs to its section."""
    counts = {}
    headings = list(HEADING.finditer(text))
    for index, heading in enumerate(headings):
        base = re.sub(r"[^\w\- ]", "", heading.group(2).strip().lower()).replace(" ", "-")
        seen = counts.get(base, 0)
        counts[base] = seen + 1
        if (base if seen == 0 else f"{base}-{seen}") != anchor:
            continue
        level = len(heading.group(1))
        end = next((later.start() for later in headings[index + 1:]
                    if len(later.group(1)) <= level), len(text))
        return text[heading.end():end]
    return None


def check_vocabulary(pages, manifest, root):
    """Each term is defined in bold once, under the heading the manifest names;
    the concepts page introduces it with a link there; and every other page's
    first mention of it in prose is a link there."""
    concepts = pages.get("concepts.md")
    if concepts is None:
        fail("concepts.md: missing; the vocabulary lives there")
    titles = {stage["title"] for stage in manifest["stages"]}
    titles |= {example["title"] for example in manifest["examples"]}
    titles |= {language["title"] for language in manifest["languages"]}
    for entry in manifest.get("vocabulary", []):
        term, pattern = entry["term"], entry["match"]
        defined = entry["defined"]
        defined_page, _, defined_anchor = defined.partition("#")
        # A space in a multi-word term is any whitespace, a soft line break
        # included: "source\nitem" is a mention of the source item, and
        # "**source\nitem**" its definition.
        spaced = pattern.replace(" ", r"\s+")
        word = re.compile(r"\b(?:" + spaced + r")\b", re.I)
        # Strong emphasis is `**` or `__`. On the defining page the term may sit
        # inside a longer bold phrase (`**target choice**`); elsewhere only the
        # bare term set in bold is a redefinition — `**conversion rule**` is
        # another term.
        # `\b` treats `_` as a word character, so the edges are spelled out.
        edge = r"(?<![A-Za-z0-9])", r"(?![A-Za-z0-9])"
        bold = re.compile(STRONG + r"(?:[^*_\n]*" + edge[0] + r")?(?:" + spaced + r")" + edge[1]
                          + r"[^*_\n]*" + STRONG, re.I)
        rebold = re.compile(STRONG + r"(?:(?:the|a|an) )?(?:" + spaced + r")" + STRONG, re.I)

        if defined_page not in pages:
            fail(f"manifest: vocabulary term '{term}' is defined on {defined_page}, "
                 "which is not a page")
        owner = pages[defined_page]
        if defined_anchor and defined_anchor not in owner.anchors:
            fail(f"manifest: vocabulary term '{term}' points at {defined}, "
                 "which has no matching heading")
        # The definition has to sit under the heading the reader is sent to.
        scope = section_of(owner.text, defined_anchor) if defined_anchor else owner.text
        if not bold.search(prose(scope or "")):
            fail(f"{defined_page}: does not define '{term}' in bold under "
                 f"#{defined_anchor or '(top)'}, and the manifest says it does")

        # One entry on the concepts page, and that entry — up to the next
        # heading of its level or higher, so a group heading ends it — links to
        # the definition itself.
        heading = re.compile(r"^###\s+" + re.escape(term) + r"\s*$", re.I | re.M)
        match = heading.search(concepts.text)
        if match is None:
            fail(f"concepts.md: no '### {term}' entry")
        # Read as prose: a link inside a code span or behind an escape renders
        # as text, and satisfies nothing.
        entry_text = prose(section_of(concepts.text, anchors(match.group(0)).pop()) or "")
        if not any(resolves(concepts, target, root) == defined
                   for _, target in INLINE_SPAN.findall(entry_text)):
            fail(f"concepts.md: the '{term}' entry does not link to {defined}")

        # A definition in a chapter's opening prose is anchored at its title, and
        # a link to the chapter itself lands there.
        title = HEADING.search(owner.text)
        accepted = {defined}
        if title and defined_anchor == anchors(title.group(0)).pop():
            accepted.add(defined_page)

        for page in pages.values():
            if page.relative in VOCABULARY_EXEMPT:
                continue
            text = prose(page.text)
            if page.relative != defined_page and rebold.search(text):
                fail(f"{page.relative}: sets '{term}' in bold; it is defined on "
                     f"{defined_page} and other pages link there")
            if page.relative == defined_page or not entry.get("link_first_mention", True):
                continue
            # A chapter or element title quoted in a link — in the navigation
            # header, an element TOC, a chapter's index — is not a mention,
            # whichever link syntax quotes it. Any other link is prose like any
            # other, and one that wraps a first mention leaves it unlinked to
            # the definition. A link's target is never a mention.
            def quoted_title(m):
                return re.sub(r"\s+", " ", m.group(1)).strip() in titles
            searched = REFERENCE_TEXT.sub(
                lambda m: blank(m) if quoted_title(m) else m.group(0), text)
            searched = INLINE_SPAN.sub(
                lambda m: blank(m) if quoted_title(m) else m.group(0), searched)
            searched = re.sub(r"\]\([^)]*\)|\]\[[A-Za-z0-9_]+\]", blank, searched)
            mention = word.search(searched)
            if mention is None:
                continue
            linked = any(span.start() <= mention.start() < span.end()
                         and resolves(page, span.group(2).strip(), root) in accepted
                         for span in INLINE_SPAN.finditer(text))
            if not linked:
                line = text.count("\n", 0, mention.start()) + 1
                fail(f"{page.relative}:{line}: the first mention of '{term}' must "
                     f"link to {defined}")


# "native" names no element: it says only "not the source side", leaving the
# wrapper, its boundary, the generated Rust and the shipped library
# indistinguishable. Each of those has a name. JNI's own title keeps the word.
BANNED = re.compile(r"\bnative\b", re.I)
PROPER_NAME = re.compile(r"Java Native Interface")


def check_banned_words(page):
    """Prose says which element it means; code and JNI's proper name are exempt."""
    text = PROPER_NAME.sub(blank, prose(page.text))
    found = BANNED.search(text)
    if found:
        line = text.count("\n", 0, found.start()) + 1
        fail(f"{page.relative}:{line}: '{found.group(0)}' names no element — "
             f"say which one (wrapper, wrapper boundary, generated Rust, "
             f"dynamic library, `external` method)")


def resolves(page, target, root):
    """A link target on `page`, as a root-relative path with its anchor."""
    target = re.sub(r"\s+", "", target)
    file, _, anchor = target.partition("#")
    if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target):
        return None
    destination = (page.path.parent / file).resolve() if file else page.path.resolve()
    try:
        relative = str(destination.relative_to(root.resolve()))
    except ValueError:
        return None
    return f"{relative}#{anchor}" if anchor else relative


def check_cell(page, ids, root, manifest, stage_by_id, example_by_id, order):
    for heading in REQUIRED_SECTIONS:
        section(page, heading)
    if "Owner:" not in header(page):
        fail(f"{page.relative}: the header must name the owner of this step")
    example = page.meta["example"]
    stage = stage_by_id[page.meta["stage"]]
    chapter = f"../../stages/{stage['id']}.md"
    if chapter not in INLINE.findall(page.text):
        fail(f"{page.relative}: must link to its stage chapter ({chapter})")
    if example not in page.references:
        fail(f"{page.relative}: must link back to its element path [{example}]")
    if page.kind() == "variant":
        common = f"{example}_{stage['slug']}"
        if common not in page.references:
            fail(f"{page.relative}: must link to its common cell [{common}]")
        return
    languages = [cell["languages"] for cell in manifest["cells"]
                 if cell["example"] == example and cell["stage"] == stage["id"]][0]
    if languages:
        listed = ordered_ids(section(page, "Language variants"), set(ids))
        expected = [f"{example}_{stage['slug']}_{language}" for language in languages]
        if listed != expected:
            fail(f"{page.relative}: 'Language variants' lists {listed}, expected {expected}")
    position = order[example].index(page.relative)
    for step, label in ((-1, "previous"), (1, "next")):
        index = position + step
        if 0 <= index < len(order[example]):
            neighbor = ids_by_path(ids)[order[example][index]]
            if neighbor not in ordered_ids(header(page), set(ids)):
                fail(f"{page.relative}: the header must link its {label} "
                     f"cell [{neighbor}]")


def ids_by_path(ids):
    return {path: link_id for link_id, path in ids.items()}


def check_stage_page(page, manifest, stage_by_id, example_by_id, ids):
    stage = stage_by_id[page.meta["stage"]]
    declared = []
    for cell in manifest["cells"]:
        if cell["stage"] != stage["id"]:
            continue
        base = f"{cell['example']}_{stage['slug']}"
        declared.append(base)
        declared.extend(f"{base}_{language}" for language in cell["languages"])
    listed = ordered_ids(section(page, "Elements at this stage"), set(declared))
    if listed != declared:
        fail(f"{page.relative}: 'Elements at this stage' lists {listed}, "
             f"expected {declared} in manifest order")
    stages = [entry["id"] for entry in manifest["stages"]]
    position = stages.index(stage["id"])
    targets = INLINE.findall(page.text)
    if position > 0 and f"{stages[position - 1]}.md" not in targets:
        fail(f"{page.relative}: missing the previous chapter link")
    if position + 1 < len(stages) and f"{stages[position + 1]}.md" not in targets:
        fail(f"{page.relative}: missing the next chapter link")


def check_example_page(page, ids, order):
    example = page.meta["example"]
    own = {link_id for link_id in ids if link_id.split("_")[0] == example}
    listed = ordered_ids(page.text, own)
    expected = [ids_by_path(ids)[path] for path in order[example]]
    expanded = []
    for cell_id in expected:
        expanded.append(cell_id)
        expanded.extend(sorted(link_id for link_id in own
                               if link_id.startswith(f"{cell_id}_")))
    if listed != expanded:
        fail(f"{page.relative}: contents list {listed}, expected {expanded}")


def validate(root):
    root = Path(root)
    manifest, languages, stages, examples, stage_by_id, example_by_id = read_manifest(root)
    ids, identity = build_index(manifest, stage_by_id, example_by_id)
    check_files(root, identity)
    pages = {relative: Page(root, relative) for relative in sorted(identity)}
    order = {}
    for example in examples:
        order[example["id"]] = [
            f"examples/{example['dir']}/{cell['stage']}.md"
            for cell in manifest["cells"] if cell["example"] == example["id"]]
    for page in pages.values():
        check_identity(page, identity)
        check_definitions(page, ids, root)
        check_banned_words(page)
        links = list(check_inline_links(page, root))
        check_anchors(page, pages, root, links)
        if page.kind() in ("cell", "variant"):
            check_cell(page, ids, root, manifest, stage_by_id, example_by_id, order)
        elif page.kind() == "stage":
            check_stage_page(page, manifest, stage_by_id, example_by_id, ids)
        elif page.kind() == "example":
            check_example_page(page, ids, order)
    check_vocabulary(pages, manifest, root)
    return (f"Valid: {len(pages)} pages, {len(stages)} stages, "
            f"{len(examples)} element paths, {len(manifest['cells'])} cells, "
            f"{len(ids)} link ids, {len(manifest.get('vocabulary', []))} vocabulary terms.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", nargs="?", default=Path(__file__).parent)
    arguments = parser.parse_args()
    try:
        print(validate(arguments.root))
    except InvalidSpec as error:
        print(f"Invalid: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
