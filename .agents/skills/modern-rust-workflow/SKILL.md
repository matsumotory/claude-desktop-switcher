# Modern Rust & Trunk-Based Workflow

## Abstract
This skill defines the workflow and coding standards for developing Rust applications in 2026. It focuses on robust module structuring, compiler-driven development, and CI/CD compatibility.

## Core Directives

### 1. Compiler as a Guardrail (Clippy & FMT)
* **Strict Linting:** The Rust compiler and Clippy are the ultimate authorities. Never bypass warnings unless absolutely necessary (and if so, justify it with a `// ALLOW: ` comment).
* **Treat Warnings as Errors:** In CI/CD pipelines, `#![deny(warnings)]` should be the standard. The AI agent must resolve all Clippy warnings during the refactor phase.
* **Standard Formatting:** Always run `cargo fmt` before concluding a task.

### 2. Error Handling & Robustness
* **No `unwrap()` or `expect()` in production code.** Unless it is provably impossible for an operation to fail (and documented as such), always propagate errors using `?`.
* **Use `thiserror` and `anyhow`:** Use `thiserror` for library crates where specific error typing is required. Use `anyhow` in application binaries (like CLIs) for easy context propagation (`.context("Failed to read file")?`).

### 3. Workspace & Dependency Management
* **Minimize Dependencies:** Before pulling in a heavy crate, determine if the standard library or a lightweight alternative is sufficient. This keeps compile times low and binary sizes small.
* **Workspace inheritance:** Always use workspace-level dependencies in the root `Cargo.toml` (`[workspace.dependencies]`) and reference them in member crates (`{ workspace = true }`) to ensure version consistency.

### 4. Semantic Commits & Trunk-Based Development
* **Semantic Pull Requests:** All commit messages must follow conventional commits (`feat:`, `fix:`, `chore:`, `refactor:`, `docs:`). This is critical for automated versioning tools like `Release Please`.
* **Short-Lived Feature Branches:** Work on short-lived topic branches that merge quickly via PR — never push directly to `main` (branch protection forbids it; see CLAUDE.md Branch Strategy Enforcement). Avoid long-running divergence. Hide unfinished features behind feature flags (`#[cfg(feature = "unstable")]`).

## Anti-Patterns
* **God Objects:** Creating massive structs with dozens of fields. Break them down into smaller, composable traits and structs.
* **Stringly Typed Code:** Using `String` for everything. Use Newtypes (e.g., `struct ProfileName(String);`) to leverage Rust's type system for validation.

## 不正な状態を型で表現不能にする

「X は常に Y でなければならない」という不変条件は、実行時のガードで守るのではなく、型・データモデルの側で不正な状態を構築できないようにします（make illegal states unrepresentable）。X を一般型の自由なフィールドとして持たせ、クランプ・バリデーション・サニタイズで後から矯正する設計は対症療法です。ガードを通らない経路（別の API、将来追加される呼び出し、シリアライズからの復元）が増えるたびに穴が空き、不変条件が破れます。

### 1. 固定ポリシーは設定可能なフィールドから外す
* **不正値を構築不能にすることを第一手にする。** 常に一定でなければならない項目（例: 常に分離すべき対象、常に有効でなければならない状態）は、次のいずれかで表現します。
  * 設定可能なフィールドから外し、コードに埋め込む。
  * 取りうる値だけを持つ専用の型に分ける（Newtype やラッパ型）。
  * `enum` で構築できる値そのものを有限に絞る。
* **クランプ・バリデーションは境界に限定する。** ユーザー入力・外部ファイル・ネットワークなど、本質的に任意の値が入ってくる境界でのみ検証を行い、そこで一度だけ正当な内部型へ変換します。内部データモデルの不変条件を実行時チェックで維持しようとしないでください。境界で受けた値は、検証を通した時点で不正を表現できない型に落とし込み、以降はその型を信頼します。

### 2. 経路ごとのガードでなく下位モデルで担保する
* **非対称を検知したら共有モデルへ寄せる。** 「片方の入口（例: GUI）は守っているが、もう片方の入口（例: CLI）は守っていない」という非対称が出たら、各入口にガードを足して回ってはいけません。両者が共有している下位モデルの側で不正を表現不能にし、どの入口を通っても同じ不変条件を構造的に満たすようにします。
* **多層防御は最後の砦であって一次設計ではない。** 安全・セキュリティに関わる不変条件では、実行時のチェックは万一のための最終防御線として置くことはあっても、それを一次的な保証手段にしないでください。第一の保証は常に「そもそも不正な値を作れない型」です。

### 3. 適用と検証の手順
* **設計時のチェック。** 新しい構造体・enum・設定フィールドを追加するときは、「この型で、あってはならない値を構築できるか」を自問します。構築できるなら、そのフィールドを別型に切り出すか、`enum` で値域を絞るか、フィールド自体を廃してコードに固定できないかを検討します。
* **RED テストで穴を突く。** 不変条件を守っているつもりの型に対して、ガードを通らない経路（デシリアライズ、別コンストラクタ、複数入口）から不正値の構築を試みるテストを書きます。それがコンパイルを通ってしまう、あるいは不正値を保持できてしまうなら、設計が実行時ガード頼みである証拠です。型を修正して、そのテストが「そもそも書けない・コンパイルできない」状態になることを目標にします。
* **grep で入口を数える。** 同じ不変条件を複数箇所でチェックしている（同種のバリデーション呼び出しが散在している）なら、それは下位モデルへ寄せるべき兆候です。検証呼び出しを grep で列挙し、内部モデルに一元化できないか確認します。
