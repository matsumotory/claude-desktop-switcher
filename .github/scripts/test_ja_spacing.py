#!/usr/bin/env python3
"""Tests for ja_spacing.py. Run: python3 .github/scripts/test_ja_spacing.py"""

import importlib.util
import pathlib
import unittest

_spec = importlib.util.spec_from_file_location(
    "ja_spacing", pathlib.Path(__file__).with_name("ja_spacing.py"))
ja = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(ja)


def md(text: str) -> str:
    return ja.fix("x.md", text)


def html(text: str) -> str:
    return ja.fix("x.html", text)


def js(text: str) -> str:
    return ja.fix("x.js", text)


def rs(text: str) -> str:
    return ja.fix("x.rs", text)


class Spacing(unittest.TestCase):
    def test_space_between_japanese_and_latin(self):
        self.assertEqual(md("既存の Claude を使う"), "既存のClaudeを使う")
        self.assertEqual(md("`csw doctor` を実行"), "`csw doctor`を実行")
        self.assertEqual(js("const a = '既存の Claude';"), "const a = '既存のClaude';")

    def test_kept_spaces(self):
        for text in ["Claude Code is here", "共有 ・ 分離", "## 8.1 見出し",
                     "- [ ] 項目", "日本語 (Japanese)"]:
            self.assertEqual(md(text), text)


class Colons(unittest.TestCase):
    def test_label_and_value(self):
        self.assertEqual(js("T(`最終起動: ${x}`, `Last launched ${x}`)"),
                         "T(`最終起動：${x}`, `Last launched ${x}`)")
        self.assertEqual(js("'例: 仕事用、検証用'"), "'例：仕事用、検証用'")
        self.assertEqual(html('<input placeholder="例: 仕事用">'), '<input placeholder="例：仕事用">')

    def test_markdown_labels(self):
        self.assertEqual(md("- **共有**: 既存のClaudeと同じ"), "- **共有**：既存のClaudeと同じ")
        self.assertEqual(md("- `csw init`: ベース設定の初期化"), "- `csw init`：ベース設定の初期化")
        self.assertEqual(md("* **CLI**: iTerm2などのターミナル"), "* **CLI**：iTerm2などのターミナル")
        self.assertEqual(md("- `a` (`b`): OAuthの設定"), "- `a` (`b`)：OAuthの設定")
        self.assertEqual(md("- **フロー (`x`)**:"), "- **フロー (`x`)**：")
        self.assertEqual(md("### ステップ1: インストール"), "### ステップ1：インストール")
        self.assertEqual(md("# Claude Desktop Switcher: ユーザーガイド"),
                         "# Claude Desktop Switcher：ユーザーガイド")
        self.assertEqual(md("> **メモ: 環境の同時起動**"), "> **メモ：環境の同時起動**")

    def test_spaces_around_fullwidth_colon(self):
        self.assertEqual(md("名前 ：値"), "名前：値")
        self.assertEqual(md("**名前**： 例"), "**名前**：例")
        self.assertEqual(rs('let s = "既存の Claude ：利用中";'), 'let s = "既存のClaude：利用中";')

    def test_half_width_colons_that_stay(self):
        kept = [
            "開始は12:30です",
            "https://ja.wikipedia.org/wiki/Help:目次 を見る",
            "<https://ja.wikipedia.org/wiki/Help:目次>",
            "[^1]: 注記の本文",
            "[ref]: https://example.com",
            "`key: value`の形",
            "（Tauri command: `set_profile_note`。空文字でクリア）",
            "std::fs::remove_dir",
            "- **Unified management**: one place for both",
        ]
        for text in kept:
            self.assertEqual(md(text), text, text)
        self.assertEqual(rs('format!("● {}: In use", display)'), 'format!("● {}: In use", display)')
        self.assertEqual(html('<a href="https://ja.wikipedia.org/wiki/Help:目次">目次</a>'),
                         '<a href="https://ja.wikipedia.org/wiki/Help:目次">目次</a>')
        self.assertEqual(html("<style>p { font-family: ヒラギノ; }</style>"),
                         "<style>p { font-family: ヒラギノ; }</style>")
        self.assertEqual(html("<code>例: 値</code>"), "<code>例: 値</code>")

    def test_front_matter_is_data(self):
        text = "---\nname: x\ndescription: 日本語の説明\n---\n\n例: 本文"
        self.assertEqual(md(text), "---\nname: x\ndescription: 日本語の説明\n---\n\n例：本文")

    def test_ignore_marker(self):
        line = 'assert_eq!(f("仕事"), "● 仕事: In use"); // ja-spacing: ignore'
        self.assertEqual(rs(line), line)

    def test_idempotent(self):
        for text in ["- **共有**: 既存のClaude", "最終起動: 3時間前", "例: `Work`"]:
            once = md(text)
            self.assertEqual(md(once), once)


if __name__ == "__main__":
    unittest.main(verbosity=1)
