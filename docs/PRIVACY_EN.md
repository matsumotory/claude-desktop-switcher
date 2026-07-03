# Privacy and Transparency

This document lists everything Claude Desktop Switcher (CSW) reads, writes, and never touches on your Mac, together with the steps to verify each promise yourself. A bare "it is safe" claim gives you nothing to check, so we publish the facts and the verification steps side by side.

This document is kept in sync with the implementation. Whenever the implementation changes, the same change updates this document, and a pre-release consistency check looks for any gap between the two.

## The four promises

1. **No internet communication.** CSW sends no usage data and performs no auto-update requests.
2. **No access to passwords or sign-in data.** CSW never reads the macOS Keychain, sign-in tokens, or browser cookies.
3. **No access to other apps' data.** CSW never reads or writes your existing Claude data on its own. It touches only the items you explicitly choose to share or copy when creating an environment.
4. **Zero impact by default.** CSW rewrites no shell configuration, no OS-level environment variables, and nothing in your existing Claude. Launch Claude without CSW and it behaves exactly as before.

## About network access

CSW does not connect to the internet and has no built-in facility to do so.

The single exception is the links at the bottom of the window (User guide, Report an issue, Check for updates) and the "See how to verify" button in the About screen. Pressing one hands a predetermined GitHub URL to macOS, which opens it in your default browser. The browser communicates; CSW itself does not. The URLs are fixed to the following five, and the implementation additionally rejects any URL that is not `https`.

- `https://github.com/matsumotory/claude-desktop-switcher/blob/main/docs/USER_GUIDE.md`
- `https://github.com/matsumotory/claude-desktop-switcher/issues`
- `https://github.com/matsumotory/claude-desktop-switcher/releases`
- `https://github.com/matsumotory/claude-desktop-switcher/blob/main/docs/PRIVACY.md`
- `https://github.com/matsumotory/claude-desktop-switcher/blob/main/docs/PRIVACY_EN.md`

Update checks work the same way: CSW never queries for new versions; it simply opens the GitHub releases page in your browser.

## Where CSW writes

CSW writes files only inside its own folder, `~/.context-switcher-claude/`. The one exception is deleting an environment, which moves that environment's folder, as is, to the macOS Trash. Nothing new is written out; only the environment folders listed below are ever moved.

| Path | Contents |
|---|---|
| `~/.context-switcher-claude/config.toml` | The name of the currently selected environment |
| `~/.context-switcher-claude/profiles/<name>/profile.toml` | That environment's settings (name, note, creation record, and sharing mode) |
| `~/.context-switcher-claude/profiles/<name>/state.toml` | When CSW last launched that environment |
| `~/.context-switcher-claude/profiles/<name>/desktop-data/` | That environment's data for the Claude Desktop App |
| `~/.context-switcher-claude/profiles/<name>/cli-data/` | That environment's data for Claude Code |

In addition, the settings window stores its own display preferences (the accent color and the first-run flag) in the app's own screen data location managed by macOS. This holds only CSW's appearance settings, never your data or Claude's data.

## Where CSW reads

- CSW reads and writes its own folder listed above. When the detail screen shows the data breakdown, it reads only file names and their sizes and dates inside this folder, never file contents. The targets of shared links are never included in the totals.
- CSW touches the existing Claude folders, `~/Library/Application Support/Claude` and `~/.claude`, in exactly three cases:
  1. **Existence checks**: it checks whether a folder or file exists, without reading its contents.
  2. **Creating and re-pointing shared links**: for items you set to Shared when creating an environment, it creates symbolic links inside the new environment that point to the existing Claude's originals. The isolation check's repair command, `csw doctor --fix`, likewise only re-points share links that no longer point at their declared source. Only the link is swapped; real files are never touched. The links live in the environment's folder; nothing is written on the existing Claude's side.
  3. **Copying**: for items you set to Copy when creating an environment, it reads those files and copies them into the new environment. Only the items you chose are read, and the existing Claude's side is never modified.
