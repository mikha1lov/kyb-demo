
use async_trait::async_trait;

use crate::domain::Extracted;

pub mod openai;

pub struct DocumentFile {
    pub filename: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("unsupported file type: {0}")]
    Unsupported(String),
    #[error("could not reach the extraction provider")]
    Network,
    #[error("the extraction provider rejected the request ({status}): {message}")]
    Api { status: u16, message: String },
    #[error("the extraction provider returned an answer we could not read")]
    BadAnswer,
}

#[async_trait]
pub trait DocumentExtractor: Send + Sync {
    async fn extract(&self, file: DocumentFile) -> Result<Extracted, ExtractError>;
    fn label(&self) -> String;
}
