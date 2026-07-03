---
title: 更新後の乗り移り検知と案内
created: 2026-07-03
status: approved
related_prs: []
related_issues: []
---

# 更新後の乗り移り検知と案内

## 背景

Claude Desktop の更新機構 Squirrel.Mac は、更新後の自動再起動でアプリを引数なしで開き直す。実機の更新ログ `~/Library/Caches/com.anthropic.claudefordesktop.ShipIt/ShipIt_stderr.log` に、2026-06-27 以降の 3 回の更新すべてで引数なしの再起動が記録されている。CSW がプロファイルの引数付きで起動した Claude も、更新の再起動で既定のデータ領域、つまり既存の Claude のデータで開き直される。見た目は同じウィンドウのまま中身だけ変わるため、気づかず別のアカウントで作業を続ける取り違えにつながる。2026-07-03 にユーザーが問題を指摘し、サポートの必要性を確認済み (案 5)。

前提の訂正: 引数なしの Claude を既定のデータ領域として扱う判定は `desktop_dir_running` に実装済みで、トレイと一覧の「利用中」は乗り移り後も既存の Claude を正しく指す。本 plan の対象は「遷移そのものの検知と、ユーザーへの案内」だけである。

## ゴール

- ある環境の Claude が消え、直後に CSW を経由しない Claude が現れた遷移を検知し、設定ウィンドウに案内バナーを表示する。文言は、更新のあとの自動再起動でこうなることがあること、いまの Claude は既存の Claude のデータで動いていること、元の環境で続けるには Claude を終了してから切り替えて起動すること。
- バナーには元の環境への切り替えボタンを添える。押すと既存の切替フローに入り、起動中は従来どおり終了の案内が出る。CSW から Claude を終了させることはしない (2026-07-03 のユーザー判断)。
- 誤検知を抑えるため、遷移の時間窓を短くする (環境の Claude が消えてから約 12 秒以内に CSW を経由しない Claude が現れたときだけ)。ユーザーが自分の意思で Claude を Dock から開き直した場合も文面が事実として成立するよう、可能性の表現で書く。

## 対象ユーザーと規模

環境を使う全ユーザー。Claude の更新は頻繁で、ユーザー自身が「よく再起動して更新がくる」と確認している。実装はコアの判定関数 1 つ、デスクトップの監視ループへの数十行、バナー 1 枚で、対象の広さに見合う軽さに収まっている。

## 設計

- 判定の材料は既に読んでいるプロセスの起動引数だけ。新しい読み取りも書き込みも権限もない。
- `unmanaged_default_running(args)`: 主プロセスの行に `--user-data-dir` が無いものがあるか (= CSW を経由しない起動)。core の純関数としてテスト付きで置く。
- 遷移検知はトレイの 3 秒監視ループ (Rust) に置く。「既定以外の環境が利用中だった直近時刻」を覚え、その環境が消えて 12 秒以内に CSW を経由しない Claude が現れたら、乗り移りイベント (元の環境名と時刻) を AppState に記録する。イベントは、閉じる操作、または CSW を経由しない Claude の終了で消える。ディスクには書かない (揮発でよい。CSW 再起動後は遷移自体が観測できないため出さない)。
- WebView は既存のフォーカス時再検証と同じ経路でイベントを取得し (`get_takeover_notice`)、バナーを表示する。閉じるは `dismiss_takeover_notice`。
- 自動切替・自動終了は作らない。バナーは案内と、既存の切替フローへの入口だけ。

## タスク詳細

### 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `crates/core/src/switcher/mod.rs` | `unmanaged_default_running` (純関数 + テスト) |
| `crates/desktop/src/main.rs` | 監視ループの遷移検知、AppState のイベント保持、`get_takeover_notice` / `dismiss_takeover_notice` |
| `crates/desktop/ui/main.js` | バナー表示 (フォーカス時再検証に載せる)、日英文言、QA 用モック |
| `crates/desktop/ui/index.html` / `style.css` | バナー要素 (DMG バナーと同じ型) |
| `docs/SPECIFICATION.md` | §5.A に乗り移り検知の仕様を追記 |
| `docs/USER_GUIDE.md` / `USER_GUIDE_EN.md` | FAQ「Claude Desktop の更新のあと、環境から外れてしまうことはありますか？」を追加 |

### RED テスト一覧 (期待値は本仕様から導出)

- `unmanaged_default_running`: 引数なしの主プロセス行があれば true / すべて `--user-data-dir` 付きなら false / 空なら false

### 実装ステップ

1. RED: core のテストを書き、失敗を確認する
2. GREEN: core 実装 → desktop の検知とコマンド → バナー UI
3. モックで遷移を再現し、日英の実描画でバナーと切替導線を確認する
4. docs 伝播 → 敵対検証 → commit / PR / CI green

### 検証計画

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
```

GUI はモックの遷移シナリオで、バナーの表示・閉じる・切り替えボタン・状態変化での自動消滅を実描画で確認する。可能なら実機で、環境の Claude を終了して素の Claude を 12 秒以内に開き、実遷移でバナーを確認する。

## リスクと対応

- 誤検知: ユーザーが意図して素の Claude を開いた場合も条件を満たし得る。時間窓を 12 秒に絞り、文面を可能性の表現と事実 (いまの Claude は既存の Claude のデータで動いている) だけで構成し、誤検知でも嘘にならないようにする。
- 検知漏れ: CSW が動いていない間の遷移は観測できない。バナーは補助であり、恒常的な状態表示 (利用中が既存の Claude を指す) は既存機能が担う。

## スコープ外 (混ぜない)

- CSW からの Claude の終了 (ユーザー判断で恒久見送り)
- 現在地の確認 (案 4、別 PR)

## 完了条件

- [ ] RED から GREEN にしたテストが CI で green
- [ ] モック遷移でバナーの表示・閉じる・切替導線・自動消滅を実描画で確認 (日英)
- [ ] docs ja/en 伝播、japanese-copy-discipline の出荷前チェックを実測で通過
