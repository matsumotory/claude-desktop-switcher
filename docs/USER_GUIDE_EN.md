# Claude Desktop Switcher: User Guide

Claude Desktop Switcher is a macOS menu bar utility for safely isolating and managing account environments for the Claude Desktop App and Claude Code (CLI).

### Why do you need this tool? (vs. existing workarounds)
The official Claude Desktop App lacks multi-account switching. To work around this, users have historically relied on messy hacks, such as forcing separate instances via terminal `--user-data-dir` arguments or using CLI-only switchers like `direnv`. 
However, these methods fail to bridge the gap between "safe desktop app isolation (a dedicated data directory per environment)" and "CLI environment syncing."

This tool eliminates the need for complex shell scripts. It achieves **"Desktop App Isolation"** with a single click from the menu bar, and allows **"Linked CLI Launching"** via simple terminal commands. It is based on a "Zero-Impact Principle" that never destroys or mutates your system's global environment variables.

---

## 0. The mental model: separate from "Existing Claude"

Understand this one thing and nothing else is confusing.

First, the account. An account is the Claude account you sign in to; your billing and usage attach to it. Each environment signs in with its own account, so the sign-in info (the OAuth token in `config.json`) is always separate per environment, in every mode.

- **"Existing Claude" = your existing setup** (the Claude Desktop and Claude Code you already use). It is the reference point, and CSW never changes it.
- **Creating a new environment = deciding, item by item, what to inherit from "Existing Claude" and what to keep separate.** The counterpart of share / isolate / copy is always "Existing Claude".
  - **Share**: use the same thing as your existing Claude (changes apply to both)
  - **Isolate**: keep a new thing just for this environment (independent, empty at first)
  - **Copy**: duplicate once at creation, then they diverge
- **If you only want to keep using the same account, you do not need a new environment.** Just use "Existing Claude" as usual; you do not even need to open CSW. CSW is for when you want a *separate environment for a different account or project*.

---

## 1. Installation & Initial Setup

Your existing Claude environment (e.g., your default personal environment) is preserved as-is.

### Step 1: Install the Application
1. Download the latest `.dmg` file from the [Releases](https://github.com/matsumotory/claude-desktop-switcher/releases/latest) page.
2. Drag and drop the downloaded `Claude-Desktop-Switcher.app` into your macOS `Applications` folder.
3. Launch the app. A blue icon will appear in your macOS menu bar.
4. On first launch only, the welcome screen shows a few helper cards (your Existing Claude is protected, you can switch between environments, and the terminal (Claude Code) shares the same environment). They appear once and then go away.

![First-run onboarding](../website/assets/screen_onboarding.png)

### Step 2: Create and Customize a New Environment
Create a new isolated environment for work or research.

![Claude Desktop Switcher Settings UI](../website/assets/hero.png)

**By default, this app separates everything to strictly prevent accidental data mixing.**

For other cases (such as "I want to reuse my common rules and skills, but route usage to my Work account"), you can adjust what carries over.

1. Click the Claude Desktop Switcher icon in the menu bar and select **"Settings..."**.
2. In the settings window, click **"環境を作る" (Create environment)**.
3. Enter the environment information.
   * **Name**: (e.g., `Work`, `Research`)
   * **Icon (optional)**: pick from the prepared icons (or use an emoji if none fit)
4. **Choose how it should be separated (pick one of three modes)**
   The account is always separate in every mode (the create screen states this once up front). The three modes differ only in what else carries over, listed from most carried over to least. Open **"Configure in detail (per item)"** if you want to change individual components.

   * **Separate the account only**: Carry over conversation history and auto-memory too; only the account is separate. Run research and development on separate billing while keeping one continuous workspace.
   * **Separate conversations & memory too**: Carry over your common rules, skills, plugins and tool permissions; keep conversation history and auto-memory to this environment. Split by purpose while reusing your setup.
   * **Separate everything (recommended, default)**: Nothing carries over — the new environment is fully independent. For clients and projects, or work-vs-personal, that must not mix. Your existing Claude is never touched.

   The difference between "Separate conversations & memory too" and "Separate the account only" is the single point of whether conversation history and auto-memory carry over.

   > If your existing Claude isn't found at the standard locations (for example, when Claude Code's config has been moved elsewhere via `CLAUDE_CONFIG_DIR`), the carry-over modes won't bring over whichever side is missing. If neither the Desktop nor the CLI config is found, there's nothing to carry over, so only "Separate everything" can be created. The create screen explains this in either case.

   **< How to choose >**
   * **Case A (Completely separate project)**:
     Pick **"Separate everything"**. You get a pristine environment cleanly detached from your personal one.
   * **Case B (Switching the account while keeping your common rules and skills)**:
     Pick **"Separate conversations & memory too"** to bring your common rules (CLAUDE.md) and skills into the new environment.
   * **Case C (Split billing for research vs development while keeping one continuous workspace)**:
     Pick **"Separate the account only"** to carry over conversation history and auto-memory too, so only the account is separate while your work continues uninterrupted.

5. Click **"Create this environment"** to save.

![Create-environment dialog](../website/assets/screen_create.png)

> **Duplicating an existing environment**
> Select an environment in the settings window and click the **複製 (Duplicate)** button to create a new environment that inherits its sharing configuration and layout. The account sign-in is always separate, so you sign in again in the new environment; isolated components also start empty.

---

## 2. Daily Workflow

Here is the daily usage flow after setup. No manual configuration is required.

