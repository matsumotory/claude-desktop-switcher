# appshot: アプリのスクリーンショット生成

`website/assets/` に置く CSW アプリのスクリーンショットを、実際の Tauri UI
(`crates/desktop/ui`) からヘッドレス Chrome + DevTools Protocol で再生成する。

UI は「dumb terminal」設計で、`window.__TAURI__` が無いときは `main.js` の dev
モック (`devInvoke`) にフォールバックする。そのため素の Chrome で実画面をサンプル
データ付きで描画でき、CDP で各画面の状態を作って撮影する。

## 使い方

```bash
node scripts/appshot/gen-screenshots.mjs
```

- 必要: Google Chrome、Node 21 以降 (global `WebSocket` / `fetch`)。
- 出力: `website/assets/ja_screen_onboarding.png` / `ja_screen_overview.png` / `ja_hero.png`。

## 撮影レシピ (出荷アセットと同じ)

- 幅 760 CSS、`deviceScaleFactor` 2 (= 1520px 幅)、ダークテーマ。
- `overview` / `onboarding`: アプリは固定ビューポート + 内部スクロール + 固定
  フッターなので、可視スクロール枠のはみ出し分だけウィンドウ高を伸ばして全内容を
  収めてから撮る (サイドバー全高・フッター固定を保つ)。
- `hero`: 760x498 のビューポート切り抜き (詳細画面の上部)。

UI のボタン名・レイアウト・オンボーディング文言を変えたら、このスクリプトで
スクショを再生成し、`docs/`・`website/` の記述と食い違わないようにする
(`propagate-changes-to-all-surfaces` / `verify-product-not-diff`)。

## 未対応: 英語スクリーンショット (TODO)

`website/index.html` (EN LP) と `docs/USER_GUIDE_EN.md` が使う英語 UI のスクショ
(`screen_onboarding.png` / `screen_overview.png` / `hero.png`) は、アプリに英語の
実行モードが無く、日本語 UI の文字列を英語へ差し替える辞書 (en-dict) を注入して
作られていた。その辞書はリポジトリに残っておらず、本スクリプトはまだ英語版を
生成しない。英語アセットを更新するには、`main.js` / `index.html` のユーザー可視
文字列を EN LP の用語に合わせて英訳した辞書を用意し、撮影前に DOM へ適用する処理を
足す必要がある。
