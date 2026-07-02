# ogshot: OG カード画像の生成

`website/assets/og.png` / `ja_og.png` (1200x630) を HTML/CSS テンプレートから
headless Chrome で再生成する。画像に焼き込まれるコピーをコード上のテキストとして
管理し、LP のコピー変更に追従できるようにする (画像エディタも生成 AI も使わない)。

## 使い方

```bash
node scripts/ogshot/gen-og.mjs
```

- 必要: Google Chrome、Node 21 以降、ネットワーク (LP と同じ Outfit フォントを取得)。
- コピーの一次ソースは LP の hero (`website/index.html` / `website/ja/index.html`)。
  文言を変えるときは `gen-og.mjs` の `LOCALES` を編集して再実行する。
- 禁止記号 (em-dash・※・＊) を使わない。日本語の折り返しは文節境界に
  `<span class="nb">` を当てて制御する (`japanese-typography-qa`)。
- 再生成したら `website/index.html` / `website/ja/index.html` の
  `og:image` / `twitter:image` のキャッシュバスター (`?v=`) を上げる。
