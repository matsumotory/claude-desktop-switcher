#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Arc;
use tauri::{
    AppHandle, Manager, State, Wry,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

mod dmg;

use csw_core::platform::create_provider;
use csw_core::profile::{ProfileManager, SharingConfig, SharingMode};
use csw_core::switcher::{ContextSwitcher, desktop_dir_running};

// Global state holding our components
struct AppState {
    provider: Arc<dyn csw_core::platform::PlatformProvider + Send + Sync>,
    profile_manager: Arc<ProfileManager>,
}

// Tauri commands exposed to frontend settings UI

#[tauri::command]
async fn get_active_profile(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.profile_manager.active_profile_name())
}

#[tauri::command]
async fn list_profiles(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let names = state
        .profile_manager
        .list_profiles()
        .map_err(|e| e.to_string())?;
    let mut list = Vec::new();
    for name in names {
        if let Ok(p) = state.profile_manager.get_profile(&name) {
            list.push(serde_json::json!({
                "name": p.profile.name,
                "icon": p.profile.icon,
                "is_default": p.profile.is_default,
            }));
        }
    }
    Ok(list)
}

#[tauri::command]
async fn get_profile_details(
    name: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let p = state
        .profile_manager
        .get_profile(&name)
        .map_err(|e| e.to_string())?;

    // Convert to JSON representation for frontend
    let val = serde_json::json!({
        "name": p.profile.name,
        "icon": p.profile.icon,
        "color": p.profile.color,
        "is_default": p.profile.is_default,
        "desktop_path": p.isolation.desktop_user_data_dir,
        "cli_path": p.isolation.cli_config_dir,
        // Only fully-isolated (すべて分ける) non-default environments may be opened
        // in additional concurrent windows: they share nothing, so parallel
        // instances cannot race on a shared file. The UI shows the dedicated
        // "重複して起動" button only when this is true.
        "supports_concurrent_windows": !p.profile.is_default && p.sharing.is_fully_isolated(),
        "sharing": {
            "cli_settings": format!("{:?}", p.sharing.cli_settings).to_lowercase(),
            "cli_claude_md": format!("{:?}", p.sharing.cli_claude_md).to_lowercase(),
            "cli_project_memory": format!("{:?}", p.sharing.cli_project_memory).to_lowercase(),
            "cli_plugins": format!("{:?}", p.sharing.cli_plugins).to_lowercase(),
            "cli_skills": format!("{:?}", p.sharing.cli_skills).to_lowercase(),
            "cli_history": format!("{:?}", p.sharing.cli_history).to_lowercase(),
            "desktop_worktrees": format!("{:?}", p.sharing.desktop_worktrees).to_lowercase()
        }
    });

    Ok(val)
}

/// Parse a sharing-mode string coming from the frontend ("share" / "isolate" / "copy").
fn parse_sharing_mode(value: &str) -> Option<SharingMode> {
    match value {
        "share" => Some(SharingMode::Share),
        "isolate" => Some(SharingMode::Isolate),
        "copy" => Some(SharingMode::Copy),
        _ => None,
    }
}

