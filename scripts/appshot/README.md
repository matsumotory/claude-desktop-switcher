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
- 出力 (日英 8 枚): 日本語は `ja_screen_onboarding.png` / `ja_screen_overview.png` / `ja_screen_create.png` / `ja_hero.png`、英語は `screen_onboarding.png` / `screen_overview.png` / `screen_create.png` / `hero.png`。すべて `website/assets/` に書き出す。

## 撮影レシピ (出荷アセットと同じ)

- 幅 760 CSS、`deviceScaleFactor` 2 (= 1520px 幅)、ダークテーマ。
- `overview` / `onboarding`: アプリは固定ビューポート + 内部スクロール + 固定
  フッターなので、可視スクロール枠のはみ出し分だけウィンドウ高を伸ばして全内容を
  収めてから撮る (サイドバー全高・フッター固定を保つ)。
- `hero`: 760x498 のビューポート切り抜き (詳細画面の上部)。

UI のボタン名・レイアウト・オンボーディング文言を変えたら、このスクリプトで
スクショを再生成し、`docs/`・`website/` の記述と食い違わないようにする
(`propagate-changes-to-all-surfaces` / `verify-product-not-diff`)。

## 日本語と英語の両方を生成する

アプリ本体が日英対応 (i18n) になったため、英語スクショも同じ実 UI から撮る。
UI は OS/ブラウザのロケールで日英を判定し、撮影時は `?lang=ja|en` のクエリ上書きで
言語を確定させる。dev モックはロケールに応じてサンプル環境名 (日本語=仕事用/研究用/
検証用、英語=Work/Research/Testing) を返すので、各言語で自然に読める。日本語アセットは
`ja_` を付け、英語アセットは素の名前で書き出す。`main.js` / `index.html` のユーザー可視
文字列を変えたら本スクリプトで両言語を撮り直す。
