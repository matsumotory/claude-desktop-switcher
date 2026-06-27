---
name: core_worktree_for_derived_tasks
description: メイン作業から派生したサブタスク (chip 提案 / spawn_task / Agent 委譲 / 別 PR 化したい修正 / 割り込み作業) は、必ず `git worktree add` でリポジトリ階層内の `.claude/worktrees/<name>` を `origin/main` 起点に切り、親 repo の working tree / branch / stash には一切触らない。派生タスクを認識した瞬間 (= 別 PR にしたい / 別観点で並行させたい / 割り込みを頼まれた) に発動する。
---

# 派生タスクは必ず worktree で切る

CSW (Claude Desktop Switcher) の開発で **メイン作業から派生したサブタスク** は、**必ずリポジトリ階層内の `<repo-root>/.claude/worktrees/<name>` に `git worktree` を切ってから作業する**。親 repo の working tree / branch / stash には一切触らない。worktree は `origin/main` を起点にし、`/tmp/` 配下には絶対に作らない。

これは Rust + Tauri v2 のワークスペース (`crates/core` `crates/cli` `crates/desktop`) を 1 つの作業ツリーで共有しているため特に重要で、派生タスクが親ツリーを切り替えると `cargo` のビルドキャッシュや `cargo tauri dev` の dev-server が巻き込まれる (後述)。

## 1. 派生タスクとは

以下のいずれかに該当する作業:
- `mcp__ccd_session__spawn_task` (chip 提案)
- `Agent` (Task) tool 委譲 (`isolation: "worktree"` 指定を含む)
- 「これは別 PR にした方が良さそう」と判断して別 branch で進めたい修正
- メイン作業の CI (Test ワークフロー / build.yml) 待ち時間に並行して進めたい関連作業
- ユーザーに「先に別の小さい修正やって」と頼まれた割り込み作業

メイン作業の scope に収まらない / 別 PR に分けたいと気付いた **その瞬間** に本 skill を発動する。「一旦親ツリーで直してから分ける」は禁止 (親ツリーが汚れて scope 混在 PR になる)。

## 2. なぜ別 worktree が必須か

> [!CAUTION]
> 派生タスクが親 repo の HEAD / working tree を巻き込むと、未 commit の変更が discard され、Tauri dev-server が過去状態を reload し、cargo の再ビルドが走る。事故の再発防止としてこの運用を strict に守る。

被害の連鎖:
1. **未 commit の変更が discard** され、再実装が必要になる
2. **`cargo tauri dev` の file watcher が過去の HEAD を reload** して GUI が壊れたように見える (Tauri v2 は frontend asset と Rust 側の両方を watch しており、親ツリーの branch を切り替えると意図しない rebuild / reload が走る)
3. **`cargo` のインクリメンタルキャッシュが切り替わって全 rebuild** が発生し、検証が遅くなる
4. **`git stash` 由来のコンフリクト marker** がファイルに混入し、復旧操作が必要になる
5. ユーザーの作業集中が切れる

tool 任せにせず **明示的に自分で worktree を切る運用** にすることで、親 repo を物理的に保護する。

## 3. 派生タスクを切るときの基本手順

### 3.1 開始 (pre git state check)

```bash
# 親 repo の HEAD を確認。必ず commit + push 済みの安定状態にしてから切る
git status
git branch --show-current   # 親の branch を記録しておく
# clean でなければ先に commit + push する (中途半端な状態で worktree を切らない)

# origin/main を最新化してから、それを起点に worktree を作る
git fetch origin main
git worktree add -b feat/<topic> .claude/worktrees/<topic> origin/main
#   ↑ 配置先は必ずリポジトリ階層内の .claude/worktrees/<topic> (/tmp/ 禁止)
#   ↑ base は必ず origin/main (親の作業 branch から派生させない = scope 混在 PR の防止)

# 親 repo が何も変わっていないことを再確認
git status
git branch --show-current   # 開始時に記録した branch のままであること
```

### 3.2 作業

worktree 内のファイルを編集するときは **絶対パスで指定** し (`.../.claude/worktrees/<topic>/...`)、親の working tree のファイルを誤って触らない。Read / Grep / Edit / Bash の対象パスもすべて worktree 配下に向ける。

