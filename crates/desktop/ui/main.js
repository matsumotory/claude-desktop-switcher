// ============================================================================
// Claude Desktop Switcher — settings UI logic (Tauri v2)
// The WebView is a "dumb terminal": all profile logic lives in Rust.
// When window.__TAURI__ is absent (plain browser, e.g. screenshots) a dev mock
// stands in. The shipped app sets withGlobalTauri:true and always reaches Rust.
//
// All dynamic UI is built with createElement + textContent (never innerHTML),
// so backend/user strings are never parsed as HTML — XSS is structurally
// impossible.
// ============================================================================

const HAS_TAURI = !!(window.__TAURI__ && window.__TAURI__.core);
const invoke = HAS_TAURI ? window.__TAURI__.core.invoke : devInvoke;

// --- Internationalization (ja default, en fallback) -------------------------
// The app ships Japanese copy inline. When the OS/browser locale is not Japanese
// we render English. Standalone strings are translated by matching the exact
// Japanese text against EN below (applied to the static HTML at load and to every
// node the app builds afterwards, via a MutationObserver). Strings that embed a
// name or a number are translated at construction with T(ja, en).
// Locale detection: Japanese when the OS/browser locale is Japanese, else English.
// A ?lang=ja|en query override makes screenshot capture deterministic regardless of
// the headless browser locale; the shipped app is loaded without query params.
const LANG = (() => {
  const forced = new URLSearchParams(location.search).get('lang');
  if (forced === 'ja' || forced === 'en') return forced;
  return String(navigator.language || navigator.userLanguage || 'en').toLowerCase().startsWith('ja') ? 'ja' : 'en';
})();
const T = (ja, en) => (LANG === 'ja' ? ja : en);
// Japanese -> English for every standalone visible string. Keys must match the
// rendered text exactly (trimmed). Terminology follows the shipped EN landing page
// and the established English screenshots (Existing Claude / In use / Work etc.).
const EN = {
  // Sidebar / chrome
  'Claude Desktop Switcher': 'Claude Desktop Switcher',
  '既存の Claude': 'Existing Claude',
  'あなたの標準環境である既存の Claude': 'Existing Claude, your standard environment',
  '利用中': 'In use',
  '環境を作る': 'New environment',
  'テーマ': 'Theme',
  'ユーザーガイド': 'User guide',
  '問題を報告': 'Report an issue',
  'このアプリについて': 'About',
  '更新を確認': 'Check for updates',
  'ブルー': 'Blue', 'ティール': 'Teal', 'インディゴ': 'Indigo', 'テラコッタ': 'Terracotta',
  // Empty / onboarding
  'はじめまして': 'Welcome',
  '環境を分けて、混ぜない。': 'Separate environments. Never mixed.',
  '既存の Claude はそのまま。そこを基準に、何を引き継ぎ何を分けるかを決めて、新しい環境を作ります。':
    'Your existing Claude stays untouched. Starting from it, decide what to carry over and what to separate, then create a new environment.',
  'いまの環境はそのまま残ります': 'Your current setup stays intact',
  'あなたが普段使っている Claudeデスクトップアプリと Claude Code の環境は「既存の Claude」として保護され、設定・履歴・ログインは変更されません。':
    'Your everyday Claude Desktop App and Claude Code setup is protected as "Existing Claude"; its settings, history, and sign-in are never changed.',
  '環境を切り替えて使えます': 'Switch between environments',
  '「利用中」は、いま Claude が起動している環境です。Claude を終了すれば「利用中」は外れ、その環境はまた「切り替えて起動」から開き直せます。設定を共有する環境は衝突を防ぐため一度に1つずつ開くので、別の環境に切り替えるときは先に起動中の Claude を終了します。「すべて分ける」で作った環境は何も共有しないので、起動中の Claude を終了せず、新しいウィンドウで並べて開けます。':
    '"In use" marks the environment Claude is currently running for. Quit Claude and it clears, so you can reopen that environment from "Switch and launch". Environments that share settings open one at a time to avoid conflicts, so quit the running Claude before switching to another. An environment set to "separate everything" shares nothing, so you can open it in a new window alongside a running Claude, without quitting.',
  'ターミナルの Claude Code も同じ環境で使えます': 'Claude Code in the terminal uses the same environment',
  // Detail: taglines, sections, sharing
  'あなた自身の環境': 'Your own setup',
  '利用中の環境': 'Environment in use',
  '作成した環境': 'Created environment',
  'Claudeデスクトップアプリ': 'Claude Desktop App',
  'Claude Code': 'Claude Code',
  'この環境が引き継いでいるもの': 'What this environment inherits',
  '「共有」は既存の Claude と中身を共通にすること、「コピー」は作成時に一度だけ写して以後は別々、「分離」はこの環境だけで持つことです。':
    '"Shared" keeps the contents in common with your existing Claude; "Copied" duplicates once at creation and they diverge after; "Isolated" means this environment keeps its own.',
  'ワークツリー一覧': 'Worktrees', 'Git ワークツリーと repo の対応': 'Mapping of Git worktrees to repos',
  '共通ルール': 'Global rules', 'CLAUDE.md に書いた常時ルール': 'Always-on rules in CLAUDE.md',
  'プラグイン': 'Plugins', '導入したプラグイン': 'Installed plugins',
  'スキル': 'Skills', 'カスタムスキル': 'Custom skills',
  'ツール権限・フック': 'Tool permissions & hooks', 'ツールの実行可否とフック': 'Whether tools may run, and hooks',
  'プロジェクトの会話・メモリ': 'Project conversations & memory', 'プロジェクトごとの会話履歴と自動メモリ': 'Per-project conversation history and auto-memory',
  '入力履歴': 'Input history', '入力したプロンプトの履歴': 'History of entered prompts',
  '端末 ID': 'Device ID', '端末を識別するための ID': 'ID that identifies your device',
  '共有': 'Shared', 'コピー': 'Copied', '分離': 'Isolated',
  'この環境のデータの場所': "Location of this environment's data",
  'あなたの Claude フォルダの場所': 'Location of your Claude folder',
  'これは CSW が作った場所ではなく、あなた自身の Claude フォルダです。': 'This is not a folder CSW created; it is your own Claude folder.',
  'ターミナルで Claude Code を使う': 'Use Claude Code in the terminal',
  'CSW から開いたアプリの中で使うターミナルは、最初からこの環境です。コマンドは要りません。自分で別に開いた iTerm2 などのターミナルで対象の環境に揃えるには、環境名を渡して次を実行します。この設定はそのタブだけに効き、普段の環境には影響しません。':
    "A terminal opened inside the app you launched from CSW is already in this environment, so no command is needed. To switch a terminal you opened yourself, such as iTerm2, to a target environment, pass the environment name and run the command below. It applies to that tab only and never affects your usual environment.",
  'あなた自身の Claudeデスクトップアプリと Claude Code の環境です。CSW はここを表示しているだけで、設定・履歴・ログインを変更したり削除したりしません。':
    'This is your own Claude Desktop App and Claude Code setup. CSW only displays it; it never changes or deletes your settings, history, or sign-in.',
  // Buttons / actions
  '複製': 'Duplicate', '削除': 'Delete',
  'この環境の複製': 'Duplicate this environment',
  '設定をそのままコピーして、別の名前の環境を作ります。元の環境は変わりません。': 'Copies the settings as they are to create a new environment under a different name. The original is left unchanged.',
  '起動のしかた': 'How to launch',
  '切り替えて起動': 'Switch and launch', '重複して起動': 'Launch alongside',
  'この環境に切り替えて Claude を開きます。ほかの環境の Claude が起動しているときは切り替えられないので、先に終了してください。':
    'Opens this environment and switches to it. If another environment’s Claude is running you cannot switch, so quit it first.',
  '起動中の Claude を終了せずに、この環境を新しいウィンドウで同時に開きます。':
    'Opens this environment in a new window at the same time, without quitting a running Claude.',
  '既存の Claude に切り替える': 'Switch to Existing Claude',
  '既存の Claude は変更・削除できません': 'Existing Claude cannot be changed or deleted',
  '完全に削除': 'Delete permanently',
  'ゴミ箱へ移動': 'Move to Trash',
  '完全に削除する': 'Delete permanently',
  '完全に削除すると、ゴミ箱を経由せずにすぐ消え、戻せません。':
    'Permanent deletion skips the Trash, takes effect immediately, and cannot be undone.',
  'ゴミ箱へ移動できませんでした。すぐに消す場合は「完全に削除」を選んでください。':
    'Could not move it to the Trash. To remove it now, choose "Delete permanently".',
  'やめる': 'Cancel', '複製を作る': 'Duplicate', '戻る': 'Back',
  'この環境は、起動中の Claude を終了せずに開けます。': 'You can open this environment without quitting a running Claude.',
  '起動中の Claude を終了してから、もう一度押してください。設定の衝突を防ぐため、共有を含む環境の Claude は同時に開けません。':
    'Quit the running Claude, then press this again. To avoid configuration conflicts, environments that share settings cannot be open at once.',
  '閉じる': 'Close', 'もう一度試す': 'Try again',
  // Create flow
  '新しい環境を作る': 'Create a new environment',
  'この環境を作る': 'Create this environment', '作成する': 'Create',
  '各項目を、既存の Claude と共有するか、この環境だけにするかを選びます。': 'For each item, choose whether to share it with your existing Claude or keep it only in this environment.',
  '共有 = いまのを使う ／ 分離 = この環境だけ ／ コピー = 最初だけ写す': 'Shared = use the current one / Isolated = only this environment / Copied = copied once at the start',
  'アカウントのサインイン情報（config.json）と、コネクタ・アプリ設定（claude_desktop_config.json）は、アカウント別の情報を含むため、どのモードでも必ずこの環境だけに分離します。':
    'The account sign-in (config.json) and the connector/app settings (claude_desktop_config.json) contain per-account data, so they are always isolated to this environment in every mode.',
  '2つのアカウントが同じ端末として結び付かないよう、端末 ID は常に分離します。': 'The device ID is always isolated so two accounts are not linked as the same device.',
  'カスタム設定': 'Custom', 'モードの既定どおり': 'Mode defaults',
  // Misc / toasts (standalone)
  'コピーしました': 'Copied', 'コピーできませんでした。': 'Could not copy.',
  '環境を読み込めませんでした。': 'Could not load the environment.',
  '切り替えできませんでした。もう一度お試しください。': 'Could not switch. Please try again.',
  '起動できませんでした。もう一度お試しください。': 'Could not launch. Please try again.',
  '削除できませんでした。利用中の環境は切り替えてから削除してください。': 'Could not delete. Switch away from an in-use environment before deleting it.',
  '複製できませんでした。同じ名前がすでにあるか確認してください。': 'Could not duplicate. Check whether the same name already exists.',
  '作成できませんでした。同じ名前がすでにあるか確認してください。': 'Could not create. Check whether the same name already exists.',
  'モードを変えたので高度設定をリセットしました': 'Changed the mode, so advanced settings were reset',
  'リンクを開けませんでした。': 'Could not open the link.',
  '既存の Claude に切り替えました': 'Switched to Existing Claude',
  '取り出す': 'Eject', 'あとで': 'Later',
  'ディスクイメージを取り出しました': 'Disk image ejected',
  '取り出せませんでした。ディスクイメージを使用中のウィンドウを閉じてから、もう一度お試しください。': 'Could not eject. Close any window using the disk image, then try again.',
  'パスをコピー': 'Copy path', 'コマンドをコピー': 'Copy command',
  '複製先の名前。例: 仕事用-控え': 'Name for the copy. e.g. Work-backup',
  '名前を入力してください。': 'Enter a name.',
  '"default" は使えません。いまの環境を指す予約名です。': '"default" is reserved for the current environment and cannot be used.',
  '名前は64文字までにしてください。': 'Use at most 64 characters.',
  '使えるのは文字・数字・ハイフン・アンダースコアだけです。空白や記号は使えません。': 'Use letters, digits, hyphens and underscores only. No spaces or symbols.',
  '既存の Claude が標準の場所に見つかりません。引き継げるものが無いため、「すべて分ける」だけで作成できます。':
    'Your existing Claude was not found at the standard locations. There is nothing to carry over, so you can create only with "Separate everything".',
  'Claude Code（CLI）の設定が標準の場所に見つかりません。引き継ぐモードを選んでも、ルールやスキル、会話などの CLI 側は引き継がれません。':
    'No Claude Code (CLI) settings were found at the standard location. Even if you choose a mode that carries things over, the CLI side (rules, skills, and conversations) is not carried over.',
  // About dialog
  '非公式のオープンソースのコミュニティプロジェクトです。': 'An unofficial, open-source community project.',
  '無保証': 'No warranty',
  '本ソフトウェアは MIT ライセンスのもとで、そのままの状態で提供されます。動作や品質について、いかなる保証もありません。': 'This software is provided as is under the MIT License, with no warranty of any kind as to operation or quality.',
  '自己責任でのご利用': 'Use at your own risk',
  'ご利用は自己責任でお願いします。データの損失や不具合などについて、作者は責任を負いません。大切なデータは、お試しになる前にバックアップしておくことをおすすめします。': 'Use at your own risk. The author is not responsible for data loss or malfunctions. Back up important data before trying it.',
  'プライバシー': 'Privacy',
  'CSW はインターネット通信も、利用状況の送信も行いません。パスワードを保管する macOS のキーチェーンにも触れず、すべて手元のフォルダと設定の操作だけで動きます。': 'CSW makes no network requests and sends no usage data. It never touches the macOS Keychain that stores your passwords; it works only with local folders and settings.',
  '何を読み書きするか': 'What it reads and writes',
  '環境のデータの書き込み先は、CSW 専用のフォルダ ~/.context-switcher-claude/ の中だけです。読む場所・書く場所の詳しい一覧と、通信していないことをご自身の Mac で確かめる手順は、プライバシーと透明性の文書で公開しています。':
    'Environment data is written only inside CSW\'s own folder, ~/.context-switcher-claude/. The full lists of what it reads and writes, and the steps to verify on your own Mac that it makes no network requests, are published in the Privacy and Transparency document.',
  '確かめ方を見る': 'See how to verify',
  '非公式プロジェクト': 'Unofficial project',
  // Isolation check (csw doctor)
  '分離の検査': 'Isolation check',
  '共有と分離が設定どおりに保たれているかを、この場で点検します。ファイルの中身は読まず、何も書き換えません。':
    'Checks on the spot that sharing and isolation still match this environment\'s settings. It reads no file contents and changes nothing.',
  '検査する': 'Check now',
  '検査できませんでした。': 'The check could not be run.',
  'この環境の Claudeデスクトップアプリが起動中です。書き換えの途中を検査した可能性があるため、問題が出た場合は Claude を終了してからもう一度検査してください。':
    'The Claude Desktop App is running in this environment. The check may have caught a rewrite in progress; if issues appear, quit Claude and check again.',
  '正常に共有': 'Shared (OK)',
  '正常に分離': 'Isolated (OK)',
  '正常にコピー': 'Copied (OK)',
  '共有元がまだありません': 'Nothing to share yet',
  '要確認': 'Needs attention',
  '共有元が見つかりません。': 'The shared source is missing.',
  'リンク先が想定と異なります。ターミナルで csw doctor --fix を実行すると張り直せます。':
    'The link points somewhere unexpected. Run csw doctor --fix in a terminal to re-point it.',
  'リンク先が想定と異なり、共有元も見つかりません。':
    'The link points somewhere unexpected, and the shared source is missing.',
  '共有のリンクが実体のファイルに置き換わっています。手元の内容を失わないよう、自動では直しません。':
    'The share link has been replaced by a regular file. To avoid losing your local contents, it is not repaired automatically.',
  '共有のリンクがありません。': 'The share link is missing.',
  '常に分離する項目がリンクになっています。': 'An always-isolated item has become a link.',
  'コピーの項目がリンクになっています。': 'An item set to copy has become a link.',
  '分離の項目がリンクになっています。': 'An isolated item has become a link.',
  // Data map (場所のデータの内訳)
  '読み込んでいます…': 'Loading…',
  '内訳を読み込めませんでした。': 'The breakdown could not be loaded.',
  'Finder で表示': 'Show in Finder',
  'Finder で表示できませんでした。': 'Could not show it in Finder.',
  '常に分離': 'Always isolated',
  '既存の Claude と共有': 'Shared with your existing Claude',
  'まだありません': 'Not present yet',
  '状態を判定できませんでした。': 'The state could not be determined.',
  'セッション状態': 'Session state',
  'アカウントのサインイン情報': 'Account sign-in',
  'コネクタ・アプリ設定': 'Connectors & app settings',
  '本プロジェクトは非公式のコミュニティ製で、Anthropic 社とは関係ありません。「Claude」は Anthropic の商標です。': 'This is an unofficial community project, not affiliated with Anthropic. "Claude" is a trademark of Anthropic.',
  // Avatar picker labels
  '仕事': 'Work', '個人': 'Personal', '検証': 'Testing', '開発': 'Dev', '会社': 'Company',
  '学習': 'Study', 'デザイン': 'Design', 'ローンチ': 'Launch', 'プロジェクト': 'Project',
  'お気に入り': 'Favorite', 'グローバル': 'Global', 'ハート': 'Heart', 'ツール': 'Tool',
  'アイデア': 'Idea', '資料': 'Docs', '会話': 'Chat',
  // Static create-view strings and chrome aria-labels.
  '環境一覧': 'Environments', 'テーマカラー': 'Accent color', 'ヘルプ': 'Help', 'アイコンを選ぶ': 'Choose an icon', '分け方': 'How to separate',
  '名前': 'Name', '例: 仕事用、検証用': 'e.g. Work, Testing', 'アイコン（任意）': 'Icon (optional)',
  'メモ（任意）': 'Note (optional)', 'この環境のメモ': 'Note for this environment',
  '編集': 'Edit', '保存': 'Save', 'メモを書く': 'Add a note',
  '例: 仕事用。会社の Google アカウントでサインイン': 'e.g. Work. Signs in with the company Google account',
  '既存の Claude から、どう分けますか?': 'How do you want to separate this from your existing Claude?',
  'どのモードでも、サインインするアカウントは環境ごとに分かれます。課金や利用量はそのアカウントにひも付きます。違いは、会話・メモリ・設定をどこまで引き継ぐかです。':
    'In every mode the signed-in account is separate per environment, and billing and usage follow that account. What differs is how much of your conversations, memory, and settings carry over.',
  'アカウントだけ分ける': 'Separate the account only',
  '会話履歴も自動メモリも、共通ルールやスキルもそのまま引き継ぎます。分けるのは課金アカウントだけなので、研究と開発で利用量を分けつつ作業を続けられます。':
    'Conversation history, auto-memory, global rules and skills all carry over. Only the billing account is separated, so you can keep working while splitting usage between, say, research and development.',
  'アカウント': 'Account', '会話・メモリ': 'Conversations & memory',
  '会話とメモリも分ける': 'Separate conversations and memory too',
  '共通ルール・スキル・プラグインとツール権限は引き継ぎ、会話履歴と自動メモリはこの環境だけにします。慣れた設定はそのままに、用途ごとに会話を分けたいときに向いています。':
    'Global rules, skills, plugins and tool permissions carry over, while conversation history and auto-memory stay in this environment only. Good when you want your familiar setup but separate conversations per purpose.',
  'ルール・設定': 'Rules & settings',
  'すべて分ける': 'Separate everything', '推奨': 'Recommended',
  'ルールも設定も会話履歴もメモリも、何も引き継がずにまっさらな環境を作ります。既存の Claude には一切触れないので、案件やクライアント、仕事と個人を完全に分けられます。':
    'Creates a blank environment that carries nothing over: no rules, settings, conversation history, or memory. It never touches your existing Claude, so you can fully separate projects, clients, or work and personal use.',
  '設定・会話': 'Settings & conversations',
  '項目ごとに詳しく設定する': 'Configure in detail, item by item',
  'モードの既定に戻す': 'Reset to mode defaults',
};

