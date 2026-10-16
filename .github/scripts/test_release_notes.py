import pathlib
import tempfile
import unittest
from unittest.mock import patch

import release_notes as notes


class ReleaseNotesTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.directory = self.root / '.github/release-notes'
        self.directory.mkdir(parents=True)
        self.addCleanup(patch.stopall)
        patch.object(notes, 'ROOT', self.root).start()
        patch.object(notes, 'CHANGELOG', self.root / 'CHANGELOG.md').start()
        patch.object(notes, 'ASSET_GUIDE', self.root / 'guide.md').start()
        notes.CHANGELOG.write_text('## [1.2.3](compare)\n\n### Bug Fixes\n\n- 日本語の修正\n\n## [1.2.2]\n古い版\n')
        notes.ASSET_GUIDE.write_text('### 添付ファイルの説明\n__VERSION__\n\n### About the attached files\nGuide')
        self.source = self.directory / 'v1.2.3.en.md'
        self.source.write_text('# v1.2.3\n\n## Changes (English)\n\n- Correct the behavior.\n')

    def test_source_exact_tag(self):
        self.assertEqual(notes.load_english('v1.2.3'), '## Changes (English)\n\n- Correct the behavior.')

    def test_source_missing_or_invalid(self):
        for tag in ['v1.2.4', '../v1.2.3', '1.2.3', '--help', 'v1.2.3/other']:
            with self.subTest(tag=tag), self.assertRaises((ValueError, FileNotFoundError)):
                notes.load_english(tag)

    def test_source_wrong_version_empty_heading_or_duplicate(self):
        for source in ['# v1.2.4\n\n## Changes (English)\n- Fixed.', '# v1.2.3\n\n## Changes (English)', '# v1.2.3\n\n## English\n- Guide', '# v1.2.3\n\n## Changes (English)\n<!-- TODO -->', '# v1.2.3\n\n## Changes (English)\n- Fixed.\n## Changes (English)\n- Duplicate.']:
            self.source.write_text(source)
            with self.subTest(source=source), self.assertRaises(ValueError):
                notes.load_english('v1.2.3')

    def test_assembly_recovery_and_idempotency(self):
        en = notes.load_english('v1.2.3')
        body = notes.assemble('v1.2.3', '', None, source_english=en)
        notes.check('v1.2.3', body, True)
        self.assertEqual(notes.assemble('v1.2.3', body, None, source_english=en), body)
        self.assertEqual(body.count('## Changes (English)'), 1)
        self.assertNotIn('古い版', body)

    def test_source_updates_existing_english_preserving_japanese(self):
        ja = '## [1.2.3](compare)\n\n- 日本語を保持'
        old = ja + '\n\n## Changes (English)\n\n- Old text.\n\n### 添付ファイルの説明\nOld guide'
        body = notes.assemble('v1.2.3', old, None, source_english=notes.load_english('v1.2.3'))
        self.assertTrue(body.startswith(ja + '\n\n'))
        self.assertNotIn('Old text', body)
        self.assertIn('Correct the behavior', body)

    def test_check_rejects_empty_english_or_wrong_version(self):
        for body in ['## [1.2.3]\n日本語\n\n## Changes (English)\n\n### 添付ファイルの説明\nguide', '## [9.9.9]\n日本語\n\n## Changes (English)\n- Fixed\n\n### 添付ファイルの説明\nguide']:
            with self.subTest(body=body), self.assertRaises(SystemExit):
                notes.check('v1.2.3', body, True)

    def test_validate_current_requires_bumped_version_source(self):
        (self.root / '.release-please-manifest.json').write_text('{".":"1.2.3"}')
        notes.validate_current()
        (self.root / '.release-please-manifest.json').write_text('{".":"1.2.4"}')
        with self.assertRaises(FileNotFoundError):
            notes.validate_current()


if __name__ == '__main__':
    unittest.main()
