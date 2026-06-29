---
name: commit_standard
description: Defines the standard format and rules for git commit messages in this project.
---

# Commit Message Standard

This project uses a standardized git commit message format based on Conventional Commits, with an added touch of our project's specific context.

## Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

## Rules

### 1. Type
Must be one of the following:
- `feat`: A new feature (e.g., adding a new user profile section)
- `fix`: A bug fix (e.g., fixing tooltip text)
- `docs`: Documentation only changes (e.g., updating README, SPECIFICATION, or USER_GUIDE)
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools and libraries such as documentation generation

### 1.1 リリースへの影響(release-please・超重要)

このリポジトリは release-please で版を管理する。**`fix:` / `feat:` / 破壊的変更(`!` または `BREAKING CHANGE`)のいずれかが `main` に入ったときだけ**リリース PR が起票され、マージすると署名・公証つき DMG をビルドして GitHub Release を切る。`docs:` / `chore:` / `style:` / `refactor:` / `perf:` / `test:` / `ci:` は版を上げず、リリース PR を作らない。

判断基準は「**出荷するアプリ本体(`crates/`)を変えるか**」であって、概念的に bug fix かどうかではない:

- `crates/`(core / cli / desktop のロジック)を変える → `fix:` / `feat:`(= リリース対象)
- `website/`(LP)・`docs/`・`README` だけ、または `.agents/` のスキル・CI 補助だけ → **`docs:` か `chore:` を使う。`fix:` / `feat:` は使わない**

理由: LP は GitHub Pages で**アプリのリリースとは独立にデプロイ**される。LP やドキュメントだけの変更で `fix:` を使うと、アプリのコードは何も変わっていないのに版が上がり、中身が前バージョンと同一の DMG をリリースしてしまう(2026-06-29、技術コラムとスクショを `fix(website)` で積んだ結果、ドキュメントだけの 0.9.2 リリース PR が起票された)。LP の不具合(リンク切れ・レイアウト崩れ)を直すときも、`crates/` を触らないなら `docs(website):` を使う。

例:
- `docs(website): 技術コラムを平易な説明文に書き直す`(LP のみ → リリースされない)
- `docs: USER_GUIDE に環境切替の手順を追記する`(docs のみ → リリースされない)
- `fix(core): プロファイル名検証で日本語を許可する`(アプリ変更 → リリース対象)

### 2. Scope (Optional but Recommended)
Indicates the area of the codebase the commit affects. Common scopes in this project:
- `core`: `csw-core` library (profile / switcher / linker / keychain / platform)
- `cli`: `csw-cli` command-line interface
- `desktop`: `csw-desktop` Tauri GUI (tray, settings UI)
- `ci`: GitHub Actions workflows, release tooling
- `docs`: SPECIFICATION / USER_GUIDE / README
- `website`: landing page (`website/`)

### 3. Subject
- Use the imperative, present tense: "change" not "changed" nor "changes".
- Don't capitalize the first letter.
- No dot (.) at the end.
- Keep it concise (under 50 characters if possible).

### 4. Body
- Just like in the subject, use the imperative, present tense: "change" not "changed" nor "changes".
- The body should include the motivation for the change and contrast this with previous behavior.
- Wrap the body at 72 characters.

### 5. Footer (Optional)
- Reference issue tracker IDs if applicable.
- Note any BREAKING CHANGES here.

## Example

feat(auth): add OAuth login support

Implement Google and GitHub OAuth providers for faster
user onboarding and integrate them into the login screen.
```

## Agent Instructions
When you are asked to commit changes, ALWAYS reference this skill and follow the format precisely. If committing multiple diverse changes, try to group them logically or use `feat(core)` / `chore(all)` if they span the entire project.

**CRITICAL RULE FOR THE AGENT (Claude Code):**
1. All commit subjects and bodies MUST be written in **Japanese (日本語)**, regardless of the prompt language. The `<type>(<scope>)` prefix remains in English.
   Example: `feat(core): プロファイル複製コマンドを追加`
   Commit only when the user asks. End commit messages with the trailer `Co-Authored-By: Claude <noreply@anthropic.com>`.
2. **Untracked Files Check**: Before committing, ALWAYS ensure you haven't forgotten to stage newly created files (e.g., new workflow files, reports, components). Do not blindly use `git add file1 file2` without considering if you created `file3` during the task. Use `git add .` or explicitly add all relevant new files.
3. **対話エディタの回避**: Claude Code の Bash ツールは対話的エディタを開けないため、`git merge` や `git commit`（マージコミット等）でエディタが起動するとハングする。以下の対策を**必ず**実施すること：
   - `git merge` → `git merge --no-edit <branch>` を使用
   - `git commit`（マージコミット等） → `GIT_EDITOR=true git commit --no-edit` を使用
   - **絶対にエディタが開くコマンドを素のまま実行しないこと**

## Branch Strategy

### 命名規則
- `feat/<short-description>` — 新機能
- `fix/<short-description>` — バグ修正
- `refactor/<short-description>` — リファクタリング
- `docs/<short-description>` — ドキュメント変更

### 運用ルール
1. **すべての変更**（コード・ドキュメント・plan ファイル含む）は必ず feature ブランチで作業する
2. PR を作成し、CI（`cargo fmt` / `cargo clippy` / `cargo test`）が全てパスしてからマージする
3. `main` への直接 push は禁止（GitHub branch protection rule で物理的に阻止されており例外はない）。ドキュメントのみの変更も feature ブランチ + PR 経由でマージする。詳細は [CLAUDE.md](../../../CLAUDE.md) の Branch Strategy Enforcement
