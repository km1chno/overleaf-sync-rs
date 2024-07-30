use anyhow::{Context, Result};
use bytes::Bytes;
use chrono::Utc;
use headless_chrome::protocol::cdp::{types::JsFloat, Network::Cookie};
use reqwest::{
    header::{HeaderMap, HeaderValue, COOKIE},
    Client,
};
use serde::{Deserialize, Serialize};
use soup::prelude::*;

pub const BASE_URL: &str = "https://www.overleaf.com";
pub const LOGIN_URL: &str = "https://www.overleaf.com/login";
pub const PROJECTS_URL: &str = "https://www.overleaf.com/project";
pub const DOWNLOAD_PROJECT_URL: &str = "https://www.overleaf.com/project/{}/download/zip";

pub const SESSION_COOKIE_NAME: &str = "overleaf_session2";
pub const GCLB_COOKIE_NAME: &str = "GCLB";

#[derive(Debug, Deserialize, Serialize)]
pub struct OlCookie {
    pub name: String,
    pub value: String,
    pub expires: JsFloat,
}

impl OlCookie {
    pub fn from_chrome_cookie(cookie: Cookie) -> Self {
        OlCookie {
            name: cookie.name,
            value: cookie.value,
            expires: cookie.expires,
        }
    }

    pub fn has_expired(&self) -> bool {
        self.expires <= ((Utc::now().timestamp_millis() / 1000) as f64)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SessionInfo {
    pub session_cookie: OlCookie,
    pub gclb_cookie: OlCookie,
    pub csrf_token: String,
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

pub struct ProjectInfo {
    pub root_folder_id: String,
}

pub struct OverleafClient {
    session_info: SessionInfo,
    reqwest_client: Client,
}

impl OverleafClient {
    pub fn new(session_info: SessionInfo) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert(
            COOKIE,
            HeaderValue::from_str(
                &[&session_info.session_cookie, &session_info.gclb_cookie]
                    .map(|cookie| format!("{}={}", cookie.name, cookie.value))
                    .join(";"),
            )
            .expect("Failed to build default headers for Overleaf client."),
        );

        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build reqwest client.");

        Self {
            session_info,
            reqwest_client,
        }
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

    // Fetch specified project.
    pub async fn get_project(&self, project_name: &String) -> Result<Project> {
        self.get_all_projects()
            .await?
            .projects
            .into_iter()
            .filter(|project| project.name == *project_name)
            .last()
            .context(format!("Project {project_name} not found."))
    }

    // Fetch specified project info.
    pub async fn get_project_info(&self, project_id: &String) -> Result<ProjectInfo> {
        Ok(ProjectInfo {
            root_folder_id: format!("fake_{}", project_id),
        })
        // println!("Getting project info!!!");
        //
        // let socket = ClientBuilder::new(BASE_URL)
        //     .namespace(format!("projectId={}", project_id))
        //     .opening_header(
        //         "Cookie",
        //         format!(
        //             "{}={}",
        //             self.session_info.session_cookie.name, self.session_info.session_cookie.value
        //         ),
        //     )
        //     .on("joinProjectResponse", |payload, _| {
        //         async move { println!("{:?}", payload) }.boxed()
        //     })
        //     .connect()
        //     .await
        //     .expect("Socket IO Connection failed");
        //
        // println!("Connected, now going to sleep... zzzz....");
        //
        // sleep(time::Duration::from_secs(20));
        //
        // socket.disconnect().await.expect("Disconnect failed");
        //
        // Ok(ProjectInfo {
        //     root_folder_id: "fake_id".to_owned(),
        // })
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
