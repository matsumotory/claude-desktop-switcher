# Claude Desktop Switcher

<p align="center">
  <img src="website/assets/logo.png" width="96" alt="CSW Logo">
</p>

<p align="center">
  <a href="https://matsumotory.github.io/claude-desktop-switcher/"><strong>Website</strong></a>
  &nbsp;·&nbsp;
  <a href="https://github.com/matsumotory/claude-desktop-switcher/releases/latest">Download (.dmg)</a>
  &nbsp;·&nbsp;
  <a href="docs/USER_GUIDE_EN.md">User Guide</a>
</p>

*Read this in other languages: [日本語 (Japanese)](#日本語-japanese)*

Claude Desktop Switcher is a macOS menu bar utility for separating the whole Claude Desktop App suite (chat, Projects, Claude Cowork, Artifacts, Claude Design, and Claude Code integration) by account and use case on a single Mac. (Switching CLI profiles alone is already well served by existing tools like direnv and claude-swap; CSW's focus is keeping your whole desktop workspace separate, with the CLI linked when you want it.) The Desktop App has no native multi-account switching, so CSW lets you switch environments without signing out each time. You can customize the isolation level per use case, and it also links the Claude Code (CLI) environment to your GUI selection. It works for non-engineers who never touch a terminal, as well as developers who want CLI integration.

<p align="center">
  <img src="website/assets/hero.png" width="560" alt="Claude Desktop Switcher Settings UI">
</p>

## Use Cases & Solutions

How should you partition your environments on a single PC? You can flexibly adjust the isolation level to suit your workflow.

- **01. Separate the whole suite per project/client**
  When running multiple clients or projects in parallel, you want to avoid mixing chat history, Projects memory, Cowork working folders, and Artifacts. Selecting the "Isolated" configuration allocates a separate data directory for each environment (each keeping its own login), structurally preventing information from leaking across the desktop suite.

- **02. Separate work and personal on one Mac (no terminal needed)**
  Keep your business and personal accounts apart without touching the terminal. Pick an environment from the menu bar and the Claude Desktop App switches without signing in again. It is complete in the GUI.

- **03. Separate by use case while sharing common rules**
  When you want separate accounts but want to reuse common rules and settings. A "Shared" configuration synchronizes specific files (such as `CLAUDE.md`) across environments while keeping history and logins separate.

- **04. Link the CLI to your GUI separation (for developers)**
  Link the environment you chose in the GUI to the CLI's `CLAUDE_CONFIG_DIR`. Run `eval $(csw env Work)` in a separate terminal to use the CLI in the same isolated environment. This adds GUI consistency on top of what existing CLI-only switchers already solve.

## Features

- **Integrated Management for GUI & CLI**: Centrally manages the environments behind the Claude Desktop App and Claude Code (CLI). Switch GUI environments from the menu bar; sync the CLI to the same isolated environment by running a command in a separate terminal.
- **Customizable Isolation**: You can choose which settings to share and which data to separate based on your use case.
- **Zero-Impact Isolation**: It does not arbitrarily overwrite your Mac's global system settings. Settings are applied locally only to the apps or specific terminal tabs launched by this tool. It makes absolutely no changes to your existing default Claude environment.

## Usage

### GUI Workflow

Managing the desktop app requires no complex configuration. Everything can be operated intuitively from the menu bar.

1. **Install & Launch**: Place `ClaudeDesktopSwitcher.app` in your Applications folder and launch it. It will reside in the menu bar.
2. **Create Environments**: From `Settings...` in the menu, create independent environments such as "Work" or "Research" with a single click.
3. **Launch in Isolated Environment**: When you select an environment, the Claude desktop app launches referencing its dedicated, isolated directory.

### Terminal (Claude Code) Integration

There are two kinds of terminal, and they need different steps.

**1. The terminal inside the Claude Desktop app you launched from CSW (built-in)**
When you switch to an environment and launch it from CSW, any terminal you open inside that app is already in that environment. No extra command is needed; just type `claude` to start working.

**2. A separate terminal you open yourself (external, e.g. iTerm2)**
A terminal you open on your own stays in your usual environment. To use a specific environment, run the sync command below (it points the CLI's `CLAUDE_CONFIG_DIR` at that environment's directory, applies to that tab only, and never affects your usual environment):

```bash
eval $(csw env Work)
```

## Build from Source (.dmg)

You can easily build the macOS installer (`.dmg`) using Tauri.

```bash
# Execute build (npm install is required beforehand)
npm run tauri build

# Or, if you use Cargo
cargo tauri build
```
Once the build is complete, `Claude Desktop Switcher.dmg` will be generated in `crates/desktop/src-tauri/target/release/bundle/dmg/`.

## Project Structure (For Developers)

```
crates/
  core/      # Core library (Profile management, file watching, etc.)
  cli/       # CLI tool (csw command)
  desktop/   # Tauri v2 menu bar app and settings UI
```

## License

MIT

---

## 日本語 (Japanese)

<p align="center">
  <img src="website/assets/logo.png" width="96" alt="CSW Logo">
</p>

<p align="center">
  <a href="https://matsumotory.github.io/claude-desktop-switcher/"><strong>Web サイト</strong></a>
  &nbsp;·&nbsp;
  <a href="https://github.com/matsumotory/claude-desktop-switcher/releases/latest">ダウンロード (.dmg)</a>
  &nbsp;·&nbsp;
  <a href="docs/USER_GUIDE.md">ユーザーガイド</a>
</p>

Claudeデスクトップアプリのスイート全体（チャット・Projects・Claude Cowork・Artifacts・Claude Design・Claude Code 連携）を、アカウント／用途ごとに安全に分けて 1 つの Mac で使い分けるためのメニューバーアプリです。（CLI のプロファイル切り替えだけなら direnv や claude-swap など既存ツールが既によく解決しています。CSW の主眼は、デスクトップの作業全体を分け、必要なら CLI も同じ環境に連動させることです。）デスクトップアプリはネイティブの複数アカウント切替を備えていないため、再ログインなしで環境を切り替えられます。用途に合わせて分離度を細かくカスタマイズでき、GUI 側の環境分離に Claude Code（CLI）も連動させられます。ターミナルを使わない方から、CLI 連携まで一貫させたい開発者まで対応します。

<p align="center">
  <img src="website/assets/ja_hero.png" width="560" alt="Claude Desktop Switcher 設定 UI">
</p>

### 解決する課題と利用シーン

1つのPCの中で環境をどう分けるか。あなたのワークスタイルに合わせて柔軟に分離度を調整できます。

- **01. 案件・クライアントごとにスイート丸ごと分ける**
  複数のクライアント案件やプロジェクトを並行する際、チャット履歴・Projects のメモリ・Cowork の作業フォルダ・Artifacts が混ざる懸念を避けたい場合に。「完全隔離」構成を選ぶと、環境ごとに独立したデータディレクトリ（各々が自分のログインを保持）が割り当てられ、別案件の文脈へ漏れる事故を構造的に防ぎます。

- **02. 仕事用と個人用を 1 台で分ける（非エンジニア向け）**
  業務アカウントと個人アカウントを混ぜたくないが、ターミナル操作はしたくない場合に。メニューバーから環境を選ぶだけで、再ログインなしに別アカウントのデスクトップアプリへ切り替わります。GUI だけで完結します。

- **03. 用途ごとに分けつつ共通ルールは共有する**
  アカウントは用途別に分けたいが、共通の運用ルールや設定は使い回したい場合に。特定の設定ファイル（CLAUDE.md 等）だけを全環境で共有する構成に対応し、履歴やログインは分離したまま設定の二重管理を防ぎます。

- **04. GUI の分離に CLI を連動させる（開発者向け）**
  GUI で分けた環境を Claude Code（CLI）側でも同じ環境で使いたい場合に。GUI で選んだ環境に `CLAUDE_CONFIG_DIR` を連動させ、別に開いたターミナルで `eval $(csw env Work)` を実行すれば、CLI もその隔離環境で動きます。CLI 単体スイッチャーが既に解決している領域に、GUI 分離との一貫性を足します。

### アプリの特徴

- **GUIとCLIの統合環境管理**: ClaudeデスクトップアプリとClaude Code（CLI）の背後の環境を一元管理します。メニューバーからGUIの環境を切り替えるほか、別に開いたターミナルでコマンドを実行すれば、CLIも同じ隔離環境へ同期できます。
- **分離度のカスタマイズ**: 共有したい設定と分けたいデータを用途に合わせて選択可能です。
- **局所的な環境の適用 (Zero-Impact Isolation)**: Mac本体のシステム設定を無差別に上書きすることはありません。設定は本ツールから起動したアプリや特定のターミナルタブに対してのみ局所的に適用されます。既存のClaude環境には一切変更を加えません。

### 使用方法

#### GUI ワークフロー

デスクトップアプリの管理に複雑な設定は不要です。すべてメニューバーから直感的に操作できます。

1. **インストール＆起動**: `ClaudeDesktopSwitcher.app` をアプリケーションフォルダに入れて起動。メニューバーに常駐します。
2. **環境の作成**: メニューの `Settings...` から、ワンクリックで「業務」「研究」など、独立した環境を作成。
3. **分離環境での起動**: 環境を選択すると、Claudeデスクトップアプリが隔離された専用ディレクトリを参照して起動します。

#### ターミナル（Claude Code）連携

ターミナルには 2 種類あり、必要な操作が違います。

**1. CSW から開いた Claudeデスクトップアプリの中のターミナル（内蔵）**
このアプリから環境を切り替えて起動した場合、その中で開くターミナルは最初からその環境になっています。追加のコマンドは不要で、そのまま `claude` と入力して作業を始められます。

**2. 別に開くターミナル（外部・iTerm2 等）**
ご自身で新しく開いたターミナルは、普段の環境のままです。対象の環境を使うときは、次の連携コマンドを実行します（CLI の `CLAUDE_CONFIG_DIR` を対象環境のディレクトリに向けます。そのタブだけに適用され、普段の環境には影響しません）。

```bash
eval $(csw env Work)
```

### 配布用ビルド（DMGファイルの作成）

Tauriを利用して、Mac用のインストーラ（`.dmg`）を簡単にビルドできます。

```bash
# ビルドの実行（事前に npm install 等が必要です）
npm run tauri build

# または、Cargo を使う場合
cargo tauri build
```
ビルドが完了すると、`crates/desktop/src-tauri/target/release/bundle/dmg/` に `Claude Desktop Switcher.dmg` が生成されます。

### プロジェクト構成（開発者向け）

```
crates/
  core/      # コアライブラリ (プロファイル管理、ファイル監視など)
  cli/       # CLIツール (csw コマンド)
  desktop/   # Tauri v2 を用いたメニューバーアプリと設定画面
```
