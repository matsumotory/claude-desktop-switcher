# Claude Desktop Switcher 詳細仕様書

本仕様書は、Claude Desktop Switcher (以下 CSW) の2026年最新アーキテクチャに基づく公式リファレンスです。ソースコード（`crates/core`, `crates/cli`, `crates/desktop`）の実際の実装に完全に準拠しています。

## 1. 開発の動機と市場の課題 (Background & Motivation)

### 1.1. 公式デスクトップアプリの限界
Claudeデスクトップアプリは、ネイティブで複数アカウントの切り替え（マルチアカウント機能）をサポートしていません（関連するGitHubの機能要望: Issue #18435, #60607等）。
これに対し、コミュニティのパワーユーザーはターミナルから `--user-data-dir` フラグを用いて強引に別インスタンスを起動するか、自作のAppleScript等（例: `claude-clone.sh` 等）でワークアラウンドを行っていました。しかし、これらはデスクトップアプリのデータを恒常的に分けて運用するには煩雑でした。

### 1.2. 既存のCLIプロファイル管理の限界
一方でCLI（`Claude Code`）側のプロファイル管理は、`direnv` や環境変数 `CLAUDE_CONFIG_DIR` のエイリアス切り替え、または `claude-swap` などの「CLI専用スイッチャー」によって既に解決されています。しかし、これらは**デスクトップアプリのデータ（`~/Library/Application Support/Claude` 配下）を分離・管理する用途には対応していません**。

### 1.3. 本アプリケーション（CSW）の位置づけ
「CLI環境の切り替え」は既存のツール（`direnv` / `CLAUDE_CONFIG_DIR` のエイリアス / `claude-swap` 等）で既に解決されています。本アプリ（CSW）の主眼は、**Claudeデスクトップアプリというスイート全体（チャット・Projects・Claude Cowork・Artifacts・Claude Design、および Claude Code 連携）を、アカウント／用途ごとに専用データディレクトリで安全に分けて使い分けられるようにする（必要なら Claude Code（CLI）も同じ環境に連動）**ことにあります。デスクトップアプリはネイティブの複数アカウント切替を備えていないため、CSW がこの恒常的な切替を担います。

各環境は自分のディレクトリ内にログインを保持するため、認証情報ストアを操作することなく分離が成立します。GUI 側の環境分離に CLI 側の設定ディレクトリ（環境変数 `CLAUDE_CONFIG_DIR`）を連動させ、特定の設定ファイル（`CLAUDE.md` 等）だけを共有・分離・コピーで柔軟に切り替えられる構造を提供します。ターミナルを使わない利用者でも GUI だけでスイートを分けて使え、開発者は CLI 連携まで一貫させられます。

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
├── config.toml               # アクティブな環境を記録 (AppConfig.active_profile)
└── profiles/
    └── <Profile_Name>/       # 各環境の専用ディレクトリ
        ├── profile.toml      # メタデータ (名前, アイコン, SharingConfig等)
        ├── desktop-data/     # デスクトップ用: ~/Library/Application Support/Claude の代わり (--user-data-dir)
        └── cli-data/         # CLI用: ~/.claude の代わり (CLAUDE_CONFIG_DIR)
