use std::sync::Arc;

use hivemq_openapi::apis::configuration::Configuration;
use hivemq_openapi::apis::data_hub_behavior_policies_api::{
    get_all_behavior_policies, CreateBehaviorPolicyParams, DeleteBehaviorPolicyParams,
    GetAllBehaviorPoliciesParams, UpdateBehaviorPolicyParams,
};
use hivemq_openapi::models::BehaviorPolicy;

use crate::hivemq_rest_client;
use crate::hivemq_rest_client::{get_cursor, transform_api_err};
use crate::repository::Repository;

pub struct BehaviorPolicyService {
    repository: Arc<Repository<BehaviorPolicy>>,
    config: Configuration,
}

impl BehaviorPolicyService {
    pub fn new(repository: Arc<Repository<BehaviorPolicy>>, host: &str) -> Self {
        let config = hivemq_rest_client::build_rest_api_config(host.to_string());
        BehaviorPolicyService { repository, config }
    }

    pub async fn load_behavior_policies(&self) -> Result<(), String> {
        let mut params = GetAllBehaviorPoliciesParams {
            fields: None,
            policy_ids: None,
            limit: Some(500),
            cursor: None,
            client_ids: None,
        };

        loop {
            let response = get_all_behavior_policies(&self.config, params.clone())
                .await
                .map_err(transform_api_err)?;

            for behavior_policy in response.items.into_iter().flatten() {
                self.repository.save(&behavior_policy).unwrap();
            }

            let cursor = get_cursor(response._links);
            if cursor.is_none() {
                return Ok(());
            } else {
                params.cursor = cursor;
            }
        }
    }

    pub async fn create_behavior_policy(&self, behavior_policy: &String) -> Result<String, String> {
        let behavior_policy: BehaviorPolicy =
            serde_json::from_str(behavior_policy.as_str()).or_else(|err| Err(err.to_string()))?;
        let behavior_policy_id = behavior_policy.id.clone();

        let params = CreateBehaviorPolicyParams { behavior_policy };

        let response =
            hivemq_openapi::apis::data_hub_behavior_policies_api::create_behavior_policy(
                &self.config,
                params,
            )
            .await
            .map_err(transform_api_err)?;

        self.repository.save(&response).unwrap();

        Ok(behavior_policy_id)
    }

    pub async fn update_behavior_policy(&self, behavior_policy: &String) -> Result<String, String> {
        let behavior_policy: BehaviorPolicy =
            serde_json::from_str(behavior_policy.as_str()).or_else(|err| Err(err.to_string()))?;
        let policy_id = behavior_policy.id.clone();

        let params = UpdateBehaviorPolicyParams {
            policy_id: policy_id.clone(),
            behavior_policy,
        };

        let response =
            hivemq_openapi::apis::data_hub_behavior_policies_api::update_behavior_policy(
                &self.config,
                params,
            )
            .await
            .map_err(transform_api_err)?;

        self.repository.save(&response).unwrap();

        Ok(policy_id)
    }

