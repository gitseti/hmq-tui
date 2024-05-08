use std::sync::Arc;

use hivemq_openapi::apis::configuration::Configuration;
use hivemq_openapi::apis::data_hub_scripts_api::{
    get_all_scripts, CreateScriptParams, DeleteScriptParams, GetAllScriptsParams,
};
use hivemq_openapi::models::Script;

use crate::hivemq_rest_client;
use crate::hivemq_rest_client::{get_cursor, transform_api_err};
use crate::repository::Repository;

pub struct ScriptService {
    repository: Arc<Repository<Script>>,
    config: Configuration,
}

impl ScriptService {
    pub fn new(repository: Arc<Repository<Script>>, host: &str) -> Self {
        let config = hivemq_rest_client::build_rest_api_config(host.to_string());
        ScriptService { repository, config }
    }

    pub async fn load_scripts(&self) -> Result<(), String> {
        let mut params = GetAllScriptsParams {
            fields: None,
            function_types: None,
            limit: Some(500),
            cursor: None,
            script_ids: None,
        };

        loop {
            let response = get_all_scripts(&self.config, params.clone())
                .await
                .map_err(transform_api_err)?;

            for script in response.items.into_iter().flatten() {
                self.repository.save(&script).unwrap();
            }

            let cursor = get_cursor(response._links);
            if cursor.is_none() {
                return Ok(());
            } else {
                params.cursor = cursor;
            }
        }
    }

    pub async fn create_script(&self, script: &String) -> Result<String, String> {
        let script: Script =
            serde_json::from_str(script.as_str()).or_else(|err| Err(err.to_string()))?;
        let script_id = script.id.clone();

        let params = CreateScriptParams { script };

        let response =
            hivemq_openapi::apis::data_hub_scripts_api::create_script(&self.config, params)
                .await
                .map_err(transform_api_err)?;

        self.repository.save(&response).unwrap();

        Ok(script_id)
    }

    pub async fn delete_script(&self, script_id: &str) -> Result<String, String> {
        let params = DeleteScriptParams {
            script_id: script_id.to_owned(),
        };

        hivemq_openapi::apis::data_hub_scripts_api::delete_script(&self.config, params)
            .await
            .map_err(transform_api_err)?;

        self.repository.delete_by_id(&script_id).unwrap();

        Ok(script_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hivemq_openapi::apis::data_hub_scripts_api::{
        CreateScriptError, DeleteScriptError, GetAllScriptsError,
    };
    use hivemq_openapi::models::script::FunctionType;
    use hivemq_openapi::models::{Errors, PaginationCursor, Script, ScriptList};
    use httpmock::Method::{DELETE, POST};
    use httpmock::MockServer;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use serde_json::json;

    use crate::repository::Repository;
    use crate::services::scripts_service::ScriptService;

    fn build_script(script_num: usize) -> Script {
        Script::new(
            FunctionType::Transformation,
            format!("script-${script_num}"),
            "function transform(publish, context) { return publish; }".to_string(),
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

    fn setup() -> (
        MockServer,
        Pool<SqliteConnectionManager>,
        Arc<Repository<Script>>,
        ScriptService,
    ) {
        let broker = MockServer::start();
        let connection_pool = Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo =
            Repository::<Script>::init(&connection_pool, "scripts", |script| script.id.clone())
                .unwrap();
        let repo = Arc::new(repo);
        let service = ScriptService::new(repo.clone(), &broker.base_url());
        (broker, connection_pool, repo, service)
    }

    #[tokio::test]
    async fn test_load_scripts() {
        let (broker, _pool, repo, service) = setup();

        let responses = crate::hivemq_rest_client::tests::create_responses(
            "/api/v1/data-hub/scripts",
            build_script_list,
        );
        let _mocks = crate::hivemq_rest_client::tests::mock_cursor_responses(
            &broker,
            "/api/v1/data-hub/scripts",
            &responses,
            "foobar",
        );

        service.load_scripts().await.unwrap();

        let mut created_scripts = Vec::new();
        for mut script_list in responses {
            script_list
                .items
                .iter_mut()
                .for_each(|scripts| created_scripts.append(scripts))
        }

        assert_eq!(created_scripts, repo.find_all().unwrap());
    }

    #[tokio::test]
    async fn test_load_scripts_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = GetAllScriptsError::Status503(Errors::new());
        broker.mock(|when, then| {
            when.any_request();
            then.status(503).json_body(json!(error));
        });

        let response = service.load_scripts().await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_create_script() {
        let (broker, _pool, repo, service) = setup();

        let script = build_script(1);
        let script_json = serde_json::to_string(&script).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(script_json.clone());
        });

        let _response = service.create_script(&script_json).await.unwrap();

        assert_eq!(script, repo.find_by_id(&script.id).unwrap());
    }

    #[tokio::test]
    async fn test_create_script_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = CreateScriptError::Status503(Errors::new());
        let script = build_script(1);
        let script_json = serde_json::to_string(&script).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.create_script(&script_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_delete_script() {
        let (broker, _pool, repo, service) = setup();

        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(200);
        });

        let script = build_script(1);
        repo.save(&script).unwrap();

        let _response = service.delete_script("script-1").await;

        assert!(repo.find_by_id("script-1").is_err());
    }

    #[tokio::test]
    async fn test_delete_script_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = DeleteScriptError::Status404(Errors::new());
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.delete_script("script-1").await;
        assert!(response.is_err())
    }
}
