use hivemq_openapi::apis::mqtt_clients_api;
use hivemq_openapi::apis::configuration::Configuration;
use hivemq_openapi::apis::data_hub_data_policies_api::{get_all_data_policies, GetAllDataPoliciesParams};
use hivemq_openapi::apis::mqtt_clients_api::{DisconnectClientParams, get_all_mqtt_clients, GetAllMqttClientsParams, GetMqttClientDetailsParams};
use hivemq_openapi::models::{ClientDetails, DataPolicy, PaginationCursor};
use mqtt_clients_api::get_mqtt_client_details;

pub async fn fetch_client_details(client_id: String, host: String) -> Result<ClientDetails, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let params = GetMqttClientDetailsParams {
        client_id: client_id.clone()
    };

    let details = get_mqtt_client_details(&configuration, params)
        .await
        .or_else(|error| Err(format!("Failed to fetch client details for client {client_id}: {error}")))?;

    let details = details.client.expect(format!("Client details for client {client_id} were empty").as_str());
    Ok(*details)
}

pub async fn fetch_client_ids(host: String) -> Result<Vec<String>, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;
    let mut client_ids = vec![];

    let mut params = GetAllMqttClientsParams {
        limit: Some(2_500),
        cursor: None,
    };

    loop {
        let response = get_all_mqtt_clients(&configuration, params.clone())
            .await
            .or_else(|error| Err(format!("Failed to fetch clients: {error}")))?;

        for client in response.items.unwrap() {
            client_ids.push(client.id.unwrap())
        }

        let cursor = match response._links {
            None => {
                break;
            }
            Some(cursor) => {
                cursor.unwrap().next
            }
        };
        params.cursor = cursor;
    }

    Ok(client_ids)
}

//TODO: Test
pub async fn fetch_policies(host: String) -> Result<Vec<(String, DataPolicy)>, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let mut params = GetAllDataPoliciesParams {
        fields: None,
        policy_ids: None,
        schema_ids: None,
        topic: None,
        limit: Some(2_500),
        cursor: None,
    };

    let mut policies = vec![];
    loop {
        let response = get_all_data_policies(&configuration, params.clone())
            .await
            .or_else(|error| Err(format!("Failed to fetch data policies: {error}")))?;

        for policy in response.items.unwrap() {
            policies.push((policy.id.clone(), policy));
        }

        let cursor = match response._links {
            None => {
                break;
            }
            Some(cursor) => {
                cursor.unwrap().next
            }
        };
        params.cursor = cursor;
    }

    Ok(policies)
}

