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
    const isActive = p.name === activeName;
    const open = () => withTransition(() => showDetail(p.name));
    // One pill at most: the active ("利用中") row. The default row needs no extra
    // marker — its name ("既存の Claude") + monitor icon + first position already
    // signal the baseline.
    const pill = isActive
      ? h('span', { class: 'pill pill-active', text: '利用中' })
      : null;
    return h('li', {
      class: 'profile-item' + (p.name === selectedName ? ' selected' : ''),
      role: 'button', tabindex: '0', onclick: open,
      'aria-label': isDefault ? '既存の Claude（標準環境）' : null,
      'aria-current': isActive ? 'true' : null, // current = in-use target
      onkeydown: (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); open(); } },
    },
      h('span', { class: 'profile-avatar' }, isDefault ? icon('i-monitor') : avatarContent(p.icon, p.name)),
      h('span', { class: 'profile-name', text: isDefault ? '既存の Claude' : p.name }),
      pill);
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
  const isActive = name === activeName;

  const header = h('div', { class: 'detail-header' },
    h('div', { class: 'detail-bezel' },
      h('div', { class: 'detail-avatar' }, isDefault ? icon('i-monitor') : avatarContent(d.icon, name))),
    h('div', { class: 'detail-titles' },
      h('div', { class: 'detail-name', text: isDefault ? '既存の Claude' : name }),
      h('div', { class: 'detail-tagline', text: isDefault ? 'あなた自身の環境' : (isActive ? '利用中の環境' : '作成した環境') })),
    isActive ? h('span', { class: 'pill pill-active', style: 'margin-left:auto', 'aria-label': '利用中', text: '利用中' }) : null);

  const nodes = [header];

  if (isDefault) {
    // Keep only the reassurance (the user's real worry: "will this break my
    // setup?"). The paths are collapsed; the "all 11 shared" line is dropped as
    // self-evident for the baseline.
    nodes.push(section('', [
      h('div', { class: 'note-card' },
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
    nodes.push(terminalSection(name));
    if (!isActive) {
      nodes.push(h('p', { class: 'detail-switch-hint', text:
        '先に起動中の Claude を終了してから押すと、この環境の Claude が開きます。' }));
    }
  }

  el.detailContent.replaceChildren(...nodes);
  renderDetailFooter(name, isDefault, isActive);
  setView('detail');
}

function section(label, children) {
  return h('div', { class: 'section' },
    label ? h('div', { class: 'section-label', text: label }) : null, ...children);
}

function pathsSection(d, isDefault) {
  const rows = [
    pathRow('Claudeデスクトップアプリ', d.desktop_path),
    pathRow('Claude Code', d.cli_path),
    isDefault ? h('p', { class: 'path-caption', text: 'これは CSW が作った場所ではなく、あなた自身の Claude フォルダです。' }) : null,
  ];
  // Paths are needed rarely (not for switch/clone/delete) → collapsed by default.
  return section('', [disclosure(isDefault ? '場所（あなたの Claude フォルダ）' : '場所（この環境のデータ）', null, rows)]);
}

function pathRow(label, value) {
  return h('div', { class: 'path-row' },
    h('div', { class: 'path-meta' },
      h('span', { class: 'path-label', text: label }),
      h('code', { class: 'path-code', text: value })),
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
    h('p', { class: 'share-basis', text:
      'CSW から開いたターミナルは、最初からこの環境です。別に開く iTerm2 などのターミナルでは、次を実行します（そのタブだけに適用されます）。' }),
    h('div', { class: 'path-row' },
      h('div', { class: 'path-meta' }, h('code', { class: 'path-code', text: cmd })),
      copyButton(cmd, 'コマンドをコピー')),
  ];
  return section('', [disclosure('ターミナル（Claude Code）で使う', null, inner)]);
}

// Generic collapsible disclosure: a summary row stays visible, the panel opens
// on click. Keeps secondary detail (paths, the terminal command, the full
// sharing breakdown) off the always-on surface so the detail view shows state +
// actions first and discloses the rest only when asked.
function disclosure(label, sub, innerNodes) {
  const wrap = h('div', { class: 'disclosure' });
  const innerWrap = h('div', { class: 'disclosure-inner' }, ...innerNodes);
  innerWrap.inert = true; // keep collapsed content out of the tab order
  const toggle = h('button', {
    type: 'button', class: 'sharing-summary', 'aria-expanded': 'false',
    onclick: () => {
      const open = wrap.classList.toggle('open');
      toggle.setAttribute('aria-expanded', String(open));
      innerWrap.inert = !open;
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
    ? `共有 ${shareCount}・コピー ${copyCount}・分離 ${isoCount} 件`
    : `共有 ${shareCount}・分離 ${isoCount} 件`;

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
    [disclosure(summary, ' ／ アカウントは常に分離', inner)]);
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

function renderDetailFooter(name, isDefault, isActive) {
  el.detailFooter.className = 'view-footer split';
  const switchBtn = h('button', {
    type: 'button', class: 'btn btn-primary', disabled: isActive,
    onclick: () => { if (!isActive) doSwitch(name); },
  }, icon('i-switch'), h('span', { text: isActive ? '利用中の環境' : (isDefault ? '既存の Claude に切り替える' : 'この環境で Claude を起動') }));

  if (isDefault) {
    el.detailFooter.replaceChildren(
      h('span', { class: 'confirm-text', style: 'color:var(--ink-muted)', text: '既存の Claude は変更・削除できません' }),
      switchBtn);
    return;
  }
  el.detailFooter.replaceChildren(
    h('div', { class: 'footer-group' },
      switchBtn,
      h('button', { type: 'button', class: 'btn btn-ghost', onclick: () => showCloneRow(name) },
        icon('i-duplicate'), h('span', { text: '複製' }))),
    h('button', { type: 'button', class: 'btn btn-danger', onclick: () => showDeleteRow(name) },
      icon('i-trash'), h('span', { text: '削除' })));
}

// --- Inline confirm / clone rows (no window.confirm) ------------------------
function showDeleteRow(name) {
  el.detailFooter.className = 'view-footer split';
  const cancel = h('button', { type: 'button', class: 'btn btn-ghost', onclick: () => showDetail(name) }, 'やめる');
  el.detailFooter.replaceChildren(
    h('span', { class: 'confirm-text', text: 'この環境を削除します。共有リンクと分離データが消えます。既存の Claude には影響しません。' }),
    h('div', { class: 'footer-group' },
      cancel,
      h('button', { type: 'button', class: 'btn btn-danger-solid', onclick: () => doDelete(name) }, '削除する')));
  cancel.focus(); // default focus on the safe action
}

function showCloneRow(name) {
  el.detailFooter.className = 'view-footer';
  const input = h('input', {
    type: 'text', class: 'input', placeholder: '複製先の名前（例: 仕事用-控え）',
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
function showSwitchBlocked(name) {
  el.detailFooter.className = 'view-footer split';
  el.detailFooter.replaceChildren(
    h('span', { class: 'confirm-text', text: '起動中の Claude を終了してから、もう一度押してください。設定の衝突を防ぐため、複数の環境の Claude は同時に開けません。' }),
    h('div', { class: 'footer-group' },
      h('button', { type: 'button', class: 'btn btn-ghost', onclick: () => showDetail(name) }, '閉じる'),
      h('button', { type: 'button', class: 'btn btn-primary', onclick: () => doSwitch(name) }, 'もう一度試す')));
}

async function doSwitch(name) {
  try {
    if (await invoke('get_desktop_running_status')) { showSwitchBlocked(name); return; }
    await invoke('switch_profile', { name, noLaunch: false });
    await refreshProfiles();
    withTransition(() => showDetail(name));
    showToast(name === 'default' ? '既存の Claude に切り替えました' : `${name} の Claude を起動しました`);
  } catch (err) {
    if (String(err || '').includes('Claude Desktop is running')) { showSwitchBlocked(name); return; }
    showToast('切り替えできませんでした。もう一度お試しください。', true);
  }
}

async function doDelete(name) {
  try {
    await invoke('delete_profile', { name });
    selectedName = null;
    await refreshProfiles();
    const remaining = profiles.filter((p) => p.name !== 'default');
    withTransition(() => (remaining.length ? showDetail(remaining[0].name) : showEmpty()));
    showToast(`${name}を削除しました`);
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
    showToast(`${target}を複製しました（元の環境はそのまま）`);
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
    h('span', { class: 'confirm-text', text: `「${name}」を作成します。` }),
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
    h('div', { class: 'seg', role: 'radiogroup', 'aria-label': item.name + '：既存の Claude と共有・分離・コピーのどれにするか' },
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
  if (!/^[\p{L}\p{N}_-]+$/u.test(name)) return '文字・数字・ハイフン・アンダースコアだけ使えます（空白や記号は使えません）。';
  return null;
}

async function submitCreate() {
  const name = el.inputName.value.trim();
  const iconVal = createIcon;
  const err = validateName(name);
  if (err) { el.nameError.textContent = err; el.nameError.hidden = false; el.inputName.focus(); return; }
  el.nameError.hidden = true;

  const args = { name, mode: currentMode, icon: iconVal || null };
  if (advancedCustomized) args.sharingOverrides = { ...overrides };

  const confirmBtn = el.createFooter.querySelector('.btn-primary');
  if (confirmBtn) confirmBtn.disabled = true;
  try {
    await invoke('create_profile', args);
    localStorage.setItem('csw_onboarded', '1');
    selectedName = name;
    await refreshProfiles();
    withTransition(() => showDetail(name));
    showToast(`${name}を作成しました`);
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
  renderSidebar();
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
  ['非公式プロジェクト', '本プロジェクトは非公式のコミュニティ製で、Anthropic 社とは関係ありません。「Claude」は Anthropic の商標です。'],
];

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
  const card = h('div', { class: 'about-card', role: 'dialog', 'aria-modal': 'true', 'aria-label': 'このアプリについて' },
    h('div', { class: 'about-title', text: 'Claude Desktop Switcher' }),
    h('div', { class: 'about-version', text: el.appVersion.textContent || '' }),
    ...DISCLAIMER.map(([t, b]) => h('div', { class: 'about-item' },
      h('div', { class: 'about-item-title', text: t }),
      h('div', { class: 'about-item-body', text: b }))),
    h('div', { class: 'about-foot' }, closeBtn));
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

async function init() {
  initAccent();
  wireEvents();
  wireFooter();
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
  const sample = {
    default: { name: 'default', icon: '', is_default: true, desktop_path: '~/Library/Application Support/Claude', cli_path: '~/.claude', sharing: Object.fromEntries(ALL_KEYS.map((k) => [k, 'share'])) },
    仕事用: { name: '仕事用', icon: 'briefcase', is_default: false, desktop_path: '~/.context-switcher-claude/profiles/仕事用/desktop-data', cli_path: '~/.context-switcher-claude/profiles/仕事用/cli-data', sharing: { ...PRESETS.share_settings } },
    研究用: { name: '研究用', icon: 'graduation-cap', is_default: false, desktop_path: '~/.context-switcher-claude/profiles/研究用/desktop-data', cli_path: '~/.context-switcher-claude/profiles/研究用/cli-data', sharing: { ...PRESETS.share_workspace } },
    検証用: { name: '検証用', icon: 'flask', is_default: false, desktop_path: '~/.context-switcher-claude/profiles/検証用/desktop-data', cli_path: '~/.context-switcher-claude/profiles/検証用/cli-data', sharing: { ...PRESETS.isolate } },
  };
  switch (cmd) {
    case 'list_profiles':
      return Promise.resolve([
        { name: 'default', icon: '', is_default: true },
        { name: '仕事用', icon: 'briefcase', is_default: false },
        { name: '研究用', icon: 'graduation-cap', is_default: false },
        { name: '検証用', icon: 'flask', is_default: false },
      ]);
    case 'get_active_profile':
      return Promise.resolve('default');
    case 'get_profile_details':
      return Promise.resolve(sample[args.name] || sample['検証用']);
    case 'get_desktop_running_status':
      return Promise.resolve(false);
    case 'get_default_roots_status':
      return Promise.resolve({ desktop_present: true, cli_present: true });
    case 'app_version':
      return Promise.resolve('0.12.0');
    case 'open_url':
      return Promise.resolve(null);
    default:
      return Promise.resolve(null);
  }
}
