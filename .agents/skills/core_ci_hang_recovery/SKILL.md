---
name: core_ci_hang_recovery
description: CI (GitHub Actions の Test / Build / Security job) が想定時間を大幅に超過した場合のハング検出と cancel / rerun フロー。macos-latest runner の混雑や cargo の依存取得・コンパイル stall 等、transient な GHA 側障害の典型対処。CI が想定より長く `in_progress` のまま動かない時に発動。
---

# CI ハング検出と recovery

## 1. 背景

CSW の CI は GitHub Actions 上で動く Rust ワークフロー群:

| Workflow | runner | 内容 | 通常の目安 |
|---|---|---|---|
| **Test** | macos-latest | `cargo test --workspace --exclude csw-desktop` (core + cli) | 数分〜十数分 (依存コンパイル含む) |
| **Build** | macos-latest | `cargo build --workspace` (desktop 含む全 crate) | 数分〜十数分 |
| **Security** | ubuntu-latest | gitleaks secret scan | 1〜2 分 |

これらを大幅に超えて `in_progress` のまま動かない場合、特定 step (toolchain setup / 依存 crate の取得・コンパイル / runner 起動待ち) でハングしている可能性が高い。

CSW で疑うべき transient 障害の典型:

- **macos-latest runner の混雑**: macOS runner はキューが詰まりやすく、起動・割当待ちで `queued` / `in_progress` が長引くことがある。
- **cargo の依存取得 stall**: crates.io / registry への network が一時的に詰まり `Updating crates.io index` や `Downloading` で固まる。
- **コンパイル中のメモリ/IO 枯渇**: 大きい依存ツリーで runner リソースが逼迫し step が進まなくなる。

> いずれも **PR のコード側ではなく GHA runner 側の transient な問題**。force push やブランチ切り直しでは直らない。cancel + rerun が正解 (§5 アンチパターン参照)。

放置すると:
- マージできず止まり、ユーザー体験が悪化する
- バックグラウンド polling が無限ループする
- 後続 PR の rebase / branch update も詰まる

## 2. いつ実行するか

以下のいずれかに該当する時:

- CI 状態が **10 分以上 `in_progress`** のまま step が進まない
- バックグラウンド poll タスクが想定の倍の時間を超えても完了しない
- ユーザーから「CI 固まってない?」と指摘された時

> 注意: CSW の Test / Build は依存コンパイルで通常でも数分〜十数分かかる。**「遅い」だけで即 cancel しない**。step が `in_progress` のまま進捗が止まっているか (§3.1 で step 名を確認) を見て判断する。

## 3. 手順

`<owner>/<repo>` は `matsumotory/claude-desktop-switcher`。`<branch>` は作業ブランチ。

### 3.1 ハングしている step を特定

```bash
RUN_ID=$(gh run list --repo <owner>/<repo> --branch <branch> --limit 1 --json databaseId --jq '.[0].databaseId')

# job ごとに in_progress な step 名を出す (Test / Build を見分ける)
gh run view "$RUN_ID" --repo <owner>/<repo> --json jobs \
  --jq '.jobs[] | {job: .name, step: (.steps[] | select(.status=="in_progress") | .name)}'
```

例: `Setup Rust` / `Build all crates` / `Run unit and integration tests` が長時間 `in_progress` のまま、かつ進捗ログが止まっていればハングと判断する。`gh run view "$RUN_ID" --repo <owner>/<repo> --log | tail -50` でログ末尾も確認するとよい (`Downloading` / `Updating crates.io index` で固まっていれば network stall)。

### 3.2 Cancel + Rerun

```bash
gh run cancel "$RUN_ID" --repo <owner>/<repo>

# cancel は async — completed を待ってから rerun
until [[ "$(gh run view "$RUN_ID" --repo <owner>/<repo> --json status --jq '.status')" == "completed" ]]; do
  sleep 5
done

gh run rerun "$RUN_ID" --repo <owner>/<repo>
```

> `gh run rerun` は cancel 中 / in_progress 中は実行できない (`cannot be rerun; This workflow is already running`)。必ず `completed` を待つ。

