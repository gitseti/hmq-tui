use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use regex::Regex;
use rusqlite::functions::FunctionFlags;
use color_eyre::eyre::{Ok, Result};
use rusqlite::Error;

pub fn init_sqlite() -> Pool<SqliteConnectionManager> {
    let sqlite_pool = Pool::new(SqliteConnectionManager::memory()).unwrap();
    add_regexp_function(&sqlite_pool.clone()).unwrap();
    sqlite_pool
}

fn add_regexp_function(db: &Pool<SqliteConnectionManager>) -> rusqlite::Result<()> {
    db.get().unwrap().create_scalar_function(
        "regexp",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            assert_eq!(ctx.len(), 2, "called with unexpected number of arguments");
            let regexp: Arc<Regex> = ctx.get_or_create_aux(0, |vr| -> Result<Regex> {
                Ok(Regex::new(vr.as_str()?)?)
            })?;
            let is_match = {
                let text = ctx
                    .get_raw(1)
                    .as_str()
                    .map_err(|e| Error::UserFunctionError(e.into()))?;

                regexp.is_match(text)
            };

            Result::Ok(is_match)
        },
    )
}