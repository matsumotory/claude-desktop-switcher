// ============================================================================
// Claude Desktop Switcher — Frontend Logic (Tauri v2 bindings)
// ============================================================================

// Helper to safely invoke Tauri commands
const invoke = window.__TAURI__ ? window.__TAURI__.core.invoke : async (cmd, args) => {
  console.log(`Mock calling command "${cmd}" with args:`, args);
  // Mock implementations for design testing
  if (cmd === 'list_profiles') return [
    { name: 'default', icon: '', is_default: true },
    { name: 'Work', icon: '💼', is_default: false },
    { name: 'Personal', icon: '🏠', is_default: false }
  ];
  if (cmd === 'get_active_profile') return 'default';
  if (cmd === 'get_profile_details') {
    return {
      name: args.name,
      icon: args.name === 'default' ? '' : (args.name === 'Work' ? '💼' : '🏠'),
      color: '#4A90D9',
      is_default: args.name === 'default',
      desktop_path: `~/.context-switcher-claude/profiles/${args.name.toLowerCase()}/desktop-data`,
      cli_path: `~/.context-switcher-claude/profiles/${args.name.toLowerCase()}/cli-data`,
      sharing: {
        desktop_config: 'share',
        desktop_app_config: 'share',
        cli_settings: 'share',
        cli_claude_md: 'share',
        cli_project_memory: 'isolate',
        cli_plugins: 'share',
        cli_skills: 'share',
        cli_sessions: 'isolate',
        cli_history: 'isolate',
        desktop_worktrees: 'isolate',
        desktop_device_id: 'share'
      }
    };
  }
  return null;
};

// DOM State
let profilesList = [];
let activeProfileName = 'default';
let selectedProfileName = null;

// DOM Elements
const elProfileList = document.getElementById('profile-list');
const elWelcomePanel = document.getElementById('welcome-panel');
const elDetailsPanel = document.getElementById('profile-details');

const elDetailIcon = document.getElementById('detail-icon');
const elDetailName = document.getElementById('detail-name');
const elDetailActiveTag = document.getElementById('detail-active-tag');
const elPathDesktop = document.getElementById('path-desktop');
const elPathCli = document.getElementById('path-cli');

const elShareDesktopConfig = document.getElementById('share-desktop-config');
const elShareDesktopAppConfig = document.getElementById('share-desktop-app-config');
const elShareCliSettings = document.getElementById('share-cli-settings');
const elShareCliClaudeMd = document.getElementById('share-cli-claude-md');
const elShareCliProjectMemory = document.getElementById('share-cli-project-memory');
const elShareCliPlugins = document.getElementById('share-cli-plugins');
const elShareCliSkills = document.getElementById('share-cli-skills');
const elShareCliSessions = document.getElementById('share-cli-sessions');
const elShareCliHistory = document.getElementById('share-cli-history');
const elShareDesktopWorktrees = document.getElementById('share-desktop-worktrees');

const elBtnSwitch = document.getElementById('btn-switch');
const elBtnClone = document.getElementById('btn-clone');
const elBtnDelete = document.getElementById('btn-delete');
const elBtnAddProfile = document.getElementById('btn-add-profile');

// Modal Elements
const elModalCreate = document.getElementById('modal-create');
const elInputName = document.getElementById('input-name');
const elInputIcon = document.getElementById('input-icon');
const elSelectPreset = document.getElementById('select-preset');
const elBtnModalCancel = document.getElementById('btn-modal-cancel');
const elBtnModalSubmit = document.getElementById('btn-modal-submit');

// Clone Modal Elements
const elModalClone = document.getElementById('modal-clone');
const elInputCloneName = document.getElementById('input-clone-name');
const elBtnModalCloneCancel = document.getElementById('btn-modal-clone-cancel');
const elBtnModalCloneSubmit = document.getElementById('btn-modal-clone-submit');

// Onboarding Elements
const elOnboardingOverlay = document.getElementById('onboarding-overlay');
const elBtnOnboardingNext = document.getElementById('btn-onboarding-next');

// Init
async function init() {
  await refreshProfiles();
  setupEventListeners();
  checkOnboarding();
}

function checkOnboarding() {
  const onboarded = localStorage.getItem('csw_onboarded');
  if (!onboarded) {
    elOnboardingOverlay.classList.remove('hidden');
    showSlide(1);
  }
}

