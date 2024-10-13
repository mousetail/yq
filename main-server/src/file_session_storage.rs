use std::{fs::OpenOptions, path::PathBuf};

use axum::async_trait;
use tokio::fs::remove_file;
use tower_sessions::{
    session::{Id, Record},
    session_store, SessionStore,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FileSessionStorage;

#[async_trait]
impl SessionStore for FileSessionStorage {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        std::fs::create_dir_all(".sessions")
            .map_err(|_| session_store::Error::Backend("Failed to create folder".to_string()))?;

        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(PathBuf::from(".sessions").join(record.id.to_string()))
            .map_err(|_| session_store::Error::Backend("Failed to open file".to_string()))?;
        serde_json::to_writer(file, &record)
            .map_err(|_| session_store::Error::Backend("Failed to serialize/decode".to_string()))?;
        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(PathBuf::from(".sessions").join(record.id.to_string()))
            .map_err(|_| session_store::Error::Backend("Failed to open file".to_string()))?;
        serde_json::to_writer(file, &record)
            .map_err(|_| session_store::Error::Backend("Failed to serialize/decode".to_string()))?;
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let file = OpenOptions::new()
            .read(true)
            .open(PathBuf::from(".sessions").join(session_id.to_string()))
            .map_err(|_| session_store::Error::Backend("Failed to open file".to_string()))?;
        let out = serde_json::from_reader(file)
            .map_err(|_| session_store::Error::Backend("Failed to serialize/decode".to_string()))?;
        Ok(out)
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        remove_file(PathBuf::from(".sessions").join(session_id.to_string()))
            .await
            .map_err(|_| session_store::Error::Backend("Failed to Delete".to_string()))?;
        Ok(())
    }
}
