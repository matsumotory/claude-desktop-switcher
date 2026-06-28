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
