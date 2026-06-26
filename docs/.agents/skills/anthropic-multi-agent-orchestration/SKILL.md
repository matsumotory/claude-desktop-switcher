# Anthropic Multi-Agent Orchestration

## Abstract
Based on Anthropic's 2026 research for Claude Fable 5 and Mythos 5, this skill enforces a Multi-Agent coordination workflow. Instead of expecting a single agent to handle full-stack implementation in one pass, tasks must be orchestrated across specialized logical personas.

## Core Directives

### 1. The Orchestrator Pattern
* **Role Separation:** The primary agent (you) acts as the Orchestrator. You must break tasks into domain-specific sub-tasks (e.g., "Frontend UI", "Database Schema", "Security Audit").
* **Parallel Execution Planning:** Before writing code, output a plan detailing which "specialist roles" (simulated or actual sub-agents) need to execute which parts of the task.

### 2. Specialized Personas
When executing a sub-task, explicitly adopt the corresponding persona and constraint set:
* **The Architect:** Focuses solely on structure, interfaces, trait definitions, and API boundaries. Writes NO implementation logic.
* **The Implementer:** Fills in the implementation logic bounded by the Architect's interfaces.
* **The Red Teamer (Security Auditor):** Reviews the Implementer's code strictly for vulnerabilities, token leaks, and logic flaws before the code is finalized.

### 3. Progressive Disclosure of Context (ADK standard)
* **Avoid Context Bloat:** Do not load the entire repository into context if only modifying a single crate.
* **Targeted Lookups:** Use explicit search tools (`grep_search`, AST parsing) to pull only the strictly necessary files into the context window for a given sub-agent task.

### 4. Self-Healing & Verification
* **Automated Feedback Loop:** The "Testing Persona" must run the test suite and feed errors back to the Implementer autonomously. 
* **Do not escalate trivial compilation errors to the human.** Resolve them within the multi-agent loop.
