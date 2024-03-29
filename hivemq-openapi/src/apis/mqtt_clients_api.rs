/*
 * HiveMQ REST API
 *
 *  # Introduction  HiveMQ's REST API provides endpoints for the following use cases: - Listing all MQTT Clients - Getting detailed information about a specific MQTT client - Listing all subscriptions for a specific MQTT client - Getting the connection status for a specific MQTT client - Creating and restoring a backup - Starting and stopping a trace recording - Downloading backups and trace recordings  ## API style HiveMQ's API is organized in a [RESTful](http://en.wikipedia.org/wiki/Representational_State_Transfer) fashion.  The API has predictable resource-oriented URLs that consume and return JSON with the content-type `application/json`. It uses standard HTTP response codes and verbs. Some endpoints do return files, those are using the content type `application/octet-stream` or `application/zip`.  The base URL is the Host and configured port of your HiveMQ instances. In most cases it makes sense to configure a reverse-proxy or load balancer to access HiveMQ's REST API.  ## Pagination Some endpoints support returning the results in a paginated fashion. In those cases a cursor can be returned that contains the relative URL for the next page. The desired page size can be specified by using the `limit` query parameter.  Example URL: `http://my-broker-host:8888/api/v1/mqtt/clients?limit=100`  Example Response: ``` {   \"items\": [     {       \"id\": \"client-id-1\"     },      ...      {       \"id\": \"client-id-99\"     }   ],   \"_links\": {     \"next\": \"/api/v1/mqtt/clients?cursor=a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=&limit=100\"   } } ``` To fetch the next page with more results, the URL `http://my-broker-host:8888/api/v1/mqtt/clients?cursor=a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=&limit=100` is called. If the value for `_links.next` is not present, then this is the last page and no further pages are available.  **Note**: If a generated REST API client is used the cursor value must be extracted from the `next URL` and then passed as the cursor in the API call for fetching the next page.  Steps to use pagination in a REST API client: 1. Returned next URL: ``` http://my-broker-host:8888/api/v1/mqtt/clients?cursor=a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=&limit=100 ```  2. Extract the cursor from the next URL: ``` a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A= ```  3. Use the cursor in the REST API client to fetch the next page: ``` restClient.mqttClientsGet(pageLimitForRequest, \"a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=\"); ```  ## Errors Conventional HTTP response codes are used to indicate the success or failure of an API request. Codes in the 2xx range generally indicate success. Codes in the 4xx range indicate an error that failed given the information provided (e.g., a required parameter was omitted). Codes in the 5xx range indicate an error on the server side.  For all errors a JSON response with additional details is returned in the format [Problem JSON](https://tools.ietf.org/html/rfc7807).  ## OpenAPI HiveMQ's REST API provides an OpenAPI 3.0 schema definition that can imported into popular API tooling (e.g. Postman) or can be used to generate client-code for multiple programming languages. 
 *
 * The version of the OpenAPI document: 4.23.0
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;

use crate::apis::ResponseContent;
use super::{Error, configuration};

/// struct for passing parameters to the method [`disconnect_client`]
#[derive(Clone, Debug)]
pub struct DisconnectClientParams {
    /// The MQTT client identifier.
    pub client_id: String,
    /// Whether to prevent the will message.
    pub prevent_will_message: Option<bool>
}

/// struct for passing parameters to the method [`get_all_mqtt_clients`]
#[derive(Clone, Debug)]
pub struct GetAllMqttClientsParams {
    /// Specifies the page size for the returned results. Has to be between 50 and 2500. Default page size is 500.
    pub limit: Option<i32>,
    /// The cursor that has been returned by the previous result page. Do not pass this parameter if you want to fetch the first page.
    pub cursor: Option<String>
}

/// struct for passing parameters to the method [`get_mqtt_client_connection_state`]
#[derive(Clone, Debug)]
pub struct GetMqttClientConnectionStateParams {
    /// The MQTT client identifier.
    pub client_id: String
}

/// struct for passing parameters to the method [`get_mqtt_client_details`]
#[derive(Clone, Debug)]
pub struct GetMqttClientDetailsParams {
    /// The MQTT client identifier.
    pub client_id: String
}

/// struct for passing parameters to the method [`get_subscriptions_for_mqtt_client`]
#[derive(Clone, Debug)]
pub struct GetSubscriptionsForMqttClientParams {
    /// The MQTT client identifier.
    pub client_id: String
}

/// struct for passing parameters to the method [`invalidate_client_session`]
#[derive(Clone, Debug)]
pub struct InvalidateClientSessionParams {
    /// The MQTT client identifier.
    pub client_id: String,
    /// Whether to prevent the will message.
    pub prevent_will_message: Option<bool>
}


/// struct for typed errors of method [`disconnect_client`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DisconnectClientError {
    Status400(crate::models::Errors),
    Status404(crate::models::Errors),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_all_mqtt_clients`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetAllMqttClientsError {
    Status400(crate::models::Errors),
    Status410(crate::models::Errors),
    Status503(crate::models::Errors),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_mqtt_client_connection_state`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetMqttClientConnectionStateError {
    Status400(crate::models::Errors),
    Status404(crate::models::Errors),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_mqtt_client_details`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetMqttClientDetailsError {
    Status400(crate::models::Errors),
    Status404(crate::models::Errors),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_subscriptions_for_mqtt_client`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetSubscriptionsForMqttClientError {
    Status400(crate::models::Errors),
    Status404(crate::models::Errors),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`invalidate_client_session`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InvalidateClientSessionError {
    Status400(crate::models::Errors),
    Status404(crate::models::Errors),
    UnknownValue(serde_json::Value),
}


/// Disconnects a specific client if its is currently connected.   If your client identifiers contain special characters, please make sure that the clientId is URL Encoded (a.k.a. percent-encoding, as in RFC 3986).
pub async fn disconnect_client(configuration: &configuration::Configuration, params: DisconnectClientParams) -> Result<(), Error<DisconnectClientError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let client_id = params.client_id;
    let prevent_will_message = params.prevent_will_message;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/mqtt/clients/{clientId}/connection", local_var_configuration.base_path, clientId=crate::apis::urlencode(client_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::DELETE, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = prevent_will_message {
        local_var_req_builder = local_var_req_builder.query(&[("preventWillMessage", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<DisconnectClientError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Lists all client sessions (online and offline) known to the whole HiveMQ cluster.  The result contains each client's client identifier. For more details about each client you can call the endpoints that have a clientId in their URL.  This endpoint uses pagination with a cursor. The results are not sorted in any way, no ordering of any kind is guaranteed.  This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.
pub async fn get_all_mqtt_clients(configuration: &configuration::Configuration, params: GetAllMqttClientsParams) -> Result<crate::models::ClientList, Error<GetAllMqttClientsError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let limit = params.limit;
    let cursor = params.cursor;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/mqtt/clients", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = limit {
        local_var_req_builder = local_var_req_builder.query(&[("limit", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = cursor {
        local_var_req_builder = local_var_req_builder.query(&[("cursor", &local_var_str.to_string())]);
    }
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
        let local_var_entity: Option<GetAllMqttClientsError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Returns the information if a specific client is currently connected.   If you are only interested in the connection status of a client prefer this endpoint over the the full client detail. If your client identifiers contain special characters, please make sure that the clientId is URL Encoded (a.k.a. percent-encoding, as in RFC 3986).
pub async fn get_mqtt_client_connection_state(configuration: &configuration::Configuration, params: GetMqttClientConnectionStateParams) -> Result<crate::models::ConnectionItem, Error<GetMqttClientConnectionStateError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let client_id = params.client_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/mqtt/clients/{clientId}/connection", local_var_configuration.base_path, clientId=crate::apis::urlencode(client_id));
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
        let local_var_entity: Option<GetMqttClientConnectionStateError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Returns detailed information for a specific client with it is current state.   Including all session and connection information. If your client identifiers contain special characters, please make sure that the clientId is URL Encoded (a.k.a. percent-encoding, as in RFC 3986).
pub async fn get_mqtt_client_details(configuration: &configuration::Configuration, params: GetMqttClientDetailsParams) -> Result<crate::models::ClientItem, Error<GetMqttClientDetailsError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let client_id = params.client_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/mqtt/clients/{clientId}", local_var_configuration.base_path, clientId=crate::apis::urlencode(client_id));
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
        let local_var_entity: Option<GetMqttClientDetailsError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// List all subscriptions for a specific client.  This endpoint does not support pagination with cursor at the moment, but it might be added in future versions. Please make sure to check if a cursor is returned and another page is available to have a future-proof implementation.
pub async fn get_subscriptions_for_mqtt_client(configuration: &configuration::Configuration, params: GetSubscriptionsForMqttClientParams) -> Result<crate::models::ClientSubscriptionList, Error<GetSubscriptionsForMqttClientError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let client_id = params.client_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/mqtt/clients/{clientId}/subscriptions", local_var_configuration.base_path, clientId=crate::apis::urlencode(client_id));
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
        let local_var_entity: Option<GetSubscriptionsForMqttClientError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Invalidates the client session for a client with the given client identifier. If the client is currently connected, it will be disconnected as well.   If your client identifiers contain special characters, please make sure that the clientId is URL encoded (a.k.a. percent-encoding, as in RFC 3986).
pub async fn invalidate_client_session(configuration: &configuration::Configuration, params: InvalidateClientSessionParams) -> Result<(), Error<InvalidateClientSessionError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let client_id = params.client_id;
    let prevent_will_message = params.prevent_will_message;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/mqtt/clients/{clientId}", local_var_configuration.base_path, clientId=crate::apis::urlencode(client_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::DELETE, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = prevent_will_message {
        local_var_req_builder = local_var_req_builder.query(&[("preventWillMessage", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<InvalidateClientSessionError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