/// Build the per-component sharing config for a new profile from the chosen mode preset,
/// then apply any explicit per-component overrides coming from the "advanced settings" UI.
fn build_sharing_config(mode: &str, overrides: Option<HashMap<String, String>>) -> SharingConfig {
    // Three modes surfaced in the UI, framed by use case (every account belongs to
    // the same user, so sharing is a continuity choice, not a cross-tenant leak):
    //   "isolate"         — すべて分ける: a fully separated environment, nothing
    //                       carried over (= SharingConfig::default()). For clients,
    //                       projects, or work-vs-personal that must not mix.
    //   "share_settings"  — 会話とメモリも分ける: reuse the common setup (CLAUDE.md,
    //                       plugins, skills shared; settings/worktrees copied) while
    //                       keeping conversations and login separate.
    //   "share_workspace" — アカウントだけ分ける: also carry the conversation history,
    //                       project memory and command history across, separating
    //                       only the account (billing / resource usage).
    // In every mode the login, OAuth tokens (config.json), claude_desktop_config.json
    // and device id stay isolated. "share" is a backward-compatible alias for
    // "share_settings". See SharingConfig::{share_settings,share_workspace}_preset.
    let mut sharing = match mode {
        "share_settings" | "share" => SharingConfig::share_settings_preset(),
        "share_workspace" => SharingConfig::share_workspace_preset(),
        _ => SharingConfig::default(),
    };

    // Advanced settings: explicit per-component choices override the mode preset.
    // The always-isolated files (config.json, claude_desktop_config.json, sessions/,
    // ant-did) are not SharingConfig fields and have no override key, so neither the
    // UI nor any caller can share or copy them — the isolation is structural,
    // enforced unconditionally by the linker.
    if let Some(overrides) = overrides {
        for (key, value) in overrides {
            let Some(m) = parse_sharing_mode(&value) else {
                continue;
            };
            match key.as_str() {
                "cli_settings" => sharing.cli_settings = m,
                "cli_claude_md" => sharing.cli_claude_md = m,
                "cli_project_memory" => sharing.cli_project_memory = m,
                "cli_plugins" => sharing.cli_plugins = m,
                "cli_skills" => sharing.cli_skills = m,
                "cli_history" => sharing.cli_history = m,
                "desktop_worktrees" => sharing.desktop_worktrees = m,
                _ => {}
            }
        }
    }

    sharing
}

#[tauri::command]
async fn create_profile(
    name: String,
    mode: String,
    icon: Option<String>,
    sharing_overrides: Option<HashMap<String, String>>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let sharing = build_sharing_config(&mode, sharing_overrides);

    state
        .profile_manager
        .create_profile(&name, sharing, icon)
        .map_err(|e| e.to_string())?;

    // Update system tray menu after change
    update_tray_menu(&app)?;

    Ok(())
}

#[tauri::command]
async fn delete_profile(
    name: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    state
        .profile_manager
        .delete_profile(&name)
        .map_err(|e| e.to_string())?;
    update_tray_menu(&app)?;
    Ok(())
}

#[tauri::command]
async fn clone_profile(
    source: String,
    target: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    state
        .profile_manager
        .clone_profile(&source, &target)
        .map_err(|e| e.to_string())?;
    update_tray_menu(&app)?;
    Ok(())
}

/// Read-only isolation check of one environment's link points (SPECIFICATION.md
/// §5.A「分離の検査」). The GUI never repairs anything; fixing stays in the CLI.
#[tauri::command]
async fn inspect_profile(
    name: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let report = state
        .profile_manager
        .inspect_profile_isolation(&name)
        .map_err(|e| e.to_string())?;
    serde_json::to_value(report).map_err(|e| e.to_string())
}

#[tauri::command]
async fn switch_profile(
    name: String,
    no_launch: bool,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let switcher = ContextSwitcher::new(state.provider.clone(), state.profile_manager.clone());
    switcher.switch_to(&name).map_err(|e| e.to_string())?;

    // Update system tray menu
    update_tray_menu(&app)?;

    // Auto-launch after switching, including back to the default ("既存の Claude").
    // For default this resolves to the standard data dirs, so it is equivalent to a
    // normal Finder/Dock launch; `open -n` starts a fresh LaunchServices process that
    // inherits no CSW state, and switching is already refused while Claude is running.
    if !no_launch {
        let profile = state
            .profile_manager
            .get_profile(&name)
            .map_err(|e| e.to_string())?;
        let _ = csw_core::switcher::desktop::launch_desktop(&profile, state.provider.as_ref());
    }

    Ok(())
}

#[tauri::command]
async fn get_desktop_running_status(state: State<'_, AppState>) -> Result<bool, String> {
    state
        .provider
        .is_claude_desktop_running()
        .map_err(|e| e.to_string())
}

