pub mod custom_log;
pub mod login_browser;
pub mod overleaf_client;
pub mod repository;
pub mod utils;

use crate::{
    custom_log::custom_log_format,
    repository::{
        create_local_backup, download_project, get_olsync_directory, get_project_dir,
        init_ols_repository, is_ols_repository, push_files,
    },
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use log::{error, info, LevelFilter};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Clone {
        #[arg(short, long, required = true)]
        name: String,
    },
    Push {
        #[arg(short, long, required = true)]
        files: Vec<String>,
    },
    Pull,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .format(custom_log_format)
        .filter(None, LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Clone { name }) => match clone_action(name).await {
            Ok(()) => success!("Cloned project {name}."),
            Err(err) => error!("Failed to clone project {name} with the following error:\n{err}"),
        },
        Some(Commands::Push { files }) => match push_action(files).await {
            Ok(()) => success!("Pushed all files."),
            Err(err) => error!("Failed to push some files with the following error:\n{err}"),
        },
        Some(Commands::Pull) => match pull_action().await {
            Ok(()) => success!("Pulled current project state from Overleaf."),
            Err(err) => error!("Failed to pull the project with the following error:\n{err}"),
        },
        None => {}
    }

    Ok(())
}

// Clone project into current directory.
async fn clone_action(project_name: &String) -> Result<()> {
    if is_ols_repository() {
        bail!(concat!(
            "An Overleaf project has already been cloned in this directory. ",
            "Remove the .olsync directory before cloning another project."
        ));
    }

    init_ols_repository(project_name)?;

    let olsync_dir = get_olsync_directory().with_context(|| "Failed to find .olsync directory.")?;

    download_project(&olsync_dir, &get_project_dir(&olsync_dir)?).await
}

// Push files to Overleaf. Currently only files in root project directory are supported.
async fn push_action(files: &[String]) -> Result<()> {
    info!("Pushing list of files {:?}.", files);

    if !is_ols_repository() {
        bail!("Not a olsync repository! Clone a project before pushing.")
    }

    let olsync_dir = get_olsync_directory().with_context(|| "Failed to find .olsync directory.")?;

    push_files(&olsync_dir, files).await
}

// Pull the current state from Overleaf.
async fn pull_action() -> Result<()> {
    if !is_ols_repository() {
        bail!("Not a olsync repository! Clone a project before pulling.")
    }

    let olsync_dir = get_olsync_directory().with_context(|| "Failed to find .olsync directory.")?;

    create_local_backup(&olsync_dir)?;

    download_project(&olsync_dir, &get_project_dir(&olsync_dir)?).await
}
