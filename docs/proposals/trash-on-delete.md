---
title: 環境削除のゴミ箱化と削除前サマリー
created: 2026-07-03
status: in-progress
pr: 153
related_prs: []
related_issues: []
---

# 環境削除のゴミ箱化と削除前サマリー

## 背景・本セッションの完了事項

環境のフォルダは、そのアカウントの会話履歴・自動メモリ・サインイン状態という取り返しのつかないデータの器だが、現状の削除は `remove_dir_all` による即時の完全削除で、取り違えから戻す手段がない。2026-07-03 のサーベイで、既存の切替ツールにプロファイル削除の復元手段を持つものは確認できなかった。主対象がターミナルを使わないデスクトップユーザーである以上、macOS の利用者が身につけている既定 (消してもゴミ箱から戻せる) に合わせる。ユーザーは 4 案すべての実施を承認済みで、本 plan は最後の 4 本目である (PR #149 / #151 / #152)。

## ゴール

1. 環境の削除を、既定で「フォルダごと macOS のゴミ箱へ移動」に変える。ゴミ箱を空にするまで、サインイン状態を含むすべてのデータを戻せる。
2. 削除の確認時に、その環境の専有容量 (PR #152 の集計を再利用) と、ゴミ箱の挙動を正直に示す。
3. すぐに完全削除したい利用者の導線も残す (GUI の「完全に削除」、CLI の `--purge`)。

## 設計原則 (敵対検証で確定した仕様条件)

- **文言は事実に一致させる**: 「ゴミ箱へ移動され、空にするまでサインイン状態を含む全データが復元できる」ことを隠さず示す。セキュリティを理由に即時の完全削除を選びたい利用者のために、GUI にも完全削除の選択肢を置く。
- **「Finder の削除と同じ」とは書かない**: ゴミ箱の「戻す」メニューが効くとは限らないため。復元手順は「ゴミ箱からフォルダを `profiles/` へ戻す」と自分の言葉で書き、戻した環境を CSW が再認識することをテストで固定してから「戻せる」と謳う。
- **サイレントフォールバック禁止**: ゴミ箱への移動に失敗したら、完全削除に切り替えずエラーとして伝え、完全削除の導線 (`--purge` / 「完全に削除」) を案内する。
- **集計は PR #152 の data_map を再利用**: シンボリックリンクを辿らず、中身を開かない (テスト固定済み)。
- **ゴミ箱行きはフォルダをそのまま移動する (実装中に確定)**: 事前の `unlink_profile` は行わない。unlink は実体ファイルも消すため、ゴミ箱に入る前にデータを破壊してしまう (RED テストで検出)。共有リンクはリンクのまま移動し、既存の Claude 側は無傷。復元すると共有リンクも含めて完全に元へ戻る。完全削除 (`--purge`) のみ従来どおり unlink してから削除する。
- **ゴミ箱操作は trait 経由**: `PlatformProvider::move_to_trash` として抽象化し、テストは MockPlatformProvider (tempdir への移動) で流す。macOS 実装は trash crate v5.2.6。**NSFileManager 方式を明示指定する (実装中に確定)**: 同 crate の macOS 既定は Finder への AppleScript で、Automation 権限のプロンプトとタイムアウトを招き (実機で確認)、PRIVACY の「実行する OS コマンド」一覧とも矛盾するため使わない。

## タスク詳細

### 変更ファイル一覧

| ファイル | 変更 |
|---|---|
| `crates/core/Cargo.toml` | trash = "5.2.6" を追加 |
| `crates/core/src/platform/` | `move_to_trash` を trait に追加 (macos 実装 + mock 実装) |
| `crates/core/src/profile/mod.rs` | `delete_profile` を「unlink → ゴミ箱へ移動」に変更し、従来の完全削除を `purge_profile` として分離 |
| `crates/core/src/profile/tests.rs` | ゴミ箱移動・復元での再認識・purge・移動失敗時のエラーのテスト (RED から) |
| `crates/cli/src/main.rs` | `csw profile delete` は既定でゴミ箱、`--purge` で完全削除。移動失敗時は --purge を案内 |
| `crates/desktop/src/main.rs` | `delete_profile` (ゴミ箱) + `purge_profile` command |
| `crates/desktop/ui/` | 削除確認に専有容量とゴミ箱の説明を表示し、「ゴミ箱へ移動」「完全に削除」の 2 択にする (ja/en) |
| `docs/SPECIFICATION.md` / `USER_GUIDE.md` / `USER_GUIDE_EN.md` | 削除の挙動と復元手順 |
| `docs/PRIVACY.md` / `PRIVACY_EN.md` | 書く場所に「削除時は環境のフォルダをゴミ箱へ移動する」を追記 (実装と同期の宣言を守る) |

### 実装ステップ

1. テストを先に書き RED を確認 → trait + 実装で GREEN
2. CLI / desktop / GUI (japanese-typography-qa / csw_product_canon のゲートを通す)
3. docs / PRIVACY 同期
4. 敵対検証 workflow → 指摘反映 → CI green → マージ

### 検証計画

```bash
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace && cargo test --workspace
```

- GUI は launch の app (devInvoke モック) で ja/en の実描画確認。
- 実機検証: 使い捨て環境を作成 → CLI で削除 → macOS のゴミ箱に入っていることを確認 → `profiles/` へ戻して一覧に再表示されることを確認 → `--purge` で完全削除。

## リスクと対応

- **ゴミ箱 API の環境差**: trait 化により CI (テスト) は mock で決定的に流れる。実機は手元で検証する。
- **完全削除の誤操作**: 「完全に削除」はゴミ箱行きと別ボタンにし、ラベルで結果を自明にする。

## スコープ外 (混ぜない)

- ゴミ箱からの復元を GUI で行う機能 (手順の案内のみとする)。
- 既存の Claude (default) の削除 (従来どおり不可)。

## 完了条件

- [ ] ゴミ箱移動・復元再認識・purge・失敗時エラーのテストが RED から GREEN
- [ ] 実機で「削除 → ゴミ箱で確認 → 戻して再認識 → purge」の一巡を実測
- [ ] GUI の確認ダイアログが ja/en で実描画確認済み
- [ ] docs / PRIVACY の同期
- [ ] CI 全ジョブ green、squash マージ済み
