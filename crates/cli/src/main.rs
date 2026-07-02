use clap::{Parser, Subcommand};
use colored::*;
use csw_core::platform::create_provider;
use csw_core::profile::{ProfileManager, SharingConfig};
use csw_core::switcher::ContextSwitcher;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "csw")]
#[command(about = "Claude Desktop Switcher: synchronize Desktop & CLI environments")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Claude Desktop Switcher (create ~/.context-switcher-claude/)
    Init,

    /// Manage profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Switch to a profile (launch Desktop + set CLI env)
    Switch {
        /// Profile name to switch to
        name: String,

        /// Do not launch Claude Desktop after switching
        #[arg(long)]
        no_launch: bool,
    },

    /// Output shell environment variables for a profile
    Env {
        /// Profile name (defaults to active profile if omitted)
        name: Option<String>,
    },

    /// Show current status (active profile, running processes)
    Status,

    /// Check that each environment's isolation and share links are intact.
    /// Unrelated to Claude Code's own `claude doctor` (install diagnostics):
    /// this inspects the environments CSW created. Read-only unless --fix.
    Doctor {
        /// Environment name (checks all environments if omitted)
        name: Option<String>,

        /// Re-point share links that no longer resolve to their existing
        /// expected source. Only symlinks are swapped; real files are never
        /// touched (drifted real copies are reported, not repaired).
        #[arg(long)]
        fix: bool,
    },
}

