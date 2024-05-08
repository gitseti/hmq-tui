use hivemq_openapi::{
    apis::{
        backup_restore_api::get_all_backups,
        configuration::Configuration,
        data_hub_behavior_policies_api::{
            CreateBehaviorPolicyParams, DeleteBehaviorPolicyParams, get_all_behavior_policies,
            GetAllBehaviorPoliciesParams,
        },
        data_hub_data_policies_api::{
            CreateDataPolicyParams, DeleteDataPolicyParams, get_all_data_policies,
            GetAllDataPoliciesParams,
        },
        data_hub_schemas_api::{
            CreateSchemaParams, DeleteSchemaParams, get_all_schemas, GetAllSchemasParams,
        },
        data_hub_scripts_api::{
            CreateScriptParams, DeleteScriptParams, get_all_scripts, GetAllScriptsParams,
        },
        Error,
        mqtt_clients_api,
        mqtt_clients_api::{
            get_all_mqtt_clients, GetAllMqttClientsParams, GetMqttClientDetailsParams,
        },
        trace_recordings_api::{DeleteTraceRecordingParams, get_all_trace_recordings},
    },
    models::{BehaviorPolicy, ClientDetails, DataPolicy, PaginationCursor, Schema, Script},
};
use hivemq_openapi::apis::data_hub_behavior_policies_api::UpdateBehaviorPolicyParams;
use hivemq_openapi::apis::data_hub_data_policies_api::UpdateDataPolicyParams;
use hivemq_openapi::apis::trace_recordings_api::CreateTraceRecordingParams;
use hivemq_openapi::models::TraceRecording;
use lazy_static::lazy_static;
use mqtt_clients_api::get_mqtt_client_details;
use regex::Regex;
use serde::Serialize;

use crate::action::{
    Item,
    Item::{
        BackupItem, BehaviorPolicyItem, DataPolicyItem, SchemaItem, ScriptItem, TraceRecordingItem,
    },
};

pub fn get_cursor(links: Option<Option<Box<PaginationCursor>>>) -> Option<String> {
    lazy_static! {
        static ref CURSOR_REGEX: Regex = Regex::new(r"cursor=([^&]*)").unwrap();
    }

    let next = links.flatten().and_then(|cursor| cursor.next)?;

    CURSOR_REGEX
        .captures_iter(&next)
        .next()
        .and_then(|cap| cap.get(1))
        .map(|mat| mat.as_str().to_string())
}

pub fn build_rest_api_config(host: String) -> Configuration {
    let mut configuration = Configuration::default();
    configuration.base_path = host.to_string();
    configuration
}

pub fn transform_api_err<T: Serialize>(error: Error<T>) -> String {
    let message = if let Error::ResponseError(response) = error {
        match &response.entity {
            None => response.content.clone(),
            Some(entity) => serde_json::to_string_pretty(entity).expect("Can not serialize entity"),
        }
    } else {
        error.to_string()
    };

    format!("API request failed: {}", message)
}

pub async fn fetch_client_details(
    client_id: &str,
    host: String,
) -> Result<(String, ClientDetails), String> {
    let configuration = build_rest_api_config(host);
    let params = GetMqttClientDetailsParams {
        client_id: client_id.to_string(),
    };

    let details = get_mqtt_client_details(&configuration, params)
        .await
        .map_err(transform_api_err)?
        .client
        .ok_or_else(|| format!("Client details for client {client_id} were empty"))?;

    Ok((client_id.to_string(), *details))
}

pub async fn fetch_client_ids(host: String) -> Result<Vec<String>, String> {
    let configuration = build_rest_api_config(host);
    let mut params = GetAllMqttClientsParams {
        limit: Some(2_500),
        cursor: None,
    };

    let mut client_ids = vec![];
    loop {
        let response = get_all_mqtt_clients(&configuration, params.clone())
            .await
            .map_err(transform_api_err)?;

        for client in response.items.into_iter().flatten() {
            client_ids.push(
                client
                    .id
                    .ok_or_else(|| String::from("Client id was empty"))?,
            )
        }

        let cursor = get_cursor(response._links);
        if cursor.is_none() {
            break;
        } else {
            params.cursor = cursor;
        }
    }

    Ok(client_ids)
}

pub async fn fetch_data_policies(host: String) -> Result<Vec<(String, Item)>, String> {
    let configuration = build_rest_api_config(host);
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
            .map_err(transform_api_err)?;

        for policy in response.items.into_iter().flatten() {
            policies.push((policy.id.clone(), DataPolicyItem(policy)))
        }

        let cursor = get_cursor(response._links);
        if cursor.is_none() {
            break;
        } else {
            params.cursor = cursor;
        }
    }

    Ok(policies)
}

