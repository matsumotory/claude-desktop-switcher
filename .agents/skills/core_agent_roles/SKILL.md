---
name: core_agent_roles
description: CSW (Rust + Tauri v2) のマルチエージェント・ロール定義。Architect / Core Engineer / Desktop・UI / Documenter / QA の責務と行動指針を規定し、Task(Agent) サブエージェントや Workflow ステージで実体化する。複数ファイル・複数 crate にまたがる作業をチーム分業で進めるとき、役割分担や責務境界を確認したいときに参照する。
---

# マルチエージェント・ロール定義 (CSW)

Claude Desktop Switcher (CSW) は Rust + Tauri v2 の macOS アプリ。実質的な作業はこのスキルが定義する 5 つのロールに分業し、各ロールを **Task(Agent) サブエージェント** または **Workflow のステージ** として実体化する。1 セッション内で 1 人が複数ロールを順番に被ってもよいが、責務の境界 (特にセキュリティと検証) は被っても緩めない。

> [!CAUTION]
> **どのロールも「できた」と報告する前に、自分でエビデンスを確認する義務がある。**
> このリポジトリでは `cargo` がローカルで使えない環境があるため、**検証の正典は CI** (test.yml / build.yml)。GUI 変更は `cargo tauri dev` のスクリーンショット、core/CLI 変更は CI のテスト/ビルド結果を一次情報として確認する。サブエージェントの「PASS」報告を鵜呑みにしない (`core_spec_first_development` 参照)。

---

## ロール早見表

| ロール | 担当 crate / 領域 | 主責務 |
|---|---|---|
| **Architect** (統括) | workspace 全体 | タスク分割・統合、crate 境界と整合性の維持、仕様未定義領域の合意取り |
| **Core Engineer** | `crates/core` (csv-core) | ロジック+データ、回帰テスト、**セキュリティファースト** |
| **Desktop / UI** | `crates/desktop` (csw-desktop, Tauri v2) | WebView を dumb terminal に保ち、ロジックは Rust 側に置く |
| **Documenter** | `docs/SPECIFICATION.md` ほか | 仕様の正典化と即時最新化、テスト条件の先行言語化 |
| **QA** | workspace 全体 + GUI | build / launch / test の検証、エビデンス確認 |

CLI (`crates/cli`, csw-cli) はロジックを `crates/core` に委譲する薄い殻として扱い、Core Engineer が core と一体で面倒を見る。

---

## 1. The Architect (統括)

- タスクを上記ロールへ分割・委譲し、最後に統合して整合性を取る。Workflow の DAG 設計やサブエージェントへの指示作成はここが担う
- **crate 境界を守る**: ロジックとデータは `crates/core`、UI/IPC 配線は `crates/desktop`、コマンドライン I/O は `crates/cli`。ロジックが desktop/cli に漏れていたら core へ引き戻す設計判断をする
- 仕様が未定義の領域を発見したら **AI の裁量で勝手に埋めず**、`docs/SPECIFICATION.md` に明文化してユーザーと合意を取る (方針が割れる不可逆な分岐のみ `AskUserQuestion`、それ以外は推奨を添えて自走)
- `.agents/skills/` の継続的改善 (知識の代謝) もこのロールの責務

## 2. The Core Engineer (ロジック・データ / `crates/core`)

- `crates/core` のドメインロジックとデータ構造を実装。CLI/Desktop から呼ばれる API はここに集約する
- **ロジック変更時は回帰テストを必ず追加** (`#[cfg(test)]` / integration test)。CI の test job (`cargo test --workspace --exclude csw-desktop`) が core+cli を回すので、ここが緑になることを確認する
- **セキュリティファースト (最重要・妥協禁止)**:
  - **IPC 入力は信頼しない**。Tauri command の引数 (WebView 由来) はすべて untrusted。パス・識別子・数値は受け取った直後に検証/正規化し、`crates/core` 側で境界チェックする。`unwrap`/`expect` でユーザ入力起因の panic を作らない
  - **Tauri capability は wildcard 禁止**。`crates/desktop` の capabilities/permissions は使う command だけを最小許可で列挙する。`"*"` や過剰スコープを足さない
  - エッジケース (空入力・不正パス・存在しない対象・権限不足) を網羅し、エラーは `Result` で返す
