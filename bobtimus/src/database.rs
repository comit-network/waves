use anyhow::Result;
use diesel::{Connection, SqliteConnection};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

embed_migrations!("./migrations");

#[derive(Clone)]
pub struct Sqlite {
    connection: Arc<Mutex<SqliteConnection>>,
}

impl Sqlite {
    /// Return a handle that can be used to access the database.
    ///
    /// Reads or creates an SQLite database file at 'file'. When this returns
    /// an Sqlite database exists, a successful connection to the database has
    /// been made, and the database migrations have been run.
    pub fn new(file: &Path) -> Result<Self> {
        ensure_folder_tree_exists(file)?;

        let connection = SqliteConnection::establish(&format!("file:{}", file.display()))?;
        embedded_migrations::run(&connection)?;

        tracing::info!("SQLite database file loaded: {}", file.display());

        Ok(Sqlite {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// Return a ephemeral db handle to be used for tests.
    ///
    /// The db file will be removed at the end of the process lifetime.
    #[cfg(test)]
    pub fn new_ephemeral_db() -> Result<Self> {
        let temp_file = tempfile::Builder::new()
            .suffix(".sqlite")
            .tempfile()
            .unwrap();
        Self::new(temp_file.path())
    }

    pub async fn do_in_transaction<F, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&SqliteConnection) -> anyhow::Result<T>,
    {
        let guard = self.connection.lock().await;
        let connection = &*guard;

        let result = connection.transaction(|| f(&connection))?;

        Ok(result)
    }
}

fn ensure_folder_tree_exists(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db() -> PathBuf {
        let temp_file = tempfile::Builder::new()
            .suffix(".sqlite")
            .tempfile()
            .unwrap();

        temp_file.into_temp_path().to_path_buf()
    }

    #[test]
    fn can_create_a_new_temp_db() {
        let path = temp_db();

        let db = Sqlite::new(&path);

        assert!(&db.is_ok());
    }

    #[test]
    fn given_no_database_exists_calling_new_creates_it() {
        let path = temp_db();
        // validate assumptions: the db does not exist yet
        assert_eq!(path.as_path().exists(), false);

        let db = Sqlite::new(&path);

        assert!(&db.is_ok());
        assert!(&path.as_path().exists());
    }

    #[test]
    fn given_db_in_non_existing_directory_tree_calling_new_creates_it() {
        let tempfile = tempfile::tempdir().unwrap();
        let mut path = PathBuf::new();

        path.push(tempfile);
        path.push("i_dont_exist");
        path.push("database.sqlite");

        // validate assumptions:
        // 1. the db does not exist yet
        // 2. the parent folder does not exist yet
        assert_eq!(path.as_path().exists(), false);
        assert_eq!(path.parent().unwrap().exists(), false);

        let db = Sqlite::new(&path);

        assert!(&db.is_ok());
        assert!(&path.exists());
    }
}
