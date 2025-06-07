use crate::error::{Result, VastError};
use crate::models::Vast;
use crate::parser;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

/// Maximum depth of VAST wrapper chain to follow
const MAX_WRAPPER_DEPTH: usize = 10;

/// Unwrap a VAST document by following wrappers until an InLine ad is found
/// 
/// This function will:
/// 1. Parse the initial VAST XML
/// 2. If there's an InLine ad, return it
/// 3. If there's a Wrapper ad, fetch the VASTAdTagURI and repeat the process
/// 4. Continue until an InLine ad is found or MAX_WRAPPER_DEPTH is reached
/// 
/// If no InLine ad is found, or if any request fails, returns an empty VAST document
pub fn unwrap_vast(xml_content: &str) -> Result<Vast> {
    unwrap_vast_with_depth(xml_content, 0, &mut HashSet::new())
}

/// Internal implementation of unwrap_vast with depth tracking and cycle detection
fn unwrap_vast_with_depth(xml_content: &str, depth: usize, visited_urls: &mut HashSet<String>) -> Result<Vast> {
    // If we've reached the maximum depth, return an empty VAST
    if depth >= MAX_WRAPPER_DEPTH {
        println!("Maximum wrapper depth exceeded");
        return Ok(Vast {
            version: "4.0".to_string(), // Default to latest version
            ads: Vec::new(),
            error: Some("Maximum wrapper depth exceeded".to_string()),
        });
    }

    // Parse the VAST XML
    let vast = match parser::parse_vast(xml_content) {
        Ok(vast) => vast,
        Err(e) => {
            println!("Failed to parse VAST XML: {:?}", e);
            // If parsing fails, return an empty VAST
            return Ok(Vast {
                version: "4.0".to_string(),
                ads: Vec::new(),
                error: Some("Failed to parse VAST XML".to_string()),
            });
        }
    };

    // Process each ad in the VAST document
    let mut result_ads = Vec::new();
    
    for ad in vast.ads {
        // If the ad has an InLine element, include it in the result
        if ad.inline.is_some() {
            result_ads.push(ad);
        }
        // If the ad has a Wrapper element, follow the VASTAdTagURI
        else if let Some(wrapper) = &ad.wrapper {
            let vast_ad_tag_uri = &wrapper.vast_ad_tag_uri;
            
            println!("Following wrapper: {}", vast_ad_tag_uri);
            
            // Check for cycles (the same URL appearing more than once in the chain)
            if visited_urls.contains(vast_ad_tag_uri) {
                println!("Cycle detected in wrapper chain, skipping: {}", vast_ad_tag_uri);
                continue; // Skip this wrapper to avoid infinite loops
            }
            
            // Add this URL to the set of visited URLs
            visited_urls.insert(vast_ad_tag_uri.clone());
            
            // Fetch the next VAST document
            match fetch_vast_content(vast_ad_tag_uri) {
                Ok(next_xml) => {
                    // Recursively unwrap the next VAST document
                    match unwrap_vast_with_depth(&next_xml, depth + 1, visited_urls) {
                        Ok(next_vast) => {
                            // Add any ads from the next level to our result
                            for next_ad in next_vast.ads {
                                result_ads.push(next_ad);
                            }
                        }
                        Err(e) => {
                            println!("Error unwrapping next level: {:?}", e);
                            // If unwrapping fails, continue with the next ad
                            continue;
                        }
                    }
                }
                Err(e) => {
                    println!("Error fetching next VAST: {:?}", e);
                    // If fetching fails, continue with the next ad
                    continue;
                }
            }
        }
    }
    
    // Return a new VAST document with the collected ads
    Ok(Vast {
        version: vast.version,
        ads: result_ads,
        error: vast.error,
    })
}

/// Fetch VAST content from a URL or file path
pub fn fetch_vast_content(url_or_path: &str) -> Result<String> {
    // Check if it's a file URL
    if url_or_path.starts_with("file://") {
        let path = url_or_path.trim_start_matches("file://");
        
        #[cfg(target_os = "windows")]
        let path = path.trim_start_matches("/");
        
        // If the path doesn't exist directly, try to resolve it relative to the current directory
        let path_buf = std::path::PathBuf::from(path);
        let file_path = if path_buf.exists() {
            path_buf
        } else {
            // Check if we need to look in the samples directory
            let samples_path = std::path::PathBuf::from("samples").join(path);
            if samples_path.exists() {
                samples_path
            } else {
                // Try current directory
                std::path::PathBuf::from(path)
            }
        };
        
        println!("Reading from file: {}", file_path.display());
        return fs::read_to_string(file_path)
            .map_err(|e| VastError::IoError(e));
    }
    
    // Check if it's a plain file path
    if Path::new(url_or_path).exists() {
        println!("Reading from local file: {}", url_or_path);
        return fs::read_to_string(url_or_path)
            .map_err(|e| VastError::IoError(e));
    }
    
    // Assume it's a web URL - use a runtime to run the async function
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| VastError::Other(format!("Failed to create Tokio runtime: {}", e)))?;
    
    rt.block_on(fetch_vast_from_url(url_or_path))
}

/// Fetch VAST XML from a URL
async fn fetch_vast_from_url(url: &str) -> Result<String> {
    // Generate a random request ID for tracking in logs
    let req_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    
    // Validate URL
    let url = url::Url::parse(url).map_err(|e| VastError::UrlError(e))?;
    
    println!("[{}] Fetching from URL: {}", req_id, url);
    
    // Start timing
    let start_time = std::time::Instant::now();
    
    // Create a client with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| VastError::Other(format!("Failed to build HTTP client: {}", e)))?;
    
    // Fetch content from URL with timeout
    let response = client.get(url).send().await.map_err(|e| {
        println!("[{}] Request failed after {:?}", req_id, start_time.elapsed());
        VastError::Other(format!("Failed to fetch URL: {}", e))
    })?;
    
    println!("[{}] Received response in {:?}", req_id, start_time.elapsed());
    
    if !response.status().is_success() {
        return Err(VastError::Other(
            format!("Failed to fetch URL: HTTP status {}", response.status())
        ));
    }
    
    // Get the response body as text
    let xml_content = response.text().await.map_err(|e| {
        VastError::Other(format!("Failed to read response body: {}", e))
    })?;
    
    println!("[{}] Total request completed in {:?}", req_id, start_time.elapsed());
    
    Ok(xml_content)
} 