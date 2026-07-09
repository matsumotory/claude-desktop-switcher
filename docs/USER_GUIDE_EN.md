# Claude Desktop Switcher: User Guide

Claude Desktop Switcher is a macOS menu bar utility for safely isolating and managing account environments for the Claude Desktop App and Claude Code (CLI).

### Why do you need this tool? (vs. existing workarounds)
The official Claude Desktop App lacks multi-account switching. To work around this, users have historically relied on messy hacks, such as forcing separate instances via terminal `--user-data-dir` arguments or using CLI-only switchers like `direnv` or shell aliases. 
However, these methods fail to bridge the gap between "safe desktop app isolation (a dedicated data directory per environment)" and "CLI environment syncing."

This tool eliminates the need for complex shell scripts. It achieves **"Desktop App Isolation"** with a single click in the settings window, and allows **"Linked CLI Launching"** via simple terminal commands. It is based on a "Zero-Impact Principle" that never destroys or mutates your system's global environment variables.

---

## 0. The mental model: separate from "Existing Claude"

Understand this one thing and nothing else is confusing.

First, the account. An account is the Claude account you sign in to; your billing and usage attach to it. Each environment signs in with its own account, so the sign-in info (the OAuth token in `config.json`), the device identifiers, and the desktop app settings including connectors are always separate per environment, in every mode. What changes per environment is whether you carry over your conversation history and auto-memory, and whether you carry over settings such as your common rules and skills.

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
3. Launch the app. The settings window opens, and a menu bar icon also appears (it can be hidden when the menu bar is crowded, but you can always reopen the settings window from the Dock icon). If the installer disk image is still mounted, a prompt appears at the top of the settings window: press "Eject" to eject it right there.
4. On first launch only, the welcome screen shows a few helper cards. They cover three points: your Existing Claude is protected, you can switch between environments, and Claude Code in the terminal uses the same environment. They appear once and then go away.

![First-run onboarding](../website/assets/screen_onboarding.png)

### Step 2: Create and Customize a New Environment
Create a new isolated environment for work or research.

![Claude Desktop Switcher Settings UI](../website/assets/hero.png)

**By default, this app separates everything to strictly prevent accidental data mixing.**

For other cases (such as "I want to reuse my common rules, skills, and plugins, but route usage to my Work account"), you can adjust what carries over.

1. Use the settings window, which opens automatically on launch (if you closed it, reopen it from the Dock icon or via **"Settings..."** on the menu bar icon).
2. Click **"New environment"**.
3. Enter the environment information.
   * **Name**: (e.g., `Work`, `Research`)
   * **Icon (optional)**: pick from the prepared icons
   * **Note (optional)**: write down what the environment is for and which account signs in. The note appears in the environment list and can be edited later on the detail screen.
4. **Choose how it should be separated: pick one of three modes**
   The account is always separate in every mode (the create screen states this once up front). The three modes differ only in what else carries over, listed from most carried over to least. Open **"Configure in detail, item by item"** if you want to change individual components.

   * **Separate the account only**: Carry over conversation history and auto-memory too; only the account is separate. Run research and development on separate billing while keeping one continuous workspace.
   * **Separate conversations & memory too**: Carry over your common rules, skills, plugins and tool permissions; keep conversation history and auto-memory to this environment. Split by purpose while reusing your setup.
   * **Separate everything**: The recommended default. Nothing carries over. The new environment is fully independent. For clients and projects, or work-vs-personal, that must not mix. Your existing Claude is never touched.

   The difference between "Separate conversations & memory too" and "Separate the account only" is the single point of whether conversation history and auto-memory carry over.

   > If your existing Claude isn't found at the standard locations (for example, when Claude Code's config has been moved elsewhere via `CLAUDE_CONFIG_DIR`), the carry-over modes won't bring over whichever side is missing. If neither the Desktop nor the CLI config is found, there's nothing to carry over, so only "Separate everything" can be created. The create screen explains this in either case.

   **< How to choose >**
   * **Case A: a completely separate project**
     Pick **"Separate everything"**. You get a pristine environment cleanly detached from your personal one.
   * **Case B: switching the account while keeping your common rules and skills**
     Pick **"Separate conversations & memory too"** to bring your common rules (CLAUDE.md) and skills into the new environment.
   * **Case C: split billing for research vs development while keeping one continuous workspace**
     Pick **"Separate the account only"** to carry over conversation history and auto-memory too, so only the account is separate while your work continues uninterrupted.