```bash
cd <repo-root>/.claude/worktrees/<topic>
# ここで編集 → 検証 → commit → push → PR を完結させる
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace        # desktop 含むワークスペース全体 (build.yml と同等)
cargo test --workspace --exclude csw-desktop   # core + cli (test.yml と同等)
git add ... && git commit -m "..."   # 規約は core_commit_standard
git push -u origin feat/<topic>
gh pr create --title "..." --body "..."

# 完了後、親 repo に戻る
cd <repo-root>
```

> [!NOTE]
> `cargo` がローカルで使えない環境では、push 後の CI (Test ワークフロー / build.yml) を一次検証経路とする。CI を Workflow フィードバックループで監視し、緑になるまで修正を回す (core_ai_workflow)。

### 3.3 終了 (cleanup)

PR が CI 緑でマージされたら worktree を撤収する:

```bash
# 親 dir から
git worktree remove .claude/worktrees/<topic>
git branch -D feat/<topic>          # 不要なら branch も削除 (次回 worktree add の衝突防止)

# primary checkout のローカル main を origin/main に同期する
git pull origin main
```

撤収を放置すると次回 `worktree add` 時に既存 branch / dir で衝突する。

## 4. tool 経由で派生タスクを出すとき

### 4.1 `Agent` (Task) tool

`isolation: "worktree"` を必ず指定する。指定すれば自動的に worktree が切られ、親に影響しない。委譲先 agent には「対象パスは worktree 配下に限定し、親 checkout を読まない・編集しない」ことを明示する。

### 4.2 `mcp__ccd_session__spawn_task` (chip)

説明文には「session and worktree」を起動すると書かれているが、念のため **使う前後で必ず親 repo の状態を確認する**:

1. `git status` で親 working tree が clean か確認
2. すべて commit + push して安定状態にする (未 commit / 未 push 変更があれば消失リスク)
3. spawn 後すぐ `git status` / `git branch --show-current` で親 branch が変わっていないか確認
4. 親 branch が変わっていたら即座に元 branch へ戻し、失った変更があれば再実装

これらを踏めないなら **`spawn_task` を使わず、自分で `git worktree add` を切って作業する**。

## 5. 並行作業の鉄則 (post git state check)

- **`cargo tauri dev` が動いている最中の派生タスクは特に注意**。親ツリーの branch を切り替えると file watcher が即 reload / rebuild する。worktree なら親の dev-server は元 branch のファイルを見続けるので影響なし。派生作業側で GUI 実機確認が必要なら worktree 内で別途 `cargo tauri dev` を起動する。
- 派生タスク完了後、親 dir に戻って必ず確認:
  - `git status` (何も変わっていない)
  - `git branch --show-current` (開始時に記録した branch のまま)
  - `git stash list` (新しい stash が増えていない。増えていたら内容を確認してから扱う)
  - `git reflog -10` (想定外の reset / checkout がない)
- worktree は親と `.git` を共有するため、派生タスクが作った stash は親でも見える。内容を確認せず drop しない。

## 6. 反例 (やってはいけないこと)

- 親 working tree で `git stash` + `git checkout other-branch` で派生作業する (元 branch の `cargo tauri dev` が切り替わり rebuild / reload が走る)
- `spawn_task` を未 commit 変更がある状態で起動する (変更消失リスク)
- worktree を `/tmp/` 配下に作る (`.claude/settings.json` が継承されず auto モード / hooks が効かない)
- worktree 内から親 dir のファイルを編集する (隔離の意味がなくなる)
- worktree を親の作業 branch から派生させる (必ず `origin/main` 起点。そうでないと scope 混在 PR になる)
- `main` へ直接 push / 直接マージする (必ず PR 経由。CI 緑が前提)
- PR マージ後に worktree / branch を撤収せず放置する

## 7. 関連 skill

- [`core_ai_workflow`](../core_ai_workflow/SKILL.md): Workflow / Agent による多段委譲と CI フィードバックループ
- [`core_pr_review_cycle`](../core_pr_review_cycle/SKILL.md): PR レビュー運用 (派生 PR も同じ流れ)
- [`core_commit_standard`](../core_commit_standard/SKILL.md): commit メッセージ規約
- [`modern-rust-workflow`](../modern-rust-workflow/SKILL.md): cargo fmt / clippy / test の検証規律
- [`core_spec_first_development`](../core_spec_first_development/SKILL.md): 仕様正典は `docs/SPECIFICATION.md`、セッション計画は `docs/proposals/`
