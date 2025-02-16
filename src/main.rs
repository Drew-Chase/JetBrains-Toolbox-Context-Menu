#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::arguments::{JetbrainsToolBoxContextArguments, SubCommands};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use log::error;

mod arguments;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod darwin;
#[cfg(target_os = "linux")]
#[path = "./linux/dolphin.rs"]
mod dolphin;
mod toolbox_state;
// TODO: Add update feature...
//mod update;
#[cfg(target_os = "windows")]
mod windows;
#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "error"); // Just in case the parse fails.
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 || atty::is(atty::Stream::Stderr) {
        #[cfg(target_os = "windows")]
        windows::attach_console();
    }

    let args = JetbrainsToolBoxContextArguments::parse();
    log::debug!("Parsed command line arguments: {:?}", args);

    if args.quiet {
        log::trace!("Quiet mode enabled; setting logging to error level");
        std::env::set_var("RUST_LOG", "error");
    } else if args.verbose {
        log::trace!("Verbose mode enabled; setting logging to trace level");
        std::env::set_var("RUST_LOG", "trace");
    } else {
        log::trace!("Default mode; setting logging to info level");
        std::env::set_var("RUST_LOG", "info");
    }

    env_logger::init();
    log::info!("Logger initialized");

    let subcommand = args.subcommands.unwrap_or(SubCommands::Scan);
    log::info!("Subcommand chosen: {:?}", subcommand);

    match subcommand {
        // TODO: Add update feature...
        //        SubCommands::Update(args) => {
        //            log::debug!("Entered Update subcommand with args: {:?}", args);
        //            if args.dry_run {
        //                log::info!("Dry-run flag enabled for Update subcommand");
        //                if args.all {
        //                    log::debug!("Fetching all releases");
        //                    update::print_all_releases(args.style).await?;
        //                } else {
        //                    log::debug!("Fetching latest release");
        //                    update::print_latest_release(args.style).await?;
        //                }
        //                return Ok(());
        //            }
        //        }
        SubCommands::Scan => {
            log::info!("Entered Scan subcommand");
            scan()?;
            if args.subcommands.is_none() {
                #[cfg(target_os = "windows")]
                windows::show_installed_message();
            }
        }
        SubCommands::Uninstall => {
            log::info!("Entered Uninstall subcommand");
            #[cfg(target_os = "windows")]
            {
                log::debug!("Attempting to remove existing context menu entries on Windows");
                windows::remove_existing_context_menu()
                    .context("Failed to remove existing context menu entries")?;
                let current_exe =
                    std::env::current_exe().context("Failed to get current executable path")?;
                let uninstall_exe = current_exe
                    .parent()
                    .ok_or_else(|| {
                        anyhow!("Failed to locate parent directory for {:?}", current_exe)
                    })?
                    .join("uninstall.exe")
                    .to_string_lossy()
                    .to_string();
                log::debug!("Uninstall executable path: {}", uninstall_exe);
                if !std::path::Path::new(&uninstall_exe).exists() {
                    error!("uninstall.exe not found at {}", uninstall_exe);
                } else {
                    // Start the uninstall process detached from the current process
                    let mut command = std::process::Command::new(&uninstall_exe);
                    command
                        .spawn()
                        .context("Failed to start uninstall process in detached mode")?;
                    log::info!("Uninstall process started");
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    windows::detach_console();
    Ok(())
}
fn scan() -> Result<()> {
    log::debug!("Starting scan function");
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        log::trace!("Calling Darwin specific scan");
        darwin::scan();
    }
    #[cfg(target_os = "windows")]
    {
        log::trace!("Calling Windows scan");
        windows::scan().context("Failed to scan for context menu entries on Windows")?;
    }
    #[cfg(target_os = "linux")]
    {
        log::trace!("Calling Linux scan");
        dolphin::initialize().context("Failed to initialize context menu entries on Linux")?;
    }
    log::debug!("Scan function completed");
    Ok(())
}
