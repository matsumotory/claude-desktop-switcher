---
title: 検証可能な透明性パッケージ (PRIVACY 文書 + アプリ内開示 + cargo-deny/SBOM)
created: 2026-07-03
status: approved
related_prs: []
related_issues: []
---

# 検証可能な透明性パッケージ

## 背景・本セッションの完了事項

v0.18.0 で環境ごとの利用量表示を撤去した ([usage-display-removal.md](usage-display-removal.md))。理由は、CSW の原則 (通信しない・認証情報に触れない・他アプリのデータに触れない) を守ったまま実現する手段が無かったためである。

次の一手を決めるため、2026-07-03 に類似・隣接プロダクトのサーベイと多視点のアイデア出し・敵対検証・採点を行った。その結論として、macOS のローカル完結型ツールは「通信しない」と宣言するだけでは信頼されず、ユーザーが自分で確かめる手段まで公開しているツールが信頼を得ている、という事実が確認できた。CSW は撤去の判断で原則を守った直後であり、その原則を「検証できる形」にして公開することが次の一手として最も筋が良い。ユーザーは 4 案すべての実施を承認済みで、本 plan はその 1 本目である。

## ゴール

「CSW は通信しない・認証情報に触れない・他アプリのデータに触れない・既定では何も書き換えない」という 4 原則を、宣言ではなく検証可能な形で提供する。

1. どのファイルを読み書きし、何に絶対に触れないかを、日英の文書 (docs/PRIVACY.md / docs/PRIVACY_EN.md) にすべて列挙する。
2. ユーザーが自分の Mac で確かめる手順 (通信の確認・署名の確認・リンクの確認) を同じ文書に載せる。
3. 通信機能を持つライブラリが依存に紛れ込んだら CI が失敗する検査 (cargo-deny) を導入する。
4. リリースに部品表 (SBOM、CycloneDX 形式) を添付する。
5. アプリの「このアプリについて」に、読むもの・書くもの・触れないものの要約を載せる (日英)。
6. LP の FAQ と USER_GUIDE から PRIVACY 文書へ誘導する。

## 設計原則

- セキュリティ・プライバシー・倫理を利便性と引き換えにしない。本 plan は実行時挙動を一切変えず、原則の宣言と検証手段だけを追加する。
- 主張は実装と正確に一致させる (誇張しない)。特に次の 3 点は敵対検証で指摘済みの穴であり、仕様として固定する。
  - Cargo.lock には tauri が他 OS 向けに引く reqwest / hyper が載る。cargo-deny の対象を macOS ターゲット (aarch64/x86_64-apple-darwin) に絞った上で、この事実を PRIVACY 文書に先回りで明記し、主張を「macOS ビルドの依存グラフにネットワーククライアントが無い」に正確化する。
  - lsof の検証手順は、WKWebView の通信が別プロセス (WebKit ヘルパー) で行われる事実を踏まえて書く。「アプリ本体の PID の lsof が空 = 通信ゼロの証明」という単純化はしない。
  - open_url の記述は実装と一致させる。コマンド層は https スキームのみ許可し、UI 層が渡すのは固定の GitHub URL である。アプリ自体は通信せず、URL を OS に渡して既定ブラウザで開くだけである。
- PRIVACY 文書に記載するパス一覧は docs_impl_consistency_audit の監査対象に加え、実装との乖離を継続検出する。

## タスク詳細

### 変更ファイル一覧

