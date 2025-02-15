use crate::arguments::FormatOptions;
use anyhow::{Context, Result};
use log::*;
use octocrate::{repos::list_releases::Response, APIConfig, GitHubAPI};

pub async fn get_list_of_releases() -> Result<Response> {
    debug!("Initializing API configuration for listing releases");
    let config = APIConfig::default().shared();
    let api = GitHubAPI::new(&config);
    info!("Fetching list of releases for drew-chase/JetBrains-Toolbox-Context-Menu");
    api.repos
        .list_releases("drew-chase", "JetBrains-Toolbox-Context-Menu")
        .send()
        .await
        .context("Failed to fetch the list of releases")
}

pub async fn get_latest_release() -> Result<octocrate::repos::get_latest_release::Response> {
    debug!("Initializing API configuration for latest release");
    let config = APIConfig::default().shared();
    let api = GitHubAPI::new(&config);
    info!("Fetching latest release for drew-chase/JetBrains-Toolbox-Context-Menu");
    api.repos
        .get_latest_release("drew-chase", "JetBrains-Toolbox-Context-Menu")
        .send()
        .await
        .context("Failed to fetch the latest release")
}

pub async fn print_latest_release(format_options: FormatOptions) -> Result<()> {
    debug!(
        "Attempting to get latest release with format: {:?}",
        format_options
    );
    let release = get_latest_release()
        .await
        .context("Error getting release information")?;
    debug!(
        "Successfully retrieved latest release: {}",
        release.tag_name
    );

    let formatted_list = match format_options {
        FormatOptions::PlainText => {
            trace!("Formatting release as plain text");
            let tag_name = release.tag_name.clone();
            let body_text = release.body.clone().unwrap_or_else(|| {
                warn!("No changelog provided for release {}", tag_name);
                "No changelog provided".to_string()
            });
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
            error!("Failed to format release as JSON: {}", e);
            String::from("Error formatting as JSON")
        }),
        FormatOptions::Yaml => serde_yaml::to_string(&release).unwrap_or_else(|e| {
            error!("Failed to format release as YAML: {}", e);
            String::from("Error formatting as YAML")
        }),
    };
    debug!("Printing formatted release information");
    println!("{}", formatted_list);
    Ok(())
}

pub(crate) async fn print_all_releases(format_options: FormatOptions) -> Result<()> {
    debug!(
        "Attempting to get all releases with format: {:?}",
        format_options
    );
    let releases = get_list_of_releases()
        .await
        .context("Error getting release information")?;
    info!("Successfully retrieved {} releases", releases.len());

    let formatted_list: String = match format_options {
        FormatOptions::PlainText => {
            trace!("Formatting releases as plain text");
            releases
                .iter()
                .map(|release| {
                    let tag_name = release.tag_name.clone();
                    trace!("Processing release: {}", tag_name);
                    let body_text = release.body.clone().unwrap_or_else(|| {
                        warn!("No changelog provided for release {}", tag_name);
                        "No changelog provided".to_string()
                    });
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
                .join("\n")
        }
        FormatOptions::Json => serde_json::to_string(&releases).unwrap_or_else(|e| {
            error!("Failed to format releases as JSON: {}", e);
            String::from("Error formatting as JSON")
        }),
        FormatOptions::Yaml => serde_yaml::to_string(&releases).unwrap_or_else(|e| {
            error!("Failed to format releases as YAML: {}", e);
            String::from("Error formatting as YAML")
        }),
    };
    debug!("Printing formatted releases information");
    println!("{}", formatted_list);
    Ok(())
}
