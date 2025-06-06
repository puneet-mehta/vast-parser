use crate::error::{Result, VastError};
use crate::models::*;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::str::from_utf8;

/// Parse a VAST XML string into a Vast struct
pub fn parse_vast(xml: &str) -> Result<Vast> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut vast = Vast {
        version: String::new(),
        ads: Vec::new(),
        error: None,
    };
    
    // Look for the VAST element
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"VAST" => {
                // Extract version from attributes
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        if attr.key.as_ref() == b"version" {
                            if let Ok(value) = from_utf8(&attr.value) {
                                vast.version = value.to_string();
                            }
                        }
                    }
                }
                
                // If we didn't find a version attribute, error out
                if vast.version.is_empty() {
                    return Err(VastError::MissingField("VAST version".to_string()));
                }
                
                // Parse Ad elements
                vast.ads = parse_ads(&mut reader)?;
                break;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(vast)
}

/// Parse Ad elements from the VAST XML
fn parse_ads(reader: &mut Reader<&[u8]>) -> Result<Vec<Ad>> {
    let mut ads = Vec::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"Ad" => {
                // Parse a single Ad element
                let ad = parse_ad_element(reader, e)?;
                ads.push(ad);
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"VAST" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(ads)
}

/// Parse a single Ad element
fn parse_ad_element(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Result<Ad> {
    let mut ad = Ad {
        id: None,
        sequence: None,
        conditional_ad: None,
        inline: None,
        wrapper: None,
    };
    
    // Extract attributes
    for attr in start.attributes() {
        if let Ok(attr) = attr {
            match attr.key.as_ref() {
                b"id" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        ad.id = Some(value.to_string());
                    }
                },
                b"sequence" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        if let Ok(seq) = value.parse::<u32>() {
                            ad.sequence = Some(seq);
                        }
                    }
                },
                b"conditionalAd" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        ad.conditional_ad = Some(value.to_lowercase() == "true");
                    }
                },
                _ => (),
            }
        }
    }
    
    let mut buf = Vec::new();
    
    // Parse InLine or Wrapper
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"InLine" => {
                        ad.inline = Some(parse_inline_element(reader)?);
                    },
                    b"Wrapper" => {
                        ad.wrapper = Some(parse_wrapper_element(reader)?);
                    },
                    _ => {
                        // Skip other elements
                        skip_element(reader, e.name().as_ref())?;
                    }
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Ad" => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(ad)
}

/// Parse an InLine element
fn parse_inline_element(reader: &mut Reader<&[u8]>) -> Result<InLine> {
    let mut inline = InLine {
        ad_system: AdSystem {
            name: String::new(),
            version: None,
        },
        ad_title: String::new(),
        impressions: Vec::new(),
        description: None,
        advertiser: None,
        survey: None,
        error: None,
        pricing: None,
        extensions: Vec::new(),
        creatives: Vec::new(),
    };
    
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"AdSystem" => {
                        inline.ad_system = parse_ad_system(reader, e)?;
                    },
                    b"AdTitle" => {
                        inline.ad_title = read_text_element(reader)?;
                    },
                    b"Impression" => {
                        let impression = parse_impression(reader, e)?;
                        inline.impressions.push(impression);
                    },
                    b"Description" => {
                        inline.description = Some(read_text_element(reader)?);
                    },
                    b"Advertiser" => {
                        inline.advertiser = Some(read_text_element(reader)?);
                    },
                    b"Survey" => {
                        inline.survey = Some(read_text_element(reader)?);
                    },
                    b"Error" => {
                        inline.error = Some(read_text_element(reader)?);
                    },
                    b"Pricing" => {
                        inline.pricing = Some(parse_pricing(reader, e)?);
                    },
                    b"Extensions" => {
                        inline.extensions = parse_extensions(reader)?;
                    },
                    b"Creatives" => {
                        inline.creatives = parse_creatives(reader)?;
                    },
                    _ => {
                        // Skip other elements
                        skip_element(reader, e.name().as_ref())?;
                    }
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"InLine" => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(inline)
}

