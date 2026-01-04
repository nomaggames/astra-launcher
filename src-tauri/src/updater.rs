use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use futures_util::StreamExt;

const GITHUB_REPO: &str = "nomaggames/astra";
const GAME_FOLDER_NAME: &str = "astra-game";
// GitHub token for private repo access - embedded at build time
const GITHUB_TOKEN: &str = env!("ASTRA_GITHUB_TOKEN");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub download_url: String,
    pub release_notes: String,
    pub is_update_available: bool,
    pub installed_version: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub status: String,
}

fn get_install_dir() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let base = dirs::data_local_dir()
        .ok_or("Could not find local data directory")?;
    Ok(base.join(GAME_FOLDER_NAME))
}

fn get_version_file_path() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    Ok(get_install_dir()?.join("version.json"))
}

pub async fn get_installed_version() -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let version_path = get_version_file_path()?;

    if !version_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(version_path)?;
    let version_info: serde_json::Value = serde_json::from_str(&content)?;

    Ok(version_info["version"].as_str().map(String::from))
}

pub async fn check_for_updates() -> Result<UpdateInfo, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    // Fetch latest release from GitHub API
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let game_release = client
        .get(&url)
        .header("User-Agent", "ASTRA-Launcher")
        .header("Authorization", format!("Bearer {}", GITHUB_TOKEN))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    // Check if we got a valid release
    if game_release.get("tag_name").is_none() {
        return Err("No game release found".into());
    }

    let latest_version = game_release["tag_name"]
        .as_str()
        .unwrap_or("unknown")
        .trim_start_matches('v')
        .to_string();

    let release_notes = game_release["body"]
        .as_str()
        .unwrap_or("No release notes available.")
        .to_string();

    // Find the correct asset for this platform
    let asset_name = get_platform_asset_name();
    let download_url = game_release["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find(|a| {
                a["name"]
                    .as_str()
                    .map(|n| n.to_lowercase().contains(&asset_name))
                    .unwrap_or(false)
            })
        })
        .and_then(|asset| asset["browser_download_url"].as_str())
        .unwrap_or("")
        .to_string();

    let installed_version = get_installed_version().await?;

    let is_update_available = installed_version
        .as_ref()
        .map(|v| v != &latest_version)
        .unwrap_or(true); // No version installed = update needed

    Ok(UpdateInfo {
        latest_version,
        download_url,
        release_notes,
        is_update_available,
        installed_version,
    })
}

fn get_platform_asset_name() -> String {
    #[cfg(target_os = "windows")]
    return "windows".to_string();

    #[cfg(target_os = "macos")]
    return "macos".to_string();

    #[cfg(target_os = "linux")]
    return "linux".to_string();
}

pub async fn download_update<F>(
    version: &str,
    download_url: &str,
    progress_callback: F,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn(DownloadProgress) + Send + 'static,
{
    let client = reqwest::Client::new();
    let install_dir = get_install_dir()?;

    // Create install directory if it doesn't exist
    fs::create_dir_all(&install_dir)?;

    // Download file
    progress_callback(DownloadProgress {
        downloaded_bytes: 0,
        total_bytes: 0,
        percentage: 0.0,
        status: "Starting download...".to_string(),
    });

    let response = client
        .get(download_url)
        .header("User-Agent", "ASTRA-Launcher")
        .header("Authorization", format!("Bearer {}", GITHUB_TOKEN))
        .header("Accept", "application/octet-stream")
        .send()
        .await?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let temp_zip = install_dir.join("update.zip");
    let mut file = fs::File::create(&temp_zip)?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;

        progress_callback(DownloadProgress {
            downloaded_bytes: downloaded,
            total_bytes: total_size,
            percentage: if total_size > 0 {
                (downloaded as f32 / total_size as f32) * 100.0
            } else {
                0.0
            },
            status: "Downloading...".to_string(),
        });
    }

    // Extract zip
    progress_callback(DownloadProgress {
        downloaded_bytes: total_size,
        total_bytes: total_size,
        percentage: 100.0,
        status: "Extracting...".to_string(),
    });

    let file = fs::File::open(&temp_zip)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(&install_dir)?;

    // Clean up zip
    fs::remove_file(&temp_zip)?;

    // Write version file
    let version_info = serde_json::json!({
        "version": version,
        "installed_at": chrono::Utc::now().to_rfc3339(),
    });

    fs::write(
        get_version_file_path()?,
        serde_json::to_string_pretty(&version_info)?,
    )?;

    progress_callback(DownloadProgress {
        downloaded_bytes: total_size,
        total_bytes: total_size,
        percentage: 100.0,
        status: "Complete!".to_string(),
    });

    Ok(())
}

pub async fn uninstall_game() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let install_dir = get_install_dir()?;

    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }

    Ok(())
}
