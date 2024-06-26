pub mod login_browser;
pub mod overleaf_client;

use crate::{
    login_browser::launch_login_browser,
    overleaf_client::{OverleafClient, SessionCookie},
};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use std::{
    env::{self},
    io::Cursor,
};
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
            _ => {}
        },
        None => {}
    }

    Ok(())
}

// Returns .olsync directory in current repository. It traverses directory tree starting from
// currect directory and going upwards.
fn get_olsync_directory() -> Option<PathBuf> {
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
fn is_ols_repository() -> bool {
    get_olsync_directory().is_some()
}

// Initialize empty .olsync directory.
fn init_ols_repository(project_name: &String) -> Result<()> {
    if is_ols_repository() {
        bail!("Already an olsync repository.");
    }

    fs::create_dir(".olsync")?;
    fs::write(".olsync/name", project_name)?;

    Ok(())
}

// Try to retrieve cached session cookie from .olsync/olauth.
fn get_session_cookie_from_file(olsync_dir: &PathBuf) -> Option<SessionCookie> {
    let file = File::open(olsync_dir);

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
fn save_session_cookie_to_file(cookie: &SessionCookie, olsync_dir: &PathBuf) -> Result<()> {
    let serialized_cookie = serde_json::to_string(cookie)?;
    fs::write(olsync_dir, serialized_cookie)
        .or_else(|_| bail!("Failed to save session cookie to .olsync/olauth"))
}

// Read cached session cookie or spawn browser to login and
// save the new cookie in .olsync/olauth.
fn get_session_cookie(olsync_dir: &PathBuf) -> Result<SessionCookie> {
    match get_session_cookie_from_file(olsync_dir) {
        Some(cookie) => Ok(cookie),
        _ => {
            let cookie = launch_login_browser()?;
            save_session_cookie_to_file(&cookie, olsync_dir)?;
            Ok(cookie)
        }
    }
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
    Ok(())
}