/// Launch a fully-isolated environment in an additional window, alongside whatever
/// Claude is already running. Unlike `switch_profile`, this neither changes the
/// active environment nor requires quitting a running Claude first: fully-isolated
/// environments share nothing, so concurrent instances cannot race on a shared
/// file. Refused for the default environment and for any environment that shares
/// or copies a component, keeping the unsafe concurrent case unrepresentable.
#[tauri::command]
async fn launch_additional_window(
    name: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let profile = state
        .profile_manager
        .get_profile(&name)
        .map_err(|e| e.to_string())?;
    if profile.profile.is_default || !profile.sharing.is_fully_isolated() {
        return Err(
            "この環境は完全分離ではないため、複数のウィンドウで同時に開けません。".to_string(),
        );
    }
    csw_core::switcher::desktop::launch_desktop(&profile, state.provider.as_ref())
        .map_err(|e| e.to_string())?;
    // Best-effort: the new process may not appear in the process list instantly,
    // so the 3s running-state watcher is what ultimately refreshes the "利用中"
    // markers; rebuild now anyway so an already-visible change is reflected early.
    update_tray_menu(&app)?;
    Ok(())
}

/// Names of environments whose Claude Desktop is currently running. With
/// fully-isolated environments able to run side by side, this can be more than one.
/// Each profile's `--user-data-dir` is matched against the live main processes.
#[tauri::command]
async fn get_running_profiles(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(running_profile_names(&state))
}

/// Shared by the tray and `get_running_profiles`: resolve which environments are
/// running from the live Claude main processes' `--user-data-dir`.
fn running_profile_names(state: &AppState) -> Vec<String> {
    let args = state.provider.running_desktop_args().unwrap_or_default();
    let default_dir = state.provider.claude_desktop_default_dir();
    let names = state.profile_manager.list_profiles().unwrap_or_default();
    names
        .into_iter()
        .filter(|name| {
            state
                .profile_manager
                .get_profile(name)
                .map(|p| {
                    desktop_dir_running(&p.isolation.desktop_user_data_dir, &args, &default_dir)
                })
                .unwrap_or(false)
        })
        .collect()
}

/// Report whether the standard existing-Claude data dirs (Desktop / CLI) hold
/// data. The create flow gates the "share" mode when there is nothing to share.
#[tauri::command]
async fn get_default_roots_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let s = state.profile_manager.default_roots_status();
    Ok(serde_json::json!({
        "desktop_present": s.desktop_present,
        "cli_present": s.cli_present,
    }))
}

/// Mounted CSW installer disk images the user can eject (SPECIFICATION.md §5.A
/// インストール用ディスクイメージの取り出し案内). Empty when there is nothing
/// to prompt about.
#[tauri::command]
async fn get_dmg_mount_status() -> Result<Vec<String>, String> {
    Ok(dmg::current_mount_status())
}

/// Detach a mount point previously reported by get_dmg_mount_status. Any other
/// path is refused inside dmg::eject.
#[tauri::command]
async fn eject_dmg(mount_point: String) -> Result<(), String> {
    dmg::eject(&mount_point)
}

// Native menus bypass the WebView, so its Japanese→English dictionary cannot
// reach them; the tray labels are resolved here under the same locale rule as
// ui/main.js (docs/SPECIFICATION.md §5.A 表示言語): a system locale starting
// with "ja" selects Japanese, anything else (including unknown) English.
fn is_ja_locale(locale: Option<&str>) -> bool {
    locale.is_some_and(|l| l.to_ascii_lowercase().starts_with("ja"))
}

fn tray_profile_label(profile_name: &str, running: bool, ja: bool) -> String {
    // Only the built-in default profile has a localized display name; a
    // user-given environment name is user data and is never translated.
    let display = if profile_name == "default" {
        if ja {
            "既存の Claude"
        } else {
            "Existing Claude"
        }
    } else {
        profile_name
    };
    if running {
        format!("● {} ({})", display, if ja { "利用中" } else { "In use" })
    } else {
        format!("○ {}", display)
    }
}

fn tray_settings_label(ja: bool) -> &'static str {
    if ja { "設定..." } else { "Settings..." }
}

fn tray_quit_label(ja: bool) -> &'static str {
    if ja { "終了" } else { "Quit" }
}

