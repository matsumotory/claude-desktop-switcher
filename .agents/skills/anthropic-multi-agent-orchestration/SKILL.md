---
description: 大きなタスクを Claude Code のサブエージェント (Task/Agent) と Workflow に分担させる多エージェント協調の指針。
---

# Multi-Agent Orchestration (Claude Code)

## Abstract
This skill enforces a multi-agent coordination workflow on Claude Code. Instead of expecting a single pass to handle a large, multi-faceted task, the main agent orchestrates the work across specialized logical personas, realized as `Task`/`Agent` subagents (run in parallel) or, for larger deterministic fan-out/verification, a `Workflow`.

## Core Directives

### 1. The Orchestrator Pattern
* **Role Separation:** The primary agent (you) acts as the Orchestrator. Break tasks into domain-specific sub-tasks (e.g., "core logic", "Tauri UI", "Security Audit") and dispatch each to a `Task`/`Agent` subagent. Launch independent subagents in a single message so they run concurrently.
* **Parallel Execution Planning:** Before writing code, output a plan detailing which specialist roles need to execute which parts. For large audits/migrations with verification stages, use a `Workflow` instead of ad-hoc subagents.

### 2. Specialized Personas
When executing a sub-task, explicitly adopt the corresponding persona and constraint set:
* **The Architect:** Focuses solely on structure, interfaces, trait definitions, and API boundaries. Writes NO implementation logic.
* **The Implementer:** Fills in the implementation logic bounded by the Architect's interfaces.
* **The Red Teamer (Security Auditor):** Reviews the Implementer's code strictly for vulnerabilities, token leaks, and logic flaws before the code is finalized.

### 3. Progressive Disclosure of Context
* **Avoid Context Bloat:** Do not load the entire repository into context if only modifying a single crate.
* **Targeted Lookups:** Use `Grep` / `Glob` (and the `Explore` subagent for broad fan-out searches) to pull only the strictly necessary files into the context window for a given sub-task.

### 4. Self-Healing & Verification
* **Automated Feedback Loop:** The "Testing Persona" must run the test suite and feed errors back to the Implementer autonomously. 
* **Do not escalate trivial compilation errors to the human.** Resolve them within the multi-agent loop.