- To tell which environment a running Claude is using, CSW reads the list of running processes and their launch arguments. It reads nothing else about those processes and no communication contents.

## About duplicating an environment

Duplicate, in the environment's detail screen, copies the selected environment's entire folder, as is, into a new environment folder on your Mac. What is copied includes that environment's sign-in state, so the duplicated environment starts signed in to the same account. Even then, all CSW does is copy the files verbatim; it never interprets their contents and never sends them anywhere. The existing Claude cannot be duplicated.

## OS commands CSW runs

CSW runs only the following four standard macOS commands. All of them are local operations and none of them communicate.

| Command | Purpose |
|---|---|
| `open` | Launches Claude with the selected environment's data folder, opens the fixed links in your default browser, and opens an environment's data folder in Finder |
| `pgrep` | Checks whether the Claude Desktop App is running |
| `ps` | Reads the launch arguments of running Claude processes to learn which environment's data folder they use |
| `hdiutil` | Detects a CSW installer disk image that is still mounted and ejects it |

## What CSW never touches

- The macOS Keychain that stores your passwords. There is simply no code that reads or writes it.
- Sign-in tokens and cookies of your browser or Claude. Each environment's sign-in data lives inside that environment's data folder, and CSW never opens and interprets those files' contents. When duplicating an environment they are copied verbatim, nothing more.
- Any other app's data beyond the cases listed under "Where CSW reads".
- The network. Nothing is sent and nothing is received.

## Verify it yourself

### Verify that CSW does not communicate

Start CSW, open the settings window, and run the following in a terminal.

```bash
lsof -i -a -p $(pgrep -f Claude-Desktop-Switcher | head -1)
```

This lists the internet connections held by the CSW process. An empty output means there are none.

One note for full accuracy: macOS renders the settings window through separate WebKit processes. CSW's screens load only local files bundled with the app, so those processes make no external connections either. The command above inspects the app's own process only, so if you want to audit the whole machine, watch for CSW-related connections with `nettop` or a dedicated network monitor.

### Verify the download is genuine

Notarization is Apple's review that confirms an app's identity. You can confirm that the CSW you downloaded passed this review with the following commands:

```bash
codesign -dv --verbose=2 "/Applications/Claude-Desktop-Switcher.app"
spctl --assess --type execute -v "/Applications/Claude-Desktop-Switcher.app"
```

If `spctl` reports `accepted` with `Notarized Developer ID`, the artifact passed Apple's notarization.

### Verify the isolation and links

You can inspect how an environment's data is actually wired, in Finder or a terminal:

```bash
ls -la ~/.context-switcher-claude/profiles/<name>/cli-data/
```

Items you set to Shared appear as symbolic links pointing at the existing Claude's originals. Isolated items appear as real files owned by that environment alone.

### Verify that no dependency can communicate

The no-network promise is not just written down; it is enforced mechanically at the dependency level. The repository's `deny.toml` registers HTTP and WebSocket client libraries as banned, and CI fails the build if any of them enters the dependency graph. This check runs automatically on every pull request.

One fact worth stating up front: the dependency record file `Cargo.lock` does contain entries for the HTTP libraries `reqwest` and `hyper`. They are declared by Tauri, the UI foundation of CSW, as dependencies for Windows and Linux, and they are not part of the macOS build. You can confirm this from the source with the commands below.

```bash
cargo tree --target aarch64-apple-darwin -i reqwest
cargo tree --target aarch64-apple-darwin -i hyper
```

If a command prints `warning: nothing to print.`, that library is not part of the macOS dependency graph.

### Verify the bill of materials (SBOM)

Every release from v0.19.0 onward ships with a CycloneDX-format list of all libraries contained in the artifacts. Download files such as `csw-desktop_aarch64-apple-darwin.cdx.json` from the GitHub releases page to see every component and version in the macOS build.

## About this document

CSW is open source and its entire source code is public. If anything in this document disagrees with the implementation, that is a bug: please tell us on [GitHub Issues](https://github.com/matsumotory/claude-desktop-switcher/issues).
