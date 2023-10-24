/*
 * HiveMQ REST API
 *
 *  # Introduction  HiveMQ's REST API provides endpoints for the following use cases: - Listing all MQTT Clients - Getting detailed information about a specific MQTT client - Listing all subscriptions for a specific MQTT client - Getting the connection status for a specific MQTT client - Creating and restoring a backup - Starting and stopping a trace recording - Downloading backups and trace recordings  ## API style HiveMQ's API is organized in a [RESTful](http://en.wikipedia.org/wiki/Representational_State_Transfer) fashion.  The API has predictable resource-oriented URLs that consume and return JSON with the content-type `application/json`. It uses standard HTTP response codes and verbs. Some endpoints do return files, those are using the content type `application/octet-stream` or `application/zip`.  The base URL is the Host and configured port of your HiveMQ instances. In most cases it makes sense to configure a reverse-proxy or load balancer to access HiveMQ's REST API.  ## Pagination Some endpoints support returning the results in a paginated fashion. In those cases a cursor can be returned that contains the relative URL for the next page. The desired page size can be specified by using the `limit` query parameter.  Example URL: `http://my-broker-host:8888/api/v1/mqtt/clients?limit=100`  Example Response: ``` {   \"items\": [     {       \"id\": \"client-id-1\"     },      ...      {       \"id\": \"client-id-99\"     }   ],   \"_links\": {     \"next\": \"/api/v1/mqtt/clients?cursor=a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=&limit=100\"   } } ``` To fetch the next page with more results, the URL `http://my-broker-host:8888/api/v1/mqtt/clients?cursor=a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=&limit=100` is called. If the value for `_links.next` is not present, then this is the last page and no further pages are available.  **Note**: If a generated REST API client is used the cursor value must be extracted from the `next URL` and then passed as the cursor in the API call for fetching the next page.  Steps to use pagination in a REST API client: 1. Returned next URL: ``` http://my-broker-host:8888/api/v1/mqtt/clients?cursor=a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=&limit=100 ```  2. Extract the cursor from the next URL: ``` a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A= ```  3. Use the cursor in the REST API client to fetch the next page: ``` restClient.mqttClientsGet(pageLimitForRequest, \"a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=\"); ```  ## Errors Conventional HTTP response codes are used to indicate the success or failure of an API request. Codes in the 2xx range generally indicate success. Codes in the 4xx range indicate an error that failed given the information provided (e.g., a required parameter was omitted). Codes in the 5xx range indicate an error on the server side.  For all errors a JSON response with additional details is returned in the format [Problem JSON](https://tools.ietf.org/html/rfc7807).  ## OpenAPI HiveMQ's REST API provides an OpenAPI 3.0 schema definition that can imported into popular API tooling (e.g. Postman) or can be used to generate client-code for multiple programming languages. 
 *
 * The version of the OpenAPI document: 4.21.0
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;

use crate::apis::ResponseContent;
use super::{Error, configuration};

/// struct for passing parameters to the method [`download_backup_file`]
#[derive(Clone, Debug)]
pub struct DownloadBackupFileParams {
    /// The id of the backup.
    pub backup_id: String
}

/// struct for passing parameters to the method [`get_backup`]
#[derive(Clone, Debug)]
pub struct GetBackupParams {
    /// The id of the backup.
    pub backup_id: String
}

/// struct for passing parameters to the method [`restore_backup`]
#[derive(Clone, Debug)]
pub struct RestoreBackupParams {
    /// The id of the backup.
    pub backup_id: String
}


/// struct for typed errors of method [`create_backup`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateBackupError {
    Status503(crate::models::Errors),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`download_backup_file`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DownloadBackupFileError {
    Status400(crate::models::Errors),
    Status404(crate::models::Errors),
    Status503(crate::models::Errors),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_all_backups`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetAllBackupsError {
    Status503(crate::models::Errors),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_backup`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetBackupError {
    Status400(crate::models::Errors),
    Status404(crate::models::Errors),
    Status503(crate::models::Errors),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`restore_backup`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RestoreBackupError {
    Status400(crate::models::Errors),
    Status404(crate::models::Errors),
    Status503(crate::models::Errors),
    UnknownValue(serde_json::Value),
}


/// Triggers the creation of a new backup.  This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.
pub async fn create_backup(configuration: &configuration::Configuration) -> Result<crate::models::BackupItem, Error<CreateBackupError>> {
    let local_var_configuration = configuration;

    // unbox the parameters


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/management/backups", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<CreateBackupError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Download a specific backup file.    This endpoint returns the content of the backup file with the content-type `application/octet-stream`.    Only backups in the states `COMPLETED`, `RESTORE_IN_PROGRESS`, `RESTORE_FAILED` or `RESTORE_COMPLETED` can be downloaded.   This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.
pub async fn download_backup_file(configuration: &configuration::Configuration, params: DownloadBackupFileParams) -> Result<std::path::PathBuf, Error<DownloadBackupFileError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let backup_id = params.backup_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/management/files/backups/{backupId}", local_var_configuration.base_path, backupId=crate::apis::urlencode(backup_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<DownloadBackupFileError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Lists all available backups with their current state.  This endpoint can be used to get an overview over all backups that are in progress or can be restored.  Canceled or failed backups are included in the results for up to 1 hour after they have been requested.  This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.
pub async fn get_all_backups(configuration: &configuration::Configuration) -> Result<crate::models::BackupList, Error<GetAllBackupsError>> {
    let local_var_configuration = configuration;

    // unbox the parameters


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/management/backups", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<GetAllBackupsError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Returns the information for a specific backup with its current state.   This endpoint can be used to check the progress of a specific backup when it is being created or being restored.    Canceled or failed backups are returned for up to 1 hour after the have been requested.   This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.
pub async fn get_backup(configuration: &configuration::Configuration, params: GetBackupParams) -> Result<crate::models::BackupItem, Error<GetBackupError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let backup_id = params.backup_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/management/backups/{backupId}", local_var_configuration.base_path, backupId=crate::apis::urlencode(backup_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<GetBackupError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Triggers the restore of a stored backup.  This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.
pub async fn restore_backup(configuration: &configuration::Configuration, params: RestoreBackupParams) -> Result<crate::models::BackupItem, Error<RestoreBackupError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let backup_id = params.backup_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/management/backups/{backupId}", local_var_configuration.base_path, backupId=crate::apis::urlencode(backup_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<RestoreBackupError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

