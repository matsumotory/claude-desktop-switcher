# Claude Desktop Switcher 詳細仕様書

本仕様書は、Claude Desktop Switcher (以下 CSW) の2026年最新アーキテクチャに基づく公式リファレンスです。ソースコード（`crates/core`, `crates/cli`, `crates/desktop`）の実際の実装に完全に準拠しています。

## 1. 開発の動機と市場の課題 (Background & Motivation)

### 1.1. 公式デスクトップアプリの限界
Claudeデスクトップアプリは、ネイティブで複数アカウントの切り替え（マルチアカウント機能）をサポートしていません（関連するGitHubの機能要望: Issue #18435, #60607等）。
これに対し、コミュニティのパワーユーザーはターミナルから `--user-data-dir` フラグを用いて強引に別インスタンスを起動するか、自作のAppleScript等（例: `claude-clone.sh` 等）でワークアラウンドを行っていました。しかし、これらはmacOSのKeychain連携などを安全に分離できず、運用が煩雑でした。

### 1.2. 既存のCLIプロファイル管理の限界
一方でCLI（`Claude Code`）側のプロファイル管理は、`direnv` や環境変数 `CLAUDE_CONFIG_DIR` のエイリアス切り替え、または `claude-swap` などの「CLI専用スイッチャー」によって既に解決されています。しかし、これらは**デスクトップアプリの深い依存関係（Application SupportやKeychain）を一切管理できません**。

### 1.3. 本アプリケーションの真の価値
「CLI環境の切り替え」は既存のツール（`direnv` / `CLAUDE_CONFIG_DIR` のエイリアス / `claude-swap` 等）で既に解決されています。本アプリ（CSW）の主眼は、**Claudeデスクトップアプリというスイート全体 — チャット・Projects・Claude Cowork・Artifacts・Claude Design（Claude Code とも連携）— を、アカウント／用途ごとに専用データディレクトリで安全に分けて使い分けられるようにする（必要なら Claude Code（CLI）も同じ環境に連動）**ことにあります。デスクトップアプリはネイティブの複数アカウント切替を備えていないため、CSW がこの恒常的な切替を担います。

各プロファイルは自分のディレクトリ内にログインを保持するため、OSキーチェーンから秘密を取り出すことなく分離が成立します。GUI 側の環境分離に CLI のコンテキスト（`CLAUDE_CONFIG_DIR`）を連動させ、特定の設定ファイル（`CLAUDE.md` 等）だけを柔軟に共有（Isolate / Share / Copy）できる構造を提供します。ターミナルを使わない利用者でもメニューバーから GUI スイートを分けて使え、開発者は CLI 連携まで一貫させられます。

## 2. システムアーキテクチャ
本アプリはRustのワークスペースで構成され、3つのクレートに分割されています。

- **`csw-core`**: ビジネスロジックを担うライブラリ。OSパス抽象化、プロファイルとシンボリックリンクの管理。
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
        ├── desktop-data/     # デスクトップ用: ~/Library/Application Support/Claude の代わり (--user-data-dir)
        └── cli-data/         # CLI用: ~/.claude の代わり (CLAUDE_CONFIG_DIR)
