// ============================================================================
// Claude Desktop Switcher — Frontend Logic (Tauri v2 bindings)
// ============================================================================

// Helper to safely invoke Tauri commands
const invoke = window.__TAURI__ ? window.__TAURI__.core.invoke : async (cmd, args) => {
  console.log(`Mock calling command "${cmd}" with args:`, args);
  // Mock implementations for design testing
  if (cmd === 'list_profiles') return ['default', 'Work', 'Personal'];
  if (cmd === 'get_active_profile') return 'default';
  if (cmd === 'get_profile_details') {
    return {
      name: args.name,
      icon: args.name === 'default' ? '💻' : '💼',
      color: '#4A90D9',
      is_default: args.name === 'default',
      desktop_path: `~/.claude-desktop-switcher/profiles/${args.name.toLowerCase()}/desktop-data`,
      cli_path: `~/.claude-desktop-switcher/profiles/${args.name.toLowerCase()}/cli-data`,
      sharing: {
        desktop_config: 'share',
        cli_settings: 'share',
        cli_claude_md: 'share',
        cli_project_memory: 'isolate',
        cli_plugins: 'share',
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
const elShareCliSettings = document.getElementById('share-cli-settings');
const elShareCliClaudeMd = document.getElementById('share-cli-claude-md');
const elShareCliProjectMemory = document.getElementById('share-cli-project-memory');
const elShareCliPlugins = document.getElementById('share-cli-plugins');
const elShareDesktopWorktrees = document.getElementById('share-desktop-worktrees');

const elBtnSwitch = document.getElementById('btn-switch');
const elBtnDelete = document.getElementById('btn-delete');
const elBtnAddProfile = document.getElementById('btn-add-profile');

// Modal Elements
const elModalCreate = document.getElementById('modal-create');
const elInputName = document.getElementById('input-name');
const elSelectPreset = document.getElementById('select-preset');
const elBtnModalCancel = document.getElementById('btn-modal-cancel');
const elBtnModalSubmit = document.getElementById('btn-modal-submit');

// Init
async function init() {
  await refreshProfiles();
  setupEventListeners();
}

// Fetch and render profiles list
async function refreshProfiles() {
  try {
    profilesList = await invoke('list_profiles');
    activeProfileName = await invoke('get_active_profile');
    
    renderProfileList();
    
    // Auto-select active profile detail if none is selected
    if (selectedProfileName && profilesList.includes(selectedProfileName)) {
      await showProfileDetails(selectedProfileName);
    } else {
      elDetailsPanel.classList.add('hidden');
      elWelcomePanel.classList.remove('hidden');
      selectedProfileName = null;
    }
  } catch (err) {
    console.error('Failed to load profiles:', err);
  }
}

// Render profiles sidebar list
function renderProfileList() {
  elProfileList.innerHTML = '';
  
  profilesList.forEach(name => {
    const isDefault = name === 'default';
    const isActive = name === activeProfileName;
    const icon = isDefault ? '💻' : '💼';
    
    const li = document.createElement('li');
    li.className = `profile-item ${name === selectedProfileName ? 'active' : ''}`;
    li.innerHTML = `
      <span class="profile-icon">${icon}</span>
      <span class="profile-name">${name}</span>
      ${isActive ? '<span class="active-dot"></span>' : ''}
    `;
    
    li.addEventListener('click', () => {
      // Remove active class from previous
      document.querySelectorAll('.profile-item').forEach(el => el.classList.remove('active'));
      li.classList.add('active');
      showProfileDetails(name);
    });
    
    elProfileList.appendChild(li);
  });
}

// Load and display profile detailed configuration
async function showProfileDetails(name) {
  try {
    selectedProfileName = name;
    const p = await invoke('get_profile_details', { name });
    
    elWelcomePanel.classList.add('hidden');
    elDetailsPanel.classList.remove('hidden');
    
    // Meta info
    elDetailIcon.textContent = p.name === 'default' ? '💻' : '💼';
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
    updateSharingBadge(elShareCliSettings, p.sharing.cli_settings);
    updateSharingBadge(elShareCliClaudeMd, p.sharing.cli_claude_md);
    updateSharingBadge(elShareCliProjectMemory, p.sharing.cli_project_memory);
    updateSharingBadge(elShareCliPlugins, p.sharing.cli_plugins);
    updateSharingBadge(elShareDesktopWorktrees, p.sharing.desktop_worktrees);
    
    // Switch / Delete button states
    if (p.name === 'default') {
      elBtnDelete.classList.add('hidden');
    } else {
      elBtnDelete.classList.remove('hidden');
    }
    
    if (p.name === activeProfileName) {
      elBtnSwitch.textContent = 'Active Environment';
      elBtnSwitch.disabled = true;
      elBtnSwitch.className = 'btn btn-secondary';
    } else {
      elBtnSwitch.textContent = 'Switch to Profile';
      elBtnSwitch.disabled = false;
      elBtnSwitch.className = 'btn btn-success';
    }
    
  } catch (err) {
    console.error('Failed to load profile details:', err);
  }
}

function updateSharingBadge(el, mode) {
  el.textContent = mode;
  el.className = `sharing-badge ${mode}`;
}

// Event Listeners
function setupEventListeners() {
  // Create Profile Modal
  elBtnAddProfile.addEventListener('click', () => {
    elInputName.value = '';
    elModalCreate.classList.remove('hidden');
    elInputName.focus();
  });
  
  elBtnModalCancel.addEventListener('click', () => {
    elModalCreate.classList.add('hidden');
  });
  
  elBtnModalSubmit.addEventListener('click', async () => {
    const name = elInputName.value.trim();
    const mode = elSelectPreset.value;
    
    if (!name) {
      alert('Please enter a profile name.');
      return;
    }
    
    // Name validations (no default, alphanumeric)
    if (name.toLowerCase() === 'default') {
      alert('Profile name "default" is reserved.');
      return;
    }
    
    if (!/^[a-zA-Z0-9_-]+$/.test(name)) {
      alert('Profile name can only contain letters, numbers, hyphens, and underscores.');
      return;
    }
    
    try {
      await invoke('create_profile', { name, mode });
      elModalCreate.classList.add('hidden');
      selectedProfileName = name;
      await refreshProfiles();
    } catch (err) {
      alert(`Error creating profile: ${err}`);
    }
  });
  
  // Switching Action
  elBtnSwitch.addEventListener('click', async () => {
    if (!selectedProfileName || selectedProfileName === activeProfileName) return;
    
    const originalText = elBtnSwitch.textContent;
    elBtnSwitch.textContent = 'Switching...';
    elBtnSwitch.disabled = true;
    
    try {
      await invoke('switch_profile', { name: selectedProfileName, noLaunch: false });
      await refreshProfiles();
    } catch (err) {
      alert(`Switch failed: ${err}`);
      elBtnSwitch.textContent = originalText;
      elBtnSwitch.disabled = false;
    }
  });
  
  // Deleting Action
  elBtnDelete.addEventListener('click', async () => {
    if (!selectedProfileName || selectedProfileName === 'default') return;
    if (selectedProfileName === activeProfileName) {
      alert('Cannot delete active profile. Switch to another profile first.');
      return;
    }
    
    if (confirm(`Are you sure you want to delete profile "${selectedProfileName}"?\nThis will clean up its symlinks and isolated directories.`)) {
      try {
        await invoke('delete_profile', { name: selectedProfileName });
        selectedProfileName = null;
        await refreshProfiles();
      } catch (err) {
        alert(`Delete failed: ${err}`);
      }
    }
  });
}

// Start
document.addEventListener('DOMContentLoaded', init);
