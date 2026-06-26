# Claude Desktop Switcher - Technical Specification

## 1. System Architecture

Claude Desktop Switcher is a Rust-based, macOS-native utility designed to completely isolate multiple instances (Profiles) of Claude Desktop and Claude Code (CLI). 

The project is structured into three workspaces (crates) using Rust 2024 Edition and Tauri v2:
- **`csw-core`**: The domain logic. Handles profile management, configuration files (`toml`), and file watching (`notify` v7).
- **`csw-cli`**: The CLI interface. Provides headless commands to create, switch, and delete profiles.
- **`csw-desktop`**: The Tauri v2 application. Provides a system tray / menu bar interface on macOS.

## 2. Isolation Strategy (Zero-Impact Guarantee)

To prevent existing environments from being polluted, the application **never modifies global environment variables** or global system paths permanently.

### Core Mechanisms:
- **Default Environment Preservation**: If the user launches Claude Desktop or Claude Code without using this switcher, they run under the unmodified system default (e.g., `~/Library/Application Support/Claude`).
- **Profile Data Segregation**: Every newly created profile is assigned two isolated sandbox directories:
  - `desktop_user_data_dir`: `~/.gemini/antigravity/profiles/<ProfileName>/desktop-data`
  - `cli_config_dir`: `~/.gemini/antigravity/profiles/<ProfileName>/cli-data`

### Execution Context:
When a profile is activated, the app launches the Claude binaries by injecting custom runtime arguments (e.g., `--user-data-dir`) or by temporarily swapping symlinks immediately before launch, and tearing them down immediately after. This guarantees that crash-restarts or external launches are not contaminated.

## 3. Configuration & Sharing Modes

Each profile defines a `SharingConfig` that controls whether specific configuration components are isolated or synchronized with a "source" profile (usually `default`).

There are exactly three `SharingMode` types:
1. **`Isolate` (Default)**: The file/directory is completely blank for the new profile. It shares no history or data.
2. **`Share`**: A soft symlink (`ln -s`) is created pointing to the source profile's path. Any changes made by the profile are immediately reflected globally.
3. **`Copy`**: A one-time hard copy is performed. A background `FileWatcher` (via `notify`) monitors the source file for changes and propagates them down to the profile. This is ideal for `CLAUDE.md` where global rules need to flow down, but local project rules do not flow up.

### Component Mapping:
| Component | Default Mode | Path | Description |
|-----------|--------------|------|-------------|
| **API Tokens / Auth** | **Isolate** (Enforced) | Keychain / Secure Storage | Never synced to prevent accidental consumption. |
| **History (Memory)** | **Isolate** (Enforced) | `.../cli-data/projects/` | Conversation memory is strictly isolated. |
| **MCP Config** | `Isolate` | `claude_desktop_config.json` | Can be set to `Copy` or `Share` for unified tooling. |
| **CLAUDE.md** | `Isolate` | `CLAUDE.md` | Can be set to `Copy` for unified system prompt rules. |
| **Plugins** | `Isolate` | `plugins/` | Installed Claude Code extensions. |

## 4. Verification Checklist

To verify that the implementation adheres to this specification, the following automated or manual tests must pass:

- [ ] **Test: Independent Data**: Create "Work" and "Research" profiles. Login to different accounts in each. Ensure closing one and opening the other does not log out the other.
- [ ] **Test: Default Integrity**: Launch `Claude.app` manually via Spotlight. Ensure it opens the original (default) account, proving zero environment pollution.
- [ ] **Test: Copy Sync**: Set `CLAUDE.md` to `Copy` mode. Edit the default profile's `CLAUDE.md`. The `FileWatcher` must successfully replicate the edit into the target profile within 1 second.
- [ ] **Test: CI Build**: The `tauri-action@v2` CI pipeline must compile without errors on `universal-apple-darwin` and attach a valid DMG artifact to the GitHub release.