/// Parse a Wrapper element
fn parse_wrapper_element(reader: &mut Reader<&[u8]>) -> Result<Wrapper> {
    let mut wrapper = Wrapper {
        ad_system: AdSystem {
            name: String::new(),
            version: None,
        },
        vast_ad_tag_uri: String::new(),
        impressions: Vec::new(),
        error: None,
        extensions: Vec::new(),
        creatives: Vec::new(),
    };
    
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"AdSystem" => {
                        wrapper.ad_system = parse_ad_system(reader, e)?;
                    },
                    b"VASTAdTagURI" => {
                        wrapper.vast_ad_tag_uri = read_text_element(reader)?;
                    },
                    b"Impression" => {
                        let impression = parse_impression(reader, e)?;
                        wrapper.impressions.push(impression);
                    },
                    b"Error" => {
                        wrapper.error = Some(read_text_element(reader)?);
                    },
                    b"Extensions" => {
                        wrapper.extensions = parse_extensions(reader)?;
                    },
                    b"Creatives" => {
                        wrapper.creatives = parse_creatives(reader)?;
                    },
                    _ => {
                        // Skip other elements
                        skip_element(reader, e.name().as_ref())?;
                    }
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Wrapper" => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(wrapper)
}

/// Helper function to read the text content of an XML element
fn read_text_element(reader: &mut Reader<&[u8]>) -> Result<String> {
    let mut text = String::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => {
                text = e.unescape()?.into_owned();
            },
            Ok(Event::CData(e)) => {
                if let Ok(value) = from_utf8(&e) {
                    text = value.to_string();
                }
            },
            Ok(Event::End(_)) => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(text)
}

/// Helper function to skip an XML element and all its children
fn skip_element(reader: &mut Reader<&[u8]>, name: &[u8]) -> Result<()> {
    let mut buf = Vec::new();
    let mut depth = 0;
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == name && depth == 0 {
                    depth += 1;
                } else if depth > 0 {
                    depth += 1;
                }
            },
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == name {
                    if depth == 0 {
                        break;
                    } else {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                } else if depth > 0 {
                    depth -= 1;
                }
            },
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(())
}

/// Parse AdSystem element
fn parse_ad_system(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Result<AdSystem> {
    let mut ad_system = AdSystem {
        name: String::new(),
        version: None,
    };
    
    // Extract version attribute
    for attr in start.attributes() {
        if let Ok(attr) = attr {
            if attr.key.as_ref() == b"version" {
                if let Ok(value) = from_utf8(&attr.value) {
                    ad_system.version = Some(value.to_string());
                }
            }
        }
    }
    
    // Read the AdSystem name
    ad_system.name = read_text_element(reader)?;
    
    Ok(ad_system)
}

/// Parse Impression element
fn parse_impression(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Result<Impression> {
    let mut impression = Impression {
        id: None,
        url: String::new(),
    };
    
    // Extract id attribute
    for attr in start.attributes() {
        if let Ok(attr) = attr {
            if attr.key.as_ref() == b"id" {
                if let Ok(value) = from_utf8(&attr.value) {
                    impression.id = Some(value.to_string());
                }
            }
        }
    }
    
    // Read the Impression URL
    impression.url = read_text_element(reader)?;
    
    Ok(impression)
}

/// Parse Pricing element
fn parse_pricing(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Result<Pricing> {
    let mut pricing = Pricing {
        model: String::new(),
        currency: String::new(),
        value: String::new(),
    };
    
    // Extract attributes
    for attr in start.attributes() {
        if let Ok(attr) = attr {
            match attr.key.as_ref() {
                b"model" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        pricing.model = value.to_string();
                    }
                },
                b"currency" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        pricing.currency = value.to_string();
                    }
                },
                _ => (),
            }
        }
    }
    
    // Read the Pricing value
    pricing.value = read_text_element(reader)?;
    
    Ok(pricing)
}

