use uuid::Uuid;

use crate::cqrs::{AsyncCommandHandler, AsyncQueryHandler, Command, Query};

pub const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;
pub const AVATAR_STEM: &str = "avatar";
pub const AVATAR_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadCommand {
    pub file_name: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredFile {
    pub name: String,
    pub original_name: String,
    pub size: u64,
}

impl Command for UploadCommand {
    type Output = StoredFile;
    type Error = UploadError;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadError {
    FileNameRequired,
    FileTooLarge { max: usize },
    AvatarDirectoryInvalid,
    AvatarExtensionUnsupported,
    Storage,
}

impl std::fmt::Display for UploadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNameRequired => formatter.write_str("file name is required"),
            Self::FileTooLarge { max } => {
                write!(formatter, "file exceeds the {max} byte upload limit")
            }
            Self::AvatarDirectoryInvalid => formatter.write_str("avatar directory is invalid"),
            Self::AvatarExtensionUnsupported => {
                formatter.write_str("avatar image format is unsupported")
            }
            Self::Storage => formatter.write_str("file could not be stored"),
        }
    }
}

impl std::error::Error for UploadError {}

#[allow(async_fn_in_trait)]
pub trait FileStore: Clone + Send + Sync + 'static {
    async fn store(&self, file_name: &str, content: &[u8]) -> Result<(), UploadError>;

    async fn load(&self, file_name: &str) -> Result<Option<Vec<u8>>, UploadError>;
}

pub fn avatar_file_name(directory: &str, extension: &str) -> Result<String, UploadError> {
    if !is_safe_segment(directory) {
        return Err(UploadError::AvatarDirectoryInvalid);
    }

    let extension = extension.trim().to_ascii_lowercase();
    if !AVATAR_EXTENSIONS.contains(&extension.as_str()) {
        return Err(UploadError::AvatarExtensionUnsupported);
    }

    Ok(format!("{directory}/{AVATAR_STEM}.{extension}"))
}

pub fn is_avatar_extension(extension: &str) -> bool {
    AVATAR_EXTENSIONS.contains(&extension.trim().to_ascii_lowercase().as_str())
}

fn is_safe_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment.len() <= 64
        && segment
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

#[derive(Debug, Clone)]
pub struct UploadCommandHandler<S> {
    store: S,
}

impl<S> UploadCommandHandler<S>
where
    S: FileStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> AsyncCommandHandler<UploadCommand> for UploadCommandHandler<S>
where
    S: FileStore,
{
    async fn handle(&self, command: UploadCommand) -> Result<StoredFile, UploadError> {
        let original_name = command.file_name.trim().to_owned();
        if original_name.is_empty() {
            return Err(UploadError::FileNameRequired);
        }

        if command.content.len() > MAX_UPLOAD_BYTES {
            return Err(UploadError::FileTooLarge {
                max: MAX_UPLOAD_BYTES,
            });
        }

        let name = stored_file_name(&original_name);
        self.store.store(&name, &command.content).await?;

        Ok(StoredFile {
            name,
            original_name,
            size: command.content.len() as u64,
        })
    }
}

fn stored_file_name(original_name: &str) -> String {
    let mut name = Uuid::now_v7().simple().to_string();
    if let Some(extension) = safe_extension(original_name) {
        name.push('.');
        name.push_str(&extension);
    }
    name
}