```

### 共有モード（用途ベースの3モード）

CSW が管理する複数の環境は、すべて**同じ利用者本人**のアカウントです。したがって何を共有するかは「他者への漏洩を防ぐ」ためではなく「自分の作業をどこまで引き継ぐか」という選択になります。その前提で、環境の作成時に用途ベースの3モードから選びます。

- **アカウントだけ分ける** (`share_workspace`): 会話履歴・自動メモリも引き継ぎ、分離するのはアカウントだけにする。研究と開発を別アカウントの決済にしつつ、作業の文脈を連続させたい場合向け。
- **会話とメモリも分ける** (`share_settings`): 共通ルール・スキル・プラグインとツール権限は引き継ぎ、会話履歴と自動メモリはこの環境だけにする。用途別に分けつつ設定を使い回したい場合向け。
- **すべて分ける** (`isolate`): 何も引き継がず、各環境を完全に独立させる。案件・クライアント別や、仕事用と個人用など、混ざってはいけない用途向け。

**アカウント**＝サインインしている Claude のアカウント。課金・利用量がここにひも付く。各環境はそれぞれのアカウントでサインインするため、サインイン情報（`config.json` の OAuth トークン）はどのモードでも環境ごとに分かれます。「アカウントだけ分ける」と「会話とメモリも分ける」の違いは「会話履歴と自動メモリを引き継ぐか」の一点です。加えて作成画面の詳細設定から、項目ごとに共有・分離・コピーを手動で上書きできます（カスタム）。ただし後述の「常に分離する項目」は安全のため上書きできません。

### 用語の定義（CSW の項目と Claude 本体の機能）

ユーザー向けラベル・モード名・説明は、Claude 本体（Desktop / Code）の現行の機能用語に合わせます。特に **メモリ・会話履歴・セッションは別物**です。

| CSW の項目（実体） | Claude 本体の機能 | 意味（1語1義） |
|---|---|---|
| 共通ルール (`CLAUDE.md`) | メモリ（人が書く指示） | あなたが書く常時ルール。Claude Code の `/memory` で編集する対象 |
| 自動メモリ (`projects/<project>/memory/`) | メモリ（Claude が書く学習） | 過去のやり取りから Claude が要約・抽出した持続メモ。会話の生ログではない |
| 会話履歴 (`projects/<project>/*.jsonl`) | 会話履歴 (conversation history) | 各プロジェクトの会話の本体。やり取りの記録そのもの |
| 入力履歴 (`history.jsonl`) | プロンプト履歴 | あなたが入力したプロンプトの一覧 |
| セッション状態 (`sessions/`) | セッション | 実行中セッションの軽い状態（pid・作業フォルダ・開始時刻など）。会話の中身ではない |
| ツール権限・フック (`settings.json`) | 設定（権限・フック・モデル） | ツールの実行可否（許可・拒否・確認）とフック。アカウント権限ではない |
| プラグイン (`plugins/`) / スキル (`skills/`) | プラグイン / スキル | 導入したプラグインとカスタムスキル |
| コネクタ・アプリ設定 (`claude_desktop_config.json`) | コネクタ（MCP）ほか | 外部接続の設定（コネクタは MCP の製品名）と信頼フォルダ等。アカウント別の権限ゲートを含む |
| アカウントのサインイン情報 (`config.json`) | アカウント認証 | OAuth トークンとアカウント別状態 |
| ワークツリー一覧 (`git-worktrees.json`) | Desktop の worktree 管理 | worktree 名 → repo/branch の対応 |
| 端末 ID (`ant-did`) | 端末識別子 | 端末を識別する ID |

要点: **メモリ ≠ 会話履歴**（メモリは要約・抽出された洞察、会話履歴は生のやり取りの記録）。**セッション ≠ 会話履歴**（セッションは pid 等の実行時状態で、会話の中身ではない）。

### ファイル単位のモード (SharingMode)

各ファイル／ディレクトリは以下のいずれかで扱います（`SharingConfig`、デフォルトは `Isolate`）。
- **Isolate**: 何も引き継がず、その環境専用に持つ。
- **Share**: 既存の Claude（内部識別子 `default`）のファイルを**シンボリックリンク**で参照する。アプリが読むだけで利用者が単独で編集するファイル（共通ルール・スキル・プラグイン）や、追記しかしないディレクトリ（会話履歴・自動メモリ）に限る。アプリが起動時に temp+rename で書き換えるファイルはリンクが壊れるため Share にしない。
- **Copy**: 作成時に一度だけコピーする（以後は独立して乖離する）。

各モードでの扱い（`crates/core/src/profile/mod.rs` の `SharingConfig::share_settings_preset` / `share_workspace_preset` と一致）:

| 項目 (ファイル) | アカウントだけ分ける | 会話とメモリも分ける | すべて分ける |
|---|---|---|---|
| `cli_claude_md` (共通ルール `CLAUDE.md`) | Share | Share | Isolate |
| `cli_plugins` (プラグイン `plugins/`) | Share | Share | Isolate |
| `cli_skills` (スキル `skills/`) | Share | Share | Isolate |
| `cli_settings` (ツール権限・フック `settings.json`) | Copy | Copy | Isolate |
| `desktop_worktrees` (ワークツリー一覧 `git-worktrees.json`) | Copy | Copy | Isolate |
| `cli_project_memory` (会話履歴＋自動メモリ `projects/`) | Share | Isolate | Isolate |
| `cli_history` (入力履歴 `history.jsonl`) | Share | Isolate | Isolate |

**常に分離する項目（モードや詳細設定でも上書き不可）**: 認証・アカウント状態を含むか、共有しても無意味か、安全に共有できないため、どのモードでも `Isolate` に固定します。
- `desktop_app_config` (`config.json`): OAuth トークンキャッシュ（`oauth:tokenCache`）とアカウント別状態を含む。共有すると2つのアカウントのログインが1ファイルに混ざる。
- `desktop_config` (`claude_desktop_config.json`): アカウント別の権限ゲート（`bypassPermissionsGateByAccount`）を含み、アプリ起動時に temp+rename で書き換えられてシンボリックリンクが壊れるため、共有も安全なコピーもできない（コネクタ＝MCP の構成もここに置かれるが、この理由により共有しない）。
- `cli_sessions` (`sessions/`): 実行中セッションの状態（pid・作業フォルダ・開始時刻）で、会話の中身ではない。環境ごとの実行時状態なので共有しても無意味なため、常に分離する。
- `desktop_device_id` (`ant-did`): 端末固有 ID。共有すると2つのアカウントが同一端末として相関するため分離する。
- アカウントのサインイン本体（デスクトップの Cookie、CLI の資格情報）は各環境のデータディレクトリ内にあり、linker は触れない。

## 4. セキュリティと認証の分離 (Security & Isolation)

CSW はアカウント／セッションの分離を、環境ごとに独立した**データディレクトリ**を割り当てることだけで成立させます。OS の認証情報ストア（パスワード/鍵）は読み書きしません。

- **デスクトップ (Claude.app)**: `--user-data-dir=<profile>/desktop-data` を付けて起動する。ログインセッション（Cookie 等）は各データディレクトリ内に保存されるため、データディレクトリを分ければアカウントが分離される。CSW はこの保存・暗号化の仕組みには関与せず、ディレクトリの割り当てだけを行う。
- **CLI (Claude Code)**: `CLAUDE_CONFIG_DIR=<profile>/cli-data` を設定して実行する。**Claude Code の認証は config ディレクトリ単位でネイティブに分離される**（実機検証: 新規 `CLAUDE_CONFIG_DIR` は `claude auth status` で `loggedIn: false` となり、既定プロファイルのログインを引き継がない）。

各プロファイルは**初回に一度ログインすれば、そのログインが自身のディレクトリ内に保持され、切り替えで失われない**（再ログイン不要）。

### 設計判断: 分離はディレクトリ隔離のみで行う
CSW はアカウント／セッションの分離を、環境ごとに独立したデータディレクトリと環境変数（`--user-data-dir` / `CLAUDE_CONFIG_DIR`）の割り当てだけで実現する。OS の認証情報ストアの読み書きや、資格情報のファイルへの退避・削除・復元は一切行わない。利便性のために秘密情報をディスクへ持ち出す設計は採用しない。

**スイッチング・フロー (`switcher.switch_to`)**:
1. 切り替え先の環境が存在することを検証する。
2. `config.toml` の `active_profile` を更新する。
3. （GUI/CLI の起動処理が）対象の `--user-data-dir` / `CLAUDE_CONFIG_DIR` を指定して Claude を起動する。

認証情報ストアへのアクセス・退避・削除・復元は一切行わない。

### 完全分離環境の同時起動（複数ウィンドウ）
「すべて分ける（`isolate`）」で作った環境は共有（symlink）を一切持たず（`linker` はディレクトリを作るだけで symlink を張らない）、各インスタンスは自分の `--user-data-dir` にのみ書き込むため、複数を同時に起動しても共有ファイルのレースが起きない。この安全性を保証できる完全分離の環境に限り、詳細画面に専用ボタン「重複して起動」を表示し、起動中の Claude を終了せずに並べて起動できる。判定は `SharingConfig::is_fully_isolated()`（共有もコピーも1つも含まず全項目が分離）で行い、共有やコピーを含む環境と既存の Claude（`default`）ではこのボタンを出さない（backend の `launch_additional_window` も同条件で拒否する）。

この同時起動は「追加のウィンドウを開く」操作であり、アクティブな環境（`active_profile`、CLI の `csw env` の既定対象）は変更しない。主ボタン「この環境で Claude を起動」と、切り替え時の「一度に1環境」制約（`switch_to` は Claude 起動中の切り替えを拒否し、共有を含む環境のレースを防ぐ）は従来どおり据え置く。

同時起動に対応するため、「利用中」の判定は「アクティブかつ Claude 起動中」ではなく、起動中の各 Claude プロセスの `--user-data-dir` から求める。CSW は起動中の Claude のうちヘルパープロセスを除く主プロセスの起動引数を読み、その `--user-data-dir` が一致する環境を「利用中」とする（`--user-data-dir` を持たない通常起動は既存の Claude とみなす）。これにより完全分離環境を並べて起動したときは複数の行が同時に「利用中」となり、どの環境が動いているかを正しく示す。

## 5. インターフェース仕様

### A. Tauri GUI (Desktop)
- システムトレイに常駐。いま Claude が起動している環境（＝利用中）を `● 既存の Claude (利用中)` のように `●` と `(利用中)` で示し、それ以外は `○ <名前>` で表示する（既定の環境は名称「既存の Claude」で先頭に並ぶ）。この印は実際の起動状態を反映し、CSW の外で Claude を起動・終了しても数秒で追随する。どの環境の Claude も起動していないときは、いずれの行にも `●`／`(利用中)` は付かない。完全分離環境を並べて起動しているときは複数の行が同時に利用中になる。
- **Auto-launch**: トレイから環境を切り替えた際、既存の Claude（内部識別子 `default`）以外であれば、即座に該当の `user-data-dir` を指定して Claudeデスクトップアプリを自動起動します。
- `settings` メニューから環境の操作（Tauri commands: `create_profile`, `clone_profile`（複製）, `switch_profile`, `delete_profile`, `launch_additional_window`（完全分離環境を追加のウィンドウで起動）, `get_profile_details`, `list_profiles`, `get_active_profile`, `get_desktop_running_status`, `get_running_profiles`（起動中の環境名の一覧）, `get_default_roots_status`, `app_version`, `open_url`）を提供するWebUIを表示可能。`open_url` は固定の https GitHub URL のみを既定ブラウザで開く（アプリ自体は通信しない）。
- 初回起動時のみ、ようこそ画面の下に補足カードが表示される（既存の Claude が保護されること／環境を切り替えて使えること（共有を含む環境は設定の衝突を防ぐため1環境ずつ、すべて分ける環境は並べて同時に開ける）／ターミナル（Claude Code）も同じ環境で使えること、の3点。`localStorage` の `csw_onboarded` で表示済みを判定）。
- アクセントカラーはサイドバー下部のスウォッチで4セット（ブルー既定／ティール／インディゴ／テラコッタ）から選択でき、`html[data-accent]` 属性で CSS 変数を切り替える。選択は `localStorage` の `csw_accent` に保存される。意味色（共有／分離／削除）は不変。
- サイドバー最下部に控えめなヘルプフッターを置く（ユーザーガイド／問題を報告＝GitHub Issues／このアプリについて＝免責ダイアログ／バージョン表示＋更新を確認＝GitHub Releases）。バージョンは `app_version`（tauri.conf.json）から取得し、外部リンクは `open_url` で既定ブラウザを開く。免責文は LP の「免責・利用について」と同文。

### B. CLI コマンド (`csw`)
> 配布についての注記: `csw`（CLI）は主にエンジニアが使う想定で `.dmg` には同梱しないが、署名・公証済みの単体バイナリを各リリースに添付する（`csw-cli.yml` が universal でビルド→署名→公証→Release アセットに添付。`workflow_dispatch` で既存タグへの後付けも可能）。利用者はそれをダウンロードして `PATH` に置く。Rust 環境があれば `cargo install --path crates/cli` でも導入できる。GUI からの環境切り替えだけなら不要。

- `csw init`: ベース設定の初期化
- `csw profile create <name> [--mode isolate|share_settings|share_workspace]`: 環境（プロファイル）の作成（既定は `isolate`＝すべて分ける。`share` は `share_settings` の後方互換エイリアス）
- `csw profile list | show <name> | delete <name>`: プロファイル管理
- `csw switch <name> [--no-launch]`: プロファイルの切り替え（Tauriと同等の処理を実行）
- `csw env [<環境名>]`: 指定した環境（省略時は現在のアクティブな環境）に対応する環境変数スクリプト（`export CLAUDE_CONFIG_DIR=...`）を出力。
  - **内蔵ターミナル**: `launch_claude_desktop` は `open -n --env CLAUDE_CONFIG_DIR=<profile>/cli-data --args --user-data-dir=<profile>/desktop-data` で起動するため、CSW が起動した Claudeデスクトップアプリの子プロセス（アプリ内蔵ターミナル）は `CLAUDE_CONFIG_DIR` を継承する。`csw env` の実行は不要。
  - **外部ターミナル**: ユーザーが新規に開いた iTerm2 等は継承しないため、`eval $(csw env <環境名>)` のように環境名を渡して対象セッションの `CLAUDE_CONFIG_DIR` を上書きする。
- `csw status`: 現在のアクティブな環境と、Claudeデスクトップアプリの起動状態（起動中ならその PID）を表示。

## 6. ゼロインパクト保証 (Non-invasive Guarantee)
本アプリは、OS のグローバル環境変数（`.zshrc` 等）を直接書き換えません。
GUI、あるいはターミナル上で手動の環境変数評価を行わずに `claude` (CLI) や `Claude.app` (Spotlight経由) を起動した場合、完全に標準の（隔離されていない）デフォルト状態として動作します。既存の環境を破壊するリスクはゼロに設計されています。
