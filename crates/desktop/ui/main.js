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

// --- The 11 sharing components, grouped and described in plain language ------
const SHARE_GROUPS = [
  {
    label: 'Claudeデスクトップアプリ',
    items: [
      { key: 'desktop_config', name: 'MCP サーバー', desc: '接続済みの MCP サーバー構成' },
      { key: 'desktop_app_config', name: 'アプリ設定', desc: '表示や挙動などの一般設定' },
      { key: 'desktop_worktrees', name: 'ワークツリー', desc: 'Git ワークツリーの対応表' },
    ],
  },
  {
    label: 'Claude Code',
    items: [
      { key: 'cli_settings', name: '権限・フック', desc: '許可設定とフック' },
      { key: 'cli_claude_md', name: 'グローバルルール', desc: 'CLAUDE.md の共通ルール' },
      { key: 'cli_project_memory', name: 'プロジェクト記憶', desc: 'プロジェクトごとのメモリ' },
      { key: 'cli_plugins', name: 'プラグイン', desc: 'インストール済みプラグイン' },
      { key: 'cli_skills', name: 'スキル', desc: 'カスタムスキル定義' },
      { key: 'cli_sessions', name: '会話履歴', desc: 'これまでの会話セッション' },
      { key: 'cli_history', name: 'コマンド履歴', desc: '入力したコマンドの履歴' },
    ],
  },
];
const DEVICE_ID = { key: 'desktop_device_id', name: '端末 ID', desc: '端末を識別するための ID。共有が既定です' };
const ALL_KEYS = [...SHARE_GROUPS.flatMap((g) => g.items.map((i) => i.key)), DEVICE_ID.key];

