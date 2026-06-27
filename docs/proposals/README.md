# Implementation Plans (Proposals)

セッション間のバトンパス用 Implementation Plan を置くディレクトリ。運用は [`core_session_handoff`](../../.agents/skills/core_session_handoff/SKILL.md) を参照。

- ファイル名: `<kebab-case-slug>.md`（日付はファイル名に付けず frontmatter と git 履歴で持つ）
- 各 plan の先頭に frontmatter（`title` / `created` / `status` / `pr`）を必須で付ける
- `status` は `draft → approved → in-progress → completed / rejected` で遷移させ、各段階で commit する
- Plan は「次に何をやるか（前方参照）」、`docs/SPECIFICATION.md` は「何が完成したか（完了状態の正典）」。役割を混同しない
- Plan も実装と同じ feature ブランチ + PR 経由で更新する（`main` 直 push 禁止）