pub async fn create_data_policy(host: String, data_policy: String) -> Result<Item, String> {
    let configuration = build_rest_api_config(host);

    let data_policy: DataPolicy =
        serde_json::from_str(data_policy.as_str()).or_else(|err| Err(err.to_string()))?;

    let params = CreateDataPolicyParams { data_policy };

    let response = hivemq_openapi::apis::data_hub_data_policies_api::create_data_policy(
        &configuration,
        params,
    )
    .await
    .map_err(transform_api_err)?;

    Ok(DataPolicyItem(response))
}

pub async fn update_data_policy(
    host: String,
    policy_id: String,
    data_policy: String,
) -> Result<Item, String> {
    let configuration = build_rest_api_config(host);

    let data_policy: DataPolicy =
        serde_json::from_str(data_policy.as_str()).or_else(|err| Err(err.to_string()))?;

    let params = UpdateDataPolicyParams {
        policy_id,
        data_policy,
    };

    let response = hivemq_openapi::apis::data_hub_data_policies_api::update_data_policy(
        &configuration,
        params,
    )
    .await
    .map_err(transform_api_err)?;

    Ok(DataPolicyItem(response))
}

pub async fn delete_data_policy(host: String, policy_id: String) -> Result<String, String> {
    let configuration = build_rest_api_config(host);

    let params = DeleteDataPolicyParams {
        policy_id: policy_id.clone(),
    };

    let response = hivemq_openapi::apis::data_hub_data_policies_api::delete_data_policy(
        &configuration,
        params,
    )
    .await
    .map(|_| policy_id)
    .map_err(transform_api_err)?;

    Ok(response)
}

pub async fn fetch_behavior_policies(host: String) -> Result<Vec<(String, Item)>, String> {
    let configuration = build_rest_api_config(host);

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
            .map_err(transform_api_err)?;

        for policy in response.items.into_iter().flatten() {
            policies.push((policy.id.clone(), BehaviorPolicyItem(policy)))
        }

        let cursor = get_cursor(response._links);
        if cursor.is_none() {
            break;
        } else {
            params.cursor = cursor;
        }
    }

    Ok(policies)
}

pub async fn create_behavior_policy(host: String, behavior_policy: String) -> Result<Item, String> {
    let configuration = build_rest_api_config(host);

    let behavior_policy: BehaviorPolicy =
        serde_json::from_str(behavior_policy.as_str()).or_else(|err| Err(err.to_string()))?;

    let params = CreateBehaviorPolicyParams { behavior_policy };

    let response = hivemq_openapi::apis::data_hub_behavior_policies_api::create_behavior_policy(
        &configuration,
        params,
    )
    .await
    .map_err(transform_api_err)?;

    Ok(Item::BehaviorPolicyItem(response))
}

pub async fn update_behavior_policy(
    host: String,
    policy_id: String,
    behavior_policy: String,
) -> Result<Item, String> {
    let configuration = build_rest_api_config(host);

    let behavior_policy: BehaviorPolicy =
        serde_json::from_str(behavior_policy.as_str()).or_else(|err| Err(err.to_string()))?;

    let params = UpdateBehaviorPolicyParams {
        policy_id,
        behavior_policy,
    };

    let response = hivemq_openapi::apis::data_hub_behavior_policies_api::update_behavior_policy(
        &configuration,
        params,
    )
    .await
    .map_err(transform_api_err)?;

    Ok(BehaviorPolicyItem(response))
}

pub async fn delete_behavior_policy(host: String, policy_id: String) -> Result<String, String> {
    let configuration = build_rest_api_config(host);

    let params = DeleteBehaviorPolicyParams {
        policy_id: policy_id.clone(),
    };

    let response = hivemq_openapi::apis::data_hub_behavior_policies_api::delete_behavior_policy(
        &configuration,
        params,
    )
    .await
    .map(|_| policy_id)
    .map_err(transform_api_err)?;

    Ok(response)
}

pub async fn fetch_schemas(host: String) -> Result<Vec<(String, Item)>, String> {
    let configuration = build_rest_api_config(host);
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
            .map_err(transform_api_err)?;

        for schema in response.items.into_iter().flatten() {
            schemas.push((schema.id.clone(), SchemaItem(schema)))
        }

        let cursor = get_cursor(response._links);
        if cursor.is_none() {
            break;
        } else {
            params.cursor = cursor;
        }
    }

    Ok(schemas)
}