5. Click **"Create this environment"** to save.

![Create-environment dialog](../website/assets/screen_create.png)

> **Duplicating an existing environment**
> Select an environment in the settings window and click the **"Duplicate"** button to create a new environment that inherits its sharing configuration and layout. The account sign-in is always separate, so you sign in again in the new environment; isolated components also start empty.

---

## 2. Daily Workflow

Here is the daily usage flow after setup. No manual configuration is required.

### Scenario A: Starting work with your Work account
1. If the environment you are about to open shares settings and another Claude Desktop app is running, quit it first: environments that share settings open one at a time to avoid configuration conflicts. An environment set to "separate everything" can be opened in a new window without quitting.
2. In the settings window, select the "Work" environment you created. You can also open the settings window from the menu bar or Dock icon. The list shows each environment's note and when it was last launched, so you can find the right one at a glance even as you add more environments.
3. **Press "Launch this environment", and that environment's dedicated Claude Desktop app launches.**
   This window has a completely independent, dedicated data directory. Sign in with your work account the first time you open it. CSW also shows a short guide card with the same reminder on that first launch.

> **Note: running environments at the same time**
> Environments created with "separate conversations & memory too" or "separate the account only" share settings, so they open one at a time to avoid configuration conflicts. To go back to your personal setup, quit the running Claude, select "Existing Claude" in the sidebar and press "Launch Existing Claude". It returns to your Existing Claude and opens it. An environment set to "separate everything" shares nothing, so you can open it alongside a running Claude with "Launch alongside" in the detail view, without quitting.

### Scenario B: Using Claude Code in the terminal
There are two kinds of terminal, and they need different steps. In either case, the first time you use Claude Code in an environment, you need to sign in to the CLI once (see "First time only: signing in to Claude Code" below).

**1. The built-in terminal inside the Claude Desktop App you launched from CSW**
When you switch to an environment and launch it from CSW, any terminal you open inside that app is already in that environment. No command to switch environment variables is needed; just type `claude` and start working.

**2. An external terminal you open yourself, such as iTerm2 or the standard Terminal**
A terminal you open on your own stays in your usual environment. To use a specific environment, run the sync command.

