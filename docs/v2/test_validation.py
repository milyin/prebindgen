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
                  "04-values.md#what-a-relation-is",
                  "04-values.md#what-a-relation")
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

    def test_vocabulary_term_defined_twice(self):
        self.edit("stages/05-boundary.md", "the same input conversion", "the same input **conversion**")
        self.rejects("sets 'conversion' in bold")

    def test_vocabulary_first_mention_unlinked(self):
        self.edit("stages/02-flat.md",
                  "[relations](04-values.md#what-a-relation-is)", "relations")
        self.rejects("first mention of 'relation'")

    def test_vocabulary_first_mention_links_elsewhere(self):
        self.edit("stages/02-flat.md",
                  "[relations](04-values.md#what-a-relation-is)",
                  "[relations](04-values.md#plan-value-conversions)")
        self.rejects("first mention of 'relation'")

    def test_vocabulary_definition_under_another_heading_is_rejected(self):
        # The heading exists and the bold definition exists, but not under it.
        manifest = self.root / "manifest.json"
        manifest.write_text(manifest.read_text().replace(
            "stages/04-values.md#what-a-relation-is",
            "stages/04-values.md#responsibility-boundary-with-flat"))
        for page in self.root.rglob("*.md"):
            text = page.read_text()
            if "04-values.md#what-a-relation-is" in text or "#what-a-relation-is" in text:
                page.write_text(text.replace("#what-a-relation-is",
                                             "#responsibility-boundary-with-flat"))
        self.rejects("does not define 'relation' in bold under")

    def test_vocabulary_link_wrapped_across_lines_is_a_link(self):
        self.edit("stages/02-flat.md",
                  "[relations](04-values.md#what-a-relation-is)",
                  "[relations\n](04-values.md#what-a-relation-is)")
        self.assertTrue(validate.validate(self.root).startswith("Valid:"))

    def test_vocabulary_mention_in_a_wrapped_code_span_is_not_a_mention(self):
        self.edit("stages/02-flat.md",
                  "[relations](04-values.md#what-a-relation-is)",
                  "`a\nrelation` [relations](04-values.md#what-a-relation-is)")
        self.assertTrue(validate.validate(self.root).startswith("Valid:"))

    def test_vocabulary_first_mention_before_a_soft_break_is_found(self):
        self.edit("stages/02-flat.md",
                  "[relations](04-values.md#what-a-relation-is)",
                  "a\nrelation and [relations](04-values.md#what-a-relation-is)")
        self.rejects("first mention of 'relation'")

    def test_vocabulary_bold_inside_code_is_not_a_definition(self):
        self.edit("stages/02-flat.md",
                  "[relations](04-values.md#what-a-relation-is)",
                  "`**relation**` [relations](04-values.md#what-a-relation-is)")
        self.assertTrue(validate.validate(self.root).startswith("Valid:"))

    def test_vocabulary_concepts_entry_linking_elsewhere_is_rejected(self):
        # Another entry on the page links to the same destination; this one must
        # link there itself.
        self.edit("concepts.md",
                  "[Plan value conversions](stages/04-values.md#plan-value-conversions)\n\n### Crossing",
                  "[What a relation is](stages/04-values.md#what-a-relation-is)\n\n### Crossing")
        self.rejects("the 'conversion' entry does not link")

    def test_vocabulary_first_mention_inside_a_reference_link_is_rejected(self):
        # Copilot's mutation on #736: the term's only mention wrapped in a
        # reference-style link to somewhere else.
        self.edit("examples/struct/07-emit.md",
                  "in the [wrapper](../../stages/05-boundary.md#assemble-the-native-boundary) "
                  "of [the function that uses the record][fn_emit]:",
                  "in the [wrapper][fn_emit] of the function that uses the record:")
        self.rejects("first mention of 'wrapper'")

    def test_vocabulary_quoted_title_in_a_reference_link_is_not_a_mention(self):
        # A quoted title placed before the page's linked prose mention of
        # "conversion" must not count as the first mention.
        self.edit("examples/fn/05-boundary.c.md",
                  "# Function taking an owned record — Assemble the native boundary — C\n",
                  "# Function taking an owned record — Assemble the native boundary — C\n\n"
                  "See [Plan value conversions][fn_values_c] first.\n")
        self.assertTrue(validate.validate(self.root).startswith("Valid:"))
        # And the same words outside a title-quoting link are a first mention.
        self.edit("examples/fn/05-boundary.c.md",
                  "See [Plan value conversions][fn_values_c] first.",
                  "See how value conversions are planned first.")
        self.rejects("first mention of 'conversion'")

    def test_vocabulary_multiword_first_mention_across_a_soft_break_is_found(self):
        self.edit("stages/03-requests.md", "# Record binding requests\n",
                  "# Record binding requests\n\nA source\nitem comes from capture.\n")
        self.rejects("first mention of 'source item'")

    def test_vocabulary_double_backtick_code_is_code(self):
        # Neither a definition nor a mention, whatever the delimiter length.
        self.edit("stages/02-flat.md",
                  "[relations](04-values.md#what-a-relation-is)",
                  "``**relation**`` [relations](04-values.md#what-a-relation-is)")
        self.assertTrue(validate.validate(self.root).startswith("Valid:"))
        self.edit("stages/02-flat.md",
                  "``**relation**``",
                  "``a\nrelation `here` too``")
        self.assertTrue(validate.validate(self.root).startswith("Valid:"))
        self.edit("stages/06-retain.md", "exactly one **outcome**", "exactly one ``**outcome**``")
        self.rejects("does not define 'outcome' in bold")

    def test_vocabulary_concepts_entry_ends_at_a_group_heading(self):
        # The next group's introduction must not satisfy the previous entry.
        self.edit("concepts.md",
                  "[Record binding requests](stages/03-requests.md#record-binding-requests)\n\n"
                  "## What the registry plans\n",
                  "Record binding requests\n\n## What the registry plans\n\n"
                  "See [Record binding requests](stages/03-requests.md#record-binding-requests).\n")
        self.rejects("the 'part' entry does not link")

    def test_vocabulary_wrapped_title_in_a_reference_link_is_still_a_title(self):
        self.edit("examples/fn/README.md", "[Capture source items][fn_source]",
                  "[Capture\nsource items][fn_source]")
        self.assertTrue(validate.validate(self.root).startswith("Valid:"))

    def test_vocabulary_entry_missing_from_concepts(self):
        self.edit("concepts.md", "### Relation\n", "### Relations\n")
        self.rejects("no '### relation' entry")

    def test_vocabulary_definition_missing(self):
        self.edit("stages/06-retain.md", "exactly one **outcome**", "exactly one outcome")
        self.rejects("does not define 'outcome' in bold")

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
