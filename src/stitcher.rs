use crate::error::Result;
use crate::models::*;
use crate::unwrap;
use std::collections::{HashMap, VecDeque};

/// Stitch together a new VAST XML that combines the InLine ad with all the wrapper chain elements
pub fn stitch_vast(xml_content: &str) -> Result<String> {
    // First, collect all wrapper tracking elements
    let wrapper_tracking = collect_wrapper_tracking(xml_content)?;
    
    // Then, unwrap the VAST to find the InLine ad
    let unwrapped_vast = unwrap::unwrap_vast(xml_content)?;
    
    // Now stitch together the final VAST
    let stitched_vast = stitch_vast_from_unwrapped(unwrapped_vast, wrapper_tracking)?;
    
    // Convert to XML
    vast_to_xml(&stitched_vast)
}

/// Async version of stitch_vast that combines the InLine ad with all the wrapper chain elements
pub async fn stitch_vast_async(xml_content: &str) -> Result<String> {
    // First, collect all wrapper tracking elements asynchronously
    let wrapper_tracking = collect_wrapper_tracking_async(xml_content).await?;
    
    // Then, unwrap the VAST to find the InLine ad asynchronously
    let unwrapped_vast = unwrap::unwrap_vast_async(xml_content).await?;
    
    // Now stitch together the final VAST
    let stitched_vast = stitch_vast_from_unwrapped(unwrapped_vast, wrapper_tracking)?;
    
    // Convert to XML
    vast_to_xml(&stitched_vast)
}

/// Structure to hold wrapper tracking information
#[derive(Default)]
struct WrapperTracking {
    impressions: Vec<Impression>,
    error_urls: Vec<String>,
    tracking_events: HashMap<String, Vec<String>>, // event -> URLs
    click_tracking: Vec<String>,
    custom_click: Vec<String>,
}

/// Collect tracking information from all wrappers in the chain
fn collect_wrapper_tracking(xml_content: &str) -> Result<WrapperTracking> {
    let mut result = WrapperTracking::default();
    collect_wrapper_tracking_recursive(xml_content, &mut result, &mut Vec::new())?;
    Ok(result)
}

/// Async version to collect tracking information from all wrappers in the chain
/// Uses an iterative approach instead of recursion to avoid issues with async recursion
async fn collect_wrapper_tracking_async(xml_content: &str) -> Result<WrapperTracking> {
    let mut result = WrapperTracking::default();
    let mut visited_urls = Vec::new();
    
    // Use a queue for breadth-first traversal instead of recursion
    let mut queue = VecDeque::new();
    queue.push_back(xml_content.to_string());
    
    while let Some(current_xml) = queue.pop_front() {
        // Parse the VAST XML
        let vast = crate::parser::parse_vast(&current_xml)?;
        
        // Process each ad
        for ad in vast.ads {
            if let Some(wrapper) = ad.wrapper {
                // Extract tracking information from this wrapper
                extract_wrapper_tracking(&wrapper, &mut result);
                
                // Check if we've seen this URL before to avoid cycles
                if visited_urls.contains(&wrapper.vast_ad_tag_uri) {
                    continue;
                }
                
                // Add this URL to the visited list
                visited_urls.push(wrapper.vast_ad_tag_uri.clone());
                
                // Fetch the next VAST XML asynchronously
                match unwrap::fetch_vast_content_async(&wrapper.vast_ad_tag_uri).await {
                    Ok(next_xml) => {
                        // Add to the queue for processing
                        queue.push_back(next_xml);
                    }
                    Err(_) => {
                        // If we can't fetch the next XML, just continue
                        continue;
                    }
                }
            }
        }
    }
    
    Ok(result)
}

/// Helper function to recursively collect wrapper tracking
fn collect_wrapper_tracking_recursive(
    xml_content: &str, 
    result: &mut WrapperTracking,
    visited_urls: &mut Vec<String>
) -> Result<()> {
    // Parse the VAST XML
    let vast = crate::parser::parse_vast(xml_content)?;
    
    // Process each ad
    for ad in vast.ads {
        if let Some(wrapper) = ad.wrapper {
            // Extract tracking information from this wrapper
            extract_wrapper_tracking(&wrapper, result);
            
            // Check if we've seen this URL before to avoid cycles
            if visited_urls.contains(&wrapper.vast_ad_tag_uri) {
                continue;
            }
            
            // Add this URL to the visited list
            visited_urls.push(wrapper.vast_ad_tag_uri.clone());
            
            // Fetch the next VAST XML
            match fetch_vast_content(&wrapper.vast_ad_tag_uri) {
                Ok(next_xml) => {
                    // Recursively collect tracking from the next level
                    collect_wrapper_tracking_recursive(&next_xml, result, visited_urls)?;
                }
                Err(_) => {
                    // If we can't fetch the next XML, just continue
                    continue;
                }
            }
        }
    }
    
    Ok(())
}

