use rusqlite::Connection;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
enum RepositoryError {
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

struct Repository<'a, T: Serialize + DeserializeOwned> {
    conn: &'a Connection,
    table_name: String,
    get_id: Box<fn(&T) -> String>,
}

impl<'a, T: Serialize + DeserializeOwned> Repository<'a, T> {
    fn init(
        connection: &'a Connection,
        table_name: &'a str,
        get_id: fn(&T) -> String,
    ) -> Result<Self, RepositoryError> {
        connection.execute(
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
            conn: connection,
            table_name: table_name.clone().to_string(),
            get_id: Box::new(get_id),
        })
    }

    fn save(&self, t: &T) -> Result<(), RepositoryError> {
        let get_id = &self.get_id;
        let json = &serde_json::to_string(t).unwrap();
        Ok(self
            .conn
            .execute(
                &format!(
                    "
        REPLACE INTO {} (id, data)
        VALUES ({}, json('{}'));
        ",
                    &self.table_name,
                    &get_id(t),
                    json
                ),
                [],
            )
            .map(|size| ())?)
    }

    fn find_by_id(&self, id: &str) -> Result<T, RepositoryError> {
        Ok(self.conn.query_row(
            &format!(
                "
        SELECT *
        FROM {}
        WHERE id='{}'
        ",
                &self.table_name, id
            ),
            [],
            |row| {
                let json: String = row.get("data")?;
                let result: Result<T, serde_json::Error> = serde_json::from_str(&json);
                Ok(result)
            },
        )??)
    }

    fn find_by(&self, json_path: &str, to_match: &str) -> Result<Vec<T>, RepositoryError> {
        let mut stmt = self.conn.prepare(&format!(
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

    fn find_all(&self) -> Result<Vec<T>, RepositoryError> {
        let mut stmt = self.conn.prepare(&format!(
            "
        SELECT *
        FROM {}",
            self.table_name
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

    fn delete_by_id(&self, id: &str) -> Result<(), RepositoryError> {
        Ok(self
            .conn
            .execute(
                &format!(
                    "
        DELETE
        FROM {}
        WHERE id='{}'
        ",
                    &self.table_name, id
                ),
                [],
            )
            .map(|size| ())?)
    }

    fn delete_all(&self) -> Result<(), RepositoryError> {
        Ok(self
            .conn
            .execute(
                &format!(
                    "
        DELETE
        FROM {}
        ",
                    &self.table_name
                ),
                [],
            )
            .map(|size| ())?)
    }
}

#[cfg(test)]
mod tests {
    use hivemq_openapi::models::ClientDetails;
    use rusqlite::Connection;
    use serde_json::Value::Bool;
    use serde_json::{json, Number, Value};
    use std::collections::HashMap;
    use crate::repository::{Repository, RepositoryError};


    pub fn init_repo(conn: &Connection) -> Repository<Value> {
        Repository::<Value>::init(&conn, "test_values", |x| x.get("id").unwrap().to_string())
            .unwrap()
    }

    #[test]
    fn test_save() {
        let conn = Connection::open_in_memory().unwrap();
        let repo = init_repo(&conn);
        let expected = json!({ "id": "id1", "val1": 2, "val2": true });

        repo.save(&expected).unwrap();
        let actual = repo.find_by_id("id1").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_find_by_id() {
        let conn = Connection::open_in_memory().unwrap();
        let repo = init_repo(&conn);
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
        let conn = Connection::open_in_memory().unwrap();
        let repo = init_repo(&conn);
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
        let conn = Connection::open_in_memory().unwrap();
        let repo = init_repo(&conn);
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
        let conn = Connection::open_in_memory().unwrap();
        let repo = init_repo(&conn);
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
        let conn = Connection::open_in_memory().unwrap();
        let repo = init_repo(&conn);
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
