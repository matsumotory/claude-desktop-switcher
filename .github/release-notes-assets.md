### 添付ファイルの説明

- `Claude-Desktop-Switcher___VERSION___universal.dmg`: デスクトップアプリ本体です。Apple SiliconとIntelのどちらのMacでも動きます。
- `csw`: ターミナルで使うcswコマンド単体です。アプリだけを使う場合は、分離の検査が案内するリンクの修復を除いて不要です。
- `csw-desktop_*.cdx.json` / `csw-cli_*.cdx.json`: 部品表（SBOM）です。アプリとcswに含まれる全ライブラリの名前とバージョンを、標準のCycloneDX形式で列挙しています。配布物の中身を誰でも確かめられるように毎リリースに添付しており、読み方は[docs/PRIVACY.md](https://github.com/matsumotory/claude-desktop-switcher/blob/main/docs/PRIVACY.md)にあります。
- `RELEASE-README.md`: この説明のファイル版です。英語の説明も含みます。

### About the attached files

- `Claude-Desktop-Switcher___VERSION___universal.dmg`: the desktop app for both Apple Silicon and Intel Macs.
- `csw`: the standalone command-line tool. Not needed if you only use the app, except to repair links that the isolation check reports.
- `csw-desktop_*.cdx.json` / `csw-cli_*.cdx.json`: software bills of materials (SBOM) in the standard CycloneDX format, listing every library inside each binary, so anyone can verify what ships. See [docs/PRIVACY_EN.md](https://github.com/matsumotory/claude-desktop-switcher/blob/main/docs/PRIVACY_EN.md).
- `RELEASE-README.md`: this guide as a downloadable file, in Japanese and English.
