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

/// struct for passing parameters to the method [`get_client_state`]
#[derive(Clone, Debug)]
pub struct GetClientStateParams {
    /// The client identifier.
    pub client_id: String
}


/// struct for typed errors of method [`get_client_state`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetClientStateError {
    Status400(crate::models::Errors),
    Status404(crate::models::Errors),
    Status503(crate::models::Errors),
    UnknownValue(serde_json::Value),
}


/// Use this endpoint to get the stored state of a client for DataHub.  This endpoint requires at least HiveMQ version 4.20.0 on the REST API node.
pub async fn get_client_state(configuration: &configuration::Configuration, params: GetClientStateParams) -> Result<crate::models::FsmStatesInformationListItem, Error<GetClientStateError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let client_id = params.client_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/api/v1/data-hub/behavior-validation/states/{clientId}", local_var_configuration.base_path, clientId=crate::apis::urlencode(client_id));
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
        let local_var_entity: Option<GetClientStateError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

