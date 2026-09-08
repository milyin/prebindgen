"""Negative checks for the documentation structure, isolated in temporary copies."""
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from validate import InvalidSpec, validate


class StructureTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name) / 'repo/docs/v2'
        shutil.copytree(Path(__file__).resolve().parent, self.root,
                        ignore=shutil.ignore_patterns('__pycache__'))
        source_dir = Path(__file__).resolve().parents[2] / 'prebindgen-flat/src/flat'
        target_dir = self.root.parents[1] / 'prebindgen-flat/src/flat'
        target_dir.mkdir(parents=True)
        for name in ['mod.rs', 'element.rs', 'ty.rs']:
            shutil.copy2(source_dir / name, target_dir / name)

    def change(self, name, before, after):
        path = self.root / name
        text = path.read_text()
        self.assertIn(before, text)
        path.write_text(text.replace(before, after, 1))

    def manifest(self, edit):
        path = self.root / 'manifest.json'
        data = json.loads(path.read_text())
        edit(data)
        path.write_text(json.dumps(data))

    def rejected(self, message):
        with self.assertRaisesRegex(InvalidSpec, message):
            validate(self.root)

    def test_valid_applicable_paths_skip_struct_boundary(self):
        self.assertFalse((self.root / 'examples/struct/05-boundary.md').exists())
        self.assertEqual(validate(self.root), (53, 7, 2, 13))

    def test_missing_declared_page(self):
        (self.root / 'examples/struct/04-values.kotlin.md').unlink()
        self.rejected('page coverage')

    def test_unlisted_page(self):
        (self.root / 'examples/struct/05-boundary.md').write_text('# invented\n')
        self.rejected('unlisted')

    def test_missing_language(self):
        self.manifest(lambda d: next(c for c in d['cells'] if c['languages'])['languages'].pop())
        self.rejected('language coverage')

    def test_duplicate_cell(self):
        self.manifest(lambda d: d['cells'].append(d['cells'][0]))
        self.rejected('duplicate cell')

    def test_wrong_identity(self):
        self.change('examples/struct/02-flat.md', '"example": "struct"', '"example": "function"')
        self.rejected('metadata')

    def test_backlink_to_wrong_existing_chapter(self):
        self.change('examples/function/02-flat.md', '../../stages/02-flat.md', '../../stages/01-source.md')
        self.rejected('missing required link')

    def test_stage_index_omits_variant(self):
        self.change('stages/03-requests.md', ' · [kotlin](../examples/function/03-requests.kotlin.md)', '')
        self.rejected('Apply this stage links')

    def test_wrong_existing_next_cell(self):
        self.change('examples/struct/04-values.md', '[Next](06-retain.md)', '[Next](07-emit.md)')
        self.rejected('Along this example links')

    def test_empty_contract_section(self):
        self.change('examples/function/01-source.md',
                    'The source capture mechanism records one function item.', '')
        self.rejected('empty')

    def test_broken_anchor(self):
        self.change('examples/function/02-flat.md', '../../stages/02-flat.md', '../../stages/02-flat.md#absent')
        self.rejected('missing anchor')

    def test_unclosed_fence(self):
        path = self.root / 'source.md'
        path.write_text(path.read_text() + '\n```rust\n')
        self.rejected('unclosed code fence')

    def test_missing_detailed_contract(self):
        (self.root / 'stages/04-values/target-operations.md').unlink()
        self.rejected('page coverage')

    def test_missing_contract_backlink(self):
        self.change('stages/02-flat/flat-model.md', '../02-flat.md', '../01-source.md')
        self.rejected('missing required link')

    def test_stage_omits_contract(self):
        self.change('stages/04-values.md',
                    '- [Individual target operations](04-values/target-operations.md)', '')
        self.rejected('Detailed contracts links')

    def test_duplicate_contract(self):
        self.manifest(lambda d: d['contracts'].append(d['contracts'][0]))
        self.rejected('duplicate contract ID')

    def test_unknown_manifest_field(self):
        self.manifest(lambda d: d.update(unknown=True))
        self.rejected('expected fields')


if __name__ == '__main__':
    unittest.main()
