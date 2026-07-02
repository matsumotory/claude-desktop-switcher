---
title: 環境ごとの利用量バー（statusline 連携）
created: 2026-07-02
status: in-progress
pr: 141
related_prs: []
related_issues: []
---

# 環境ごとの利用量バー（statusline 連携）

## 背景・前セッションの完了事項

- v0.16.1 まで出荷済み（i18n 完全化、DMG 取り出し案内、バナー修正、整合性監査、スクショ/OG 再生成）。
- 本機能はユーザー承認済み。データ源の候補 3 案から A 案（Claude Code の statusline 連携）で確定している。
  - B 案（非公開 usage API の直叩き）は、通信ゼロ・認証情報非接触というプロダクトの前提と正面衝突するため不採用（再検討しない）。
  - C 案（ローカル JSONL 集計）は Desktop 分が入らず数字が不正確になるため不採用。
- 着手前の既出確認（spec-first 前工程 A）: LP（ja/en）・docs/SPECIFICATION.md・直近 30 コミットのいずれにも本機能への言及なし。完全新規。
- 対象ユーザーと規模（spec-first 前工程 B）: 対象は Claude Code も使う利用者（Pro/Max サブスクリプション）。GUI に組み込む理由は、CSW の価値が「環境 = アカウント別の利用量」を一覧できる点にあり、環境一覧（サイドバー）に出すことがドキュメント案内では代替できないため。CLI 手順の案内だけでは環境横断の一覧が得られない。

## ゴール

サイドバーの各環境行に、Claude の 5 時間セッション上限と週間上限の使用率バーを表示する。

- データ源: Claude Code が statusline スクリプトの stdin に渡す JSON の `rate_limits`（公式ドキュメント https://code.claude.com/docs/en/statusline で 2026-07-02 に実読検証済み）。
  - `rate_limits.five_hour.used_percentage`（0〜100 の数値）/ `resets_at`（epoch 秒）、`rate_limits.seven_day` も同形。
  - Pro/Max のみ・セッション初回応答後に出現・両窓は独立に欠落しうる。防御的にパースする。
- 通信ゼロ・認証情報ストア非接触を維持する（既存のプライバシー主張 16 箇所と両立）。
- 環境ごとに opt-in。既定では誰の settings.json にも触れない（ゼロインパクト保証の維持）。

## 設計原則（CLAUDE.md / スキルから適用）

- セキュリティ・プライバシーを利便性と引き換えにしない: 通信する実装・資格情報を読む実装は採らない（A 案の採用理由そのもの）。
- 不正な状態を表現不能に: 有効/無効の状態は別フラグで二重管理せず、settings.json の実態（statusLine.command が CSW のスクリプトを指すか）から導出する。
- 古い値を新鮮に見せない: 取得時刻（ファイル mtime）と `resets_at` を突き合わせ、期限切れの窓は表示しない。値には経過時間を添える。
- 用語は Claude 本体の現行呼称に追従（canon §3）: 英語は公式 UI の "Current session" / "Current week"（ヘルプセンターの正式名は "five-hour session limit" / "weekly usage limit"）。公式日本語呼称は 2026-07-02 時点で確認できなかったため、日本語は「セッション（5時間）」「週間」を本仕様で確定し全サーフェスに一貫適用する。

## 仕組み

```
有効化（環境ごとの opt-in、GUI のトグル）
  → <cli-data>/settings.json の statusLine を CSW 生成のスクリプトに向ける
     （既存の statusLine 設定は退避し、スクリプトが元コマンドへ stdin を透過して出力も引き継ぐ）
Claude Code が応答するたび（イベント駆動）
  → スクリプトが stdin JSON を ~/.context-switcher-claude/usage/<環境名>.json に保存
     （環境名は CLAUDE_CONFIG_DIR から導出。導出できないパスでは保存しない）
GUI（60 秒間隔 + フォーカス時のローカル read のみ）
  → usage/<環境名>.json を読み、サイドバーの環境行にバーを描画
```

- ファイル配置: スクリプト = `~/.context-switcher-claude/statusline/<環境名>.sh`、退避 = 同 `<環境名>.original.json`、データ = `~/.context-switcher-claude/usage/<環境名>.json`。
- 既存 statusline が無い環境には「5h 42% / week 13%」形式の簡潔な既定表示を入れる（値の抽出は macOS 標準の plutil を使用。jq に依存しない）。
- 既存の Claude（default = 実ユーザーの `~/.claude`）は既定で触らない。トグルで明示有効化した場合のみ設定し、無効化で復元する。
- `cli_settings` が「共有」の環境は有効化を拒否する（settings.json が既存の Claude と同一実体のため、書き込むと既存の Claude 側も変わる。理由を UI で説明する）。
- 制約の明示: 値の更新は Claude Code の応答時のみ（Desktop だけ使っている時間帯は更新されない）。設定 UI と docs に明記する。

