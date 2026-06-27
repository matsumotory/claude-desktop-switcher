# CLAUDE.md

このファイルは Claude Code がセッション開始時に自動ロードするプロジェクト指示書です。詳細な規約は `.agents/AGENTS.md` を一次正典とし、ここから取り込みます。

@.agents/AGENTS.md

## プロジェクト概要

Claude Desktop Switcher (CSW) は、Claude Desktop アプリ (GUI) と Claude Code (CLI) のプロファイル (データディレクトリ + macOS Keychain) を安全に隔離・共有する Rust + Tauri v2 製の macOS アプリ。

- `crates/core` (`csw-core`): ビジネスロジック (OS パス抽象化・Keychain 操作・プロファイル/シンボリックリンク管理)
- `crates/cli` (`csw`): clap ベースの CLI
- `crates/desktop` (`csw-desktop`): Tauri v2 GUI (システムトレイ常駐)

正典ドキュメント: [docs/SPECIFICATION.md](docs/SPECIFICATION.md) / [docs/USER_GUIDE.md](docs/USER_GUIDE.md) / [docs/USER_GUIDE_EN.md](docs/USER_GUIDE_EN.md)。LP は `website/`。

## ビルド・テスト・検証コマンド

```bash
cargo build --workspace            # 全クレートビルド
cargo test --workspace             # テスト (core はテスト時 MockKeychainProvider を使用)
cargo clippy --workspace --all-targets -- -D warnings   # lint
cargo fmt --all                    # フォーマット
```

GUI の実機確認は `cargo tauri dev` (要 `cargo install tauri-cli`)。デスクトップ署名/公証/DMG は GitHub Actions の `Release Please` ワークフローが担う。

## Claude Code セッションでの運用

- **このリポジトリのスキルは `.agents/skills/<name>/SKILL.md` に置かれている。** バグ修正は `/bugfix`、仕様ファースト開発は `/spec-first` のスラッシュコマンドから起動できる (`.claude/commands/`)。スキル本体を読む必要があるときは Read で開く。
- **作業前の必読**: タスク着手前に本 `CLAUDE.md` と `.agents/AGENTS.md`、および関連する `.agents/skills/*/SKILL.md` を Read で確認する (AGENTS.md の Pre-work Skill Checks)。
- **ブランチ戦略**: `main` への直接 push は禁止。`feat/*` `fix/*` `docs/*` `refactor/*` のトピックブランチで作業し、CI (Test ワークフロー) が緑になってから PR 経由でマージする。
- **コミット規約**: [.agents/skills/core_commit_standard/SKILL.md](.agents/skills/core_commit_standard/SKILL.md) に従う (Conventional Commits、subject/body は日本語)。

## ツールの対応関係 (旧プロバイダ → Claude Code)

過去のドキュメントに他社エージェント (Antigravity / Gemini / Jules) のツール名が残っている場合は、以下に読み替える:

| 旧表記 | Claude Code での実体 |
|---|---|
| `view_file` / ファイル閲覧 | `Read` (PNG/JPG/PDF も描画可) |
| `grep_search` / コード検索 | `Grep` / `Glob` |
| `notify_user` / ユーザー確認 | 平文でユーザーに尋ねる、または `AskUserQuestion` |
| サブエージェント委譲 | `Task`/`Agent` ツール (並列可)、大規模な多段オーケストレーションは `Workflow` |
| `/gemini review` (PR ボット) | `/code-review` スラッシュコマンド、または GitHub PR レビュー |
| AI スクラッチ領域 | ハーネス提供の session-local scratchpad / 一時ディレクトリ |
