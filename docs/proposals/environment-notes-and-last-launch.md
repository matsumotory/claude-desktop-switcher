---
title: 環境の用途メモと最終起動・来歴の表示
created: 2026-07-03
status: approved
related_prs: []
related_issues: []
---

# 環境の用途メモと最終起動・来歴の表示

## 背景

v0.19.1 時点の環境一覧は、名前とアイコンしか手がかりを出さない。環境名は日本語も使えるが、空白と句読点が使えない 64 文字以内の短いハンドルで、`csw env` にもそのまま渡る。そのため「どのアカウントでサインインしたか」「何の案件用か」のような説明文を書ける場所がどこにも無く、環境が増えるほど選び間違いと削除前のためらいが起きる。2026-07-03 のアイデア検討でユーザー承認済み。LP・SPECIFICATION に既出の約束は無い (grep 確認済み)。

## ゴール

- 環境に自由記述のメモ (1 行、最大 200 文字) を持たせる。作成ダイアログで入力でき、既存環境の詳細画面からも編集できる。複製はメモを複製元から引き継ぐ。
- 一覧の各行に、メモと「最終起動: 3 日前」の形式のサブテキストを表示する。
- 詳細画面に来歴 (作成日、複製元、まだ一度も起動していない状態) を表示する。
- CLI でも同じ情報を扱える (`csw profile list` / `show` の表示、`csw profile note` での編集、`create` の `--note`)。
- 既存の Claude は対象外 (CSW を経由せず起動されるのが普通のため、最終起動もメモも表示しない)。

## 対象ユーザーと規模

環境を複数使うすべてのユーザーで、非エンジニアを含む。UI はサブテキストとフォーム 1 欄の追加に収まり、docs の案内だけでは一覧を開いた瞬間に効かないため、この規模の作り込みが妥当 (軽い代替との比較済み)。

## 設計原則

- セキュリティ・プライバシーを利便性と引き換えにしない。サインイン状態の検出・推測はせず、メモは「ユーザーが書いた文字列」以上の意味を持たせない。自動分類・自動切替を作らない。
- 安全宣言を含む profile.toml に高頻度の書き込みをしない。最終起動時刻は `profiles/<name>/state.toml` に分離し、temp+rename の atomic 書き込みにする。メモ編集 (profile.toml の書き換え) も atomic にする。
- ラベルは記録実態どおり「最終起動」とする (利用終了は観測しない)。`csw switch --no-launch` では刻印しない。
- メモ本文と環境名はユーザーデータであり翻訳対象から除外する (I18N_SKIP)。実行時に連結する相対時刻の文言は T() で言語別に構築する。
- UI コピー・レイアウトの編集前に japanese-copy-discipline / csw_product_canon / minimalist-ui を Read する (Skill-First Gate)。

## タスク詳細

### 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `crates/core/src/profile/mod.rs` | ProfileMeta に `note` / `created_at` / `cloned_from` を serde default 付きで追加 (後方互換)。`set_profile_note` (検証 + atomic 書き換え)。create / clone での刻印 |
| `crates/core/src/profile/state.rs` (新規) | 起動状態ファイル `profiles/<name>/state.toml` の読み書き (atomic)。`record_last_launch` / `last_launched_at` |
| `crates/core/src/switcher/mod.rs` | 起動を伴う切替経路で最終起動を刻印 (no-launch 経路では呼ばない) |
| `crates/core/src/profile/tests.rs` | RED テスト一式 (下記) |
| `crates/cli/src/main.rs` | `profile list` / `show` の表示追加、`profile note` サブコマンド、`create` / `clone` の `--note` |
| `crates/desktop/src/main.rs` | `set_profile_note` コマンド、起動系コマンドでの刻印、get 系への新フィールド露出 |
| `crates/desktop/ui/main.js` ほか | 一覧サブテキスト、詳細画面の来歴とメモ編集、ダイアログのメモ欄「メモ（任意）」、相対時刻の日英 |
| `docs/PRIVACY.md` / `PRIVACY_EN.md` | 書く場所に state.toml を追加、profile.toml の更新タイミング (メモ編集時) を追記 |
| `docs/USER_GUIDE.md` / `USER_GUIDE_EN.md` | メモと最終起動の説明を追加 |
| `docs/SPECIFICATION.md` | 完了後に現在地を更新 |
| `website/assets/` スクショ | UI 変更に伴い `scripts/appshot` で ja / en 再生成 |

### RED テスト一覧 (期待値は本仕様から導出)

- 新フィールドの無い既存 profile.toml がそのまま読める (後方互換)
- `create_profile` が `created_at` を刻印し、`--note` 相当のメモを保存する
- `clone_profile` が `cloned_from` に複製元を刻印する
- `set_profile_note`: 200 文字超と制御文字を拒否する / default を拒否する / 上書きできる / 空文字でクリアできる / 書き換え後に再読込できる (atomic)
- `record_last_launch`: `state.toml` が生成され再読込で時刻が返る / default を拒否する / profile.toml のバイト列が変化しない
- 未記録の環境の `last_launched_at` は None

### 実装ステップ

1. RED: 上記テストを書き、`cargo test` で失敗を確認する
2. GREEN: core を実装してテストを通す
3. CLI → desktop (Tauri command + UI) の順に実装する
4. PRIVACY / USER_GUIDE の ja・en 伝播、スクショ再生成
5. 敵対検証 (事実・日本語コピー・確定語彙の 3 観点) を通してから commit / PR / CI green / 実機確認

### 検証計画

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
```

GUI は `cargo tauri dev` での実機確認と `scripts/appshot` の再生成画像を Read で目視する。CI green を正典とする。

## リスクと対応

- GUI と CLI が同時にメモを編集した場合は最後の書き込みが勝つ (atomic 書き込みなので破損はしない)。
- ゴミ箱から復元した環境は `state.toml` もフォルダごと戻るため、最終起動の記録も一緒に復元される (設計どおり)。
- 相対時刻の文言は実行時連結のため全文一致辞書に当たらない。T() による言語別構築で対応する (CLAUDE.md の i18n 原則 5)。

## スコープ外 (混ぜない)

- 複製ダイアログのメモ入力欄と、CLI への clone コマンド追加。複製はコアがメモを引き継ぎ、詳細画面ですぐ編集できるため、入力欄を追加しない
- 初回サインインの道しるべ (案 2、次 PR。`first_launched_at` はそちらで追加する)
- 一覧の並び替え・アーカイブ
- 更新後の乗り移り検知 (案 5)・現在地 HUD (案 4)

## 完了条件

- [ ] RED から GREEN にしたテストが CI で green
- [ ] 一覧・詳細・ダイアログ・CLI の表示をエビデンス付きで確認 (スクショ / コマンド出力を自分で開く)
- [ ] PRIVACY / USER_GUIDE の ja・en 伝播、スクショ再生成、旧表現の grep 残存ゼロ
- [ ] japanese-copy-discipline / japanese-typography-qa の出荷前チェックを実測で通過