## 変更ファイル

| ファイル | 内容 |
|---|---|
| `crates/core/src/usage/mod.rs`（新規） | 有効化/無効化/状態導出/スクリプト生成/usage JSON の読み出しと鮮度判定 + ユニットテスト |
| `crates/core/src/lib.rs` | `pub mod usage;` 追加 |
| `crates/core/Cargo.toml` | serde_json の `preserve_order`（ユーザーの settings.json のキー順を壊さない） |
| `crates/desktop/src/main.rs` | Tauri コマンド `get_usage_settings` / `set_usage_tracking` / `get_usage_snapshots` |
| `crates/desktop/ui/main.js` `index.html` `style.css` | サイドバーのバー、詳細画面のトグルと説明、60 秒更新、i18n |
| `docs/SPECIFICATION.md` | §3 ディレクトリ構造・§5.A に仕様追記、ゼロインパクト保証との整合を明記 |
| `docs/USER_GUIDE.md` / `USER_GUIDE_EN.md` | 使い方と制約（更新タイミング・Pro/Max 前提）の追記 |
| `website/index.html` / `website/ja/index.html` | 機能の言及（規模は実装後にスキルを当てて判断） |

## 実装ステップ

1. `docs/SPECIFICATION.md` に仕様を追記（本 plan と同時に commit）。
2. RED: `crates/core/src/usage/` にテストを作成し失敗を確認。
   - settings.json が無い環境での有効化（statusLine 設定 + スクリプト生成 + 実行権限）
   - 既存 statusLine の退避とスクリプトへの透過埋め込み
   - `cli_settings = Share` の非 default 環境で拒否
   - 無効化での復元（元設定あり/なし、他キーの保存）
   - 有効化の冪等性（2 回目で退避を上書きしない）
   - 壊れた settings.json（不正 JSON）で非破壊エラー
   - usage JSON の読み出し（期限内/`resets_at` 超過/ファイル無し/rate_limits 欠落）
   - スクリプト実測（macOS）: sh に stdin を渡し、CLAUDE_CONFIG_DIR からの環境名導出・保存・既定表示・元コマンド透過を検証
3. GREEN: core 実装 → desktop backend（Tauri コマンド + devInvoke モック）。
4. UI: minimalist-ui / csw_product_canon / japanese-typography-qa を Read してから、サイドバーのバー（accent 不使用、バー 2 本 vs 1 本はスキルを当てて判断）と詳細画面のトグルを実装。i18n（data-en / EN 辞書 / T()、hidden 要素は `.クラス[hidden]{display:none}` 再表明）。
5. 検証: cargo fmt --check / clippy -D warnings / build / test を実測。GUI は実描画で既定状態（opt-in 前 = バー無し）と有効化後の両方をスクショ確認。
6. docs / LP 同期、スクショに影響があれば scripts/appshot で ja/en 再生成。
7. PR（本 plan へのリンクを冒頭に）→ CI green → squash マージ → release-please の minor リリースを自律実施。

## リスクと対応

- **statusline スキーマの将来変更**: パースは防御的に（欠落フィールドは None）。公式ドキュメント URL を仕様に記録。
- **ユーザーが statusline 設定を手で変更**: 無効化時は「statusLine が CSW のスクリプトを指す場合」のみ書き換える。手動変更されていたら触らずスクリプト等の残骸だけ削除。
- **環境の削除**: delete_profile 時にスクリプト・退避・usage JSON も削除（残骸ゼロ）。
- **Copy 環境の settings.json 同期（FileWatcher）**: FileWatcher は現状どこからも起動されていない休眠コード。かつスクリプトは保存先を CLAUDE_CONFIG_DIR から導出するため、仮に settings.json が環境間コピーされても他環境のデータを汚さない。
- **プラン差異**: rate_limits が来ない（API キー利用・無印プラン）環境ではバーを出さない。「Pro/Max のみ」を docs に明記。

## スコープ外（混ぜない）

- B 案 / C 案（不採用確定）。
- トレイメニューへの利用量表示。
- PAT 導入・カスタムサブドメイン・Pages action の SHA pin（別件の残課題）。

## 完了条件

- [ ] cargo fmt / clippy / build / test が全て green（CI 実測）
- [ ] 有効化 → Claude Code 応答 → バー表示、無効化 → 復元、の実機確認
- [ ] 既定状態（opt-in 前）の実描画確認（v0.16.0 バナー再発防止の手順）
- [ ] docs（SPEC / USER_GUIDE ja・en）と UI 文言の整合、禁止記号 grep ゼロ
- [ ] PR マージ・リリース（minor）・署名/公証 DMG の確認
