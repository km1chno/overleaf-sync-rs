pub mod auth;
pub mod constants;
pub mod custom_log;
pub mod overleaf_client;
pub mod repository;
pub mod utils;

use std::path::PathBuf;

use crate::{
    auth::get_session_info,
    custom_log::custom_log_format,
    overleaf_client::{OverleafClient, Project},
    repository::{
        create_local_backup, download_project, get_project_info, get_repo_root,
        init_olsync_repository, is_olsync_repository, push_files, wipe_project,
    },
    utils::path_to_str,
};

use anyhow::{bail, Result};
use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};
use log::{error, info, LevelFilter};

#[tokio::main]
async fn main() {
    let matches = Command::new("olsync")
        .version("0.1.0")
        .author("Katzper Michno <katzper.michno@gmail.com>")
        .about("Overleaf projects synchronization tool")
        .subcommand(
            Command::new("clone")
                .about("Clone remote project")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .help("Project name"),
                )
                .arg(Arg::new("id").short('i').long("id").help("Project id"))
                .group(
                    ArgGroup::new("Project key")
                        .args(["name", "id"])
                        .required(true)
                        .multiple(false),
                ),
        )
        .subcommand(
            Command::new("push")
                .about("Push local files to remote project")
                .arg(
                    Arg::new("files")
                        .short('f')
                        .long("files")
                        .help("List of files to push")
                        .action(ArgAction::Append)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("pull")
                .about("Override local state with remote project")
                .arg(
                    Arg::new("no-backup")
                        .long("no-backup")
                        .help("Skip creating backup of local state before pulling")
                        .action(ArgAction::SetTrue),
                ),
        )
        .get_matches();

    env_logger::Builder::new()
        .format(custom_log_format)
        .filter_level(LevelFilter::Info)
        .init();

    if let Err(error) = run_olsync(matches).await {
        error!("{}", error)
    }
}

async fn run_olsync(matches: ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("clone", matches)) => {
            if is_olsync_repository() {
                bail!(concat!(
                    "An Overleaf project has already been cloned in this directory. ",
                    "Remove the .olsync directory before cloning another project."
                ));
            }

            if matches.contains_id("name") {
                let project_name = matches.get_one::<String>("name").unwrap();
                match clone_by_name_action(project_name).await {
                    Ok(project_path) => success!(
                        "Cloned project {project_name} into {}.",
                        path_to_str(&project_path)
                    ),
                    Err(err) => {
                        bail!("Failed to clone project {project_name} with the following error:\n{err}")
                    }
                }
            } else {
                let project_id = matches.get_one("id").unwrap();
                match clone_by_id_action(project_id).await {
                    Ok(project_path) => success!(
                        "Cloned project with id {project_id} into {}.",
                        path_to_str(&project_path)
                    ),
                    Err(err) => {
                        bail!(
                            "Failed to clone project with id {project_id} with the following error:\n{err}"
                        )
                    }
                }
            }
        }
        Some(("push", matches)) => {
            if !is_olsync_repository() {
                bail!("Not a olsync repository! Clone a project before pushing.")
            }

            let files: Vec<_> = matches.get_many::<String>("files").unwrap().collect();

            match push_action(files).await {
                Ok(()) => success!("Pushed all files."),
                Err(err) => bail!("Failed to push some files with the following error:\n{err}"),
            }
        }
        Some(("pull", matches)) => {
            if !is_olsync_repository() {
                bail!("Not a olsync repository! Clone a project before pulling.")
            }

            let no_backup = matches.get_one::<bool>("no-backup").unwrap_or(&false);

            match pull_action(no_backup).await {
                Ok(()) => success!("Pulled current project state from Overleaf."),
                Err(err) => bail!("Failed to pull the project with the following error:\n{err}"),
            }
        }
        _ => bail!("Unknown subcommand."),
    }

    Ok(())
}

// Clone project by name into current directory.
async fn clone_by_name_action(project_name: &String) -> Result<PathBuf> {
    let session_info = get_session_info().await?;
    let overleaf_client = OverleafClient::new(session_info)?;

    let project: Project = overleaf_client.get_project_by_name(project_name).await?;
    let repo_root = init_olsync_repository(&project)?;

    download_project(&overleaf_client, &project.id, &repo_root, None).await?;

    Ok(repo_root)
}

// Clone project by id into current directory.
async fn clone_by_id_action(project_id: &String) -> Result<PathBuf> {
    let session_info = get_session_info().await?;
    let overleaf_client = OverleafClient::new(session_info)?;

    let project: Project = overleaf_client.get_project_by_id(project_id).await?;
    let repo_root = init_olsync_repository(&project)?;

    download_project(&overleaf_client, &project.id, &repo_root, None).await?;

    Ok(repo_root)
}

// Push files to remote. Currently only files in root project directory are supported.
async fn push_action(files: Vec<&String>) -> Result<()> {
    info!("Pushing list of files {:?}.", files);

    push_files(files).await
}

// Pull the current state from remote.
async fn pull_action(no_backup: &bool) -> Result<()> {
    let session_info = get_session_info().await?;
    let overleaf_client = OverleafClient::new(session_info)?;

    if !no_backup {
        create_local_backup()?;
    }

    let project = get_project_info()?;
    let repo_root = get_repo_root()?;

    wipe_project()?;

    download_project(&overleaf_client, &project.id, &repo_root, None).await
}
