use crate::hivemq_rest_client;
use crate::repository::{Repository, RepositoryError};
use hivemq_openapi::apis::configuration::Configuration;
use hivemq_openapi::apis::mqtt_clients_api::{
    get_all_mqtt_clients, get_mqtt_client_details, GetAllMqttClientsParams,
    GetMqttClientDetailsParams,
};
use hivemq_openapi::models::ClientDetails;
use std::error::Error;
use std::sync::Arc;

pub struct ClientDetailsService {
    repository: Arc<Repository<ClientDetails>>,
    config: Configuration,
}

impl ClientDetailsService {
    pub fn new(repository: Arc<Repository<ClientDetails>>, host: &str) -> Self {
        let config = hivemq_rest_client::build_rest_api_config(host.to_string());
        ClientDetailsService { repository, config }
    }

    pub async fn load_details(&self) -> Result<(), String> {
        let client_ids = self.fetch_client_ids().await?;
        for client_id in client_ids {
            let (_, client_details) = self.fetch_client_details(&client_id).await?;
            self.repository
                .save(&client_details)
                .map_err(|err| match err {
                    RepositoryError::SerdeError(err) => err.to_string(),
                    RepositoryError::SqlError(err) => err.to_string(),
                })?;
        }
        Ok(())
    }


    async fn fetch_client_details(
        &self,
        client_id: &str,
    ) -> Result<(String, ClientDetails), String> {
        let params = GetMqttClientDetailsParams {
            client_id: client_id.to_string(),
        };

        let details = get_mqtt_client_details(&self.config, params)
            .await
            .map_err(hivemq_rest_client::transform_api_err)?
            .client
            .ok_or_else(|| format!("Client details for client {client_id} were empty"))?;

        Ok((client_id.to_string(), *details))
    }

    async fn fetch_client_ids(&self) -> Result<Vec<String>, String> {
        let mut params = GetAllMqttClientsParams {
            limit: Some(2_500),
            cursor: None,
        };

        let mut client_ids = vec![];
        loop {
            let response = get_all_mqtt_clients(&self.config, params.clone())
                .await
                .map_err(hivemq_rest_client::transform_api_err)?;

            for client in response.items.into_iter().flatten() {
                client_ids.push(
                    client
                        .id
                        .ok_or_else(|| String::from("Client id was empty"))?,
                )
            }

            let cursor = hivemq_rest_client::get_cursor(response._links);
            if cursor.is_none() {
                break;
            } else {
                params.cursor = cursor;
            }
        }

        Ok(client_ids)
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::Repository;
    use crate::services::client_details_service::ClientDetailsService;
    use hivemq_openapi::models::{
        Client, ClientDetails, ClientItem, ClientList, ClientRestrictions, ConnectionDetails,
    };
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use r2d2_sqlite::SqliteConnectionManager;
    use std::sync::Arc;

    fn build_client_details(client_id: &str) -> ClientDetails {
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
            id: Some(client_id.to_string()),
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
        let broker = MockServer::start();
        let connection_pool = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo =
            Repository::<ClientDetails>::init(&connection_pool, "test_values", |client_details| {
                client_details.id.clone().unwrap()
            })
            .unwrap();
        let repo = Arc::new(repo);
        let service = ClientDetailsService::new(repo.clone(), &broker.base_url());

        let client_items = build_client_items(0, 5);
        let client_list = ClientList {
            _links: None,
            items: Some(client_items.clone()),
        };

        let _ = broker.mock(|when, then| {
            when.method(GET).path("/api/v1/mqtt/clients");
            then.status(200)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&client_list).unwrap());
        });

        for client in client_items {
            let client_details = build_client_details(&client.id.clone().unwrap());
            let client_item = ClientItem {
                client: Some(Box::new(client_details.clone())),
            };
            let client_json = serde_json::to_string(&client_item).unwrap();
            let _ = broker.mock(|when, then| {
                when.method(GET).path(format!(
                    "/api/v1/mqtt/clients/{}",
                    &client.id.clone().unwrap()
                ));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(client_json);
            });
        }

        service.load_details().await.unwrap();

        let client_details = repo.find_all().unwrap();
        let client_ids: Vec<String> = client_details
            .iter()
            .map(|detail| detail.id.clone().unwrap())
            .collect();
        assert_eq!(
            vec!["client-0", "client-1", "client-2", "client-3", "client-4"],
            client_ids
        );
    }
}
