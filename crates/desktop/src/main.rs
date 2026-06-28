#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Arc;
use tauri::{
    AppHandle, Manager, State, Wry,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

use csw_core::platform::create_provider;
use csw_core::profile::{ProfileManager, SharingConfig, SharingMode};
use csw_core::switcher::ContextSwitcher;

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
        "sharing": {
            "desktop_config": format!("{:?}", p.sharing.desktop_config).to_lowercase(),
            "desktop_app_config": format!("{:?}", p.sharing.desktop_app_config).to_lowercase(),
            "cli_settings": format!("{:?}", p.sharing.cli_settings).to_lowercase(),
            "cli_claude_md": format!("{:?}", p.sharing.cli_claude_md).to_lowercase(),
            "cli_project_memory": format!("{:?}", p.sharing.cli_project_memory).to_lowercase(),
            "cli_plugins": format!("{:?}", p.sharing.cli_plugins).to_lowercase(),
            "cli_skills": format!("{:?}", p.sharing.cli_skills).to_lowercase(),
            "cli_sessions": format!("{:?}", p.sharing.cli_sessions).to_lowercase(),
            "cli_history": format!("{:?}", p.sharing.cli_history).to_lowercase(),
            "desktop_worktrees": format!("{:?}", p.sharing.desktop_worktrees).to_lowercase(),
            "desktop_device_id": format!("{:?}", p.sharing.desktop_device_id).to_lowercase()
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
    // Two-mode model surfaced in the UI:
    //   "isolate"        — a brand-new, empty environment (= SharingConfig::default()).
    //   "share_settings" — reuse the settings assets (MCP servers, global rules, skills,
    //                      plugins, app config, worktrees) from the default profile via
    //                      symlink, while keeping the account login, conversation sessions,
    //                      command history and project memory isolated for safety.
    // "share" is kept as a backward-compatible alias for "share_settings".
    let mut sharing = SharingConfig::default();
    if mode == "share_settings" || mode == "share" {
        sharing.desktop_config = SharingMode::Share; // MCP servers
        sharing.cli_settings = SharingMode::Share; // permissions / hooks
        sharing.cli_claude_md = SharingMode::Share; // global rules
        sharing.cli_plugins = SharingMode::Share;
        sharing.cli_skills = SharingMode::Share;
        sharing.desktop_app_config = SharingMode::Share;
        sharing.desktop_worktrees = SharingMode::Share;
        // cli_project_memory / cli_sessions / cli_history stay Isolate (safety side).
        // desktop_device_id is already Share in SharingConfig::default().
    }

    // Advanced settings: explicit per-component choices override the mode preset.
    if let Some(overrides) = overrides {
        for (key, value) in overrides {
            let Some(m) = parse_sharing_mode(&value) else {
                continue;
            };
            match key.as_str() {
                "desktop_config" => sharing.desktop_config = m,
                "cli_settings" => sharing.cli_settings = m,
                "cli_claude_md" => sharing.cli_claude_md = m,
                "cli_project_memory" => sharing.cli_project_memory = m,
                "cli_plugins" => sharing.cli_plugins = m,
                "cli_skills" => sharing.cli_skills = m,
                "cli_sessions" => sharing.cli_sessions = m,
                "cli_history" => sharing.cli_history = m,
                "desktop_app_config" => sharing.desktop_app_config = m,
                "desktop_worktrees" => sharing.desktop_worktrees = m,
                "desktop_device_id" => sharing.desktop_device_id = m,
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

    if !no_launch && name != "default" {
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

// Function to update the system tray menu dynamically
fn update_tray_menu(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let profiles = state
        .profile_manager
        .list_profiles()
        .map_err(|e| e.to_string())?;
    let active_name = state.profile_manager.active_profile_name();

    let mut menu_items = Vec::new();

    // 1. Title Item
    let title_item =
        MenuItem::with_id(app, "title", "Claude Desktop Switcher", false, None::<&str>)
            .map_err(|e| e.to_string())?;
    menu_items.push(Box::new(title_item) as Box<dyn tauri::menu::IsMenuItem<Wry>>);

    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    menu_items.push(Box::new(sep1) as Box<dyn tauri::menu::IsMenuItem<Wry>>);

    // 2. Add profile switchers
    for p_name in profiles {
        // The settings UI shows the default profile as "既存の Claude" and marks the
        // active one as "利用中"; mirror both here so the menu-bar wording matches.
        let display: &str = if p_name == "default" {
            "既存の Claude"
        } else {
            p_name.as_str()
        };
        let label = if p_name == active_name {
            format!("● {} (利用中)", display)
        } else {
            format!("○ {}", display)
        };

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

    let settings_item = MenuItem::with_id(app, "settings", "設定...", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu_items.push(Box::new(settings_item) as Box<dyn tauri::menu::IsMenuItem<Wry>>);

    let quit_item =
        MenuItem::with_id(app, "quit", "終了", true, None::<&str>).map_err(|e| e.to_string())?;
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
                            // Auto-launch if switched (except to default)
                            if profile_name != "default"
                                && let Ok(profile) = state.profile_manager.get_profile(profile_name)
                            {
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
            switch_profile,
            get_desktop_running_status
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
