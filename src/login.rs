use anyhow::{anyhow, Context, Result};
use cookie::Cookie;
use headless_chrome::browser::{Browser, LaunchOptionsBuilder};
use log::info;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, SET_COOKIE};
use std::time::Duration;

use crate::{
    constants::{
        GCLB_COOKIE_NAME, LOGIN_URL, ONE_HOUR_IN_SECONDS, SESSION_COOKIE_NAME, SOCKET_URL,
    },
    overleaf_client::{OlCookie, SessionInfo},
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

    let session_cookie = tab
        .get_cookies()?
        .iter()
        .find(|cookie| cookie.name == SESSION_COOKIE_NAME)
        .context("No session cookie found.")
        .cloned()
        .map(OlCookie::from_chrome_cookie)?;

    let gclb_cookie = get_gclb(session_cookie.clone()).await?;

    let csrf_token = "token?".to_owned();

    Ok(SessionInfo {
        session_cookie,
        gclb_cookie,
        csrf_token,
    })
}
