use std::sync::Arc;

use hivemq_openapi::apis::configuration::Configuration;
use hivemq_openapi::apis::data_hub_data_policies_api::{
    get_all_data_policies, CreateDataPolicyParams, DeleteDataPolicyParams,
    GetAllDataPoliciesParams, UpdateDataPolicyParams,
};
use hivemq_openapi::models::DataPolicy;

use crate::hivemq_rest_client;
use crate::hivemq_rest_client::{get_cursor, transform_api_err};
use crate::repository::Repository;

pub struct DataPolicyService {
    repository: Arc<Repository<DataPolicy>>,
    config: Configuration,
}

impl DataPolicyService {
    pub fn new(repository: Arc<Repository<DataPolicy>>, host: &str) -> Self {
        let config = hivemq_rest_client::build_rest_api_config(host.to_string());
        DataPolicyService { repository, config }
    }

    pub async fn load_data_policies(&self) -> Result<(), String> {
        let mut params = GetAllDataPoliciesParams {
            fields: None,
            policy_ids: None,
            schema_ids: None,
            topic: None,
            limit: Some(500),
            cursor: None,
        };

        loop {
            let response = get_all_data_policies(&self.config, params.clone())
                .await
                .map_err(transform_api_err)?;

            for data_policy in response.items.into_iter().flatten() {
                self.repository.save(&data_policy).unwrap();
            }

            let cursor = get_cursor(response._links);
            if cursor.is_none() {
                return Ok(());
            } else {
                params.cursor = cursor;
            }
        }
    }

    pub async fn create_data_policy(&self, data_policy: &String) -> Result<String, String> {
        let data_policy: DataPolicy =
            serde_json::from_str(data_policy.as_str()).or_else(|err| Err(err.to_string()))?;
        let data_policy_id = data_policy.id.clone();

        let params = CreateDataPolicyParams { data_policy };

        let response = hivemq_openapi::apis::data_hub_data_policies_api::create_data_policy(
            &self.config,
            params,
        )
        .await
        .map_err(transform_api_err)?;

        self.repository.save(&response).unwrap();

        Ok(data_policy_id)
    }

    pub async fn update_data_policy(&self, data_policy: &String) -> Result<String, String> {
        let data_policy: DataPolicy =
            serde_json::from_str(data_policy.as_str()).or_else(|err| Err(err.to_string()))?;
        let policy_id = data_policy.id.clone();

        let params = UpdateDataPolicyParams {
            policy_id: policy_id.clone(),
            data_policy,
        };

        let response = hivemq_openapi::apis::data_hub_data_policies_api::update_data_policy(
            &self.config,
            params,
        )
        .await
        .map_err(transform_api_err)?;

        self.repository.save(&response).unwrap();

        Ok(policy_id)
    }

