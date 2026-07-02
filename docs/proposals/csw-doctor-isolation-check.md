---
title: csw doctor: 分離と共有リンクの健全性検査
created: 2026-07-03
status: approved
related_prs: []
related_issues: []
---

# csw doctor: 分離と共有リンクの健全性検査

## 背景・本セッションの完了事項

2026-07-03 のサーベイで、プロファイル切替ツール全般に対するユーザーの不満の最頻出は「意図せず混ざった・なぜか分からない」であり、切替機能そのものより「分離が維持されている確証」が信頼のボトルネックだと分かった。既存の Claude 系切替ツールは切替のみで、分離の継続検証を持つものは確認できなかった (認証情報の読み取りを伴う方式の diagnostics を持つものはある)。認証情報に一切触れないファイルシステム層だけの分離検証は、CSW の制約がそのまま差別化になる。

PR #149 (透明性パッケージ) で「CSW が書く場所・読む場所」を docs/PRIVACY.md に確定させた。本機能はその宣言を機械で検証する対になる。ユーザーは 4 案すべての実施を承認済みで、本 plan はその 2 本目である。

## ゴール

環境の宣言 (profile.toml の共有/分離/コピー) と実ディスクの状態を突き合わせ、分離が壊れていないかをワンクリック / 1 コマンドで点検できるようにする。

検査項目 (各環境について):

1. 共有を選んだ項目が、実際に共有元への正しいシンボリックリンクになっているか (リンク切れ・リンク先の相違・実ファイル化ドリフトの検出)。
2. 分離・コピーの項目や、常に分離する 4 項目 (config.json / claude_desktop_config.json / sessions / ant-did) が、誤ってシンボリックリンクになっていないか。
3. 設定の健全性: config.toml の active_profile が実在するか、各 profile.toml がパースできるか。

## 設計原則 (敵対検証で確定した仕様条件)

- **走査範囲の限定 (制約 3 を掠めない)**: 検査が触るのは、CSW 自身の設定ファイル (config.toml / profile.toml) と、linker が管理する固定のリンクポイント (11 項目) だけ。環境データディレクトリの中を readdir で列挙しない。共有元 (既存の Claude 側) はリンク解決先の存在確認 (stat) のみで、中身は読まない。
- **修復は最小限**: 既定は完全 read-only。`csw doctor --fix` (CLI のみ) が行うのは「期待どおりの共有元が実在するのに、リンク先が想定と異なる項目 (リンク切れを含む) の張り直し」だけ。実ファイル化ドリフトは自動修復するとデータ損失になりうるため検出と案内のみ。GUI に自動修復ボタンは置かない (サーベイで見た「自動動作の誤爆」クラスを持ち込まない)。修復先が既存の Claude のディレクトリ配下になる場合は拒否する (linker と同じ防御)。
- **TOCTOU の注記**: その環境の Claude が起動中の場合、アプリの temp+rename 書き込みの一瞬を検査した可能性がある。結果に「起動中」を明示し、問題が出たら Claude を終了して再検査するよう案内する。
- **公式との区別**: Claude Code 本体には接続・インストールを診断する `claude doctor` がある。CSW の doctor は CSW が作った環境の分離状態の検査であり別物であることを docs に一言明記する。
- セキュリティ・プライバシー・倫理を利便性と引き換えにしない。検査は認証情報に触れず、通信せず、既定では何も書き換えない。

## タスク詳細

### 変更ファイル一覧

| ファイル | 変更 |
|---|---|
| `crates/core/src/profile/linker.rs` | 項目の列挙 (パス・種別・常時分離) を共有テーブル `link_items()` に抽出し、link/unlink を同テーブル駆動にリファクタ (挙動不変、既存テストで担保) |
| `crates/core/src/profile/inspector.rs` (新規) | 宣言と実ディスクの突合。ItemHealth (SharedOk / SourceMissing / WrongTarget / Materialized / MissingLink / SourceAbsent / IsolatedOk / UnexpectedLink) と ProfileReport を返す read-only 検査器 + `fix_relinkable` |
| `crates/core/src/profile/tests.rs` ほか | 各状態の検出と --fix の範囲を tempdir で固定するユニットテスト (RED から) |
| `crates/cli/src/main.rs` | `csw doctor [name] [--fix]` サブコマンド (英語出力、問題があれば exit 1) |
| `crates/desktop/src/main.rs` | Tauri command `inspect_profile` |
| `crates/desktop/ui/` | 環境詳細画面に「分離を検査」ボタンと結果表示 (ja/en、devInvoke モック含む) |
| `docs/SPECIFICATION.md` | 検査機能の仕様を完了状態として追記 |
| `docs/USER_GUIDE.md` / `USER_GUIDE_EN.md` | 使い方と、公式 `claude doctor` との違いの一言 |
| `docs/PRIVACY.md` / `PRIVACY_EN.md` | 共有リンクの作成に「検査での張り直し」を追記 (実装と同期の宣言を守る) |
| `website/ja/index.html` / `website/index.html` | 安全性 FAQ に「アプリ内の検査で分離を点検できる」旨を 1 文追記 |
| `website/assets/` スクリーンショット | 詳細画面が変わるため appshot で ja/en 再生成 (キャッシュバスター更新含む) |

### 実装ステップ

1. linker の項目テーブル抽出 (挙動不変リファクタ、既存テスト green を確認)
2. inspector のユニットテストを先に書き RED を確認 → 実装で GREEN
3. CLI サブコマンド + テスト
4. Tauri command + GUI (minimalist-ui / japanese-typography-qa / csw_product_canon のゲートを通す)
5. docs / LP / PRIVACY 同期、スクリーンショット再生成
6. 敵対検証 workflow (事実整合・日本語校正・語彙) → 指摘反映 → CI green → マージ

### 検証計画

```bash
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace && cargo test --workspace
```

- GUI は launch の app (devInvoke モック) で ja/en の実描画確認。
- 実機検証: 手元の実環境で `csw doctor` を実行し、共有リンクを故意に壊して検出・`--fix` の復旧を確認する (実データは変更しない検査であることも lsof 相当の観点で確認)。

## リスクと対応

- **linker リファクタの挙動変化**: 既存の linker テスト 4 本 + profile テスト 36 本を green のまま保つ。テーブルは link_profile の現在の順序・モード解決を厳密に写す。
- **既定環境の誤検査**: 既存の Claude (default) はリンクを持たないため検査対象外とし、明示的に案内する。
- **起動中の偽陽性**: TOCTOU 注記と再検査導線で対応 (上記)。

## スコープ外 (混ぜない)

- 環境インスペクタ (サイズ・中身の表示) は次 plan。
- 実ファイル化ドリフトの自動修復・GUI からの修復。
- watcher モジュール (未配線の dead code) の扱いは別 PR で判断する。

## 完了条件

- [ ] inspector の各状態検出テストが RED から GREEN
- [ ] `csw doctor` / `--fix` が実機で期待どおり動く (壊す → 検出 → 修復の実測)
- [ ] GUI の検査結果が ja/en で実描画確認済み
- [ ] docs / PRIVACY / LP / スクリーンショットの同期完了
- [ ] CI 全ジョブ green、squash マージ済み
