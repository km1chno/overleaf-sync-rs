pub const LOGIN_URL: &str = "https://www.overleaf.com/login";
pub const PROJECTS_URL: &str = "https://www.overleaf.com/project";

use anyhow::{Context, Result};
use headless_chrome::protocol::cdp::Network::Cookie;
use reqwest::{
    header::{HeaderMap, HeaderValue, COOKIE},
    Client,
};

use soup::prelude::*;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsList {
    pub total_size: u64,
    pub projects: Vec<Project>,
}

pub struct OverleafClient {
    reqwest_client: Client,
}

impl OverleafClient {
    pub fn new(session_cookie: Cookie) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert(
            COOKIE,
            HeaderValue::from_str(
                format!("{}={}", session_cookie.name, session_cookie.value).as_str(),
            )
            .expect("Failed to build default headers for Overleaf client."),
        );

        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build reqwest client.");

        Self { reqwest_client }
    }

    pub async fn get_all_projects(&self) -> Result<ProjectsList> {
        let projects_page_content = self
            .reqwest_client
            .get(PROJECTS_URL)
            .send()
            .await?
            .text()
            .await?;

        let projects_list_content = Soup::new(projects_page_content.as_str())
            .tag("meta")
            .attr("name", "ol-prefetchedProjectsBlob")
            .find()
            .and_then(|tag| tag.get("content"))
            .expect("Failed to retrieve list of projects.");

        serde_json::from_str(projects_list_content.as_str())
            .context("Failed to parse list of projects.")
    }
}