// Function to update the system tray menu dynamically
fn update_tray_menu(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let profiles = state
        .profile_manager
        .list_profiles()
        .map_err(|e| e.to_string())?;
    // "利用中" reflects which environments' Claude are actually running, resolved
    // per environment from the live processes' --user-data-dir. Fully-isolated
    // environments can run side by side, so more than one may be in use at once.
    // Quitting a Claude clears its marker so that environment can be relaunched.
    let running = running_profile_names(&state);

    let mut menu_items = Vec::new();

    // 1. Title Item
    let title_item =
        MenuItem::with_id(app, "title", "Claude Desktop Switcher", false, None::<&str>)
            .map_err(|e| e.to_string())?;
    menu_items.push(Box::new(title_item) as Box<dyn tauri::menu::IsMenuItem<Wry>>);

    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    menu_items.push(Box::new(sep1) as Box<dyn tauri::menu::IsMenuItem<Wry>>);

    let ja = is_ja_locale(sys_locale::get_locale().as_deref());

    // 2. Add profile switchers
    for p_name in profiles {
        // The settings UI shows the default profile as "既存の Claude" / "Existing
        // Claude" and marks the active one as "利用中" / "In use"; mirror both here
        // so the menu-bar wording matches.
        let label = tray_profile_label(&p_name, running.iter().any(|n| n == &p_name), ja);

        let p_item = MenuItem::with_id(
            app,
            format!("profile_{}", p_name),
            label,
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;

        menu_items.push(Box::new(p_item) as Box<dyn tauri::menu::IsMenuItem<Wry>>);
    }

    // 3. Footer Operations
    let sep2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    menu_items.push(Box::new(sep2) as Box<dyn tauri::menu::IsMenuItem<Wry>>);

    let settings_item =
        MenuItem::with_id(app, "settings", tray_settings_label(ja), true, None::<&str>)
            .map_err(|e| e.to_string())?;
    menu_items.push(Box::new(settings_item) as Box<dyn tauri::menu::IsMenuItem<Wry>>);

    let quit_item = MenuItem::with_id(app, "quit", tray_quit_label(ja), true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu_items.push(Box::new(quit_item) as Box<dyn tauri::menu::IsMenuItem<Wry>>);

    // Reconstruct the menu
    let menu = Menu::with_items(
        app,
        &menu_items
            .iter()
            .map(|item| item.as_ref())
            .collect::<Vec<_>>(),
    )
    .map_err(|e| e.to_string())?;

    if let Some(tray) = app.tray_by_id("main_tray") {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// The app version from tauri.conf.json (kept in sync by release-please), shown in
/// the sidebar footer. Not the crate version, which stays at 0.1.0.
#[tauri::command]
fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Open an https URL in the user's default browser. The footer only ever passes
/// fixed GitHub URLs; we still reject any non-https scheme so the command can never
/// open an arbitrary file or app. CSW itself makes no network requests — this only
/// hands the URL to the OS, which opens the browser.
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err(format!("refused non-https url: {url}"));
    }
    std::process::Command::new("open")
        .arg(&url)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn main() {
    let provider: Arc<dyn csw_core::platform::PlatformProvider> =
        Arc::from(create_provider().expect("Failed to initialize platform provider"));
    let profile_manager = Arc::new(
        ProfileManager::new(provider.clone()).expect("Failed to initialize profile manager"),
    );

    let app_state = AppState {
        provider,
        profile_manager,
    };

    tauri::Builder::default()
        .manage(app_state)
        .on_window_event(|window, event| {
            // Closing the settings window must not destroy it or quit the app:
            // keep running in the tray/Dock and just hide, so it can be reopened.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            // Build the system tray for the first time
            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))
                .expect("Failed to load tray icon");
            let _tray = TrayIconBuilder::with_id("main_tray")
                .icon(icon)
                .icon_as_template(true)
                .tooltip("Claude Desktop Switcher")
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref();
                    if id == "quit" {
                        app.exit(0);
                    } else if id == "settings" {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    } else if let Some(profile_name) = id.strip_prefix("profile_") {
                        let state = app.state::<AppState>();
                        let switcher = ContextSwitcher::new(
                            state.provider.clone(),
                            state.profile_manager.clone(),
                        );

                        if let Err(e) = switcher.switch_to(profile_name) {
                            eprintln!("Failed to switch profile: {}", e);
                        } else {
                            let _ = update_tray_menu(app);
                            // Auto-launch the newly active profile, including the
                            // default ("既存の Claude"): for default this is the
                            // standard data dir, equivalent to a normal launch.
                            if let Ok(profile) = state.profile_manager.get_profile(profile_name) {
                                let _ = csw_core::switcher::desktop::launch_desktop(
                                    &profile,
                                    state.provider.as_ref(),
                                );
                            }
                        }
                    }
                })
                .build(app)?;

            // Initial tray update
            let _ = update_tray_menu(app.handle());

            // Keep the tray's "利用中" marker honest when Claude Desktop is started
            // or quit outside CSW. The menu is otherwise rebuilt only on explicit
            // profile actions, so without this it would show a stale marker (e.g.
            // keep "利用中" after the user quit Claude). Poll the running state and
            // rebuild only when it actually changes.
            let watch_handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut last_running: Option<bool> = None;
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let running = watch_handle
                        .state::<AppState>()
                        .provider
                        .is_claude_desktop_running()
                        .unwrap_or(false);
                    if last_running != Some(running) {
                        last_running = Some(running);
                        let rebuild_handle = watch_handle.clone();
                        let _ = watch_handle.run_on_main_thread(move || {
                            let _ = update_tray_menu(&rebuild_handle);
                        });
                    }
                }
            });

            // Show the settings window on launch so the app is usable even when
            // the menu-bar tray icon is hidden behind a crowded menu bar / notch.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_active_profile,
            list_profiles,
            get_profile_details,
            create_profile,
            delete_profile,
            clone_profile,
            inspect_profile,
            switch_profile,
            launch_additional_window,
            get_desktop_running_status,
            get_running_profiles,
            get_default_roots_status,
            app_version,
            open_url,
            get_dmg_mount_status,
            eject_dmg
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            // Clicking the Dock icon (macOS "reopen") re-shows the settings window.
            if let tauri::RunEvent::Reopen { .. } = event
                && let Some(window) = app_handle.get_webview_window("main")
            {
                let _ = window.show();
                let _ = window.set_focus();
            }
        });
}