// Elements holding user data (environment names, filesystem paths, shell command)
// must never be run through the dictionary: a user could name an environment
// "仕事" and it must not become "Work". The default row's "既存の Claude" is
// translated at construction with T() instead, so these can be skipped wholesale.
const I18N_SKIP = '.profile-name, .detail-name, .profile-note, .detail-note, .path-code, .code-line, code';
const setEN = (v) => (Object.prototype.hasOwnProperty.call(EN, v) ? EN[v] : null);

function applyI18nText(node) {
  if (LANG === 'ja') return;
  if (node.nodeType === 3) {
    if (node.parentElement && node.parentElement.closest(I18N_SKIP)) return;
    const en = setEN(node.textContent.trim());
    if (en != null) node.textContent = node.textContent.replace(node.textContent.trim(), en);
    return;
  }
  if (node.nodeType !== 1) return;
  // Elements carrying an explicit English phrase (composed markup: <strong>+text,
  // inline <code>) are replaced whole and not descended into.
  const whole = [];
  if (node.hasAttribute('data-en')) whole.push(node);
  node.querySelectorAll('[data-en]').forEach((e) => whole.push(e));
  for (const e of whole) e.textContent = e.getAttribute('data-en');
  // Attributes that surface as visible text, on this node and every descendant.
  const attrEls = [node, ...node.querySelectorAll('[placeholder], [aria-label], [title]')];
  for (const e of attrEls) {
    for (const attr of ['placeholder', 'aria-label', 'title']) {
      if (!e.hasAttribute || !e.hasAttribute(attr)) continue;
      const en = setEN(e.getAttribute(attr).trim());
      if (en != null) e.setAttribute(attr, en);
    }
  }
  // Remaining text nodes, skipping user-data containers and data-en subtrees.
  const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT, {
    acceptNode: (tn) =>
      tn.parentElement && tn.parentElement.closest(I18N_SKIP + ', [data-en]')
        ? NodeFilter.FILTER_REJECT
        : NodeFilter.FILTER_ACCEPT,
  });
  const texts = [];
  let n;
  while ((n = walker.nextNode())) texts.push(n);
  for (const tn of texts) {
    const en = setEN(tn.textContent.trim());
    if (en != null) tn.textContent = tn.textContent.replace(tn.textContent.trim(), en);
  }
}

if (LANG === 'en') {
  document.documentElement.lang = 'en';
  // main.js runs at end of <body>, so the static HTML already exists: translate it,
  // then observe for everything the app builds afterwards.
  applyI18nText(document.body);
  new MutationObserver((muts) => {
    for (const m of muts) for (const added of m.addedNodes) applyI18nText(added);
  }).observe(document.body, { childList: true, subtree: true });
}

// --- The sharing components the user can tune, grouped and described in plain
// language. config.json (OAuth tokens) and claude_desktop_config.json (account
// state, rewritten on launch) are deliberately absent: they are always isolated
// and never offered as a choice. -------------------------------------------
const SHARE_GROUPS = [
  {
    label: 'Claudeデスクトップアプリ',
    items: [
      { key: 'desktop_worktrees', name: 'ワークツリー一覧', desc: 'Git ワークツリーと repo の対応' },
    ],
  },
  {
    label: 'Claude Code',
    items: [
      { key: 'cli_claude_md', name: '共通ルール', desc: 'CLAUDE.md に書いた常時ルール' },
      { key: 'cli_plugins', name: 'プラグイン', desc: '導入したプラグイン' },
      { key: 'cli_skills', name: 'スキル', desc: 'カスタムスキル' },
      { key: 'cli_settings', name: 'ツール権限・フック', desc: 'ツールの実行可否とフック' },
      { key: 'cli_project_memory', name: 'プロジェクトの会話・メモリ', desc: 'プロジェクトごとの会話履歴と自動メモリ' },
      { key: 'cli_history', name: '入力履歴', desc: '入力したプロンプトの履歴' },
    ],
  },
];
const DEVICE_ID = { key: 'desktop_device_id', name: '端末 ID', desc: '端末を識別するための ID' };
const ALL_KEYS = [...SHARE_GROUPS.flatMap((g) => g.items.map((i) => i.key)), DEVICE_ID.key];

