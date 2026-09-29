---
title: 複製が分離データを写さず共有リンク先を取り違える不具合の修正
created: 2026-09-29
status: in-progress
pr: 194
related_prs: [192]
related_issues: []
---

# 複製が分離データを写さず共有リンク先を取り違える不具合の修正

## 背景・本セッションの完了事項

issue #190 の対応中に、複製 (`ProfileManager::clone_profile`) の 2 つの取りこぼしを実測した。

1. linker 管理の項目を汎用コピーから除外して `Linker::link_profile` に任せているが、`apply_link` の Isolate 分岐は空ディレクトリを作るだけなので、「すべて分ける」環境を複製すると `projects/` (会話履歴と自動メモリ)、`CLAUDE.md`、`settings.json`、`history.jsonl`、`skills/`、`plugins/` の実データが写らない。
2. 共有リンクを `link_profile(&target, &source_env)` で作るため、複製先のリンクが宣言した共有元 (既存の Claude) ではなく複製元環境のリンク点を指す。複製直後に分離の検査が WrongTarget を報告し、複製元をゴミ箱へ移すとリンクが切れる。

docs は 3 者で不一致だった (USER_GUIDE ja: 構成を引き継ぐ / PRIVACY: フォルダ全体をそのまま写しサインイン済みで始まる / USER_GUIDE_EN: isolated components start empty)。git 履歴に複製の設計記録は無く (`fcb503f` で実装ごと入った)、「安全側に倒した意図的な設計」の根拠は見つからなかった。

ユーザー (matsumotory 2026-09-29) の判断: 推奨案で進める。

## ゴール

- 複製は「常に分離する項目とプラグインを除き、環境の設定とデータをそのまま写す」。
- 共有リンクは宣言した共有元 (既存の Claude) を指す。
- docs (SPEC / USER_GUIDE ja,en / PRIVACY ja,en) と UI 文言を実装に合わせて統一する。

## 設計原則

- `LINK_ITEMS` を唯一の真実にする: 汎用コピーの除外もハードコードの名前一覧ではなく `LINK_ITEMS` から導く (#190 で `rules/` が増えても追随する)。
- 干渉を作らない: 写すのは複製後に完全に独立する実データだけ。`plugins/` は Claude Code の `installed_plugins.json` / `known_marketplaces.json` が絶対パスで複製元のキャッシュを指す (公式ドキュメント 2026-09-29 時点で確認) ため複製元からは写さず、宣言モードどおり共有元から用意する。
- 常に分離する項目 (サインイン情報・コネクタ設定・端末 ID・セッション状態) は作成時と同じく新規に始める。
- 主張の強度は根拠に合わせる: 複製先がサインイン済みで始まるかは Linux では検証できないため、docs は「サインインを求められることがあります」に留める。

## タスク詳細

| ファイル | 変更 |
|---|---|
| `crates/core/src/profile/mod.rs` | `clone_profile` を 3 段 (linker 非管理の実データを写す / Copy・Isolate の項目を複製元から写す (plugins 除く) / 宣言した共有元へ link) に書き直す |
| `crates/core/src/profile/tests.rs` | RED テスト 3 本 (分離とコピーの項目が写り共有が既定を指す / 完全分離環境の会話・ルール・設定が残る / コピーのプラグインは既存の Claude から用意し直す) |
| `crates/desktop/ui/main.js` | 複製の説明文を「設定とデータをそのまま写す」に (ja/en) |
| `docs/SPECIFICATION.md` / `docs/USER_GUIDE.md` / `docs/USER_GUIDE_EN.md` / `docs/PRIVACY.md` / `docs/PRIVACY_EN.md` | 複製の記述を統一 |
| `website/assets/*.png` | UI 文言が変わるので `Regenerate screenshots` ワークフローで再生成 |

## 検証計画

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
```

## リスクと対応

- 複製先のディスク使用量が会話履歴の分だけ増える。docs に「データをそのまま写す」と明記する。
- 作成時の Copy モードのプラグインも `~/.claude/plugins` の絶対パスを持ち込む (既存の挙動)。本 PR の範囲外として記録する。

## スコープ外 (混ぜない)

- 作成時の Copy モードのプラグインの絶対パス問題。
- 複製先がサインイン済みで始まるかの macOS 実機検証 (ユーザーが実施)。

## 完了条件

- [x] RED → GREEN
- [x] docs / UI 文言の統一
- [ ] スクショ再生成と LP の実寸更新
- [ ] CI 全緑、squash マージ
