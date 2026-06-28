---
name: docs_impl_consistency_audit
description: LP(website/) と人間向けドキュメント(docs/・README) と実装(crates/) が「同じ一つの事実」を語っているかを横断監査する。用語・アーキテクチャ・CLI 表面・機能主張・ja/en 整合・禁止表現を点検し、不整合を file:line と修正案つきで報告する。リリース前やドキュメント/LP/実装を変えた後に実行する read-only 監査。
---

# Docs ↔ Implementation Consistency Audit

> LP・ドキュメント・実装が食い違うと、ユーザーは「書いてあることが嘘」になる。
> このスキルは website / docs / README / crates を横断し、**主張(コピー)が実装の事実と一致しているか**を機械的に点検する。
> 既定は **read-only**(発見の報告のみ)。修正は別タスク/別 PR で、合意の上で行う。
> 一次正典: [docs/SPECIFICATION.md](../../../docs/SPECIFICATION.md)、AGENTS.md §6(Copywriting の論理整合)、§7、メモリ `csw-canonical-vocabulary` / `csw-positioning-desktop-suite` / `propagate-changes-to-all-surfaces-and-grep`。

## いつ実行するか
- リリース前、または LP・docs・実装のいずれかを変えた後(`/audit-consistency`)。
- 用語・アーキテクチャ表現・CLI コマンド・機能主張を変えたとき(伝播確認)。

## 監査対象サーフェス
- `website/index.html`(EN LP) / `website/ja/index.html`(JA LP) / `website/style.css`
- `docs/SPECIFICATION.md`(正典) / `docs/USER_GUIDE.md`(JA) / `docs/USER_GUIDE_EN.md`(EN) / `README.md`(あれば ja/en)
- `crates/core`(ロジック・OSパス・Keychain・環境/シンボリックリンク) / `crates/cli`(clap のサブコマンド・フラグ) / `crates/desktop`(Tauri コマンド・トレイ)

## 監査ディメンション(各を独立に点検)

1. **用語の正典(Terminology canon)**
   - ユーザー向け呼称は「環境」で確定(プロファイルでない / `csw-canonical-vocabulary`)。「既存のClaude/利用中」等の正典語が全サーフェスで一致。
   - 旧語・ブレを全リポ grep で残存ゼロ確認(JA/EN 両方)。例: 古い "プロファイル" のユーザー向け使用、"コンテキスト" の混在、他社エージェント名(Antigravity/Gemini/Jules)の残骸。

2. **アーキテクチャの真実(Architecture truth)**
   - CSW は **2つの別ツール**(Claude Desktop App = GUI、Claude Code = CLI)を、共通の環境(プロファイル+Keychain)基盤で隔離/共有する。一方を他方の機能と書かない(AGENTS §6.1)。
   - 何が隔離され(ログイン/履歴等)、何が共有されるかの記述が、`crates/core` の実装(シンボリックリンク/Keychain 操作の対象)と一致。

3. **CLI 表面(CLI surface)**
   - docs/LP が言及する `csw` のサブコマンド・フラグ・例(`eval $(csw env)` 等)が `crates/cli` の clap 定義に**実在**する。
   - 逆に、主要コマンドがドキュメント化されている(実装にあるのに未記載の重要機能がない)。

4. **機能主張(Feature claims)**
   - LP/docs の機能主張(Cowork/Design/Artifacts/Projects/Code 連携、メニューバー管理、ターミナル統合)が実際の挙動と一致。
   - 矛盾するワークフローを書かない(AGENTS §6.2: 端末コマンドが要るのに「設定不要・メニューバーだけで完結」等の包括主張をしない)。

5. **ja/en 整合(Localization parity)**
   - `website/ja` ↔ `website/index.html` が構造的に整合(AGENTS §1: 構造/レイアウト変更は両言語同時)。
   - `USER_GUIDE.md` ↔ `USER_GUIDE_EN.md` が主張・手順で一致。一方だけ古い記述がない。

6. **バージョン・導入手順(Version/install)**
   - ダウンロードリンク・インストール手順・バージョン参照が一貫し最新。外部ドキュメントCTAは GitHub 等の外部URL(内部アンカー #guide 不可、AGENTS)。

7. **禁止表現・トーン(Banned patterns)**
   - ユーザー可視テキストに ※ / ＊ / 絵文字 / em-dash(—)を使っていない。
   - 生の技術用語(Application Support / Keychain 等)を、利点に翻訳せず露出していない(AGENTS Copywriting)。誇張形容詞("perfect/seamless/smart")なし。
   - 日本語タイポは [[japanese-typography-qa]] のチェックリストにも適合(見出しスケール逆転・折り返し崩れがない)。

## 実行手順(Workflow でファンアウト)

横断監査は多観点なので、`core_ai_workflow` に従い Workflow で並列化する:

1. **列挙(自分で)**: grep/Glob で対象サーフェスと、CLI の clap サブコマンド一覧(`crates/cli`)、正典用語、機能リストを先に集める。これを各エージェントに渡す。
2. **並列監査(pipeline)**: 上の 7 ディメンションを 1 エージェント1観点で走らせ、各々が **findings[]** を `{surface, file:line, claim(引用), reality(実装/正典), severity, fix}` の構造化で返す。
3. **敵対的検証**: 各 finding を別エージェントが「本当に不整合か(実装/正典を確認)」で反証。誤検出(実装にある/正典どおり)は落とす。
4. **統合**: 重複を束ね、severity 順に1本のレポートへ。

エージェントには現在日付と「訓練データで即答せず、必ず該当ファイルを Read/grep して一次確認」を明示する([[instruct-agents-with-date-and-verify-latest]])。CLI 表面はソース(clap)を唯一の真実とする。

## severity
- **blocker**: 事実に反する/矛盾する主張(存在しない CLI、隔離範囲の誤り、相反ワークフロー)。
- **major**: 用語ブレ、ja/en 乖離、未反映の伝播漏れ。
- **minor**: トーン/タイポ/記号(※・絵文字・em-dash)、軽微な表現。

## 出力
- severity 別の findings 表(surface / file:line / 主張 / 実装の事実 / 修正案)。
- 「整合済み」も明記(点検したが問題なしのディメンション)。
- read-only。修正適用はユーザー合意の上、別 PR で(変更したら全サーフェス伝播+grep 残存ゼロ確認 / `propagate-changes-to-all-surfaces-and-grep`)。
