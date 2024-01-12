use futures::future::err;
use hivemq_openapi::apis::backup_restore_api::{get_all_backups, GetBackupParams};
use hivemq_openapi::apis::configuration::Configuration;
use hivemq_openapi::apis::data_hub_behavior_policies_api::{get_all_behavior_policies, CreateBehaviorPolicyParams, GetAllBehaviorPoliciesParams, DeleteBehaviorPolicyParams};
use hivemq_openapi::apis::data_hub_data_policies_api::{get_all_data_policies, CreateDataPolicyParams, GetAllDataPoliciesError, GetAllDataPoliciesParams, DeleteDataPolicyParams};
use hivemq_openapi::apis::data_hub_schemas_api::{
    get_all_schemas, CreateSchemaParams, GetAllSchemasParams, DeleteSchemaParams,
};
use hivemq_openapi::apis::data_hub_scripts_api::{get_all_scripts, CreateScriptParams, GetAllScriptsParams, DeleteScriptParams};
use hivemq_openapi::apis::mqtt_clients_api::{
    get_all_mqtt_clients, DisconnectClientParams, GetAllMqttClientsParams,
    GetMqttClientDetailsParams,
};
use hivemq_openapi::apis::trace_recordings_api::{DeleteTraceRecordingParams, get_all_trace_recordings};
use hivemq_openapi::apis::{mqtt_clients_api, Error};
use hivemq_openapi::models::{
    Backup, BehaviorPolicy, ClientDetails, DataPolicy, PaginationCursor, Schema, Script,
    TraceRecording,
};
use mqtt_clients_api::get_mqtt_client_details;
use serde::Serialize;
use crate::action::Item;
use crate::action::Item::{BackupItem, BehaviorPolicyItem, DataPolicyItem, SchemaItem, ScriptItem, TraceRecordingItem};

pub async fn fetch_client_details(
    client_id: String,
    host: String,
) -> Result<(String, ClientDetails), String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let params = GetMqttClientDetailsParams {
        client_id: client_id.clone(),
    };

    let details = get_mqtt_client_details(&configuration, params)
        .await
        .or_else(|error| {
            Err(format!(
                "Failed to fetch client details for client {client_id}: {error}"
            ))
        })?;

    let details = details
        .client
        .expect(format!("Client details for client {client_id} were empty").as_str());
    Ok((client_id, *details))
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
            Some(cursor) => cursor.unwrap().next,
        };
        params.cursor = cursor;
    }

    Ok(client_ids)
}

//TODO: Test
pub async fn fetch_data_policies(host: String) -> Result<Vec<(String, Item)>, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let mut params = GetAllDataPoliciesParams {
        fields: None,
        policy_ids: None,
        schema_ids: None,
        topic: None,
        limit: Some(500),
        cursor: None,
    };

    let mut policies = vec![];
    loop {
        let response = get_all_data_policies(&configuration, params.clone())
            .await
            .or_else(|error| Err(transform_api_err(&error)))?;

        for policy in response.items.unwrap() {
            policies.push((policy.id.clone(), DataPolicyItem(policy)));
        }

        let cursor = match response._links {
            None => {
                break;
            }
            Some(cursor) => cursor.unwrap().next,
        };
        params.cursor = cursor;
    }

    Ok(policies)
}

pub async fn create_data_policy(host: String, data_policy: String) -> Result<Item, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let data_policy: DataPolicy =
        serde_json::from_str(data_policy.as_str()).or_else(|err| Err(err.to_string()))?;

    let params = CreateDataPolicyParams { data_policy };

    let response = hivemq_openapi::apis::data_hub_data_policies_api::create_data_policy(
        &configuration,
        params,
    )
        .await
        .or_else(|error| Err(transform_api_err(&error)))?;

    Ok(DataPolicyItem(response))
}

pub async fn delete_data_policy(host: String, policy_id: String) -> Result<String, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let params = DeleteDataPolicyParams { policy_id: policy_id.clone() };

    let response =
        hivemq_openapi::apis::data_hub_data_policies_api::delete_data_policy(&configuration, params)
            .await
            .map(|_| policy_id)
            .or_else(|error| Err(transform_api_err(&error)))?;

    Ok(response)
}

