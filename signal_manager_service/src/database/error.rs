use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Document not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Firestore error: {0}")]
    Firestore(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>; 