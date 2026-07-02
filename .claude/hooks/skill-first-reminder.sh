#!/usr/bin/env bash
# PreToolUse (Edit|Write|MultiEdit) skill-first reminder.
#
# When about to edit a design / Japanese-copy / documentation surface, inject a
# non-blocking reminder so the Skill-First Gate and cross-surface propagation are
# consulted every time — not skipped as a "small change" and fixed only after the
# user points it out. Fires only for the surfaces below; other edits pass silently.
f="${CLAUDE_TOOL_INPUT_FILE_PATH:-}"
case "$f" in
  *website/*|*/docs/*|*README*|*/crates/desktop/ui/*|*.html|*.css|*.md) ;;
  *) exit 0 ;;
esac
cat <<'JSON'
{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":"[Skill-First ゲート] デザイン/日本語コピー/ドキュメントのサーフェスを編集しようとしています。編集前に確認すること: (1) 該当スキルを Read したか。視覚は minimalist-ui / design-taste-frontend、日本語コピー・タイポは japanese-typography-qa (§5: 後付けカッコ説明をしない・説明を先に述べてから用語を出す・前から読める語順・em-dash/米印/絵文字禁止。和文の丸括弧は 識別子/略語導入/フォームの必須任意マーク の 3 種以外禁止: 補足・例示・推奨・分類を丸括弧に入れない、UI ラベルを「名詞（補足）」形にしない、括弧の入れ子禁止、「何の」が曖昧な名詞 (利用量・容量等) 禁止)、整合は docs_impl_consistency_audit。(2) この変更を全サーフェスへ伝播するか (アプリ UI・トレイ・docs ja/en・README ja/en・LP ja/en・スクショ。propagate-changes-to-all-surfaces)。(3) 完了前に旧表現を全リポ grep でゼロ確認し、UI が変わったら scripts/appshot でスクショ再生成、/audit-consistency を自分で回す。指摘される前にやる。"}}
JSON
