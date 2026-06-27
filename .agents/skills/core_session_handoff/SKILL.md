---
name: core_session_handoff
description: セッション間の引き継ぎプロトコル。Implementation Plan を docs/proposals/ の git 管理ファイルとしてバトンパスに使い、SPECIFICATION.md / USER_GUIDE は完了状態の記録として使う。新規実装・大幅修正の着手前、またはセッション開始/終了時に発動する。
---

# セッション引き継ぎプロトコル

CSW (Claude Desktop Switcher / Rust + Tauri v2 macOS app) のセッション間引き継ぎ標準。Implementation Plan を git 管理のバトンパスとして使い、`docs/SPECIFICATION.md` / `docs/USER_GUIDE*.md` は完了状態の記録として使う。Plan のライフサイクル全体を本スキルに統合している。

## ドキュメントの役割分担

| ドキュメント | 目的 | タイミング | 配置 |
|---|---|---|---|
| **Implementation Plan** | セッション間のバトンパス + PR レビュー時の背景共有 | 着手前に作成 + ライフサイクルで更新 | `docs/proposals/<slug>.md`（git 管理、frontmatter の `status` で管理） |
| **`docs/SPECIFICATION.md`** | 仕様の現在地（完了状態の正典 / spec canon） | タスク**完了後**に更新 | `docs/` 直下 |
| **`docs/USER_GUIDE.md` / `USER_GUIDE_EN.md`** | ユーザー向けガイド（完了状態） | タスク**完了後**に更新 | `docs/` 直下 |

> Plan は「次に何をやるか（前方参照）」、SPECIFICATION / USER_GUIDE は「何が完成したか（後方の記録）」。役割を混同しない。`blueprint.md` は存在しない。仕様の正典は必ず `docs/SPECIFICATION.md`。

## Plan の配置と命名規約

- **ディレクトリ**: `docs/proposals/`（git 管理。worktree ローカルパスには**絶対に置かない** — worktree 削除で消失する）
- **ファイル名**: `<kebab-case-slug>.md`。中身が分かる英語短縮トピック名。日付はファイル名に付けず、frontmatter の `created` と git 履歴に持たせる
- **例**: `docs/proposals/cli-env-eval-profile-switch.md` / `docs/proposals/keychain-isolation-per-profile.md`

## Plan の frontmatter（必須）

各 plan ファイルの先頭に以下の YAML frontmatter を必ず記載する：

```yaml
---
title: タスクの短いタイトル（50字以内）
created: YYYY-MM-DD
status: draft | approved | in-progress | completed | rejected
pr: 42                     # PR 作成後に番号を追記（未確定時は省略可）
related_prs: []            # 関連 PR 番号（任意）
related_issues: []         # 関連 Issue 番号（任意）
---
```

### status の遷移

| status | 意味 |
|---|---|
| `draft` | 作成中・ユーザ Approve 未取得 |
| `approved` | ユーザが Approve 済み、実装着手可能 |
| `in-progress` | PR 作成済み、実装中 |
| `completed` | PR マージ済み、タスク完了 |
| `rejected` | PR Close または plan 自体が見送りになった |

遷移は `draft → approved → in-progress → completed`（または途中で `rejected`）。

## Plan のライフサイクル

| ステージ | アクション | commit |
|---|---|---|
| 1. ドラフト作成 | `docs/proposals/<slug>.md` を作成、`status: draft` | ✅ する |
| 2. ユーザ Approve | `status: approved` に更新 | ✅ する |
| 3. PR 作成 | PR 本文に plan へのリンクを貼り、`status: in-progress` に更新 | ✅ する |
| 4. 実装中の更新 | 必要に応じて plan を更新（変更履歴は git diff で追える） | ✅ する |
| 5. PR マージ | `status: completed` に、`pr` フィールドに番号確定 | ✅ する |
| 6. PR Close | `status: rejected` に、末尾に Close 理由を追記 | ✅ する |

- 「ドラフトだから commit しない」運用はしない。worktree ローカルにのみ置くと worktree 削除で消失し、別セッション・別 PC・別エージェントから参照不能になる。
- **plan の commit は必ず実装と同じ feature branch に含め、PR で main にマージする。** PR の diff に plan が含まれることで、人間 + Claude のレビュアーが「なぜこの変更が入ったのか」を追える。
- **main 直接 push は禁止**（branch protection で物理強制）。`status: in-progress → completed` の事後追記・誤字修正・リンク切れ修正など、軽微な変更も含めて plan へのすべての変更は PR 経由。本スキル自体の更新も同様に PR 経由。
- plan 単体の軽微 PR でも CI green を待ってマージする。`gh pr merge --squash` のみ。`--admin` / `--no-verify` で CI をスキップして例外を作らない。待ち時間が惜しければ軽微更新を実装 PR にまとめる。

## Plan の必須セクション

次セッションが**読むだけで即着手できる**レベルにする。最低限：

