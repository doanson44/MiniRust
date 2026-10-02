use std::path::{Component, Path, PathBuf};

use minirust_services::{FileStore, UploadError};

#[derive(Debug, Clone)]
pub struct FilesystemStore {
    root: PathBuf,
}

impl FilesystemStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn resolve(&self, file_name: &str) -> Result<PathBuf, UploadError> {
        let mut path = self.root.clone();

        for component in Path::new(file_name).components() {
            match component {
                Component::Normal(part) => path.push(part),
                _ => return Err(UploadError::Storage),
            }
        }

        Ok(path)
    }
}

impl FileStore for FilesystemStore {
    async fn store(&self, file_name: &str, content: &[u8]) -> Result<(), UploadError> {
        let path = self.resolve(file_name)?;

        if let Some(directory) = path.parent() {
            if let Err(error) = tokio::fs::create_dir_all(directory).await {
                tracing::error!(%error, "failed to prepare the upload directory");
                return Err(UploadError::Storage);
            }
        }

        if let Err(error) = tokio::fs::write(&path, content).await {
            tracing::error!(%error, "failed to write an uploaded file");
            return Err(UploadError::Storage);
        }

        Ok(())
    }

    async fn load(&self, file_name: &str) -> Result<Option<Vec<u8>>, UploadError> {
        let path = self.resolve(file_name)?;

        match tokio::fs::read(&path).await {
            Ok(content) => Ok(Some(content)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => {
                tracing::error!(%error, "failed to read an uploaded file");
                Err(UploadError::Storage)
            }
        }
    }
}
