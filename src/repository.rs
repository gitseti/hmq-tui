use r2d2_sqlite::SqliteConnectionManager;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug)]
pub enum RepositoryError {
    SerdeError(serde_json::Error),
    SqlError(rusqlite::Error),
}

impl From<serde_json::Error> for RepositoryError {
    fn from(value: serde_json::Error) -> Self {
        RepositoryError::SerdeError(value)
    }
}

impl From<rusqlite::Error> for RepositoryError {
    fn from(value: rusqlite::Error) -> Self {
        RepositoryError::SqlError(value)
    }
}

pub struct Repository<T: Serialize + DeserializeOwned> {
    connection_pool: r2d2::Pool<SqliteConnectionManager>,
    table_name: String,
    get_id: Box<fn(&T) -> String>,
}

impl<'a, T: Serialize + DeserializeOwned> Repository<T> {
    pub fn init(
        connection_pool: &'a r2d2::Pool<SqliteConnectionManager>,
        table_name: &'a str,
        get_id: fn(&T) -> String,
    ) -> Result<Self, RepositoryError> {
        connection_pool.get().unwrap().execute(
            &format!(
                "
        CREATE TABLE {table_name}(
            id STRING PRIMARY KEY,
            data JSON
        );
        "
            ),
            [],
        )?;
        Ok(Repository {
            connection_pool: connection_pool.clone(),
            table_name: table_name.to_string(),
            get_id: Box::new(get_id),
        })
    }

    pub fn save(&self, t: &T) -> Result<(), RepositoryError> {
        let get_id = &self.get_id;
        let id = &get_id(t);
        let json = &serde_json::to_string(t).unwrap();
        let table_name = &self.table_name;
        Ok(self
            .connection_pool
            .get()
            .unwrap()
            .execute(
                &format!(
                    "
        REPLACE INTO {table_name} (id, data)
        VALUES ('{id}', json('{json}'));
        "
                ),
                [],
            )
            .map(|_size| ())?)
    }

    pub fn find_by_id(&self, id: &str) -> Result<T, RepositoryError> {
        let table_name = &self.table_name;
        Ok(self.connection_pool.get().unwrap().query_row(
            &format!(
                "
        SELECT *
        FROM {table_name}
        WHERE id='{id}'
        ",
            ),
            [],
            |row| {
                let json: String = row.get("data")?;
                let result: Result<T, serde_json::Error> = serde_json::from_str(&json);
                Ok(result)
            },
        )??)
    }

    pub fn find_by(&self, json_path: &str, to_match: &str) -> Result<Vec<T>, RepositoryError> {
        let binding = self.connection_pool.get().unwrap();
        let mut stmt = binding.prepare(&format!(
            "
        SELECT data
        FROM {}
        WHERE data -> '{}' LIKE '%{}%'",
            &self.table_name, json_path, to_match
        ))?;

        let items = stmt.query_map([], |row| {
            let json: String = row.get("data")?;
            let t: Result<T, serde_json::Error> = serde_json::from_str(&json);
            Ok(t)
        })?;

        let mut vec = Vec::with_capacity(items.size_hint().0);
        for item in items {
            match item {
                Ok(Ok(value)) => vec.push(value),
                _ => {}
            }
        }

        Ok(vec)
    }

    pub fn find_ids_by(
        &self,
        json_path: &str,
        to_match: &str,
    ) -> Result<Vec<String>, RepositoryError> {
        let binding = self.connection_pool.get().unwrap();
        let mut stmt = binding.prepare(&format!(
            "
        SELECT id
        FROM {}
        WHERE data -> '{}' LIKE '%{}%'",
            &self.table_name, json_path, to_match
        ))?;

        let items = stmt.query_map([], |row| {
            let id: String = row.get("id")?;
            Ok(id)
        })?;

        let mut vec = Vec::with_capacity(items.size_hint().0);
        for item in items {
            match item {
                Ok(value) => vec.push(value),
                _ => {}
            }
        }

        Ok(vec)
    }

    pub fn find_all(&self) -> Result<Vec<T>, RepositoryError> {
        let binding = self.connection_pool.get().unwrap();
        let table_name = &self.table_name;
        let mut stmt = binding.prepare(&format!(
            "
        SELECT *
        FROM {table_name}",
        ))?;

        let items = stmt.query_map([], |row| {
            let json: String = row.get("data")?;
            let result: Result<T, serde_json::Error> = serde_json::from_str(&json);
            Ok(result)
        })?;

        let mut vec = Vec::with_capacity(items.size_hint().0);
        for item in items {
            match item {
                Ok(Ok(value)) => vec.push(value),
                _ => {}
            }
        }

        Ok(vec)
    }

    pub fn find_all_ids(&self) -> Result<Vec<String>, RepositoryError> {
        let binding = self.connection_pool.get().unwrap();
        let table_name = &self.table_name;
        let mut stmt = binding.prepare(&format!(
            "
        SELECT id
        FROM {table_name}",
        ))?;

        let items = stmt.query_map([], |row| {
            let id: String = row.get("id")?;
            Ok(id)
        })?;

        let mut vec = Vec::with_capacity(items.size_hint().0);
        for item in items {
            match item {
                Ok(value) => vec.push(value),
                _ => {}
            }
        }

        Ok(vec)
    }

    pub fn delete_by_id(&self, id: &str) -> Result<(), RepositoryError> {
        let table_name = &self.table_name;
        Ok(self
            .connection_pool
            .get()
            .unwrap()
            .execute(
                &format!(
                    "
        DELETE
        FROM {table_name}
        WHERE id='{id}'
        "
                ),
                [],
            )
            .map(|_size| ())?)
    }

    fn delete_all(&self) -> Result<(), RepositoryError> {
        let table_name = &self.table_name;
        Ok(self
            .connection_pool
            .get()
            .unwrap()
            .execute(
                &format!(
                    "
        DELETE
        FROM {table_name}
        "
                ),
                [],
            )
            .map(|_size| ())?)
    }
}

