use crate::{
    login_browser::launch_login_browser,
    overleaf_client::{OverleafClient, SessionInfo},
    utils::path_to_str,
};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use fs_extra::dir::CopyOptions;
use log::{info, warn};
use std::io::Cursor;
use std::{env, path::Path};
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

// Returns .olsync directory in current repository. It traverses directory tree starting from
// currect directory and going upwards.
pub fn get_olsync_directory() -> Option<PathBuf> {
    let mut current_dir = env::current_dir().ok();

    while let Some(pb) = current_dir {
        let dir_str = match pb.to_str() {
            Some(s) => format!("{s}/.olsync"),
            _ => return None,
        };

        if matches!(fs::exists(dir_str.clone()), Ok(true)) {
            return Some(PathBuf::from(dir_str));
        }

        current_dir = pb.parent().map(PathBuf::from);
    }

    None
}

// Check whether .olsync directory exists.
pub fn is_ols_repository() -> bool {
    get_olsync_directory().is_some()
}

// Initialize empty .olsync directory.
pub fn init_ols_repository(project_name: &String) -> Result<()> {
    info!("Initializing empty olsync repository.");

    if is_ols_repository() {
        bail!("Already an olsync repository.");
    }

    fs::create_dir(".olsync")?;
    fs::write(".olsync/name", project_name)?;

    Ok(())
}

// Try to retrieve cached session info from .olsync/olauth.
fn get_session_info_from_file(olsync_dir: &PathBuf) -> Option<SessionInfo> {
    info!("Trying to retrieve cached session information.");

    let info_path = &PathBuf::from(olsync_dir).join("olauth");

    match File::open(info_path) {
        Ok(f) => {
            let reader = BufReader::new(f);
            serde_json::from_reader(reader)
                .ok()
                .filter(|i: &SessionInfo| !i.session_cookie.has_expired())
        }
        Err(_) => None,
    }
}

// Save session info to .olsync/olauth.
fn save_session_info_to_file(olsync_dir: &PathBuf, session_info: &SessionInfo) -> Result<()> {
    info!("Saving session information to cache.");

    let serialized_info = serde_json::to_string(session_info)?;
    let info_path = &PathBuf::from(olsync_dir).join("olauth");

    fs::write(info_path, serialized_info)
        .or_else(|_| bail!("Failed to save session info to {}", path_to_str(info_path)))
}

// Read cached session info or spawn browser to login and
// save new info in .olsync/olauth.
fn get_session_info(olsync_dir: &PathBuf) -> Result<SessionInfo> {
    get_session_info_from_file(olsync_dir)
        .map(Ok)
        .unwrap_or_else(|| {
            warn!("Unable to detect cached session information. Opening browser for manual login.");
            let session_info = launch_login_browser()?;
            save_session_info_to_file(olsync_dir, &session_info)?;
            Ok(session_info)
        })
}

// Get current repository project name.
fn get_project_name(olsync_dir: &PathBuf) -> Result<String> {
    fs::read_to_string(PathBuf::from(olsync_dir).join("name"))
        .context("Failed to read project name.")
}

// Get project directory, equal to {olsync_dir}/../{project_name}. Does not check whether the
// directory exists!
pub fn get_project_dir(olsync_dir: &PathBuf) -> Result<PathBuf> {
    let project_name = &get_project_name(olsync_dir)?;
    let root_dir = olsync_dir
        .parent()
        .with_context(|| "Could not find repository root directory.")?;

    Ok(root_dir.join(project_name))
}

// Create a zipped, timestamp annotated backup of current local project.
pub fn create_local_backup(olsync_dir: &PathBuf) -> Result<()> {
    let project_dir = get_project_dir(olsync_dir)?;

    if !matches!(fs::exists(project_dir.clone()), Ok(true)) {
        warn!("No local project files found. Not backup created.");
        return Ok(());
    }

    let bak_name = &format!(
        "{}-{}.local.bak",
        &get_project_name(olsync_dir)?,
        Utc::now().timestamp_millis()
    );
    let bak_path = olsync_dir.join(bak_name);

    info!(
        "Creating local backup in {}.",
        path_to_str(bak_path.clone().as_path())
    );

    fs::create_dir(bak_path.clone())?;
    fs_extra::dir::copy(project_dir, bak_path, &CopyOptions::new())?;

    Ok(())
}

// Download project from Overleaf in zip and save in target directory. It will be extracted if
// target_dir extension is not 'zip'. If target_dir already exists, it will be overriden.
pub async fn download_project(olsync_dir: &PathBuf, target_dir: &Path) -> Result<()> {
    info!("Downloading project into {}.", path_to_str(target_dir));

    let session_info = get_session_info(olsync_dir)?;
    let overleaf_client = OverleafClient::new(session_info)?;
    let project_name = &get_project_name(olsync_dir)?;
    let project = overleaf_client.get_project(project_name).await?;

    let archive: Vec<u8> = overleaf_client
        .download_project_zip(project.id)
        .await?
        .to_vec();

    // If target_dir has .zip extension, do not extract.
    if matches!(target_dir.extension().and_then(|e| e.to_str()), Some("zip")) {
        info!("Not extracting downloaded archive since target has zip extension.");

        fs::write(target_dir, archive).context(format!(
            "Failed to save downloaded project to {}.",
            path_to_str(target_dir)
        ))
    } else {
        info!("Extracting downloaded archive.");

        // Wipe out current contents of target_dir before extracting.
        if matches!(fs::exists(target_dir), Ok(true)) {
            fs::remove_dir_all(target_dir)?;
        }

        zip_extract::extract(Cursor::new(archive), target_dir, true).or_else(|_| {
            bail!(
                "Failed to extract downloaded project zip file to {}.",
                path_to_str(target_dir)
            )
        })
    }
}

pub async fn push_files(olsync_dir: &PathBuf, files: &[String]) -> Result<()> {
    let session_info = get_session_info(olsync_dir)?;
    let overleaf_client = OverleafClient::new(session_info)?;
    let project_name = &get_project_name(olsync_dir)?;
    let project = overleaf_client.get_project(project_name).await?;

    overleaf_client.get_project_info(&project.id).await?;

    Ok(())
}
