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

    def test_colon_after_paren_and_plain_labels(self):
        self.assertEqual(md("CSWで確定したJPスケール(参考):"), "CSWで確定したJPスケール(参考)：")
        self.assertEqual(md("- MDN: `line-break`、Safari"), "- MDN：`line-break`、Safari")
        self.assertEqual(md("- MDN: see `line-break`"), "- MDN: see `line-break`")
        for text in ["f(): x", "(12:30)"]:
            self.assertEqual(md(text), text)

    def test_bold_label_with_the_colon_inside(self):
        self.assertEqual(md("- **共有:** 既存のClaudeと同じ"), "- **共有**：既存のClaudeと同じ")
        self.assertEqual(md("**注意：** 設定は環境ごと"), "**注意**：設定は環境ごと")

    def test_english_line_quoting_a_japanese_label(self):
        self.assertIn("**Launch**: Select",
                      md("3. **Launch**: Select an environment and press「この環境を起動」."))
        self.assertEqual(md("Press the button: 「この環境を起動」."), "Press the button: 「この環境を起動」.")
        self.assertEqual(md("- **ボタン**: 「複製」を押す"), "- **ボタン**：「複製」を押す")

    def test_guards_next_to_japanese(self):
        self.assertEqual(md("Rustの::演算子"), "Rustの::演算子")
        self.assertEqual(md("演算子:/で"), "演算子:/で")
        self.assertEqual(js("'https://ja.wikipedia.org/wiki/Help:目次'"),
                         "'https://ja.wikipedia.org/wiki/Help:目次'")
        self.assertEqual(rs('format!("{}: 利用中", x)'), 'format!("{}：利用中", x)')

    def test_url_ends_at_japanese_punctuation(self):
        kept = ["公式ドキュメント（https://code.claude.com/docs）の`statusline`を使う",
                "URLはhttps://example.com<!-- 注 -->です"]
        for text in kept:
            self.assertEqual(md(text), text)
        self.assertEqual(html("<p>公式（https://example.com）の<code>csw</code>を使う</p>"),
                         "<p>公式（https://example.com）の<code>csw</code>を使う</p>")
        self.assertEqual(md("（https://example.com）を参照。手順: 実行"),
                         "（https://example.com）を参照。手順：実行")

    def test_code_and_link_targets_stay(self):
        for text in ["~~~yaml\ndescription: 日本語の説明\n~~~", "[ドキュメント](#見出し:1)を参照"]:
            self.assertEqual(md(text), text)
        for text in ['<a href="#sec:日本語">目次</a>', '<span style="font-family: ヒラギノ角ゴシック">x</span>']:
            self.assertEqual(html(text), text)
        literal = "const h = `<style>p { font-family: ヒラギノ角ゴシック; }</style>`;"
        self.assertEqual(js(literal), literal)

    def test_front_matter_value_is_prose(self):
        self.assertEqual(md("---\nname: x\ndescription: 日本語の Claude を使う\n---\n本文"),
                         "---\nname: x\ndescription: 日本語のClaudeを使う\n---\n本文")

    def test_ignore_marker(self):
        line = 'assert_eq!(f("仕事"), "● 仕事: In use"); // ja-spacing: ignore'
        self.assertEqual(rs(line), line)

    def test_idempotent(self):
        for text in ["- **共有**: 既存のClaude", "最終起動: 3時間前", "例: `Work`",
                     "**どう分けますか? 3つのモードから選びます**", "本当に削除しますか?! 戻せません"]:
            once = md(text)
            self.assertEqual(md(once), once)


class Marks(unittest.TestCase):
    def test_after_japanese(self):
        self.assertEqual(md("4. **どう分けますか? 3つのモードから選びます**"),
                         "4. **どう分けますか？3つのモードから選びます**")
        self.assertEqual(html('<span class="field-label">既存のClaudeから、どう分けますか?</span>'),
                         '<span class="field-label">既存のClaudeから、どう分けますか？</span>')
        self.assertEqual(md("警告! 戻せません"), "警告！戻せません")
        self.assertEqual(md("本当に削除しますか?!"), "本当に削除しますか？！")
        self.assertEqual(md("**本当**? 次へ"), "**本当**？次へ")
        self.assertEqual(md("どう分けますか?Claude Codeの場合"), "どう分けますか？Claude Codeの場合")
        self.assertEqual(rs('let s = "本当?";'), 'let s = "本当？";')

    def test_dictionary_key_only(self):
        line = ("  '既存のClaudeから、どう分けますか?': "
                "'How do you want to separate this from your existing Claude?',")
        self.assertEqual(js(line), line.replace("か?'", "か？'"))

    def test_question_between_latin_and_japanese(self):
        self.assertEqual(md("対象はClaude Code? 次は環境です"), "対象はClaude Code？次は環境です")

    def test_spaces_around_fullwidth_marks(self):
        self.assertEqual(md("分けますか ？ 3つ"), "分けますか？3つ")
        self.assertEqual(md("秘密とは？ この本で"), "秘密とは？この本で")
        self.assertEqual(md("完了！ 次へ"), "完了！次へ")
        self.assertEqual(md("**本当？** 次へ"), "**本当？** 次へ")

    def test_marks_that_stay(self):
        kept = [
            "Is this your existing Claude?",
            "Ready? Go!",
            "Did you press「この環境を起動」?",
            "設定は?lang=jaで切り替える",
            "画像は?v=0.24.5で更新する",
            "CSSの!importantを使わない",
            "Rustのformat!マクロ",
            "次の図![画面](a.png)を見る",
            "[LP](https://example.com/?lang=ja)を見る",
            "公式（https://example.com/?lang=ja）を見る",
            "`csw doctor?`を実行",
            "演算子!=で比べる",
        ]
        for text in kept:
            self.assertEqual(md(text), text, text)
        self.assertEqual(html('<a href="?lang=en">English</a>'), '<a href="?lang=en">English</a>')
        self.assertEqual(html('<img src="assets/作成.png?v=0.24.5">'), '<img src="assets/作成.png?v=0.24.5">')
        self.assertEqual(html("<style>p { color: red !important; }</style>"),
                         "<style>p { color: red !important; }</style>")
        self.assertEqual(html("<code>本当?</code>"), "<code>本当?</code>")
        self.assertEqual(html("<p>Ready? Go!</p>"), "<p>Ready? Go!</p>")
        for code in ['println!("完了");', 'let s = format!("{}件", n)?;',
                     'if !ok { return Err("失敗".into()); }']:
            self.assertEqual(rs(code), code, code)
        for code in ["const a = ok ? 'はい' : 'いいえ';",
                     "const t = `${ok ? 'はい' : 'いいえ'}`;",
                     "const u = `${base}?lang=${lang}`;",
                     "if (!a && b?.c) { f('完了'); }"]:
            self.assertEqual(js(code), code, code)

    def test_ignore_marker(self):
        line = "例: どう分けますか? ja-spacing: ignore"
        self.assertEqual(md(line), line)


if __name__ == "__main__":
    unittest.main(verbosity=1)