```

### 隔離モード (SharingMode)
`SharingConfig` によってファイル単位で以下のモードを選択可能（デフォルトは `Isolate`）。
- **Isolate**: 完全に新規ファイルとして運用。
- **Share**: 元の環境（デフォルトは `default` プロファイル）のファイルを**シンボリックリンク**として参照。
- **Copy**: 作成時に元の環境から一度だけコピーする（以後は独立して乖離する）。

共有・隔離の対象（`crates/core/src/profile/mod.rs` の `SharingConfig` と同順）:
- `desktop_config`: `claude_desktop_config.json` (MCPサーバー構成)
- `cli_settings`: `settings.json` (権限・テーマ設定)
- `cli_claude_md`: `CLAUDE.md` (グローバルルール)
- `cli_project_memory`: プロジェクト記憶 (`projects/` ディレクトリ)
- `cli_plugins`: CLIプラグインディレクトリ (`plugins/`)
- `cli_skills`: CLIスキル定義 (`skills/`)
- `cli_sessions`: CLI会話セッション (`sessions/`)
- `cli_history`: CLIコマンド履歴 (`history.jsonl`)
- `desktop_app_config`: デスクトップアプリ設定 (`config.json`)
- `desktop_worktrees`: Gitワークツリー構成 (`git-worktrees.json`)
- `desktop_device_id`: 端末固有ID (`ant-did`)（これのみデフォルトで Share 設定）

## 4. セキュリティと認証の分離 (Security & Isolation)

CSW は **キーチェーンを一切読み書きしません**。アカウント／セッションの分離は、プロファイルごとに独立した**ディレクトリ**を割り当てることだけで成立します。

- **デスクトップ (Claude.app)**: `--user-data-dir=<profile>/desktop-data` を付けて起動する。ログインセッション（Cookie 等）は各データディレクトリ内に保存され、`Claude Safe Storage` キーチェーン項目（全プロファイル共通の暗号鍵）で暗号化される。データディレクトリを分ければアカウントが分離される。
- **CLI (Claude Code)**: `CLAUDE_CONFIG_DIR=<profile>/cli-data` を設定して実行する。**Claude Code の認証は config ディレクトリ単位でネイティブに分離される**（実機検証: 新規 `CLAUDE_CONFIG_DIR` は `claude auth status` で `loggedIn: false` となり、既定プロファイルのログインを引き継がない）。

各プロファイルは**初回に一度ログインすれば、そのログインが自身のディレクトリ内に保持され、切り替えで失われない**（再ログイン不要）。

### 設計判断: キーチェーン退避を行わない理由
旧実装は、切り替え時にキーチェーンの資格情報をファイル (`keychain_backup.json`) へ退避・削除・復元していたが、これは以下の理由で**撤去**した。
- 秘密情報を OS キーチェーン（暗号化・ACL 保護）から**平文ファイルとしてディスクに持ち出す**ことになり、Time Machine・ホーム同期・同一ユーザー権限のプロセスから読まれうる。利便性（再ログイン回避）のためにセキュリティを下げる設計であり許容しない。
- そもそもディレクトリ単位の隔離で分離が成立するため、キーチェーン操作は不要。
- キーチェーン項目の削除は、共通暗号鍵を消して全プロファイルの暗号化データを復号不能にする恐れがあり危険。

**スイッチング・フロー (`switcher.switch_to`)**:
1. 切り替え先プロファイルが存在することを検証する。
2. `config.toml` の `active_profile` を更新する。
3. （GUI/CLI の起動処理が）対象プロファイルの `--user-data-dir` / `CLAUDE_CONFIG_DIR` を指定して Claude を起動する。

キーチェーンへのアクセス・退避・削除・復元は一切行わない。

## 5. インターフェース仕様

### A. Tauri GUI (Desktop)
- システムトレイに常駐。アクティブなプロファイルは `●`、その他は `○` で表示。
- **Auto-launch**: トレイからプロファイルを切り替えた際、デフォルトプロファイル以外であれば、即座に該当の `user-data-dir` を指定して Claude Desktop を自動起動します。
- `settings` メニューからプロファイルの操作（Tauri commands: `create_profile`, `clone_profile`（複製）, `switch_profile`, `delete_profile`, `get_profile_details`, `list_profiles`, `get_active_profile`, `get_desktop_running_status`）を提供するWebUIを表示可能。
- 初回起動時には 3 ステップのオンボーディング（ようこそ／使い方／外部ターミナル連携）が表示される（`localStorage` の `csw_onboarded` で初回判定）。

### B. CLI コマンド (`csw`)
- `csw init`: ベース設定の初期化
- `csw profile create <name> [--mode share]`: プロファイル作成
- `csw profile list | show <name> | delete <name>`: プロファイル管理
- `csw switch <name> [--no-launch]`: プロファイルの切り替え（Tauriと同等の処理を実行）
- `csw env`: 現在のアクティブプロファイルに対応する環境変数スクリプト（`export CLAUDE_CONFIG_DIR=...`）を出力。
  - **内蔵ターミナル**: `launch_claude_desktop` は `open -n --env CLAUDE_CONFIG_DIR=<profile>/cli-data --args --user-data-dir=<profile>/desktop-data` で起動するため、CSW が起動した Claudeデスクトップアプリの子プロセス（アプリ内蔵ターミナル）は `CLAUDE_CONFIG_DIR` を継承する。`csw env` の実行は不要。
  - **外部ターミナル**: ユーザーが新規に開いた iTerm2 等は継承しないため、`eval $(csw env <name>)` を実行して対象セッションの `CLAUDE_CONFIG_DIR` を上書きする。
- `csw status`: 実行中プロセスのPIDと現在のコンテキスト状態を表示。

## 6. ゼロインパクト保証 (Non-invasive Guarantee)
本アプリは、OS のグローバル環境変数（`.zshrc` 等）を直接書き換えません。
GUI、あるいはターミナル上で手動の環境変数評価を行わずに `claude` (CLI) や `Claude.app` (Spotlight経由) を起動した場合、完全に標準の（隔離されていない）デフォルト状態として動作します。既存の環境を破壊するリスクはゼロに設計されています。
