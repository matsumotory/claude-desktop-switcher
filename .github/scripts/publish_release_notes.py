#!/usr/bin/env python3
"""Update existing release bodies from reviewed version-keyed sources only.

The sole write is PATCH /releases/{numeric id} with a body field. This script
never creates releases, changes tags, or uploads/deletes assets.
"""
import json
import os
import re
import subprocess
import sys

from release_notes import assemble, check, load_english, validate_tag


def api(endpoint: str, *, body: str | None = None, allow_missing: bool = False):
    args = ['gh', 'api', endpoint]
    payload = None
    if body is not None:
        args += ['--method', 'PATCH', '--input', '-']
        payload = json.dumps({'body': body})
    result = subprocess.run(args, input=payload, text=True, capture_output=True)
    if result.returncode:
        if allow_missing and '(HTTP 404)' in result.stderr:
            return None
        raise RuntimeError(result.stderr)
    return json.loads(result.stdout)


def update(repo: str, tag: str, *, skip_missing: bool = False) -> None:
    validate_tag(tag)
    english = load_english(tag)  # Fail before any API write on missing/invalid source.
    release = api(f'repos/{repo}/releases/tags/{tag}', allow_missing=skip_missing)
    if release is None:
        if skip_missing:
            print(f'{tag}: no existing release; the release workflow will publish its notes')
            return
        raise ValueError(f'{tag}: release does not exist')
    if release['tag_name'] != tag or release['draft']:
        raise ValueError(f'{tag}: expected an existing published release for this exact tag')
    release_id = release['id']
    if not isinstance(release_id, int) or release_id <= 0:
        raise ValueError('Invalid release id')
    body = assemble(tag, release.get('body') or '', None, source_english=english)
    check(tag, body, require_english=True)
    if body == release.get('body'):
        print(f'{tag}: body already matches the reviewed source')
        return
    endpoint = f'repos/{repo}/releases/{release_id}'
    api(endpoint, body=body)
    written = api(endpoint)
    if written['tag_name'] != tag or written.get('body') != body:
        raise ValueError(f'{tag}: written release body did not match')
    check(tag, written['body'], require_english=True)
    print(f'{tag}: verified Japanese changes, English changes, and attached-files guide')


def changed_tags(before: str, after: str) -> list[str]:
    if not all(re.fullmatch(r'[0-9a-f]{40}', ref) and ref != '0' * 40 for ref in (before, after)):
        raise ValueError('Expected two existing commit SHAs')
    paths = subprocess.check_output(
        ['git', 'diff', '--no-renames', '--name-only', '--diff-filter=AM', '-z', before, after,
         '--', '.github/release-notes/'], text=True)
    tags = set()
    for path in paths.split('\0'):
        match = re.fullmatch(r'\.github/release-notes/(v[0-9]+\.[0-9]+\.[0-9]+)\.en\.md', path)
        if match:
            validate_tag(match[1])
            tags.add(match[1])
    return sorted(tags)


def main(argv: list[str]) -> None:
    repo = os.environ['GITHUB_REPOSITORY']
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repo):
        raise ValueError('Invalid repository')
    if len(argv) == 2 and argv[0] == '--existing':
        update(repo, argv[1], skip_missing=True)
    elif len(argv) == 1:
        update(repo, argv[0])
    else:
        raise ValueError('Usage: publish_release_notes.py [--existing] <tag>')


if __name__ == '__main__':
    main(sys.argv[1:])