| ファイル | 変更 |
|---|---|
| `deny.toml` (新規) | macOS ターゲットに絞った bans (ネットワーククライアント crate の禁止) |
| `.github/workflows/deny.yml` (新規) | PR ごとに cargo-deny check bans を ubuntu で実行 |
| `.github/workflows/release.yml` | リリース時に SBOM (CycloneDX JSON) を生成し release へ添付する ubuntu ジョブを追加 |
| `docs/PRIVACY.md` (新規・日本語) | 宣言・読むもの/書くもの/触れないものの列挙・実行する OS コマンド・検証手順 |
| `docs/PRIVACY_EN.md` (新規・英語) | 同上の英語版 |
| `docs/USER_GUIDE.md` / `docs/USER_GUIDE_EN.md` | 「知っておくべきこと」に確かめ方への誘導を追記 |
| `docs/SPECIFICATION.md` | 透明性の提供物 (PRIVACY 文書・deny CI・SBOM) を完了状態として追記 |
| `crates/desktop/ui/main.js` | 「このアプリについて」に読む/書く/触れないの要約と PRIVACY 文書への導線を追加 (日英辞書含む) |
| `website/ja/index.html` / `website/index.html` | FAQ の安全性の回答から PRIVACY 文書へ誘導 |
| `.agents/skills/docs_impl_consistency_audit/SKILL.md` | 監査対象サーフェスに PRIVACY 文書を追加 |
| `crates/core/src/profile/mod.rs` / `tests.rs` | 複製の source に既存の Claude (default) を拒否するガードを追加 (PRIVACY の「既存の Claude をまるごと読まない」主張をバックエンドで強制。敵対検証の指摘反映) |

### 実装ステップ

1. deny.toml を作成し、RED (対象を絞らない設定では reqwest 検出で失敗) → GREEN (macOS ターゲットでは通過) を実測する。
2. deny.yml (CI) を追加する。アクションは SHA ピン留め (EmbarkStudios/cargo-deny-action v2.0.20)。
3. release.yml に SBOM ジョブを追加する (cargo-cyclonedx、ubuntu、macOS ターゲット指定)。フラグはローカルで実測してから書く。
4. PRIVACY 文書 (ja/en) を書く。実行する OS コマンド (open / pgrep / ps / hdiutil) と読み書きパスは実装から列挙済みの事実だけを書く。
5. アプリの about ダイアログ・LP FAQ・USER_GUIDE に要約と導線を追加する。
6. 全サーフェス伝播の確認と整合性監査、日本語校正パスを通す。

### 検証計画

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
cargo deny check bans          # GREEN (macOS グラフにネットワーククライアント無し)
cargo cyclonedx (実測したフラグ) # SBOM が生成されること
```

- GUI (about ダイアログ) は `.claude/launch.json` の app (ブラウザ + devInvoke モック) で ja/en 両方の実描画を確認する。
- LP は launch の lp でレンダし、追記した FAQ 文の折り返し・横溢れを ja/en とも数値確認する。
- CI (Test / Build / Lint / Security / Deny) の green を一次エビデンスとする。

## リスクと対応

- **cargo-deny が既存依存で偽陽性を出す**: 対象を bans のみに絞り (advisories / licenses / sources は本 plan のスコープ外)、macOS ターゲットに限定する。実測で GREEN を確認してから CI に載せる。
- **SBOM 生成フラグの誤り**: ローカルで実測してから release.yml に書く。リリース時にしか動かないジョブなので、失敗時は次リリースで検知される。ジョブ失敗が DMG 公開を妨げないよう独立ジョブにする。
- **文書と実装の乖離 (将来)**: PRIVACY 文書のパス一覧を整合性監査の対象に加える。

## スコープ外 (混ぜない)

- csw doctor (分離の健全性検査) は次の plan で扱う。PRIVACY 文書の「書くパスの列挙」が doctor の検査正典になる関係だけ本文に明記する。
- ネットワーク遮断の実行時強制 (sandbox 等) は扱わない。
- 撤去済みの利用量表示の再検討はしない。

## 完了条件

- [ ] cargo deny check bans が RED → GREEN の実測を経て CI (PR) で green
- [ ] リリースに SBOM が添付される定義が release.yml に入っている
- [ ] docs/PRIVACY.md / PRIVACY_EN.md が存在し、実行する OS コマンドと読み書きパスが実装と一致
- [ ] about ダイアログ・LP FAQ (ja/en)・USER_GUIDE (ja/en) に導線がある
- [ ] 日本語校正パスを通し、禁止記号 (em-dash と米印類) の grep 残存ゼロ
- [ ] CI 全ジョブ green、squash マージ済み
