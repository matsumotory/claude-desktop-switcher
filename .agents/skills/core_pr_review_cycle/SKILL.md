---
name: pr_review_cycle
description: PR完成時にGemini Code Assistでレビューし、指摘修正→再レビューのサイクルを回すプロトコル。
---

# PR Geminiレビューサイクル

## 発動条件

- 実装が完了し、lint/typecheck/test全パス後のPRプッシュ時

## コマンド判断フロー（最重要）

```
コード変更をプッシュした？
├─ Yes → 修正報告 + `/gemini review` を1コメントで投稿
└─ No（指摘への返答・棄却理由の説明のみ）
       → `/gemini [説明内容]` で投稿（review は絶対に付けない）
```

| コマンド | 用途 | いつ使う |
|---|---|---|
| `/gemini review` | コードレビュー依頼 | **コード変更をプッシュした後だけ** |
| `/gemini [内容]` | 質問・返答・棄却理由の説明 | レビュー指摘に対する議論時 |

❌ **アンチパターン**: レビュー指摘の棄却理由を説明するだけなのに `/gemini review` を送る → 不要なレビューが走り、同じ指摘が繰り返される

## プロトコル

### 1. 既存レビューの確認と初回レビュー依頼

> [!WARNING]
> **多重レビューの防止**
> レビュー依頼を連続して投げると重複してレビューが実行されるため、依頼前に必ず `gh pr view --comments` や `gh api .../comments` などですでに未対応のレビューが存在しないか確認すること。未対応のレビューがあるまま追加で依頼することは厳禁。

```bash
gh pr comment <PR番号> --body "/gemini review"
```

### 2. レビュー結果の確認

```bash
gh api repos/<owner>/<repo>/pulls/<PR番号>/reviews \
  --jq '.[] | select(.submitted_at > "TIMESTAMP") | .body'
gh api repos/<owner>/<repo>/pulls/<PR番号>/comments \
  --jq '.[] | select(.created_at > "TIMESTAMP") | {path, line, body}'
```

### 3. 指摘への対応

1. 各指摘を分析し、正当な指摘は修正する
2. **仕様を変えずにテストを変えてはならない**（テスト失敗→実装で対応）
3. 修正後は `lint → typecheck → test` を全実行
4. `Co-authored-by: gemini-code-assist` を付与

### 4. 再レビュー依頼（コード修正した場合）

> [!IMPORTANT]
> **レスポンスコメントとレビュー依頼は必ず別コメントにすること。** 1つにまとめるとGeminiはレスポンス内容を無視してレビューのみ実行してしまう。

**Step 1: 修正内容の報告（@メンション付き）**

```bash
gh pr comment <PR番号> --body "@gemini-code-assist レビュー指摘を修正しました。
- [修正内容の説明]
- [棄却した指摘とその理由]"
```

**Step 2: レビュー依頼（別コメント）**

```bash
gh pr comment <PR番号> --body "/gemini review"
```

### 5. 指摘への返答・棄却（コード修正なし）

```bash
gh pr comment <PR番号> --body "/gemini [棄却理由や質問内容]"
```

### 6. サイクルの終了とマージ基準

**指摘レベル別の対応ルール:**

| レベル | 対応 |
|---|---|
| **Critical** | 必ず修正。修正後に再レビュー必須 |
| **High** | 必ず修正。修正後に再レビュー必須 |
| **Medium** | 内容が妥当であれば修正する |
| **Low** | 判断して対応 or `/gemini` で棄却理由を報告 |

**マージ判定フロー:**
1. Critical/High指摘が0件になるまでサイクルを繰り返す
2. Medium指摘を対応し、CI（lint/typecheck/test）がパスしたら**マージ可能**
3. マージ前に必ず**ユーザーの承認**を得る（`main` への直接マージ禁止ルール）
