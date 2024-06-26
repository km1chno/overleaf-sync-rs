use crate::{login_browser::launch_login_browser, overleaf_client::SessionCookie};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use fs_extra::dir::CopyOptions;
use std::env::{self};
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

// Returns .olsync directory in current repository. It traverses directory tree starting from
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
pub fn is_ols_repository() -> bool {
    get_olsync_directory().is_some()
}

// Initialize empty .olsync directory.
pub fn init_ols_repository(project_name: &String) -> Result<()> {
    if is_ols_repository() {
        bail!("Already an olsync repository.");
    }

    fs::create_dir(".olsync")?;
    fs::write(".olsync/name", project_name)?;

    Ok(())
}

// Try to retrieve cached session cookie from .olsync/olauth.
pub fn get_session_cookie_from_file(olsync_dir: &PathBuf) -> Option<SessionCookie> {
    let cookie_path = &PathBuf::from(olsync_dir).join("olauth");
    let file = File::open(cookie_path);

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
pub fn save_session_cookie_to_file(cookie: &SessionCookie, olsync_dir: &PathBuf) -> Result<()> {
    let serialized_cookie = serde_json::to_string(cookie)?;
    let cookie_path = &PathBuf::from(olsync_dir).join("olauth");

    fs::write(cookie_path, serialized_cookie).or_else(|_| {
        bail!(
            "Failed to save session cookie to {}",
            cookie_path.to_str().unwrap_or("INVALID PATH")
        )
    })
}

// Read cached session cookie or spawn browser to login and
// save the new cookie in .olsync/olauth.
pub fn get_session_cookie(olsync_dir: &PathBuf) -> Result<SessionCookie> {
    match get_session_cookie_from_file(olsync_dir) {
        Some(cookie) => Ok(cookie),
        _ => {
            let cookie = launch_login_browser()?;
            save_session_cookie_to_file(&cookie, olsync_dir)?;
            Ok(cookie)
        }
    }
}

// Get current repository project name.
pub fn get_current_project_name(olsync_dir: &PathBuf) -> Result<String> {
    fs::read_to_string(PathBuf::from(olsync_dir).join("name"))
        .context("Failed to read project name.")
}

// Create a timestamp annotated backup of current local project.
pub fn create_backup(olsync_dir: &PathBuf) -> Result<()> {
    let name = &get_current_project_name(olsync_dir)?;
    let timestamp = Utc::now().timestamp_millis();
    let root_dir = olsync_dir
        .parent()
        .with_context(|| "Could not find repository root directory.")?;

    let project_dir = root_dir.join(name);
    let bak_name = &format!("{name}-{timestamp}.bak");
    let renamed_dir = root_dir.join(bak_name);
    let bak_dir = olsync_dir.join(bak_name);

    println!(
        "Creating backup of {} in {}",
        project_dir.to_str().unwrap_or("INVALID PATH"),
        bak_dir.to_str().unwrap_or("INVALID PATH")
    );

    fs::rename(project_dir.clone(), bak_name)?;
    fs_extra::dir::copy(renamed_dir.clone(), olsync_dir, &CopyOptions::new())?;
    fs::rename(renamed_dir, name)?;

    Ok(())
}
