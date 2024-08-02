use crate::{
    auth::get_session_info,
    overleaf_client::{OverleafClient, Project},
    utils::path_to_str,
};

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use fs_extra::dir::CopyOptions;
use log::{info, warn};
use std::io::Cursor;
use std::{env, path::Path};
use std::{
    fs::{self},
    path::PathBuf,
};

// Returns .olsync directory in current repository. It traverses directory hierarchy starting from
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
pub fn is_olsync_repository() -> bool {
    get_olsync_directory().is_some()
}

// Initialize new .olsync directory.
pub fn init_olsync_repository(project: &Project) -> Result<()> {
    info!(
        "Initializing empty olsync repository for project {}.",
        project.id
    );

    if is_olsync_repository() {
        bail!("Already an olsync repository.");
    }

    fs::create_dir(".olsync")?;
    fs::write(".olsync/name", serde_json::to_string(project)?)?;

    Ok(())
}

// Get current repository project name.
fn get_project_name(olsync_dir: &PathBuf) -> Result<String> {
    fs::read_to_string(PathBuf::from(olsync_dir).join("name"))
        .context("Failed to read project name.")
}

// Get project directory.
pub fn get_project_dir() -> Result<PathBuf> {
    return get_olsync_directory()
        .map(|s| s.parent().map(PathBuf::from))
        .flatten()
        .ok_or_else(|| anyhow!("Failed to obtain project directory."));
}

// Create a zipped, timestamp annotated backup of current local project.
pub fn create_local_backup(olsync_dir: &PathBuf) -> Result<()> {
    let project_dir = get_project_dir()?;

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
pub async fn download_project(
    overleaf_client: &OverleafClient,
    project_id: String,
    target_dir: &Path,
) -> Result<()> {
    info!(
        "Downloading project {project_id} into {}.",
        path_to_str(target_dir)
    );

    let archive: Vec<u8> = overleaf_client
        .download_project_zip(project_id)
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

pub async fn push_files(olsync_dir: &PathBuf, files: Vec<&String>) -> Result<()> {
    let session_info = get_session_info().await?;
    let overleaf_client = OverleafClient::new(session_info)?;
    let project_name = &get_project_name(olsync_dir)?;

    let project = overleaf_client.get_project_by_name(project_name).await?;

    let project_details = overleaf_client.get_project_details(&project.id).await?;

    info!("Root folder is {}.", project_details.root_folder[0].id);

    Ok(())
}
