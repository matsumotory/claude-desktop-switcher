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
  &nbsp;·&nbsp;
  <a href="docs/PRIVACY_EN.md">Privacy and Transparency</a>
</p>

*Read this in other languages: [日本語 (Japanese)](#日本語-japanese)*

Claude Desktop Switcher is a macOS menu bar utility for separating the whole Claude Desktop App suite (chat, Projects, Claude Cowork, Artifacts, Claude Design, and Claude Code integration) by account and use case on a single Mac. (Switching CLI profiles alone is already well served by existing tools like direnv and claude-swap; CSW's focus is keeping your whole desktop workspace separate, with the CLI linked when you want it.) The Desktop App has no native multi-account switching, so CSW lets you switch environments without signing out each time. You can customize the isolation level per use case, and it also links the Claude Code (CLI) environment to your GUI selection. It works for non-engineers who never touch a terminal, as well as developers who want CLI integration.

<p align="center">
  <img src="website/assets/hero.png" width="560" alt="Claude Desktop Switcher Settings UI">
</p>

## Use Cases & Solutions

How should you partition your environments on a single PC? You can flexibly adjust the isolation level to suit your workflow.

- **01. Separate the whole suite per project/client**
  When running multiple clients or projects in parallel, you want to avoid mixing chat history, Projects memory, Cowork working folders, and Artifacts. Selecting "Separate everything" allocates a separate data directory for each environment (each keeping its own login), structurally preventing information from leaking across the desktop suite.

- **02. Separate work and personal on one Mac (no terminal needed)**
  Keep your business and personal accounts apart without touching the terminal. Just pick an environment and the Claude Desktop App switches without signing in again. It is complete in the GUI.

- **03. Separate by use case while sharing common rules**
  When you want separate accounts but want to reuse common rules and settings. Pick "Separate conversations & memory too" to keep specific files (such as `CLAUDE.md`) shared across environments while keeping history and logins separate.

- **04. Link the CLI to your GUI separation (for developers)**
  Link the environment you chose in the GUI to the CLI's `CLAUDE_CONFIG_DIR`. Run `eval $(csw env <env-name>)` (replace `<env-name>` with your environment's name) in a separate terminal to use the CLI in the same isolated environment. This adds GUI consistency on top of what existing CLI-only switchers already solve.

## Features

- **Integrated Management for GUI & CLI**: Centrally manages the environments behind the Claude Desktop App and Claude Code (CLI). Switch GUI environments by selecting one; sync the CLI to the same isolated environment by running a command in a separate terminal.
- **Customizable Isolation**: You can choose which settings to share and which data to separate based on your use case.
- **Zero-Impact Isolation**: It does not arbitrarily overwrite your Mac's global system settings. Settings are applied locally only to the apps or specific terminal tabs launched by this tool. It makes absolutely no changes to your existing default Claude environment.

## Usage

### GUI Workflow

Managing the desktop app requires no complex configuration. Everything is done in the settings window, with no terminal or scripts.

1. **Install & Launch**: Place `Claude-Desktop-Switcher.app` in your Applications folder and launch it. The settings window opens.
2. **Create Environments**: In the settings window, create independent environments such as "Work" or "Research" with a single click.
3. **Launch in Isolated Environment**: Select an environment and press "Switch and launch" (`切り替えて起動`); that environment's Claude Desktop App launches against its own isolated directory. Environments that share settings open one at a time, so quit a running Claude before switching. An environment set to "separate everything" shares nothing, so you can open it alongside a running Claude with "Launch alongside" (`重複して起動`), without quitting.

### Terminal (Claude Code) Integration

There are two kinds of terminal, and they need different steps.

**1. The terminal inside the Claude Desktop app you launched from CSW (built-in)**
When you switch to an environment and launch it from CSW, any terminal you open inside that app is already in that environment. No extra command is needed; just type `claude` to start working.

**2. A separate terminal you open yourself (external, e.g. iTerm2)**
A terminal you open on your own stays in your usual environment. To use a specific environment, run the sync command below (it points the CLI's `CLAUDE_CONFIG_DIR` at that environment's directory, applies to that tab only, and never affects your usual environment):

```bash
eval $(csw env <env-name>)
```

**Get the `csw` CLI.** The `.dmg` ships the menu-bar app only. Download the signed, notarized `csw` binary from the [latest release](https://github.com/matsumotory/claude-desktop-switcher/releases/latest), make it executable, and move it onto your `PATH` (`chmod +x csw && mv csw /usr/local/bin/`). With a Rust toolchain you can instead run `cargo install --path crates/cli`. The GUI works without this step.

## Build from Source (.dmg)

You can easily build the macOS installer (`.dmg`) using Tauri.

```bash
# Execute build (npm install is required beforehand)
npm run tauri build

# Or, if you use Cargo
cargo tauri build
```
Once the build is complete, the disk image (`Claude-Desktop-Switcher_<version>_<arch>.dmg`) is generated in `target/release/bundle/dmg/`.

## Project Structure (For Developers)

```
crates/
  core/      # Core library (Profile management, file watching, etc.)
  cli/       # CLI tool (csw command)
  desktop/   # Tauri v2 menu bar app and settings UI
```

## Why Rust and Tauri

CSW is built around a shared Rust library (`crates/core`) that handles environment isolation, path resolution, and data safety; the CLI (`csw`) and the menu-bar GUI are thin layers over that same core. Because the top priority is never destroying your existing Claude data, the core lives in Rust, where invariants are enforced at compile time.

The GUI uses Tauri v2, which renders through the operating system's WebView (WebKit on macOS) instead of bundling a browser engine. The result is a small download (the universal DMG is around 7 MB), and code signing plus notarization run through Tauri's standard distribution flow.

This also keeps the development and verification loop easy to automate: the core is checked continuously by the Rust compiler, clippy, and tests, and the UI can be rendered and inspected with a standard headless browser in CI.

For native macOS UI, Apple's Swift and SwiftUI are the first choice, and an excellent one for apps that need deep system integration or a richly crafted native experience. Because CSW's value centers on systems-level logic with a thin UI, we prioritized core correctness and an automatable verification loop. A different kind of app would call for a different choice.

## License

MIT

---

## 日本語 (Japanese)

*Read this in English: [English](#claude-desktop-switcher)*

<p align="center">
  <img src="website/assets/logo.png" width="96" alt="CSW Logo">
</p>

<p align="center">
  <a href="https://matsumotory.github.io/claude-desktop-switcher/"><strong>Web サイト</strong></a>
  &nbsp;·&nbsp;
  <a href="https://github.com/matsumotory/claude-desktop-switcher/releases/latest">ダウンロード (.dmg)</a>
  &nbsp;·&nbsp;
  <a href="docs/USER_GUIDE.md">ユーザーガイド</a>
  &nbsp;·&nbsp;
  <a href="docs/PRIVACY.md">プライバシーと透明性</a>
</p>

Claudeデスクトップアプリのスイート全体（チャット・Projects・Claude Cowork・Artifacts・Claude Design・Claude Code 連携）を、アカウント／用途ごとに安全に分けて 1 つの Mac で使い分けるためのメニューバーアプリです。（CLI のプロファイル切り替えだけなら direnv や claude-swap など既存ツールが既によく解決しています。CSW の主眼は、デスクトップの作業全体を分け、必要なら CLI も同じ環境に連動させることです。）デスクトップアプリはネイティブの複数アカウント切替を備えていないため、再ログインなしで環境を切り替えられます。用途に合わせて分離度を細かくカスタマイズでき、GUI 側の環境分離に Claude Code（CLI）も連動させられます。ターミナルを使わない方から、CLI 連携まで一貫させたい開発者まで対応します。

<p align="center">
  <img src="website/assets/ja_hero.png" width="560" alt="Claude Desktop Switcher 設定 UI">
</p>

### 解決する課題と利用シーン

1つのPCの中で環境をどう分けるか。あなたのワークスタイルに合わせて柔軟に分離度を調整できます。

- **01. 案件・クライアントごとにスイート丸ごと分ける**
  複数のクライアント案件やプロジェクトを並行する際、チャット履歴・Projects のメモリ・Cowork の作業フォルダ・Artifacts が混ざる懸念を避けたい場合に。「すべて分ける」を選ぶと、環境ごとに独立したデータディレクトリ（各々が自分のログインを保持）が割り当てられ、別案件の文脈へ漏れる事故を構造的に防ぎます。

- **02. 仕事用と個人用を 1 台で分ける（非エンジニア向け）**
  業務アカウントと個人アカウントを混ぜたくないが、ターミナル操作はしたくない場合に。環境を選ぶだけで、再ログインなしに別アカウントのデスクトップアプリへ切り替わります。GUI だけで完結します。

- **03. 用途ごとに分けつつ共通ルールは共有する**
  アカウントは用途別に分けたいが、共通の運用ルールや設定は使い回したい場合に。「会話とメモリも分ける」を選ぶと、特定の設定ファイル（CLAUDE.md 等）は全環境で共有したまま、履歴やログインは分離され、設定の二重管理を防げます。

- **04. GUI の分離に CLI を連動させる（開発者向け）**
  GUI で分けた環境を Claude Code（CLI）側でも同じ環境で使いたい場合に。GUI で選んだ環境に `CLAUDE_CONFIG_DIR` を連動させ、別に開いたターミナルで `eval $(csw env <環境名>)`（`<環境名>` は対象の環境名に置き換え）を実行すれば、CLI もその隔離環境で動きます。CLI 単体スイッチャーが既に解決している領域に、GUI 分離との一貫性を足します。

### アプリの特徴

- **GUIとCLIの統合環境管理**: ClaudeデスクトップアプリとClaude Code（CLI）の背後の環境を一元管理します。GUI の環境を選んで切り替えるほか、別に開いたターミナルでコマンドを実行すれば、CLIも同じ隔離環境へ同期できます。
- **分離度のカスタマイズ**: 共有したい設定と分けたいデータを用途に合わせて選択可能です。
- **局所的な環境の適用 (Zero-Impact Isolation)**: Mac本体のシステム設定を無差別に上書きすることはありません。設定は本ツールから起動したアプリや特定のターミナルタブに対してのみ局所的に適用されます。既存のClaude環境には一切変更を加えません。

### 使用方法

#### GUI ワークフロー

デスクトップアプリの管理に複雑な設定は不要です。すべて設定ウインドウで操作できます。

1. **インストール＆起動**: `Claude-Desktop-Switcher.app` をアプリケーションフォルダに入れて起動。設定ウインドウが開きます。
2. **環境の作成**: 設定ウインドウで、ワンクリックで「業務」「研究」など、独立した環境を作成。
3. **分離環境での起動**: 環境を選んで「切り替えて起動」を押すと、その環境専用の隔離ディレクトリで Claudeデスクトップアプリが起動します。設定を共有する環境は衝突を防ぐため一度に1つずつなので、切り替える前に起動中の Claude を終了してください。「すべて分ける」で作った環境は何も共有しないため、起動中の Claude を終了せず「重複して起動」で並べて開けます。

#### ターミナル（Claude Code）連携

ターミナルには 2 種類あり、必要な操作が違います。

**1. CSW から開いた Claudeデスクトップアプリの中のターミナル（内蔵）**
このアプリから環境を切り替えて起動した場合、その中で開くターミナルは最初からその環境になっています。追加のコマンドは不要で、そのまま `claude` と入力して作業を始められます。

**2. 別に開くターミナル（外部・iTerm2 等）**
ご自身で新しく開いたターミナルは、普段の環境のままです。対象の環境を使うときは、次の連携コマンドを実行します（CLI の `CLAUDE_CONFIG_DIR` を対象環境のディレクトリに向けます。そのタブだけに適用され、普段の環境には影響しません）。

```bash
eval $(csw env <環境名>)
```

**`csw` コマンドを入手します。** `.dmg` にはメニューバーアプリのみが含まれます。署名・公証済みの `csw` バイナリを[最新リリース](https://github.com/matsumotory/claude-desktop-switcher/releases/latest)からダウンロードし、実行権を付けて `PATH` の通った場所に置いてください（`chmod +x csw && mv csw /usr/local/bin/`）。Rust の開発環境があれば `cargo install --path crates/cli` でも導入できます。GUI だけで使う場合は不要です。

### 配布用ビルド（DMGファイルの作成）

Tauriを利用して、Mac用のインストーラ（`.dmg`）を簡単にビルドできます。

```bash
# ビルドの実行（事前に npm install 等が必要です）
npm run tauri build

# または、Cargo を使う場合
cargo tauri build
```
ビルドが完了すると、`target/release/bundle/dmg/` に `.dmg`（`Claude-Desktop-Switcher_<version>_<arch>.dmg`）が生成されます。

### プロジェクト構成（開発者向け）

```
crates/
  core/      # コアライブラリ (プロファイル管理、ファイル監視など)
  cli/       # CLIツール (csw コマンド)
  desktop/   # Tauri v2 を用いたメニューバーアプリと設定画面
```

### なぜ Rust と Tauri なのか

CSW の中核は、環境の隔離・パス解決・データ保護を担う共有の Rust ライブラリ（`crates/core`）です。CLI（`csw`）とメニューバー GUI は、どちらもこの同じコアの上に薄く乗っています。ユーザーの既存 Claude データを壊さないことが最優先のため、不変条件をコンパイル時に固められる Rust を中核に置いています。

GUI は Tauri v2 で、macOS 標準の WebView（WebKit）を使い、ブラウザエンジンを同梱しません。そのため配布物は小さく（universal の DMG で約 7MB）、署名と公証も Tauri の標準的な配布フローでそのまま行えます。

この構成は、開発と検証のループを自動で回しやすい利点もあります。中核は Rust のコンパイラ・clippy・テストで継続的に検証でき、画面は標準的なヘッドレスブラウザで CI 上でもレンダリング確認できます。

macOS ネイティブ UI の第一候補は Apple の Swift と SwiftUI で、深いシステム統合や作り込んだネイティブ体験が必要なアプリには優れた選択肢です。CSW は価値の中心がシステム寄りのロジックにあり UI が薄いため、コアの正しさと自動で回る検証を優先しました。用途が変われば、最適な選択も変わります。

## ライセンス

MIT