pub async fn create_schema(host: String, schema: String) -> Result<Item, String> {
    let configuration = build_rest_api_config(host);

    let schema: Schema =
        serde_json::from_str(schema.as_str()).or_else(|err| Err(err.to_string()))?;

    let params = CreateSchemaParams { schema };

    let response =
        hivemq_openapi::apis::data_hub_schemas_api::create_schema(&configuration, params)
            .await
            .map_err(transform_api_err)?;

    Ok(SchemaItem(response))
}

pub async fn delete_schema(host: String, schema_id: String) -> Result<String, String> {
    let configuration = build_rest_api_config(host);

    let params = DeleteSchemaParams {
        schema_id: schema_id.clone(),
    };

    let response =
        hivemq_openapi::apis::data_hub_schemas_api::delete_schema(&configuration, params)
            .await
            .map(|_| schema_id)
            .map_err(transform_api_err)?;

    Ok(response)
}

pub async fn fetch_scripts(host: String) -> Result<Vec<(String, Item)>, String> {
    let configuration = build_rest_api_config(host);

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
            .map_err(transform_api_err)?;

        for script in response.items.into_iter().flatten() {
            scripts.push((script.id.clone(), ScriptItem(script)))
        }

        let cursor = get_cursor(response._links);
        if cursor.is_none() {
            break;
        } else {
            params.cursor = cursor;
        }
    }

    Ok(scripts)
}

pub async fn create_script(host: String, script: String) -> Result<Item, String> {
    let configuration = build_rest_api_config(host);

    let script: Script =
        serde_json::from_str(script.as_str()).or_else(|err| Err(err.to_string()))?;

    let params = CreateScriptParams { script };

    let response =
        hivemq_openapi::apis::data_hub_scripts_api::create_script(&configuration, params)
            .await
            .map_err(transform_api_err)?;

    Ok(ScriptItem(response))
}

pub async fn delete_script(host: String, script_id: String) -> Result<String, String> {
    let configuration = build_rest_api_config(host);

    let params = DeleteScriptParams {
        script_id: script_id.clone(),
    };

    let response =
        hivemq_openapi::apis::data_hub_scripts_api::delete_script(&configuration, params)
            .await
            .map(|_| script_id)
            .map_err(transform_api_err)?;

    Ok(response)
}

pub async fn fetch_backups(host: String) -> Result<Vec<(String, Item)>, String> {
    let configuration = build_rest_api_config(host);

    let mut backups = vec![];
    let response = get_all_backups(&configuration)
        .await
        .map_err(transform_api_err)?;

    for backup in response.items.into_iter().flatten() {
        if let Some(id) = &backup.id {
            backups.push((id.clone(), BackupItem(backup)));
        }
    }

    Ok(backups)
}

pub async fn start_backup(host: String) -> Result<Item, String> {
    let configuration = build_rest_api_config(host);

    let response = hivemq_openapi::apis::backup_restore_api::create_backup(&configuration)
        .await
        .map_err(transform_api_err)?;

    Ok(BackupItem(*response.backup.unwrap()))
}

pub async fn fetch_trace_recordings(host: String) -> Result<Vec<(String, Item)>, String> {
    let configuration = build_rest_api_config(host);

    let mut trace_recordings = vec![];
    let response = get_all_trace_recordings(&configuration)
        .await
        .map_err(transform_api_err)?;

    for trace_recording in response.items.into_iter().flatten() {
        if let Some(name) = &trace_recording.name {
            trace_recordings.push((name.clone(), TraceRecordingItem(trace_recording)));
        }
    }

    Ok(trace_recordings)
}

pub async fn create_trace_recording(host: String, trace_recording: String) -> Result<Item, String> {
    let configuration = build_rest_api_config(host);

    let trace_recording: TraceRecording =
        serde_json::from_str(trace_recording.as_str()).or_else(|err| Err(err.to_string()))?;
    let trace_recording_item = Some(hivemq_openapi::models::TraceRecordingItem::new(
        trace_recording,
    ));

    let params = CreateTraceRecordingParams {
        trace_recording_item,
    };

    let response =
        hivemq_openapi::apis::trace_recordings_api::create_trace_recording(&configuration, params)
            .await
            .map_err(transform_api_err)?;

    Ok(TraceRecordingItem(*response.trace_recording))
}