This command needs the `csw` CLI. Download the signed, notarized `csw` binary from the [latest release](https://github.com/matsumotory/claude-desktop-switcher/releases/latest), make it executable, and move it onto your `PATH` (`chmod +x csw && mv csw /usr/local/bin/`). With a Rust toolchain you can instead run `cargo install --path crates/cli`. You do not need it if you only use the desktop app.

1. Open your terminal (iTerm2, the standard Terminal, etc.).
2. Run `eval $(csw env Work)`. Replace `Work` with the target environment name.
3. That tab's environment variables switch to the target environment. It applies to that tab only and never affects your usual environment, so type `claude` to start.

**First time only: signing in to Claude Code**
The Claude Code (CLI) sign-in is managed separately from the desktop app's sign-in, and it is needed once per environment. In either terminal above, running `claude` in an environment you haven't signed into yet will prompt you to sign in. Follow the prompts and, from the sign-in options, choose the subscription account (your Claude account); this is different from the API-billed account. Once you sign in, it persists for that environment, so you don't need to sign in again on later runs or when you switch environments. The built-in terminal and an external terminal switched into the same environment share the same sign-in.

### Scenario C: Returning to your usual personal environment
* **Desktop**: Select "Existing Claude" in the settings window (or from the menu bar icon), or simply launch `Claude.app` normally via Spotlight. It will always open your standard personal environment.
* **CLI**: If you open a standard terminal and type `claude`, it will always operate as your personal Existing Claude environment.

---

## 3. What you should know: safety and zero impact

* **It's safe even if you forget to launch the app**
  Claude Desktop Switcher never silently alters system environment variables. If you launch Claude normally without using this app, it acts as your Existing Claude 100% of the time. Your existing setup cannot be broken.
* **How to tell which account you are using, to avoid the wrong one**
  If you are unsure which account your terminal is using, run the `csw status` command to see the current active environment. When you have several environments open side by side in the desktop app and cannot tell which window is which, select that environment in CSW and press "Bring to front" to raise its Claude. macOS groups multiple windows of the same app under a single Dock icon, so you cannot tell them apart by the Dock icon; CSW raises the one you name instead.
* **Pick the accent color**
  Use the swatches at the bottom of the sidebar to choose the accent: blue (default), teal, indigo, or terracotta. Your choice is saved and applied next time (the semantic colors for shared, isolated, and delete stay the same).
* **Help and version**
  From the very bottom of the sidebar you can open the user guide, report an issue, and "About". Issues go to GitHub, and the disclaimer lives inside "About". The current version and "Check for updates", which opens the latest release, live there too. External links open in your default browser; the app itself makes no network requests.
* **Everything the app reads and writes is documented**
  [Privacy and Transparency](PRIVACY_EN.md) lists every path this app reads, every path it writes, and everything it never touches, together with the steps to verify on your own Mac that it makes no network requests. We publish the verification steps, not just the claims.
* **Deleting an environment by mistake is recoverable**
  Deleting an environment simply moves its folder to the macOS Trash. Until you empty the Trash, nothing is lost, including the sign-in state and share links. To bring it back, move the folder from the Trash into `~/.context-switcher-claude/profiles/`; when you return to the CSW window, it reappears in the list as is. If you want it gone immediately, choose "Delete permanently" in the confirmation (it skips the Trash and cannot be undone).
* **See where this environment's data lives and how much there is**
  Open "Location of this environment's data" in the detail screen to see, beyond the folder paths, how much space the environment occupies by itself, the approximate size of each folder, and a per-item breakdown. Shared items show their link target; everything else shows its approximate size and last-modified date. Only file names, sizes, and dates are read for this; contents are never opened. Each folder opens directly with "Show in Finder", which also makes backups straightforward.
* **Check anytime that the isolation still holds**
  In an environment's detail screen, press "Check now" under "Isolation check" to confirm, item by item, that sharing and isolation still match the settings. The check reads no file contents and changes nothing. In a terminal, `csw doctor` runs the same check, and `csw doctor --fix` only re-points share links that no longer point at their declared source. Claude Code's own `claude doctor` is a different command that diagnoses the installation, unrelated to this check.

---

## 4. FAQ

**Q. Share / isolate: relative to what?**
Always relative to Existing Claude, the setup you already use. Using the same thing as your existing Claude is called share, and keeping a new one just for this environment is called isolate; you choose one per item in a new environment.

**Q. What is "Existing Claude"?**
It is your existing Claude Desktop and Claude Code setup. CSW only displays it and uses it as the reference; it never changes or deletes your settings, history, or account sign-in. In the app it appears as the first row of the environment list.

**Q. Do I need to create a new environment just to use the same account?**
No. If you only want to keep using the same account, you do not need a new environment; launch Claude as usual and "Existing Claude" is used directly. CSW helps when you want additional, independent environments for different accounts or projects.

**Q. Will creating a new environment break my original Claude?**
No. Each new environment is created in its own dedicated directory, physically separate from the original. Deleting an environment does not affect your original Claude.

**Q. Do I have to sign in again every time I switch?**
No. Each environment keeps its own account sign-in info (`config.json`) inside its own directory. Sign in once per environment and it persists across switches. Because the account is always separate per environment, you sign in once right after creating a new environment.

**Q. Does switching the desktop app also switch Claude Code in the terminal?**
A terminal inside the app you launched from CSW is already in the same environment as that app, so no command is needed. A terminal you open separately stays in your usual environment, so pass the target environment name and run `eval $(csw env Work)` to sync it. It applies to that tab only and never affects your usual environment. The environment's config directory is synced automatically, but the first time you use Claude Code in that environment you still need to sign in to the CLI once. See the next Q for details.

**Q. If I'm signed in to the desktop app, is Claude Code in the terminal signed in too?**
No. The desktop app's sign-in and the Claude Code (CLI) sign-in are managed separately. The first time you run `claude` in a terminal for a given environment, you need to sign in to the CLI once for that environment (choose the subscription account as the sign-in method). Once signed in, it persists for that environment, so you don't need to sign in again when you switch. The built-in terminal and an external terminal switched into the same environment share the same sign-in. See Scenario B under "Daily Workflow" for details.

**Q. Can CSW list each environment's Claude usage?**
Not at the moment. CSW is designed to make no internet connection and to never touch your passwords or sign-in. There is currently no way to obtain an accurate figure for Claude usage against its limits, including desktop-app activity, while keeping to those principles, so CSW does not offer it. If an official, safe way becomes available, we will consider adding it. You can check each environment's Claude usage in Claude itself by opening Claude for that environment.

**Q. What exactly carries over with "Separate conversations & memory too"?**
Your common rules (CLAUDE.md), tool permissions and hooks (settings.json), plugins, and skills carry over from "Existing Claude". This mode keeps the project conversations and auto-memory (projects/) and the prompt history (history.jsonl) separate for this environment. The account sign-in info (config.json), the connector and app settings (claude_desktop_config.json, where MCP connectors live), and the session state (sessions/) are always separate, regardless of mode. To fine-tune, use "Configure in detail, item by item" on the create screen.

**Q. What changes if I pick "Separate the account only"?**
On top of the common rules, skills, plugins, and tool permissions, this mode also carries over the per-project conversations and memory (projects/) and the prompt history (history.jsonl). The only thing kept separate is the account (the sign-in info in config.json, which billing and usage are tied to). It suits splitting payment between, say, research and development while keeping one continuous stream of work. The connector and app settings and the session run state stay separate in this mode as well.

**Q. What's the difference between memory and conversation history?**
Memory is the distilled, summarized insight carried over from the past; it comes in two forms, the human-written CLAUDE.md and Claude's auto-memory under projects/<project>/memory/. Conversation history is the raw record of the exchanges themselves, stored as the .jsonl files under projects/<project>/. "Separate the account only" carries over both; "Separate conversations & memory too" keeps both just for this environment.

**Q. Are the Desktop app's settings and MCP connectors shared?**
No. The Desktop side handles account authentication and rewrites parts of its config at startup, so it cannot be shared safely. The connector and app settings (claude_desktop_config.json) are always separate, and that file is where MCP connectors are configured. What carries over is centered on the Claude Code (CLI) side: common rules, skills, plugins, and tool permissions.

**Q. Can I check which environment the Claude in front of me is using?**
Yes. Press **"Check the current environment"** in the menu bar icon's menu: CSW resolves which environment the frontmost Claude is using and answers by selecting that environment in the settings window. Useful when fully isolated environments run side by side and the identical-looking windows are easy to mix up. The frontmost application is read only for this action.

**Q. Can an update to Claude Desktop take me out of my environment?**
Yes, it can happen. The automatic relaunch after an update does not go through CSW, so the reopened Claude runs on Existing Claude's data. The window looks the same, but the "In use" marker in CSW moves to Existing Claude, and if CSW is running, the settings window shows a notice. To continue in your environment, quit Claude and press "Launch this environment" again.

**Q. Reopening every environment by hand after each update is tedious.**
Applying an update requires quitting every running Claude (the app's updater waits for them all to quit before it swaps the app). So CSW remembers the environments that were open together the last time you quit them all, and a single "Reopen" in the settings window banner opens them again. Environments already running are skipped automatically. If a shared environment cannot run alongside another Claude, only that one is flagged with "quit the Claude that is running" first. Nothing launches automatically; it opens only when you press the button.

