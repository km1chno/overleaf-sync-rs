pub mod auth;
pub mod constants;
pub mod custom_log;
pub mod overleaf_client;
pub mod repository;
pub mod utils;

use crate::{
    auth::get_session_info,
    custom_log::custom_log_format,
    overleaf_client::{OverleafClient, Project},
    repository::{
        create_local_backup, download_project, get_olsync_directory, get_project_dir,
        init_ols_repository, is_olsync_repository, push_files,
    },
};

use anyhow::{bail, Context, Result};
use clap::{Arg, ArgAction, ArgGroup, Command};
use log::{error, info, LevelFilter};

#[tokio::main]
async fn main() -> Result<()> {
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
                    Ok(()) => success!("Cloned project {project_name}."),
                    Err(err) => {
                        error!("Failed to clone project {project_name} with the following error:\n{err}")
                    }
                }
            } else {
                let project_id = matches.get_one("id").unwrap();
                match clone_by_id_action(project_id).await {
                    Ok(()) => success!("Cloned project {project_id}."),
                    Err(err) => {
                        error!(
                            "Failed to clone project {project_id} with the following error:\n{err}"
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
                Err(err) => error!("Failed to push some files with the following error:\n{err}"),
            }
        }
        Some(("pull", matches)) => {
            if !is_olsync_repository() {
                bail!("Not a olsync repository! Clone a project before pulling.")
            }

            let no_backup = matches.get_one::<bool>("no-backup").unwrap_or(&false);

            match pull_action(no_backup).await {
                Ok(()) => success!("Pulled current project state from Overleaf."),
                Err(err) => error!("Failed to pull the project with the following error:\n{err}"),
            }
        }
        _ => bail!("Unknown subcommand."),
    }

    Ok(())
}

// Clone project by name into current directory.
async fn clone_by_name_action(project_name: &String) -> Result<()> {
    let session_info = get_session_info().await?;
    let overleaf_client = OverleafClient::new(session_info)?;
    let project: Project = overleaf_client.get_project_by_name(project_name).await?;

    init_ols_repository(project_name)?;

    let olsync_dir = get_olsync_directory().with_context(|| "Failed to find .olsync directory.")?;

    download_project(&olsync_dir, &get_project_dir(&olsync_dir)?).await
}

// Clone project by id into current directory.
async fn clone_by_id_action(project_id: &String) -> Result<()> {
    Ok(())
}

// Push files to remote. Currently only files in root project directory are supported.
async fn push_action(files: Vec<&String>) -> Result<()> {
    info!("Pushing list of files {:?}.", files);

    if !is_olsync_repository() {
        bail!("Not a olsync repository! Clone a project before pushing.")
    }

    let olsync_dir = get_olsync_directory().with_context(|| "Failed to find .olsync directory.")?;

    push_files(&olsync_dir, files).await
}

// Pull the current state from remote.
async fn pull_action(no_backup: &bool) -> Result<()> {
    let olsync_dir = get_olsync_directory().with_context(|| "Failed to find .olsync directory.")?;

    create_local_backup(&olsync_dir)?;

    download_project(&olsync_dir, &get_project_dir(&olsync_dir)?).await
}
