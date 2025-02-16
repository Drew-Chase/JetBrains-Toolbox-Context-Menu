use crate::toolbox_state::{Tool, ToolboxState};
use anyhow::{anyhow, bail, Context, Result};
use log::*;
use winreg::enums::*;
use winreg::RegKey;

pub(crate) fn scan() -> Result<()> {
    debug!("Starting Windows registry scan");
    info!("Initializing Windows specific code");
    match get_jetbrains_toolbox_path() {
        Some(toolbox_path) => {
            debug!("Found Toolbox path: {}", toolbox_path);
            let toolbox_executable_path = std::path::Path::new(&toolbox_path)
                .join("jetbrains-toolbox.exe")
                .to_string_lossy()
                .to_string();

            if !std::path::Path::new(&toolbox_executable_path).exists() {
                error!(
                    "jetbrains-toolbox.exe not found at {}",
                    toolbox_executable_path
                );
                return Ok(());
            }
            debug!("Found Toolbox executable at {}", toolbox_executable_path);

            let state_file_path = std::path::Path::new(&toolbox_path)
                .parent()
                .ok_or_else(|| anyhow!("Failed to locate parent directory for {}", toolbox_path))?
                .join("state.json")
                .display()
                .to_string();
            if !std::path::Path::new(&state_file_path).exists() {
                error!("state.json not found at {}", state_file_path);
                bail!("state.json not found at {}", state_file_path);
            }
            debug!("Found state file at {}", state_file_path);

            debug!("Removing existing context menu entries");
            remove_existing_context_menu()?;

            match get_tools(&state_file_path) {
                Ok(tools) => {
                    debug!("Found {} tools in state file", tools.len());
                    create_context_menu(&toolbox_executable_path, &tools)?;
                }
                Err(e) => error!("Error retrieving tools: {}", e),
            }
            add_self_to_context_menu()?;
            info!("Done!");
        }
        None => {
            error!("JetBrains Toolbox not found in registry");
            bail!("JetBrains Toolbox not found in registry");
        }
    }
    Ok(())
}

fn get_jetbrains_toolbox_path() -> Option<String> {
    debug!("Attempting to read Toolbox path from registry");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(toolbox_key) = hkcu.open_subkey("SOFTWARE\\JetBrains\\Toolbox") {
        if let Ok(path) = toolbox_key.get_value::<String, _>("") {
            debug!("Successfully read Toolbox path from registry: {}", path);
            return Some(path);
        }
    }
    warn!("Failed to read Toolbox path from registry");
    None
}

fn get_tools(state_file_path: &str) -> Result<Vec<Tool>> {
    debug!("Parsing state file at {}", state_file_path);
    let tool_states =
        ToolboxState::from_file(state_file_path).context("Failed to parse state.json")?;
    debug!("Successfully parsed {} tools", tool_states.tools.len());
    Ok(tool_states.tools)
}

fn create_context_menu(toolbox_path: &str, tools: &[Tool]) -> Result<()> {
    info!("Creating context menu items for JetBrains Toolbox");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Ok((key, _)) =
        hkcu.create_subkey("Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox")
    {
        debug!("Creating background menu entries");
        key.set_value("MUIVerb", &"Open with JetBrains")
            .context("Failed to set MUIVerb value")?;
        key.set_value("SubCommands", &"")
            .context("Failed to set SubCommands value")?;
        key.set_value("Icon", &format!("\"{}\"", toolbox_path))
            .context("Failed to set Icon value")?;
        create_context_menu_item(&key, tools)?;
    }

    if let Ok((key, _)) =
        hkcu.create_subkey("Software\\Classes\\Directory\\shell\\JetBrainsToolbox")
    {
        debug!("Creating directory menu entries");
        key.set_value("MUIVerb", &"Open with JetBrains")
            .context("Failed to set MUIVerb value")?;
        key.set_value("SubCommands", &"")
            .context("Failed to set SubCommands value")?;
        key.set_value("Icon", &format!("\"{}\"", toolbox_path))
            .context("Failed to set Icon value")?;
        create_context_menu_item(&key, tools)?;
    }
    Ok(())
}

