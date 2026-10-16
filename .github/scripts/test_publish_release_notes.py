import json
import unittest
from unittest.mock import patch

import publish_release_notes as publish


class PublishTests(unittest.TestCase):
    def setUp(self):
        self.en = '## Changes (English)\n\n- Fixed behavior.'
        self.body = '## [1.2.3]\n日本語\n\n' + self.en + '\n\n### 添付ファイルの説明\nguide\n'
        self.release = dict(id=123, tag_name='v1.2.3', body='old', draft=False)

    def test_patch_only_body_and_read_back(self):
        with patch.object(publish, 'load_english', return_value=self.en), patch.object(publish, 'assemble', return_value=self.body), patch.object(publish, 'api', side_effect=[self.release, {}, dict(self.release, body=self.body)]) as api:
            publish.update('owner/repo', 'v1.2.3')
        self.assertEqual(api.call_args_list[1].args, ('repos/owner/repo/releases/123',))
        self.assertEqual(api.call_args_list[1].kwargs, {'body': self.body})
        self.assertEqual(api.call_count, 3)

    def test_idempotent_no_patch(self):
        with patch.object(publish, 'load_english', return_value=self.en), patch.object(publish, 'assemble', return_value=self.body), patch.object(publish, 'api', return_value=dict(self.release, body=self.body)) as api:
            publish.update('owner/repo', 'v1.2.3')
        self.assertEqual(api.call_count, 1)

    def test_missing_release_backfill_skips_but_release_call_fails(self):
        with patch.object(publish, 'load_english', return_value=self.en), patch.object(publish, 'api', return_value=None):
            publish.update('owner/repo', 'v1.2.3', skip_missing=True)
            with self.assertRaises(ValueError):
                publish.update('owner/repo', 'v1.2.3')

    def test_invalid_source_never_calls_api(self):
        with patch.object(publish, 'load_english', side_effect=ValueError('invalid')), patch.object(publish, 'api') as api:
            with self.assertRaises(ValueError):
                publish.update('owner/repo', 'v1.2.3')
        api.assert_not_called()

    def test_wrong_release_never_writes(self):
        with patch.object(publish, 'load_english', return_value=self.en), patch.object(publish, 'api', return_value=dict(self.release, tag_name='v9.9.9')) as api:
            with self.assertRaises(ValueError):
                publish.update('owner/repo', 'v1.2.3')
        self.assertEqual(api.call_count, 1)

    def test_readback_mismatch_fails(self):
        with patch.object(publish, 'load_english', return_value=self.en), patch.object(publish, 'assemble', return_value=self.body), patch.object(publish, 'api', side_effect=[self.release, {}, self.release]):
            with self.assertRaises(ValueError):
                publish.update('owner/repo', 'v1.2.3')

    def test_api_only_skips_404(self):
        for status in [403, 500, 404]:
            result = publish.subprocess.CompletedProcess([], 1, '', f'gh: error (HTTP {status})')
            with patch.object(publish.subprocess, 'run', return_value=result):
                if status == 404:
                    self.assertIsNone(publish.api('repos/o/r/releases/tags/v1.2.3', allow_missing=True))
                else:
                    with self.assertRaises(RuntimeError):
                        publish.api('repos/o/r/releases/tags/v1.2.3', allow_missing=True)

    def test_api_sends_body_as_json_stdin_without_shell(self):
        result = publish.subprocess.CompletedProcess([], 0, '{}', '')
        text = '- Keep $(touch /tmp/unwanted) as plain text.'
        with patch.object(publish.subprocess, 'run', return_value=result) as run:
            publish.api('repos/o/r/releases/123', body=text)
        args, kwargs = run.call_args
        self.assertEqual(args[0], ['gh', 'api', 'repos/o/r/releases/123', '--method', 'PATCH', '--input', '-'])
        self.assertEqual(json.loads(kwargs['input']), {'body': text})
        self.assertNotIn('shell', kwargs)

    def test_changed_paths_only_exact_notes(self):
        paths = '.github/release-notes/v1.2.3.en.md\0README.md\0.github/release-notes/README.md\0'
        with patch.object(publish.subprocess, 'check_output', return_value=paths) as git:
            self.assertEqual(publish.changed_tags('a'*40, 'b'*40), ['v1.2.3'])
            self.assertIn('--no-renames', git.call_args.args[0])
        with self.assertRaises(ValueError):
            publish.changed_tags('--bad', 'b'*40)


if __name__ == '__main__':
    unittest.main()
