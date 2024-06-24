pub mod login_browser;
pub mod overleaf_client;

use crate::login_browser::launch_login_browser;

use anyhow::Result;

fn main() -> Result<()> {
    let session_cookie = launch_login_browser().expect("Session cookie not found.");

    println!("{}: {}", session_cookie.name, session_cookie.value);

    Ok(())
}