//TODO: Tests
pub async fn fetch_behavior_policies(
    host: String,
) -> Result<Vec<(String, Item)>, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let mut params = GetAllBehaviorPoliciesParams {
        fields: None,
        policy_ids: None,
        client_ids: None,
        limit: Some(500),
        cursor: None,
    };

    let mut policies = vec![];
    loop {
        let response = get_all_behavior_policies(&configuration, params.clone())
            .await
            .or_else(|error| Err(transform_api_err(&error)))?;

        for policy in response.items.unwrap() {
            policies.push((policy.id.clone(), BehaviorPolicyItem(policy)));
        }

        let cursor = match response._links {
            None => {
                break;
            }
            Some(cursor) => cursor.unwrap().next,
        };
        params.cursor = cursor;
    }

    Ok(policies)
}

pub async fn create_behavior_policy(
    host: String,
    behavior_policy: String,
) -> Result<Item, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let behavior_policy: BehaviorPolicy =
        serde_json::from_str(behavior_policy.as_str()).or_else(|err| Err(err.to_string()))?;

    let params = CreateBehaviorPolicyParams { behavior_policy };

    let response = hivemq_openapi::apis::data_hub_behavior_policies_api::create_behavior_policy(
        &configuration,
        params,
    )
        .await
        .or_else(|error| Err(transform_api_err(&error)))?;


    Ok(Item::BehaviorPolicyItem(response))
}

pub async fn delete_behavior_policy(host: String, policy_id: String) -> Result<String, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let params = DeleteBehaviorPolicyParams { policy_id: policy_id.clone() };

    let response =
        hivemq_openapi::apis::data_hub_behavior_policies_api::delete_behavior_policy(&configuration, params)
            .await
            .map(|_| policy_id)
            .or_else(|error| Err(transform_api_err(&error)))?;

    Ok(response)
}

//TODO: Test | Refactor?
pub async fn fetch_schemas(host: String) -> Result<Vec<(String, Item)>, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let mut params = GetAllSchemasParams {
        fields: None,
        types: None,
        limit: Some(500),
        cursor: None,
        schema_ids: None,
    };

    let mut schemas = vec![];
    loop {
        let response = get_all_schemas(&configuration, params.clone())
            .await
            .or_else(|error| Err(transform_api_err(&error)))?;

        for schema in response.items.unwrap() {
            schemas.push((schema.id.clone(), Item::SchemaItem(schema)));
        }

        let cursor = match response._links {
            None => {
                break;
            }
            Some(cursor) => cursor.unwrap().next,
        };
        params.cursor = cursor;
    }

    Ok(schemas)
}

pub async fn create_schema(host: String, schema: String) -> Result<Item, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let schema: Schema =
        serde_json::from_str(schema.as_str()).or_else(|err| Err(err.to_string()))?;

    let params = CreateSchemaParams { schema };

    let response =
        hivemq_openapi::apis::data_hub_schemas_api::create_schema(&configuration, params)
            .await
            .or_else(|error| Err(transform_api_err(&error)))?;

    Ok(SchemaItem(response))
}

pub async fn delete_schema(host: String, schema_id: String) -> Result<String, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let params = DeleteSchemaParams { schema_id: schema_id.clone() };

    let response =
        hivemq_openapi::apis::data_hub_schemas_api::delete_schema(&configuration, params)
            .await
            .map(|_| schema_id)
            .or_else(|error| Err(transform_api_err(&error)))?;

    Ok(response)
}

pub async fn fetch_scripts(host: String) -> Result<Vec<(String, Item)>, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let mut params = GetAllScriptsParams {
        fields: None,
        function_types: None,
        limit: Some(500),
        cursor: None,
        script_ids: None,
    };

    let mut scripts = vec![];
    loop {
        let response = get_all_scripts(&configuration, params.clone())
            .await
            .or_else(|error| Err(transform_api_err(&error)))?;

        for script in response.items.unwrap() {
            scripts.push((script.id.clone(), ScriptItem(script)));
        }

        let cursor = match response._links {
            None => {
                break;
            }
            Some(cursor) => cursor.unwrap().next,
        };
        params.cursor = cursor;
    }

    Ok(scripts)
}

pub async fn create_script(host: String, script: String) -> Result<Item, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let script: Script =
        serde_json::from_str(script.as_str()).or_else(|err| Err(err.to_string()))?;

    let params = CreateScriptParams { script };

    let response =
        hivemq_openapi::apis::data_hub_scripts_api::create_script(&configuration, params)
            .await
            .or_else(|error| Err(transform_api_err(&error)))?;

    Ok(ScriptItem(response))
}

