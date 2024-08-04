use anyhow::{anyhow, bail, Context, Result};
use bytes::Bytes;
use chrono::{TimeZone, Utc};
use headless_chrome::protocol::cdp::{types::JsFloat, Network::Cookie};
use reqwest::{
    header::{HeaderMap, HeaderValue, COOKIE},
    multipart, Client,
};
use serde::{Deserialize, Serialize};
use soup::prelude::*;
use std::process::Command;

use crate::{
    constants::{DOWNLOAD_PROJECT_URL, PROJECTS_URL, UPLOAD_FILE_URL},
    custom_log::OlSpinner,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
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

    pub fn expiry_date_pretty(&self) -> String {
        // Optionally convert to DateTime<Utc> for time zone handling
        // let datetime_utc = Utc.timestamp(unix_timestamp, 0);

        // Print the results
        // println!("Naive DateTime (UTC): {}", naive_datetime);
        // println!("DateTime<Utc>: {}", datetime_utc);

        // Formatting example
        // println!("Formatted DateTime: {}", datetime_utc.format("%Y-%m-%d %H:%M:%S"));
        Utc.timestamp_opt(self.expires as i64, 0)
            .unwrap()
            .to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SessionInfo {
    pub email: String,
    pub session_cookie: OlCookie,
    pub gclb_cookie: OlCookie,
    pub csrf_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize)]
pub struct RootFolder {
    #[serde(rename = "_id")]
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetails {
    pub root_folder: Vec<RootFolder>,
}

pub struct OverleafClient {
    session_info: SessionInfo,
    reqwest_client: Client,
}

impl OverleafClient {
    pub fn new(session_info: SessionInfo) -> Result<Self> {
        let mut headers = HeaderMap::new();

        headers.insert(
            COOKIE,
            HeaderValue::from_str(
                &[&session_info.session_cookie]
                    .map(|cookie| format!("{}={}", cookie.name, cookie.value))
                    .join(";"),
            )
            .context("Failed to build Cookie header for Overleaf client.")?,
        );

        headers.insert(
            "X-CSRF-TOKEN",
            HeaderValue::from_str(&session_info.csrf_token)
                .context("Failed to build X-CSRF-TOKEN header for Overleaf client.")?,
        );

        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build reqwest client.")?;

        Ok(Self {
            session_info,
            reqwest_client,
        })
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
            .context("Failed to retrieve list of projects. Please try again.")?;

        serde_json::from_str(projects_list_content.as_str()).map_err(|e| {
            anyhow!(format!(
                "Failed to deserialize projects list with error: {e}."
            ))
        })
    }

    // Fetch specified project by name.
    pub async fn get_project_by_name(&self, project_name: &String) -> Result<Project> {
        self.get_all_projects()
            .await?
            .projects
            .into_iter()
            .filter(|project| project.name == *project_name)
            .last()
            .context(format!("Project {project_name} not found."))
    }

    // Fetch specified project by id.
    pub async fn get_project_by_id(&self, project_id: &String) -> Result<Project> {
        self.get_all_projects()
            .await?
            .projects
            .into_iter()
            .filter(|project| project.id == *project_id)
            .last()
            .context(format!("Project with id {project_id} not found."))
    }

    // Fetch specified project info.
    pub fn get_project_details(&self, project_id: &String) -> Result<ProjectDetails> {
        let mut spinner = OlSpinner::new("Fetching project details...".to_owned());

        let details_result = {
            let details_closure = || {
                let output = String::from_utf8(
                    Command::new("olsync-rs-socketio-client")
                        .args([
                            self.session_info.gclb_cookie.value.as_str(),
                            self.session_info.session_cookie.value.as_str(),
                            project_id.as_str(),
                        ])
                        .output()
                        .context(format!(
                            "Failed to obtain project info for project {project_id}."
                        ))?
                        .stdout,
                )
                .context("Invalid UTF-8")?
                .replace("'", "\"")
                .replace("None", "null")
                .replace("True", "true")
                .replace("False", "false");

                serde_json::from_str(output.as_str()).map_err(|e| {
                    anyhow!(format!(
                        "Failed to deserialize project details with error: {e}."
                    ))
                })
            };

            details_closure()
        };

        if details_result.is_ok() {
            spinner.stop_with_success("Fetched project details.".to_owned());
        } else {
            spinner.stop_with_error("Failed to fetch project details.".to_owned());
        }

        details_result
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

    // Upload file to specified filed in remote project.
    pub async fn upload_file(
        &self,
        project_id: &str,
        folder_id: &String,
        file_name: String,
        file: Vec<u8>,
    ) -> Result<()> {
        let file_part = multipart::Part::bytes(file).file_name(file_name.clone());

        let form = multipart::Form::new()
            .part("name", multipart::Part::text(file_name.clone()))
            .part("qqfile", file_part);

        let res = self
            .reqwest_client
            .post(UPLOAD_FILE_URL.replace("{}", project_id))
            .query(&[("folder_id", folder_id)])
            .multipart(form)
            .send()
            .await?;

        if !res.status().is_success() {
            bail!(
                "Failed to upload file {file_name} with response:\n{}: {}.",
                res.status(),
                String::from_utf8(res.bytes().await?.to_vec())
                    .unwrap_or("Invalid UTF-8 response.".to_owned())
            )
        }

        Ok(())
    }
}
