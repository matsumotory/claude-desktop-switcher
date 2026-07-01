---
name: core_pr_merge_checklist
description: PR をマージする際のレビュー・マージ実行チェックリスト (Rust + Tauri v2 / CSW)。スコープ外変更・安全装置の格下げ・テスト負債・高リスクインフラファイル改変を検出し、CI green を確認したうえで自律的に squash merge する手順を定義する。
---

# PR マージチェックリスト

PR (自分・他エージェント・共同作業者が作成したもの) をマージ可否判断し、実際にマージするまでのプロトコル。CSW は Rust + Tauri v2 macOS アプリ。verification の一次経路は CI (cargo がローカルに無い env がある)。

## 発動条件

- PR をマージしようとするとき / マージ可否を判断するとき。
- レビュー (`/code-review`) が一巡し、指摘対応が済んだ後の最終ゲートとして。

## 0. マージ前の前提 (常に)

- worktree (`.claude/worktrees/<name>`) で作業する。primary checkout は読まない・触らない。
- 参照前に必ず `cd <repo> && git fetch origin main`。ローカル checkout は信用しない。
- `main` に直接 push しない。マージは PR 経由 (`gh pr merge`) のみ。

## 1. スコープの厳密確認

PR の全 diff がタイトル・説明の目的と一致しているか確認する。

> [!CAUTION]
> **スコープ外の変更は最大のリスク**。「ついで修正」で無関係なインフラ設定が混入する事故は実際に起きる。diff を端から端まで読む。

- [ ] 全 diff が PR タイトル・説明の目的と合致しているか
- [ ] 無関係な「ついで修正」が混入していないか (リファクタ・依存追加・設定変更)
- [ ] **想定外の大量削除がないか** (関数・テスト・ガードのまとめ消し)
- [ ] 高リスクインフラファイルの変更に特に注意 (下記)

### 高リスクインフラファイル (CSW)

これらが diff に含まれたら、変更理由が PR の目的と直結しているか必ず精査する:

- `crates/desktop/tauri.conf.json` — CSP / 権限 / バンドル / 署名設定
- `crates/desktop/capabilities/*` — Tauri v2 capability (コマンド許可範囲)
- ルート・各 crate の `Cargo.toml` — 依存追加 / feature / edition (2024)
- `.github/workflows/release.yml`・署名 / entitlement / notarization 関連
- `.github/workflows/{test,build,security}.yml` — CI 定義そのもの

## 2. 安全装置の緩和検出 (Rust 視点)

既存の防御ロジック・エラーハンドリングが弱められていないか確認する。以下は **格下げの臭い**:

- [ ] `Result` / `Option` → `unwrap_or_default()` でエラーを握り潰していないか
- [ ] `.expect()` / 明示的なエラー伝播 → silent return (`return Ok(())` / 早期 return) への置換
- [ ] `#[allow(...)]` を足して clippy / コンパイラ warning を黙らせていないか (追加時はユーザー承認必須)
- [ ] CSP / capability の wildcard 化 (`"*"`, 過剰な scope, `dangerous*` の追加)
- [ ] 入力バリデーション・権限ガード・パス検証の削除や緩和
- [ ] `unsafe` ブロックの新規追加 (正当性が説明されているか)

> [!CAUTION]
> セキュリティ / プライバシー / 倫理を「楽だから」で交換しない。安全装置の緩和が見つかったら、利便性を理由にした緩和は却下する。

## 3. テストの健全性

テスト負債を増やす変更を防止する。

> [!IMPORTANT]
> **安直なテスト無効化は禁止**。テストを落とす / 飛ばすなら代替カバレッジの作成が必須。

- [ ] `#[ignore]` が追加されていないか
- [ ] `#[cfg(not(test))]` や条件付きコンパイルでテスト経路を握り潰していないか
- [ ] テストの削除は、同等以上の代替カバレッジがある場合のみ許容
- [ ] アサーションの弱体化 (`assert_eq!` → 緩い条件 / コメントアウト) がないか
- [ ] ロジックは `crates/core` (csw-core) にテストが付いているか

## 4. CI 状態の確認

CSW の CI ジョブ構成を踏まえて判定する:

- **Test** (`test.yml`): `cargo test --workspace --exclude csw-desktop` (= core + cli)
- **Build** (`build.yml`): `cargo build --workspace` (desktop 含む全 crate)
- **Security** (`security.yml`): gitleaks による secret scan

チェック:

- [ ] 上記 CI ジョブが全て green か (`gh pr checks <num>` で確認)
- [ ] 失敗が main 既存の問題か、この PR 起因かを区別する
- [ ] **PR 起因の失敗があるまま絶対にマージしない**
- [ ] secret scan (gitleaks) の指摘は最優先で対処