// Fetch and render profiles list
async function refreshProfiles() {
  try {
    profilesList = await invoke('list_profiles');
    activeProfileName = await invoke('get_active_profile');
    
    renderProfileList();
    
    // Auto-select active profile detail if none is selected
    if (selectedProfileName && profilesList.some(p => p.name === selectedProfileName)) {
      await showProfileDetails(selectedProfileName);
    } else {
      elDetailsPanel.classList.add('hidden');
      elWelcomePanel.classList.remove('hidden');
      selectedProfileName = null;
    }
  } catch (err) {
    console.error('プロファイルの読み込みに失敗しました:', err);
  }
}

// Render profiles sidebar list
function renderProfileList() {
  elProfileList.innerHTML = '';
  
  profilesList.forEach(p => {
    const isActive = p.name === activeProfileName;
    const iconContent = p.icon ? p.icon : p.name.charAt(0).toUpperCase();
    
    const li = document.createElement('li');
    li.className = `profile-item ${p.name === selectedProfileName ? 'active' : ''}`;
    li.innerHTML = `
      <span class="profile-avatar">${iconContent}</span>
      <span class="profile-name">${p.name}</span>
      ${isActive ? '<span class="active-dot"></span>' : ''}
    `;
    
    li.addEventListener('click', () => {
      // Remove active class from previous
      document.querySelectorAll('.profile-item').forEach(el => el.classList.remove('active'));
      li.classList.add('active');
      showProfileDetails(p.name);
    });
    
    elProfileList.appendChild(li);
  });
}

// Sharing mode display labels
function sharingLabel(mode) {
  return mode === 'share' ? '共有' : '分離';
}

// Load and display profile detailed configuration
async function showProfileDetails(name) {
  try {
    selectedProfileName = name;
    const p = await invoke('get_profile_details', { name });
    
    elWelcomePanel.classList.add('hidden');
    elDetailsPanel.classList.remove('hidden');
    
    // Meta info
    elDetailIcon.textContent = p.icon ? p.icon : p.name.charAt(0).toUpperCase();
    elDetailName.textContent = p.name;
    
    // Status Tag
    if (p.name === activeProfileName) {
      elDetailActiveTag.classList.remove('hidden');
    } else {
      elDetailActiveTag.classList.add('hidden');
    }
    
    // Paths
    elPathDesktop.textContent = p.desktop_path;
    elPathCli.textContent = p.cli_path;
    
    // Sharing Badges
    updateSharingBadge(elShareDesktopConfig, p.sharing.desktop_config);
    updateSharingBadge(elShareDesktopAppConfig, p.sharing.desktop_app_config);
    updateSharingBadge(elShareCliSettings, p.sharing.cli_settings);
    updateSharingBadge(elShareCliClaudeMd, p.sharing.cli_claude_md);
    updateSharingBadge(elShareCliProjectMemory, p.sharing.cli_project_memory);
    updateSharingBadge(elShareCliPlugins, p.sharing.cli_plugins);
    updateSharingBadge(elShareCliSkills, p.sharing.cli_skills);
    updateSharingBadge(elShareCliSessions, p.sharing.cli_sessions);
    updateSharingBadge(elShareCliHistory, p.sharing.cli_history);
    updateSharingBadge(elShareDesktopWorktrees, p.sharing.desktop_worktrees);
    
    // Switch / Delete button states
    if (p.name === 'default') {
      elBtnDelete.classList.add('hidden');
      elBtnClone.classList.add('hidden');
    } else {
      elBtnDelete.classList.remove('hidden');
      elBtnClone.classList.remove('hidden');
    }
    
    if (p.name === activeProfileName) {
      elBtnSwitch.textContent = '使用中の環境';
      elBtnSwitch.disabled = true;
      elBtnSwitch.className = 'btn btn-secondary';
    } else {
      elBtnSwitch.textContent = 'このプロファイルに切り替え';
      elBtnSwitch.disabled = false;
      elBtnSwitch.className = 'btn btn-success';
    }
    
  } catch (err) {
    console.error('プロファイル詳細の読み込みに失敗しました:', err);
  }
}

function updateSharingBadge(el, mode) {
  el.textContent = sharingLabel(mode);
  el.className = `sharing-badge ${mode}`;
}

