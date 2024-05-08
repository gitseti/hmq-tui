use std::sync::Arc;

use hivemq_openapi::apis::configuration::Configuration;
use hivemq_openapi::apis::data_hub_schemas_api::{
    get_all_schemas, CreateSchemaParams, DeleteSchemaParams, GetAllSchemasParams,
};
use hivemq_openapi::models::Schema;

use crate::hivemq_rest_client;
use crate::hivemq_rest_client::{get_cursor, transform_api_err};
use crate::repository::Repository;

pub struct SchemaService {
    repository: Arc<Repository<Schema>>,
    config: Configuration,
}

impl SchemaService {
    pub fn new(repository: Arc<Repository<Schema>>, host: &str) -> Self {
        let config = hivemq_rest_client::build_rest_api_config(host.to_string());
        SchemaService { repository, config }
    }

    pub async fn load_schemas(&self) -> Result<(), String> {
        let mut params = GetAllSchemasParams {
            fields: None,
            types: None,
            limit: Some(500),
            cursor: None,
            schema_ids: None,
        };

        loop {
            let response = get_all_schemas(&self.config, params.clone())
                .await
                .map_err(transform_api_err)?;

            for schema in response.items.into_iter().flatten() {
                self.repository.save(&schema).unwrap();
            }

            let cursor = get_cursor(response._links);
            if cursor.is_none() {
                return Ok(());
            } else {
                params.cursor = cursor;
            }
        }
    }

    pub async fn create_schema(&self, schema: &String) -> Result<String, String> {
        let schema: Schema =
            serde_json::from_str(schema.as_str()).or_else(|err| Err(err.to_string()))?;
        let schema_id = schema.id.clone();

        let params = CreateSchemaParams { schema };

        let response =
            hivemq_openapi::apis::data_hub_schemas_api::create_schema(&self.config, params)
                .await
                .map_err(transform_api_err)?;

        self.repository.save(&response).unwrap();

        Ok(schema_id)
    }

    pub async fn delete_schema(&self, schema_id: &str) -> Result<String, String> {
        let params = DeleteSchemaParams {
            schema_id: schema_id.to_owned(),
        };

        hivemq_openapi::apis::data_hub_schemas_api::delete_schema(&self.config, params)
            .await
            .map_err(transform_api_err)?;

        self.repository.delete_by_id(&schema_id).unwrap();

        Ok(schema_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hivemq_openapi::apis::data_hub_schemas_api::{
        CreateSchemaError, DeleteSchemaError, GetAllSchemasError,
    };
    use hivemq_openapi::models::{Errors, PaginationCursor, Schema, SchemaList};
    use httpmock::Method::{DELETE, POST};
    use httpmock::MockServer;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use serde_json::json;

    use crate::repository::Repository;
    use crate::services::schema_service::SchemaService;

    fn build_schema(schema_num: usize) -> Schema {
        Schema::new(
            format!("schema-{schema_num}"),
            String::from("{}"),
            String::from("JSON"),
        )
    }

    fn build_schema_list(
        start: usize,
        end: usize,
        cursor: Option<Option<Box<PaginationCursor>>>,
    ) -> SchemaList {
        let schemas: Vec<Schema> = (start..end).map(|i| build_schema(i)).collect();
        SchemaList {
            _links: cursor,
            items: Some(schemas),
        }
    }

    fn setup() -> (
        MockServer,
        Pool<SqliteConnectionManager>,
        Arc<Repository<Schema>>,
        SchemaService,
    ) {
        let broker = MockServer::start();
        let connection_pool = Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo =
            Repository::<Schema>::init(&connection_pool, "schemas", |schema| schema.id.clone())
                .unwrap();
        let repo = Arc::new(repo);
        let service = SchemaService::new(repo.clone(), &broker.base_url());
        (broker, connection_pool, repo, service)
    }

    #[tokio::test]
    async fn test_load_schemas() {
        let (broker, _pool, repo, service) = setup();

        let responses = crate::hivemq_rest_client::tests::create_responses(
            "/api/v1/data-hub/schemas",
            build_schema_list,
        );
        let _mocks = crate::hivemq_rest_client::tests::mock_cursor_responses(
            &broker,
            "/api/v1/data-hub/schemas",
            &responses,
            "foobar",
        );

        service.load_schemas().await.unwrap();

        let mut created_schemas = Vec::new();
        for mut schema_list in responses {
            schema_list
                .items
                .iter_mut()
                .for_each(|schemas| created_schemas.append(schemas))
        }

        assert_eq!(created_schemas, repo.find_all().unwrap());
    }

    #[tokio::test]
    async fn test_load_schemas_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = GetAllSchemasError::Status503(Errors::new());
        broker.mock(|when, then| {
            when.any_request();
            then.status(503).json_body(json!(error));
        });

        let response = service.load_schemas().await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_create_schema() {
        let (broker, _pool, repo, service) = setup();

        let schema = build_schema(1);
        let schema_json = serde_json::to_string(&schema).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(schema_json.clone());
        });

        let _response = service.create_schema(&schema_json).await.unwrap();

        assert_eq!(schema, repo.find_by_id(&schema.id).unwrap());
    }

    #[tokio::test]
    async fn test_create_schema_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = CreateSchemaError::Status503(Errors::new());
        let schema = build_schema(1);
        let schema_json = serde_json::to_string(&schema).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.create_schema(&schema_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_delete_schema() {
        let (broker, _pool, repo, service) = setup();

        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(200);
        });

        repo.save(&Schema::new(
            "schema-1".to_string(),
            "{}".to_string(),
            "JSON".to_string(),
        ))
        .unwrap();

        let _response = service.delete_schema("schema-1").await;

        assert!(repo.find_by_id("schema-1").is_err());
    }

    #[tokio::test]
    async fn test_delete_schema_error() {
        let (broker, _pool, _repo, service) = setup();

        let error = DeleteSchemaError::Status404(Errors::new());
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.delete_schema("schema-1").await;
        assert!(response.is_err())
    }
}
