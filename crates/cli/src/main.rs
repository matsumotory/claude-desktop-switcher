use std::sync::Arc;
use clap::{Parser, Subcommand};
use colored::*;
use csw_core::platform::create_provider;
use csw_core::profile::{ProfileManager, SharingConfig, SharingMode};
use csw_core::switcher::ContextSwitcher;

#[derive(Parser)]
#[command(name = "csw")]
#[command(about = "Claude Desktop Switcher — Synchronize Desktop & CLI environments")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Claude Desktop Switcher (create ~/.claude-desktop-switcher/)
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
        /// Sharing mode preset: "isolate" (default) or "share"
        #[arg(long, default_value = "isolate")]
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
            println!("{} Base configuration created.", "✔".green());
            println!("{} Profiles directory: {:?}", "✔".green(), provider.app_data_dir().join("profiles"));
            println!("{} Active profile is: {}", "✔".green(), manager.active_profile_name().cyan());
            println!("\nRun '{}' to see available profiles.", "csw profile list".yellow());
        }
        Commands::Profile { action } => {
            let manager = ProfileManager::new(provider.clone())?;
            match action {
                ProfileAction::Create { name, mode } => {
                    println!("Creating profile '{}' with preset '{}'...", name.cyan(), mode.bold());
                    
                    let mut sharing = SharingConfig::default();
                    if mode == "share" {
                        sharing.desktop_config = SharingMode::Share;
                        sharing.cli_settings = SharingMode::Share;
                        sharing.cli_claude_md = SharingMode::Share;
                        sharing.cli_plugins = SharingMode::Share;
                        sharing.desktop_worktrees = SharingMode::Share;
                    } // Else keep defaults (Isolate)
                    
                    match manager.create_profile(&name, sharing, None) {
                        Ok(_) => {
                            println!("{} Profile '{}' created successfully!", "✔".green(), name.cyan());
                            println!("  Desktop Data: {:?}", provider.app_data_dir().join("profiles").join(&name).join("desktop-data"));
                            println!("  CLI Data: {:?}", provider.app_data_dir().join("profiles").join(&name).join("cli-data"));
                        }
                        Err(e) => {
                            eprintln!("{} Failed to create profile: {}", "✘".red(), e);
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
                            println!("  {} {} {}", "*".green().bold(), name.green().bold(), "(active)".green());
                        } else {
                            println!("    {}", name);
                        }
                    }
                }
                ProfileAction::Show { name } => {
                    match manager.get_profile(&name) {
                        Ok(p) => {
                            println!("{}: {}", "Profile".bold(), p.profile.name.cyan());
                            println!("  Icon: {}", p.profile.icon);
                            println!("  Color: {}", p.profile.color);
                            println!("  Is Default: {}", p.profile.is_default);
                            println!("  Desktop Path: {:?}", p.isolation.desktop_user_data_dir);
                            println!("  CLI Path: {:?}", p.isolation.cli_config_dir);
                            println!("  Sharing Configurations:");
                            println!("    Desktop Config (MCP): {:?}", p.sharing.desktop_config);
                            println!("    CLI Settings: {:?}", p.sharing.cli_settings);
                            println!("    CLAUDE.md (Rules): {:?}", p.sharing.cli_claude_md);
                            println!("    CLI Project Memory: {:?}", p.sharing.cli_project_memory);
                            println!("    CLI Plugins: {:?}", p.sharing.cli_plugins);
                            println!("    Desktop Worktrees: {:?}", p.sharing.desktop_worktrees);
                            println!("    Desktop Device ID: {:?}", p.sharing.desktop_device_id);
                        }
                        Err(e) => {
                            eprintln!("{} Failed to find profile '{}': {}", "✘".red(), name, e);
                            std::process::exit(1);
                        }
                    }
                }
                ProfileAction::Delete { name } => {
                    println!("Deleting profile '{}'...", name.cyan());
                    match manager.delete_profile(&name) {
                        Ok(_) => println!("{} Profile '{}' deleted successfully.", "✔".green(), name.cyan()),
                        Err(e) => {
                            eprintln!("{} Failed to delete profile: {}", "✘".red(), e);
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
                    println!("{} Switched successfully.", "✔".green());
                    
                    let profile = manager.get_profile(&name)?;
                    
                    // Shell environment tip
                    println!("\n{}", "To update your terminal context, run:".bold());
                    println!("  {}", format!("eval $(csw env {})", name).yellow());

                    // Launch Desktop if requested and not default/disabled
                    if !no_launch && name != "default" {
                        println!("\nLaunching Claude Desktop for '{}'...", name.cyan());
                        if let Err(e) = csw_core::switcher::desktop::launch_desktop(&profile, provider.as_ref()) {
                            eprintln!("{} Failed to launch Claude Desktop: {}", "⚠".yellow(), e);
                        } else {
                            println!("{} Claude Desktop launched.", "✔".green());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} Switch failed: {}", "✘".red(), e);
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
                    eprintln!("# Error: Failed to generate env for '{}': {}", profile_name, e);
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
            println!("  Desktop User Data Dir: {:?}", profile.isolation.desktop_user_data_dir);
            
            // Check running status of Claude Desktop
            match provider.is_claude_desktop_running() {
                Ok(running) => {
                    if running {
                        let pids = provider.claude_desktop_pids().unwrap_or_default();
                        println!("  Claude Desktop: {} (PIDs: {:?})", "RUNNING".green().bold(), pids);
                    } else {
                        println!("  Claude Desktop: {}", "STOPPED".yellow());
                    }
                }
                Err(e) => {
                    println!("  Claude Desktop: {} (error checking status: {})", "UNKNOWN".red(), e);
                }
            }
        }
    }

    Ok(())
}

