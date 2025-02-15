use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Tool {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "installLocation")]
    pub install_location: String,
    #[serde(rename = "launchCommand")]
    pub launch_command: String,
}

#[derive(Deserialize)]
pub(crate) struct ToolboxState {
    pub tools: Vec<Tool>,
}

impl ToolboxState {
    pub(crate) fn from_file(file_path: &str) -> Result<ToolboxState> {
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let state: ToolboxState = serde_json::from_reader(reader)?;
        Ok(state)
    }
}