// Labels for the isolation check (csw doctor). Reuses the share-UI vocabulary;
// the always-isolated items the share UI deliberately hides get their
// SPECIFICATION.md §3 names here.
const DOCTOR_LABELS = {
  ...Object.fromEntries(SHARE_GROUPS.flatMap((g) => g.items).map((i) => [i.key, i.name])),
  [DEVICE_ID.key]: DEVICE_ID.name,
  cli_sessions: 'セッション状態',
  desktop_app_config: 'アカウントのサインイン情報',
  desktop_config: 'コネクタ・アプリ設定',
};

// Mode presets — must mirror SharingConfig::{share_settings,share_workspace}_preset
// and build_sharing_config in main.rs. config.json / claude_desktop_config.json are
// always isolated and are not part of these maps.
const PRESETS = {
  // すべて分ける: a fully separated environment, nothing carried over.
  isolate: Object.fromEntries(ALL_KEYS.map((k) => [k, 'isolate'])),
  // 会話とメモリも分ける: reuse the common setup, keep conversation history + memory and the account separate.
  share_settings: {
    cli_claude_md: 'share',
    cli_plugins: 'share',
    cli_skills: 'share',
    cli_settings: 'copy',
    desktop_worktrees: 'copy',
    cli_project_memory: 'isolate',
    cli_history: 'isolate',
    desktop_device_id: 'isolate',
  },
  // アカウントだけ分ける: also carry conversation history + memory across, separating only the account.
  share_workspace: {
    cli_claude_md: 'share',
    cli_plugins: 'share',
    cli_skills: 'share',
    cli_settings: 'copy',
    desktop_worktrees: 'copy',
    cli_project_memory: 'share',
    cli_history: 'share',
    desktop_device_id: 'isolate',
  },
};

// --- App state --------------------------------------------------------------
let profiles = [];
let activeName = 'default';
let desktopRunning = false; // whether any Claude Desktop is running right now
let runningProfiles = []; // names of environments whose Claude is running (can be several)
let selectedName = null;
let currentMode = 'isolate';
let overrides = { ...PRESETS.isolate };
let advancedCustomized = false;

// --- DOM builder helpers (no innerHTML) -------------------------------------
const SVG_NS = 'http://www.w3.org/2000/svg';

function h(tag, props, ...kids) {
  const node = document.createElement(tag);
  if (props) {
    for (const k in props) {
      const v = props[k];
      if (v == null || v === false) continue;
      if (k === 'class') node.className = v;
      else if (k === 'text') node.textContent = v;
      else if (k === 'dataset') Object.assign(node.dataset, v);
      else if (k.startsWith('on') && typeof v === 'function') node.addEventListener(k.slice(2), v);
      else if (k === 'disabled' || k === 'checked' || k === 'hidden') { if (v) node[k] = true; }
      else node.setAttribute(k, v);
    }
  }
  appendKids(node, kids);
  return node;
}

function appendKids(node, kids) {
  for (const c of kids.flat(Infinity)) {
    if (c == null || c === false) continue;
    node.appendChild(typeof c === 'object' ? c : document.createTextNode(String(c)));
  }
}

function icon(id, cls) {
  const svg = document.createElementNS(SVG_NS, 'svg');
  svg.setAttribute('class', cls || 'icon');
  const use = document.createElementNS(SVG_NS, 'use');
  use.setAttribute('href', '#' + id);
  svg.appendChild(use);
  return svg;
}

const $ = (id) => document.getElementById(id);

// Avatar icons: a curated set drawn from the app's Phosphor sprite (never
// hand-rolled). A profile's `icon` is either one of these slugs (rendered as the
// SVG glyph) or a short emoji/character (rendered as text); empty falls back to
// the first letter of the name.
const AVATAR_ICONS = [
  { slug: 'briefcase', label: '仕事' },
  { slug: 'user', label: '個人' },
  { slug: 'flask', label: '検証' },
  { slug: 'code', label: '開発' },
  { slug: 'buildings', label: '会社' },
  { slug: 'graduation-cap', label: '学習' },
  { slug: 'palette', label: 'デザイン' },
  { slug: 'rocket', label: 'ローンチ' },
  { slug: 'folder', label: 'プロジェクト' },
  { slug: 'star', label: 'お気に入り' },
  { slug: 'globe', label: 'グローバル' },
  { slug: 'heart', label: 'ハート' },
  { slug: 'wrench', label: 'ツール' },
  { slug: 'lightbulb', label: 'アイデア' },
  { slug: 'book-open', label: '資料' },
  { slug: 'chat-circle', label: '会話' },
];
const ICON_SLUGS = AVATAR_ICONS.map((i) => i.slug);

function avatarContent(iconVal, name) {
  if (iconVal && ICON_SLUGS.includes(iconVal)) return icon('i-' + iconVal, 'icon avatar-glyph');
  if (iconVal) return iconVal;
  return (name || '?').charAt(0).toUpperCase();
}
const reduceMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
function withTransition(update) {
  if (document.startViewTransition && !reduceMotion()) document.startViewTransition(update);
  else update();
}

// --- DOM refs ---------------------------------------------------------------
const el = {
  profileList: $('profileList'),
  dmgBanner: $('dmgBanner'),
  btnDmgEject: $('btnDmgEject'),
  btnDmgLater: $('btnDmgLater'),
  btnCreate: $('btnCreate'),
  btnCreateEmpty: $('btnCreateEmpty'),
  viewEmpty: $('viewEmpty'),
  viewDetail: $('viewDetail'),
  viewCreate: $('viewCreate'),
  detailContent: $('detailContent'),
  detailFooter: $('detailFooter'),
  emptyFirstRun: $('emptyFirstRun'),
  firstRunExtra: $('firstRunExtra'),
  inputName: $('inputName'),
  inputNote: $('inputNote'),
  iconPicker: $('iconPicker'),
  createNotice: $('createNotice'),
  nameError: $('nameError'),
  advancedToggle: $('advancedToggle'),
  advancedState: $('advancedState'),
  advancedReset: $('advancedReset'),
  advancedGroups: $('advancedGroups'),
  createFooter: $('createFooter'),
  emptyScroll: $('emptyScroll'),
  emptyFade: $('emptyFade'),
  detailScroll: $('detailScroll'),
  detailFade: $('detailFade'),
  createScroll: $('createScroll'),
  createFade: $('createFade'),
  toastArea: $('toastArea'),
  appVersion: $('appVersion'),
  btnAbout: $('btnAbout'),
};

// --- Toast ------------------------------------------------------------------
function showToast(msg, isError) {
  const t = h('div', { class: 'toast' + (isError ? ' error' : '') },
    icon(isError ? 'i-info' : 'i-check'), h('span', { text: msg }));
  if (isError) t.setAttribute('role', 'alert'); // announce failures assertively
  el.toastArea.appendChild(t);
  setTimeout(() => {
    t.style.transition = 'opacity .2s';
    t.style.opacity = '0';
    setTimeout(() => t.remove(), 220);
  }, 3200);
}

// --- View switching ---------------------------------------------------------
function setView(view) {
  el.viewEmpty.hidden = view !== 'empty';
  el.viewDetail.hidden = view !== 'detail';
  el.viewCreate.hidden = view !== 'create';
  requestAnimationFrame(refreshFades);
}

// An environment is "in use" (利用中) when a Claude Desktop instance is actually
// running for it. Fully-isolated environments can run side by side, so several may
// be in use at once. Resolved per environment by the backend from the live
// processes' --user-data-dir (csw_core::switcher::desktop_dir_running); quitting a
// Claude clears its marker so that environment can be launched again.
const inUse = (name) => runningProfiles.includes(name);

// --- Sidebar ----------------------------------------------------------------
function renderSidebar() {
  // One rail: "既存の Claude" (default) pinned first, then created environments.
  // default is a selectable row like any other, not a separate card. Pin it
  // explicitly so order never depends on the backend's list_profiles order.
  const created = profiles.filter((p) => p.name !== 'default');
  const defaultP = profiles.find((p) => p.name === 'default') || { name: 'default', icon: '', is_default: true };
  const ordered = [defaultP, ...created];

  const rows = ordered.map((p) => {
    const isDefault = p.name === 'default';
    const isInUse = inUse(p.name);
    const open = () => withTransition(() => showDetail(p.name));
    // One pill at most: the "利用中" row, shown only while Claude Desktop is
    // actually running for it. The default row needs no extra marker — its name
    // ("既存の Claude") + monitor icon + first position already signal the baseline.
    const pill = isInUse
      ? h('span', { class: 'pill pill-active', text: '利用中' })
      : null;
    // Sub lines under the name: the user's own note first (never translated),
    // then the launch stamp. Rows without them keep the compact height.
    const subs = [];
    if (!isDefault && p.note) subs.push(h('span', { class: 'profile-note', text: p.note }));
    if (!isDefault && p.last_launched_at) {
      subs.push(h('span', {
        class: 'profile-meta',
        text: T(`最終起動: ${relTime(p.last_launched_at)}`, `Last launched ${relTime(p.last_launched_at)}`),
      }));
    }
    return h('li', {
      class: 'profile-item' + (p.name === selectedName ? ' selected' : ''),
      role: 'button', tabindex: '0', onclick: open,
      'aria-label': isDefault ? 'あなたの標準環境である既存の Claude' : null,
      'aria-current': isInUse ? 'true' : null, // current = the running environment
      onkeydown: (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); open(); } },
    },
      h('span', { class: 'profile-avatar' }, isDefault ? icon('i-monitor') : avatarContent(p.icon, p.name)),
      h('span', { class: 'profile-text' },
        h('span', { class: 'profile-top' },
          h('span', { class: 'profile-name', text: isDefault ? T('既存の Claude', 'Existing Claude') : p.name }),
          pill),
        ...subs));
  });
  el.profileList.replaceChildren(...rows);
}

