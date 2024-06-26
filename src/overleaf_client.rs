pub const LOGIN_URL: &str = "https://www.overleaf.com/login";
pub const PROJECTS_URL: &str = "https://www.overleaf.com/project";
pub const DOWNLOAD_PROJECT_URL: &str = "https://www.overleaf.com/project/{}/download/zip";

use anyhow::{Context, Result};
use bytes::Bytes;
use headless_chrome::protocol::cdp::{types::JsFloat, Network::Cookie};
use reqwest::{
    header::{HeaderMap, HeaderValue, COOKIE},
    Client,
};
use serde::{Deserialize, Serialize};
use soup::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionCookie {
    pub name: String,
    pub value: String,
    pub expires: JsFloat,
}

impl SessionCookie {
    pub fn from_chrome_cookie(cookie: Cookie) -> Self {
        SessionCookie {
            name: cookie.name,
            value: cookie.value,
            expires: cookie.expires,
        }
    }
}

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
    pub fn new(session_cookie: SessionCookie) -> Self {
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

    // Fetch all projects.
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

    // Fetch specified project metadata.
    pub async fn get_project(&self, name: &String) -> Result<Project> {
        self.get_all_projects()
            .await?
            .projects
            .into_iter()
            .filter(|project| project.name == *name)
            .last()
            .context(format!("Project {name} not found."))
    }

    // Download specified project as zip.
    pub async fn download_project_zip(&self, project_id: String) -> Result<Bytes> {
        self.reqwest_client
            .get(DOWNLOAD_PROJECT_URL.replace("{}", project_id.as_str()))
            .send()
            .await?
            .bytes()
            .await
            .context(format!(
                "Error occured while downloading project {project_id} as zip.",
            ))
    }
}