pub fn add_self_to_context_menu() -> Result<()> {
    debug!("Adding self to context menu");
    let generator_path =
        std::env::current_exe().context("Failed to get current executable path")?;
    let generator_path = generator_path
        .to_str()
        .context("Failed to convert path to string")?;
    debug!("Generator path: {}", generator_path);

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // TODO: Add update feature...
    // Add entry for 'update' command
    //    if let Ok((key, _)) = hkcu.create_subkey(
    //        "Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox\\shell\\update\\command",
    //    ) {
    //        debug!("Creating update command entry");
    //        key.set_value("", &format!("\"{}\" -q update", generator_path))
    //            .context("Failed to set update command")?;
    //    }
    //    if let Ok((key, _)) = hkcu.create_subkey(
    //        "Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox\\shell\\update",
    //    ) {
    //        key.set_value("MUIVerb", &"Update")
    //            .context("Failed to set Update MUIVerb")?;
    //        key.set_value("Icon", &format!("\"{}\"", generator_path))
    //            .context("Failed to set Update Icon")?;
    //    }

    // Add entry for 'scan' command
    if let Ok((key, _)) = hkcu.create_subkey(
        "Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox\\shell\\scan\\command",
    ) {
        debug!("Creating scan command entry");
        key.set_value("", &format!("\"{}\" -q scan", generator_path))
            .context("Failed to set scan command")?;
    }
    if let Ok((key, _)) = hkcu.create_subkey(
        "Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox\\shell\\scan",
    ) {
        key.set_value("MUIVerb", &"Refresh")
            .context("Failed to set Refresh MUIVerb")?;
        key.set_value("Icon", &format!("\"{}\"", generator_path))
            .context("Failed to set Refresh Icon")?;
    }
    
    // Add entry for 'uninstall' command
    if let Ok((key, _)) = hkcu.create_subkey(
        "Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox\\shell\\uninstall\\command",
    ) {
        debug!("Creating uninstall command entry");
        key.set_value("", &format!("\"{}\" -q uninstall", generator_path))
            .context("Failed to set uninstall command")?;
    }
    if let Ok((key, _)) = hkcu.create_subkey(
        "Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox\\shell\\uninstall",
    ) {
        key.set_value("MUIVerb", &"Uninstall")
            .context("Failed to set Uninstall MUIVerb")?;
        key.set_value("Icon", &format!("\"{}\"", generator_path))
            .context("Failed to set Uninstall Icon")?;
    }
    info!("Successfully added self to context menu");
    Ok(())
}

fn create_context_menu_item(key: &RegKey, tools: &[Tool]) -> Result<()> {
    debug!("Creating context menu items for {} tools", tools.len());
    if let Ok((shell_key, _)) = key.create_subkey("shell") {
        for tool in tools {
            let exe_path = format!("{}/{}", tool.install_location, tool.launch_command);
            debug!(
                "Creating menu item for {} at {}",
                tool.display_name, exe_path
            );

            if let Ok((sub_key, _)) = shell_key.create_subkey(&tool.display_name) {
                sub_key
                    .set_value("MUIVerb", &tool.display_name)
                    .context(format!("Failed to set MUIVerb for {}", tool.display_name))?;
                sub_key
                    .set_value("Icon", &format!("\"{}\"", exe_path))
                    .context(format!("Failed to set Icon for {}", tool.display_name))?;
            }
            if let Ok((sub_key, _)) =
                shell_key.create_subkey(format!("{}\\command", tool.display_name))
            {
                sub_key
                    .set_value("", &format!("\"{}\" \"%V\"", exe_path))
                    .context(format!("Failed to set command for {}", tool.display_name))?;
                info!("\t- Added context menu item for {}", tool.display_name);
            }
        }
    }
    Ok(())
}

pub fn remove_existing_context_menu() -> Result<()> {
    debug!("Removing existing context menu entries");
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if hkcu
        .delete_subkey_all("Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox")
        .is_err()
    {
        debug!("Background context menu subkey does not exist or could not be deleted");
    }
    if hkcu
        .delete_subkey_all("Software\\Classes\\Directory\\shell\\JetBrainsToolbox")
        .is_err()
    {
        debug!("Directory context menu subkey does not exist or could not be deleted");
    }
    debug!("Successfully removed existing context menu entries");
    Ok(())
}