/// Parse Extensions element
fn parse_extensions(reader: &mut Reader<&[u8]>) -> Result<Vec<Extension>> {
    let mut extensions = Vec::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"Extension" => {
                let extension = parse_extension(reader, e)?;
                extensions.push(extension);
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Extensions" => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(extensions)
}

/// Parse Extension element
fn parse_extension(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Result<Extension> {
    let mut extension = Extension {
        r#type: None,
        content: String::new(),
    };
    
    // Extract type attribute
    for attr in start.attributes() {
        if let Ok(attr) = attr {
            if attr.key.as_ref() == b"type" {
                if let Ok(value) = from_utf8(&attr.value) {
                    extension.r#type = Some(value.to_string());
                }
            }
        }
    }
    
    // For simplicity, we're just capturing the text content
    // In a real implementation, we might want to capture the entire XML subtree
    extension.content = read_text_element(reader)?;
    
    Ok(extension)
}

/// Parse Creatives element
fn parse_creatives(reader: &mut Reader<&[u8]>) -> Result<Vec<Creative>> {
    let mut creatives = Vec::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"Creative" => {
                let creative = parse_creative(reader, e)?;
                creatives.push(creative);
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Creatives" => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(creatives)
}

/// Parse Creative element
fn parse_creative(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Result<Creative> {
    let mut creative = Creative {
        id: None,
        sequence: None,
        ad_id: None,
        api_framework: None,
        linear: None,
        companion_ads: None,
        non_linear_ads: None,
    };
    
    // Extract attributes
    for attr in start.attributes() {
        if let Ok(attr) = attr {
            match attr.key.as_ref() {
                b"id" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        creative.id = Some(value.to_string());
                    }
                },
                b"sequence" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        if let Ok(seq) = value.parse::<u32>() {
                            creative.sequence = Some(seq);
                        }
                    }
                },
                b"adId" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        creative.ad_id = Some(value.to_string());
                    }
                },
                b"apiFramework" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        creative.api_framework = Some(value.to_string());
                    }
                },
                _ => (),
            }
        }
    }
    
    let mut buf = Vec::new();
    
    // Parse Linear, CompanionAds, or NonLinearAds
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"Linear" => {
                        creative.linear = Some(parse_linear(reader)?);
                    },
                    b"CompanionAds" => {
                        creative.companion_ads = Some(parse_companion_ads(reader)?);
                    },
                    b"NonLinearAds" => {
                        creative.non_linear_ads = Some(parse_non_linear_ads(reader)?);
                    },
                    _ => {
                        // Skip other elements
                        skip_element(reader, e.name().as_ref())?;
                    }
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Creative" => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(creative)
}

/// Parse Linear element
fn parse_linear(reader: &mut Reader<&[u8]>) -> Result<Linear> {
    let mut linear = Linear {
        duration: None,
        media_files: Vec::new(),
        video_clicks: None,
        tracking_events: Vec::new(),
    };
    
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"Duration" => {
                        linear.duration = Some(read_text_element(reader)?);
                    },
                    b"MediaFiles" => {
                        linear.media_files = parse_media_files(reader)?;
                    },
                    b"VideoClicks" => {
                        linear.video_clicks = Some(parse_video_clicks(reader)?);
                    },
                    b"TrackingEvents" => {
                        linear.tracking_events = parse_tracking_events(reader)?;
                    },
                    _ => {
                        // Skip other elements
                        skip_element(reader, e.name().as_ref())?;
                    }
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Linear" => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(linear)
}

