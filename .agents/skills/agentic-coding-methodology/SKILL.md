# Agentic Coding Methodology

## Abstract
This skill defines the engineering discipline required for AI agents (and human developers orchestrating them) to successfully write, maintain, and refactor software. It strictly prohibits "vibe coding" and enforces a Design-First, Test-Driven, and Modular orchestration approach.

## Core Directives

### 1. Design-First Development (No "Vibe Coding")
* **Never write code without a plan.** AI agents must not generate functional code based on vague prompts. 
* **Always create an implementation plan first.** When given a complex task, the agent must output a design document detailing the components to be touched and the data structures required.
* **Smart Escalation (Avoid Meaningless Confirmations):** Do not block execution to ask the human for trivial implementation details. Make a sensible engineering decision, document it in the plan, and proceed to execute. Only wait for explicit human approval if the decision drastically alters the system architecture or introduces a breaking change.

### 2. Spec-Driven & Test-Driven Development (TDD)
* **Write tests before functional code.** This forces the agent to reason about requirements, interfaces, and edge cases *before* getting lost in implementation details.
* **Red-Green-Refactor:** 
  1. Write failing tests based on the spec.
  2. Write the minimum amount of functional code to pass the tests.
  3. Refactor the code for performance, readability, and security without breaking the tests.

### 3. Modular Task Decomposition
* **Single Responsibility Principle for Prompts:** Do not attempt to complete massive tasks (e.g., "build a backend") in a single prompt or generation step.
* **Step-by-Step Execution:** Break tasks down into:
  1. Interface / Trait definition.
  2. Mocking and Test writing.
  3. Core logic implementation.
  4. Integration and Verification.
* If context becomes overloaded or the agent begins to hallucinate, STOP. Request a context reset and focus on a narrower sub-task.

### 4. Autonomous Feedback Loops & Self-Correction
* **Self-Healing Execution:** The agent MUST autonomously run verification steps (e.g., `cargo test`, `cargo check`, `cargo clippy`) after writing code.
* **Read, Diagnose, Fix:** If a test or build fails, do NOT immediately stop and ask the human what to do. Read the error log, diagnose the root cause, apply a fix, and verify again. Repeat this feedback loop autonomously until the code is green.
* **Escalate on Dead-Ends:** Only escalate to the human if you are trapped in an infinite loop of errors or if fixing the error requires changing a fundamental requirement.

### 5. Code Review & Security
* **Silently Failing Code:** Agents often write syntactically correct but logically flawed code (e.g., missing authentication checks). Always highlight security-sensitive logic for human review after the autonomous loop completes.

## Anti-Patterns
* **Vibe Coding:** Generating massive blocks of code from a 1-sentence prompt.
* **Skipping Tests:** Believing the AI's code is correct simply because it compiles.
* **Context Overload:** Continuing to modify a broken file over and over in the same session instead of starting fresh with a clear plan.
