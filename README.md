# Claude Desktop Switcher

*Read this in other languages: [日本語 (Japanese)](#日本語-japanese)*

Claude Desktop Switcher is a macOS menu bar utility for safely isolating and managing multiple account environments (Personal, Work, etc.) for the Claude Desktop App on a single Mac. You can precisely customize the isolation level for each use case, and it allows for integrated management of both the Claude Desktop App and Claude Code (CLI) environments.

## Use Cases & Solutions

How should you partition your contexts on a single PC? You can flexibly adjust the isolation level to suit your workflow.

- **01. Complete Isolation per Project/Use Case**
  When running multiple client projects or projects with completely different tech stacks in parallel, you want to eliminate the risk of mixing Claude chat histories or project contexts. By selecting the "Isolated" configuration, a separate data area and keychain are allocated for each profile, structurally preventing information leakage.

- **02. Partial Sharing for Development and Research**
  When you want to separate billing accounts by use case, but want to reuse plugins or common settings. Supports a "Shared" configuration that synchronizes specific configuration files (such as `CLAUDE.md`) across all profiles, preventing duplicate management of settings.

- **03. Multi-Account Expansion**
  When you want to easily manage multiple accounts according to free tier limits or diverse usage scenarios. You can instantly access a different environment without logging in again just by selecting a profile from the menu.

## Features

- **Integrated Profile Management for GUI & CLI**: Centrally manages the profiles behind the Claude Desktop app and Claude Code (CLI). In addition to switching GUI environments from the menu bar, you can sync your CLI to the exact same isolated environment by running a single command from your terminal.
- **Customizable Isolation**: You can choose which settings to share and which data to separate based on your use case.
- **Zero-Impact Isolation**: It does not arbitrarily overwrite your Mac's global system settings. Settings are applied locally only to the apps or specific terminal tabs launched by this tool. It makes absolutely no changes to your existing default Claude environment.

## Usage

### GUI Workflow

Managing the desktop app requires no complex configuration. Everything can be operated intuitively from the menu bar.

1. **Install & Launch**: Place `ClaudeDesktopSwitcher.app` in your Applications folder and launch it. It will reside in the menu bar.
2. **Create Profiles**: From `Settings...` in the menu, create independent profiles such as "Work" or "Research" with a single click.
3. **Launch in Isolated Environment**: When you select a profile, the Claude desktop app launches referencing its dedicated, isolated directory.

### Advanced Integration with External Terminals (CLI)

You can use the same profiles not only in the terminal built into the desktop app, but also in your usual external terminal (like iTerm2).

By simply running the following command in your terminal, you can instantly access the specified environment (Keychain backup and restoration are handled automatically):

```bash
eval $(csw env <Profile_Name>)
```

*Note: Since the GUI and CLI contexts are isolated, "launching the desktop app does not automatically switch the terminal." You must explicitly sync each terminal session using the command above.*

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

Claude デスクトップアプリのアカウント環境（個人用・仕事用など）を安全に分けて、1つのMacで使い分けるためのメニューバーアプリです。用途に合わせてアカウントの分離度を細かくカスタマイズでき、ClaudeデスクトップアプリとClaude Code（CLI）の両方の環境を統合管理できます。

### 解決する課題と利用シーン

1つのPCの中でコンテキストをどう分けるか。あなたのワークスタイルに合わせて柔軟に分離度を調整できます。

- **01. 案件や用途ごとの完全分離**
  複数のクライアント案件や、技術スタックが全く異なるプロジェクトを並行して進める際、Claudeのチャット履歴やプロジェクトコンテキストが混ざってしまうリスクを完全に排除したい場合に。「完全隔離」構成を選択することで、プロファイルごとに独立したデータ領域とキーチェーンが割り当てられ、情報漏洩の事故を構造的に防ぎます。

- **02. 開発と研究の部分共有**
  用途ごとに課金アカウントは分けたいが、プラグインや共通設定などは使い回したい場合に。特定の設定ファイル（CLAUDE.md等）を、全プロファイルで同期させる「部分共有」構成に対応し、設定の二重管理を防ぎます。

- **03. マルチアカウント拡張**
  無料枠の制限や用途の広がりに合わせ、複数アカウントを手軽に運用したい場合に。メニューからプロファイルを選ぶだけで、再ログインなしに別環境へ即座にアクセス可能です。

### アプリの特徴

- **GUIとCLIの統合プロファイル管理**: ClaudeデスクトップアプリとClaude Code（CLI）の背後のプロファイルを一元管理します。メニューバーからGUIの環境を切り替えるだけでなく、ターミナルからコマンドを1つ叩くだけで、CLIも全く同じ隔離環境へ同期させることができます。
- **分離度のカスタマイズ**: 共有したい設定と分けたいデータを用途に合わせて選択可能です。
- **局所的なプロファイル適用 (Zero-Impact Isolation)**: Mac本体のシステム設定を無差別に上書きすることはありません。設定は本ツールから起動したアプリや特定のターミナルタブに対してのみ局所的に適用されます。既存のClaude環境には一切変更を加えません。

### 使用方法

#### GUI ワークフロー

デスクトップアプリの管理に複雑な設定は不要です。すべてメニューバーから直感的に操作できます。

1. **インストール＆起動**: `ClaudeDesktopSwitcher.app` をアプリケーションフォルダに入れて起動。メニューバーに常駐します。
2. **プロファイルの作成**: メニューの `Settings...` から、ワンクリックで「業務」「研究」など、独立したプロファイルを作成。
3. **分離環境での起動**: プロファイルを選択すると、Claudeデスクトップアプリが隔離された専用ディレクトリを参照して起動します。

#### 外部ターミナル環境との高度な連携 (CLI)

デスクトップアプリ内蔵のターミナルだけでなく、普段お使いの外部ターミナル（iTerm2等）でも同じプロファイルを活用できます。

ターミナル内で以下のコマンドを実行するだけで、指定環境へ即座にアクセスできます（Keychainの退避・復元を自動で行います）：

```bash
eval $(csw env <プロファイル名>)
```

*※ GUIとCLIのコンテキストは分離されているため、「デスクトップアプリを起動しただけでターミナルも自動的に切り替わる」ことはありません。ターミナルごとに上記のコマンドで明示的に連携させてください。*

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
