use anyhow::{Context, Result};
use headless_chrome::browser::{Browser, LaunchOptionsBuilder};
use std::time::Duration;

use crate::overleaf_client::{SessionCookie, LOGIN_URL};

const ONE_HOUR_IN_SECONDS: u64 = 3600;
const SESSION_COOKIE_NAME: &str = "overleaf_session2";

pub fn launch_login_browser() -> Result<SessionCookie> {
    let launch_options = LaunchOptionsBuilder::default().headless(false).build()?;

    let browser = Browser::new(launch_options)?;

    let tab = browser.new_tab()?;

    tab.navigate_to(LOGIN_URL)?;

    tab.wait_for_element_with_custom_timeout(
        "button#new-project-button-sidebar",
        Duration::new(ONE_HOUR_IN_SECONDS, 0),
    )?;

    tab.get_cookies()?
        .iter()
        .find(|cookie| cookie.name == SESSION_COOKIE_NAME)
        .context("No session cookie.")
        .cloned()
        .map(SessionCookie::from_chrome_cookie)
}
