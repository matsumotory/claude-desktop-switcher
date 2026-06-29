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
    /// Delete a profile
    Delete {
        /// Profile name
        name: String,
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
                ProfileAction::Delete { name } => {
                    println!("Deleting profile '{}'...", name.cyan());
                    match manager.delete_profile(&name) {
                        Ok(_) => println!(
                            "{} Profile '{}' deleted successfully.",
                            "OK".green(),
                            name.cyan()
                        ),
                        Err(e) => {
                            eprintln!("{} Failed to delete profile: {}", "Error:".red(), e);
                            std::process::exit(1);
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
    }

    Ok(())
}
