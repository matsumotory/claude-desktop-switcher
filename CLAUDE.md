# CLAUDE.md

このファイルは Claude Code がセッション開始時に自動ロードする、本リポジトリの **一次正典** です。プロジェクトの規約・運用ルールはすべて本ファイルに集約します（旧 `.agents/AGENTS.md` の内容を取り込み済み。AGENTS.md は本ファイルへの薄いポインタに縮約しています）。横断ルールはマシン全体のグローバル `~/.claude/CLAUDE.md` を、デザイン等の具体手順は `.agents/skills/<name>/SKILL.md` を一次情報とします。

## 0. Skill-First Gate（最優先・例外なし）

デザイン/視覚やドメイン作業に着手する前に、**同じターン内・編集の前に**、(1) 該当する `.agents/skills/<name>/SKILL.md` を `Read` し、(2)「どのスキルのどのルールをこの変更に当てるか」を 1 行で明示してから編集する。

- **対象**: アプリ UI・CSS・レイアウト・余白・配色・タイポグラフィ・アイコン・コピー/マイクロコピー・LP・スクリーンショットなど、デザイン/視覚に関わるすべて。加えて各ドメイン作業（Rust 実装・Tauri・バグ修正・仕様変更・コミット・PR レビュー・整合性監査・日本語タイポ等）は下の Skill トリガーテーブルに従う。
- **「小さい/自明だから」は免除理由にならない**。フッターの寄せ・スウォッチの形・選択ハイライト・左バー——まさにこの「小さい変更」を雰囲気でやって繰り返し外している。サイズや明白さに関係なくゲートを通す。
- **自己捕捉**: スキルを引かずに着手してしまったと気づいたら、即停止・違反を自己申告し、スキルを Read して引用したルールでやり直す。ユーザーに指摘される前に自分で直す。
- **選び方**: 日本語の UI/LP/見出しは `japanese-typography-qa`、フロント/LP デザインは `design-taste-frontend` / `high-end-visual-design` / `minimalist-ui` 等。用途別の選び方はグローバル `~/.claude/CLAUDE.md` と各スキルの説明を参照。
- **強制の仕組み（意志力に頼らない）**: `.claude/settings.json` の PreToolUse フック `.claude/hooks/skill-first-reminder.sh` が、`website/`・`docs/`・`README`・`crates/desktop/ui/`・`*.html/*.css/*.md` を Edit/Write する直前に、本ゲート（該当スキルを読む）と全サーフェス伝播（`propagate-changes-to-all-surfaces`）・変更後の `/audit-consistency`・スクショ再生成（`scripts/appshot`）を自動リマインドする。フックの注意が出たら従う。指摘される前に自分でゲートを通す。
- この Gate はグローバル `~/.claude/CLAUDE.md` を一次正典とし、本ファイルで二重化する。

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

GUI の実機確認は `cargo tauri dev` (要 `cargo install tauri-cli`)。デスクトップ署名/公証/DMG は GitHub Actions の `Release Please` ワークフローが担う。リリースの実施手順 (release-please の release PR を `--admin` を使わずマージし、署名・公証つき DMG と `csw` の公開まで監視する正規手順) は [.agents/skills/core_pr_merge_checklist/SKILL.md](.agents/skills/core_pr_merge_checklist/SKILL.md) の「リリース PR (release-please) のマージ」節を参照する。ユーザーの明示指示があってから行う。

LP (`website/`) のプレビューは Claude の launch 機能で `.claude/launch.json` の `lp` を起動する (preview_start で `website/` を `python3 -m http.server` 配信、EN は `/`、日本語は `/ja/`)。`website/` をルートに配信しないと相対パス (`../style.css` 等) が解決しないので、単体 HTML を file:// で開くのではなくこの設定で見る。

## Skill トリガーテーブル

以下の条件に合致したら、該当する `.agents/skills/<name>/SKILL.md` を `Read` で読み、その手順に従う。`/bugfix` `/spec-first` `/audit-consistency` はスラッシュコマンドからも起動できる (`.claude/commands/`)。

