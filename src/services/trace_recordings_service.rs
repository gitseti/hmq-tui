use std::sync::Arc;

use hivemq_openapi::apis::configuration::Configuration;
use hivemq_openapi::apis::trace_recordings_api::{create_trace_recording, CreateTraceRecordingParams, delete_trace_recording, DeleteTraceRecordingParams, get_all_trace_recordings};
use hivemq_openapi::models::{TraceRecording, TraceRecordingItem};

use crate::hivemq_rest_client;
use crate::hivemq_rest_client::transform_api_err;
use crate::repository::Repository;

pub struct TraceRecordingService {
    repository: Arc<Repository<TraceRecording>>,
    config: Configuration,
}

impl TraceRecordingService {
    pub fn new(repository: Arc<Repository<TraceRecording>>, host: &str) -> Self {
        let config = hivemq_rest_client::build_rest_api_config(host.to_string());
        TraceRecordingService {
            repository,
            config,
        }
    }

    pub async fn load_trace_recordings(&self) -> Result<(), String> {
        let response = get_all_trace_recordings(&self.config)
            .await
            .map_err(transform_api_err)?;

        for trace_recording in response.items.into_iter().flatten() {
            self.repository.save(&trace_recording).unwrap();
        }

        Ok(())
    }

    pub async fn create_trace_recording(&self, trace_recording: &String) -> Result<String, String> {
        let trace_recording: TraceRecordingItem =
            serde_json::from_str(trace_recording.as_str()).or_else(|err| Err(err.to_string()))?;

        let params = CreateTraceRecordingParams { trace_recording_item: Some(trace_recording) };

        let response = create_trace_recording(&self.config, params)
            .await
            .map_err(transform_api_err)?;

        self.repository.save(&response.trace_recording).unwrap();

        Ok(response.trace_recording.name.clone().unwrap())
    }

    pub async fn delete_trace_recording(&self, trace_recording_id: &str) -> Result<String, String> {
        let params = DeleteTraceRecordingParams {
            trace_recording_id: trace_recording_id.to_owned(),
        };

        delete_trace_recording(&self.config, params)
            .await
            .map_err(transform_api_err)?;

        self.repository.delete_by_id(&trace_recording_id).unwrap();

        Ok(trace_recording_id.to_string())
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hivemq_openapi::apis::trace_recordings_api::{CreateTraceRecordingError, DeleteTraceRecordingError, GetAllTraceRecordingsError};
    use hivemq_openapi::models::{Errors, trace_recording, TraceRecording, TraceRecordingItem, TraceRecordingList};
    use httpmock::Method::{DELETE, GET, POST};
    use httpmock::MockServer;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use serde_json::{json, Value};

    use crate::repository::Repository;
    use crate::services::trace_recordings_service::TraceRecordingService;

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


    fn setup() -> (MockServer, Pool<SqliteConnectionManager>, Arc<Repository<TraceRecording>>, TraceRecordingService) {
        let broker = MockServer::start();
        let connection_pool = Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo =
            Repository::<TraceRecording>::init(&connection_pool, "trace_recordings", |trace_recording| {
                trace_recording.name.clone().unwrap()
            }).unwrap();
        let repo = Arc::new(repo);
        let service = TraceRecordingService::new(repo.clone(), &broker.base_url());
        (broker, connection_pool, repo, service)
    }

    #[tokio::test]
    async fn test_load_trace_recordings() {
        let (broker, pool, repo, service) = setup();

        let trace_recordings = build_trace_recording_list(0, 10);
        broker.mock(|when, then| {
            when.any_request().method(GET);
            then.status(200)
                .body(serde_json::to_string(&trace_recordings).unwrap());
        });
        service.load_trace_recordings().await.unwrap();

        assert_eq!(trace_recordings.items.unwrap(), repo.find_all().unwrap());
    }

    #[tokio::test]
    async fn test_load_trace_recordings_error() {
        let (broker, pool, repo, service) = setup();

        let error = GetAllTraceRecordingsError::UnknownValue(Value::String("unknown".to_string()));
        broker.mock(|when, then| {
            when.any_request();
            then.status(503).json_body(json!(error));
        });

        let response = service.load_trace_recordings().await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_create_trace_recording() {
        let (broker, pool, repo, service) = setup();

        let trace_recording = build_trace_recording(1);
        let trace_recording = TraceRecordingItem::new(trace_recording);
        let trace_recording_json = serde_json::to_string(&trace_recording).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(trace_recording_json.clone());
        });

        let response = service.create_trace_recording(&trace_recording_json).await.unwrap();

        assert_eq!(trace_recording.trace_recording, Box::new(repo.find_by_id(&trace_recording.trace_recording.name.clone().unwrap()).unwrap()));
    }

    #[tokio::test]
    async fn test_create_trace_recording_error() {
        let (broker, pool, repo, service) = setup();

        let error = CreateTraceRecordingError::Status400(Errors::new());
        let trace_recording = build_trace_recording(1);
        let trace_recording_json = serde_json::to_string(&trace_recording).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.create_trace_recording(&trace_recording_json).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_delete_trace_recording() {
        let (broker, pool, repo, service) = setup();

        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(200);
        });

        let trace_recording = build_trace_recording(1);
        repo.save(&trace_recording).unwrap();

        let response = service.delete_trace_recording("trace_recording-1").await;

        assert!(repo.find_by_id("trace_recording-1").is_err());
    }

    #[tokio::test]
    async fn test_delete_trace_recording_error() {
        let (broker, pool, repo, service) = setup();

        let error = DeleteTraceRecordingError::Status404(Errors::new());
        broker.mock(|when, then| {
            when.any_request().method(DELETE);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.delete_trace_recording("trace_recording-1").await;
        assert!(response.is_err())
    }
}