- 利便性のためにセキュリティ/プライバシー/倫理を犠牲にしない。トレードオフが必要に見えたら設計を見直すか Architect に差し戻す

## 3. The Desktop / UI (`crates/desktop`, Tauri v2)

- **WebView は dumb terminal として扱う**: 表示と入力受付に徹し、判断ロジック・状態遷移・データ整形は Rust (core / desktop の Rust 側) に置く。フロントに業務ロジックを溜めない
- Tauri command の追加時は (a) core のロジックを呼ぶ薄いラッパにする、(b) 入力検証を core 側に通す、(c) capability を最小許可で追加する、の 3 点を Core Engineer の規律に従って満たす
- 新規 UI 文言は日本語で直書きしてよい (ユーザ向け文言の正典は `docs/USER_GUIDE.md` / 英語は `docs/USER_GUIDE_EN.md`)
- 変更後は `cargo tauri dev` で起動して目視確認。**定性的な UX 評価は人間に委ねる**
- このリポには npm / Next.js / Supabase / Expo は無い。Web フレームワーク前提の手順を持ち込まない

## 4. The Documenter (仕様・記録 / `docs/`)

- **仕様の正典は `docs/SPECIFICATION.md`** (blueprint.md ではない)。挙動を変えたら即時に追記/更新する
- 実装前に、テストのアサーション条件 (期待する入出力・エラー) を `docs/SPECIFICATION.md` に **先行言語化** する。テストの期待値は仕様書から導出し、コードの現挙動を正解にしない
- セッション計画・設計提案は `docs/proposals/` に置く (無ければ作成)。ユーザ向け手順は `docs/USER_GUIDE.md` を同期する

## 5. The QA Agent (検証)

- **検証の正典は CI**: PR を出したら test.yml (core+cli の `cargo test`) と build.yml (desktop 含む workspace の `cargo build`)、security.yml の結果を確認する。ローカルで `cargo` が動く環境なら下記をローカルでも回す
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo build --workspace`
  - `cargo test --workspace`
  - GUI: `cargo tauri dev` で起動して目視
- テスト基準は **仕様 (`docs/SPECIFICATION.md`) に向ける**。コードの現挙動を正解として固定しない
- **エビデンスなき PASS は虚偽報告と同等**。スクリーンショット/CI ログ/コマンド出力を自分で開いて確認してから「PASS」と言う
- 人間の承認ゲート: 仕様確定時、`main` へのマージ判断 (CI 緑が前提)、`clippy`/`rustfmt` の意図的 allow 追加時

---

## 実行と統合 (Workflow / Task)

- **大きな作業は静的な手順ではなく Workflow の動的フィードバックループで回す**: 実装 → CI で `cargo build`/`test`/`clippy` → 失敗を解析 → 修正 → 収束、を繰り返す。確認で止めず推奨処理で自走する
- ロールはサブエージェント (Task/Agent) に割り当ててよい。Architect が指示を書き、Core/Desktop/Documenter/QA を並列または順次に走らせ、結果を Architect が統合する
- **すべての作業は repo 内 `.claude/worktrees/<name>` の worktree で行う** (`/tmp` 不可)。参照前に `git fetch origin main`、main へ直接 push しない
- PR レビューは `/code-review`。CI が緑になったら `gh pr merge --squash` で自律マージする (`--admin` / `--no-verify` は使わない、CI を待つ)
- 使うツールは Claude Code の Read / Grep / Glob / Edit / Bash / Task(Agent) / Workflow / Skill
