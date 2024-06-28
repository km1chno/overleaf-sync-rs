pub mod login_browser;
pub mod overleaf_client;
pub mod repository;

use crate::repository::{
    download_project, get_olsync_directory, init_ols_repository, is_ols_repository,
    move_project_to_backup,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

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
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Clone { name }) => match clone_action(name).await {
            Ok(()) => println!("Successfully cloned project {name}."),
            Err(err) => eprintln!("{err}\nFailed to clone project {name}."),
        },
        Some(Commands::Push { files }) => match push_action(files).await {
            Ok(()) => println!("Successfully pushed all files."),
            Err(err) => eprintln!("{err}\nFailed to push some files."),
        },
        Some(Commands::Pull) => match pull_action().await {
            Ok(()) => println!("Successfully pulled current project state from Overleaf."),
            Err(err) => eprintln!("{err}\nFailed to pull the project."),
        },
        None => {}
    }

    Ok(())
}

// Clone project into current directory.
async fn clone_action(name: &String) -> Result<()> {
    if is_ols_repository() {
        bail!(concat!(
            "An Overleaf project has already been cloned in this directory. ",
            "Remove the .olsync directory before cloning another project."
        ));
    }

    init_ols_repository(name)?;

    let olsync_dir = get_olsync_directory().with_context(|| "Failed to find .olsync directory.")?;

    download_project(&olsync_dir).await
}

// Push files to remote.
async fn push_action(files: &Vec<String>) -> Result<()> {
    files
        .into_iter()
        .for_each(|file| println!("Pushing {file}... PLACEHOLDER"));
    Ok(())
}

// Pull the current state from Overleaf.
async fn pull_action() -> Result<()> {
    if !is_ols_repository() {
        bail!("Not a olsync repository! Clone a project before pulling.")
    }

    let olsync_dir = get_olsync_directory().with_context(|| "Failed to find .olsync directory.")?;

    move_project_to_backup(&olsync_dir)?;

    download_project(&olsync_dir).await
}