### Scenario A: Starting work with your Work account
1. If another Claude Desktop app is running, quit it first. (To avoid configuration conflicts, you cannot run more than one environment's Claude at once.)
2. Click the Claude Desktop Switcher icon in the menu bar and select the "Work" environment you created.
3. **Press "Launch Claude for this environment", and that environment's dedicated Claude Desktop app launches.**
   （This window has a completely independent, dedicated data directory. Log in with your work account the first time you open it.）

> **Note: one environment at a time**
> To avoid configuration conflicts, you cannot run more than one environment's Claude simultaneously. To go back to your personal setup, quit the running Claude, select "Existing Claude" in the sidebar and switch to it, then launch Claude as usual.

### Scenario B: Using the Terminal (Claude Code)
There are two kinds of terminal, and they need different steps.

**1. The terminal inside the Claude Desktop App you launched from CSW (built-in)**
When you switch to an environment and launch it from CSW, any terminal you open inside that app is already in that environment. No extra command is needed; just type `claude` and start working.

**2. A separate terminal you open yourself (external, e.g. iTerm2 or the standard Terminal)**
A terminal you open on your own stays in your usual environment. To use a specific environment, run the sync command.

1. Open your terminal (iTerm2, the standard Terminal, etc.).
2. Run `eval $(csw env Work)` (replace `Work` with the target environment name).
3. That tab's environment variables switch to the target environment. Type `claude` to start (it applies to that tab only and never affects your usual environment).

### Scenario C: Returning to your usual (Personal) environment
* **Desktop**: Select "Existing Claude" from the menu bar, or simply launch `Claude.app` normally via Spotlight. It will always open your standard personal environment.
* **CLI**: If you open a standard terminal and type `claude`, it will always operate as your Existing Claude (personal) environment.

---

## 3. What You Should Know (Safety & Zero-Impact)

* **It's safe even if you forget to launch the app (Zero-Impact)**
  Claude Desktop Switcher never silently alters system environment variables. If you launch Claude normally without using this app, it acts as your Existing Claude 100% of the time. Your existing setup cannot be broken.
* **How to prevent accidental token consumption**
  If you are unsure which account your terminal is using, simply run the `csw status` command. This safely shows the current active environment.
* **Pick the accent color**
  Use the swatches at the bottom of the sidebar to choose the accent: blue (default), teal, indigo, or terracotta. Your choice is saved and applied next time (the semantic colors for shared, isolated, and delete stay the same).
* **Help and version**
  From the very bottom of the sidebar you can open the user guide, report an issue (GitHub), and "About" (the disclaimer). The current version and "Check for updates" (the latest release) live there too. External links open in your default browser; the app itself makes no network requests.

---

## 4. FAQ

**Q. Share / isolate: relative to what?**
Always relative to "Existing Claude" (your existing setup). For each item in a new environment, you choose whether to use the same thing as your existing Claude (share) or keep a new one just for this environment (isolate).

**Q. What is "Existing Claude"?**
It is your existing Claude Desktop and Claude Code setup. CSW only displays it and uses it as the reference; it never changes or deletes your settings, history, or account sign-in. In the app it appears as the first row of the environment list.

**Q. Do I need to create a new environment just to use the same account?**
No. If you only want to keep using the same account, you do not need a new environment; launch Claude as usual and "Existing Claude" is used directly. CSW helps when you want additional, independent environments for different accounts or projects.

**Q. Will creating a new environment break my original Claude?**
No. Each new environment is created in its own dedicated directory, physically separate from the original. Deleting an environment does not affect your original Claude.

**Q. Do I have to sign in again every time I switch?**
No. Each environment keeps its own account sign-in info (`config.json`) inside its own directory. Sign in once per environment and it persists across switches (items set to "isolate" start empty in the new environment).

**Q. Does switching the desktop app also switch the terminal (Claude Code)?**
The terminal you open inside the app launched from CSW (built-in) is already in the same environment as that app, so no command is needed. A terminal you open separately (external) stays in your usual environment, so run `eval $(csw env Work)` to sync it explicitly (it applies to that tab only and never affects your usual environment).

**Q. What exactly carries over with "Separate conversations & memory too"?**
Your common rules (CLAUDE.md), tool permissions and hooks (settings.json), plugins, and skills carry over from "Existing Claude". Conversation history and auto-memory (projects/), prompt history (history.jsonl), and the account sign-in info (config.json) stay separate. The connector and app settings (claude_desktop_config.json, which is where MCP connectors live) and the session state (sessions/) are always separate as well. To fine-tune, use "Configure in detail (per item)" on the create screen.

**Q. What changes if I pick "Separate the account only"?**
On top of the common rules, skills, plugins, and tool permissions, this mode also carries over the per-project conversations and memory (projects/) and the prompt history (history.jsonl). The only thing kept separate is the account (the sign-in info in config.json, which billing and usage are tied to). It suits splitting payment between, say, research and development while keeping one continuous stream of work. The connector and app settings and the session run state stay separate in this mode as well.

**Q. What's the difference between memory and conversation history?**
Memory is the distilled, summarized insight carried over from the past (the human-written CLAUDE.md and Claude's auto-memory under projects/<project>/memory/). Conversation history is the raw record of the exchanges themselves (the .jsonl files under projects/<project>/). "Separate the account only" carries over both; "Separate conversations & memory too" keeps both just for this environment.

**Q. Are the Desktop app's settings and MCP connectors shared?**
No. The Desktop side handles account authentication and rewrites parts of its config at startup, so it cannot be shared safely. The connector and app settings (claude_desktop_config.json) are always separate, and that file is where MCP connectors are configured. What carries over is centered on the Claude Code (CLI) side: common rules, skills, plugins, and tool permissions.

