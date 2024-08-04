pub mod auth;
pub mod constants;
pub mod custom_log;
pub mod overleaf_client;
pub mod repository;
pub mod utils;

use crate::{
    auth::{get_session_info, get_session_info_from_file},
    custom_log::{custom_log_format, OlSpinner},
    overleaf_client::OverleafClient,
    repository::{
        create_local_backup, download_project, get_project_info, get_repo_root,
        init_olsync_repository, is_olsync_repository, push_files, wipe_project,
    },
    utils::path_to_str,
};

use anyhow::{anyhow, bail, Result};
use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};
use colored::Colorize;
use log::{error, LevelFilter};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let matches = Command::new("olsync")
        .version("0.1.0")
        .author("Katzper Michno <katzper.michno@gmail.com>")
        .about("CLI for synchronizing LaTeX projects between Overleaf and your local machine.")
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
        .subcommand(Command::new("whoami").about("Print current session info"))
        .subcommand(
            Command::new("push")
                .about("Push local files to remote project")
                .arg(
                    Arg::new("files")
                        .help("List of files to push")
                        .action(ArgAction::Append)
                        .required(true),
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .help("Skip confirm prompt")
                        .action(ArgAction::SetTrue),
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
                )
                .arg(
                    Arg::new("force")
                        .long("force")
                        .help("Skip confirm prompt")
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
        Some(("whoami", _matches)) => match whoami_action().await {
            Ok(()) => {}
            Err(err) => {
                bail!("Failed to obtain session info with the following error:\n{err}")
            }
        },
        Some(("clone", matches)) => {
            if is_olsync_repository() {
                bail!(concat!(
                    "An Overleaf project has already been cloned in this directory. ",
                    "Remove the .olsync directory before cloning another project."
                ));
            }

            let project_name = matches.get_one::<String>("name");
            let project_id = matches.get_one::<String>("id");

            match clone_action(project_name, project_id).await {
                Ok(project_path) => success!(
                    "Successfully cloned project {} into {}.",
                    project_name.unwrap_or_else(|| project_id.unwrap()),
                    path_to_str(&project_path)
                ),
                Err(err) => {
                    bail!(
                        "Failed to clone project {} with the following error:\n{err}",
                        project_name.unwrap_or_else(|| project_id.unwrap())
                    )
                }
            }
        }
        Some(("push", matches)) => {
            if !is_olsync_repository() {
                bail!("Not a olsync repository! Clone a project before pushing.")
            }

            let force = matches.get_one::<bool>("force").unwrap_or(&false);
            let files: Vec<_> = matches.get_many::<String>("files").unwrap().collect();

            match push_action(files, force).await {
                Ok(true) => success!("Successfully pushed all files!"),
                Err(err) => bail!("Failed to push some files with the following error:\n{err}"),
                _ => {}
            }
        }
        Some(("pull", matches)) => {
            if !is_olsync_repository() {
                bail!("Not a olsync repository! Clone a project before pulling.")
            }

            let no_backup = matches.get_one::<bool>("no-backup").unwrap_or(&false);
            let force = matches.get_one::<bool>("force").unwrap_or(&false);

            match pull_action(no_backup, force).await {
                Ok(true) => success!("Successfully pulled current project state from Overleaf!"),
                Err(err) => bail!("Failed to pull the project with the following error:\n{err}"),
                _ => {}
            }
        }
        _ => bail!("Unknown subcommand."),
    }

    Ok(())
}

// Print session info.
async fn whoami_action() -> Result<()> {
    if let Some(info) = get_session_info_from_file() {
        println!("{}", info.email.green());
        println!(
            "Session expires at {}",
            info.session_cookie.expiry_date_pretty()
        );
    } else {
        println!("Not logged in. Use {}.", "olsync login".cyan());
    }

    Ok(())
}

// Clone project into ./{project_name} directory.
async fn clone_action(
    project_name: Option<&String>,
    project_id: Option<&String>,
) -> Result<PathBuf> {
    let session_info = get_session_info().await?;
    let overleaf_client = OverleafClient::new(session_info)?;

    let mut spinner = OlSpinner::new("Fetching project information...".to_owned());

    let project_result = match project_name {
        Some(name) => overleaf_client.get_project_by_name(name).await,
        None => overleaf_client.get_project_by_id(project_id.unwrap()).await,
    };

    if project_result.is_err() {
        spinner.stop_with_error("Failed to fetch project information.".to_owned());
        return Err(project_result.err().unwrap());
    }

    let project = project_result.unwrap();

    spinner.stop_with_success(format!("Fetched information for project {}.", project.name));

    let repo_root = init_olsync_repository(&project)?;

    download_project(&overleaf_client, &project.id, &repo_root, None).await?;

    Ok(repo_root)
}

// Push files to remote. Currently only files in root project directory are supported.
async fn push_action(files: Vec<&String>, force: &bool) -> Result<bool> {
    let confirm = inquire::Confirm::new(
        "Pushing files to Overleaf will override them. Do you want to continue?",
    )
    .with_default(false);

    let ans = if *force { Ok(true) } else { confirm.prompt() };

    if matches!(ans, Ok(true)) {
        let session_info = get_session_info().await?;
        let overleaf_client = OverleafClient::new(session_info)?;

        let project = get_project_info()?;

        push_files(&overleaf_client, &project.id, files).await?;
    }

    ans.map_err(|e| anyhow!("An error ocurred in prompt: {e}"))
}

// Pull the current state from remote.
async fn pull_action(no_backup: &bool, force: &bool) -> Result<bool> {
    let confirm = inquire::Confirm::new(
        "Pulling project from Overleaf will override your local state. Do you want to continue?")
        .with_help_message("If you proceed, your local project will be backed up (unless --no-backup option has been used).")
        .with_default(false);

    let ans = if *force { Ok(true) } else { confirm.prompt() };

    if matches!(ans, Ok(true)) {
        let session_info = get_session_info().await?;
        let overleaf_client = OverleafClient::new(session_info)?;

        if !no_backup {
            create_local_backup()?;
        }

        let project = get_project_info()?;
        let repo_root = get_repo_root()?;

        wipe_project()?;

        download_project(&overleaf_client, &project.id, &repo_root, None).await?;
    }

    ans.map_err(|e| anyhow!("An error ocurred in prompt: {e}"))
}
