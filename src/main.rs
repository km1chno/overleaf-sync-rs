pub mod login_browser;
pub mod overleaf_client;

use crate::{login_browser::launch_login_browser, overleaf_client::OverleafClient};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let session_cookie = launch_login_browser().expect("Session cookie not found.");

    let overleaf_client = OverleafClient::new(session_cookie);

    let projects = overleaf_client.get_all_projects().await?;

    projects
        .projects
        .into_iter()
        .for_each(|project| println!("{}: {}", project.id, project.name));

    Ok(())
}
