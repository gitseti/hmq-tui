use std::sync::Arc;

use hivemq_openapi::apis::backup_restore_api::get_all_backups;
use hivemq_openapi::apis::configuration::Configuration;
use hivemq_openapi::models::Backup;

use crate::hivemq_rest_client;
use crate::hivemq_rest_client::transform_api_err;
use crate::repository::Repository;

pub struct BackupService {
    repository: Arc<Repository<Backup>>,
    config: Configuration,
}

impl BackupService {
    pub fn new(repository: Arc<Repository<Backup>>, host: &str) -> Self {
        let config = hivemq_rest_client::build_rest_api_config(host.to_string());
        BackupService { repository, config }
    }

    pub async fn load_backups(&self) -> Result<(), String> {
        let response = get_all_backups(&self.config)
            .await
            .map_err(transform_api_err)?;

        for item in response.items.into_iter().flatten() {
            self.repository.save(&item).unwrap();
        }

        Ok(())
    }

    pub async fn start_backup(&self) -> Result<String, String> {
        let response = hivemq_openapi::apis::backup_restore_api::create_backup(&self.config)
            .await
            .map_err(transform_api_err)?;

        if let Some(backup) = response.backup {
            self.repository.save(&backup).unwrap();
            Ok((backup.id.unwrap()))
        } else {
            return Err(String::from("No backup was created"));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hivemq_openapi::apis::backup_restore_api::{CreateBackupError, GetAllBackupsError};
    use hivemq_openapi::models::{Backup, BackupList, Errors};
    use httpmock::Method::{GET, POST};
    use httpmock::MockServer;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use serde_json::json;

    use crate::repository::Repository;
    use crate::services::backups_service::BackupService;

    fn build_backup(backup_num: usize) -> Backup {
        Backup {
            bytes: None,
            created_at: None,
            fail_reason: None,
            id: Some(format!("backup-${backup_num}")),
            state: None,
        }
    }

    fn build_backup_list(start: usize, end: usize) -> BackupList {
        let backups: Vec<Backup> = (start..end).map(|i| build_backup(i)).collect();
        BackupList {
            items: Some(backups),
        }
    }

    fn setup() -> (
        MockServer,
        Pool<SqliteConnectionManager>,
        Arc<Repository<Backup>>,
        BackupService,
    ) {
        let broker = MockServer::start();
        let connection_pool = Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo = Repository::<Backup>::init(&connection_pool, "backups", |backup| {
            backup.id.clone().unwrap()
        })
        .unwrap();
        let repo = Arc::new(repo);
        let service = BackupService::new(repo.clone(), &broker.base_url());
        (broker, connection_pool, repo, service)
    }

    #[tokio::test]
    async fn test_load_backups() {
        let (broker, pool, repo, service) = setup();
        let backup_list = build_backup_list(0, 10);
        broker.mock(|when, then| {
            when.any_request().method(GET);
            then.status(200)
                .body(serde_json::to_string(&backup_list).unwrap());
        });

        service.load_backups().await.unwrap();

        let created_backups = backup_list.items.unwrap();
        assert_eq!(created_backups, repo.find_all().unwrap());
    }

    #[tokio::test]
    async fn test_load_backups_error() {
        let (broker, pool, repo, service) = setup();

        let error = GetAllBackupsError::Status503(Errors::new());
        broker.mock(|when, then| {
            when.any_request();
            then.status(503).json_body(json!(error));
        });

        let response = service.load_backups().await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_start_backup() {
        let (broker, pool, repo, service) = setup();

        let backup = hivemq_openapi::models::BackupItem {
            backup: Some(Box::new(build_backup(1))),
        };
        let backup_json = serde_json::to_string(&backup).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(201).body(backup_json.clone());
        });

        let response = service.start_backup().await.unwrap();

        assert_eq!(
            backup.backup.unwrap(),
            Box::new(repo.find_by_id(&response).unwrap())
        );
    }

    #[tokio::test]
    async fn test_create_backup_error() {
        let (broker, pool, repo, service) = setup();

        let error = CreateBackupError::Status503(Errors::new());
        let backup = build_backup(1);
        let backup_json = serde_json::to_string(&backup).unwrap();
        broker.mock(|when, then| {
            when.any_request().method(POST);
            then.status(503)
                .body(serde_json::to_string(&error).unwrap());
        });

        let response = service.start_backup().await;

        assert!(response.is_err());
    }
}
