pub mod models;
pub mod parser;
pub mod error;
pub mod unwrap;
pub mod stitcher;

pub mod async_api {
    use crate::error::Result;
    use crate::models::Vast;

    pub async fn parse_vast(xml: &str) -> Result<Vast> {
        // Parsing is CPU-bound, so we can just wrap the sync version
        crate::parser::parse_vast(xml)
    }

    pub async fn unwrap_vast(xml_content: &str) -> Result<Vast> {
        crate::unwrap::unwrap_vast_async(xml_content).await
    }

    pub async fn stitch_vast(xml_content: &str) -> Result<String> {
        crate::stitcher::stitch_vast_async(xml_content).await
    }
} 