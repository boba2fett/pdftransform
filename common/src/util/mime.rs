use std::{path::Path, str::FromStr};

use mime::Mime;

pub fn get_content_type(mime_type: Option<&str>, filename: &str) -> Mime {
    if let Some(mime_type) = mime_type {
        if let Some(content_type) = Mime::from_str(mime_type).ok() {
            return content_type;
        }
    }
    if let Some(extension) = Path::new(filename).extension() {
        if let Some(extension) = extension.to_str() {
            return match extension {
                "pdf" => mime::APPLICATION_PDF,
                "png" => mime::IMAGE_PNG,
                "jpg" => mime::IMAGE_JPEG,
                "jpeg" => mime::IMAGE_JPEG,
                "bmp" => mime::IMAGE_BMP,
                _ => mime::APPLICATION_OCTET_STREAM,
            };
        }
    }
    mime::APPLICATION_OCTET_STREAM
}