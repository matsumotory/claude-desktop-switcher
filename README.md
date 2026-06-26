# claude-context-switcher

Claude DesktopとClaude Code (CLI)の認証コンテキストを同期管理するmacOSメニューバーアプリケーション。

## 解決する課題

同一マシンで複数のClaudeアカウント環境（個人用/組織用等）を使い分ける際、DesktopとCLIの認証コンテキストが独立しているため、意図しないアカウントのトークンリソースを消費してしまう事故が発生する。本ツールはワンアクションで両者のコンテキストを確実に同期切り替えし、この事故を防止する。

## 設計原則

- **非破壊**: 既存の `~/.claude/` や `~/Library/Application Support/Claude/` には一切変更を加えない
- **逆方向リダイレクション**: 新環境側から既存ファイルへsymlinkを生成して共有
- **連動隔離**: Desktop (`--user-data-dir`) と CLI (`CLAUDE_CONFIG_DIR`) を必ずペアで切り替え
- **選択的共有**: 設定やメモリはコンポーネント単位で共有/隔離をオプトイン

## 技術スタック

- **コア**: Rust (クロスプラットフォーム対応)
- **UI**: Tauri v2 (macOSメニューバーアプリ) — Phase 2で実装
- **CLI**: `csw` コマンド (clap)

## プロジェクト構成

```
crates/
  core/    # コアライブラリ (PAL, Profile Manager, Switcher)
  cli/     # CLIインターフェース
scripts/
  sandbox-test.sh  # Phase 0 検証スクリプト
```

## 開発フェーズ

| Phase | 内容 | 状態 |
|-------|------|------|
| 0 | サンドボックス検証 | 🔄 |
| 1a | PAL + Profile Manager | 📋 |
| 1b | Context Switcher + Keychain | 📋 |
| 1c | File Watcher + Lock | 📋 |
| 2 | Tauri v2 メニューバーアプリ | 📋 |

## Quick Start

```bash
# ビルド
cargo build --workspace

# Phase 0 検証
chmod +x scripts/sandbox-test.sh
./scripts/sandbox-test.sh

# CLI (Phase 1)
cargo run -p csw-cli -- --help
```

## License

MIT
