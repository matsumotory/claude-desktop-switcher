---
description: 仕様ファースト開発の起動（.agents/skills/core_spec_first_development/SKILL.md）
---

対象機能・変更: $ARGUMENTS

`.agents/skills/core_spec_first_development/SKILL.md` を読み、その手順に厳密に従って仕様ファースト開発を進めてください。

順序厳守：

1. 仕様合意（ユーザーと `docs/SPECIFICATION.md` レベルで合意）
2. テスト作成（RED）
3. 実装（GREEN）
4. エビデンス付き検証

テストの期待値は **`docs/SPECIFICATION.md` から導出** し、現在のコード挙動から導出することは禁止です（core_spec_first_development の絶対ルール）。
