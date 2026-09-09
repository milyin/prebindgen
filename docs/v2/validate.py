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
}
REQUIRED_SECTIONS = ("Input", "Owner", "Result", "Checks", "Representation")
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
    for line in text.splitlines():
        marker = re.match(r"^\s*(`{3,}|~{3,})", line)
        if marker:
            run = marker.group(1)
            if fence is None:
                fence, kept = run, kept + ["<code>"]
            elif run[0] == fence[0] and len(run) >= len(fence):
                fence = None
            continue
        if fence is None:
            kept.append(line)
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
    """The block between the page title and its first section: links and neighbours."""
    title = re.search(r"^# .*$", page.text, re.M)
    if title is None:
        fail(f"{page.relative}: missing a title")
    tail = page.text[title.end():]
    end = re.search(r"^## ", tail, re.M)
    return tail[: end.start()] if end else tail


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


def check_cell(page, ids, root, manifest, stage_by_id, example_by_id, order):
    for heading in REQUIRED_SECTIONS:
        section(page, heading)
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
        links = list(check_inline_links(page, root))
        check_anchors(page, pages, root, links)
        if page.kind() in ("cell", "variant"):
            check_cell(page, ids, root, manifest, stage_by_id, example_by_id, order)
        elif page.kind() == "stage":
            check_stage_page(page, manifest, stage_by_id, example_by_id, ids)
        elif page.kind() == "example":
            check_example_page(page, ids, order)
    return (f"Valid: {len(pages)} pages, {len(stages)} stages, "
            f"{len(examples)} element paths, {len(manifest['cells'])} cells, "
            f"{len(ids)} link ids.")


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
