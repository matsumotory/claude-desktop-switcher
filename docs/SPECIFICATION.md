# Claude Desktop Switcher Specification

## Overview
Claude Desktop Switcher is a macOS menu bar utility designed to manage multiple, isolated profiles for the Anthropic Claude Desktop App and Claude Code (CLI). It prevents data mixing, token exhaustion on a single account, and credential conflicts across different work contexts.

## Core Features
1. **Profile Management**: Users can create, switch, and delete dedicated profiles (e.g., "Personal", "Work", "Research").
2. **Environment Isolation**: 
   - **Desktop App**: Manages dedicated `~/Library/Application Support/Claude` directories for each profile.
   - **CLI (Claude Code)**: Integrates seamlessly by injecting environment variables (`CLAUDE_CONFIG_DIR`, etc.) when launching a terminal from a specific profile.
3. **Keychain Segregation**: Securely isolates authentication credentials per profile to prevent cross-account contamination.

## Isolation Modes
1. **Isolated (Default)**
   - Complete data separation.
   - Separate API tokens, history, and MCP configurations.
2. **Shared**
   - Separate API tokens/accounts (for independent billing/usage).
   - Shared configuration files (e.g., `CLAUDE.md`, global MCP server definitions).

## Technical Stack
- **Framework**: Tauri v2
- **Backend**: Rust
- **Frontend**: Vanilla HTML/CSS/JS (Lightweight)
- **Target OS**: macOS (Universal Apple Darwin)
- **Deployment**: GitHub Actions (Release Please for automated semantic versioning and `.dmg` artifact generation).

## Zero Impact Guarantee
The application does not modify global macOS environment variables. If a user launches `Claude.app` normally via Spotlight without using the Switcher, it operates entirely within the default, untouched system environment.