// Event Listeners
function setupEventListeners() {
  // Create Profile Modal
  elBtnAddProfile.addEventListener('click', () => {
    elInputName.value = '';
    elInputIcon.value = '';
    elModalCreate.classList.remove('hidden');
    elInputName.focus();
  });
  
  elBtnModalCancel.addEventListener('click', () => {
    elModalCreate.classList.add('hidden');
  });
  
  elBtnModalSubmit.addEventListener('click', async () => {
    const name = elInputName.value.trim();
    const icon = elInputIcon.value.trim();
    const mode = elSelectPreset.value;
    
    if (!name) {
      alert('プロファイル名を入力してください。');
      return;
    }
    
    // Name validations (no default, alphanumeric)
    if (name.toLowerCase() === 'default') {
      alert('「default」は予約されたプロファイル名です。');
      return;
    }
    
    if (!/^[a-zA-Z0-9_-]+$/.test(name)) {
      alert('プロファイル名には英数字、ハイフン、アンダースコアのみ使用できます。');
      return;
    }
    
    try {
      await invoke('create_profile', { name, mode, icon: icon || null });
      elModalCreate.classList.add('hidden');
      selectedProfileName = name;
      await refreshProfiles();
    } catch (err) {
      alert(`プロファイルの作成に失敗しました: ${err}`);
    }
  });
  
  // Switching Action
  elBtnSwitch.addEventListener('click', async () => {
    if (!selectedProfileName || selectedProfileName === activeProfileName) return;
    
    const originalText = elBtnSwitch.textContent;
    elBtnSwitch.textContent = '切り替え中...';
    elBtnSwitch.disabled = true;
    
    try {
      await invoke('switch_profile', { name: selectedProfileName, noLaunch: false });
      await refreshProfiles();
    } catch (err) {
      alert(`切り替えに失敗しました: ${err}`);
      elBtnSwitch.textContent = originalText;
      elBtnSwitch.disabled = false;
    }
  });
  
  // Deleting Action
  elBtnDelete.addEventListener('click', async () => {
    if (!selectedProfileName || selectedProfileName === 'default') return;
    if (selectedProfileName === activeProfileName) {
      alert('使用中のプロファイルは削除できません。先に別のプロファイルに切り替えてください。');
      return;
    }
    
    if (confirm(`プロファイル「${selectedProfileName}」を削除しますか？\nシンボリックリンクと分離ディレクトリがクリーンアップされます。`)) {
      try {
        await invoke('delete_profile', { name: selectedProfileName });
        selectedProfileName = null;
        await refreshProfiles();
      } catch (err) {
        alert(`削除に失敗しました: ${err}`);
      }
    }
  });

  // Clone Modal Show
  elBtnClone.addEventListener('click', () => {
    if (!selectedProfileName) return;
    elInputCloneName.value = '';
    elModalClone.classList.remove('hidden');
    elInputCloneName.focus();
  });

  elBtnModalCloneCancel.addEventListener('click', () => {
    elModalClone.classList.add('hidden');
  });

  elBtnModalCloneSubmit.addEventListener('click', async () => {
    const name = elInputCloneName.value.trim();
    if (!name) {
      alert('プロファイル名を入力してください。');
      return;
    }

    if (name.toLowerCase() === 'default') {
      alert('「default」は予約されたプロファイル名です。');
      return;
    }

    if (!/^[a-zA-Z0-9_-]+$/.test(name)) {
      alert('プロファイル名には英数字、ハイフン、アンダースコアのみ使用できます。');
      return;
    }

    try {
      await invoke('clone_profile', { source: selectedProfileName, target: name });
      elModalClone.classList.add('hidden');
      selectedProfileName = name;
      await refreshProfiles();
    } catch (err) {
      alert(`プロファイルの複製に失敗しました: ${err}`);
    }
  });

  // Onboarding Slides Control
  let currentSlide = 1;
  const totalSlides = 3;

  function showSlide(num) {
    currentSlide = num;
    for (let i = 1; i <= totalSlides; i++) {
      const slide = document.getElementById(`slide-${i}`);
      if (i === num) {
        slide.classList.remove('hidden');
      } else {
        slide.classList.add('hidden');
      }
    }

    // Update Dots
    document.querySelectorAll('.slide-dots .dot').forEach(dot => {
      if (parseInt(dot.getAttribute('data-slide')) === num) {
        dot.classList.add('active');
      } else {
        dot.classList.remove('active');
      }
    });

    // Update Button Text
    if (num === totalSlides) {
      elBtnOnboardingNext.textContent = '開始する';
    } else {
      elBtnOnboardingNext.textContent = '次へ';
    }
  }

  elBtnOnboardingNext.addEventListener('click', () => {
    if (currentSlide < totalSlides) {
      showSlide(currentSlide + 1);
    } else {
      // End onboarding
      localStorage.setItem('csw_onboarded', 'true');
      elOnboardingOverlay.classList.add('hidden');
    }
  });

  document.querySelectorAll('.slide-dots .dot').forEach(dot => {
    dot.addEventListener('click', () => {
      const target = parseInt(dot.getAttribute('data-slide'));
      showSlide(target);
    });
  });
}

// Start
document.addEventListener('DOMContentLoaded', init);