/// Extract tracking information from a wrapper
fn extract_wrapper_tracking(wrapper: &Wrapper, result: &mut WrapperTracking) {
    // Add impressions
    for impression in &wrapper.impressions {
        result.impressions.push(impression.clone());
    }
    
    // Add error URL if present
    if let Some(error) = &wrapper.error {
        result.error_urls.push(error.clone());
    }
    
    // Process creatives
    for creative in &wrapper.creatives {
        if let Some(linear) = &creative.linear {
            // Add tracking events
            for event in &linear.tracking_events {
                result.tracking_events
                    .entry(event.event.clone())
                    .or_insert_with(Vec::new)
                    .push(event.url.clone());
            }
            
            // Add video clicks
            if let Some(video_clicks) = &linear.video_clicks {
                // Add click tracking
                for url in &video_clicks.click_tracking {
                    result.click_tracking.push(url.clone());
                }
                
                // Add custom click
                for url in &video_clicks.custom_click {
                    result.custom_click.push(url.clone());
                }
            }
        }
    }
}

/// Stitch a VAST document from unwrapped VAST and wrapper tracking
fn stitch_vast_from_unwrapped(
    mut unwrapped_vast: Vast,
    wrapper_tracking: WrapperTracking
) -> Result<Vast> {
    // Process each ad
    for ad in &mut unwrapped_vast.ads {
        if let Some(inline) = &mut ad.inline {
            // Add wrapper impressions
            for impression in &wrapper_tracking.impressions {
                inline.impressions.push(impression.clone());
            }
            
            // Add wrapper error URL if inline doesn't have one
            if inline.error.is_none() && !wrapper_tracking.error_urls.is_empty() {
                inline.error = Some(wrapper_tracking.error_urls[0].clone());
            }
            
            // Process creatives
            for creative in &mut inline.creatives {
                if let Some(linear) = &mut creative.linear {
                    // Add wrapper tracking events
                    for (event, urls) in &wrapper_tracking.tracking_events {
                        for url in urls {
                            linear.tracking_events.push(TrackingEvent {
                                event: event.clone(),
                                url: url.clone(),
                            });
                        }
                    }
                    
                    // Process video clicks
                    if let Some(video_clicks) = &mut linear.video_clicks {
                        // Add wrapper click tracking
                        for url in &wrapper_tracking.click_tracking {
                            video_clicks.click_tracking.push(url.clone());
                        }
                        
                        // Add wrapper custom click
                        for url in &wrapper_tracking.custom_click {
                            video_clicks.custom_click.push(url.clone());
                        }
                    } else if !wrapper_tracking.click_tracking.is_empty() || !wrapper_tracking.custom_click.is_empty() {
                        // Create video clicks if it doesn't exist
                        linear.video_clicks = Some(VideoClicks {
                            click_through: None,
                            click_tracking: wrapper_tracking.click_tracking.clone(),
                            custom_click: wrapper_tracking.custom_click.clone(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(unwrapped_vast)
}

/// Convert a Vast struct to XML
fn vast_to_xml(vast: &Vast) -> Result<String> {
    let mut xml = String::new();
    
    // XML declaration
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    
    // VAST root element
    xml.push_str(&format!("<VAST version=\"{}\">\n", vast.version));
    
    // Error URL if present
    if let Some(error) = &vast.error {
        xml.push_str(&format!("  <Error><![CDATA[{}]]></Error>\n", error));
    }
    
    // Add ads
    for ad in &vast.ads {
        xml.push_str(&ad_to_xml(ad));
    }
    
    // Close VAST element
    xml.push_str("</VAST>");
    
    Ok(xml)
}

/// Convert an Ad to XML
fn ad_to_xml(ad: &Ad) -> String {
    let mut xml = String::new();
    
    // Open Ad element with attributes
    xml.push_str("  <Ad");
    if let Some(id) = &ad.id {
        xml.push_str(&format!(" id=\"{}\"", id));
    }
    if let Some(sequence) = &ad.sequence {
        xml.push_str(&format!(" sequence=\"{}\"", sequence));
    }
    if let Some(conditional_ad) = &ad.conditional_ad {
        xml.push_str(&format!(" conditionalAd=\"{}\"", conditional_ad));
    }
    xml.push_str(">\n");
    
    // Add InLine or Wrapper
    if let Some(inline) = &ad.inline {
        xml.push_str(&inline_to_xml(inline));
    } else if let Some(wrapper) = &ad.wrapper {
        xml.push_str(&wrapper_to_xml(wrapper));
    }
    
    // Close Ad element
    xml.push_str("  </Ad>\n");
    
    xml
}

/// Convert an InLine to XML
fn inline_to_xml(inline: &InLine) -> String {
    let mut xml = String::new();
    
    // Open InLine element
    xml.push_str("    <InLine>\n");
    
    // Add AdSystem
    xml.push_str("      <AdSystem");
    if let Some(version) = &inline.ad_system.version {
        xml.push_str(&format!(" version=\"{}\"", version));
    }
    xml.push_str(&format!(">{}</AdSystem>\n", inline.ad_system.name));
    
    // Add AdTitle
    xml.push_str(&format!("      <AdTitle>{}</AdTitle>\n", inline.ad_title));
    
    // Add Description if present
    if let Some(description) = &inline.description {
        xml.push_str(&format!("      <Description>{}</Description>\n", description));
    }
    
    // Add Advertiser if present
    if let Some(advertiser) = &inline.advertiser {
        xml.push_str(&format!("      <Advertiser>{}</Advertiser>\n", advertiser));
    }
    
    // Add Survey if present
    if let Some(survey) = &inline.survey {
        xml.push_str(&format!("      <Survey><![CDATA[{}]]></Survey>\n", survey));
    }
    
    // Add Impressions
    for impression in &inline.impressions {
        xml.push_str("      <Impression");
        if let Some(id) = &impression.id {
            xml.push_str(&format!(" id=\"{}\"", id));
        }
        xml.push_str(&format!("><![CDATA[{}]]></Impression>\n", impression.url));
    }
    
    // Add Error if present
    if let Some(error) = &inline.error {
        xml.push_str(&format!("      <Error><![CDATA[{}]]></Error>\n", error));
    }
    
    // Add Pricing if present
    if let Some(pricing) = &inline.pricing {
        xml.push_str(&format!("      <Pricing model=\"{}\" currency=\"{}\">{}</Pricing>\n",
                             pricing.model, pricing.currency, pricing.value));
    }
    
    // Add Extensions if present
    if !inline.extensions.is_empty() {
        xml.push_str("      <Extensions>\n");
        for extension in &inline.extensions {
            xml.push_str("        <Extension");
            if let Some(extension_type) = &extension.r#type {
                xml.push_str(&format!(" type=\"{}\"", extension_type));
            }
            xml.push_str(&format!(">{}</Extension>\n", extension.content));
        }
        xml.push_str("      </Extensions>\n");
    }
    
    // Add Creatives
    if !inline.creatives.is_empty() {
        xml.push_str("      <Creatives>\n");
        for creative in &inline.creatives {
            xml.push_str(&creative_to_xml(creative));
        }
        xml.push_str("      </Creatives>\n");
    }
    
    // Close InLine element
    xml.push_str("    </InLine>\n");
    
    xml
}

/// Convert a Wrapper to XML
fn wrapper_to_xml(wrapper: &Wrapper) -> String {
    let mut xml = String::new();
    
    // Open Wrapper element
    xml.push_str("    <Wrapper>\n");
    
    // Add AdSystem
    xml.push_str("      <AdSystem");
    if let Some(version) = &wrapper.ad_system.version {
        xml.push_str(&format!(" version=\"{}\"", version));
    }
    xml.push_str(&format!(">{}</AdSystem>\n", wrapper.ad_system.name));
    
    // Add VASTAdTagURI
    xml.push_str(&format!("      <VASTAdTagURI><![CDATA[{}]]></VASTAdTagURI>\n", wrapper.vast_ad_tag_uri));
    
    // Add Impressions
    for impression in &wrapper.impressions {
        xml.push_str("      <Impression");
        if let Some(id) = &impression.id {
            xml.push_str(&format!(" id=\"{}\"", id));
        }
        xml.push_str(&format!("><![CDATA[{}]]></Impression>\n", impression.url));
    }
    
    // Add Error if present
    if let Some(error) = &wrapper.error {
        xml.push_str(&format!("      <Error><![CDATA[{}]]></Error>\n", error));
    }
    
    // Add Creatives
    if !wrapper.creatives.is_empty() {
        xml.push_str("      <Creatives>\n");
        for creative in &wrapper.creatives {
            xml.push_str(&creative_to_xml(creative));
        }
        xml.push_str("      </Creatives>\n");
    }
    
    // Close Wrapper element
    xml.push_str("    </Wrapper>\n");
    
    xml
}

/// Convert a Creative to XML
fn creative_to_xml(creative: &Creative) -> String {
    let mut xml = String::new();
    
    // Open Creative element with attributes
    xml.push_str("        <Creative");
    if let Some(id) = &creative.id {
        xml.push_str(&format!(" id=\"{}\"", id));
    }
    if let Some(sequence) = &creative.sequence {
        xml.push_str(&format!(" sequence=\"{}\"", sequence));
    }
    if let Some(ad_id) = &creative.ad_id {
        xml.push_str(&format!(" adId=\"{}\"", ad_id));
    }
    if let Some(api_framework) = &creative.api_framework {
        xml.push_str(&format!(" apiFramework=\"{}\"", api_framework));
    }
    xml.push_str(">\n");
    
    // Add Linear if present
    if let Some(linear) = &creative.linear {
        xml.push_str(&linear_to_xml(linear));
    }
    
    // Add CompanionAds if present
    if let Some(companion_ads) = &creative.companion_ads {
        xml.push_str(&companion_ads_to_xml(companion_ads));
    }
    
    // Add NonLinearAds if present
    if let Some(non_linear_ads) = &creative.non_linear_ads {
        xml.push_str(&non_linear_ads_to_xml(non_linear_ads));
    }
    
    // Close Creative element
    xml.push_str("        </Creative>\n");
    
    xml
}

/// Convert a Linear to XML
fn linear_to_xml(linear: &Linear) -> String {
    let mut xml = String::new();
    
    // Open Linear element
    xml.push_str("          <Linear>\n");
    
    // Add Duration if present
    if let Some(duration) = &linear.duration {
        xml.push_str(&format!("            <Duration>{}</Duration>\n", duration));
    }
    
    // Add TrackingEvents if present
    if !linear.tracking_events.is_empty() {
        xml.push_str("            <TrackingEvents>\n");
        for event in &linear.tracking_events {
            xml.push_str(&format!("              <Tracking event=\"{}\"><![CDATA[{}]]></Tracking>\n",
                                 event.event, event.url));
        }
        xml.push_str("            </TrackingEvents>\n");
    }
    
    // Add VideoClicks if present
    if let Some(video_clicks) = &linear.video_clicks {
        xml.push_str("            <VideoClicks>\n");
        
        if let Some(click_through) = &video_clicks.click_through {
            xml.push_str(&format!("              <ClickThrough><![CDATA[{}]]></ClickThrough>\n", click_through));
        }
        
        for url in &video_clicks.click_tracking {
            xml.push_str(&format!("              <ClickTracking><![CDATA[{}]]></ClickTracking>\n", url));
        }
        
        for url in &video_clicks.custom_click {
            xml.push_str(&format!("              <CustomClick><![CDATA[{}]]></CustomClick>\n", url));
        }
        
        xml.push_str("            </VideoClicks>\n");
    }
    
    // Add MediaFiles if present
    if !linear.media_files.is_empty() {
        xml.push_str("            <MediaFiles>\n");
        for media_file in &linear.media_files {
            xml.push_str("              <MediaFile");
            
            // Add attributes
            xml.push_str(&format!(" type=\"{}\"", media_file.mime_type));
            
            if let Some(delivery) = &media_file.delivery {
                xml.push_str(&format!(" delivery=\"{}\"", delivery));
            }
            
            if let Some(width) = &media_file.width {
                xml.push_str(&format!(" width=\"{}\"", width));
            }
            
            if let Some(height) = &media_file.height {
                xml.push_str(&format!(" height=\"{}\"", height));
            }
            
            if let Some(codec) = &media_file.codec {
                xml.push_str(&format!(" codec=\"{}\"", codec));
            }
            
            if let Some(bitrate) = &media_file.bitrate {
                xml.push_str(&format!(" bitrate=\"{}\"", bitrate));
            }
            
            xml.push_str(&format!("><![CDATA[{}]]></MediaFile>\n", media_file.url));
        }
        xml.push_str("            </MediaFiles>\n");
    }
    
    // Close Linear element
    xml.push_str("          </Linear>\n");
    
    xml
}

/// Convert a CompanionAds to XML
fn companion_ads_to_xml(companion_ads: &CompanionAds) -> String {
    let mut xml = String::new();
    
    // Open CompanionAds element
    xml.push_str("          <CompanionAds>\n");
    
    // Add Companions
    for companion in &companion_ads.companions {
        xml.push_str("            <Companion");
        
        // Add attributes
        if let Some(id) = &companion.id {
            xml.push_str(&format!(" id=\"{}\"", id));
        }
        
        xml.push_str(&format!(" width=\"{}\" height=\"{}\"", companion.width, companion.height));
        
        xml.push_str(">\n");
        
        // Add resource based on type
        if companion.resource_type == "StaticResource" {
            xml.push_str(&format!("              <StaticResource><![CDATA[{}]]></StaticResource>\n", companion.resource));
        } else if companion.resource_type == "IFrameResource" {
            xml.push_str(&format!("              <IFrameResource><![CDATA[{}]]></IFrameResource>\n", companion.resource));
        } else if companion.resource_type == "HTMLResource" {
            xml.push_str(&format!("              <HTMLResource><![CDATA[{}]]></HTMLResource>\n", companion.resource));
        }
        
        // Add ClickThrough if present
        if let Some(click_through) = &companion.click_through {
            xml.push_str(&format!("              <CompanionClickThrough><![CDATA[{}]]></CompanionClickThrough>\n", click_through));
        }
        
        // Add TrackingEvents if present
        if !companion.tracking_events.is_empty() {
            xml.push_str("              <TrackingEvents>\n");
            for event in &companion.tracking_events {
                xml.push_str(&format!("                <Tracking event=\"{}\"><![CDATA[{}]]></Tracking>\n",
                                     event.event, event.url));
            }
            xml.push_str("              </TrackingEvents>\n");
        }
        
        // Close Companion element
        xml.push_str("            </Companion>\n");
    }
    
    // Close CompanionAds element
    xml.push_str("          </CompanionAds>\n");
    
    xml
}

/// Convert a NonLinearAds to XML
fn non_linear_ads_to_xml(non_linear_ads: &NonLinearAds) -> String {
    let mut xml = String::new();
    
    // Open NonLinearAds element
    xml.push_str("          <NonLinearAds>\n");
    
    // Add NonLinear elements
    for non_linear in &non_linear_ads.non_linears {
        xml.push_str("            <NonLinear");
        
        // Add attributes
        if let Some(id) = &non_linear.id {
            xml.push_str(&format!(" id=\"{}\"", id));
        }
        
        xml.push_str(&format!(" width=\"{}\" height=\"{}\"", non_linear.width, non_linear.height));
        
        if let Some(expand_width) = &non_linear.expand_width {
            xml.push_str(&format!(" expandedWidth=\"{}\"", expand_width));
        }
        
        if let Some(expand_height) = &non_linear.expand_height {
            xml.push_str(&format!(" expandedHeight=\"{}\"", expand_height));
        }
        
        if let Some(scalable) = &non_linear.scalable {
            xml.push_str(&format!(" scalable=\"{}\"", scalable));
        }
        
        if let Some(maintain_aspect_ratio) = &non_linear.maintain_aspect_ratio {
            xml.push_str(&format!(" maintainAspectRatio=\"{}\"", maintain_aspect_ratio));
        }
        
        xml.push_str(">\n");
        
        // Add resource based on type
        if non_linear.resource_type == "StaticResource" {
            xml.push_str(&format!("              <StaticResource><![CDATA[{}]]></StaticResource>\n", non_linear.resource));
        } else if non_linear.resource_type == "IFrameResource" {
            xml.push_str(&format!("              <IFrameResource><![CDATA[{}]]></IFrameResource>\n", non_linear.resource));
        } else if non_linear.resource_type == "HTMLResource" {
            xml.push_str(&format!("              <HTMLResource><![CDATA[{}]]></HTMLResource>\n", non_linear.resource));
        }
        
        // Add ClickThrough if present
        if let Some(click_through) = &non_linear.click_through {
            xml.push_str(&format!("              <NonLinearClickThrough><![CDATA[{}]]></NonLinearClickThrough>\n", click_through));
        }
        
        // Close NonLinear element
        xml.push_str("            </NonLinear>\n");
    }
    
    // Close NonLinearAds element
    xml.push_str("          </NonLinearAds>\n");
    
    xml
}

/// Helper function that calls unwrap module's fetch_vast_content
fn fetch_vast_content(url_or_path: &str) -> Result<String> {
    unwrap::fetch_vast_content(url_or_path)
} 