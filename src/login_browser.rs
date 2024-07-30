use anyhow::{Context, Result};
use headless_chrome::browser::{Browser, LaunchOptionsBuilder};
use std::time::Duration;

use crate::overleaf_client::{
    OlCookie, SessionInfo, GCLB_COOKIE_NAME, LOGIN_URL, SESSION_COOKIE_NAME,
};

const ONE_HOUR_IN_SECONDS: u64 = 3600;

pub fn launch_login_browser() -> Result<SessionInfo> {
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

    // tab.wait_for_element_with_custom_timeout(
    //     "button#non-existing",
    //     Duration::new(ONE_HOUR_IN_SECONDS, 0),
    // )?;
    //
    // let gclb_cookie = tab
    //     .get_cookies()?
    //     .iter()
    //     .find(|cookie| cookie.name == GCLB_COOKIE_NAME)
    //     .context("No GCLB cookie found.")
    //     .cloned()
    //     .map(OlCookie::from_chrome_cookie)?;

    // Do I even need it?
    let gclb_cookie = OlCookie {
        name: "GCLB".to_owned(),
        value: "abc".to_owned(),
        expires: 0 as f64,
    };

    let csrf_token = "token?".to_owned();

    Ok(SessionInfo {
        session_cookie,
        gclb_cookie,
        csrf_token,
    })
}
