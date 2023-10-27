use hivemq_openapi::apis::mqtt_clients_api;
use hivemq_openapi::apis::configuration::Configuration;
use hivemq_openapi::apis::mqtt_clients_api::GetMqttClientDetailsParams;
use hivemq_openapi::models::ClientDetails;
use mqtt_clients_api::get_mqtt_client_details;

async fn fetch_client_details(client_id: String, host: String) -> Result<ClientDetails, String> {
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

#[cfg(test)]
mod tests {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use crate::hivemq_rest_client::fetch_client_details;

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
}