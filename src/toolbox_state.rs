use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Tool {
    pub channel_id: String,
    pub tool_id: String,
    pub product_code: String,
    pub tag: String,
    pub display_name: String,
    pub display_version: String,
    pub build_number: String,
    pub install_location: String,
    pub launch_command: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ToolboxState {
    pub version: i64,
    pub app_version: String,
    pub tools: Vec<Tool>,
}

impl ToolboxState {
    pub(crate) fn from_file(file_path: &str) -> Result<ToolboxState, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let state: ToolboxState = serde_json::from_reader(reader)?;
        Ok(state)
    }
}
