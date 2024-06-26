pub mod login_browser;
pub mod overleaf_client;
pub mod repository;

use crate::{
    overleaf_client::OverleafClient,
    repository::{
        create_backup, get_olsync_directory, get_session_cookie, init_ols_repository,
        is_ols_repository,
    },
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Clone {
        #[arg(short, long)]
        name: String,
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

    let session_cookie = get_session_cookie(&olsync_dir)?;
    let overleaf_client = OverleafClient::new(session_cookie);
    let project = overleaf_client.get_project(name).await?;

    let archive: Vec<u8> = overleaf_client
        .download_project_zip(project.id)
        .await?
        .to_vec();

    zip_extract::extract(Cursor::new(archive), &PathBuf::from(name), true)
        .or_else(|_| bail!("Failed to extract downloaded project zip file."))
}

// Pull the current state from Overleaf.
async fn pull_action() -> Result<()> {
    if !is_ols_repository() {
        bail!("Not a olsync repository! Clone a project before pulling.")
    }

    let olsync_dir = get_olsync_directory().with_context(|| "Failed to find .olsync directory.")?;

    create_backup(&olsync_dir)?;

    Ok(())
}