fn safe_extension(file_name: &str) -> Option<String> {
    let (_, extension) = file_name.rsplit_once('.')?;
    let extension = extension.trim().to_ascii_lowercase();

    if extension.is_empty()
        || extension.len() > 16
        || !extension
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        return None;
    }

    Some(extension)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadAvatarCommand {
    pub directory: String,
    pub file_name: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredAvatar {
    pub file_name: String,
    pub extension: String,
}

impl Command for UploadAvatarCommand {
    type Output = StoredAvatar;
    type Error = UploadError;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadAvatarQuery {
    pub directory: String,
    pub extension: String,
}

impl Query for LoadAvatarQuery {
    type Output = Result<Option<Vec<u8>>, UploadError>;
}

#[derive(Debug, Clone)]
pub struct UploadAvatarCommandHandler<S> {
    store: S,
}

impl<S> UploadAvatarCommandHandler<S>
where
    S: FileStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> AsyncCommandHandler<UploadAvatarCommand> for UploadAvatarCommandHandler<S>
where
    S: FileStore,
{
    async fn handle(&self, command: UploadAvatarCommand) -> Result<StoredAvatar, UploadError> {
        if command.content.len() > MAX_UPLOAD_BYTES {
            return Err(UploadError::FileTooLarge {
                max: MAX_UPLOAD_BYTES,
            });
        }

        let extension =
            safe_extension(&command.file_name).ok_or(UploadError::AvatarExtensionUnsupported)?;
        let file_name = avatar_file_name(&command.directory, &extension)?;
        self.store.store(&file_name, &command.content).await?;

        Ok(StoredAvatar {
            file_name,
            extension,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LoadAvatarQueryHandler<S> {
    store: S,
}

impl<S> LoadAvatarQueryHandler<S>
where
    S: FileStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> AsyncQueryHandler<LoadAvatarQuery> for LoadAvatarQueryHandler<S>
where
    S: FileStore,
{
    async fn handle(&self, query: LoadAvatarQuery) -> Result<Option<Vec<u8>>, UploadError> {
        let file_name = avatar_file_name(&query.directory, &query.extension)?;
        self.store.load(&file_name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type MemoryFiles = std::sync::Arc<std::sync::Mutex<Vec<(String, Vec<u8>)>>>;

    #[derive(Clone, Default)]
    struct RecordingStore {
        stored: std::sync::Arc<std::sync::Mutex<Vec<(String, usize)>>>,
    }

    impl FileStore for RecordingStore {
        async fn store(&self, file_name: &str, content: &[u8]) -> Result<(), UploadError> {
            self.stored
                .lock()
                .expect("recording store lock must be available")
                .push((file_name.to_owned(), content.len()));
            Ok(())
        }

        async fn load(&self, _file_name: &str) -> Result<Option<Vec<u8>>, UploadError> {
            Ok(None)
        }
    }

    #[derive(Clone)]
    struct FailingStore;

    impl FileStore for FailingStore {
        async fn store(&self, _file_name: &str, _content: &[u8]) -> Result<(), UploadError> {
            Err(UploadError::Storage)
        }

        async fn load(&self, _file_name: &str) -> Result<Option<Vec<u8>>, UploadError> {
            Err(UploadError::Storage)
        }
    }

    #[derive(Clone, Default)]
    struct MemoryStore {
        files: MemoryFiles,
    }

    impl FileStore for MemoryStore {
        async fn store(&self, file_name: &str, content: &[u8]) -> Result<(), UploadError> {
            let mut files = self
                .files
                .lock()
                .expect("memory store lock must be available");
            files.push((file_name.to_owned(), content.to_vec()));
            Ok(())
        }

        async fn load(&self, file_name: &str) -> Result<Option<Vec<u8>>, UploadError> {
            let files = self
                .files
                .lock()
                .expect("memory store lock must be available");
            Ok(files
                .iter()
                .find(|(name, _)| name == file_name)
                .map(|(_, content)| content.clone()))
        }
    }

    fn command(file_name: &str, size: usize) -> UploadCommand {
        UploadCommand {
            file_name: file_name.to_owned(),
            content: vec![0; size],
        }
    }

    #[tokio::test]
    async fn handler_stores_a_file_and_keeps_the_extension() {
        let store = RecordingStore::default();
        let handler = UploadCommandHandler::new(store.clone());

        let stored = handler
            .handle(command("report.PDF", 12))
            .await
            .expect("a valid upload must be stored");

        assert!(stored.name.ends_with(".pdf"));
        assert_eq!(stored.original_name, "report.PDF");
        assert_eq!(stored.size, 12);

        let recorded = store.stored.lock().expect("lock must be available");
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].0, stored.name);
        assert_eq!(recorded[0].1, 12);
    }

    #[tokio::test]
    async fn handler_drops_an_unsafe_extension() {
        let store = RecordingStore::default();
        let handler = UploadCommandHandler::new(store);

        let stored = handler
            .handle(command("payload.tar.gz/../../etc/passwd", 1))
            .await
            .expect("a valid upload must be stored");

        assert!(!stored.name.contains('/'));
        assert!(!stored.name.contains(".."));
        assert!(!stored.name.contains('.'));
    }

    #[tokio::test]
    async fn handler_rejects_an_empty_file_name() {
        let handler = UploadCommandHandler::new(RecordingStore::default());

        let error = handler
            .handle(command("   ", 1))
            .await
            .expect_err("an empty file name must be rejected");

        assert_eq!(error, UploadError::FileNameRequired);
    }

    #[tokio::test]
    async fn handler_rejects_a_file_over_the_limit() {
        let handler = UploadCommandHandler::new(RecordingStore::default());

        let error = handler
            .handle(command("big.bin", MAX_UPLOAD_BYTES + 1))
            .await
            .expect_err("an oversized upload must be rejected");

        assert_eq!(
            error,
            UploadError::FileTooLarge {
                max: MAX_UPLOAD_BYTES
            }
        );
    }

    #[tokio::test]
    async fn handler_reports_a_storage_failure() {
        let handler = UploadCommandHandler::new(FailingStore);

        let error = handler
            .handle(command("file.txt", 1))
            .await
            .expect_err("a storage failure must be reported");

        assert_eq!(error, UploadError::Storage);
    }

    fn avatar(directory: &str, file_name: &str, size: usize) -> UploadAvatarCommand {
        UploadAvatarCommand {
            directory: directory.to_owned(),
            file_name: file_name.to_owned(),
            content: vec![7; size],
        }
    }

    #[tokio::test]
    async fn avatar_handler_stores_it_under_the_user_directory() {
        let store = RecordingStore::default();
        let handler = UploadAvatarCommandHandler::new(store.clone());

        let stored = handler
            .handle(avatar(
                "01924f0a-aaaa-7000-8000-000000000000",
                "Portrait.PNG",
                9,
            ))
            .await
            .expect("a valid avatar must be stored");

        assert_eq!(stored.extension, "png");
        assert_eq!(
            stored.file_name,
            "01924f0a-aaaa-7000-8000-000000000000/avatar.png"
        );

        let recorded = store.stored.lock().expect("lock must be available");
        assert_eq!(recorded[0].0, stored.file_name);
        assert_eq!(recorded[0].1, 9);
    }

    #[tokio::test]
    async fn avatar_handler_rejects_a_non_image_extension() {
        let handler = UploadAvatarCommandHandler::new(RecordingStore::default());

        let error = handler
            .handle(avatar("user-1", "notes.txt", 1))
            .await
            .expect_err("a non-image avatar must be rejected");

        assert_eq!(error, UploadError::AvatarExtensionUnsupported);
    }

    #[tokio::test]
    async fn avatar_handler_rejects_a_directory_traversal_attempt() {
        let handler = UploadAvatarCommandHandler::new(RecordingStore::default());

        let error = handler
            .handle(avatar("../outside", "photo.png", 1))
            .await
            .expect_err("a directory traversal attempt must be rejected");

        assert_eq!(error, UploadError::AvatarDirectoryInvalid);
    }

    #[tokio::test]
    async fn avatar_handler_rejects_a_file_over_the_limit() {
        let handler = UploadAvatarCommandHandler::new(RecordingStore::default());

        let error = handler
            .handle(avatar("user-1", "photo.png", MAX_UPLOAD_BYTES + 1))
            .await
            .expect_err("an oversized avatar must be rejected");

        assert_eq!(
            error,
            UploadError::FileTooLarge {
                max: MAX_UPLOAD_BYTES
            }
        );
    }

    #[tokio::test]
    async fn avatar_query_returns_the_stored_bytes() {
        let store = MemoryStore::default();
        let upload = UploadAvatarCommandHandler::new(store.clone());
        let load = LoadAvatarQueryHandler::new(store);

        upload
            .handle(avatar("user-1", "photo.png", 3))
            .await
            .expect("a valid avatar must be stored");

        let loaded = load
            .handle(LoadAvatarQuery {
                directory: "user-1".to_owned(),
                extension: "PNG".to_owned(),
            })
            .await
            .expect("loading a stored avatar must succeed");

        assert_eq!(loaded, Some(vec![7; 3]));
    }

    #[tokio::test]
    async fn avatar_query_reports_a_missing_file() {
        let load = LoadAvatarQueryHandler::new(MemoryStore::default());

        let loaded = load
            .handle(LoadAvatarQuery {
                directory: "user-1".to_owned(),
                extension: "png".to_owned(),
            })
            .await
            .expect("loading a missing avatar must not fail");

        assert_eq!(loaded, None);
    }

    #[tokio::test]
    async fn avatar_query_rejects_an_unsafe_directory() {
        let load = LoadAvatarQueryHandler::new(MemoryStore::default());

        let error = load
            .handle(LoadAvatarQuery {
                directory: "user-1/../user-2".to_owned(),
                extension: "png".to_owned(),
            })
            .await
            .expect_err("an unsafe directory must be rejected");

        assert_eq!(error, UploadError::AvatarDirectoryInvalid);
    }
}