// --- Detail view ------------------------------------------------------------
async function showDetail(name) {
  selectedName = name;
  let d;
  try {
    d = await invoke('get_profile_details', { name });
  } catch (err) {
    showToast('環境を読み込めませんでした。', true);
    return;
  }
  renderSidebar();
  const isDefault = !!d.is_default;
  const isInUse = inUse(name);

  const header = h('div', { class: 'detail-header' },
    h('div', { class: 'detail-bezel' },
      h('div', { class: 'detail-avatar' }, isDefault ? icon('i-monitor') : avatarContent(d.icon, name))),
    h('div', { class: 'detail-titles' },
      h('div', { class: 'detail-name', text: isDefault ? T('既存の Claude', 'Existing Claude') : name }),
      h('div', { class: 'detail-tagline', text: isDefault ? 'あなた自身の環境' : (isInUse ? '利用中の環境' : '作成した環境') })),
    isInUse ? h('span', { class: 'pill pill-active', style: 'margin-left:auto', 'aria-label': '利用中', text: '利用中' }) : null);

  const nodes = [header];

  if (!isDefault) {
    // Identity first: the user's own note and the environment's record (created,
    // duplicated from, last launched) answer "which environment is this?" before
    // the sharing state below answers "what does it inherit?".
    nodes.push(noteSection(d));
  }

  if (isDefault) {
    // Keep only the reassurance (the user's real worry: "will this break my
    // setup?"). The paths are collapsed; the "all 11 shared" line is dropped as
    // self-evident for the baseline.
    nodes.push(section('', [
      h('div', { class: 'note-card', dataset: { en: 'This is your own Claude Desktop App and Claude Code setup. CSW only displays it; it never changes or deletes your settings, history, or sign-in.' } },
        'あなたが普段使っている ', h('strong', { text: 'Claudeデスクトップアプリ' }), 'と ',
        h('strong', { text: 'Claude Code' }),
        ' の環境です。CSW はここを表示しているだけで、設定・履歴・ログインを変更したり削除したりしません。'),
    ]));
    nodes.push(pathsSection(d, true));
  } else {
    // State first (what this environment inherits), then collapsed detail, then
    // a one-line switch hint by the action. The "利用中"/multi-window concept lives
    // in onboarding, not repeated as an always-on block here.
    nodes.push(sharingDisclosure(d.sharing));
    nodes.push(pathsSection(d, false));
    nodes.push(doctorSection(name));
    nodes.push(terminalSection(name));
    // Cloning is a rare management action, so it sits here as a quiet button rather
    // than taking a prominent footer slot next to the launch actions.
    nodes.push(cloneSection(name));
    // Explain the footer's launch buttons, right above them (last content block), so
    // each short-label button reads unambiguously.
    nodes.push(launchHelp(!!d.supports_concurrent_windows));
  }

  el.detailContent.replaceChildren(...nodes);
  renderDetailFooter(name, isDefault, isInUse, !!d.supports_concurrent_windows);
  setView('detail');
}

// Clone is a quiet, infrequent management action. It is presented as a labeled row
// in a flat bordered card (minimalist-ui: clear hierarchy, no orphan button) with a
// one-line explanation of what it does, so it never reads as a mystery button. It
// stays out of the footer so the footer's slots are reserved for launching.
function cloneSection(name) {
  return section('この環境の複製', [
    h('div', { class: 'manage-row' },
      h('p', { class: 'firstrun-body manage-text', text: '設定をそのままコピーして、別の名前の環境を作ります。元の環境は変わりません。' }),
      h('button', { type: 'button', class: 'btn btn-ghost manage-action', onclick: () => showCloneRow(name) },
        icon('i-duplicate'), h('span', { text: '複製' }))),
  ]);
}

// The isolation check (csw doctor) is a quiet, on-demand, read-only action.
// Same presentation as cloneSection: a labeled row with a one-line explanation
// and a ghost button; the report renders below when run. The GUI never repairs
// anything (auto-fixes are the classic source of profile-switcher accidents);
// re-pointing broken links stays in the CLI as `csw doctor --fix`.
function doctorSection(name) {
  const results = h('div', { class: 'doctor-results' });
  const btn = h('button', {
    type: 'button', class: 'btn btn-ghost manage-action',
    onclick: () => runDoctor(name, results, btn),
  }, icon('i-check'), h('span', { text: '検査する' }));
  return section('分離の検査', [
    h('div', { class: 'manage-row' },
      h('p', { class: 'firstrun-body manage-text', text: '共有と分離が設定どおりに保たれているかを、この場で点検します。ファイルの中身は読まず、何も書き換えません。' }),
      btn),
    results,
  ]);
}

async function runDoctor(name, results, btn) {
  btn.disabled = true;
  try {
    const report = await invoke('inspect_profile', { name });
    results.replaceChildren(renderDoctorReport(report));
  } catch (err) {
    showToast('検査できませんでした。', true);
  } finally {
    btn.disabled = false;
  }
}

// The four link points that are structurally isolated in every mode
// (SPECIFICATION.md §3「常に分離する項目」). Mirrors LINK_ITEMS' fixed_mode.
const ALWAYS_ISOLATED_KEYS = ['desktop_config', 'cli_sessions', 'desktop_device_id', 'desktop_app_config'];

// Status text per item. Healthy states are short labels; issues add one full
// explanatory sentence below the row. Copy stays a distinct word from 分離
// (csw_product_canon: 共有 / 分離 / コピー only). item.mode carries the serde
// form of SharingMode ('share' / 'copy' / 'isolate', lowercase).
function doctorStatus(item) {
  const st = item.health.state;
  if (st === 'shared_ok') return { status: '正常に共有', issue: false };
  if (st === 'isolated_ok') {
    return { status: item.mode === 'copy' ? '正常にコピー' : '正常に分離', issue: false };
  }
  if (st === 'source_absent') return { status: '共有元がまだありません', issue: false };
  const unexpectedLink = ALWAYS_ISOLATED_KEYS.includes(item.key)
    ? '常に分離する項目がリンクになっています。'
    : (item.mode === 'copy' ? 'コピーの項目がリンクになっています。' : '分離の項目がリンクになっています。');
  const detail = {
    source_missing: '共有元が見つかりません。',
    wrong_target: item.health.fixable
      ? 'リンク先が想定と異なります。ターミナルで csw doctor --fix を実行すると張り直せます。'
      : 'リンク先が想定と異なり、共有元も見つかりません。',
    materialized: '共有のリンクが実体のファイルに置き換わっています。手元の内容を失わないよう、自動では直しません。',
    missing_link: '共有のリンクがありません。',
    unexpected_link: unexpectedLink,
  }[st] || '状態を判定できませんでした。';
  return { status: '要確認', detail, issue: true };
}

function renderDoctorReport(report) {
  const wrap = h('div', { class: 'doctor-report' });
  if (report.running) {
    wrap.appendChild(h('p', { class: 'doctor-note', text: 'この環境の Claudeデスクトップアプリが起動中です。書き換えの途中を検査した可能性があるため、問題が出た場合は Claude を終了してからもう一度検査してください。' }));
  }
  const n = report.items.length;
  const c = report.issue_count;
  const summary = c === 0
    ? T(`問題は見つかりませんでした。${n} 項目すべてが設定どおりです。`, `No issues found. All ${n} items match the settings.`)
    : T(`${c} 件の問題が見つかりました。`, c === 1 ? '1 issue found.' : `${c} issues found.`);
  wrap.appendChild(h('p', {
    class: report.issue_count === 0 ? 'doctor-summary' : 'doctor-summary doctor-summary-issue',
    text: summary,
  }));
  const list = h('div', { class: 'doctor-list' });
  for (const item of report.items) {
    const label = DOCTOR_LABELS[item.key] || item.key;
    const s = doctorStatus(item);
    list.appendChild(h('div', { class: 'doctor-item' },
      h('span', { class: 'doctor-item-label', text: label }),
      h('span', { class: s.issue ? 'doctor-item-state doctor-item-issue' : 'doctor-item-state', text: s.status })));
    if (s.detail) list.appendChild(h('p', { class: 'doctor-item-detail', text: s.detail }));
  }
  wrap.appendChild(list);
  return wrap;
}

// Describe the footer's launch buttons just above them. Uses the same bordered
// head+body card the onboarding uses to explain multiple items (minimalist-ui:
// flat, 1px border, clear hierarchy) rather than a bare muted paragraph. Copy is
// plain prose with the reason before the instruction, not a term:definition list.
// "重複して起動" only appears for fully-isolated environments (they run concurrently).
function launchHelp(supportsConcurrent) {
  const way = (head, body) => h('div', { class: 'launch-way' },
    h('p', { class: 'firstrun-head', text: head }),
    h('p', { class: 'firstrun-body', text: body }));
  const ways = [
    way('切り替えて起動', 'この環境に切り替えて Claude を開きます。ほかの環境の Claude が起動しているときは切り替えられないので、先に終了してください。'),
  ];
  if (supportsConcurrent) {
    ways.push(way('重複して起動', '起動中の Claude を終了せずに、この環境を新しいウィンドウで同時に開きます。'));
  }
  return section('起動のしかた', [h('div', { class: 'firstrun-card' }, ...ways)]);
}

function section(label, children) {
  return h('div', { class: 'section' },
    label ? h('div', { class: 'section-label', text: label }) : null, ...children);
}

function pathsSection(d, isDefault) {
  if (isDefault) {
    // The default is the user's real Claude data: paths only, never aggregated.
    const rows = [
      pathRow('Claudeデスクトップアプリ', d.desktop_path, 'default', 'desktop'),
      pathRow('Claude Code', d.cli_path, 'default', 'cli'),
      h('p', { class: 'path-caption', text: 'これは CSW が作った場所ではなく、あなた自身の Claude フォルダです。' }),
    ];
    return section('', [disclosure('あなたの Claude フォルダの場所', null, rows)]);
  }
  const name = d.name;
  const rows = [
    pathRow('Claudeデスクトップアプリ', d.desktop_path, name, 'desktop'),
    pathRow('Claude Code', d.cli_path, name, 'cli'),
  ];
  // The breakdown (sizes, link targets) is aggregated lazily on first open:
  // it stats files but never reads contents and never follows symlinks.
  const details = h('div', { class: 'datamap' });
  // Loaded on open; retried on the next open after a failure.
  let loading = false;
  let loaded = false;
  const loadMap = async () => {
    if (loading || loaded) return;
    loading = true;
    details.replaceChildren(h('p', { class: 'datamap-loading', text: '読み込んでいます…' }));
    try {
      const map = await invoke('get_profile_data_map', { name });
      details.replaceChildren(renderDataMap(map));
      loaded = true;
    } catch (err) {
      details.replaceChildren(h('p', { class: 'datamap-loading', text: '内訳を読み込めませんでした。' }));
    } finally {
      loading = false;
    }
  };
  return section('', [disclosure('この環境のデータの場所', null, [...rows, details], loadMap)]);
}

// Item labels for the data map: same vocabulary as the isolation check.
function fmtBytes(n) {
  if (n < 1024) return `${n} B`;
  const units = ['KB', 'MB', 'GB', 'TB'];
  let v = n / 1024;
  let u = 0;
  while (v >= 1024 && u < units.length - 1) { v /= 1024; u += 1; }
  return `${v >= 100 ? Math.round(v) : v.toFixed(1)} ${units[u]}`;
}

