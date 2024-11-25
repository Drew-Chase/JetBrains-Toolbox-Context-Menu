use crate::arguments::{JetbrainsToolBoxContextArguments, SubCommands};
use clap::Parser;
use log::{debug, error};
use std::fs;
use std::path::Path;
use std::process::exit;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod darwin;

#[cfg(target_os = "windows")]
mod windows;

mod arguments;
#[cfg(target_os = "linux")]
#[path = "./linux/dolphin.rs"]
mod dolphin;
mod toolbox_state;
mod update;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "error"); // Just in case the parse fails.
    let args = JetbrainsToolBoxContextArguments::parse();

    if args.quiet {
        std::env::set_var("RUST_LOG", "error");
    } else if args.verbose {
        std::env::set_var("RUST_LOG", "trace");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    let subcommand = args.subcommands.unwrap_or(SubCommands::Scan);
    match subcommand {
        SubCommands::Update(args) => {
            if args.dry_run {
                if args.all {
                    update::print_all_releases(args.style).await;
                } else {
                    update::print_latest_release(args.style).await;
                }
                return;
            }
        }
        SubCommands::Scan => {
            scan();
        }
        SubCommands::Uninstall => {
            #[cfg(target_os = "windows")]
            windows::remove_existing_context_menu();
        }
        SubCommands::Install(args) => {
            let current_exe = std::env::current_exe().unwrap();

            #[cfg(target_os = "windows")]
            let installed_exe = Path::new(&args.path).join("JetBrainsToolboxContext.exe");
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            let installed_exe = Path::new(&args.path).join("JetBrainsToolboxContext");

            if let Err(e) = fs::copy(current_exe, &installed_exe) {
                error!("Unable to install to path: {:?} - {}", installed_exe, e);
                exit(1);
            }

            #[cfg(target_os = "windows")]
            windows::add_self_to_context_menu();

            scan();

            exit(1);
        }
    }
}

fn scan() {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    darwin::scan();

    #[cfg(target_os = "windows")]
    windows::scan();

    #[cfg(target_os = "linux")]
    dolphin::initialize();
}