pub async fn disconnect(client_id: String, host: String) -> Result<(), String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let params = DisconnectClientParams {
        client_id: client_id.clone(),
        prevent_will_message: Some(false)
    };

    mqtt_clients_api::disconnect_client(&configuration, params)
        .await
        .or_else(|error| Err(format!("Failed to disconnect client '{client_id}': {error}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tracing_subscriber::fmt::format;
    use crate::hivemq_rest_client::{fetch_client_details, fetch_client_ids};

    #[tokio::test]
    async fn test_fetch_client_details_online_client() {
        let response = r#"
            {
              "client": {
                "id": "my-client",
                "connected": true,
                "sessionExpiryInterval": 15000,
                "connectedAt": "2020-07-20T14:59:50.580Z",
                "messageQueueSize": 0,
                "willPresent": false,
                "restrictions": {
                  "maxMessageSize": 268435460,
                  "maxQueueSize": 1000,
                  "queuedMessageStrategy": "DISCARD"
                },
                "connection": {
                  "keepAlive": 60,
                  "mqttVersion": "MQTTv5",
                  "connectedListenerId": "TCP Listener",
                  "connectedNodeId": "bRIG4",
                  "cleanStart": true,
                  "sourceIp": "127.0.0.1"
                }
              }
            }
        "#;

        let broker = MockServer::start();
        let client_details_mock = broker.mock(|when, then| {
            when.method(GET)
                .path("/api/v1/mqtt/clients/my-client");
            then.status(200)
                .header("content-type", "application/json")
                .body(response);
        });

        let client_details = fetch_client_details("my-client".to_string(), broker.base_url()).await;
        let client_details = client_details.unwrap();

        // assert top level client details
        assert_eq!("my-client", client_details.id.unwrap().as_str());
        assert_eq!(true, client_details.connected.unwrap());
        assert_eq!(15000, client_details.session_expiry_interval.unwrap().unwrap());
        assert_eq!("2020-07-20T14:59:50.580Z", client_details.connected_at.unwrap().unwrap());
        assert_eq!(0, client_details.message_queue_size.unwrap());
        assert_eq!(false, client_details.will_present.unwrap());

        // assert restrictions
        let restrictions = client_details.restrictions.unwrap().unwrap();
        assert_eq!(268435460, restrictions.max_message_size.unwrap().unwrap());
        assert_eq!(1000, restrictions.max_queue_size.unwrap().unwrap());
        assert_eq!("DISCARD", restrictions.queued_message_strategy.unwrap().unwrap());

        // assert connection
        let connection = client_details.connection.unwrap().unwrap();
        assert_eq!(60, connection.keep_alive.unwrap().unwrap());
        assert_eq!("MQTTv5", connection.mqtt_version.unwrap());
        assert_eq!("TCP Listener", connection.connected_listener_id.unwrap());
        assert_eq!("bRIG4", connection.connected_node_id.unwrap());
        assert_eq!(true, connection.clean_start.unwrap());
        assert_eq!("127.0.0.1", connection.source_ip.unwrap().unwrap());

        client_details_mock.assert_hits(1);
    }

    #[tokio::test]
    async fn test_fetch_client_details_error() {
        let response = r#"
            {
              "errors": [
                {
                  "title": "Required parameter missing",
                  "detail": "Required URL parameter 'parameterName' is missing"
                }
              ]
            }
        "#;

        let broker = MockServer::start();
        let client_details_mock = broker.mock(|when, then| {
            when.method(GET)
                .path("/api/v1/mqtt/clients/my-client");
            then.status(400)
                .header("content-type", "application/json")
                .body(response);
        });

        let client_details = fetch_client_details("my-client".to_string(), broker.base_url()).await;

        assert!(client_details.is_err());
        client_details.expect_err("Failed to fetch client details for client my-client: error in response: status code 400 Bad Request");
    }

    #[tokio::test]
    async fn test_fetch_client_ids() {
        let response = r#"
        {
            "items": [
            {
              "id": "client-12"
            },
            {
              "id": "client-5"
            },
            {
              "id": "client-32"
            },
            {
              "id": "my-client-id2"
            },
            {
              "id": "my-client-id"
            }
            ]
        }
        "#;

        let broker = MockServer::start();
        let clients_mock = broker.mock(|when, then| {
            when.method(GET)
                .path("/api/v1/mqtt/clients");
            then.status(200)
                .header("content-type", "application/json")
                .body(response);
        });

        let client_ids = fetch_client_ids(broker.base_url()).await;
        let client_ids = client_ids.unwrap();

        clients_mock.assert_hits(1);
        assert!(client_ids.contains(&"client-12".to_string()));
        assert!(client_ids.contains(&"client-5".to_string()));
        assert!(client_ids.contains(&"client-32".to_string()));
        assert!(client_ids.contains(&"my-client-id2".to_string()));
        assert!(client_ids.contains(&"my-client-id".to_string()));
    }

    #[tokio::test]
    async fn test_fetch_client_ids_with_cursor() {
        let response1 = r#"
        {
          "items": [
            {
              "id": "client-1"
            },
            {
              "id": "client-2"
            },
            {
              "id": "client-3"
            },
            {
              "id": "client-4"
            },
            {
              "id": "client-5"
            }
          ],
          "_links": {
            "next": "/api/v1/mqtt/clients?cursor=a-MvelExpd5y0SrXBxDhBvnGmohbpzwGDQFdUyOYWBACqs1TgI4-cUo-A=&limit=2500"
          }
        }
        "#;

        let response2 = r#"
        {
            "items": [
            {
              "id": "client-6"
            },
            {
              "id": "client-7"
            },
            {
              "id": "client-8"
            },
            {
              "id": "client-9"
            },
            {
              "id": "client-10"
            }
          ]
        }
        "#;

        let broker = MockServer::start();
        let clients_mock2 = broker.mock(|when, then| {
            when.method(GET)
                .query_param_exists("cursor")
                .path("/api/v1/mqtt/clients");
            then.status(200)
                .header("content-type", "application/json")
                .body(response2);
        });
        let clients_mock1 = broker.mock(|when, then| {
            when.method(GET)
                .path("/api/v1/mqtt/clients");
            then.status(200)
                .header("content-type", "application/json")
                .body(response1);
        });

        let client_ids = fetch_client_ids(broker.base_url()).await;
        let client_ids = client_ids.unwrap();

        clients_mock1.assert_hits(1);
        clients_mock2.assert_hits(1);
        for i in 1..=10 {
            let client_id = format! {"client-{i}"}.to_string();
            assert!(client_ids.contains(&client_id));
        }
    }
}