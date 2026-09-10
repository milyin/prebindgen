#!/usr/bin/env python3
"""Negative tests for validate.py: each rule must reject its own breakage."""
import json
import shutil
import tempfile
import unittest
from pathlib import Path

import validate

SOURCE = Path(__file__).parent


class SpecStructure(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.mkdtemp()
        self.root = Path(self.directory) / "v2"
        shutil.copytree(SOURCE, self.root, ignore=shutil.ignore_patterns("__pycache__"))

    def tearDown(self):
        shutil.rmtree(self.directory)

    def edit(self, relative, old, new):
        path = self.root / relative
        text = path.read_text()
        self.assertIn(old, text, f"anchor missing in {relative}")
        path.write_text(text.replace(old, new, 1))

    def rejects(self, fragment):
        with self.assertRaises(validate.InvalidSpec) as caught:
            validate.validate(self.root)
        self.assertIn(fragment, str(caught.exception))

    def test_unmodified_tree_is_valid(self):
        self.assertTrue(validate.validate(self.root).startswith("Valid:"))

    def test_missing_metadata_line(self):
        self.edit("source.md", '<!-- spec: {"kind": "fixture"} -->\n', "")
        self.rejects("missing")

    def test_wrong_identity(self):
        self.edit("examples/fn/02-flat.md", '"stage": "02-flat"', '"stage": "03-requests"')
        self.rejects("does not match its canonical identity")

    def test_definition_pointing_elsewhere(self):
        self.edit("examples/fn/04-values.md",
                  "[fn_values_c]: 04-values.c.md", "[fn_values_c]: 04-values.jni.md")
        self.rejects("points at")

    def test_undefined_reference(self):
        self.edit("examples/fn/04-values.md", "[fn_values_c]: 04-values.c.md", "")
        self.rejects("used but not defined")

    def test_unused_definition(self):
        self.edit("stages/01-source.md", "[struct_source]: ../examples/struct/01-source.md",
                  "[struct_source]: ../examples/struct/01-source.md\n"
                  "[struct_flat]: ../examples/struct/02-flat.md")
        self.rejects("defined but never used")

    def test_unknown_link_id(self):
        self.edit("stages/01-source.md", "[struct_source]", "[struct_capture]")
        self.edit("stages/01-source.md", "[struct_source]:", "[struct_capture]:")
        self.rejects("not a declared link id")

    def test_inline_link_into_examples(self):
        self.edit("stages/06-retain.md", "[Record with scalar fields][struct_retain]",
                  "[Record with scalar fields](../examples/struct/06-retain.md)")
        self.edit("stages/06-retain.md",
                  "[struct_retain]: ../examples/struct/06-retain.md", "")
        self.rejects("must use a reference-style link id")

    def test_broken_local_link(self):
        self.edit("examples/fn/02-flat.md", "../../stages/02-flat.md",
                  "../../stages/02-flat-model.md")
        self.rejects("does not exist")

    def test_unknown_anchor(self):
        self.edit("stages/03-requests.md",
                  "04-values.md#describing-source-construction-and-decomposition",
                  "04-values.md#describing-source-construction")
        self.rejects("no matching heading")

    def test_missing_required_section(self):
        self.edit("examples/struct/06-retain.md", "## Checks", "## Notes")
        self.rejects("exactly one '## Checks' section")

    def test_empty_required_section(self):
        path = self.root / "examples/struct/06-retain.md"
        text = path.read_text()
        start = text.index("## Input")
        end = text.index("## Result")
        path.write_text(text[:start] + "## Input\n\n" + text[end:])
        self.rejects("empty '## Input' section")

    def test_missing_owner(self):
        self.edit("examples/struct/06-retain.md", "Owner: the registry · ", "")
        self.rejects("must name the owner")

    def test_stage_index_out_of_order(self):
        self.edit("stages/06-retain.md",
                  "- [Function taking an owned record][fn_retain]\n"
                  "- [Record with scalar fields][struct_retain]",
                  "- [Record with scalar fields][struct_retain]\n"
                  "- [Function taking an owned record][fn_retain]")
        self.rejects("Elements at this stage")

    def test_stage_index_incomplete(self):
        self.edit("stages/06-retain.md",
                  "- [Record with scalar fields][struct_retain]\n", "")
        self.edit("stages/06-retain.md",
                  "[struct_retain]: ../examples/struct/06-retain.md", "")
        self.rejects("Elements at this stage")

    def test_example_contents_incomplete(self):
        self.edit("examples/struct/README.md",
                  "5. [Retain supported output][struct_retain]\n", "")
        self.edit("examples/struct/README.md",
                  "[struct_retain]: 06-retain.md", "")
        self.rejects("contents list")

    def test_missing_neighbor_link(self):
        self.edit("examples/fn/04-values.md",
                  "Previous: [Record binding requests][fn_requests] · ", "")
        self.edit("examples/fn/04-values.md", "[fn_requests]: 03-requests.md", "")
        self.rejects("must link its previous cell")

    def test_variant_without_common_cell_link(self):
        self.edit("examples/fn/04-values.c.md",
                  "[Common cell][fn_values] · ", "")
        self.edit("examples/fn/04-values.c.md", "[fn_values]: 04-values.md", "")
        self.rejects("must link to its common cell")

    def test_missing_language_variant_listing(self):
        self.edit("examples/struct/04-values.md", "- [C][struct_values_c]\n", "")
        self.rejects("'Language variants' lists")

    def test_unlisted_markdown_file(self):
        (self.root / "examples/fn/08-extra.md").write_text("<!-- spec: {} -->\n")
        self.rejects("not declared in the manifest")

    def test_missing_declared_page(self):
        (self.root / "examples/struct/06-retain.md").unlink()
        self.rejects("missing")

    def test_unclosed_code_fence(self):
        path = self.root / "examples/fn/07-emit.c.md"
        path.write_text(path.read_text() + "\n```rust\nlet unterminated = 1;\n")
        self.rejects("unclosed code fence")

    def test_language_dependent_stage_needs_every_language(self):
        path = self.root / "manifest.json"
        manifest = json.loads(path.read_text())
        for cell in manifest["cells"]:
            if cell["stage"] == "04-values" and cell["example"] == "fn":
                cell["languages"] = ["c"]
        path.write_text(json.dumps(manifest, indent=2))
        self.rejects("must declare every language")


if __name__ == "__main__":
    unittest.main()
