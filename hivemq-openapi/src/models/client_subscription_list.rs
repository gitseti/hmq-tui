/*
 * HiveMQ REST API
 *
 *  # Introduction  HiveMQ's REST API provides endpoints for the following use cases: - Listing all MQTT Clients - Getting detailed information about a specific MQTT client - Listing all subscriptions for a specific MQTT client - Getting the connection status for a specific MQTT client - Creating and restoring a backup - Starting and stopping a trace recording - Downloading backups and trace recordings  ## API style HiveMQ's API is organized in a [RESTful](http://en.wikipedia.org/wiki/Representational_State_Transfer) fashion.  The API has predictable resource-oriented URLs that consume and return JSON with the content-type `application/json`. It uses standard HTTP response codes and verbs. Some endpoints do return files, those are using the content type `application/octet-stream` or `application/zip`.  The base URL is the Host and configured port of your HiveMQ instances. In most cases it makes sense to configure a reverse-proxy or load balancer to access HiveMQ's REST API.  ## Pagination Some endpoints support returning the results in a paginated fashion. In those cases a cursor can be returned that contains the relative URL for the next page. The desired page size can be specified by using the `limit` query parameter.  Example URL: `http://my-broker-host:8888/api/v1/mqtt/clients?limit=100`  Example Response: ``` {   \"items\": [     {       \"id\": \"client-id-1\"     },      ...      {       \"id\": \"client-id-99\"     }   ],   \"_links\": {     \"next\": \"/api/v1/mqtt/clients?cursor=a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=&limit=100\"   } } ``` To fetch the next page with more results, the URL `http://my-broker-host:8888/api/v1/mqtt/clients?cursor=a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=&limit=100` is called. If the value for `_links.next` is not present, then this is the last page and no further pages are available.  **Note**: If a generated REST API client is used the cursor value must be extracted from the `next URL` and then passed as the cursor in the API call for fetching the next page.  Steps to use pagination in a REST API client: 1. Returned next URL: ``` http://my-broker-host:8888/api/v1/mqtt/clients?cursor=a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=&limit=100 ```  2. Extract the cursor from the next URL: ``` a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A= ```  3. Use the cursor in the REST API client to fetch the next page: ``` restClient.mqttClientsGet(pageLimitForRequest, \"a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=\"); ```  ## Errors Conventional HTTP response codes are used to indicate the success or failure of an API request. Codes in the 2xx range generally indicate success. Codes in the 4xx range indicate an error that failed given the information provided (e.g., a required parameter was omitted). Codes in the 5xx range indicate an error on the server side.  For all errors a JSON response with additional details is returned in the format [Problem JSON](https://tools.ietf.org/html/rfc7807).  ## OpenAPI HiveMQ's REST API provides an OpenAPI 3.0 schema definition that can imported into popular API tooling (e.g. Postman) or can be used to generate client-code for multiple programming languages. 
 *
 * The version of the OpenAPI document: 4.21.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientSubscriptionList {
    #[serde(rename = "_links", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub _links: Option<Option<Box<crate::models::PaginationCursor>>>,
    /// List of result items that are returned by this endpoint
    #[serde(rename = "items", skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<crate::models::ClientSubscription>>,
}

impl ClientSubscriptionList {
    pub fn new() -> ClientSubscriptionList {
        ClientSubscriptionList {
            _links: None,
            items: None,
        }
    }
}