pub async fn delete_script(host: String, script_id: String) -> Result<String, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let params = DeleteScriptParams { script_id: script_id.clone() };

    let response =
        hivemq_openapi::apis::data_hub_scripts_api::delete_script(&configuration, params)
            .await
            .map(|_| script_id)
            .or_else(|error| Err(transform_api_err(&error)))?;

    Ok(response)
}

pub async fn fetch_backups(host: String) -> Result<Vec<(String, Item)>, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let mut backups = vec![];
    let response = get_all_backups(&configuration)
        .await
        .or_else(|error| Err(transform_api_err(&error)))?;

    for backup in response.items.unwrap() {
        backups.push((backup.id.clone().unwrap(), BackupItem(backup)));
    }

    Ok(backups)
}

pub async fn fetch_trace_recordings(host: String) -> Result<Vec<(String, Item)>, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let mut trace_recordings = vec![];
    let response = get_all_trace_recordings(&configuration)
        .await
        .or_else(|error| Err(transform_api_err(&error)))?;

    for trace_recording in response.items.unwrap() {
        trace_recordings.push((trace_recording.name.clone().unwrap(), TraceRecordingItem(trace_recording)));
    }

    Ok(trace_recordings)
}

pub async fn delete_trace_recording(host: String, trace_recording_id: String) -> Result<String, String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let params = DeleteTraceRecordingParams { trace_recording_id: trace_recording_id.clone() };

    let response =
        hivemq_openapi::apis::trace_recordings_api::delete_trace_recording(&configuration, params)
            .await
            .map(|_| trace_recording_id)
            .or_else(|error| Err(transform_api_err(&error)))?;

    Ok(response)
}

pub async fn disconnect(client_id: String, host: String) -> Result<(), String> {
    let mut configuration = Configuration::default();
    configuration.base_path = host;

    let params = DisconnectClientParams {
        client_id: client_id.clone(),
        prevent_will_message: Some(false),
    };

    mqtt_clients_api::disconnect_client(&configuration, params)
        .await
        .or_else(|error| {
            Err(format!(
                "Failed to disconnect client '{client_id}': {error}"
            ))
        })?;

    Ok(())
}

pub fn transform_api_err<T: Serialize>(error: &Error<T>) -> String {
    let msg = if let Error::ResponseError(response) = error {
        match &response.entity {
            None => response.content.clone(),
            Some(entity) => serde_json::to_string_pretty(entity).expect("Can not serialize entity"),
        }
    } else {
        error.to_string()
    };

    format!("API request failed: {}", msg)
}

#[cfg(test)]
mod tests {
    use crate::hivemq_rest_client::{fetch_client_details, fetch_client_ids};
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tracing_subscriber::fmt::format;

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
            when.method(GET).path("/api/v1/mqtt/clients/my-client");
            then.status(200)
                .header("content-type", "application/json")
                .body(response);
        });

        let client_details = fetch_client_details("my-client".to_string(), broker.base_url()).await;
        let client_details = client_details.unwrap();
        let client_details = client_details.1;

        // assert top level client details
        assert_eq!("my-client", client_details.id.unwrap().as_str());
        assert_eq!(true, client_details.connected.unwrap());
        assert_eq!(
            15000,
            client_details.session_expiry_interval.unwrap().unwrap()
        );
        assert_eq!(
            "2020-07-20T14:59:50.580Z",
            client_details.connected_at.unwrap().unwrap()
        );
        assert_eq!(0, client_details.message_queue_size.unwrap());
        assert_eq!(false, client_details.will_present.unwrap());

        // assert restrictions
        let restrictions = client_details.restrictions.unwrap().unwrap();
        assert_eq!(268435460, restrictions.max_message_size.unwrap().unwrap());
        assert_eq!(1000, restrictions.max_queue_size.unwrap().unwrap());
        assert_eq!(
            "DISCARD",
            restrictions.queued_message_strategy.unwrap().unwrap()
        );

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
            when.method(GET).path("/api/v1/mqtt/clients/my-client");
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
            when.method(GET).path("/api/v1/mqtt/clients");
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
            when.method(GET).path("/api/v1/mqtt/clients");
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