#[cfg(test)]
mod tests {
    use r2d2_sqlite::SqliteConnectionManager;
    use serde_json::{json, Value};

    use crate::repository::{Repository, RepositoryError};

    pub fn init_repo(connection_pool: &r2d2::Pool<SqliteConnectionManager>) -> Repository<Value> {
        Repository::<Value>::init(connection_pool, "test_values", |x| {
            if let Value::String(val) = x.get("id").unwrap() {
                val.clone()
            } else {
                panic!("id not found")
            }
        })
        .unwrap()
    }

    #[test]
    fn test_save() {
        let connection_pool = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo = init_repo(&connection_pool);
        let expected = json!({ "id": "id1", "val1": 2, "val2": true });

        repo.save(&expected).unwrap();
        let actual = repo.find_by_id("id1").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_find_by_id() {
        let connection_pool = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo = init_repo(&connection_pool);
        let expected1 = json!({ "id": "id1", "val1": 2, "val2": true });
        let expected2 = json!({ "id": "id3", "val1": 2, "val2": true });
        let expected3 = json!({ "id": "id5", "val1": 2, "val2": true });

        repo.save(&expected1).unwrap();
        repo.save(&expected2).unwrap();
        repo.save(&expected3).unwrap();

        assert_eq!(expected1, repo.find_by_id("id1").unwrap());
        assert_eq!(expected2, repo.find_by_id("id3").unwrap());
        assert_eq!(expected3, repo.find_by_id("id5").unwrap());
    }

    #[test]
    fn test_find_by_query() {
        let connection_pool = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo = init_repo(&connection_pool);
        let expected1 = json!({ "id": "id1", "val1": 2, "val2": true });
        let expected2 = json!({ "id": "id3", "val1": 2, "val2": true });
        let expected3 = json!({ "id": "id5", "val1": 3, "val2": true });

        repo.save(&expected1).unwrap();
        repo.save(&expected2).unwrap();
        repo.save(&expected3).unwrap();

        assert_eq!(
            vec![expected1, expected2],
            repo.find_by("$.val1", "2").unwrap()
        );
        assert_eq!(vec![expected3], repo.find_by("$.val1", "3").unwrap());
    }

    #[test]
    fn test_find_all() {
        let connection_pool = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo = init_repo(&connection_pool);
        let expected1 = json!({ "id": "id1", "val1": 2, "val2": true });
        let expected2 = json!({ "id": "id3", "val1": 2, "val2": true });
        let expected3 = json!({ "id": "id5", "val1": 2, "val2": true });

        repo.save(&expected1).unwrap();
        repo.save(&expected2).unwrap();
        repo.save(&expected3).unwrap();

        assert_eq!(
            vec![expected1, expected2, expected3],
            repo.find_all().unwrap()
        );
    }

    #[test]
    fn test_delete_by_id() {
        let connection_pool = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo = init_repo(&connection_pool);
        let expected = json!({ "id": "id1", "val1": 2, "val2": true });

        repo.save(&expected).unwrap();
        repo.delete_by_id("id3").unwrap();

        let error = repo.find_by_id("id3").unwrap_err();
        match error {
            RepositoryError::SqlError(rusqlite::Error::QueryReturnedNoRows) => {}
            _ => assert!(false),
        }
    }

    #[test]
    fn test_delete_all() {
        let connection_pool = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
        let repo = init_repo(&connection_pool);
        let expected1 = json!({ "id": "id1", "val1": 2, "val2": true });
        let expected2 = json!({ "id": "id3", "val1": 2, "val2": true });
        let expected3 = json!({ "id": "id5", "val1": 2, "val2": true });

        repo.save(&expected1).unwrap();
        repo.save(&expected2).unwrap();
        repo.save(&expected3).unwrap();
        repo.delete_all().unwrap();

        assert_eq!(0, repo.find_all().unwrap().len());
    }
}
