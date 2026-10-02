# Claude Desktop Switcher __VERSION__ の配布物

このリリースに含まれる全ファイルの説明です。An English version follows the Japanese.

## 日本語

- `Claude-Desktop-Switcher___VERSION___universal.dmg`：デスクトップアプリ本体です。Apple SiliconとIntelのどちらのMacでも動きます。Developer IDで署名し、Appleの公証を受けています。
- `csw`：ターミナルで使うcswコマンド単体です。アプリだけを使う場合は、分離の検査が案内するリンクの修復を除いて不要です。
- `csw-desktop_aarch64-apple-darwin.cdx.json` / `csw-desktop_x86_64-apple-darwin.cdx.json`：デスクトップアプリの部品表（SBOM）です。アプリに含まれる全ライブラリの名前とバージョンを、標準のCycloneDX形式で列挙しています。
- `csw-cli_aarch64-apple-darwin.cdx.json` / `csw-cli_x86_64-apple-darwin.cdx.json`：cswコマンドの部品表です。
- `RELEASE-README.md`：このファイルです。

部品表は、配布物の中身を誰でも確かめられるように毎リリースに添付しています。通信するライブラリが入っていないことの確認や、脆弱性スキャナでの検査に使えます。読み方と確かめ方は[docs/PRIVACY.md](https://github.com/matsumotory/claude-desktop-switcher/blob/main/docs/PRIVACY.md)にあります。

## English

- `Claude-Desktop-Switcher___VERSION___universal.dmg`: the desktop app. Runs on both Apple Silicon and Intel Macs. Signed with a Developer ID and notarized by Apple.
- `csw`: the standalone command-line tool. Not needed if you only use the app, except to repair links that the isolation check reports.
- `csw-desktop_aarch64-apple-darwin.cdx.json` / `csw-desktop_x86_64-apple-darwin.cdx.json`: software bills of materials (SBOM) for the desktop app, listing every library and version inside the build in the standard CycloneDX format.
- `csw-cli_aarch64-apple-darwin.cdx.json` / `csw-cli_x86_64-apple-darwin.cdx.json`: the same for the csw command-line tool.
- `RELEASE-README.md`: this file.

The SBOMs are attached to every release so anyone can verify what ships: check that no networking library is included, or feed them to a vulnerability scanner. See [docs/PRIVACY_EN.md](https://github.com/matsumotory/claude-desktop-switcher/blob/main/docs/PRIVACY_EN.md) for how to read and verify them.
