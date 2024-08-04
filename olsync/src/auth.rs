use anyhow::{anyhow, bail, Context, Result};
use cookie::Cookie;
use headless_chrome::browser::{Browser, LaunchOptionsBuilder};
use log::{info, warn};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, SET_COOKIE};
use std::io::BufReader;
use std::{
    fs::{self, File},
    path::PathBuf,
    time::Duration,
};

use crate::{
    constants::{
        GCLB_COOKIE_NAME, LOGIN_URL, ONE_HOUR_IN_SECONDS, SESSION_COOKIE_NAME, SOCKET_URL,
    },
    overleaf_client::{OlCookie, SessionInfo},
    success,
    utils::path_to_str,
};

// Request GCLB cookie.
async fn get_gclb(session_cookie: OlCookie) -> Result<OlCookie> {
    info!("Fetching GCLB cookie.");

    let mut headers = HeaderMap::new();

    headers.insert(
        COOKIE,
        HeaderValue::from_str(format!("{}={}", session_cookie.name, session_cookie.value).as_str())
            .context("Failed to build default headers for GCLB cookie request.")?,
    );

    let reqwest_client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .context("Failed to build reqwest client.")?;

    reqwest_client
        .get(SOCKET_URL)
        .send()
        .await?
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|val| val.to_str().ok().and_then(|s| Cookie::parse(s).ok()))
        .filter(|cookie| cookie.name() == GCLB_COOKIE_NAME)
        .last()
        .map(|cookie| OlCookie {
            name: cookie.name().to_owned(),
            value: cookie.value().to_owned(),
            expires: 0 as f64,
        })
        .ok_or(anyhow!(
            "GLCB cookie not found in Set-Cookie response header."
        ))
}

pub async fn login() -> Result<SessionInfo> {
    let launch_options = LaunchOptionsBuilder::default().headless(false).build()?;

    let browser = Browser::new(launch_options)?;

    let tab = browser.new_tab()?;

    tab.navigate_to(LOGIN_URL)?;

    tab.wait_for_element_with_custom_timeout(
        "button#new-project-button-sidebar",
        Duration::new(ONE_HOUR_IN_SECONDS, 0),
    )?;

    let email = tab
        .wait_for_element("meta[name=\"ol-usersEmail\"]")?
        .get_attribute_value("content")?
        .context("User email meta tag content attribute is empty")?;

    success!("Obtained user email.");

    let session_cookie = tab
        .get_cookies()?
        .iter()
        .find(|cookie| cookie.name == SESSION_COOKIE_NAME)
        .context("No session cookie found.")
        .cloned()
        .map(OlCookie::from_chrome_cookie)?;

    success!("Obtained session Cookie.");

    let csrf_token = tab
        .wait_for_element("meta[name=\"ol-csrfToken\"]")?
        .get_attribute_value("content")?
        .context("CSRF meta tag content attribute is empty.")?;

    success!("Obtained CSRF Token.");

    let gclb_cookie = get_gclb(session_cookie.clone()).await?;

    success!("Obtained GCLB Cookie.");

    Ok(SessionInfo {
        email,
        session_cookie,
        gclb_cookie,
        csrf_token,
    })
}

// Get PathBuf pointing to ~/.olsyncinfo (it may not exist)
fn get_olsyncinfo_path() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|p| p.join(".olsyncinfo"))
        .context("Failed to retrieve home directory.")
}

// Try to retrieve cached session info from ~/.olsyncinfo.
pub fn get_session_info_from_file() -> Option<SessionInfo> {
    let info_path = get_olsyncinfo_path().ok()?;

    match File::open(info_path) {
        Ok(f) => serde_json::from_reader(BufReader::new(f))
            .ok()
            .filter(|i: &SessionInfo| !i.session_cookie.has_expired()),
        Err(_) => None,
    }
}

// Remove ~/.olsyncinfo file.
pub fn remove_session_info() -> Result<()> {
    let info_path = get_olsyncinfo_path()?;
    fs::remove_file(info_path).map_err(|e| anyhow!("Failed to remove session info with error: {e}"))
}

// Save session info to ~/.olsyncinfo.
fn save_session_info_to_file(session_info: &SessionInfo) -> Result<()> {
    info!("Saving session information to cache.");

    let serialized_info = serde_json::to_string(session_info)?;
    let info_path = get_olsyncinfo_path()?;

    fs::write(info_path.clone(), serialized_info).or_else(|_| {
        bail!(
            "Failed to save session info to {}",
            path_to_str(info_path.as_path())
        )
    })
}

// Opens browser to log in and obtain new session information and saves it to cache.
pub async fn get_session_info_from_browser() -> Result<SessionInfo> {
    let session_info = login()
        .await
        .context("Failed to obtain session info from login browser")?;

    success!("Successfuly created new session.");

    save_session_info_to_file(&session_info)?;

    success!("Saved session info to cache.");

    Ok(session_info)
}

// Read cached session info or spawn browser to login and
// save new info in cache.
pub async fn get_session_info() -> Result<SessionInfo> {
    let session_info = get_session_info_from_file();

    if session_info.is_none() {
        warn!("Unable to detect cached session information. Opening browser for manual login.");
        get_session_info_from_browser().await
    } else {
        success!("Obtained session info from cache.");
        Ok(session_info.unwrap())
    }
}
