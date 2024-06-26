pub mod login_browser;
pub mod overleaf_client;

use crate::{
    login_browser::launch_login_browser,
    overleaf_client::{OverleafClient, ProjectsList, SessionCookie},
};

use anyhow::{bail, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use std::io::Cursor;
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

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
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Clone { name }) => match clone_action(name).await {
            Ok(()) => println!("Successfully cloned project {name}."),
            Err(err) => eprintln!("{err}\nFailed to clone project {name}."),
        },
        Some(Commands::List) => match list_action().await {
            Ok(projects) => {
                projects
                    .projects
                    .into_iter()
                    .for_each(|project| println!("{}: {}", project.id, project.name));
            }
            Err(err) => eprintln!("{err}\nFailed to list projects."),
        },
        None => {}
    }

    Ok(())
}

// Check whether .olsync directory exists.
fn is_ols_repository() -> bool {
    matches!(fs::exists(".olsync"), Ok(true))
}

// Initialize empty .olsync directory.
fn init_ols_repository() -> Result<()> {
    if is_ols_repository() {
        bail!("Already an olsync repository.");
    }

    fs::create_dir(".olsync")?;

    Ok(())
}

// Try to retrieve cached session cookie from .olsync/olauth.
fn get_session_cookie_from_file() -> Option<SessionCookie> {
    let file = File::open(".olsync/olauth");

    match file {
        Ok(f) => {
            let reader = BufReader::new(f);
            let cookie: Option<SessionCookie> = serde_json::from_reader(reader).unwrap_or(None);

            // Check whether the cookie has expired.
            cookie.filter(|c| c.expires > ((Utc::now().timestamp_millis() / 1000) as f64))
        }
        Err(_) => None,
    }
}

// Save session cookie to .olsync/olauth.
fn save_session_cookie_to_file(cookie: &SessionCookie) -> Result<()> {
    let serialized_cookie = serde_json::to_string(cookie)?;
    fs::write(".olsync/olauth", serialized_cookie)
        .or_else(|_| bail!("Failed to save session cookie to .olsync/olauth"))
}

// Read cached session cookie or spawn browser to login and
// save the new cookie in .olsync/olauth.
fn get_session_cookie() -> Result<SessionCookie> {
    if !is_ols_repository() {
        init_ols_repository()?;
    }

    match get_session_cookie_from_file() {
        Some(cookie) => Ok(cookie),
        _ => {
            let cookie = launch_login_browser()?;
            save_session_cookie_to_file(&cookie)?;
            Ok(cookie)
        }
    }
}

// List all remote projects.
async fn list_action() -> Result<ProjectsList> {
    let session_cookie = get_session_cookie()?;
    let overleaf_client = OverleafClient::new(session_cookie);
    overleaf_client.get_all_projects().await
}

// Clone project into current directory.
async fn clone_action(name: &String) -> Result<()> {
    if is_ols_repository() {
        bail!(concat!(
            "An Overleaf project has already been cloned in this directory. ",
            "Remove the .olsync directory before cloning another project."
        ));
    }

    init_ols_repository()?;

    let session_cookie = get_session_cookie()?;
    let overleaf_client = OverleafClient::new(session_cookie);
    let project = overleaf_client.get_project(name).await?;

    let archive: Vec<u8> = overleaf_client
        .download_project_zip(project.id)
        .await?
        .to_vec();

    zip_extract::extract(Cursor::new(archive), &PathBuf::from("."), true)
        .or_else(|_| bail!("Failed to extract downloaded project zip file."))
}
