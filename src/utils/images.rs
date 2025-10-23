use crate::error::{AdapterError, Result};
use base64::{engine::general_purpose, Engine as _};

pub fn process_image_url_anthropic(url: &str) -> Result<(String, String)> {
    if url.starts_with("data:") {
        let parts: Vec<&str> = url.split(',').collect();
        if parts.len() != 2 {
            return Err(AdapterError::ConfigError(
                "Invalid data URI format".to_string(),
            ));
        }

        let metadata = parts[0];
        let base64_data = parts[1];

        let media_type = if metadata.contains("image/png") {
            "image/png"
        } else if metadata.contains("image/jpeg") || metadata.contains("image/jpg") {
            "image/jpeg"
        } else if metadata.contains("image/gif") {
            "image/gif"
        } else if metadata.contains("image/webp") {
            "image/webp"
        } else {
            "image/jpeg"
        };

        Ok((media_type.to_string(), base64_data.to_string()))
    } else {
        Err(AdapterError::ConfigError(
            "Only data URIs are supported for Anthropic".to_string(),
        ))
    }
}

pub fn encode_image_to_base64(image_bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(image_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_image_url_anthropic() {
        let url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        let result = process_image_url_anthropic(url).unwrap();
        assert_eq!(result.0, "image/png");
        assert_eq!(result.1, "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==");
    }

    #[test]
    fn test_encode_image_to_base64() {
        let data = b"hello world";
        let encoded = encode_image_to_base64(data);
        assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
    }
}