1. **背景・本セッションの完了事項**: コンテキスト復元用の最小限の要約
2. **ゴール**: ユーザ方針と完了条件
3. **設計原則**: `.agents/AGENTS.md` および既存スキルから適用するルール（特にセキュリティ・プライバシー・倫理を利便性と引き換えにしない、を明記）
4. **タスク詳細**:
   - 変更ファイル一覧（テーブル形式。`crates/core` (csw-core / ロジック + テスト) / `crates/cli` (csw-cli) / `crates/desktop` (csw-desktop / Tauri v2) のどこを触るか）
   - 実装ステップ（番号付きの手順）
   - 検証計画（下記コマンド）
5. **リスクと対応**
6. **スコープ外（混ぜない）**: 別 PR で扱う変更を明示
7. **完了条件**: チェックリスト形式

### 検証計画に書くコマンド

ローカルに cargo が無い環境があるため、**検証の正典は CI**（実装 → CI で build/test/clippy → fix → 収束、の Workflow フィードバックループで回す）。ローカルで動く場合は同じコマンドを先に流す：

```bash
cargo fmt --check                                    # フォーマット
cargo clippy --workspace --all-targets -- -D warnings # lint（warning をエラー扱い）
cargo build --workspace                              # build（desktop 含む / build.yml 相当）
cargo test --workspace                               # test（ローカル）
```

- CI 上では: `test.yml` が `cargo test --workspace --exclude csw-desktop`（core + cli）、`build.yml` が `cargo build --workspace`（desktop 含むワークスペース全体）を実行する。
- GUI の手動確認が要るタスクのみ: `cargo tauri dev` を検証計画に追記する。
- edition 2024 前提。npm / Next.js / Supabase / Expo は存在しないので検証計画に書かない。

## PR 連携（必須）

PR 本文の冒頭に必ず plan へのリンクを記載する：

```markdown
## Plan
[docs/proposals/<slug>.md](https://github.com/matsumotory/claude-desktop-switcher/blob/main/docs/proposals/<slug>.md)

## Summary
...
```

これにより、レビュアー（人間 + Claude）が背景・設計判断・スコープを即座に理解でき、マージ後に `status: completed` へ更新する起点になる。PR レビューは `/code-review` スラッシュコマンド（`--comment` でインライン投稿、`--fix` で修正適用）または Task/Agent サブエージェントで観点別に回す。`/gemini review` は使わない。

## セッション開始時: 読むべきもの

1. **`git fetch origin main`** を最初に必ず実行（ローカル checkout は信用しない）。作業は `.claude/worktrees/<name>` の隔離 worktree で行い、`/tmp` には作らない
2. `docs/SPECIFICATION.md` — 仕様の現在地・全体コンテキスト
3. `docs/proposals/` 配下の `status: in-progress` または `status: approved` の plan — 具体的な着手ポイント
4. `.agents/AGENTS.md` と該当ドメインの `.agents/skills/`（`modern-rust-workflow` / `tauri-v2-best-practices` 等）
5. セキュリティ確認: `cargo audit`（導入済みなら）または `.github/workflows/security.yml` の結果を確認。脆弱性があればセキュリティ対応を優先する

## セッション終了時のチェックリスト

- [ ] Plan の `status` を実態に合わせて更新（`approved` → `in-progress` → `completed` 等）
- [ ] 未完了タスクがあれば plan の「実装ステップ」を最新化（次セッションが文脈ゼロから再開できるように）
- [ ] PR 作成済みなら plan の `pr` フィールドに番号を確定追記
- [ ] PR 本文に plan へのリンクが貼られていることを確認
- [ ] CI が green であることを確認してから `gh pr merge --squash` でマージ（自律マージ可）
- [ ] マージ後、primary checkout で `git pull origin main` を実行し、ローカル main を同期

## アンチパターン

- ❌ Implementation Plan なしで大規模実装に着手 / セッションを終了する（次セッションが文脈ゼロから始まる）
- ❌ Plan を worktree ローカル（git 管理外）にのみ置く（worktree 削除で消失、別セッション参照不能）
- ❌ Plan を main に直接 push する（PR の diff に含まれず、branch protection 違反。plan ⇔ 実装の紐付きが失われる）
- ❌ Plan を作っても PR 本文にリンクを貼らない（レビュアーが背景を追えない）
- ❌ `docs/SPECIFICATION.md` / `USER_GUIDE` に「次やること」の詳細手順を書く（完了状態の記録のみ。前方参照は plan に書く）
- ❌ `--admin` / `--no-verify` で CI を飛ばしてマージする（軽微な plan 単体 PR でも禁止）
- ❌ status を更新せず古い状態で放置 / frontmatter を省略する（plan の信頼性低下・status 集計不能）
- ❌ 1 plan に複数の独立タスクを詰め込む（PR 1 つにつき plan 1 つを基本とする）
- ❌ AI 生成の引き継ぎメモ・スクラッチを repo に commit する（scratchpad に留める。`.agents/AGENTS.md` §7 違反）
