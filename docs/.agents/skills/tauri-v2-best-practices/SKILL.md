# Tauri v2 AI Agent Architecture & Best Practices

## Abstract
This skill defines the architectural guidelines and security rules for building AI agent desktop applications using Tauri v2 and Rust 2024 edition. It emphasizes the "Native vs Sidecar" tradeoff, strict capability-based security, and optimal performance techniques.

## Core Directives

### 1. Architectural Strategy: Native Rust vs. Sidecar
* **Rust-Native First:** Always implement agent loops, tool orchestration, and LLM API calls (using `reqwest` or `async-openai`) directly in Rust if possible. This provides the best performance, smallest binary size, and native system integration.
* **Sidecar for Frameworks:** Only use a Python/Node sidecar process if you MUST rely on massive ecosystem-specific AI frameworks (e.g., LangChain, LlamaIndex) that are impractical to rewrite in Rust.
* **Separation of Concerns:** The WebView (frontend) should purely be a "dumb terminal" for rendering UI. ALL business logic, state management, and file system access MUST happen in the Rust backend.

### 2. Security Rules (Capability-Based Model)
* **Never Use Wildcards:** Tauri v2 uses a strict capability system. NEVER use wildcard permissions (`*`) in `src-tauri/capabilities/`.
* **Explicit Scopes:** Define explicit, narrow scopes for every command, fs access, and shell execution.
* **Strict CSP:** Define a strict Content Security Policy (CSP) in `tauri.conf.json`. If the frontend needs to render external images or connect to specific APIs directly, explicitly whitelist them.
* **Input Validation:** Treat all data arriving via IPC from the frontend as untrusted. Validate structs using Rust's type system and explicit checks before performing operations.

### 3. Performance & Asynchronous Operations
* **Async Commands:** AI inference and API calls are blocking. Always use `async` functions for Tauri commands (`#[tauri::command] async fn...`) to prevent freezing the main thread.
* **Token Streaming:** For LLM responses, do not wait for the entire response to complete before returning to the UI. Use Tauri Events (`app_handle.emit()`) to stream tokens in real-time to the frontend.
* **Binary Optimization:** Set `opt-level = "z"`, `lto = true`, `codegen-units = 1`, and `strip = true` in `Cargo.toml` for production release profiles to minimize the DMG/AppImage bundle size.

### 4. Implementation Patterns
* **ManagedProcess Struct:** When spawning local LLMs (e.g., Ollama) or sidecars, create a `ManagedProcess` struct in Rust to carefully manage the lifecycle (spawn, monitor `stdout/stderr`, graceful shutdown on App exit).
* **State Management:** Use Tauri's `State<'_, T>` to share API keys, database connections, or conversation memory safely across multiple command handlers. Use `std::sync::Arc<tokio::sync::Mutex<T>>` for mutable shared state.

## Anti-Patterns
* **Avoid `eval` in WebView:** Never evaluate arbitrary JavaScript from the Rust backend.
* **Don't store secrets in Frontend:** API keys (OpenAI, Anthropic) MUST be stored securely using the OS keychain (via Rust `keyring` crate) and never sent to the frontend Javascript.
