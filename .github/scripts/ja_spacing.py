#!/usr/bin/env python3
"""Keep Japanese text free of spaces between Japanese and Latin characters.

CSW's Japanese copy puts no space between Japanese characters and Latin
letters, digits or inline code (和欧間の空白を入れない). Spacing is the
renderer's job, not the text's (japanese-typography-qa §5). This script finds
such spaces (--check, run in CI Lint) and removes them (--fix) in the
user-facing files listed in TARGETS.

A space is removed only when a Japanese character sits on one side and a
Latin letter, a digit, inline code or an inline HTML tag on the other. Kept:
- spaces between Latin words ("Claude Code");
- separators written with a space on both sides (" ・ ", " ／ ", " / ",
  " - ", " -> ");
- Markdown syntax: a heading's leading number ("## 8.1 見出し"), list markers
  and task checkboxes ("- [ ] 項目");
- a half-width "(" after Japanese ("日本語 (Japanese)", an English gloss);
- inline code, <code> elements, fenced code blocks, HTML comments, and
  comments in JavaScript and Rust (English prose that quotes Japanese);
- any line that contains the marker "ja-spacing: ignore".

Markdown bold around a bracketed label is normalized so it needs no spaces:
"**「複製」** ボタン" becomes "「**複製**」ボタン". Bold whose inner text ends
with punctuation cannot close without a space, so the bracket goes outside.

--check also fails on a Markdown soft line break between Japanese and Latin
text inside a paragraph (browsers render it as a space); join those lines.

Usage:
    ja_spacing.py --check [files...]   exit 1 and list offending lines
    ja_spacing.py --fix [files...]     rewrite the files in place
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]

TARGETS = [
    "README.md",
    "docs/USER_GUIDE.md",
    "docs/PRIVACY.md",
    "docs/SPECIFICATION.md",
    "website/ja/index.html",
    "crates/desktop/ui/index.html",
    "crates/desktop/ui/main.js",
    "crates/desktop/src/main.rs",
    "scripts/ogshot/gen-og.mjs",
    ".github/release-readme.md",
    ".github/release-notes-assets.md",
    ".agents/skills/csw_product_canon/SKILL.md",
    ".agents/skills/japanese-typography-qa/SKILL.md",
]

IGNORE_MARKER = "ja-spacing: ignore"

# Japanese characters: CJK punctuation, kana, ideographs and full-width forms,
# minus the middle dot (・) and the full-width slash (／), which the UI uses as
# spaced separators.
JA = r"[　-〿぀-ヺー-ヿ㐀-䶿一-鿿＀-．０-￯]"
# Latin text before the space ( marks protected code)...
LEFT = r"[A-Za-z0-9`)\]%}\"]"
# ...or after it. A half-width "(" is left out on purpose (English gloss).
RIGHT = r"[A-Za-z0-9`\[~/$.\-\"@#%§]"
# Separator tokens written with spaces on both sides stay spaced.
NOT_SEPARATOR = r"(?![/\-.~$@#>%]+(?:[ \t]|$))"

SPACE_AFTER_JA = re.compile(rf"(?<={JA})[ \t]+(?={RIGHT}){NOT_SEPARATOR}", re.M)
SPACE_BEFORE_JA = re.compile(rf"(?<={LEFT})[ \t]+(?={JA})")
# A path that ends with a slash ("rules/ に", "<プロジェクト>/ の") is Latin
# too; a slash with a space before it ("2026 / 見出し") is a separator.
SPACE_AFTER_PATH = re.compile(rf"(?<=[A-Za-z0-9_.\-<>]/)[ \t]+(?={JA})")

# HTML inline elements that carry Latin text (<code>csw</code>, links).
INLINE = r"(?:code|a|strong|em|b|span|kbd)"
SPACE_BEFORE_TAG = re.compile(rf"({JA})[ \t]+(<{INLINE}\b)")
SPACE_AFTER_TAG = re.compile(rf"(</{INLINE}>)[ \t]+({JA})")

HTML_COMMENT = re.compile(r"<!--.*?-->", re.S)
HTML_CODE = re.compile(r"<code\b[^>]*>.*?</code>", re.S)
INLINE_CODE = re.compile(r"`[^`\n]+`")

# Markdown line prefixes that are syntax, not prose: a heading's leading
# number ("## 0. ", "## 8.1 ") and a list marker with an optional checkbox.
MD_PREFIX = re.compile(
    r"^(?:#{1,6}[ \t]+\d+(?:\.\d+)*\.[ \t]+|#{1,6}[ \t]+\d+(?:\.\d+)+[ \t]+"
    r"|[ \t]*(?:[-*+]|\d+[.)])[ \t]+(?:\[[ xX]\][ \t]+)?)"
)
# Bold around a bracketed label: move the bracket outside the bold.
BOLD_BRACKET = re.compile(r"\*\*「([^」*\n]+)」\*\*")
SPACE_BEFORE_BRACKET_BOLD = re.compile(rf"(?<={JA})[ \t]+(?=「\*\*)")
SPACE_AFTER_BRACKET_BOLD = re.compile(rf"(?<=\*\*」)[ \t]+(?={JA})")
# Bold next to Latin text: "**Claude Code** の" renders as "**Claude Code**の"
# because the closing ** follows a letter. Bold that ends with punctuation or
# code keeps its space, or the ** no longer closes.
BOLD_THEN_JA = re.compile(rf"(?<=[A-Za-z0-9])(\*\*|\*)[ \t]+(?={JA})")
JA_THEN_BOLD = re.compile(rf"(?<={JA})[ \t]+(\*\*|\*)(?=[A-Za-z0-9])")


def _squeeze(text: str) -> str:
    text = SPACE_AFTER_JA.sub("", text)
    text = SPACE_AFTER_PATH.sub("", text)
    return SPACE_BEFORE_JA.sub("", text)


def _tags(text: str) -> str:
    text = SPACE_BEFORE_TAG.sub(r"\1\2", text)
    return SPACE_AFTER_TAG.sub(r"\1\2", text)


def _protect(text: str, pattern: re.Pattern, saved: list[str]) -> str:
    """Swap each match for a placeholder that counts as a Latin boundary."""

    def keep(m: re.Match) -> str:
        saved.append(m.group(0))
        return "" + chr(0xE100 + len(saved) - 1) + ""

    return pattern.sub(keep, text)


def _restore(text: str, saved: list[str]) -> str:
    return re.sub(r"(.)", lambda m: saved[ord(m.group(1)) - 0xE100], text)


def fix_markdown(text: str) -> str:
    out, fenced = [], False
    for line in text.split("\n"):
        if line.lstrip().startswith("```"):
            fenced = not fenced
            out.append(line)
            continue
        if fenced:
            out.append(line)
            continue
        head = MD_PREFIX.match(line)
        prefix = head.group(0) if head else ""
        saved: list[str] = []
        body = _protect(line[len(prefix):], HTML_COMMENT, saved)
        body = _protect(body, INLINE_CODE, saved)
        body = BOLD_BRACKET.sub(r"「**\1**」", body)
        body = SPACE_BEFORE_BRACKET_BOLD.sub("", body)
        body = SPACE_AFTER_BRACKET_BOLD.sub("", body)
        body = BOLD_THEN_JA.sub(r"\1", body)
        body = JA_THEN_BOLD.sub(r"\1", body)
        out.append(prefix + _restore(_squeeze(_tags(body)), saved))
    return "\n".join(out)


def fix_html(text: str) -> str:
    saved: list[str] = []
    body = _protect(text, HTML_COMMENT, saved)
    body = _protect(body, HTML_CODE, saved)
    return _restore(_squeeze(_tags(body)), saved)


class ScanError(Exception):
    pass


LITERAL_START = re.compile(rf"^[ \t]+(?={JA})")
LITERAL_END = re.compile(rf"(?<={JA})[ \t]+$")


def _squeeze_literal(text: str) -> str:
    # String literals can carry HTML (the OG card template), and a literal is
    # often joined with an element or a value at runtime, so a space at either
    # end next to Japanese is a join space and goes too.
    text = _squeeze(_tags(text))
    text = LITERAL_START.sub("", text)
    return LITERAL_END.sub("", text)


RUST_CHAR = re.compile(r"'(?:\\.|\\u\{[0-9a-fA-F]+\}|[^\\'\n])'")
RUST_RAW = re.compile(r'r(#*)"')


def fix_code(text: str, rust: bool = False) -> str:
    """JavaScript and Rust: change string literals only; comments are kept."""
    out, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            out.append(text[i:j]); i = j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append(text[i:j]); i = j
        elif rust and c == "'":
            m = RUST_CHAR.match(text, i)
            j = m.end() if m else i + 1  # a char literal, or a lifetime
            out.append(text[i:j]); i = j
        elif rust and c == "r" and RUST_RAW.match(text, i) and not (i and (text[i - 1].isalnum() or text[i - 1] == "_")):
            m = RUST_RAW.match(text, i)
            close = '"' + m.group(1)
            j = text.find(close, m.end())
            if j < 0:
                raise ScanError(f"unterminated raw string at offset {i}")
            out.append(text[i:m.end()] + _squeeze_literal(text[m.end():j]) + close)
            i = j + len(close)
        elif c in ('"' if rust else "'\"`"):
            j = i + 1
            while j < n and text[j] != c:
                if text[j] == "\\":
                    j += 1
                elif not rust and c != "`" and text[j] == "\n":
                    line = text.count("\n", 0, i) + 1
                    raise ScanError(f"line {line}: a {c}-quoted literal runs to the end of the line; "
                                    "the scanner lost track (a regex literal with a quote?)")
                j += 1
            out.append(c + _squeeze_literal(text[i + 1:j]) + (text[j] if j < n else ""))
            i = j + 1
        else:
            out.append(c); i += 1
    return "".join(out)


def fixer(path: str):
    if path.endswith(".md"):
        return fix_markdown
    if path.endswith(".html"):
        return fix_html
    if path.endswith(".rs"):
        return lambda text: fix_code(text, rust=True)
    return fix_code


def fix(path: str, text: str) -> str:
    fixed = fixer(path)(text)
    before, after = text.split("\n"), fixed.split("\n")
    assert len(before) == len(after), "the fixer must not add or remove lines"
    return "\n".join(b if IGNORE_MARKER in b else a for b, a in zip(before, after))


MD_PARAGRAPH = re.compile(r"^(?![ \t]*(?:#|[-*+>|]|\d+[.)]|```|<|$))")


def soft_breaks(path: str, text: str) -> list[int]:
    """Markdown lines in one paragraph whose line break sits between Japanese
    and Latin text (a browser renders that break as a space)."""
    if not path.endswith(".md"):
        return []
    hits, lines, fenced = [], text.split("\n"), False
    for k in range(len(lines) - 1):
        a, b = lines[k], lines[k + 1]
        if a.lstrip().startswith("```"):
            fenced = not fenced
        if a.lstrip().startswith("```") or fenced or IGNORE_MARKER in a:
            continue
        # Both lines must be paragraph text (not a heading, list, table,
        # quote, fence or HTML), and the first must not end with a hard break.
        if not a.strip() or not MD_PARAGRAPH.match(a) or not MD_PARAGRAPH.match(b):
            continue
        if a.endswith("  ") or a.endswith("\\") or a.rstrip().endswith("<br>"):
            continue
        end, start = a.rstrip()[-1:], b.lstrip()[:1]
        if (re.match(JA, end) and re.match(r"[A-Za-z0-9`]", start)) or (
            re.match(r"[A-Za-z0-9`]", end) and re.match(JA, start)
        ):
            hits.append(k + 1)
    return hits


def main(argv: list[str]) -> int:
    if not argv or argv[0] not in ("--check", "--fix"):
        print(__doc__)
        return 2
    mode, files = argv[0], (argv[1:] or TARGETS)
    bad = 0
    for rel in files:
        p = ROOT / rel
        before = p.read_text(encoding="utf-8")
        try:
            after = fix(rel, before)
        except ScanError as e:
            print(f"{rel}: {e}")
            bad += 1
            continue
        for k in soft_breaks(rel, after):
            bad += 1
            print(f"{rel}:{k}: a line break inside a paragraph sits between Japanese and Latin text; join the two lines")
        if after == before:
            continue
        if mode == "--fix":
            p.write_text(after, encoding="utf-8")
            print(f"fixed {rel}")
            continue
        for k, (a, b) in enumerate(zip(before.split("\n"), after.split("\n")), 1):
            if a != b:
                bad += 1
                print(f"{rel}:{k}: space between Japanese and Latin text: {a.strip()[:120]}")
    if bad:
        print(f"\n{bad} problem(s). Run: python3 .github/scripts/ja_spacing.py --fix "
              "(line breaks inside a paragraph are joined by hand)")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