    pub async fn delete_data_policy(&self, data_policy_id: &str) -> Result<String, String> {
        let params = DeleteDataPolicyParams {
            policy_id: data_policy_id.to_owned(),
        };

        hivemq_openapi::apis::data_hub_data_policies_api::delete_data_policy(&self.config, params)
            .await
            .map_err(transform_api_err)?;

        self.repository.delete_by_id(&data_policy_id).unwrap();

        Ok(data_policy_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hivemq_openapi::apis::data_hub_data_policies_api::{
        CreateDataPolicyError, DeleteDataPolicyError, GetAllDataPoliciesError,
        UpdateDataPolicyError,
    };
    use hivemq_openapi::models::{
        DataPolicy, DataPolicyList, DataPolicyMatching, Errors, PaginationCursor,
    };
    use httpmock::Method::{DELETE, POST, PUT};
    use httpmock::MockServer;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use serde_json::json;

    use crate::repository::Repository;
    use crate::services::data_policy_service::DataPolicyService;

    fn build_data_policy(data_policy_num: usize) -> DataPolicy {
        DataPolicy::new(
            format!("data_policy-{data_policy_num}"),
            DataPolicyMatching::new(format!("my_topic-{data_policy_num}")),
        )
    }

    fn build_data_policy_list(
        start: usize,
        end: usize,
        cursor: Option<Option<Box<PaginationCursor>>>,
    ) -> DataPolicyList {
        let data_policies: Vec<DataPolicy> = (start..end).map(|i| build_data_policy(i)).collect();
        DataPolicyList {
            _links: cursor,
            items: Some(data_policies),
        }
    }

    fn setup() -> (
        MockServer,
        Pool<SqliteConnectionManager>,
        Arc<Repository<DataPolicy>>,
        DataPolicyService,
    ) {
        let broker = MockServer::start();
        let connection_pool = Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo = Repository::<DataPolicy>::init(
            &connection_pool,
            "data_policies",
            |data_policy| data_policy.id.clone(),
            "lastUpdatedAt",
        )
        .unwrap();
        let repo = Arc::new(repo);
        let service = DataPolicyService::new(repo.clone(), &broker.base_url());
        (broker, connection_pool, repo, service)
    }

    #[tokio::test]
    async fn test_load_data_policies() {
        let (broker, _pool, repo, service) = setup();

        let responses = crate::hivemq_rest_client::tests::create_responses(
            "/api/v1/data-hub/data-validation/policies",
            build_data_policy_list,
        );
        let _mocks = crate::hivemq_rest_client::tests::mock_cursor_responses(
            &broker,
            "/api/v1/data-hub/data-validation/policies",
            &responses,
            "foobar",
        );

        service.load_data_policies().await.unwrap();

        let mut created_data_policies = Vec::new();
        for mut data_policy_list in responses {
            data_policy_list
                .items
                .iter_mut()
                .for_each(|data_policies| created_data_policies.append(data_policies))
        }

        assert_eq!(created_data_policies, repo.find_all().unwrap());
    }

    #[tokio::test]
    async fn test_load_data_policies_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = GetAllDataPoliciesError::Status503(Errors::new());
        broker.mock(|when, then| {
            when.any_request();
            then.status(503).json_body(json!(error));
        });

        let response = service.load_data_policies().await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_create_data_policy() {
        let (broker, _pool, repo, service) = setup();

        let data_policy = build_data_policy(1);
        let data_policy_json = serde_json::to_string(&data_policy).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(data_policy_json.clone());
        });

        let _response = service.create_data_policy(&data_policy_json).await.unwrap();

        assert_eq!(data_policy, repo.find_by_id(&data_policy.id).unwrap());
    }

    #[tokio::test]
    async fn test_create_data_policy_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = CreateDataPolicyError::Status503(Errors::new());
        let data_policy = build_data_policy(1);
        let data_policy_json = serde_json::to_string(&data_policy).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.create_data_policy(&data_policy_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_delete_data_policy() {
        let (broker, _pool, repo, service) = setup();

        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(200);
        });

        let policy = build_data_policy(1);
        repo.save(&policy).unwrap();

        let _response = service.delete_data_policy("data_policy-1").await;

        assert!(repo.find_by_id("data_policy-1").is_err());
    }

    #[tokio::test]
    async fn test_update_data_policy() {
        let (broker, _pool, repo, service) = setup();

        let policy = build_data_policy(1);
        let policy_json = serde_json::to_string(&policy).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(PUT);
            then.status(200).body(policy_json.clone());
        });

        service.update_data_policy(&policy_json).await.unwrap();

        assert_eq!(policy, repo.find_by_id(&policy.id.clone()).unwrap());
    }

    #[tokio::test]
    async fn test_update_data_policy_error() {
        let (broker, _pool, _repo, service) = setup();

        let policy = build_data_policy(1);
        let error = UpdateDataPolicyError::Status503(Errors::new());
        let policy_json = serde_json::to_string(&policy).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(PUT);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let result = service.update_data_policy(&policy_json).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_data_policy_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = DeleteDataPolicyError::Status404(Errors::new());
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.delete_data_policy("data_policy-1").await;
        assert!(response.is_err())
    }
}
