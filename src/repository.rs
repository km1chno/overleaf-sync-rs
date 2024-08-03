use crate::{
    auth::get_session_info,
    custom_log::OlSpinner,
    overleaf_client::{OverleafClient, Project},
    utils::path_to_str,
};

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use fs_extra::dir::CopyOptions;
use log::info;
use std::io::BufReader;
use std::{env, path::Path};
use std::{fs::File, io::Cursor};
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

// Initialize new olsync repository in ./{project.name} and return its path.
pub fn init_olsync_repository(project: &Project) -> Result<PathBuf> {
    info!("Initializing empty olsync repository for project.");

    if is_olsync_repository() {
        bail!("This already is an olsync repository!");
    }

    let repo_dir = env::current_dir()?.join(project.name.clone());

    fs::create_dir_all(repo_dir.join(".olsync"))?;
    fs::write(
        repo_dir.join(".olsync").join("projectinfo"),
        serde_json::to_string(project)?,
    )?;

    Ok(repo_dir)
}

// Get current repository project info.
pub fn get_project_info() -> Result<Project> {
    get_olsync_directory()
        .map(|dir| dir.join("projectinfo"))
        .and_then(|project_info_path| File::open(project_info_path).ok())
        .and_then(|f| serde_json::from_reader(BufReader::new(f)).ok())
        .ok_or(anyhow!(
            "Failed to obtain project info from projectinfo file."
        ))
}

// Get repository root directory.
pub fn get_repo_root() -> Result<PathBuf> {
    return get_olsync_directory()
        .and_then(|s| s.parent().map(PathBuf::from))
        .ok_or_else(|| anyhow!("Failed to obtain project directory."));
}

// Create a timestamp annotated backup of local project.
pub fn create_local_backup() -> Result<()> {
    let mut spinner = OlSpinner::new("Creating backup of local project.".to_owned());

    let backup_result: Result<PathBuf, &str> = {
        let repo_root = get_repo_root()?;

        let bak_name = &format!(
            "{}-{}.local.bak",
            &get_project_info()?.name,
            Utc::now().timestamp_millis()
        );
        let bak_path = get_olsync_directory()
            .context("Failed to obtain .olsync directory.")?
            .join(bak_name);

        fs::create_dir(bak_path.clone())?;

        let items_in_root = fs::read_dir(repo_root)?;

        for item in items_in_root {
            let path = item.unwrap().path();
            let name = path.file_name().unwrap();

            if name != ".olsync" {
                fs_extra::copy_items(
                    &[path.to_str().unwrap()],
                    bak_path.clone(),
                    &CopyOptions::new(),
                )?;
            }
        }

        Ok(bak_path)
    };

    if let Ok(bak_path) = backup_result {
        spinner.stop_with_success(format!(
            "Saved backup of local project in {}.",
            path_to_str(bak_path.as_path())
        ));
        Ok(())
    } else {
        spinner.stop_with_error("Failed to create backup of local project.".to_owned());
        bail!(backup_result.err().unwrap())
    }
}

// Wipes everything in root directory except .olsync.
pub fn wipe_project() -> Result<()> {
    let repo_root = get_repo_root()?;

    info!("Wiping everything in repo root directory.");

    let items_in_root = fs::read_dir(repo_root)?;

    for item in items_in_root {
        let path = item.unwrap().path();
        let name = path.file_name().unwrap();

        if name != ".olsync" {
            fs_extra::remove_items(&[path.to_str().unwrap()])?;
        }
    }

    Ok(())
}

// Download project from Overleaf in zip and save in target directory as {archive_name.zip}.
// If archive_name is None, the archive will be extracted.
pub async fn download_project(
    overleaf_client: &OverleafClient,
    project_id: &str,
    target_dir: &Path,
    archive_name: Option<String>,
) -> Result<()> {
    info!("Downloading project into {}.", path_to_str(target_dir));

    let mut spinner = OlSpinner::new("Downloading project.".to_owned());

    let download_result = {
        let archive: Vec<u8> = overleaf_client
            .download_project_zip(project_id.to_owned())
            .await?
            .to_vec();

        match archive_name {
            Some(name) => {
                let file_name = format!("{}.zip", name);

                fs::write(PathBuf::from(target_dir).join(name), archive)
                    .map(|()| format!("Saved project as {}.", file_name))
                    .context("Failed to save downloaded project.".to_owned())
            }
            None => zip_extract::extract(Cursor::new(archive), target_dir, true)
                .map(|()| "Downloaded and extracted project.".to_owned())
                .context("Failed to extract downloaded project zip file.".to_owned()),
        }
    };

    if let Ok(message) = download_result {
        spinner.stop_with_success(message);
        Ok(())
    } else {
        spinner.stop_with_error("Failed to download and save project.".to_owned());
        bail!(download_result.err().unwrap())
    }
}

pub async fn push_files(files: Vec<&String>) -> Result<()> {
    info!("Pushing {:?}", files);

    let session_info = get_session_info().await?;
    let overleaf_client = OverleafClient::new(session_info)?;
    let project_name = &get_project_info()?.name;

    let project = overleaf_client.get_project_by_name(project_name).await?;

    let project_details = overleaf_client.get_project_details(&project.id).await?;

    info!("Root folder is {}.", project_details.root_folder[0].id);

    Ok(())
}