#[derive(Subcommand)]
enum ProfileAction {
    /// Create a new profile
    Create {
        /// Profile name
        name: String,
        /// Sharing mode: "isolate" (default; すべて分ける), "share_settings"
        /// (会話とメモリも分ける), or "share_workspace" (アカウントだけ分ける).
        /// "share" is a deprecated alias for "share_settings".
        #[arg(
            long,
            default_value = "isolate",
            value_parser = ["isolate", "share_settings", "share_workspace", "share"]
        )]
        mode: String,
    },
    /// List all profiles
    List,
    /// Show profile details
    Show {
        /// Profile name
        name: String,
    },
    /// Delete a profile (moves its folder to the Trash; restorable until the Trash is emptied)
    Delete {
        /// Profile name
        name: String,

        /// Delete permanently instead of moving to the Trash (not restorable)
        #[arg(long)]
        purge: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Create platform provider
    let provider: Arc<dyn csw_core::platform::PlatformProvider> = Arc::from(create_provider()?);

    match cli.command {
        Commands::Init => {
            println!("{}", "Initializing Claude Desktop Switcher...".bold());
            let manager = ProfileManager::new(provider.clone())?;
            println!("{} Base configuration created.", "OK".green());
            println!(
                "{} Profiles directory: {:?}",
                "OK".green(),
                provider.app_data_dir().join("profiles")
            );
            println!(
                "{} Active profile is: {}",
                "OK".green(),
                manager.active_profile_name().cyan()
            );
            println!(
                "\nRun '{}' to see available profiles.",
                "csw profile list".yellow()
            );
        }
        Commands::Profile { action } => {
            let manager = ProfileManager::new(provider.clone())?;
            match action {
                ProfileAction::Create { name, mode } => {
                    println!(
                        "Creating profile '{}' with preset '{}'...",
                        name.cyan(),
                        mode.bold()
                    );

                    // Mirror the GUI's three modes (build_sharing_config in the desktop
                    // crate). "share" is a backward-compatible alias for "share_settings".
                    // The account-keyed files (config.json, claude_desktop_config.json),
                    // the device id and sessions/ are not SharingConfig fields, so no
                    // preset can express sharing them. The linker isolates them
                    // unconditionally.
                    let sharing = match mode.as_str() {
                        "share_settings" | "share" => SharingConfig::share_settings_preset(),
                        "share_workspace" => SharingConfig::share_workspace_preset(),
                        _ => SharingConfig::default(), // "isolate"
                    };

                    match manager.create_profile(&name, sharing, None) {
                        Ok(_) => {
                            println!(
                                "{} Profile '{}' created successfully!",
                                "OK".green(),
                                name.cyan()
                            );
                            println!(
                                "  Desktop Data: {:?}",
                                provider
                                    .app_data_dir()
                                    .join("profiles")
                                    .join(&name)
                                    .join("desktop-data")
                            );
                            println!(
                                "  CLI Data: {:?}",
                                provider
                                    .app_data_dir()
                                    .join("profiles")
                                    .join(&name)
                                    .join("cli-data")
                            );
                        }
                        Err(e) => {
                            eprintln!("{} Failed to create profile: {}", "Error:".red(), e);
                            std::process::exit(1);
                        }
                    }
                }
                ProfileAction::List => {
                    let profiles = manager.list_profiles()?;
                    let active = manager.active_profile_name();
                    println!("{}", "Available profiles:".bold());
                    for name in profiles {
                        if name == active {
                            println!(
                                "  {} {} {}",
                                "*".green().bold(),
                                name.green().bold(),
                                "(active)".green()
                            );
                        } else {
                            println!("    {}", name);
                        }
                    }
                }
                ProfileAction::Show { name } => match manager.get_profile(&name) {
                    Ok(p) => {
                        println!("{}: {}", "Profile".bold(), p.profile.name.cyan());
                        println!("  Icon: {}", p.profile.icon);
                        println!("  Color: {}", p.profile.color);
                        println!("  Is Default: {}", p.profile.is_default);
                        println!("  Desktop Path: {:?}", p.isolation.desktop_user_data_dir);
                        println!("  CLI Path: {:?}", p.isolation.cli_config_dir);
                        println!("  Sharing Configurations:");
                        println!("    CLI Settings: {:?}", p.sharing.cli_settings);
                        println!("    CLAUDE.md (Rules): {:?}", p.sharing.cli_claude_md);
                        println!("    CLI Plugins: {:?}", p.sharing.cli_plugins);
                        println!("    CLI Skills: {:?}", p.sharing.cli_skills);
                        println!("    CLI Project Memory: {:?}", p.sharing.cli_project_memory);
                        println!("    CLI Input History: {:?}", p.sharing.cli_history);
                        println!("    Desktop Worktrees: {:?}", p.sharing.desktop_worktrees);
                        println!(
                            "    Always isolated: config.json, claude_desktop_config.json, sessions/, device id"
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "{} Failed to find profile '{}': {}",
                            "Error:".red(),
                            name,
                            e
                        );
                        std::process::exit(1);
                    }
                },
                ProfileAction::Delete { name, purge } => {
                    if purge {
                        println!("Permanently deleting profile '{}'...", name.cyan());
                        match manager.purge_profile(&name) {
                            Ok(_) => println!(
                                "{} Profile '{}' permanently deleted (not restorable).",
                                "OK".green(),
                                name.cyan()
                            ),
                            Err(e) => {
                                eprintln!("{} Failed to delete profile: {}", "Error:".red(), e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        println!("Moving profile '{}' to the Trash...", name.cyan());
                        match manager.delete_profile(&name) {
                            Ok(_) => {
                                println!(
                                    "{} Profile '{}' moved to the Trash.",
                                    "OK".green(),
                                    name.cyan()
                                );
                                println!(
                                    "  Until you empty the Trash, everything (including the \
sign-in state) can be restored by moving the folder back under profiles/."
                                );
                            }
                            // Advise --purge only when the Trash move itself
                            // failed; validation errors (default/active) would
                            // fail --purge identically, so the hint would be
                            // wrong and, for the default, dangerous.
                            Err(csw_core::error::CswError::TrashMoveFailed(e)) => {
                                eprintln!(
                                    "{} Could not move the profile to the Trash: {}",
                                    "Error:".red(),
                                    e
                                );
                                eprintln!(
                                    "  To delete it permanently instead, run '{}'.",
                                    format!("csw profile delete {name} --purge").yellow()
                                );
                                std::process::exit(1);
                            }
                            Err(e) => {
                                eprintln!("{} Failed to delete profile: {}", "Error:".red(), e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
            }
        }
        Commands::Switch { name, no_launch } => {
            let manager = Arc::new(ProfileManager::new(provider.clone())?);
            let switcher = ContextSwitcher::new(provider.clone(), manager.clone());

            println!("Switching to profile '{}'...", name.cyan());
            match switcher.switch_to(&name) {
                Ok(_) => {
                    println!("{} Switched successfully.", "OK".green());

                    let profile = manager.get_profile(&name)?;

                    // Shell environment tip
                    println!("\n{}", "To update your terminal context, run:".bold());
                    println!("  {}", format!("eval $(csw env {})", name).yellow());

                    // Launch Desktop if requested and not default/disabled
                    if !no_launch && name != "default" {
                        println!("\nLaunching Claude Desktop for '{}'...", name.cyan());
                        if let Err(e) =
                            csw_core::switcher::desktop::launch_desktop(&profile, provider.as_ref())
                        {
                            eprintln!(
                                "{} Failed to launch Claude Desktop: {}",
                                "Warning:".yellow(),
                                e
                            );
                        } else {
                            println!("{} Claude Desktop launched.", "OK".green());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} Switch failed: {}", "Error:".red(), e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Env { name } => {
            let manager = ProfileManager::new(provider.clone())?;
            let profile_name = name.unwrap_or_else(|| manager.active_profile_name());
            match manager.get_profile(&profile_name) {
                Ok(profile) => {
                    let script = csw_core::switcher::shell::generate_env_script(&profile);
                    print!("{}", script);
                }
                Err(e) => {
                    eprintln!(
                        "# Error: Failed to generate env for '{}': {}",
                        profile_name, e
                    );
                    std::process::exit(1);
                }
            }
        }
        Commands::Status => {
            let manager = ProfileManager::new(provider.clone())?;
            let active_name = manager.active_profile_name();
            let profile = manager.get_profile(&active_name)?;

            println!("{}: {}", "Active Profile".bold(), active_name.cyan().bold());
            println!("  CLI Config Dir: {:?}", profile.isolation.cli_config_dir);
            println!(
                "  Desktop User Data Dir: {:?}",
                profile.isolation.desktop_user_data_dir
            );

            // Check running status of Claude Desktop
            match provider.is_claude_desktop_running() {
                Ok(running) => {
                    if running {
                        let pids = provider.claude_desktop_pids().unwrap_or_default();
                        println!(
                            "  Claude Desktop: {} (PIDs: {:?})",
                            "RUNNING".green().bold(),
                            pids
                        );
                    } else {
                        println!("  Claude Desktop: {}", "STOPPED".yellow());
                    }
                }
                Err(e) => {
                    println!(
                        "  Claude Desktop: {} (error checking status: {})",
                        "UNKNOWN".red(),
                        e
                    );
                }
            }
        }
        Commands::Doctor { name, fix } => {
            let manager = ProfileManager::new(provider.clone())?;
            let names: Vec<String> = match name {
                Some(n) => vec![n],
                None => manager
                    .list_profiles()?
                    .into_iter()
                    .filter(|n| n != "default")
                    .collect(),
            };
            if names.is_empty() {
                println!("No environments to check. The existing Claude has no links by design.");
                return Ok(());
            }

            let mut total_issues = 0usize;
            for env_name in &names {
                if env_name == "default" {
                    println!(
                        "{} the existing Claude has no links to inspect.",
                        "SKIP".yellow()
                    );
                    continue;
                }

                let mut report = manager.inspect_profile_isolation(env_name)?;
                if fix && report.items.iter().any(is_fixable) {
                    let fixed = manager.doctor_fix_links(env_name)?;
                    for key in &fixed {
                        println!(
                            "{} re-pointed the share link for {}",
                            "FIXED".green(),
                            doctor_label(key).cyan()
                        );
                    }
                    report = manager.inspect_profile_isolation(env_name)?;
                }

                print_doctor_report(&report);
                total_issues += report.issue_count;
            }

            if total_issues > 0 {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn is_fixable(item: &csw_core::profile::inspector::ItemReport) -> bool {
    matches!(
        item.health,
        csw_core::profile::inspector::ItemHealth::WrongTarget { fixable: true, .. }
    )
}

/// Human label for a link-point key. English terms match the GUI's EN
/// dictionary (crates/desktop/ui/main.js) so both surfaces report identically.
fn doctor_label(key: &str) -> &'static str {
    match key {
        "cli_claude_md" => "Global rules (CLAUDE.md)",
        "cli_settings" => "Tool permissions & hooks (settings.json)",
        "cli_project_memory" => "Project conversations & memory (projects/)",
        "cli_plugins" => "Plugins (plugins/)",
        "cli_skills" => "Skills (skills/)",
        "cli_sessions" => "Session state (sessions/)",
        "cli_history" => "Input history (history.jsonl)",
        "desktop_worktrees" => "Worktrees (git-worktrees.json)",
        "desktop_device_id" => "Device ID (ant-did)",
        "desktop_app_config" => "Account sign-in (config.json)",
        "desktop_config" => "Connectors & app settings (claude_desktop_config.json)",
        _ => "Unknown item",
    }
}

/// Whether a link point is structurally isolated in every mode
/// (SPECIFICATION.md §3「常に分離する項目」).
fn is_always_isolated(key: &str) -> bool {
    csw_core::profile::linker::LINK_ITEMS
        .iter()
        .any(|i| i.key == key && i.fixed_mode.is_some())
}

fn print_doctor_report(report: &csw_core::profile::inspector::ProfileReport) {
    use csw_core::profile::inspector::ItemHealth;

    println!("{}: {}", "Environment".bold(), report.profile.cyan().bold());
    if report.running {
        println!(
            "  {} The Claude Desktop App is running in this environment. A rewrite may \
have been caught mid-flight; if issues appear, quit Claude and re-check.",
            "NOTE".yellow()
        );
    }

    for item in &report.items {
        let label = doctor_label(item.key);
        match &item.health {
            ItemHealth::SharedOk { target } => {
                println!("  {} {} shared -> {}", "OK".green(), label, target);
            }
            ItemHealth::IsolatedOk => {
                // Copy and Isolate are distinct modes; reflect the declared one.
                let word = match item.mode {
                    csw_core::profile::SharingMode::Copy => "independent copy",
                    _ => "isolated",
                };
                println!("  {} {} {}", "OK".green(), label, word);
            }
            ItemHealth::SourceAbsent => {
                println!(
                    "  {} {} share declared; nothing to share yet",
                    "OK".green(),
                    label
                );
            }
            ItemHealth::SourceMissing { expected_source } => {
                println!(
                    "  {} {} the shared source is missing: {}",
                    "!!".red().bold(),
                    label,
                    expected_source
                );
            }
            ItemHealth::WrongTarget {
                expected_source,
                actual_target,
                fixable,
            } => {
                println!(
                    "  {} {} link points to {}, expected {}{}",
                    "!!".red().bold(),
                    label,
                    actual_target,
                    expected_source,
                    if *fixable {
                        " (run `csw doctor --fix` to re-point it)"
                    } else {
                        ""
                    }
                );
            }
            ItemHealth::Materialized => {
                println!(
                    "  {} {} expected a shared link but found a real copy (drifted). \
Not auto-fixed to avoid losing the local copy.",
                    "!!".red().bold(),
                    label
                );
            }
            ItemHealth::MissingLink { expected_source } => {
                println!(
                    "  {} {} share link missing (expected -> {})",
                    "!!".red().bold(),
                    label,
                    expected_source
                );
            }
            ItemHealth::UnexpectedLink { actual_target } => {
                // Name the declared mode accurately: 常に分離する項目 is a fixed
                // set (SPECIFICATION.md §3), not every isolated/copied item.
                let declared = if is_always_isolated(item.key) {
                    "must always stay isolated"
                } else if matches!(item.mode, csw_core::profile::SharingMode::Copy) {
                    "is declared as copy"
                } else {
                    "is declared isolated"
                };
                println!(
                    "  {} {} {} but is a link to {}",
                    "!!".red().bold(),
                    label,
                    declared,
                    actual_target
                );
            }
        }
    }

    if report.issue_count == 0 {
        println!("  {} no issues found", "OK".green().bold());
    } else if report.issue_count == 1 {
        println!("  {} 1 issue found", "!!".red().bold());
    } else {
        println!(
            "  {} {} issues found",
            "!!".red().bold(),
            report.issue_count
        );
    }
}