| 発動条件 | スキル |
|---|---|
| 派生サブタスク (spawn_task / Agent 委譲 / 別 PR 化 / CI 待ち中の並行作業 / 割り込み) を始めるとき | `core_worktree_for_derived_tasks` |
| 新規実装・大幅修正の着手前、セッション開始/終了時 (Plan を `docs/proposals/` でバトンパス) | `core_session_handoff` |
| 日本語 UI / LP / ドキュメントのコピー・フォントサイズ・折り返し・行間・見出しスケールに触れるとき | `japanese-typography-qa` |
| 「完了/PASS」と報告する前・PR 提出前・リファクタ/名称変更後 (検証順序とエビデンス強制) | `core_qa_process` |
| PR をマージする / マージ可否を判断するとき | `core_pr_merge_checklist` |
| CI (GitHub Actions) が 10 分以上ハングしている疑いのとき | `core_ci_hang_recovery` |
| 複数 crate / 複数ファイルにまたがる作業を役割分担で進めるとき | `core_agent_roles` |
| 多段の調査・実装・検証を並列化したいとき (Workflow / サブエージェント) | `core_ai_workflow` |
| バグ修正に着手するとき (RED テストファースト) | `core_bug_fix_protocol` (`/bugfix`) |
| 新機能・仕様変更に着手するとき (仕様合意→RED→GREEN) | `core_spec_first_development` (`/spec-first`) |
| git commit する直前 | `core_commit_standard` |
| PR 完成時のレビューサイクル | `core_pr_review_cycle` |
| リリース前、または LP / docs / 実装を変えた後の整合性点検 (用語・アーキ・CLI 表面・機能主張・ja-en・トーン) | `docs_impl_consistency_audit` (`/audit-consistency`) |

## Claude Code セッションでの運用

- **このリポジトリのスキルは `.agents/skills/<name>/SKILL.md` に置かれている。** バグ修正は `/bugfix`、仕様ファースト開発は `/spec-first`、整合性監査は `/audit-consistency` のスラッシュコマンドから起動できる (`.claude/commands/`)。スキル本体を読む必要があるときは Read で開く。
- **作業前の必読 (Pre-work Skill Checks)**: タスク着手前に本 `CLAUDE.md` と、関連する `.agents/skills/*/SKILL.md` を Read で確認する。これを怠って編集・コマンド実行に進むのは重大な失敗。
- **ブランチ戦略**: `main` への直接 push は禁止。`feat/*` `fix/*` `docs/*` `refactor/*` のトピックブランチで作業し、CI (Test ワークフロー) が緑になってから PR 経由でマージする。
- **コミット規約**: [.agents/skills/core_commit_standard/SKILL.md](.agents/skills/core_commit_standard/SKILL.md) に従う (Conventional Commits、subject/body は日本語)。

## コーディング規約・制約 (Coding Standards & Constraints)

1. **Multi-language Sync**: `website/ja/index.html` に構造・レイアウト変更を加えたら、必ず同時に `website/index.html`（およびその逆）へ同じ変更を当てる。ローカライズ版を構造的に乖離させない。
2. **Repository Architecture（関心の構造的分離）**:
   - `website/` または `public/`: 公開向け Web ファイル専用。
   - `docs/`: 人間が読む Markdown ドキュメント専用 (例: `USER_GUIDE.md`)。
   - `.agents/`: AI エージェントの設定・スキル・タスク・仕様専用。
   - 公開 Web アセット (LP) と内部ドキュメント・エージェントファイルを決して混同しない。
3. **Asset Generation**: 画像内テキストの翻訳に生成 AI を使い、元の視覚デザインを変えてしまうことを禁止。ピクセル等価の翻訳が要るときは正確なプログラム処理 (例: Python + Pillow) を使う。
4. **CI/CD Feedback Loops**: commit 後は必ず GitHub Actions を監視する。ワークフローが落ちたら緑になるまでデバッグして push する。成功を仮定しない。

## 5. フィードバックの教訓化 (Feedback Memory Protocol / Self-Correction Mandate)

