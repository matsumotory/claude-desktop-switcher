# claude-context-switcher

Claude Desktop と Claude Code (CLI) の認証コンテキストを同期管理する macOS 常駐メニューバーアプリケーション、および CLI ツール。

## 解決する課題

同一マシンで複数の Claude アカウント環境（個人用 / 業務プロジェクト用など）を使い分ける際、Desktop と CLI の認証コンテキストが混在し、意図しないアカウントのトークンリソースやバジェットを消費してしまう事故が発生します。

本ツールは、ワンクリック（GUI）またはワンコマンド（CLI）で、両者のコンテキスト（OAuthトークン、セッション、設定、メモリ）を確実にペアで切り替え、アカウント混在による事故を完全に防止します。

## 設計原則

- **既存環境の非破壊**: 既存の `~/.claude/` や `~/Library/Application Support/Claude/` には一切変更を加えません。
- **逆方向リダイレクション**: プロファイルごとの隔離ディレクトリ（`desktop-data/`, `cli-data/`）を生成し、共有するコンポーネント（MCP設定、CLAUDE.md、`projects/` メモリ等）のみを symlink で本番実体に繋ぐことで、設定の共有とデータ隔離を両立します。
- **Keychain の自動退避・復元**: サービス名が固定の `Claude Safe Storage` と `Claude Code-credentials` に対し、切り替え時に認証情報を自動で退避・復元し、アカウントのクロス競合を防止します。
- **選択的共有**: 設定やメモリなどのコンポーネント単位で、共有 (`Share`) / コピー (`Copy`) / 隔離 (`Isolate`) をプロファイルごとに指定可能です。
- **双方向リアルタイム同期**: `Copy` モードで共有されたファイルは、`notify` によるバックグラウンド監視によって、変更時に他のプロファイルへリアルタイム同期されます。

## 技術スタック

- **コアロジック**: Rust (`csw-core` - Keychain制御、PAL、リンク構築、ファイル監視)
- **GUI アプリ**: Tauri v2 (`csw-desktop` - macOS常駐メニューバーアプリ ＋ リッチ設定UI)
- **CLI ツール**: Rust (`csw-cli` - ターミナル切替ツール)

## プロジェクト構成

```
crates/
  core/      # コアライブラリ (PAL, Profile Manager, Switcher, Watcher)
  cli/       # CLIツール (csw コマンド)
  desktop/   # Tauri v2 メニューバーアプリ & 美麗設定UI
```

## 使用方法

### GUI アプリ (メニューバー)
1. `csw-desktop` を起動すると、macOS のメニューバーに常駐します。
2. トレイメニューからプロファイル（`default`, `Work`, `Personal` 等）を選択すると、自動で Keychain 復元 ＋ Desktop 起動 ＋ CLI用のプロファイル切替が行われます。
3. `Settings...` を開くことで、ガラスモフィズムを施した美麗な設定画面からプロファイルの新規作成、削除、共有モードの設定が可能です。

### CLI ツール
```bash
# ContextSwitcherの初期化
csw init

# プロファイル一覧の表示
csw profile list

# 新しいプロファイルの作成
csw profile create MyWorkProfile --mode share   # default設定を引き継ぐ
csw profile create MySecretProfile --mode isolate # 完全に隔離する

# プロファイルの切り替え
csw switch MyWorkProfile

# ターミナルセッションの環境変数切り替え
eval $(csw env MyWorkProfile)
```

### ビルド方法
```bash
# ワークスペース全体のビルド
cargo build --workspace
```

## License

MIT

