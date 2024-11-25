use crate::arguments::FormatOptions;
use octocrate::{repos::list_releases::Response, APIConfig, Error, GitHubAPI};
use std::process::exit;

pub async fn get_list_of_releases() -> Result<Response, Error> {
    let config = APIConfig::default().shared();
    let api = GitHubAPI::new(&config);

    api.repos
        .list_releases("drew-chase", "JetBrains-Toolbox-Context-Menu")
        .send()
        .await
}
pub async fn get_latest_release() -> Result<octocrate::repos::get_latest_release::Response, Error> {
    let config = APIConfig::default().shared();
    let api = GitHubAPI::new(&config);
    api.repos
        .get_latest_release("drew-chase", "JetBrains-Toolbox-Context-Menu")
        .send()
        .await
}

pub async fn print_latest_release(format_options: FormatOptions) {
    let release = match get_latest_release().await {
        Ok(release) => release,
        Err(e) => {
            log::error!("Error getting release information: {}", e);
            exit(1);
        }
    };
    let formatted_list = match format_options {
        FormatOptions::PlainText => {
            let tag_name = release.tag_name.clone();
            let body_text = release
                .body
                .clone()
                .unwrap_or("No changelog provided".to_string());
            let assets_name_url = release
                .assets
                .iter()
                .map(|asset| (asset.name.clone(), asset.browser_download_url.clone()))
                .collect::<Vec<(String, String)>>()
                .iter()
                .map(|(name, url)| format!("{}: {}", name, url))
                .collect::<Vec<String>>()
                .join("\n");
            format!(
                "Version: {}\nChangelog:\n{}\nDownload URLs:\n{}",
                tag_name, body_text, assets_name_url
            )
        }
        FormatOptions::Json => serde_json::to_string(&release).unwrap_or_else(|e| {
            log::error!("Failed to format release as JSON: {}", e);
            String::from("Error formatting as JSON")
        }),
        FormatOptions::Yaml => serde_yaml::to_string(&release).unwrap_or_else(|e| {
            log::error!("Failed to format release as YAML: {}", e);
            String::from("Error formatting as YAML")
        }),
    };

    println!("{}", formatted_list);
}

pub(crate) async fn print_all_releases(format_options: FormatOptions) {
    let releases = match get_list_of_releases().await {
        Ok(releases) => releases,
        Err(e) => {
            log::error!("Error getting release information: {}", e);
            exit(1);
        }
    };

    let formatted_list: String = match format_options {
        FormatOptions::PlainText => releases
            .iter()
            .map(|release| {
                let tag_name = release.tag_name.clone();
                let body_text = release
                    .body
                    .clone()
                    .unwrap_or("No changelog provided".to_string());
                let assets_name_url = release
                    .assets
                    .iter()
                    .map(|asset| (asset.name.clone(), asset.browser_download_url.clone()))
                    .collect::<Vec<(String, String)>>()
                    .iter()
                    .map(|(name, url)| format!("{}: {}", name, url))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!(
                    "Version: {}\nChangelog:\n{}\nDownload URLs:\n{}\n",
                    tag_name, body_text, assets_name_url
                )
            })
            .collect::<Vec<String>>()
            .join("\n"),
        FormatOptions::Json => serde_json::to_string(&releases).unwrap_or_else(|e| {
            log::error!("Failed to format releases as JSON: {}", e);
            String::from("Error formatting as JSON")
        }),
        FormatOptions::Yaml => serde_yaml::to_string(&releases).unwrap_or_else(|e| {
            log::error!("Failed to format releases as YAML: {}", e);
            String::from("Error formatting as YAML")
        }),
    };

    println!("{}", formatted_list);
}