// Expectations come from docs/SPECIFICATION.md §5.A 表示言語（i18n）: only a
// system locale starting with "ja" selects Japanese, anything else (including
// an unknown locale) is English; user-given environment names are never
// translated; the English labels reuse the WebView's established vocabulary.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ja_locale_only_when_locale_starts_with_ja() {
        assert!(is_ja_locale(Some("ja-JP")));
        assert!(is_ja_locale(Some("ja")));
        assert!(is_ja_locale(Some("JA-JP")));
        assert!(!is_ja_locale(Some("en-US")));
        assert!(!is_ja_locale(Some("en")));
        assert!(!is_ja_locale(Some("de-DE")));
        assert!(!is_ja_locale(None));
    }

    #[test]
    fn default_profile_label_is_localized() {
        assert_eq!(
            tray_profile_label("default", true, true),
            "● 既存の Claude (利用中)"
        );
        assert_eq!(
            tray_profile_label("default", false, true),
            "○ 既存の Claude"
        );
        assert_eq!(
            tray_profile_label("default", true, false),
            "● Existing Claude (In use)"
        );
        assert_eq!(
            tray_profile_label("default", false, false),
            "○ Existing Claude"
        );
    }

    #[test]
    fn user_environment_names_are_never_translated() {
        assert_eq!(tray_profile_label("work", false, true), "○ work");
        assert_eq!(tray_profile_label("work", false, false), "○ work");
        assert_eq!(tray_profile_label("仕事", true, false), "● 仕事 (In use)");
        assert_eq!(tray_profile_label("仕事", true, true), "● 仕事 (利用中)");
    }

    #[test]
    fn footer_labels_are_localized() {
        assert_eq!(tray_settings_label(true), "設定...");
        assert_eq!(tray_settings_label(false), "Settings...");
        assert_eq!(tray_quit_label(true), "終了");
        assert_eq!(tray_quit_label(false), "Quit");
    }
}
