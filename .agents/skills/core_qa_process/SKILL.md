---
name: core_qa_process
description: CSW (Rust + Tauri v2) の QA・検証プロセスと継続的改善のルールを定義するスキル。コード変更の完了報告前、PR を出す前、機能や修正を「できた」と言う前に発動し、検証順序とエビデンス必須の原則を強制する。
---

＃ QA・検証と継続的改善

CSW (Rust + Tauri v2 / macOS) で開発した機能・修正を品質低下なく本番 (DMG リリース) に乗せるための検証プロセスと、AI 自身が同じミスを繰り返さないための継続的改善ルール。

> [!CAUTION]
> **AI は「できた」「PASS」と報告する前に、自分でエビデンスを確認する義務がある。** core/CLI 変更ならコマンド実行結果、GUI 変更ならアプリ起動 + WebView の挙動を一次情報として確認する。サブエージェントの「PASS」報告を鵜呑みにしてはならない。エビデンスなき PASS は虚偽報告と同等。

## 1. 検証の固定順序 (厳守)

コード変更後は**必ずこの順序で**検証する。前段が落ちたら次に進まず、その場で直す。

```
1. cargo fmt --check
2. cargo clippy --workspace --all-targets -- -D warnings
3. cargo build --workspace
4. cargo test --workspace
5. (GUI 変更時) cargo tauri dev でアプリ起動 + WebView devtools 確認
```

- **fmt → clippy → build → test → アプリ起動** の順を崩さない。整形・lint・型/ビルドの機械的エラーを先に潰してから、振る舞いの検証 (test / 実機) に進む。
- `clippy` は `-D warnings` で警告ゼロが合格条件。`#[allow(...)]` で抑止する場合は理由を `// ALLOW: ` コメントで明記する (modern-rust-workflow 参照)。
- `cargo fmt --all` は実際に整形する操作。検証は破壊しない `cargo fmt --check` で行い、差分があれば整形してから再検証する。

> [!IMPORTANT]
> **cargo はローカル環境で使えないことがある。その場合 CI が検証パスの本体になる。** PR を push し、CI の **Test** (`cargo test --workspace --exclude csw-desktop` = core+cli) と **Build** (`cargo build --workspace` = desktop 含む全クレート) の結果ログを一次エビデンスとして確認する。CI のグリーンを確認せずに「通った」と書かない。

## 2. クレート別の検証観点

| クレート | 検証手段 | 観点 |
|---|---|---|
| `crates/core` (csw-core) | `cargo test --workspace` / CI Test | ロジック・プロファイル隔離・Keychain (MockKeychainProvider) のユニット/結合テスト |
| `crates/cli` (csw) | `cargo test` + 実コマンド実行 | 引数解釈・exit code・標準出力/エラー出力の文言 |
| `crates/desktop` (csw-desktop) | `cargo build --workspace` (CI Build) + `cargo tauri dev` | コンパイル可否 (CI Test からは除外) + 実機でのトレイ/設定 UI 挙動 |

- `csw-desktop` は CI の Test ジョブから除外されている。**desktop のビルド健全性は Build ワークフロー (全ワークスペースビルド) で、振る舞いは `cargo tauri dev` の実機確認でしか担保できない。** test が緑でも desktop が壊れていないことの証明にはならない点に注意。

## 3. GUI 変更時の実機確認 (The QA Agent 視点)

GUI (トレイ / 設定ウィンドウ) を変えたら、`cargo tauri dev` でアプリを起動し、以下を自分で確認する。

- **見た目の確認**: 期待した UI が実際に描画されているか (スクリーンショットを取り、自分で `Read` で開いて確認する)。
- **WebView devtools のコンソールエラーチェック**: WebView 内で右クリック → Inspect Element (または開発ビルドの devtools) を開き、Console にエラー/警告が出ていないか確認する。フロントエンドの例外は build/test では検出されない。
- **トレイ常駐の動作**: メニュー項目・クリック・プロファイル切替が期待どおり反応するか。
- **エビデンスを項目ごとに個別確認する**: 複数箇所を一括修正して「全部 OK」と丸めない。1 項目 1 エビデンス。

## 4. リファクタ・名称変更後の全域確認

名称変更・リファクタリング後は `Grep` でワークスペース全体 (`crates/` / `docs/` / `website/`) を横断検索し、**旧語彙・旧 API の残存がゼロ**であることを確認する。コメント・doc・テスト名・エラーメッセージ・LP の文言まで含めて潰す。

## 5. 継続的改善とフィードバックループ (義務)

発生した問題を単に直すだけでなく、**「なぜ起きたか」「どうすれば AI が同じミスを繰り返さないか」を分析し、規約・スキルへ還元する**。

1. **問題解決後の振り返り**: 複雑な不具合解決や大規模修正の完了後、得られた知見を抽出する。
2. **ルールの言語化**: 新たなアンチパターンを見つけたら `.agents/skills/` を更新する (または `.agents/AGENTS.md` に追記)。直す前に「汎用化できる教訓か」を毎回明示的に判断する。
3. **知識の代謝**: 定期的にルールを抽象化・圧縮する。SKILL.md は 1 ファイル 80 行以内を目安に保つ。
4. **人間の承認ゲート**: 仕様確定 (`docs/SPECIFICATION.md`)、`main` へのマージ、`#[allow(...)]` 追加など不可逆/方針が割れる分岐は、推奨を添えて人間の Approve を求める。
5. **自己進化**: 各タスクフェーズの終わりに、検証プロセスの非効率やルールの穴がないか自己評価し、改善案を提示する。

## 6. 絶対に譲らない原則

- **セキュリティ・プライバシー・倫理を利便性と引き換えにしない。** 検証を急ぐために Keychain・プロファイル隔離の確認を飛ばさない。
- **コマンド出力を切り詰めない。** 「…(略)」で要約せず、失敗箇所・テスト結果は全文を確認する。途中で truncate して PASS と判断しない。
- **検証の偽装をしない。** スクリーンショットに修正が反映されていない / サブエージェントの「確認した」を転記しただけ、は虚偽。AI 自身が一次情報を開いて確かめる。

## 関連スキル

- `core_spec_first_development` (`/spec-first`): 仕様合意 → テスト(RED) → 実装(GREEN) → エビデンス付き検証の順序。
- `core_bug_fix_protocol` (`/bugfix`): リグレッションテストファースト、プロファイル隔離・並列安全性の検証観点。
- `core_pr_review_cycle` (`/code-review`): lint/test 全パス後の PR レビューサイクルとマージ基準。
- `modern-rust-workflow`: clippy `-D warnings`・エラーハンドリング・ワークスペース運用の標準。
