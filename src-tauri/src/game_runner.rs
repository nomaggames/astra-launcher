use std::path::PathBuf;
use std::process::Command;

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

    #[cfg(target_os = "windows")]
    {
        Command::new(&executable)
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        let install_dir = get_install_dir()?;
        Command::new("open")
            .arg("-a")
            .arg(install_dir.join("ASTRA.app"))
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new(&executable)
            .spawn()?;
    }

    Ok(())
}
