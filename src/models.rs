use serde::{Deserialize, Serialize};

/// Represents a VAST document (Video Ad Serving Template)
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Vast {
    /// The VAST version (e.g., "2.0", "3.0", "4.0", etc.)
    pub version: String,
    
    /// The Ad elements within the VAST document
    pub ads: Vec<Ad>,
    
    /// Any error information if present
    pub error: Option<String>,
}

/// Represents an Ad within a VAST document
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Ad {
    /// The ad ID
    pub id: Option<String>,
    
    /// The ad sequence number (for ad pods)
    pub sequence: Option<u32>,
    
    /// The conditional ad flag (VAST 4.0+)
    pub conditional_ad: Option<bool>,
    
    /// The in-line ad details
    pub inline: Option<InLine>,
    
    /// The wrapper ad details
    pub wrapper: Option<Wrapper>,
}

/// Represents an InLine ad, which contains all the media files and tracking information
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct InLine {
    /// The ad system name and version
    pub ad_system: AdSystem,
    
    /// The ad title
    pub ad_title: String,
    
    /// Impression tracking URLs
    pub impressions: Vec<Impression>,
    
    /// The description of the ad
    pub description: Option<String>,
    
    /// The advertiser name
    pub advertiser: Option<String>,
    
    /// The survey URL
    pub survey: Option<String>,
    
    /// Error tracking URLs
    pub error: Option<String>,
    
    /// Pricing information
    pub pricing: Option<Pricing>,
    
    /// Extensions
    pub extensions: Vec<Extension>,
    
    /// Creative elements
    pub creatives: Vec<Creative>,
}

/// Represents a Wrapper ad, which references another VAST document
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Wrapper {
    /// The ad system name and version
    pub ad_system: AdSystem,
    
    /// The URL of the next VAST document
    pub vast_ad_tag_uri: String,
    
    /// Impression tracking URLs
    pub impressions: Vec<Impression>,
    
    /// Error tracking URLs
    pub error: Option<String>,
    
    /// Extensions
    pub extensions: Vec<Extension>,
    
    /// Creative elements
    pub creatives: Vec<Creative>,
}

/// Represents the ad system information
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AdSystem {
    /// The ad system name
    pub name: String,
    
    /// The ad system version
    pub version: Option<String>,
}

/// Represents an impression tracking URL
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Impression {
    /// The impression ID
    pub id: Option<String>,
    
    /// The impression tracking URL
    pub url: String,
}

/// Represents pricing information
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Pricing {
    /// The pricing model (e.g., "CPM", "CPC", etc.)
    pub model: String,
    
    /// The pricing currency (e.g., "USD", "EUR", etc.)
    pub currency: String,
    
    /// The price value
    pub value: String,
}

/// Represents an extension
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Extension {
    /// The extension type
    pub r#type: Option<String>,
    
    /// The extension content
    pub content: String,
}

/// Represents a creative element
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Creative {
    /// The creative ID
    pub id: Option<String>,
    
    /// The creative sequence number
    pub sequence: Option<u32>,
    
    /// The creative ad ID
    pub ad_id: Option<String>,
    
    /// The creative API framework
    pub api_framework: Option<String>,
    
    /// Linear ad details
    pub linear: Option<Linear>,
    
    /// CompanionAds details
    pub companion_ads: Option<CompanionAds>,
    
    /// NonLinearAds details
    pub non_linear_ads: Option<NonLinearAds>,
}

/// Represents a linear ad
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Linear {
    /// The duration of the ad
    pub duration: Option<String>,
    
    /// Media files
    pub media_files: Vec<MediaFile>,
    
    /// Video clicks
    pub video_clicks: Option<VideoClicks>,
    
    /// Tracking events
    pub tracking_events: Vec<TrackingEvent>,
}

/// Represents a media file
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct MediaFile {
    /// The media file URL
    pub url: String,
    
    /// The media file MIME type
    pub mime_type: String,
    
    /// The media file codec
    pub codec: Option<String>,
    
    /// The media file bitrate
    pub bitrate: Option<u32>,
    
    /// The media file width
    pub width: Option<u32>,
    
    /// The media file height
    pub height: Option<u32>,
    
    /// The media file delivery type (progressive or streaming)
    pub delivery: Option<String>,
    
    /// The media file type (video or audio)
    pub r#type: Option<String>,
}

/// Represents video click-through and click-tracking URLs
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VideoClicks {
    /// The click-through URL
    pub click_through: Option<String>,
    
    /// Click tracking URLs
    pub click_tracking: Vec<String>,
    
    /// Custom click URLs
    pub custom_click: Vec<String>,
}

/// Represents a tracking event
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TrackingEvent {
    /// The event type (e.g., "start", "firstQuartile", "midpoint", "thirdQuartile", "complete", etc.)
    pub event: String,
    
    /// The tracking URL
    pub url: String,
}

/// Represents companion ads
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CompanionAds {
    /// The companion ads
    pub companions: Vec<Companion>,
}

/// Represents a companion ad
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Companion {
    /// The companion ID
    pub id: Option<String>,
    
    /// The companion width
    pub width: u32,
    
    /// The companion height
    pub height: u32,
    
    /// The companion asset type (StaticResource, IFrameResource, or HTMLResource)
    pub resource_type: String,
    
    /// The companion resource URL or HTML content
    pub resource: String,
    
    /// The companion click-through URL
    pub click_through: Option<String>,
    
    /// Companion tracking events
    pub tracking_events: Vec<TrackingEvent>,
}

/// Represents non-linear ads
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NonLinearAds {
    /// The non-linear ads
    pub non_linears: Vec<NonLinear>,
}

/// Represents a non-linear ad
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NonLinear {
    /// The non-linear ID
    pub id: Option<String>,
    
    /// The non-linear width
    pub width: u32,
    
    /// The non-linear height
    pub height: u32,
    
    /// The non-linear expandable width
    pub expand_width: Option<u32>,
    
    /// The non-linear expandable height
    pub expand_height: Option<u32>,
    
    /// The non-linear scalable flag
    pub scalable: Option<bool>,
    
    /// The non-linear maintain aspect ratio flag
    pub maintain_aspect_ratio: Option<bool>,
    
    /// The non-linear asset type (StaticResource, IFrameResource, or HTMLResource)
    pub resource_type: String,
    
    /// The non-linear resource URL or HTML content
    pub resource: String,
    
    /// The non-linear click-through URL
    pub click_through: Option<String>,
} 