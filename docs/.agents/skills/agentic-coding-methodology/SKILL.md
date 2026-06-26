# Agentic Coding Methodology

## Abstract
This skill defines the engineering discipline required for AI agents (and human developers orchestrating them) to successfully write, maintain, and refactor software. It strictly prohibits "vibe coding" and enforces a Design-First, Test-Driven, and Modular orchestration approach.

## Core Directives

### 1. Design-First Development (No "Vibe Coding")
* **Never write code without a plan.** AI agents must not generate functional code based on vague prompts. 
* **Always create an implementation plan first.** When given a complex task, the agent must output a design document (or update `implementation_plan.md`) detailing the components to be touched, the data structures required, and potential edge cases.
* **Wait for human review.** Crucial architectural decisions must be explicitly approved by the human orchestrator before execution begins.

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

### 4. Human-in-the-Loop & Code Review
* **AI as Subordinate, Human as Architect:** The AI agent acts as a highly capable junior developer. The human acts as the team lead.
* **Silently Failing Code:** Agents often write syntactically correct but logically flawed code (e.g., missing authentication checks). Always highlight security-sensitive logic for human review.

## Anti-Patterns
* **Vibe Coding:** Generating massive blocks of code from a 1-sentence prompt.
* **Skipping Tests:** Believing the AI's code is correct simply because it compiles.
* **Context Overload:** Continuing to modify a broken file over and over in the same session instead of starting fresh with a clear plan.
