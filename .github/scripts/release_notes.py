#!/usr/bin/env python3
"""Assemble the user-facing body of a GitHub Release.

Usage:
    release_notes.py <tag> <current-body-file> [<english-changes-file>] > new-body.md
    release_notes.py --check [--require-english] <tag> <body-file>

The body has three parts, in this order:

1. the Japanese changelog section that release-please generated. When the
   current body is empty (the API lagged, or something overwrote the release),
   it is rebuilt from the matching section of CHANGELOG.md, so a wiped release
   can always be restored from the repository;
2. an optional "## Changes (English)" section, kept from the current body or
   taken from the english-changes file (the heading may be omitted there);
3. the attached-files guide, always regenerated from
   .github/release-notes-assets.md for the tag's version.

The three parts of a body that already has them are kept as they are, so the
script is idempotent and safe to run after every job that touches the release.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
CHANGELOG = ROOT / "CHANGELOG.md"
ASSET_GUIDE = ROOT / ".github" / "release-notes-assets.md"

# The same marker verify-release-notes.yml looks for; distinct from the guide's
# "### About the attached files" and the RELEASE-README's "## English".
ENGLISH_HEADING = re.compile(r"(?im)^#{1,6}[ \t]*Changes[ \t]*\(English\)[ \t]*$")
ENGLISH_HEADING_TEXT = "## Changes (English)"
GUIDE_HEADING = re.compile(r"(?m)^### 添付ファイルの説明[ \t]*$")


def version_of(tag: str) -> str:
    return tag[1:] if tag.startswith("v") else tag


def changelog_section(version: str) -> str:
    text = CHANGELOG.read_text(encoding="utf-8")
    pattern = rf"(?ms)^## \[{re.escape(version)}\].*?(?=^## \[|\Z)"
    match = re.search(pattern, text)
    if not match:
        sys.exit(f"release_notes.py: CHANGELOG.md has no section for {version}")
    return match.group(0).strip()


def split(body: str) -> tuple[str, str]:
    """Return (changelog, english) found in the body; the guide is discarded
    because it is regenerated from the template."""
    english_match = ENGLISH_HEADING.search(body)
    guide_match = GUIDE_HEADING.search(body)
    cut = len(body)
    if english_match:
        cut = min(cut, english_match.start())
    if guide_match:
        cut = min(cut, guide_match.start())
    changelog = body[:cut].strip()
    english = ""
    if english_match:
        end = guide_match.start() if guide_match and guide_match.start() > english_match.start() else len(body)
        english = body[english_match.start():end].strip()
    return changelog, english


def assemble(tag: str, body: str, english_file: str | None) -> str:
    version = version_of(tag)
    changelog, english = split(body)
    if not changelog:
        changelog = changelog_section(version)
    if not english and english_file:
        english = pathlib.Path(english_file).read_text(encoding="utf-8").strip()
        if english and not ENGLISH_HEADING.search(english):
            english = f"{ENGLISH_HEADING_TEXT}\n\n{english}"
    guide = ASSET_GUIDE.read_text(encoding="utf-8").replace("__VERSION__", version).strip()
    parts = [changelog]
    if english:
        parts.append(english)
    parts.append(guide)
    return "\n\n".join(parts) + "\n"


def check(tag: str, body: str, require_english: bool) -> None:
    version = version_of(tag)
    problems = []
    if not body.strip():
        problems.append("the body is empty")
    if not re.search(rf"(?m)^## \[{re.escape(version)}\]", body):
        problems.append(f"the Japanese changelog section for {version} is missing")
    if not GUIDE_HEADING.search(body):
        problems.append("the attached-files guide is missing")
    has_english = ENGLISH_HEADING.search(body) is not None
    if require_english and not has_english:
        problems.append('the "## Changes (English)" section is missing')
    if problems:
        sys.exit("release_notes.py: " + "; ".join(problems))
    note = "with" if has_english else "without"
    print(f"release_notes.py: body for {tag} has the changelog and the asset guide, {note} an English section")
    if not has_english:
        print(f"::warning::Release {tag} has no '## Changes (English)' section yet; add it with the Release notes workflow.")


def main(argv: list[str]) -> None:
    if argv and argv[0] == "--check":
        argv = argv[1:]
        require_english = False
        if argv and argv[0] == "--require-english":
            require_english = True
            argv = argv[1:]
        if len(argv) != 2:
            sys.exit(__doc__)
        tag, body_file = argv
        check(tag, pathlib.Path(body_file).read_text(encoding="utf-8"), require_english)
        return
    if len(argv) not in (2, 3):
        sys.exit(__doc__)
    tag, body_file = argv[0], argv[1]
    english_file = argv[2] if len(argv) == 3 else None
    body = pathlib.Path(body_file).read_text(encoding="utf-8")
    sys.stdout.write(assemble(tag, body, english_file))


if __name__ == "__main__":
    main(sys.argv[1:])
