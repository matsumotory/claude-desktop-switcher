# AGENTS.md

Claude Code、Codex、そのほかのAIエージェントと人が、同じ規則で作業します。規則の本文は [CLAUDE.md](CLAUDE.md) にあるので、作業の前に全文を読んでください。

## コミットメッセージ

- [.agents/skills/core_commit_standard/SKILL.md](.agents/skills/core_commit_standard/SKILL.md) の形式 (Conventional Commits、件名と本文は日本語) に従う。本文には、変えたこと、変えた理由 (本人のどの依頼や指摘を受けたか)、確かめたこと (流したテストとコマンドとその結果、確かめていないこと) を書き、件名だけのコミットにしない。
- PR の説明文も同じ内容で書く。まとめて (squash で) マージしたときは、PR の説明文がそのコミットの本文になる。
