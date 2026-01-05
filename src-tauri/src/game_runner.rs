use std::path::PathBuf;
use std::process::Command;
use crate::config::load_config;

const GAME_FOLDER_NAME: &str = "astra-game";

fn get_install_dir() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let base = dirs::data_local_dir()
        .ok_or("Could not find local data directory")?;
    Ok(base.join(GAME_FOLDER_NAME))
}

#[cfg(target_os = "windows")]
fn get_game_executable() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let install_dir = get_install_dir()?;
    Ok(install_dir.join("ASTRA.exe"))
}

#[cfg(target_os = "macos")]
fn get_game_executable() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let install_dir = get_install_dir()?;
    Ok(install_dir.join("ASTRA.app/Contents/MacOS/ASTRA"))
}

#[cfg(target_os = "linux")]
fn get_game_executable() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let install_dir = get_install_dir()?;
    Ok(install_dir.join("ASTRA"))
}

pub async fn launch_game() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let executable = get_game_executable()?;

    if !executable.exists() {
        return Err("Game not installed. Please download first.".into());
    }

    // Load config to check fullscreen setting
    let config = load_config().unwrap_or_default();

    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new(&executable);

        if config.fullscreen {
            cmd.arg("--fullscreen");
        }

        cmd.spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        let install_dir = get_install_dir()?;
        let mut cmd = Command::new("open");
        cmd.arg("-a").arg(install_dir.join("ASTRA.app"));

        if config.fullscreen {
            cmd.arg("--args").arg("--fullscreen");
        }

        cmd.spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        let mut cmd = Command::new(&executable);

        if config.fullscreen {
            cmd.arg("--fullscreen");
        }

        cmd.spawn()?;
    }

    Ok(())
}