/// Parse MediaFiles element
fn parse_media_files(reader: &mut Reader<&[u8]>) -> Result<Vec<MediaFile>> {
    let mut media_files = Vec::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"MediaFile" => {
                let media_file = parse_media_file(reader, e)?;
                media_files.push(media_file);
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"MediaFiles" => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(media_files)
}

/// Parse MediaFile element
fn parse_media_file(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Result<MediaFile> {
    let mut media_file = MediaFile {
        url: String::new(),
        mime_type: String::new(),
        codec: None,
        bitrate: None,
        width: None,
        height: None,
        delivery: None,
        r#type: None,
    };
    
    // Extract attributes
    for attr in start.attributes() {
        if let Ok(attr) = attr {
            match attr.key.as_ref() {
                b"type" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        media_file.mime_type = value.to_string();
                    }
                },
                b"codec" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        media_file.codec = Some(value.to_string());
                    }
                },
                b"bitrate" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        if let Ok(bitrate) = value.parse::<u32>() {
                            media_file.bitrate = Some(bitrate);
                        }
                    }
                },
                b"width" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        if let Ok(width) = value.parse::<u32>() {
                            media_file.width = Some(width);
                        }
                    }
                },
                b"height" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        if let Ok(height) = value.parse::<u32>() {
                            media_file.height = Some(height);
                        }
                    }
                },
                b"delivery" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        media_file.delivery = Some(value.to_string());
                    }
                },
                b"mediaType" => {
                    if let Ok(value) = from_utf8(&attr.value) {
                        media_file.r#type = Some(value.to_string());
                    }
                },
                _ => (),
            }
        }
    }
    
    // Read the MediaFile URL
    media_file.url = read_text_element(reader)?;
    
    Ok(media_file)
}

/// Parse VideoClicks element
fn parse_video_clicks(reader: &mut Reader<&[u8]>) -> Result<VideoClicks> {
    let mut video_clicks = VideoClicks {
        click_through: None,
        click_tracking: Vec::new(),
        custom_click: Vec::new(),
    };
    
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"ClickThrough" => {
                        video_clicks.click_through = Some(read_text_element(reader)?);
                    },
                    b"ClickTracking" => {
                        video_clicks.click_tracking.push(read_text_element(reader)?);
                    },
                    b"CustomClick" => {
                        video_clicks.custom_click.push(read_text_element(reader)?);
                    },
                    _ => {
                        // Skip other elements
                        skip_element(reader, e.name().as_ref())?;
                    }
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"VideoClicks" => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(video_clicks)
}

/// Parse TrackingEvents element
fn parse_tracking_events(reader: &mut Reader<&[u8]>) -> Result<Vec<TrackingEvent>> {
    let mut tracking_events = Vec::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"Tracking" => {
                let tracking_event = parse_tracking_event(reader, e)?;
                tracking_events.push(tracking_event);
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"TrackingEvents" => break,
            Ok(Event::Eof) => {
                return Err(VastError::Other("Unexpected end of file".to_string()));
            },
            Err(e) => return Err(VastError::XmlParseError(e)),
            _ => (),
        }
        buf.clear();
    }
    
    Ok(tracking_events)
}

/// Parse Tracking element
fn parse_tracking_event(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Result<TrackingEvent> {
    let mut tracking_event = TrackingEvent {
        event: String::new(),
        url: String::new(),
    };
    
    // Extract event attribute
    for attr in start.attributes() {
        if let Ok(attr) = attr {
            if attr.key.as_ref() == b"event" {
                if let Ok(value) = from_utf8(&attr.value) {
                    tracking_event.event = value.to_string();
                }
            }
        }
    }
    
    // Read the Tracking URL
    tracking_event.url = read_text_element(reader)?;
    
    Ok(tracking_event)
}

/// Parse CompanionAds element
fn parse_companion_ads(reader: &mut Reader<&[u8]>) -> Result<CompanionAds> {
    // Placeholder implementation
    let companion_ads = CompanionAds {
        companions: Vec::new(),
    };
    
    // Skip to the end of the CompanionAds element
    skip_element(reader, b"CompanionAds")?;
    
    Ok(companion_ads)
}

/// Parse NonLinearAds element
fn parse_non_linear_ads(reader: &mut Reader<&[u8]>) -> Result<NonLinearAds> {
    // Placeholder implementation
    let non_linear_ads = NonLinearAds {
        non_linears: Vec::new(),
    };
    
    // Skip to the end of the NonLinearAds element
    skip_element(reader, b"NonLinearAds")?;
    
    Ok(non_linear_ads)
} 