use crate::toolbox_state::{Tool, ToolboxState};
use log::{error, info};
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use winreg::enums::*;
use winreg::RegKey;

pub(crate) fn scan() {
    info!("Initializing Windows specific code");

    match get_jetbrains_toolbox_path() {
        Some(toolbox_path) => {
            let toolbox_executable_path = format!("{}/jetbrains-toolbox.exe", toolbox_path);

            if !std::path::Path::new(&toolbox_executable_path).exists() {
                error!("jetbrains-toolbox.exe not found");
                return;
            }

            let state_file_path = format!("{}/state.json", toolbox_path);
            if !std::path::Path::new(&state_file_path).exists() {
                error!("state.json not found");
                return;
            }

            remove_existing_context_menu();

            match get_tools(&state_file_path) {
                Ok(tools) => {
                    create_context_menu(&toolbox_executable_path, &tools);
                }
                Err(e) => error!("Error retrieving tools: {}", e),
            }

            info!("Done!");
        }
        None => {
            error!("JetBrains Toolbox not found");
        }
    }
}

fn get_jetbrains_toolbox_path() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(toolbox_key) = hkcu.open_subkey("SOFTWARE\\JetBrains\\Toolbox") {
        if let Ok(path) = toolbox_key.get_value::<String, _>("") {
            return Some(path);
        }
    }
    None
}

fn get_tools(state_file_path: &str) -> Result<Vec<Tool>, Box<dyn Error>> {
    let toolstates = ToolboxState::from_file(state_file_path)?;
    let tools: Vec<Tool> = toolstates.tools;
    Ok(tools)
}

fn create_context_menu(toolbox_path: &str, tools: &[Tool]) {
    info!("Creating context menu item for JetBrains Toolbox");

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Ok((key, _)) =
        hkcu.create_subkey("Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox")
    {
        info!("Background Menu");
        key.set_value("MUIVerb", &"Open with JetBrains").unwrap();
        key.set_value("SubCommands", &"").unwrap();
        key.set_value("Icon", &format!("\"{}\"", toolbox_path))
            .unwrap();
        create_context_menu_item(&key, tools);
    }

    if let Ok((key, _)) =
        hkcu.create_subkey("Software\\Classes\\Directory\\shell\\JetBrainsToolbox")
    {
        info!("Directory Menu");
        key.set_value("MUIVerb", &"Open with JetBrains").unwrap();
        key.set_value("SubCommands", &"").unwrap();
        key.set_value("Icon", &format!("\"{}\"", toolbox_path))
            .unwrap();
        create_context_menu_item(&key, tools);
    }
}

pub fn add_self_to_context_menu(){
    let generator_path = std::env::current_exe().unwrap();
    let generator_path = generator_path.to_str().unwrap();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    
    // Add entry for 'update' command
    if let Ok((key, _)) = hkcu.create_subkey("Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox\\shell\\update\\command") {
        key.set_value("", &format!("\"{}\" -q update", generator_path)).unwrap();
    }
    if let Ok((key, _)) = hkcu.create_subkey("Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox\\shell\\update") {
        key.set_value("MUIVerb", &"Update").unwrap();
        key.set_value("Icon", &format!("\"{}\"", generator_path)).unwrap(); // Change to the actual path of the JetBrains Toolbox icon
    }

    // Add entry for 'scan' command
    if let Ok((key, _)) = hkcu.create_subkey("Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox\\shell\\scan\\command") {
        key.set_value("", &format!("\"{}\" -q scan", generator_path)).unwrap();
    }
    if let Ok((key, _)) = hkcu.create_subkey("Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox\\shell\\scan") {
        key.set_value("MUIVerb", &"Refresh").unwrap();
        key.set_value("Icon", &format!("\"{}\"", generator_path)).unwrap(); // Change to the actual path of the JetBrains Toolbox icon
    }
}

fn create_context_menu_item(key: &RegKey, tools: &[Tool]) {
    if let Ok((shell_key, _)) = key.create_subkey("shell") {
        for tool in tools {
            let exe_path = format!("{}/{}", tool.install_location, tool.launch_command);
            if let Ok((sub_key, _)) = shell_key.create_subkey(&tool.display_name) {
                sub_key.set_value("MUIVerb", &tool.display_name).unwrap();
                sub_key
                    .set_value("Icon", &format!("\"{}\"", exe_path))
                    .unwrap();
            }

            if let Ok((sub_key, _)) =
                shell_key.create_subkey(format!("{}\\command", tool.display_name))
            {
                sub_key
                    .set_value("", &format!("\"{}\" \"%V\"", exe_path))
                    .unwrap();
                info!("\t- {}", tool.display_name);
            }
        }
    }
}

pub fn remove_existing_context_menu() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let _ =
        hkcu.delete_subkey_all("Software\\Classes\\Directory\\Background\\shell\\JetBrainsToolbox");
    let _ = hkcu.delete_subkey_all("Software\\Classes\\Directory\\shell\\JetBrainsToolbox");
}
