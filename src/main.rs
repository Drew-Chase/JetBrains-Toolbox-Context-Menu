use crate::arguments::{JetbrainsToolBoxContextArguments, SubCommands};
use anyhow::{Context, Result};
use clap::Parser;
mod arguments;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod darwin;
#[cfg(target_os = "linux")]
#[path = "./linux/dolphin.rs"]
mod dolphin;
mod toolbox_state;
mod update;
#[cfg(target_os = "windows")]
mod windows;
#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "error"); // Just in case the parse fails.

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
        SubCommands::Update(args) => {
            log::debug!("Entered Update subcommand with args: {:?}", args);
            if args.dry_run {
                log::info!("Dry-run flag enabled for Update subcommand");
                if args.all {
                    log::debug!("Fetching all releases");
                    update::print_all_releases(args.style).await?;
                } else {
                    log::debug!("Fetching latest release");
                    update::print_latest_release(args.style).await?;
                }
                return Ok(());
            }
        }
        SubCommands::Scan => {
            log::info!("Entered Scan subcommand");
            scan()?;
        }
        SubCommands::Uninstall => {
            log::info!("Entered Uninstall subcommand");
            #[cfg(target_os = "windows")]
            {
                log::debug!("Attempting to remove existing context menu entries on Windows");
                windows::remove_existing_context_menu()
                    .context("Failed to remove existing context menu entries")?;
            }
        }
    }
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