pub async fn delete_trace_recording(
    host: String,
    trace_recording_id: String,
) -> Result<String, String> {
    let configuration = build_rest_api_config(host);

    let params = DeleteTraceRecordingParams {
        trace_recording_id: trace_recording_id.clone(),
    };

    let response =
        hivemq_openapi::apis::trace_recordings_api::delete_trace_recording(&configuration, params)
            .await
            .map(|_| trace_recording_id)
            .map_err(transform_api_err)?;

    Ok(response)
}

#[cfg(test)]
pub(crate) mod tests {
    use hivemq_openapi::{
        apis::{
            backup_restore_api::GetAllBackupsError,
            data_hub_behavior_policies_api::{
                CreateBehaviorPolicyError, DeleteBehaviorPolicyError, GetAllBehaviorPoliciesError,
            },
            data_hub_data_policies_api::{
                CreateDataPolicyError, DeleteDataPolicyError, GetAllDataPoliciesError,
            }
            ,
            data_hub_scripts_api::{CreateScriptError, DeleteScriptError, GetAllScriptsError},
            mqtt_clients_api::GetMqttClientDetailsError,
            trace_recordings_api::{DeleteTraceRecordingError, GetAllTraceRecordingsError},
        },
        models::{
            Backup, BackupList, BehaviorPolicy, BehaviorPolicyBehavior, BehaviorPolicyList,
            BehaviorPolicyMatching, Client, ClientDetails, ClientItem, ClientList,
            ClientRestrictions, ConnectionDetails, DataPolicy, DataPolicyList, DataPolicyMatching,
            Error, Errors, PaginationCursor, Script, script::FunctionType,
            ScriptList, TraceRecording, TraceRecordingList,
        },
    };
    use hivemq_openapi::apis::backup_restore_api::CreateBackupError;
    use hivemq_openapi::apis::data_hub_behavior_policies_api::UpdateBehaviorPolicyError;
    use hivemq_openapi::apis::data_hub_data_policies_api::UpdateDataPolicyError;
    use hivemq_openapi::apis::trace_recordings_api::CreateTraceRecordingError;
    use httpmock::{
        Method::{DELETE, GET, POST},
        Mock, MockServer,
    };
    use httpmock::Method::PUT;
    use pretty_assertions::assert_eq;
    use serde::Serialize;
    use serde_json::{json, Value};

    use crate::components::{
        item_features::ItemSelector,
        tabs::{
            backups::BackupSelector, behavior_policies::BehaviorPolicySelector,
            data_policies::DataPolicySelector, scripts::ScriptSelector,
            trace_recordings::TraceRecordingSelector,
        },
    };

    use super::*;

    pub fn create_responses<T>(
        url: &str,
        build_list: fn(usize, usize, Option<Option<Box<PaginationCursor>>>) -> T,
    ) -> Vec<T> {
        let mut responses: Vec<T> = Vec::with_capacity(100);
        for i in 0..10 {
            let start = i * 10;
            let end = start + 10;
            let cursor = if i != 9 {
                Some(Some(Box::new(PaginationCursor {
                    next: Some(format!("{url}?cursor=foobar{}", i + 1)),
                })))
            } else {
                None
            };
            responses.push(build_list(start, end, cursor));
        }
        responses
    }