failed した job だけ再実行したい場合は `gh run rerun "$RUN_ID" --repo <owner>/<repo> --failed`。runner 混雑が原因の rerun はそのまま全 job で問題ない。

### 3.3 Re-run 完了をハング検出付きで待つ

CSW の Test / Build はコンパイル時間が長いので、タイムアウトを長めに取る。**最大 20 分タイムアウト + step 詳細出力**で polling する:

```bash
START=$(date +%s)
until [[ "$(gh pr view <PR> --repo <owner>/<repo> --json statusCheckRollup --jq '[.statusCheckRollup[].status] | all(. == "COMPLETED")')" == "true" ]]; do
  NOW=$(date +%s)
  if (( NOW - START > 1200 )); then
    echo "[WARN] CI > 20min, check for hang"
    RUN_ID=$(gh run list --repo <owner>/<repo> --branch <branch> --limit 1 --json databaseId --jq '.[0].databaseId')
    gh run view "$RUN_ID" --repo <owner>/<repo> --json jobs \
      --jq '.jobs[] | {job: .name, step: (.steps[] | select(.status=="in_progress") | .name)}'
    break
  fi
  sleep 30
done
```

これで 20 分以上ハングしたら自動検知 → ユーザーに報告できる。

## 4. バックグラウンド polling の作法

長時間 (>5 分) 待つ CI 監視は `Bash` の `run_in_background: true` でバックグラウンド化する。ただし:

- **必ず最大タイムアウトを設ける** (無限ループ禁止)
- **ハング検出条件** (step 単位の進捗チェック) を組み込む
- 完了通知が来たら出力を確認してから次の判断に進む
- CSW は Test / Build / Security の複数 workflow が走るので、`statusCheckRollup` を **全 check 集約** (上記 `all(. == "COMPLETED")`) で判定する。1 件だけ見て早合点しない

## 5. アンチパターン

- ❌ **「遅い」だけで即 cancel する**: CSW の Test / Build は依存コンパイルで通常でも数分〜十数分かかる。step が実際に止まっているか確認してから cancel する
- ❌ **ハングしているのに force push / branch 切り直しで「直そうとする」**: 原因は GHA runner 側 (macos-latest 混雑 / network stall) で PR コード側ではない。cancel + rerun が正解
- ❌ **ユーザーに知らせず無限に待つ**: 20 分超えたら自分から「CI が想定より長い、ハングの可能性」と報告する
- ❌ **`gh run rerun` を cancel 完了前に呼ぶ**: エラーになる。`completed` を待つ
- ❌ **`--admin` / `--no-verify` でゲートを飛ばしてマージ回避する**: ハング回避を口実に検証を飛ばさない。CI green を待ってから `gh pr merge --squash` (上位ルール参照)

## 6. 切り分けの目安

| 症状 | 疑い | 対処 |
|---|---|---|
| `queued` のまま長い | macos-latest runner キュー混雑 | 数分待つ。10 分超なら cancel + rerun |
| `Setup Rust` で止まる | toolchain 取得 stall | cancel + rerun |
| `Updating crates.io index` / `Downloading` で止まる | registry network stall | cancel + rerun |
| コンパイル途中で長時間無進捗 | runner リソース逼迫 | cancel + rerun |
| 特定 test が毎回同じ所で落ちる/固まる | **コード側の問題 (transient ではない)** | rerun せず実装を疑う (`bugfix` skill へ) |

最後の行が重要: **rerun して同じ step で再現するならコード側の問題**。transient かどうかは「1 回 rerun して直るか」で切り分ける。2 回同じ所でハング/失敗したら GHA 側ではなく実装・依存定義を疑い、cargo (CI 上) で再現させて直す。

## 7. 関連

- 関連スキル: `core_pr_review_cycle` (PR フロー / マージ判定), `modern-rust-workflow` (cargo 検証コマンド全般), `core_bug_fix_protocol` (rerun でも直らない = コード側のとき)
- 上位ルール: CI green を待ってから `gh pr merge --squash`、`--admin` / `--no-verify` は使わない。`main` への直接 push 禁止、作業は `.claude/worktrees/<name>` で隔離