    pub async fn delete_behavior_policy(&self, behavior_policy_id: &str) -> Result<String, String> {
        let params = DeleteBehaviorPolicyParams {
            policy_id: behavior_policy_id.to_owned(),
        };

        hivemq_openapi::apis::data_hub_behavior_policies_api::delete_behavior_policy(
            &self.config,
            params,
        )
        .await
        .map_err(transform_api_err)?;

        self.repository.delete_by_id(&behavior_policy_id).unwrap();

        Ok(behavior_policy_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hivemq_openapi::apis::data_hub_behavior_policies_api::{
        CreateBehaviorPolicyError, DeleteBehaviorPolicyError, GetAllBehaviorPoliciesError,
        UpdateBehaviorPolicyError,
    };
    use hivemq_openapi::models::{
        BehaviorPolicy, BehaviorPolicyBehavior, BehaviorPolicyList, BehaviorPolicyMatching, Errors,
        PaginationCursor,
    };
    use httpmock::Method::{DELETE, POST, PUT};
    use httpmock::MockServer;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use serde_json::json;

    use crate::repository::Repository;
    use crate::services::behavior_policy_service::BehaviorPolicyService;

    fn build_behavior_policy(behavior_policy_num: usize) -> BehaviorPolicy {
        BehaviorPolicy::new(
            BehaviorPolicyBehavior::new("Mqtt.Events".to_string()),
            format!("behavior_policy-{behavior_policy_num}"),
            BehaviorPolicyMatching::new(".*".to_string()),
        )
    }

    fn build_behavior_policy_list(
        start: usize,
        end: usize,
        cursor: Option<Option<Box<PaginationCursor>>>,
    ) -> BehaviorPolicyList {
        let behavior_policies: Vec<BehaviorPolicy> =
            (start..end).map(|i| build_behavior_policy(i)).collect();
        BehaviorPolicyList {
            _links: cursor,
            items: Some(behavior_policies),
        }
    }

    fn setup() -> (
        MockServer,
        Pool<SqliteConnectionManager>,
        Arc<Repository<BehaviorPolicy>>,
        BehaviorPolicyService,
    ) {
        let broker = MockServer::start();
        let connection_pool = Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo = Repository::<BehaviorPolicy>::init(
            &connection_pool,
            "behavior_policies",
            |behavior_policy| behavior_policy.id.clone(),
            "lastUpdatedAt"
        )
        .unwrap();
        let repo = Arc::new(repo);
        let service = BehaviorPolicyService::new(repo.clone(), &broker.base_url());
        (broker, connection_pool, repo, service)
    }

    #[tokio::test]
    async fn test_load_behavior_policies() {
        let (broker, _pool, repo, service) = setup();

        let responses = crate::hivemq_rest_client::tests::create_responses(
            "/api/v1/data-hub/behavior-validation/policies",
            build_behavior_policy_list,
        );
        let _mocks = crate::hivemq_rest_client::tests::mock_cursor_responses(
            &broker,
            "/api/v1/data-hub/behavior-validation/policies",
            &responses,
            "foobar",
        );

        service.load_behavior_policies().await.unwrap();

        let mut created_behavior_policies = Vec::new();
        for mut behavior_policy_list in responses {
            behavior_policy_list
                .items
                .iter_mut()
                .for_each(|behavior_policies| created_behavior_policies.append(behavior_policies))
        }

        assert_eq!(created_behavior_policies, repo.find_all().unwrap());
    }

    #[tokio::test]
    async fn test_load_behavior_policies_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = GetAllBehaviorPoliciesError::Status503(Errors::new());
        broker.mock(|when, then| {
            when.any_request();
            then.status(503).json_body(json!(error));
        });

        let response = service.load_behavior_policies().await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_create_behavior_policy() {
        let (broker, _pool, repo, service) = setup();

        let behavior_policy = build_behavior_policy(1);
        let behavior_policy_json = serde_json::to_string(&behavior_policy).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(behavior_policy_json.clone());
        });

        let _response = service
            .create_behavior_policy(&behavior_policy_json)
            .await
            .unwrap();

        assert_eq!(
            behavior_policy,
            repo.find_by_id(&behavior_policy.id).unwrap()
        );
    }

    #[tokio::test]
    async fn test_create_behavior_policy_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = CreateBehaviorPolicyError::Status503(Errors::new());
        let behavior_policy = build_behavior_policy(1);
        let behavior_policy_json = serde_json::to_string(&behavior_policy).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.create_behavior_policy(&behavior_policy_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_delete_behavior_policy() {
        let (broker, _pool, repo, service) = setup();

        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(200);
        });

        let policy = build_behavior_policy(1);
        repo.save(&policy).unwrap();

        let _response = service.delete_behavior_policy("behavior_policy-1").await;

        assert!(repo.find_by_id("behavior_policy-1").is_err());
    }

    #[tokio::test]
    async fn test_update_behavior_policy() {
        let (broker, _pool, repo, service) = setup();

        let policy = build_behavior_policy(1);
        let policy_json = serde_json::to_string(&policy).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(PUT);
            then.status(200).body(policy_json.clone());
        });

        service.update_behavior_policy(&policy_json).await.unwrap();

        assert_eq!(policy, repo.find_by_id(&policy.id.clone()).unwrap());
    }

    #[tokio::test]
    async fn test_update_behavior_policy_error() {
        let (broker, _pool, _repo, service) = setup();

        let policy = build_behavior_policy(1);
        let error = UpdateBehaviorPolicyError::Status503(Errors::new());
        let policy_json = serde_json::to_string(&policy).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(PUT);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let result = service.update_behavior_policy(&policy_json).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_behavior_policy_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = DeleteBehaviorPolicyError::Status404(Errors::new());
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.delete_behavior_policy("behavior_policy-1").await;
        assert!(response.is_err())
    }
}
