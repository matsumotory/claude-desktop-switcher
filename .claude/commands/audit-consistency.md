---
description: ドキュメント(LP含む)↔実装の整合性監査を起動（.agents/skills/docs_impl_consistency_audit/SKILL.md）
---

監査対象(任意の絞り込み): $ARGUMENTS

`.agents/skills/docs_impl_consistency_audit/SKILL.md` を読み、その手順に従って **LP(website/) ↔ ドキュメント(docs/・README) ↔ 実装(crates/)** の整合性を監査してください。

進め方:

1. まず自分で対象サーフェスを列挙し、CLI の clap サブコマンド一覧・正典用語・機能リストを集める。
2. スキルの7ディメンション(用語正典 / アーキテクチャの真実 / CLI表面 / 機能主張 / ja-en整合 / バージョン導入 / 禁止表現・トーン)を Workflow で並列監査し、各 finding を敵対的に検証する(`core_ai_workflow`)。
3. severity(blocker / major / minor)別に、`surface / file:line / 主張 / 実装の事実 / 修正案` の表で報告する。

これは **read-only 監査**です。修正の適用はユーザー合意の上で別 PR で行い、変更したら全サーフェスへ伝播し旧表現を全リポ grep で残存ゼロ確認してください。
