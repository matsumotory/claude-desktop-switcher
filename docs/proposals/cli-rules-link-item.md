---
title: ルールファイル rules/ を共通ルールとして引き継ぐ (issue #190)
created: 2026-09-29
status: in-progress
pr: 192
related_prs: [193]
related_issues: [190]
---

# ルールファイル `rules/` を共通ルールとして引き継ぐ

## 背景・本セッションの完了事項

issue #190 (v0.24.0 / macOS) の報告: 「アカウントだけ分ける」の説明は「共通ルール」を引き継ぐと言うのに、`~/.claude/rules/` が新しい環境に引き継がれない。linker の `LINK_ITEMS` に `rules/` の項目が無く、「共通ルール」の実体は `~/.claude/CLAUDE.md` (`cli_claude_md`) だけだった。

Claude Code の公式ドキュメント (code.claude.com/docs/en/memory、2026-09-29 時点) は、`~/.claude/rules/` の個人ルールを「マシン上のすべてのプロジェクトに適用する」ユーザーレベルのルールとして定義し、`~/.claude/CLAUDE.md` と同じユーザースコープのメモリファイルとして扱っている。共通ルールを `rules/` に分けて置いている利用者は、モードの説明を信じて環境を作ると、その全部を黙って失う。

本セッションで実装まで完了した (本 plan は PR に同梱)。

## ゴール

- `~/.claude/rules/` を `CLAUDE.md` と同じ「共通ルール」の一部として、共有 / 分離 / コピーで扱う。
- 「アカウントだけ分ける」「会話とメモリも分ける」では `rules/` を共有し、「すべて分ける」では分離する。
- 既存の環境 (v0.24.0 以前に作ったもの) が、再作成なしでリンクを得られる経路を用意する。

## 設計原則

- `LINK_ITEMS` を唯一の真実に保つ: `cli_rules` を `cli_claude_md` の直後に足すだけで、linker・分離の検査・データの内訳・GUI の行がすべて追随する。
- セキュリティ・プライバシーを利便性と交換しない: 追加するのはユーザー自身が書くルールファイルの共有だけで、常に分離する項目 (サインイン情報・コネクタ設定・セッション状態・端末 ID) は変えない。
- 既存 profile.toml の後方互換: `cli_rules` キーが無ければ `cli_claude_md` と同じモードとして読む。モードの説明が約束していた意味 (共通ルールを引き継ぐ) をそのまま保つ。明示キーがあればそれを優先する。
- 自動修復はしない: 既存環境のリンク作成は `csw doctor --fix` の明示操作に限る。作るのは「何も無い場所へのリンク」だけで、実体には触れない。

## タスク詳細

### 変更ファイル一覧

| ファイル | 変更 |
|---|---|
| `crates/core/src/profile/mod.rs` | `SharingConfig.cli_rules` を追加。既定は Isolate、`share_settings` / `share_workspace` プリセットは Share、既存の Claude は Share。`is_fully_isolated` と複製時の linker 管理項目一覧に追加 |
| `crates/core/src/profile/linker.rs` | `LINK_ITEMS` に `cli_rules` (`rules/`、ディレクトリ) を `cli_claude_md` の直後に追加 |
| `crates/core/src/profile/config.rs` | `load_profile` で `cli_rules` キーが無い場合に `cli_claude_md` を引き継ぐ |
| `crates/core/src/profile/inspector.rs` | `--fix` が「リンクの欠落」(共有元が実在し、リンク位置に何も無い) も作るようにする |
| `crates/core/src/profile/tests.rs` / `config.rs` / `tests/integration_test.rs` | RED テスト: 項目の存在、各モードでの作成結果、後方互換の読み込み、欠落リンクの検出と作成 |
| `crates/desktop/src/main.rs` | `get_profile_details` の JSON と `build_sharing_config` の上書きに `cli_rules` |
| `crates/desktop/ui/main.js` | 「ルールファイル」の行 (ja/en)、プリセット、dev モック、欠落リンクの案内文 |
| `crates/cli/src/main.rs` | `csw profile show` の行、`csw doctor` のラベルと `--fix` の対象 |
| `docs/SPECIFICATION.md` / `docs/USER_GUIDE.md` / `docs/USER_GUIDE_EN.md` / `docs/PRIVACY.md` / `docs/PRIVACY_EN.md` | 用語表・モード表・分離の検査・`--fix` の範囲・FAQ を更新 |
| `website/assets/*.png` | UI に行が増えるので 8 枚を再生成 (macOS ランナーの `Regenerate screenshots` ワークフロー) |
| `.github/workflows/appshot.yml` | macOS ランナーで `scripts/appshot/gen-screenshots.mjs` を実行し、指定ブランチへ push する手動ワークフロー。Linux のクラウドセッションでは出荷スクショを忠実に再生成できないため |

### 実装ステップ

1. RED テストを書き、`cargo test` でコンパイルエラー (フィールド無し) を確認する。
2. core に `cli_rules` を通し、GREEN にする。
3. desktop / cli / docs / LP スクショへ伝播する。
4. `cargo fmt --check` → `clippy -D warnings` → `build` → `test` → 禁止記号 grep → スクショ再生成 → PR。

### 検証計画

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
```

CI: Test / Build / Lint / Deny / Security / Verify screenshots が緑。

## リスクと対応

- 既存環境の `rules/` が「共有」と表示されるのにリンクが無い: 分離の検査が「共有のリンクがありません」と案内し、`csw doctor --fix` が作る。USER_GUIDE の FAQ に手順を書く。
- 手動でリンクを張っていた利用者 (issue の回避策): 宣言 Share + 正しいリンク先なので、検査は「正常に共有」と判定する。
- Cowork セッション: Claude Code のドキュメントは、デスクトップの Cowork セッションでは、作業ディレクトリの外を指すシンボリックリンクの `~/.claude/rules/` を読み飛ばすと明記している。同じ注意書きはシンボリックリンクの `~/.claude/CLAUDE.md` にも当たるので、`rules/` だけの新しい制約ではない (共有モード全体の既知の限界。docs への明記は別 PR で扱う)。

## スコープ外 (混ぜない)

- Cowork セッションでの共有リンクの扱いを docs に書き足すこと。
- GUI からのリンク修復 (仕様どおり CLI の `--fix` に限る)。
- 複製 (clone) 機能の挙動見直し。

## 完了条件

- [x] RED → GREEN (core 84 テスト、うち新規 8)
- [x] desktop / cli / docs ja,en / PRIVACY ja,en に伝播
- [x] スクショ 8 枚を macOS で再生成し LP の実寸と `?v=` を更新 (PR #193 のワークフローで再生成)
- [ ] CI 全緑、squash マージ