ユーザーが誤りを指摘・批判・明示フィードバック（例:「この形容詞を使うな」「先にちゃんと調べろ」）したら、目の前のコードを直すだけで終わらせない。**直す前に「これは汎用化できる教訓か」を判断し**、教訓があれば本 `CLAUDE.md`（または適切な `.agents/skills/*.md`）を更新して恒久的・体系的ルールに焼く。マシン横断の教訓はグローバル `~/.claude/CLAUDE.md` とプロジェクトメモリへ。

- **Tone & Copywriting**: 主観的・誇張的な形容詞 ("perfect", "seamless", "smart") を使わない。仕組みを客観的に記述する。ユーザー可視テキスト・ドキュメント・ユーザーへの応答に、装飾やステータス目的の絵文字（チェックマーク・旗・🟢/🎨/🇺🇸/🇯🇵 等のアイコン）を、明示要求がない限り **絶対に** 使わない。プロフェッショナルで清潔なコミュニケーションを保つ。
- **Survey-First**: プロダクトのポジショニングを再定義する前に、必ず Web/公開ソフトウェアの徹底サーベイを行う。
- **Pre-work Skill Checks**: 提案・実装・実行の前に、本 `CLAUDE.md` と関連スキルファイルを `Read`（場所特定に `Grep`/`Glob`）で必ず確認する。確認せず編集・実行に進むのは重大な失敗。
- **Branch Strategy Enforcement**: いかなる場合も `main` へ直接 push しない。軽微なドキュメント・テスト・ホットフィックスでも例外なし。専用トピックブランチ (`feat/*`, `fix/*`, `docs/*`) で作業し、CI が全て通ってから PR でのみマージする。
- **Anti-Slop Design**: `high-end-visual-design` / `design-taste-frontend` スキルを厳守。デフォルトの「AI テンプレ」見た目は禁止。
- **Anti-Slop Images**: 雑な「AI フローチャート」、無意味な文字の偽ダッシュボード、光るオーブ図を生成しない。フィーチャーグラフィックは超ミニマル・構造的に正確・角丸の不揃いなし。図が要るなら、生成フローチャート風スロップより、シンプルな抽象幾何やクリーンな UI クロップを優先。
- **Anti-Slop Typography (Japanese)**: 注記・免責・補足に `※`（米印）や `*`（アスタリスク）を使わない。これは旧来の企業 Web のアンチパターンで、プレミアム感を即座に損なう。従属情報は視覚階層（フォントサイズ・色・不透明度・レイアウト）で表す。
- **User-Centric Copywriting**: 生の技術用語（例: 'Application Support', 'Keychain'）を説明なしにマーケコピーへ出さない。技術機構を明確なユーザー利益に翻訳する（例: 'チャット履歴とログインを分離'）。
- **External Documentation Links**: LP の「ドキュメントを読む」CTA は常に外部の GitHub リポ/docs（例: `https://github.com/matsumotory/claude-desktop-switcher`）を指す。内部アンカー（`#guide`）を使わない。
- **Respect for Software Ecosystem**: 既存の OSS/CLI ツールに対し、貶めたり「不可能」と断じる表現を使わない。差異は事実に基づき、敬意を持って、加点的に述べる。
- **Japanese Localization QA**: タイポグラフィ（line-height・font-size・word-break 等）は `lang="ja"` 専用 CSS を使う。英語最適化のスタイルは日本語グリッドを壊す。ヘッドレスブラウザ（Playwright / CDP）でレイアウトを視覚検証する。詳細は `japanese-typography-qa` スキル。

## 6. コピーのアーキテクチャ的・論理的整合 (Architectural & Logical Consistency, Project Specific)

1. **Accurate Architecture Representation**: CSW は **2 つの別ツール**——`Claude Desktop App`（GUI）と `Claude Code`（CLI）——を管理する。一方が他方の機能であるかのように書かない（例:「デスクトップアプリに組み込まれた CLI」）。両者はバックエンドで統一プロファイルを共有する独立した存在。
2. **Logical Marketing Workflows**: 矛盾するワークフローを書かない。あるツールが端末コマンド（CLI 連携の `eval $(csw env)` 等）を必要とするなら、「設定不要・メニューバーだけで完結」のような包括的主張をしてはならない。マーケコピーはインターフェースの実際の範囲を反映する（例:「Desktop のプロファイルはメニューバーから管理、CLI のプロファイルは端末コマンドで連携」）。
3. **Global Terminology Enforcement**: 主要な用語を変えるとき（例:「コンテキスト」→「プロファイル」）、全ローカライズファイルを横断する正規表現検索で 100% の一貫性を担保する。場当たり的な部分修正は厳禁。

