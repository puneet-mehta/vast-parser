use thiserror::Error;

/// Errors that can occur when parsing VAST XML
#[derive(Error, Debug)]
pub enum VastError {
    #[error("Failed to parse XML: {0}")]
    XmlParseError(#[from] quick_xml::Error),
    
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid VAST version: {0}")]
    InvalidVersion(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("URL error: {0}")]
    UrlError(#[from] url::ParseError),
    
    #[error("Unsupported VAST feature: {0}")]
    UnsupportedFeature(String),
    
    #[error("Unknown error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, VastError>; 