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