## 7. 運用上の絶対ルール (Operational Mandates, Zero-Tolerance)

1. **Mandatory Skill & Rule Checks**: 解決策を提案・実行する前に、本 `CLAUDE.md` と関連スキルファイル（例 `.agents/skills/design-taste-frontend/SKILL.md`）を必ず Read する。ルール無視は重大な失敗。
2. **No Unnecessary Confirmations**: ユーザーが明確なルール違反・欠陥を指摘したら、修正許可を求めない（「直していいですか？」と聞かない）。即座に修正する。不要な確認はルール未内面化の証拠とみなす。
3. **Self-QA Mandate**: QA 負担をユーザーに押し付けない。変更後は自分でコードをレビューし、テストを走らせ、ツール（Playwright / CDP 等）で修正を検証する。「直したので確認してください」だけの丸投げは厳禁。
4. **Git History Hygiene & Sensitive File Removal**: 内部・管理・機微ファイル（CI セットアップ手引き、証明書等）を公開リポに誤コミットしたら、単なる `git rm` で済ませない。浅い削除では公開コミット履歴に永久に残る。ローカルで履歴を書き換え（例 `git reset --soft`）て force-push する。リモートが保護され `push -f` を弾く場合は、解決したふりをせず、履歴汚染をリポオーナーに明示的に伝える。
5. **Premature Cleanup Prohibition (Key Management)**: デプロイ用シークレット・秘密鍵・証明書の生成を自動化するとき、下流システム（GitHub Actions CI/CD 等）が消費・検証し終えるまでローカル元ファイルを削除（`rm -rf`）しない。早すぎる削除は、上流エラー時に手動生成のやり直しを強いる。CI が GREEN を返してから一時鍵をローカル削除する。
6. **No AI Artifacts in Git**: 引き継ぎメモ・内部 AI トラッキング文書・AI が生成した一時スクラッチを、プロジェクトの Git リポに **絶対に** コミットしない。ハーネス提供の session-local scratchpad / 一時ディレクトリに留め、リポツリー内には置かない。

## 8. 徹底した正直さと秘密情報の適正管理 (Radical Honesty & Proper Secret Management, Absolute Rule)

1. **No Cover-ups or Excuses**: 致命的なミス（シークレット誤設定、ユーザーの秘密鍵削除、誤フラグ適用等）をしたら、隠したり矮小化したり、認めずに回避策へ飛びつかない。何を間違えたかを正確に透明に説明し、アーキテクチャ的に正しい解決法を示す。エラー隠蔽は信頼を破壊する。
2. **Strict Variable Mapping for Secrets**: シークレット（例 `.p8` ファイル）を意味的に誤った環境変数（`APPLE_API_KEY_CONTENT` でなく `APPLE_PASSWORD` 等）にマップしない。ダミー変数や誤フラグでエラーを握り潰すのは厳禁。ツールの公式ドキュメント（例 Tauri の `APPLE_API_ISSUER` / `APPLE_API_KEY` / `APPLE_API_KEY_PATH`）を調べ、正確で意図どおりのワークフローを実装する。
3. **No Guessing Versions or Dependencies**: 依存・GitHub Actions（例 Tauri v2 だからと `tauri-action@v2` を推測する等）・ライブラリのバージョンを、公式リリースタグ/ドキュメントで確認せずに推測・仮定しない。バージョンの当て推量は即 CI 失敗を招く。マニフェスト（`Cargo.toml` / `Cargo.lock`）・リリースノート・公式ドキュメントを `Read` で確認してから上げる。
4. **Record Mistakes into Memory**: 知識不足や誤った前提で失敗したら、正しい手順を直ちに本 `CLAUDE.md`（または該当スキル・プロジェクトメモリ）に記録し、二度と繰り返さないようにする。

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
