---
title: 前面の Claude の環境を確認する導線
created: 2026-07-03
status: in-progress
pr: 165
related_prs: []
related_issues: []
---

# 前面の Claude の環境を確認する導線

## 背景

完全分離環境の同時起動では、見た目のまったく同じ Claude ウィンドウが並ぶ。トレイの「利用中」はどの環境が動いているかまでは答えるが、「いま前面にあるこのウィンドウはどれか」には答えられない。プロファイル系ツールで最大の不満が「いま自分がどちらにいるか分からない」であることは 2026-07-03 のサーベイで確認済みで、PID と環境の対応を知っているのは CSW だけという独自性がある。案 4 としてユーザー承認済み。

検討時の条件に従い、本 plan は段階 1 だけを実装する。常時表示の名札と透過オーバーレイの表示は、追従品質の実測が要るため段階 2 として見送る。段階 1 も、独立した通知風ウィンドウは作らず、既存の設定ウィンドウとトーストで答える形に絞る。

## ゴール

- トレイに「いまの環境を確認」を追加する。押すと、前面にある Claude がどの環境かを判定し、設定ウィンドウでその環境を選択してトーストで答える。
- 応答は 4 種とする。環境あり、前面が Claude でない、Claude だがどの環境でもない、設定ウィンドウ自身が前面 (確かめたい Claude をクリックしてから再試行するよう案内する)。
- 前面のアプリの識別子を読むのはこの操作のときだけとする。常時の監視には使わない。

## 対象ユーザーと規模

完全分離環境を並べて使う全ユーザー。トレイ項目 1 つ・コアの判定関数 1 つ・トーストの応答 4 種で、独立ウィンドウや新しい権限を持ち込まない。

## 設計

- 判定の順序が重要で、トレイのメニュー操作は前面アプリを変えないため、メニュー項目が押された時点の前面アプリを読んでから設定ウィンドウを表示する。
- platform に `frontmost_claude_args` を追加する。NSWorkspace で前面アプリの PID を取り、それが Claude の主プロセスなら起動引数の行を返す。前面が Claude でなければ None。読むのはプロセスの識別子と起動引数だけで、画面の内容やウィンドウのタイトルは読まない。
- core の純関数 `environment_for_args_line(line, envs, default_dir)` が起動引数の行を環境名に解決する。`--user-data-dir` が環境のデータディレクトリに一致すればその環境、引数なしは既定のデータ領域つまり既存の Claude、どれにも一致しなければ None。テストはこの純関数に置く。
- desktop はトレイの menu event で判定し、設定ウィンドウを表示してから WebView へイベントで結果を渡す。WebView は該当環境を選択してトーストを出す。
- 依存は objc2-app-kit 0.3 を core の macOS 専用依存に追加する。tauri が同じ版を既に使っており、ネットワーク機能は無い。

## タスク詳細

### 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `crates/core/Cargo.toml` | macOS 専用依存に objc2-app-kit を追加 |
| `crates/core/src/platform/mod.rs` / `macos.rs` / `mock.rs` | `frontmost_claude_args` の追加 |
| `crates/core/src/switcher/mod.rs` | `environment_for_args_line` (純関数 + テスト) |
| `crates/desktop/src/main.rs` | トレイ項目「いまの環境を確認」、判定してから設定ウィンドウ表示、WebView へのイベント送出 |
| `crates/desktop/ui/main.js` | イベント受信、該当環境の選択とトースト、QA 用フック |
| `docs/SPECIFICATION.md` | §5.A に現在地の確認を追記 |
| `docs/PRIVACY.md` / `PRIVACY_EN.md` | 読む場所に「前面のアプリの識別子。確認操作のときだけ」を追記 |
| `docs/USER_GUIDE.md` / `USER_GUIDE_EN.md` | FAQ「いま前面にある Claude がどの環境か確かめられますか？」を追加 |

### RED テスト一覧 (期待値は本仕様から導出)

- `environment_for_args_line`: 環境のデータディレクトリに一致する行はその環境名 / 引数なしの行は既定のデータ領域の環境 / どのディレクトリにも一致しない行は None / 似た接頭辞のディレクトリを誤って一致させない

### 実装ステップ

1. RED: core のテストを書き、失敗を確認する
2. GREEN: core 実装 → platform → desktop のトレイとイベント → WebView
3. モックで 4 種の応答 (環境あり・Claude でない・どの環境でもない・設定ウィンドウ自身が前面) を実描画で確認する
4. docs 伝播 → 敵対検証 → commit / PR / CI green

### 検証計画

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
```

トレイからの実操作は WebView のモックでは再現できないため、判定ロジックは core のテストで、WebView の表示はモックのフックで確認する。

## リスクと対応

- 前面アプリの読み取りはプライバシーに関わる新しい読みなので、確認操作のときだけ読む設計にし、PRIVACY へ明記する。
- 設定ウィンドウの表示自体が前面を CSW に変えるため、判定は必ず表示の前に行う。この順序をコードのコメントで固定する。

## スコープ外 (混ぜない)

- 常時表示の名札と、Claude のウィンドウに重ねる透過オーバーレイ (段階 2。追従品質の実測が前提)
- グローバルホットキー (既定未割り当ての方針のため、需要が見えるまで作らない)

## 完了条件

- [ ] RED から GREEN にしたテストが CI で green
- [ ] 4 種の応答を実描画で確認 (日英)
- [ ] docs ja/en 伝播、japanese-copy-discipline の出荷前チェックを実測で通過