> [!NOTE]
> CI test は desktop を除外、build は desktop を含む。desktop (Tauri) 側のコンパイルエラーは Test を素通りし Build で初めて落ちる。Tauri 関連の変更があるときは Build の結果を必ず確認する。

## 5. ローカル / pre-push 品質ゲート

CI に clippy / fmt ジョブは無い。これらは push 前のローカル検証で担保する (cargo が無い env では CI build/test 通過 + diff レビューで代替):

- [ ] `cargo fmt --check` — フォーマット差分なし
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` — warning ゼロ
- [ ] `cargo build --workspace` / `cargo test --workspace` — 通過
- [ ] GUI 挙動に関わる変更は `cargo tauri dev` で実機確認 (可能なら)

## 6. レビュー指摘の反映

- [ ] `/code-review` のインライン指摘を確認し、妥当な指摘は反映する
- [ ] 言語ルール違反 (コメント / doc の言語統一) の指摘は必ず対応する
- [ ] spec に影響する変更は `docs/SPECIFICATION.md` (canon) と整合しているか
- [ ] セッション計画は `docs/proposals/` と矛盾していないか

## マージ判定フロー

```
PR の diff を端から端まで確認
  ├─ スコープ外の変更あり        → ❌ 修正要求 or 部分取り込み
  ├─ 安全装置の格下げあり        → ❌ 却下 (セキュリティ/プライバシー)
  ├─ 高リスクインフラ改変が無根拠 → ❌ 説明を求める / 却下
  ├─ #[ignore] 等のテスト無効化  → ⚠️ 代替カバレッジ無しなら修正要求
  ├─ CI 失敗 (PR 起因)           → ❌ 修正するまでマージ不可
  └─ 上記すべてクリア + CI green  → ✅ マージ実行 (下記)
```

## マージ実行手順 (自律)

> [!IMPORTANT]
> **CI が green になったら、確認を取らず自律的に squash merge する** (ユーザー standing rule)。意味のない確認で止まらない。

1. worktree で head ブランチを checkout する
2. `git merge origin/main --no-edit` で最新 main を取り込む
3. マージ後の diff を再レビューし、**想定外の大量削除 / コンフリクト解決ミスが無いか**確認する
4. `git push`
5. CI が全ジョブ green になるまで待つ (`gh pr checks <num>` を監視。green を待たずにマージしない)
6. `gh pr merge --squash`
   - **`--admin` / `--no-verify` は使わない** (保護・検証をバイパスしない)
   - **`--delete-branch` は付けない**
7. マージ後、primary checkout で `git pull origin main` してローカル main を同期する
8. 作業 worktree を `git worktree remove` で片付ける

## 部分取り込みパターン

PR の一部だけが有用な場合:

1. 有用な変更だけを cherry-pick or 手動再実装で取り込む
2. PR にクローズ理由をコメントで明記する
3. 取り込んだ変更 / 除外した変更を区別して記述する

## リリース PR (release-please) のマージ

`fix:` / `feat:` / 破壊的変更が main に入ると、release-please が `chore(main): release X.Y.Z` という PR を自動起票する。これは版更新 (`Cargo.toml` / `crates/desktop/tauri.conf.json` / `.release-please-manifest.json`) と `CHANGELOG.md` だけで、**コードは含まない** (機能は各 `feat`/`fix` PR で CI 済み)。

> [!IMPORTANT]
> **リリースはユーザーの明示指示 (「リリースして」等) があってから行う。** 公開・巻き戻し困難な操作なので、勝手に走らせない。

### なぜ通常マージが BLOCKED になるか

このリリース PR は GitHub の bot (`GITHUB_TOKEN`) が作るため、GitHub のループ防止で `pull_request` ワークフローが発火しない。結果、必須チェック (Build / Test / Lint / Secret scan) が一つも付かず、`mergeStateStatus` が `BLOCKED` になる (`mergeable` は `MERGEABLE` のまま)。

### 正しい手順 (`--admin` を使わない)

`main` の branch protection は `enforce_admins:false` なので `--admin` で押し込むことは可能だが、**使わない** (standing rule、auto モードの classifier もブロックする)。代わりに必須チェックを実際に走らせてから通常マージする:

1. `git fetch origin release-please--branches--main`
2. worktree を切り、release ブランチ (`release-please--branches--main`) に **空コミットを 1 つ** push する:
   `git commit --allow-empty -m "chore: 必須チェックを発火させる"` → `git push origin HEAD:release-please--branches--main`
   (CI ワークフローは `on: pull_request` で paths フィルタが無いため、実ユーザーの push が `synchronize` で必須チェックを発火させる。取りこぼした CHANGELOG エントリがあればこの commit で追記してもよい。)
3. `gh pr checks <PR番号>` で 4 チェックが全 green・`mergeStateStatus` が `CLEAN` になるのを待つ。
4. `gh pr merge <PR番号> --squash` (**`--admin` なし**、`--subject "chore(main): release X.Y.Z"` でメッセージを明示)。
5. マージ後の後始末 (worktree/branch 撤去、primary で `git pull origin main`)。

### マージ後のリリースビルド監視

マージすると `Release Please` (`release.yml`, `on: push main`) が走り、`release-please` ジョブが tag `vX.Y.Z` と GitHub Release を作成し、続けて `build-mac-dmg` (署名・公証つき universal DMG) と `publish-csw` (署名・公証つき `csw` バイナリ添付) が走る。

- [ ] `gh run view <id>` で全ジョブ green を確認する。
- [ ] `gh release view vX.Y.Z` で Release が draft/prerelease でなく、`*_universal.dmg` と `csw` が添付されていることを確認する。
- [ ] 署名・公証やアセット添付が落ちたら green になるまでデバッグする (成功を仮定しない)。

技術背景 (squash とメッセージのパース注意) は個人メモリ `release-please-squash-parse-pitfall` / `autonomous-merge-after-ci` も参照。


## 追記: 自律マージ・release-please・CI 課金

このセクションは、上記の手順を運用する際に踏み外しやすい 2 点 (squash メッセージのパース事故 / macOS CI の課金事故) を規則として補う。マージ実行そのものの手順は「マージ実行手順 (自律)」節、release PR の扱いは「リリース PR (release-please) のマージ」節を一次情報とする。

### squash マージのメッセージが release-please のパースを壊す

squash マージは、PR のタイトルと本文をそのまま `main` の commit メッセージにする。この commit メッセージは release-please が Conventional Commits としてパースし、CHANGELOG とリリース PR を生成する入力になる。したがって PR 本文に装飾的な markdown が含まれると、パーサが壊れてその commit が changelog / release から取りこぼされ、最悪の場合リリース PR が 1 件も起票されない。

壊す要因になる装飾:

- バッククォート (`` ` ``) で囲んだ inline code
- コマンド置換 `$(...)` やバッククォート実行構文
- 引用符 (`"` / `'`) を含む一行
- 三連バッククォートのコードブロック

