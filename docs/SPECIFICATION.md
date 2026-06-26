# Claude Desktop Switcher 詳細仕様書

本仕様書は、Claude Desktop Switcher (以下 CSW) の2026年最新アーキテクチャに基づく公式リファレンスです。ソースコード（`crates/core`, `crates/cli`, `crates/desktop`）の実際の実装に完全に準拠しています。

## 1. 開発の動機と市場の課題 (Background & Motivation)

### 1.1. 公式デスクトップアプリの限界
Claudeデスクトップアプリは、ネイティブで複数アカウントの切り替え（マルチアカウント機能）をサポートしていません（関連するGitHubの機能要望: Issue #18435, #60607等）。
これに対し、コミュニティのパワーユーザーはターミナルから `--user-data-dir` フラグを用いて強引に別インスタンスを起動するか、自作のAppleScript等（例: `claude-clone.sh` 等）でワークアラウンドを行っていました。しかし、これらはmacOSのKeychain連携などを安全に分離できず、運用が煩雑でした。

### 1.2. 既存のCLIプロファイル管理の限界
一方でCLI（`Claude Code`）側のプロファイル管理は、`direnv` や環境変数 `CLAUDE_CONFIG_DIR` のエイリアス切り替え、または `claude-swap` などの「CLI専用スイッチャー」によって既に解決されています。しかし、これらは**デスクトップアプリの深い依存関係（Application SupportやKeychain）を一切管理できません**。

### 1.3. 本アプリケーションの真の価値
「CLI環境の切り替え」は既存のツールで十分です。本アプリ（CSW）の唯一無二の価値は、「**これまで分離が困難だったデスクトップアプリのプロファイル（ディレクトリとKeychain）を安全に隔離し、さらにCLIのコンテキスト（環境変数）をそれに完全に連動させる統合管理システム**」である点にあります。単なるCLIスイッチャーとは異なり、GUIとCLIの双方で一貫したプロファイル状態を保持しながら、特定の設定ファイル（`CLAUDE.md` 等）だけを柔軟に共有（Isolate/Share）できる構造を提供します。

## 2. システムアーキテクチャ
本アプリはRustのワークスペースで構成され、3つのクレートに分割されています。

- **`csw-core`**: ビジネスロジックを担うライブラリ。OSパス抽象化、キーチェーン操作、プロファイルとシンボリックリンクの管理。
- **`csw-cli` (`csw`)**: ターミナルから操作するためのコマンドラインインターフェース（Clap使用）。
- **`csw-desktop`**: Tauri v2 ベースの GUI アプリケーション。システムトレイに常駐し、直感的な切り替え機能を提供。

## 3. ディレクトリ構造とプロファイル構成
CSW自身の全てのデータは `~/.context-switcher-claude/` に保存されます。

### アプリケーションデータ
```text
~/.context-switcher-claude/
├── config.toml               # 現在のアクティブプロファイルを記録 (AppConfig)
└── profiles/
    └── <Profile_Name>/       # 各プロファイルの専用ディレクトリ
        ├── profile.toml      # メタデータ (名前, アイコン, SharingConfig等)
        ├── desktop-data/     # デスクトップ用: ~/Library/Application Support/Claude の代わり
        └── cli-data/         # CLI用: ~/.claude の代わり
            └── keychain_backup.json # (後述) 退避されたキーチェーン情報
```

### 隔離モード (SharingMode)
`SharingConfig` によってファイル単位で以下のモードを選択可能（デフォルトは `Isolate`）。
- **Isolate**: 完全に新規ファイルとして運用。
- **Share**: 元の環境（デフォルトは `default` プロファイル）のファイルを**シンボリックリンク**として参照。

共有（Share）可能な対象:
- `desktop_config`: `claude_desktop_config.json` (MCPサーバー構成)
- `cli_settings`: `settings.json` (権限・テーマ設定)
- `cli_claude_md`: `CLAUDE.md` (グローバルルール)
- `cli_project_memory`: プロジェクト記憶 (`projects/*/memory`)
- `cli_plugins`: CLIプラグインディレクトリ
- `desktop_worktrees`: Gitワークツリー構成 (`git-worktrees.json`)
- `desktop_device_id`: 端末固有ID (`ant-did`)（これのみデフォルトで Share 設定）

## 4. セキュリティとキーチェーン分離 (Security & Isolation)
CSWは、`ContextSwitcher` クラスにより、macOSネイティブキーチェーンの安全な分離を実現しています。

**退避対象のクレデンシャル**:
- デスクトップ用: Service: `Claude Safe Storage`, Account: `Claude Key`
- CLI用: Service: `Claude Code-credentials`, Account: `CloudFlare`

**スイッチング・フロー (`switcher.switch_to`)**:
1. 現在のプロファイルのキーチェーン情報を暗号化されたJSON (`cli-data/keychain_backup.json`) として退避。
2. キーチェーン上の既存エントリを安全に**削除**（次プロファイルへの流出防止）。
3. 遷移先のプロファイルの `keychain_backup.json` があればキーチェーンに復元（なければ空の状態で認証を要求）。
4. `config.toml` の `active_profile` を更新。

## 5. インターフェース仕様

### A. Tauri GUI (Desktop)
- システムトレイに常駐。アクティブなプロファイルは `●`、その他は `○` で表示。
- **Auto-launch**: トレイからプロファイルを切り替えた際、デフォルトプロファイル以外であれば、即座に該当の `user-data-dir` を指定して Claude Desktop を自動起動します。
- `settings` メニューからプロファイルのCRUD操作（Tauri commands: `create_profile`, `switch_profile`, `delete_profile`, `get_profile_details`）を提供するWebUIを表示可能。

### B. CLI コマンド (`csw`)
- `csw init`: ベース設定の初期化
- `csw profile create <name> [--mode share]`: プロファイル作成
- `csw profile list | show <name> | delete <name>`: プロファイル管理
- `csw switch <name> [--no-launch]`: プロファイルの切り替え（Tauriと同等の処理を実行）
- `csw env`: 現在のアクティブプロファイルに対応する環境変数スクリプトを出力。
  - **使い方**: ターミナルで `eval $(csw env <name>)` を実行することで、対象セッションの `CLAUDE_CONFIG_DIR` 等が上書きされ、Claude Codeが隔離環境で実行されます。
- `csw status`: 実行中プロセスのPIDと現在のコンテキスト状態を表示。

## 6. ゼロインパクト保証 (Non-invasive Guarantee)
本アプリは、OS のグローバル環境変数（`.zshrc` 等）を直接書き換えません。
GUI、あるいはターミナル上で手動の環境変数評価を行わずに `claude` (CLI) や `Claude.app` (Spotlight経由) を起動した場合、完全に標準の（隔離されていない）デフォルト状態として動作します。既存の環境を破壊するリスクはゼロに設計されています。