// Mode presets (must mirror build_sharing_config in main.rs).
const PRESETS = {
  isolate: Object.fromEntries(ALL_KEYS.map((k) => [k, k === 'desktop_device_id' ? 'share' : 'isolate'])),
  share_settings: {
    desktop_config: 'share',
    cli_settings: 'share',
    cli_claude_md: 'share',
    cli_plugins: 'share',
    cli_skills: 'share',
    desktop_app_config: 'share',
    desktop_worktrees: 'share',
    cli_project_memory: 'isolate',
    cli_sessions: 'isolate',
    cli_history: 'isolate',
    desktop_device_id: 'share',
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
function avatarText(p) {
  return p.icon ? p.icon : (p.name || '?').charAt(0).toUpperCase();
}
const reduceMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
function withTransition(update) {
  if (document.startViewTransition && !reduceMotion()) document.startViewTransition(update);
  else update();
}

// --- DOM refs ---------------------------------------------------------------
const el = {
  nowClaude: $('nowClaude'),
  nowClaudeActive: $('nowClaudeActive'),
  createdSection: $('createdSection'),
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
  inputIcon: $('inputIcon'),
  nameError: $('nameError'),
  advancedToggle: $('advancedToggle'),
  advancedState: $('advancedState'),
  advancedReset: $('advancedReset'),
  advancedGroups: $('advancedGroups'),
  btnCreateCancel: $('btnCreateCancel'),
  btnCreateSubmit: $('btnCreateSubmit'),
  emptyScroll: $('emptyScroll'),
  emptyFade: $('emptyFade'),
  detailScroll: $('detailScroll'),
  detailFade: $('detailFade'),
  createScroll: $('createScroll'),
  createFade: $('createFade'),
  toastArea: $('toastArea'),
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
  el.nowClaudeActive.hidden = activeName !== 'default';
  el.nowClaude.classList.toggle('selected', selectedName === 'default');
  el.nowClaude.setAttribute('aria-current', selectedName === 'default' ? 'true' : 'false');

  const created = profiles.filter((p) => p.name !== 'default');
  el.createdSection.hidden = created.length === 0;

  const rows = created.map((p) => {
    const open = () => withTransition(() => showDetail(p.name));
    const li = h('li', {
      class: 'profile-item' + (p.name === selectedName ? ' selected' : ''),
      role: 'button', tabindex: '0', onclick: open,
      onkeydown: (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); open(); } },
    },
      h('span', { class: 'profile-avatar', text: avatarText(p) }),
      h('span', { class: 'profile-name', text: p.name }),
      p.name === activeName ? h('span', { class: 'pill pill-active', text: '使用中' }) : null);
    return li;
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
    showToast('プロファイルを読み込めませんでした。', true);
    return;
  }
  renderSidebar();
  const isDefault = !!d.is_default;
  const isActive = name === activeName;

  const header = h('div', { class: 'detail-header' },
    h('div', { class: 'detail-bezel' },
      h('div', { class: 'detail-avatar', text: d.icon ? d.icon : name.charAt(0).toUpperCase() })),
    h('div', { class: 'detail-titles' },
      h('div', { class: 'detail-name', text: isDefault ? 'いま使っている Claude' : name }),
      h('div', { class: 'detail-tagline', text: isDefault ? 'ふだんの環境。CSW は変更しません' : (isActive ? '使用中の環境' : '作成した環境') })),
    isActive ? h('span', { class: 'pill pill-active', style: 'margin-left:auto', text: '使用中' }) : null);

  const nodes = [header];

  if (isDefault) {
    nodes.push(section('', [
      h('div', { class: 'note-card' },
        'あなたが普段使っている ', h('strong', { text: 'Claudeデスクトップアプリ' }), 'と ',
        h('strong', { text: 'Claude Code' }),
        ' の環境です。CSW はここを表示しているだけで、設定・履歴・ログインを変更したり削除したりしません。切り替えると、いつものこの環境に戻ります。'),
    ]));
    nodes.push(pathsSection(d, true));
    nodes.push(section('共有の状態', [h('div', { class: 'note-card', text: 'すべて元のまま（11 項目すべて共有）。' })]));
  } else {
    nodes.push(pathsSection(d, false));
    nodes.push(sharingDisclosure(d.sharing));
    nodes.push(terminalSection(name));
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
  return section(isDefault ? '場所（本物の Claude フォルダ）' : '場所（このプロファイル）', [
    pathRow('Claudeデスクトップアプリ', d.desktop_path),
    pathRow('Claude Code', d.cli_path),
    isDefault ? h('p', { class: 'path-caption', text: 'これは CSW が作った場所ではなく、あなたの本物の Claude フォルダです。' }) : null,
  ]);
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
  const cmd = `eval $(csw env "${name}")`;
  return section('ターミナルで使う', [
    h('div', { class: 'note-card', text: 'Claude Code でこの環境を使うには、ターミナルで次を実行します。このタブだけに適用され、普段の環境には影響しません。' }),
    h('div', { class: 'path-row', style: 'margin-top:8px' },
      h('div', { class: 'path-meta' }, h('code', { class: 'path-code', text: cmd })),
      copyButton(cmd, 'コマンドをコピー')),
  ]);
}

function sharingDisclosure(sharing) {
  let shareCount = 0;
  for (const k of ALL_KEYS) if (sharing[k] === 'share') shareCount++;
  const isoCount = ALL_KEYS.length - shareCount;

  const inner = [];
  for (const g of SHARE_GROUPS) {
    inner.push(h('div', { class: 'share-group' },
      h('div', { class: 'share-group-label', text: g.label }),
      ...g.items.map((it) => sharingReadRow(it, sharing[it.key]))));
  }
  inner.push(h('div', { class: 'share-group' }, sharingReadRow(DEVICE_ID, sharing[DEVICE_ID.key])));
  // Name the reference point so "shared/isolated" is never ambiguous.
  inner.unshift(h('p', { class: 'share-basis', text: '「共有」はいま使っている Claude と同じもの、「分離」はこの環境だけのものです。' }));

  const wrap = h('div', { class: 'disclosure' });
  const toggle = h('button', {
    type: 'button', class: 'sharing-summary', 'aria-expanded': 'false',
    onclick: () => {
      const open = wrap.classList.toggle('open');
      toggle.setAttribute('aria-expanded', String(open));
      innerWrap.inert = !open; // keep collapsed rows out of the tab order
      requestAnimationFrame(refreshFades);
    },
  },
    h('span', { class: 'summary-text' },
      `共有 ${shareCount} 件・分離 ${isoCount} 件`,
      h('span', { class: 'summary-sub', text: ' ／ ログインと履歴は分離' })),
    icon('i-chevron', 'icon summary-chevron'));
  const innerWrap = h('div', { class: 'disclosure-inner' }, ...inner);
  innerWrap.inert = true;
  appendKids(wrap, [toggle, h('div', { class: 'disclosure-panel' }, innerWrap)]);

  return section('この環境が引き継いでいるもの', [wrap]);
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
  }, icon('i-switch'), h('span', { text: isActive ? '使用中の環境' : 'この環境に切り替える' }));

  if (isDefault) {
    el.detailFooter.replaceChildren(
      h('span', { class: 'confirm-text', style: 'color:var(--ink-muted)', text: 'この環境は変更・削除できません' }),
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
    h('span', { class: 'confirm-text', text: 'この環境を削除します。共有リンクと分離データが消えます。元の Claude には影響しません。' }),
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
async function doSwitch(name) {
  try {
    await invoke('switch_profile', { name, noLaunch: false });
    await refreshProfiles();
    withTransition(() => showDetail(name));
    showToast(`${name === 'default' ? 'いま使っている Claude' : name}に切り替えました`);
  } catch (err) {
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
    showToast('削除できませんでした。使用中のプロファイルは切り替えてから削除してください。', true);
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
function showEmpty() {
  selectedName = null;
  renderSidebar();
  setView('empty');
}

function showCreate() {
  localStorage.setItem('csw_onboarded', '1');
  el.inputName.value = '';
  el.inputIcon.value = '';
  el.nameError.hidden = true;
  setMode('isolate');
  closeAdvanced();
  setView('create');
  setTimeout(() => el.inputName.focus(), 0);
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
  // Name the reference point + define the words once, so all 11 rows read clearly.
  const nodes = [
    h('p', { class: 'share-basis', text: '各項目を、いま使っている Claude と「共有」するか「分離」するか選びます。' }),
    h('p', { class: 'share-legend', text: '共有 = いまのを使う ／ 分離 = この環境だけ ／ コピー = 最初だけ写す' }),
  ];
  for (const g of SHARE_GROUPS) {
    nodes.push(h('div', { class: 'share-group' },
      h('div', { class: 'share-group-label', text: g.label }),
      ...g.items.map((it) => segRow(it, overrides[it.key], false))));
  }
  nodes.push(h('div', { class: 'share-group' },
    segRow(DEVICE_ID, 'share', true),
    h('p', { class: 'path-caption', text: '端末識別のため、端末 ID は常に共有されます。' })));
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
    h('div', { class: 'seg', role: 'radiogroup', 'aria-label': item.name + '：いま使っている Claude と共有・分離・コピーのどれにするか' },
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
  if (!/^[a-zA-Z0-9_-]+$/.test(name)) return '英数字とハイフン、アンダースコアだけ使えます。';
  return null;
}

async function submitCreate() {
  const name = el.inputName.value.trim();
  const iconVal = el.inputIcon.value.trim();
  const err = validateName(name);
  if (err) { el.nameError.textContent = err; el.nameError.hidden = false; el.inputName.focus(); return; }
  el.nameError.hidden = true;

  const args = { name, mode: currentMode, icon: iconVal || null };
  if (advancedCustomized) args.sharingOverrides = { ...overrides };

  el.btnCreateSubmit.disabled = true;
  try {
    await invoke('create_profile', args);
    localStorage.setItem('csw_onboarded', '1');
    selectedName = name;
    await refreshProfiles();
    withTransition(() => showDetail(name));
    showToast(`${name}を作成しました`);
  } catch (e) {
    showToast('作成できませんでした。同じ名前がすでにあるか確認してください。', true);
  } finally {
    el.btnCreateSubmit.disabled = false;
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
  el.btnCreateCancel.addEventListener('click', () => withTransition(() =>
    selectedName ? showDetail(selectedName) : showEmpty()));
  el.btnCreateSubmit.addEventListener('click', submitCreate);
  el.nowClaude.addEventListener('click', () => withTransition(() => showDetail('default')));

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
  el.inputName.addEventListener('keydown', (e) => { if (e.key === 'Enter') submitCreate(); });
}

function checkFirstRun() {
  const show = !localStorage.getItem('csw_onboarded');
  el.emptyFirstRun.hidden = !show;
  el.firstRunExtra.hidden = !show;
}

async function init() {
  wireEvents();
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
    仕事用: { name: '仕事用', icon: '', is_default: false, desktop_path: '~/.context-switcher-claude/profiles/仕事用/desktop-data', cli_path: '~/.context-switcher-claude/profiles/仕事用/cli-data', sharing: { ...PRESETS.share_settings } },
    検証用: { name: '検証用', icon: '', is_default: false, desktop_path: '~/.context-switcher-claude/profiles/検証用/desktop-data', cli_path: '~/.context-switcher-claude/profiles/検証用/cli-data', sharing: { ...PRESETS.isolate } },
  };
  switch (cmd) {
    case 'list_profiles':
      return Promise.resolve([
        { name: 'default', icon: '', is_default: true },
        { name: '仕事用', icon: '', is_default: false },
        { name: '検証用', icon: '', is_default: false },
      ]);
    case 'get_active_profile':
      return Promise.resolve('default');
    case 'get_profile_details':
      return Promise.resolve(sample[args.name] || sample['検証用']);
    default:
      return Promise.resolve(null);
  }
}