function fmtEpoch(epoch) {
  const d = new Date(epoch * 1000);
  return d.toLocaleDateString(LANG === 'ja' ? 'ja-JP' : 'en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}

// RFC 3339 string (created_at / last_launched_at) to a localized date.
function fmtDate(rfc3339) {
  const t = Date.parse(rfc3339);
  if (Number.isNaN(t)) return '';
  return new Date(t).toLocaleDateString(LANG === 'ja' ? 'ja-JP' : 'en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}

// Relative time for the launch stamp. Dynamic strings are built per language
// with T() because runtime-concatenated text never matches the EN dictionary.
function relTime(rfc3339) {
  const then = Date.parse(rfc3339);
  if (Number.isNaN(then)) return '';
  const mins = Math.floor(Math.max(0, Date.now() - then) / 60000);
  const hours = Math.floor(mins / 60);
  const days = Math.floor(hours / 24);
  if (mins < 1) return T('たった今', 'just now');
  if (hours < 1) return T(`${mins} 分前`, mins === 1 ? '1 minute ago' : `${mins} minutes ago`);
  if (days < 1) return T(`${hours} 時間前`, hours === 1 ? '1 hour ago' : `${hours} hours ago`);
  if (days < 30) return T(`${days} 日前`, days === 1 ? '1 day ago' : `${days} days ago`);
  return fmtDate(rfc3339);
}

// The note and record card: the free-form note (editable in place) with the
// environment's record underneath. Never shown for the default environment.
function noteSection(d) {
  const name = d.name;
  const box = h('div', { class: 'note-box' });

  const facts = [];
  if (d.created_at) facts.push(T(`作成: ${fmtDate(d.created_at)}`, `Created ${fmtDate(d.created_at)}`));
  if (d.cloned_from) facts.push(T(`複製元: ${d.cloned_from}`, `Duplicated from ${d.cloned_from}`));
  facts.push(d.last_launched_at
    ? T(`最終起動: ${relTime(d.last_launched_at)}`, `Last launched ${relTime(d.last_launched_at)}`)
    : T('最終起動: まだありません', 'Never launched'));
  const factsRow = h('div', { class: 'detail-facts' }, ...facts.map((f) => h('span', { text: f })));

  const renderRead = () => {
    box.replaceChildren(
      h('div', { class: 'manage-row' },
        h('p', {
          class: 'manage-text detail-note' + (d.note ? '' : ' note-empty'),
          text: d.note || T('メモはまだありません。用途や、サインインに使うアカウントを書いておけます。',
            'No note yet. You can write down what this environment is for and which account signs in.'),
        }),
        h('button', { type: 'button', class: 'btn btn-ghost manage-action', onclick: renderEdit },
          d.note ? '編集' : 'メモを書く')),
      factsRow);
  };

  const renderEdit = () => {
    const save = async (val) => {
      try {
        await invoke('set_profile_note', { name, note: val });
        d.note = val;
        await refreshProfiles();
        renderRead();
        showToast(T('メモを保存しました', 'Saved the note'));
      } catch (err) {
        showToast(T('メモを保存できませんでした。', 'Could not save the note.'), true);
      }
    };
    const input = h('input', {
      type: 'text', class: 'input', value: d.note || '', maxlength: '200',
      placeholder: '例: 仕事用。会社の Google アカウントでサインイン',
      autocomplete: 'off', spellcheck: 'false', style: 'flex:1',
      // IME guard: Enter while composing Japanese must confirm the conversion,
      // not submit (e.isComposing, plus keyCode 229 for older WebKit).
      onkeydown: (e) => {
        if (e.key === 'Enter' && !e.isComposing && e.keyCode !== 229) save(input.value.trim());
      },
    });
    box.replaceChildren(
      h('div', { class: 'manage-row note-edit-row' },
        input,
        h('button', { type: 'button', class: 'btn btn-ghost', onclick: renderRead }, 'やめる'),
        h('button', { type: 'button', class: 'btn btn-primary', onclick: () => save(input.value.trim()) }, '保存')),
      factsRow);
    input.focus();
  };

  renderRead();
  return section('この環境のメモ', [box]);
}

function renderDataMap(map) {
  const wrap = h('div', { class: 'datamap-report' });
  wrap.appendChild(h('p', {
    class: 'datamap-total',
    text: T(
      `この環境が専有している容量は ${fmtBytes(map.total_size_bytes)} です。内訳は Claudeデスクトップアプリが ${fmtBytes(map.desktop_size_bytes)}、Claude Code が ${fmtBytes(map.cli_size_bytes)} で、共有リンクの先の実体は含みません。`,
      `This environment occupies ${fmtBytes(map.total_size_bytes)} by itself: ${fmtBytes(map.desktop_size_bytes)} for the Claude Desktop App and ${fmtBytes(map.cli_size_bytes)} for Claude Code. Shared originals are not included.`
    ),
  }));
  const list = h('div', { class: 'datamap-list' });
  for (const item of map.items) {
    const label = DOCTOR_LABELS[item.key] || item.key;
    let state;
    let detail = null;
    if (item.key === 'desktop_app_config') {
      state = '常に分離';
    } else if (item.mode === 'share' && item.link_target) {
      state = '既存の Claude と共有';
      detail = item.link_target;
    } else if (item.link_target) {
      // A non-share item drifted into a link: state the fact with the same
      // wording the isolation check uses, and show where it points.
      state = ALWAYS_ISOLATED_KEYS.includes(item.key)
        ? '常に分離する項目がリンクになっています。'
        : (item.mode === 'copy' ? 'コピーの項目がリンクになっています。' : '分離の項目がリンクになっています。');
      detail = item.link_target;
    } else if (!item.exists) {
      state = 'まだありません';
    } else {
      const size = fmtBytes(item.size_bytes || 0);
      state = item.modified_epoch
        ? `${size}${T(' ・ 最終更新 ', ' · updated ')}${fmtEpoch(item.modified_epoch)}`
        : size;
    }
    list.appendChild(h('div', { class: 'datamap-item' },
      h('span', { class: 'datamap-item-label', text: label }),
      h('span', { class: 'datamap-item-state', text: state })));
    if (detail) {
      list.appendChild(h('p', { class: 'datamap-item-detail' },
        h('code', { class: 'path-code', text: detail })));
    }
  }
  wrap.appendChild(list);
  return wrap;
}

function pathRow(label, value, revealName, revealWhich) {
  const reveal = revealName ? h('button', {
    type: 'button', class: 'icon-btn', title: 'Finder で表示',
    onclick: () => invoke('reveal_profile_dir', { name: revealName, which: revealWhich })
      .catch(() => showToast('Finder で表示できませんでした。', true)),
  }, icon('i-folder')) : null;
  return h('div', { class: 'path-row' },
    h('div', { class: 'path-meta' },
      h('span', { class: 'path-label', text: label }),
      h('code', { class: 'path-code', text: value })),
    reveal,
    copyButton(value, 'パスをコピー'));
}

function copyButton(value, title) {
  return h('button', {
    type: 'button', class: 'icon-btn', title,
    onclick: () => copyText(value),
  }, icon('i-copy'));
}

function terminalSection(name) {
  const cmd = `eval $(csw env ${name})`;
  // Collapsed by default: most users drive everything from the GUI; the CLI
  // command is only needed when opening a separate terminal yourself.
  const inner = [
    h('p', { class: 'share-basis', text: T(
      'CSW から開いた Claudeデスクトップアプリの中のターミナルは、最初からこの環境です。コマンドは要りません。自分で別に開いた iTerm2 などのターミナルでこの環境に揃えるには、次を実行します。この指定はそのタブだけに効き、普段の環境には影響しません。',
      'A terminal inside the Claude Desktop App you opened from CSW is already in this environment, so no command is needed. To match a terminal you opened yourself, such as iTerm2, run the command below. It applies to that tab only and never affects your usual environment.') }),
    h('div', { class: 'path-row' },
      h('div', { class: 'path-meta' }, h('code', { class: 'path-code', text: cmd })),
      copyButton(cmd, 'コマンドをコピー')),
  ];
  return section('', [disclosure('ターミナルで Claude Code を使う', null, inner)]);
}

// Generic collapsible disclosure: a summary row stays visible, the panel opens
// on click. Keeps secondary detail (paths, the terminal command, the full
// sharing breakdown) off the always-on surface so the detail view shows state +
// actions first and discloses the rest only when asked.
function disclosure(label, sub, innerNodes, onOpen) {
  const wrap = h('div', { class: 'disclosure' });
  const innerWrap = h('div', { class: 'disclosure-inner' }, ...innerNodes);
  innerWrap.inert = true; // keep collapsed content out of the tab order
  const toggle = h('button', {
    type: 'button', class: 'sharing-summary', 'aria-expanded': 'false',
    onclick: () => {
      const open = wrap.classList.toggle('open');
      toggle.setAttribute('aria-expanded', String(open));
      innerWrap.inert = !open;
      if (open && onOpen) onOpen(); // callbacks self-guard (e.g. load once, retry on failure)
      requestAnimationFrame(refreshFades);
    },
  },
    h('span', { class: 'summary-text' }, label,
      sub ? h('span', { class: 'summary-sub', text: sub }) : null),
    icon('i-chevron', 'icon summary-chevron'));
  appendKids(wrap, [toggle, h('div', { class: 'disclosure-panel' }, innerWrap)]);
  return wrap;
}

function sharingDisclosure(sharing) {
  const shareCount = ALL_KEYS.filter((k) => sharing[k] === 'share').length;
  const copyCount = ALL_KEYS.filter((k) => sharing[k] === 'copy').length;
  const isoCount = ALL_KEYS.length - shareCount - copyCount;
  const summary = copyCount
    ? T(`共有 ${shareCount}・コピー ${copyCount}・分離 ${isoCount} 件`, `${shareCount} shared · ${copyCount} copied · ${isoCount} isolated`)
    : T(`共有 ${shareCount}・分離 ${isoCount} 件`, `${shareCount} shared · ${isoCount} isolated`);

  const inner = [
    // Name the reference point so "shared/copied/isolated" is never ambiguous.
    h('p', { class: 'share-basis', text: '「共有」は既存の Claude と中身を共通にすること、「コピー」は作成時に一度だけ写して以後は別々、「分離」はこの環境だけで持つことです。' }),
  ];
  for (const g of SHARE_GROUPS) {
    inner.push(h('div', { class: 'share-group' },
      h('div', { class: 'share-group-label', text: g.label }),
      ...g.items.map((it) => sharingReadRow(it, sharing[it.key]))));
  }
  inner.push(h('div', { class: 'share-group' }, sharingReadRow(DEVICE_ID, sharing[DEVICE_ID.key])));

  return section('この環境が引き継いでいるもの',
    [disclosure(summary, T(' ／ アカウントは常に分離', ' · Account always separate'), inner)]);
}

function sharingReadRow(item, mode) {
  return h('div', { class: 'share-line' },
    h('div', { class: 'share-line-meta' },
      h('div', { class: 'share-line-name', text: item.name }),
      h('div', { class: 'share-line-desc', text: item.desc })),
    badge(mode));
}

function badge(mode) {
  if (mode === 'share') return h('span', { class: 'badge badge-share' }, icon('i-link'), '共有');
  if (mode === 'copy') return h('span', { class: 'badge badge-copy' }, icon('i-copy'), 'コピー');
  return h('span', { class: 'badge badge-isolate' }, icon('i-lock'), '分離');
}

function renderDetailFooter(name, isDefault, isInUse, supportsConcurrent) {
  el.detailFooter.className = 'view-footer split';
  // Disable the action only while this environment's Claude is actually running
  // (nothing to launch). When it is not running, the action stays available so the
  // active environment can be relaunched after Claude was quit.
  // Text-only: a switch/arrow glyph on a launch action only added noise. The
  // meaning of each short label is spelled out by launchHelp() in the body above.
  const switchBtn = h('button', {
    type: 'button', class: 'btn btn-primary', disabled: isInUse,
    onclick: () => { if (!isInUse) doSwitch(name, supportsConcurrent); },
  }, h('span', { text: isInUse ? '利用中の環境' : (isDefault ? '既存の Claude に切り替える' : '切り替えて起動') }));

  if (isDefault) {
    el.detailFooter.replaceChildren(
      h('span', { class: 'confirm-text', style: 'color:var(--ink-muted)', text: '既存の Claude は変更・削除できません' }),
      switchBtn);
    return;
  }
  // Footer holds the launch actions. For fully-isolated environments the dedicated
  // "重複して起動" sits right next to the primary launch (a frequent action for these);
  // clone/delete are management actions kept out of this row.
  const launchGroup = h('div', { class: 'footer-group' }, switchBtn);
  if (supportsConcurrent) {
    launchGroup.appendChild(h('button', {
      type: 'button', class: 'btn btn-ghost', onclick: () => doLaunchNewWindow(name),
    }, h('span', { text: '重複して起動' })));
  }
  el.detailFooter.replaceChildren(
    launchGroup,
    h('button', { type: 'button', class: 'btn btn-danger', onclick: () => showDeleteRow(name) },
      icon('i-trash'), h('span', { text: '削除' })));
}

// --- Inline confirm / clone rows (no window.confirm) ------------------------
// Deleting moves the whole folder to the Trash (restorable, links move as
// links). The copy states that honestly, including that the sign-in state
// stays restorable until the Trash is emptied; users who want the data gone
// now get an explicit, separately confirmed "完全に削除".
function showDeleteRow(name) {
  el.detailFooter.className = 'view-footer confirm-stack';
  const cancel = h('button', { type: 'button', class: 'btn btn-ghost', onclick: () => showDetail(name) }, 'やめる');
  const text = h('span', {
    class: 'confirm-text',
    text: T(
      'この環境をゴミ箱へ移動します。ゴミ箱を空にするまでは、サインイン状態を含むすべてのデータを戻せます。既存の Claude には影響しません。',
      'This environment will be moved to the Trash. Until you empty the Trash, everything including the sign-in state can be restored. Your existing Claude is not affected.'
    ),
  });
  el.detailFooter.replaceChildren(
    text,
    h('div', { class: 'footer-group' },
      cancel,
      h('button', { type: 'button', class: 'btn btn-danger', onclick: () => showPurgeRow(name) }, '完全に削除'),
      h('button', { type: 'button', class: 'btn btn-danger-solid', onclick: () => doDelete(name) }, 'ゴミ箱へ移動')));
  cancel.focus(); // default focus on the safe action
  // The occupied size arrives whenever the read-only aggregation resolves; the
  // confirmation never waits for it. Built with T(), so the observer-based
  // dictionary is not involved and the sentence stays language-correct.
  invoke('get_profile_data_map', { name }).then((map) => {
    text.textContent += T(
      `この環境の専有容量は ${fmtBytes(map.total_size_bytes)} です。`,
      ` This environment occupies ${fmtBytes(map.total_size_bytes)} by itself.`
    );
  }).catch(() => { /* size stays out of the copy */ });
}

// Second, separate confirmation for the non-restorable path.
function showPurgeRow(name) {
  el.detailFooter.className = 'view-footer split';
  const cancel = h('button', { type: 'button', class: 'btn btn-ghost', onclick: () => showDetail(name) }, 'やめる');
  el.detailFooter.replaceChildren(
    h('span', { class: 'confirm-text', text: '完全に削除すると、ゴミ箱を経由せずにすぐ消え、戻せません。' }),
    h('div', { class: 'footer-group' },
      cancel,
      h('button', { type: 'button', class: 'btn btn-danger-solid', onclick: () => doPurge(name) }, '完全に削除する')));
  cancel.focus();
}

function showCloneRow(name) {
  el.detailFooter.className = 'view-footer';
  const input = h('input', {
    type: 'text', class: 'input', placeholder: '複製先の名前。例: 仕事用-控え',
    autocomplete: 'off', spellcheck: 'false', style: 'flex:1',
    onkeydown: (e) => { if (e.key === 'Enter') doClone(name, input.value.trim()); },
  });
  el.detailFooter.replaceChildren(
    input,
    h('button', { type: 'button', class: 'btn btn-ghost', onclick: () => showDetail(name) }, 'やめる'),
    h('button', { type: 'button', class: 'btn btn-primary', onclick: () => doClone(name, input.value.trim()) }, '複製を作る'));
  input.focus();
}

// --- Backend operations -----------------------------------------------------
// Switching relinks shared/isolated config and launches that environment's
// Claude. The core refuses to switch while Claude Desktop is running (to avoid
// config write-back races), so only one environment's Claude runs at a time.
// Guide the user to quit the running one first instead of showing a dead end.
function showSwitchBlocked(name, supportsConcurrent) {
  el.detailFooter.className = 'view-footer split';
  // Fully-isolated environments never need the running Claude quit: point straight
  // at opening a new window instead of a dead "quit first" instruction.
  if (supportsConcurrent) {
    el.detailFooter.replaceChildren(
      h('span', { class: 'confirm-text', text: 'この環境は、起動中の Claude を終了せずに開けます。' }),
      h('div', { class: 'footer-group' },
        h('button', { type: 'button', class: 'btn btn-ghost', onclick: () => showDetail(name) }, '閉じる'),
        h('button', { type: 'button', class: 'btn btn-primary', onclick: () => doLaunchNewWindow(name) },
          h('span', { text: '重複して起動' }))));
    return;
  }
  el.detailFooter.replaceChildren(
    h('span', { class: 'confirm-text', text: '起動中の Claude を終了してから、もう一度押してください。設定の衝突を防ぐため、共有を含む環境の Claude は同時に開けません。' }),
    h('div', { class: 'footer-group' },
      h('button', { type: 'button', class: 'btn btn-ghost', onclick: () => showDetail(name) }, '閉じる'),
      h('button', { type: 'button', class: 'btn btn-primary', onclick: () => doSwitch(name, false) }, 'もう一度試す')));
}

async function doSwitch(name, supportsConcurrent) {
  try {
    if (await invoke('get_desktop_running_status')) { showSwitchBlocked(name, supportsConcurrent); return; }
    await invoke('switch_profile', { name, noLaunch: false });
    await refreshProfiles();
    withTransition(() => showDetail(name));
    showToast(name === 'default' ? T('既存の Claude に切り替えました', 'Switched to Existing Claude') : T(`${name} の Claude を起動しました`, `Launched Claude for ${name}`));
  } catch (err) {
    if (String(err || '').includes('Claude Desktop is running')) { showSwitchBlocked(name, supportsConcurrent); return; }
    showToast('切り替えできませんでした。もう一度お試しください。', true);
  }
}

// Open a fully-isolated environment in an additional window, alongside whatever is
// already running. Does not switch the active environment or require quitting.
async function doLaunchNewWindow(name) {
  try {
    await invoke('launch_additional_window', { name });
    await refreshProfiles();
    withTransition(() => showDetail(name));
    showToast(T(`${name} を新しいウィンドウで起動しました`, `Launched ${name} in a new window`));
  } catch (err) {
    showToast('起動できませんでした。もう一度お試しください。', true);
  }
}

async function doDelete(name) {
  try {
    await invoke('delete_profile', { name });
    selectedName = null;
    await refreshProfiles();
    const remaining = profiles.filter((p) => p.name !== 'default');
    withTransition(() => (remaining.length ? showDetail(remaining[0].name) : showEmpty()));
    showToast(T(`「${name}」をゴミ箱へ移動しました`, `Moved "${name}" to the Trash`));
  } catch (err) {
    // The backend never falls back to permanent deletion. When the Trash move
    // itself failed (CswError::TrashMoveFailed), say so and point at the
    // explicit permanent path; otherwise the usual cause is an in-use profile.
    if (String(err).includes('could not move to the Trash')) {
      showToast('ゴミ箱へ移動できませんでした。すぐに消す場合は「完全に削除」を選んでください。', true);
    } else {
      showToast('削除できませんでした。利用中の環境は切り替えてから削除してください。', true);
    }
  }
}

async function doPurge(name) {
  try {
    await invoke('purge_profile', { name });
    selectedName = null;
    await refreshProfiles();
    const remaining = profiles.filter((p) => p.name !== 'default');
    withTransition(() => (remaining.length ? showDetail(remaining[0].name) : showEmpty()));
    showToast(T(`「${name}」を完全に削除しました`, `Permanently deleted "${name}"`));
  } catch (err) {
    showToast('削除できませんでした。利用中の環境は切り替えてから削除してください。', true);
  }
}

async function doClone(source, target) {
  const err = validateName(target);
  if (err) { showToast(err, true); return; }
  try {
    await invoke('clone_profile', { source, target });
    selectedName = target;
    await refreshProfiles();
    withTransition(() => showDetail(target));
    showToast(T(`「${target}」を複製しました。元の環境はそのままです。`, `Duplicated "${target}". The original is unchanged.`));
  } catch (e) {
    showToast('複製できませんでした。同じ名前がすでにあるか確認してください。', true);
  }
}

// --- Empty / create flow ----------------------------------------------------
let createIcon = ''; // selected avatar: an AVATAR_ICONS slug, an emoji/char, or '' (none)

// Render the icon picker grid (designed glyphs) for the create view.
function renderIconPicker() {
  el.iconPicker.replaceChildren(...AVATAR_ICONS.map((it) =>
    h('button', {
      type: 'button', class: 'icon-tile', role: 'radio', 'data-icon': it.slug,
      'aria-label': it.label, 'aria-checked': 'false', title: it.label,
      onclick: () => selectIcon(it.slug),
    }, icon('i-' + it.slug))));
  syncIconPicker();
}
function selectIcon(value) {
  createIcon = createIcon === value ? '' : value; // click again to clear
  syncIconPicker();
}
function syncIconPicker() {
  document.querySelectorAll('.icon-tile').forEach((b) => {
    const on = b.dataset.icon === createIcon && createIcon !== '';
    b.classList.toggle('selected', on);
    b.setAttribute('aria-checked', on ? 'true' : 'false');
  });
}

function showEmpty() {
  selectedName = null;
  renderSidebar();
  setView('empty');
}

function showCreate() {
  localStorage.setItem('csw_onboarded', '1');
  el.inputName.value = '';
  el.inputNote.value = '';
  createIcon = '';
  renderIconPicker();
  el.nameError.hidden = true;
  setMode('isolate');
  closeAdvanced();
  renderCreateFooter();
  setView('create');
  applyRootsStatus();
  setTimeout(() => el.inputName.focus(), 0);
}

// --- Create-view footer: deliberate two-step (review → confirm) -------------
// A stray Enter (e.g. confirming a Japanese IME kanji conversion) must never
// create an environment, so creating always goes through an explicit confirm step.
function cancelCreate() {
  withTransition(() => (selectedName ? showDetail(selectedName) : showEmpty()));
}
function renderCreateFooter() {
  el.createFooter.className = 'view-footer';
  el.createFooter.replaceChildren(
    h('button', { type: 'button', class: 'btn btn-ghost', onclick: cancelCreate }, 'やめる'),
    h('button', { type: 'button', class: 'btn btn-primary', onclick: requestCreate }, 'この環境を作る'));
}
// Validate the name, then ask for an explicit confirmation before creating.
function requestCreate() {
  const name = el.inputName.value.trim();
  const err = validateName(name);
  if (err) { el.nameError.textContent = err; el.nameError.hidden = false; el.inputName.focus(); return; }
  el.nameError.hidden = true;
  el.createFooter.className = 'view-footer split';
  el.createFooter.replaceChildren(
    h('span', { class: 'confirm-text', text: T(`「${name}」を作成します。`, `Create "${name}".`) }),
    h('div', { class: 'footer-group' },
      h('button', { type: 'button', class: 'btn btn-ghost', onclick: renderCreateFooter }, '戻る'),
      h('button', { type: 'button', class: 'btn btn-primary', onclick: submitCreate }, '作成する')));
}

// Gate the "share" mode by what the existing Claude actually has at the standard
// locations. Sharing symlinks from the default dirs, so a root that is missing has
// nothing to share: both missing → only "すべて分ける"; one missing → that side
// won't carry over, so warn but keep sharing available for the present side.
async function applyRootsStatus() {
  let st = { desktop_present: true, cli_present: true };
  try { st = await invoke('get_default_roots_status'); } catch (e) { /* keep optimistic default */ }
  // Both share modes carry over from the existing Claude, so both need its data.
  const shareModes = ['share_settings', 'share_workspace'];
  const bothAbsent = !st.desktop_present && !st.cli_present;
  for (const mode of shareModes) {
    const input = document.querySelector(`input[name="mode"][value="${mode}"]`);
    if (input) input.disabled = bothAbsent;
    const card = input ? input.closest('.mode-card') : null;
    if (card) card.classList.toggle('is-disabled', bothAbsent);
  }
  if (bothAbsent) setMode('isolate'); // nothing to carry over → a fully separate environment

  let msg = '';
  if (bothAbsent) {
    msg = '既存の Claude が標準の場所に見つかりません。引き継げるものが無いため、「すべて分ける」だけで作成できます。';
  } else if (!st.cli_present) {
    msg = 'Claude Code（CLI）の設定が標準の場所に見つかりません。引き継ぐモードを選んでも、ルールやスキル、会話などの CLI 側は引き継がれません。';
  }
  el.createNotice.textContent = msg;
  el.createNotice.hidden = !msg;
}

function setMode(mode) {
  currentMode = mode;
  const radio = document.querySelector(`input[name="mode"][value="${mode}"]`);
  if (radio) radio.checked = true;
  overrides = { ...PRESETS[mode] };
  advancedCustomized = false;
  renderAdvancedRows();
  updateAdvancedState();
}

function renderAdvancedRows() {
  // Name the reference point + define the words once, so every row reads clearly.
  const nodes = [
    h('p', { class: 'share-basis', text: '各項目を、既存の Claude と共有するか、この環境だけにするかを選びます。' }),
    h('p', { class: 'share-legend', text: '共有 = いまのを使う ／ 分離 = この環境だけ ／ コピー = 最初だけ写す' }),
    h('p', { class: 'path-caption', text: 'アカウントのサインイン情報（config.json）と、コネクタ・アプリ設定（claude_desktop_config.json）は、アカウント別の情報を含むため、どのモードでも必ずこの環境だけに分離します。' }),
  ];
  for (const g of SHARE_GROUPS) {
    nodes.push(h('div', { class: 'share-group' },
      h('div', { class: 'share-group-label', text: g.label }),
      ...g.items.map((it) => segRow(it, overrides[it.key], false))));
  }
  nodes.push(h('div', { class: 'share-group' },
    segRow(DEVICE_ID, 'isolate', true),
    h('p', { class: 'path-caption', text: '2つのアカウントが同じ端末として結び付かないよう、端末 ID は常に分離します。' })));
  el.advancedGroups.replaceChildren(...nodes);
}

function segRow(item, value, fixed) {
  const opt = (val, iconId, label) => {
    const input = h('input', { type: 'radio', name: 'seg-' + item.key, value: val });
    if (value === val) input.checked = true;
    if (fixed) input.disabled = true;
    return h('label', { class: 'seg-opt' }, input, icon(iconId), label);
  };
  return h('div', { class: 'share-line' + (fixed ? ' fixed' : '') },
    h('div', { class: 'share-line-meta' },
      h('div', { class: 'share-line-name', text: item.name }),
      h('div', { class: 'share-line-desc', text: item.desc })),
    // Composed at construction with T(): a runtime-concatenated label can never
    // match the exact-text EN dictionary, so both languages are built here.
    h('div', { class: 'seg', role: 'radiogroup', 'aria-label': T(
      item.name + '：既存の Claude と共有・分離・コピーのどれにするか',
      (setEN(item.name) || item.name) + ': whether to share with, isolate from, or copy from your existing Claude'
    ) },
      opt('share', 'i-link', '共有'), opt('isolate', 'i-lock', '分離'), opt('copy', 'i-copy', 'コピー')));
}

function updateAdvancedState() {
  el.advancedState.textContent = advancedCustomized ? 'カスタム設定' : 'モードの既定どおり';
  el.advancedState.classList.toggle('custom', advancedCustomized);
  el.advancedReset.hidden = !advancedCustomized;
}

function openAdvanced() {
  const adv = el.advancedToggle.closest('.advanced');
  adv.classList.add('open');
  el.advancedToggle.setAttribute('aria-expanded', 'true');
  adv.querySelector('.advanced-inner').inert = false;
  requestAnimationFrame(refreshFades);
}
function closeAdvanced() {
  const adv = el.advancedToggle.closest('.advanced');
  adv.classList.remove('open');
  el.advancedToggle.setAttribute('aria-expanded', 'false');
  adv.querySelector('.advanced-inner').inert = true; // keep collapsed rows out of the tab order
}

function validateName(name) {
  if (!name) return '名前を入力してください。';
  if (name.toLowerCase() === 'default') return '"default" は使えません。いまの環境を指す予約名です。';
  if ([...name].length > 64) return '名前は64文字までにしてください。';
  // Allow Unicode letters/numbers (including Japanese) plus hyphen/underscore.
  // Spaces, slashes, dots and symbols are rejected (the name becomes a folder
  // name and is passed to the shell in `csw env <name>`). Mirrors the core
  // validate_profile_name guard.
  if (!/^[\p{L}\p{N}_-]+$/u.test(name)) return '使えるのは文字・数字・ハイフン・アンダースコアだけです。空白や記号は使えません。';
  return null;
}

async function submitCreate() {
  const name = el.inputName.value.trim();
  const iconVal = createIcon;
  const err = validateName(name);
  if (err) { el.nameError.textContent = err; el.nameError.hidden = false; el.inputName.focus(); return; }
  el.nameError.hidden = true;

  const args = { name, mode: currentMode, icon: iconVal || null, note: el.inputNote.value.trim() || null };
  if (advancedCustomized) args.sharingOverrides = { ...overrides };

  const confirmBtn = el.createFooter.querySelector('.btn-primary');
  if (confirmBtn) confirmBtn.disabled = true;
  try {
    await invoke('create_profile', args);
    localStorage.setItem('csw_onboarded', '1');
    selectedName = name;
    await refreshProfiles();
    withTransition(() => showDetail(name));
    showToast(T(`「${name}」を作成しました`, `Created "${name}"`));
  } catch (e) {
    showToast('作成できませんでした。同じ名前がすでにあるか確認してください。', true);
    renderCreateFooter(); // restore the form footer so the user can fix and retry
  }
}

// --- Scroll affordance ------------------------------------------------------
function fadeFor(scrollEl, fadeEl) {
  if (!scrollEl || !fadeEl) return;
  fadeEl.hidden = !(scrollEl.scrollHeight - scrollEl.scrollTop - scrollEl.clientHeight > 4);
}
function refreshFades() {
  fadeFor(el.emptyScroll, el.emptyFade);
  fadeFor(el.detailScroll, el.detailFade);
  fadeFor(el.createScroll, el.createFade);
}

// --- Clipboard --------------------------------------------------------------
async function copyText(text) {
  try {
    await navigator.clipboard.writeText(text);
    showToast('コピーしました');
  } catch (e) {
    showToast('コピーできませんでした。', true);
  }
}

// --- Data refresh -----------------------------------------------------------
async function refreshProfiles() {
  try {
    profiles = await invoke('list_profiles');
    activeName = await invoke('get_active_profile');
  } catch (e) {
    profiles = [{ name: 'default', icon: '', is_default: true }];
    activeName = 'default';
  }
  await refreshRunning();
  renderSidebar();
}

// Re-read which environments are running (per environment) and whether any Claude
// is running at all. Kept separate so it can refresh on window focus without
// re-listing profiles, since the user typically quits/starts Claude in another app
// and returns to CSW expecting the markers and actions to reflect reality.
async function refreshRunning() {
  try { runningProfiles = await invoke('get_running_profiles'); }
  catch (e) { runningProfiles = []; }
  try { desktopRunning = await invoke('get_desktop_running_status'); }
  catch (e) { desktopRunning = false; }
}

// When the window regains focus or becomes visible again, the user may have just
// quit (or started) a Claude Desktop elsewhere, or moved an environment folder
// back from the Trash. Re-list the environments and the running state so the
// sidebar and the launch actions reflect reality without a restart
// (list_profiles is a single directory scan, so this stays cheap).
async function revalidateRunning() {
  const before = JSON.stringify([desktopRunning, runningProfiles, profiles.map((p) => p.name)]);
  await refreshProfiles();
  if (JSON.stringify([desktopRunning, runningProfiles, profiles.map((p) => p.name)]) === before) return;
  if (!el.viewDetail.hidden && selectedName) {
    if (profiles.some((p) => p.name === selectedName)) {
      showDetail(selectedName);
    } else {
      // The shown environment disappeared outside CSW: fall back gracefully.
      selectedName = null;
      const remaining = profiles.filter((p) => p.name !== 'default');
      withTransition(() => (remaining.length ? showDetail(remaining[0].name) : showEmpty()));
    }
  }
}

// --- Init -------------------------------------------------------------------
function wireEvents() {
  el.btnCreate.addEventListener('click', () => withTransition(showCreate));
  el.btnCreateEmpty.addEventListener('click', () => withTransition(showCreate));

  document.querySelectorAll('input[name="mode"]').forEach((r) =>
    r.addEventListener('change', () => {
      if (advancedCustomized) showToast('モードを変えたので高度設定をリセットしました');
      setMode(r.value);
    }));

  el.advancedToggle.addEventListener('click', () => {
    const open = el.advancedToggle.closest('.advanced').classList.contains('open');
    if (open) closeAdvanced(); else openAdvanced();
  });
  el.advancedReset.addEventListener('click', () => setMode(currentMode));
  el.advancedGroups.addEventListener('change', (e) => {
    const input = e.target.closest('input[type="radio"]');
    if (!input || input.disabled) return;
    overrides[input.name.replace('seg-', '')] = input.value;
    advancedCustomized = ALL_KEYS.some((k) => overrides[k] !== PRESETS[currentMode][k]);
    updateAdvancedState();
  });

  window.addEventListener('focus', revalidateRunning);
  document.addEventListener('visibilitychange', () => { if (!document.hidden) revalidateRunning(); });

  el.emptyScroll.addEventListener('scroll', () => fadeFor(el.emptyScroll, el.emptyFade), { passive: true });
  el.detailScroll.addEventListener('scroll', () => fadeFor(el.detailScroll, el.detailFade), { passive: true });
  el.createScroll.addEventListener('scroll', () => fadeFor(el.createScroll, el.createFade), { passive: true });
  window.addEventListener('resize', refreshFades);
  el.inputName.addEventListener('keydown', (e) => {
    // Ignore Enter while an IME composition is active (e.g. confirming a Japanese
    // kanji conversion) so it never creates; otherwise go to the confirmation step.
    if (e.key === 'Enter' && !e.isComposing && e.keyCode !== 229) { e.preventDefault(); requestCreate(); }
  });
}

function checkFirstRun() {
  const show = !localStorage.getItem('csw_onboarded');
  el.emptyFirstRun.hidden = !show;
  el.firstRunExtra.hidden = !show;
}

// --- Accent theme (user-selectable, persisted) ------------------------------
const ACCENTS = ['blue', 'teal', 'indigo', 'claude'];
function applyAccent(name) {
  const a = ACCENTS.includes(name) ? name : 'blue';
  document.documentElement.dataset.accent = a;
  try { localStorage.setItem('csw_accent', a); } catch (e) {}
  document.querySelectorAll('.theme-dot').forEach((d) => {
    const on = d.dataset.accent === a;
    d.classList.toggle('active', on);
    if (on) d.setAttribute('aria-current', 'true'); else d.removeAttribute('aria-current');
  });
}
function initAccent() {
  let saved = 'blue';
  try { saved = localStorage.getItem('csw_accent') || 'blue'; } catch (e) {}
  applyAccent(saved);
  const picker = document.getElementById('themePicker');
  if (picker) picker.addEventListener('click', (e) => {
    const dot = e.target.closest('.theme-dot');
    if (dot) applyAccent(dot.dataset.accent);
  });
}

// --- Footer: help links, version, about dialog ------------------------------
// Same disclaimer the LP shows under "免責・利用について" (kept in sync by hand).
const DISCLAIMER = [
  ['無保証', '本ソフトウェアは MIT ライセンスのもとで、そのままの状態で提供されます。動作や品質について、いかなる保証もありません。'],
  ['自己責任でのご利用', 'ご利用は自己責任でお願いします。データの損失や不具合などについて、作者は責任を負いません。大切なデータは、お試しになる前にバックアップしておくことをおすすめします。'],
  ['プライバシー', 'CSW はインターネット通信も、利用状況の送信も行いません。パスワードを保管する macOS のキーチェーンにも触れず、すべて手元のフォルダと設定の操作だけで動きます。'],
  ['何を読み書きするか', '環境のデータの書き込み先は、CSW 専用のフォルダ ~/.context-switcher-claude/ の中だけです。読む場所・書く場所の詳しい一覧と、通信していないことをご自身の Mac で確かめる手順は、プライバシーと透明性の文書で公開しています。'],
  ['非公式プロジェクト', '本プロジェクトは非公式のコミュニティ製で、Anthropic 社とは関係ありません。「Claude」は Anthropic の商標です。'],
];

// Fixed GitHub URLs for the privacy document (ja/en). Listed, like the footer
// links, in docs/PRIVACY.md as the complete set of URLs open_url may receive.
const PRIVACY_URL_JA = 'https://github.com/matsumotory/claude-desktop-switcher/blob/main/docs/PRIVACY.md';
const PRIVACY_URL_EN = 'https://github.com/matsumotory/claude-desktop-switcher/blob/main/docs/PRIVACY_EN.md';

// Hand a fixed https GitHub URL to the OS (Rust open_url) so it opens in the
// default browser. CSW makes no network requests itself.
async function openExternal(url) {
  try {
    await invoke('open_url', { url });
  } catch (e) {
    showToast('リンクを開けませんでした。', true);
  }
}

function showAbout() {
  let overlay;
  const close = () => {
    overlay.remove();
    document.removeEventListener('keydown', onKey);
    el.btnAbout.focus();
  };
  const onKey = (e) => { if (e.key === 'Escape') close(); };
  const closeBtn = h('button', { type: 'button', class: 'btn btn-ghost', onclick: close }, '閉じる');
  const privacyBtn = h('button', {
    type: 'button', class: 'btn btn-ghost',
    onclick: () => openExternal(T(PRIVACY_URL_JA, PRIVACY_URL_EN)),
  }, '確かめ方を見る');
  const card = h('div', { class: 'about-card', role: 'dialog', 'aria-modal': 'true', 'aria-label': 'このアプリについて' },
    h('div', { class: 'about-title', text: 'Claude Desktop Switcher' }),
    h('div', { class: 'about-version', text: el.appVersion.textContent || '' }),
    h('p', { class: 'about-intro', text: '非公式のオープンソースのコミュニティプロジェクトです。' }),
    h('div', { class: 'about-list' },
      ...DISCLAIMER.map(([t, b]) => h('div', { class: 'about-item' },
        h('div', { class: 'about-item-title', text: t }),
        h('div', { class: 'about-item-body', text: b })))),
    h('div', { class: 'about-foot' }, privacyBtn, closeBtn));
  overlay = h('div', { class: 'about-overlay', onclick: (e) => { if (e.target === overlay) close(); } }, card);
  document.addEventListener('keydown', onKey);
  document.body.appendChild(overlay);
  closeBtn.focus();
}

function wireFooter() {
  document.querySelectorAll('.footer-link[data-url]').forEach((b) =>
    b.addEventListener('click', () => openExternal(b.dataset.url)));
  el.btnAbout.addEventListener('click', showAbout);
  // Version from tauri.conf.json (release-please keeps it current). Hide on failure.
  invoke('app_version')
    .then((v) => { el.appVersion.textContent = 'v' + v; })
    .catch(() => { el.appVersion.hidden = true; });
}

// --- Leftover installer disk image (SPECIFICATION.md §5.A) ------------------
// Prompt to eject a still-mounted CSW .dmg. "あとで" hides the banner for this
// session only; the next launch prompts again while the image stays mounted.
let dmgMounts = [];
let dmgDismissed = false;

async function checkDmgLeftover() {
  try { dmgMounts = (await invoke('get_dmg_mount_status')) || []; }
  catch (e) { dmgMounts = []; }
  el.dmgBanner.hidden = dmgDismissed || dmgMounts.length === 0;
}

async function doEjectDmg() {
  el.btnDmgEject.disabled = true;
  try {
    for (const m of dmgMounts) await invoke('eject_dmg', { mountPoint: m });
    el.dmgBanner.hidden = true;
    showToast('ディスクイメージを取り出しました');
  } catch (err) {
    showToast('取り出せませんでした。ディスクイメージを使用中のウィンドウを閉じてから、もう一度お試しください。', true);
  } finally {
    el.btnDmgEject.disabled = false;
  }
}

function wireDmgBanner() {
  el.btnDmgEject.addEventListener('click', doEjectDmg);
  el.btnDmgLater.addEventListener('click', () => {
    dmgDismissed = true;
    el.dmgBanner.hidden = true;
  });
}

async function init() {
  initAccent();
  wireEvents();
  wireFooter();
  wireDmgBanner();
  checkDmgLeftover();
  await refreshProfiles();
  checkFirstRun();
  showEmpty();
}

document.addEventListener('DOMContentLoaded', init);

// ============================================================================
// Dev-only mock (used only when window.__TAURI__ is absent, e.g. screenshots).
// The shipped app sets withGlobalTauri:true and never reaches this path.
// ============================================================================
function devInvoke(cmd, args) {
  // Locale-aware sample names so English screenshots read naturally. Real user
  // environment names are never translated (they are not run through the dictionary).
  const NM = LANG === 'ja' ? { work: '仕事用', research: '研究用', testing: '検証用' } : { work: 'Work', research: 'Research', testing: 'Testing' };
  // Sample notes are user data in the real app; here they are locale-aware so
  // screenshots read naturally in both languages.
  const NOTE = LANG === 'ja'
    ? { work: '会社の Google アカウントでサインイン', research: '研究室の Anthropic アカウントでサインイン' }
    : { work: 'Signs in with the company Google account', research: 'Signs in with the lab Anthropic account' };
  const ago = (hours) => new Date(Date.now() - hours * 3600 * 1000).toISOString();
  const prof = (name, iconVal, concurrent, sharing, extra) => ({
    name, icon: iconVal, is_default: false, supports_concurrent_windows: concurrent,
    desktop_path: `~/.context-switcher-claude/profiles/${name}/desktop-data`,
    cli_path: `~/.context-switcher-claude/profiles/${name}/cli-data`,
    sharing,
    note: '', created_at: null, cloned_from: null, last_launched_at: null,
    ...extra,
  });
  const sample = {
    default: { name: 'default', icon: '', is_default: true, desktop_path: '~/Library/Application Support/Claude', cli_path: '~/.claude', supports_concurrent_windows: false, sharing: Object.fromEntries(ALL_KEYS.map((k) => [k, 'share'])) },
    [NM.work]: prof(NM.work, 'briefcase', false, { ...PRESETS.share_settings },
      { note: NOTE.work, created_at: ago(24 * 40), last_launched_at: ago(3) }),
    [NM.research]: prof(NM.research, 'graduation-cap', false, { ...PRESETS.share_workspace },
      { note: NOTE.research, created_at: ago(24 * 20), last_launched_at: ago(24 * 3) }),
    [NM.testing]: prof(NM.testing, 'flask', true, { ...PRESETS.isolate },
      { created_at: ago(2), cloned_from: NM.work }),
  };
  switch (cmd) {
    case 'list_profiles':
      return Promise.resolve([sample.default, sample[NM.work], sample[NM.research], sample[NM.testing]]
        .map((p) => ({
          name: p.name, icon: p.icon, is_default: p.is_default,
          note: p.note || '', last_launched_at: p.last_launched_at || null,
        })));
    case 'get_active_profile':
      return Promise.resolve('default');
    case 'get_profile_details':
      return Promise.resolve(sample[args.name] || sample[NM.testing]);
    case 'get_desktop_running_status':
      // Depict the active environment as actually running so screenshots show the
      // "利用中" marker (the realistic state when Claude is open for that account).
      return Promise.resolve(true);
    case 'get_running_profiles':
      // The existing Claude is depicted as running (mirrors get_desktop_running_status).
      return Promise.resolve(['default']);
    case 'launch_additional_window':
      return Promise.resolve(null);
    case 'get_profile_data_map': {
      // Plausible healthy map shaped like the Rust serde output (lowercase
      // modes, LINK_ITEMS order, share_settings-style environment).
      const base = `~/.context-switcher-claude/profiles/${args.name}`;
      const t = 1782900000;
      const items = [
        { key: 'desktop_config', mode: 'isolate', link_target: null, size_bytes: 2048, modified_epoch: t, exists: true },
        { key: 'cli_settings', mode: 'copy', link_target: null, size_bytes: 6144, modified_epoch: t, exists: true },
        { key: 'cli_claude_md', mode: 'share', link_target: '~/.claude/CLAUDE.md', size_bytes: null, modified_epoch: null, exists: true },
        { key: 'cli_project_memory', mode: 'isolate', link_target: null, size_bytes: 48 * 1024 * 1024, modified_epoch: t, exists: true },
        { key: 'cli_plugins', mode: 'share', link_target: '~/.claude/plugins', size_bytes: null, modified_epoch: null, exists: true },
        { key: 'cli_skills', mode: 'share', link_target: '~/.claude/skills', size_bytes: null, modified_epoch: null, exists: true },
        { key: 'cli_sessions', mode: 'isolate', link_target: null, size_bytes: 12288, modified_epoch: t, exists: true },
        { key: 'cli_history', mode: 'isolate', link_target: null, size_bytes: null, modified_epoch: null, exists: false },
        { key: 'desktop_worktrees', mode: 'copy', link_target: null, size_bytes: 1024, modified_epoch: t, exists: true },
        { key: 'desktop_device_id', mode: 'isolate', link_target: null, size_bytes: 64, modified_epoch: t, exists: true },
        { key: 'desktop_app_config', mode: 'isolate', link_target: null, size_bytes: null, modified_epoch: null, exists: true },
      ];
      return Promise.resolve({
        profile: args.name,
        desktop_dir: `${base}/desktop-data`,
        cli_dir: `${base}/cli-data`,
        desktop_size_bytes: 96 * 1024 * 1024,
        cli_size_bytes: 49 * 1024 * 1024,
        total_size_bytes: 145 * 1024 * 1024,
        items,
      });
    }
    case 'reveal_profile_dir':
      return Promise.resolve(null);
    case 'set_profile_note':
      return Promise.resolve(null);
    case 'inspect_profile': {
      // Healthy sample report for browser QA / screenshots, shaped exactly like
      // the Rust inspector's serde output: lowercase SharingMode values and the
      // LINK_ITEMS order (share_settings-style environment).
      const modes = { ...PRESETS.share_settings, cli_sessions: 'isolate', desktop_app_config: 'isolate', desktop_config: 'isolate' };
      const order = ['desktop_config', 'cli_settings', 'cli_claude_md', 'cli_project_memory', 'cli_plugins',
        'cli_skills', 'cli_sessions', 'cli_history', 'desktop_worktrees', 'desktop_device_id', 'desktop_app_config'];
      const items = order.map((key) => (modes[key] === 'share'
        ? { key, mode: 'share', health: { state: 'shared_ok', target: '~/.claude/' + key }, is_issue: false }
        : { key, mode: modes[key], health: { state: 'isolated_ok' }, is_issue: false }));
      return Promise.resolve({ profile: args.name, items, issue_count: 0, running: false });
    }
    case 'get_default_roots_status':
      return Promise.resolve({ desktop_present: true, cli_present: true });
    case 'get_dmg_mount_status':
      // Browser/screenshot mode never shows the banner; QA forces it via eval.
      return Promise.resolve([]);
    case 'eject_dmg':
      return Promise.resolve(null);
    case 'app_version': {
      // The real version lives in tauri.conf.json, which the browser mock cannot
      // read. Screenshot capture injects ?appver= (scripts/appshot); without it
      // the footer version is hidden rather than showing a stale hardcoded one.
      const appver = new URLSearchParams(location.search).get('appver');
      return appver ? Promise.resolve(appver) : Promise.reject(new Error('no appver'));
    }
    case 'open_url':
      return Promise.resolve(null);
    default:
      return Promise.resolve(null);
  }
}