    pub fn mock_cursor_responses<'a, T: Serialize>(
        broker: &'a MockServer,
        url: &str,
        responses: &Vec<T>,
        cursor_prefix: &str,
    ) -> Vec<Mock<'a>> {
        let mut mocks = Vec::with_capacity(responses.len());
        for (i, response) in responses.iter().enumerate().rev() {
            let mock = broker.mock(|when, then| {
                if i == 0 {
                    when.method(GET).path(url);
                } else {
                    when.method(GET)
                        .path(url)
                        .query_param("cursor", format!("{cursor_prefix}{i}"));
                }
                then.status(200)
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&response).unwrap());
            });
            mocks.push(mock);
        }

        mocks
    }

    fn build_client_details() -> ClientDetails {
        ClientDetails {
            connected: Some(true),
            connected_at: Some(Some("2020-07-20T14:59:50.580Z".to_string())),
            connection: Some(Some(Box::new(ConnectionDetails {
                clean_start: Some(true),
                connected_listener_id: Some("TCP Listener".to_string()),
                connected_node_id: Some("node1".to_string()),
                keep_alive: Some(Some(60)),
                mqtt_version: Some("MQTTv5".to_string()),
                password: None,
                proxy_information: None,
                source_ip: Some(Some("127.0.0.1".to_string())),
                tls_information: None,
                username: None,
            }))),
            id: Some("client".to_string()),
            message_queue_size: Some(100),
            restrictions: Some(Some(Box::new(ClientRestrictions {
                max_message_size: Some(Some(268435460)),
                max_queue_size: Some(Some(1000)),
                queued_message_strategy: Some(Some("DISCARD".to_string())),
            }))),
            session_expiry_interval: Some(Some(60)),
            will_present: Some(false),
        }
    }

    fn build_client_items(offset: usize, amount: usize) -> Vec<Client> {
        let client_items: Vec<Client> = (offset..offset + amount)
            .into_iter()
            .map(|i| Client {
                id: Some(format!("client-{i}")),
            })
            .collect();
        client_items
    }

    #[tokio::test]
    async fn test_fetch_client_details_client() {
        let client_details = build_client_details();
        let client_item = ClientItem {
            client: Some(Box::new(client_details.clone())),
        };
        let client_json = serde_json::to_string(&client_item).unwrap();
        let broker = MockServer::start();
        let _client_details_mock = broker.mock(|when, then| {
            when.method(GET).path("/api/v1/mqtt/clients/client");
            then.status(200)
                .header("content-type", "application/json")
                .body(client_json);
        });

        let response = fetch_client_details("client", broker.base_url()).await;
        let (client_id, details) = response.unwrap();

        assert_eq!(client_details.id, Some(client_id));
        assert_eq!(client_details, details);
    }

    #[tokio::test]
    async fn test_fetch_client_details_error() {
        let error = GetMqttClientDetailsError::Status404(Errors {
            errors: Some(vec![Error {
                detail: Some(String::from("Client with id client was not found")),
                title: Some(String::from("Client not found")),
            }]),
        });
        let error_json = serde_json::to_string(&error).unwrap();
        let broker = MockServer::start();
        let _client_details_mock = broker.mock(|when, then| {
            when.method(GET).path("/api/v1/mqtt/clients/client");
            then.status(404)
                .header("content-type", "application/json")
                .body(error_json.clone());
        });

        let response = fetch_client_details("client", broker.base_url()).await;

        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_fetch_client_ids() {
        let client_items = build_client_items(0, 100);
        let client_list = ClientList {
            _links: None,
            items: Some(client_items.clone()),
        };
        let broker = MockServer::start();
        let _ = broker.mock(|when, then| {
            when.method(GET).path("/api/v1/mqtt/clients");
            then.status(200)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&client_list).unwrap());
        });

        let client_ids = fetch_client_ids(broker.base_url()).await.unwrap();

        for client in client_items {
            assert!(client_ids.contains(&client.id.unwrap()));
        }
    }

    #[tokio::test]
    async fn test_fetch_client_ids_cursor() {
        let mut all_client_items: Vec<Client> = Vec::with_capacity(1000);
        let mut responses: Vec<ClientList> = Vec::with_capacity(10);
        for i in 0..10 {
            let client_items: Vec<Client> = build_client_items(i * 100, 100);
            all_client_items.append(&mut client_items.clone());
            let next = if i == 9 {
                None
            } else {
                Some(Some(Box::new(PaginationCursor {
                    next: Some(format!(
                        "/api/v1/mqtt/clients?cursor=foobar{}&limit=2500",
                        i + 1
                    )),
                })))
            };
            let client_list = ClientList {
                _links: next,
                items: Some(client_items),
            };
            responses.push(client_list);
        }

        let broker = MockServer::start();
        let mocks = mock_cursor_responses(&broker, "/api/v1/mqtt/clients", &responses, "foobar");

        let client_ids = fetch_client_ids(broker.base_url()).await.unwrap();

        for mock in mocks {
            mock.assert();
        }
        for client in all_client_items {
            assert!(
                client_ids.contains(&client.id.clone().unwrap()),
                "Client {} was not contained in response",
                &client.id.unwrap()
            );
        }
    }

    fn build_data_policy(policy_num: usize) -> DataPolicy {
        DataPolicy::new(
            format!("policy-{policy_num}"),
            DataPolicyMatching::new(format!("topic{policy_num}")),
        )
    }

    fn build_data_policy_list(
        start: usize,
        end: usize,
        cursor: Option<Option<Box<PaginationCursor>>>,
    ) -> DataPolicyList {
        let policies: Vec<DataPolicy> = (start..end).map(|i| build_data_policy(i)).collect();
        DataPolicyList {
            _links: cursor,
            items: Some(policies),
        }
    }

    #[tokio::test]
    async fn test_fetch_data_policies() {
        let broker = MockServer::start();
        let responses = create_responses(
            "/api/v1/data-hub/data-validation/policies",
            build_data_policy_list,
        );
        let mocks = mock_cursor_responses(
            &broker,
            "/api/v1/data-hub/data-validation/policies",
            &responses,
            "foobar",
        );

        let response = fetch_data_policies(broker.base_url()).await.unwrap();
        let items: Vec<DataPolicy> = response
            .into_iter()
            .map(|(_id, item)| DataPolicySelector.select(item).unwrap())
            .collect();

        for mock in mocks {
            mock.assert();
        }
        for policy in responses.into_iter().flat_map(|list| list.items).flatten() {
            assert!(items.contains(&policy));
        }
    }

    #[tokio::test]
    async fn test_fetch_data_policies_error() {
        let error = GetAllDataPoliciesError::Status503(Errors::new());
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(GET);
            then.status(503).json_body(json!(error));
        });

        let response = fetch_data_policies(broker.base_url()).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_create_data_policy() {
        let policy = build_data_policy(1);
        let policy_json = serde_json::to_string(&policy).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(policy_json.clone());
        });

        let response = create_data_policy(broker.base_url(), policy_json).await;
        assert_eq!(
            policy,
            DataPolicySelector.select(response.unwrap()).unwrap()
        )
    }

    #[tokio::test]
    async fn test_create_data_policy_error() {
        let error = CreateDataPolicyError::Status503(Errors::new());
        let policy = build_data_policy(1);
        let policy_json = serde_json::to_string(&policy).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = create_data_policy(broker.base_url(), policy_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_update_data_policy() {
        let policy = build_data_policy(1);
        let policy_json = serde_json::to_string(&policy).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(PUT);
            then.status(200).body(policy_json.clone());
        });

        let response = update_data_policy(broker.base_url(), policy.id.clone(), policy_json).await;
        assert_eq!(
            policy,
            DataPolicySelector.select(response.unwrap()).unwrap()
        )
    }

    #[tokio::test]
    async fn test_update_data_policy_error() {
        let error = UpdateDataPolicyError::Status503(Errors::new());
        let policy = build_data_policy(1);
        let policy_json = serde_json::to_string(&policy).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(PUT);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = update_data_policy(broker.base_url(), policy.id.clone(), policy_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_delete_data_policy() {
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(200);
        });

        let response = delete_data_policy(broker.base_url(), String::from("policy-1")).await;
        assert_eq!(response.unwrap(), "policy-1");
    }

    #[tokio::test]
    async fn test_delete_data_policy_error() {
        let error = DeleteDataPolicyError::Status404(Errors::new());
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = delete_data_policy(broker.base_url(), String::from("policy-1")).await;
        assert!(response.is_err())
    }

    fn build_behavior_policy(policy_num: usize) -> BehaviorPolicy {
        BehaviorPolicy::new(
            BehaviorPolicyBehavior::new(String::from("foobar")),
            format!("policy-{policy_num}"),
            BehaviorPolicyMatching::new(String::from("foobar")),
        )
    }

    fn build_behavior_policy_list(
        start: usize,
        end: usize,
        cursor: Option<Option<Box<PaginationCursor>>>,
    ) -> BehaviorPolicyList {
        let policies: Vec<BehaviorPolicy> =
            (start..end).map(|i| build_behavior_policy(i)).collect();
        BehaviorPolicyList {
            _links: cursor,
            items: Some(policies),
        }
    }

    #[tokio::test]
    async fn test_fetch_behavior_policies() {
        let broker = MockServer::start();
        let responses = create_responses(
            "/api/v1/data-hub/behavior-validation/policies",
            build_behavior_policy_list,
        );
        let mocks = mock_cursor_responses(
            &broker,
            "/api/v1/data-hub/behavior-validation/policies",
            &responses,
            "foobar",
        );

        let response = fetch_behavior_policies(broker.base_url()).await.unwrap();
        let items: Vec<BehaviorPolicy> = response
            .into_iter()
            .map(|(_id, item)| BehaviorPolicySelector.select(item).unwrap())
            .collect();

        for mock in mocks {
            mock.assert();
        }
        for policy in responses.into_iter().flat_map(|list| list.items).flatten() {
            assert!(items.contains(&policy));
        }
    }

    #[tokio::test]
    async fn test_fetch_behavior_policies_error() {
        let error = GetAllBehaviorPoliciesError::Status503(Errors::new());
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request();
            then.status(503).json_body(json!(error));
        });

        let response = fetch_behavior_policies(broker.base_url()).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_create_behavior_policy() {
        let policy = build_behavior_policy(1);
        let policy_json = serde_json::to_string(&policy).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(policy_json.clone());
        });

        let response = create_behavior_policy(broker.base_url(), policy_json).await;
        assert_eq!(
            policy,
            BehaviorPolicySelector.select(response.unwrap()).unwrap()
        )
    }

    #[tokio::test]
    async fn test_create_behavior_policy_error() {
        let error = CreateBehaviorPolicyError::Status503(Errors::new());
        let policy = build_behavior_policy(1);
        let policy_json = serde_json::to_string(&policy).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = create_behavior_policy(broker.base_url(), policy_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_update_behavior_policy() {
        let policy = build_behavior_policy(1);
        let policy_json = serde_json::to_string(&policy).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(PUT);
            then.status(200).body(policy_json.clone());
        });

        let response =
            update_behavior_policy(broker.base_url(), policy.id.clone(), policy_json).await;
        assert_eq!(
            policy,
            BehaviorPolicySelector.select(response.unwrap()).unwrap()
        )
    }

    #[tokio::test]
    async fn test_update_behavior_policy_error() {
        let error = UpdateBehaviorPolicyError::Status503(Errors::new());
        let policy = build_behavior_policy(1);
        let policy_json = serde_json::to_string(&policy).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(PUT);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response =
            update_behavior_policy(broker.base_url(), policy.id.clone(), policy_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_delete_behavior_policy() {
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(200);
        });

        let response = delete_behavior_policy(broker.base_url(), String::from("policy-1")).await;
        assert_eq!(response.unwrap(), "policy-1");
    }

    #[tokio::test]
    async fn test_delete_behavior_policy_error() {
        let error = DeleteBehaviorPolicyError::Status404(Errors::new());
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = delete_behavior_policy(broker.base_url(), String::from("policy-1")).await;
        assert!(response.is_err())
    }

    fn build_script(script_num: usize) -> Script {
        Script::new(
            FunctionType::Transformation,
            format!("script-{script_num}"),
            String::from("function transform(publish, context) -> { return publish }"),
        )
    }

    fn build_script_list(
        start: usize,
        end: usize,
        cursor: Option<Option<Box<PaginationCursor>>>,
    ) -> ScriptList {
        let scripts: Vec<Script> = (start..end).map(|i| build_script(i)).collect();
        ScriptList {
            _links: cursor,
            items: Some(scripts),
        }
    }

    #[tokio::test]
    async fn test_fetch_scripts() {
        let broker = MockServer::start();
        let responses = create_responses("/api/v1/data-hub/scripts", build_script_list);
        let mocks =
            mock_cursor_responses(&broker, "/api/v1/data-hub/scripts", &responses, "foobar");

        let response = fetch_scripts(broker.base_url()).await.unwrap();
        let items: Vec<Script> = response
            .into_iter()
            .map(|(_id, item)| ScriptSelector.select(item).unwrap())
            .collect();

        for mock in mocks {
            mock.assert();
        }
        for script in responses.into_iter().flat_map(|list| list.items).flatten() {
            assert!(items.contains(&script));
        }
    }

    #[tokio::test]
    async fn test_fetch_scripts_error() {
        let error = GetAllScriptsError::Status503(Errors::new());
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request();
            then.status(503).json_body(json!(error));
        });

        let response = fetch_scripts(broker.base_url()).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_create_script() {
        let script = build_script(1);
        let script_json = serde_json::to_string(&script).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(script_json.clone());
        });

        let response = create_script(broker.base_url(), script_json).await;
        assert_eq!(script, ScriptSelector.select(response.unwrap()).unwrap())
    }

    #[tokio::test]
    async fn test_create_script_error() {
        let error = CreateScriptError::Status503(Errors::new());
        let script = build_script(1);
        let script_json = serde_json::to_string(&script).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = create_script(broker.base_url(), script_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_delete_script() {
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(200);
        });

        let response = delete_script(broker.base_url(), String::from("script-1")).await;
        assert_eq!(response.unwrap(), "script-1");
    }

    #[tokio::test]
    async fn test_delete_script_error() {
        let error = DeleteScriptError::Status404(Errors::new());
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = delete_script(broker.base_url(), String::from("script-1")).await;
        assert!(response.is_err())
    }

    fn build_backup(backup_num: usize) -> Backup {
        let mut backup = Backup::new();
        backup.id = Some(format!("backup-{backup_num}"));
        backup
    }

    fn build_backup_list(start: usize, end: usize) -> BackupList {
        let backups: Vec<Backup> = (start..end).map(|i| build_backup(i)).collect();
        BackupList {
            items: Some(backups),
        }
    }

    #[tokio::test]
    async fn test_fetch_backups() {
        let broker = MockServer::start();
        let backup_list = build_backup_list(0, 10);
        broker.mock(|when, then| {
            when.any_request().method(GET);
            then.status(200)
                .body(serde_json::to_string(&backup_list).unwrap());
        });

        let response = fetch_backups(broker.base_url()).await.unwrap();
        let items: Vec<Backup> = response
            .into_iter()
            .map(|(_id, item)| BackupSelector.select(item).unwrap())
            .collect();

        for backup in backup_list.items.into_iter().flatten() {
            assert!(items.contains(&backup));
        }
    }

    #[tokio::test]
    async fn test_fetch_backups_error() {
        let error = GetAllBackupsError::Status503(Errors::new());
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request();
            then.status(503).json_body(json!(error));
        });

        let response = fetch_backups(broker.base_url()).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_start_backup() {
        let backup = hivemq_openapi::models::BackupItem {
            backup: Some(Box::new(build_backup(1))),
        };
        let backup_json = serde_json::to_string(&backup).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(backup_json.clone());
        });

        let response = start_backup(broker.base_url()).await;
        assert_eq!(
            *backup.backup.unwrap(),
            BackupSelector.select(response.unwrap()).unwrap()
        )
    }

    #[tokio::test]
    async fn test_start_backup_error() {
        let error = CreateBackupError::Status503(Errors::new());
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = start_backup(broker.base_url()).await;
        assert!(response.is_err());
    }

    fn build_trace_recording(trace_recording_num: usize) -> TraceRecording {
        let mut trace_recording = TraceRecording::new();
        trace_recording.name = Some(format!("trace-recording-{trace_recording_num}"));
        trace_recording
    }

    fn build_trace_recording_list(start: usize, end: usize) -> TraceRecordingList {
        let trace_recordings: Vec<TraceRecording> =
            (start..end).map(|i| build_trace_recording(i)).collect();
        TraceRecordingList {
            items: Some(trace_recordings),
        }
    }

    #[tokio::test]
    async fn test_fetch_trace_recordings() {
        let broker = MockServer::start();
        let trace_recording_list = build_trace_recording_list(0, 10);
        broker.mock(|when, then| {
            when.any_request().method(GET);
            then.status(200)
                .body(serde_json::to_string(&trace_recording_list).unwrap());
        });

        let response = fetch_trace_recordings(broker.base_url()).await.unwrap();
        let items: Vec<TraceRecording> = response
            .into_iter()
            .map(|(_id, item)| TraceRecordingSelector.select(item).unwrap())
            .collect();

        for trace_recording in trace_recording_list.items.into_iter().flatten() {
            assert!(items.contains(&trace_recording));
        }
    }

    #[tokio::test]
    async fn test_fetch_trace_recordings_error() {
        let error = GetAllTraceRecordingsError::UnknownValue(Value::Null);
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request();
            then.status(503).json_body(json!(error));
        });

        let response = fetch_trace_recordings(broker.base_url()).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_create_trace_recording() {
        let trace_recording =
            hivemq_openapi::models::TraceRecordingItem::new(build_trace_recording(1));
        let trace_recording_json = serde_json::to_string(&trace_recording).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(trace_recording_json.clone());
        });

        let response = create_trace_recording(broker.base_url(), trace_recording_json).await;
        assert_eq!(
            *trace_recording.trace_recording,
            TraceRecordingSelector.select(response.unwrap()).unwrap()
        )
    }

    #[tokio::test]
    async fn test_create_trace_recording_error() {
        let error = CreateTraceRecordingError::Status400(Errors::new());
        let trace_recording = build_trace_recording(1);
        let trace_recording_json = serde_json::to_string(&trace_recording).unwrap();
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = create_trace_recording(broker.base_url(), trace_recording_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_delete_trace_recording() {
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(200);
        });

        let response =
            delete_trace_recording(broker.base_url(), String::from("trace-recording-1")).await;
        assert_eq!(response.unwrap(), "trace-recording-1");
    }

    #[tokio::test]
    async fn test_delete_trace_recording_error() {
        let error = DeleteTraceRecordingError::Status404(Errors::new());
        let broker = MockServer::start();
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response =
            delete_trace_recording(broker.base_url(), String::from("trace-recording-1")).await;
        assert!(response.is_err())
    }
}
