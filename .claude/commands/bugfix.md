---
description: バグ修正プロトコル起動（.agents/skills/core_bug_fix_protocol/SKILL.md）
---

`.agents/skills/core_bug_fix_protocol/SKILL.md` を読み、その手順に厳密に従ってバグ修正を進めてください。

順序厳守：

1. **失敗するリグレッションテストを先に書く**（修正前に必ず RED を確認）
2. テスト期待値は blueprint.md から導出（現在のバグ挙動からではない）
3. 実装で GREEN にする
4. 関連するエッジケースのテストも追加

修正後にテストを書くことは禁止です（AGENTS.md 2.1）。