規則:

- **予防**: リリースに関わる (`fix:` / `feat:` / 破壊的変更を含む) PR を squash するときは、commit メッセージを本文まかせにしない。`gh pr merge <PR番号> --squash --subject "<type>(<scope>): <要約>" --body "<装飾なしの本文>"` で Conventional Commits に沿ったクリーンなメッセージを明示する。装飾つきの詳しい説明を残したいときは、GitHub 上の PR 本文にだけ置き、commit メッセージには持ち込まない。
- **検証**: マージ後は release-please のワークフローが起票したリリース PR を確認し、対象の commit が CHANGELOG に反映されているかを見る。反映されていなければパース事故を疑う。
- **復旧**: 取りこぼしが起きたら、クリーンなメッセージの `fix:` commit を 1 本足してリリース PR を起票させ、取りこぼした変更点を `CHANGELOG.md` に手動で追記する。壊れた commit を書き換えるのではなく、正しい入力を 1 本足して前に進める。

### macOS CI runner の課金と一律失敗の切り分け

GitHub Actions の macOS runner は、分単価が Linux runner の約 10 倍である。CSW は署名・公証つき DMG のビルドに macOS runner を使うため、CI 実行時間がそのまま費用に直結する。

- **一律失敗はまず課金を疑う**: 全ジョブが例外なく数秒で failure になる場合、コードの誤りやマージ順を疑う前に、まず利用枠 (spending limit) の枯渇を疑う。ジョブが起動直後に一斉に落ちるのは、コンパイルやテストの失敗ではなく、実行そのものが課金上の理由で拒否されている兆候であることが多い。ログにコンパイル出力が全く出ていないかを確認する。
- **CI コスト削減の定石** (ワークフロー / branch protection を触るときに適用する):
  - `concurrency` に `cancel-in-progress: true` を設定し、同一ブランチへの連続 push で古い実行を止める。
  - ビルドキャッシュ (`Swatinem/rust-cache` 等) を効かせ、依存の再ビルドを避ける。
  - 重い macOS ジョブは `on: pull_request` のみに限定し、push ごとの二重実行を避ける。
  - branch protection の "Require branches to be up to date before merging" (strict) は基本 `false` にする。strict を `true` にして多数の PR を逐次マージすると、rebase のたびに全チェックが再実行され、macOS の分を焼き続ける。逐次マージが必要な事情がない限り strict にしない。
