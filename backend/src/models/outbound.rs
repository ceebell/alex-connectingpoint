use crate::error::AppError;
use base64::{Engine, engine::general_purpose::STANDARD};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

// Stickers the Messaging API may send: (package ID, first sticker ID, count).
// Source: https://developers.line.biz/en/docs/messaging-api/sticker-list/
// (IDs confirmed contiguous from each package's LINE store metadata).
pub const STICKER_PACKAGES: [(u64, u64, u64); 15] = [
    (446, 1988, 40),
    (789, 10855, 40),
    (1070, 17839, 40),
    (6136, 10551376, 24),
    (6325, 10979904, 24),
    (6359, 11069848, 24),
    (6362, 11087920, 24),
    (6370, 11088016, 24),
    (6632, 11825374, 24),
    (8515, 16581242, 24),
    (8522, 16581266, 24),
    (8525, 16581290, 24),
    (11537, 52002734, 40),
    (11538, 51626494, 40),
    (11539, 52114110, 40),
];

pub fn valid_sticker(package: &str, sticker: &str) -> bool {
    let (Ok(package), Ok(sticker)) = (package.parse::<u64>(), sticker.parse::<u64>()) else {
        return false;
    };
    STICKER_PACKAGES
        .iter()
        .any(|&(p, first, count)| p == package && (first..first + count).contains(&sticker))
}

// LINE requires X-Line-Retry-Key to be a UUID; reusing it makes retries safe.
pub fn valid_request_id(value: &str) -> bool {
    value.len() == 36
        && value.char_indices().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == '-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StickerRef {
    pub package_id: String,
    pub sticker_id: String,
}

// Exactly one of text, sticker or image_id.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SendInput {
    pub request_id: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub sticker: Option<StickerRef>,
    #[serde(default)]
    pub image_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Outgoing {
    Text(String),
    Sticker {
        package_id: String,
        sticker_id: String,
    },
    Image(ObjectId),
}

impl SendInput {
    pub fn validate(self) -> Result<(String, Outgoing), AppError> {
        let invalid = AppError::Validation("Invalid reply message");
        if !valid_request_id(&self.request_id) {
            return Err(invalid);
        }
        let outgoing = match (self.text, self.sticker, self.image_id) {
            (Some(text), None, None) => {
                let text = text.trim().to_string();
                if text.is_empty()
                    || text.chars().count() > 5000
                    || text
                        .chars()
                        .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
                {
                    return Err(invalid);
                }
                Outgoing::Text(text)
            }
            (None, Some(sticker), None)
                if valid_sticker(&sticker.package_id, &sticker.sticker_id) =>
            {
                Outgoing::Sticker {
                    package_id: sticker.package_id,
                    sticker_id: sticker.sticker_id,
                }
            }
            (None, None, Some(id)) => {
                Outgoing::Image(ObjectId::parse_str(&id).map_err(|_| AppError::BadRequest)?)
            }
            _ => return Err(invalid),
        };
        Ok((self.request_id.to_ascii_lowercase(), outgoing))
    }
}

// LINE image message limits: original up to 10 MB, preview up to 1 MB, JPEG or PNG.
pub const ORIGINAL_MAX_BYTES: usize = 10 * 1024 * 1024;
pub const PREVIEW_MAX_BYTES: usize = 1024 * 1024;

pub fn image_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else {
        None
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UploadInput {
    pub original: String,
    pub preview: String,
}

pub struct UploadedImage {
    pub bytes: Vec<u8>,
    pub mime: &'static str,
}

impl UploadInput {
    pub fn validate(self) -> Result<(UploadedImage, UploadedImage), AppError> {
        let decode = |data: &str, max: usize| -> Result<UploadedImage, AppError> {
            // Base64 is 4/3 of the payload; reject oversized input before decoding.
            if data.len() > max / 3 * 4 + 8 {
                return Err(AppError::Validation("Image is too large for LINE"));
            }
            let bytes = STANDARD
                .decode(data)
                .map_err(|_| AppError::Validation("Invalid image data"))?;
            let mime = image_type(&bytes).ok_or(AppError::Validation("Only JPEG or PNG images"))?;
            if bytes.len() > max {
                return Err(AppError::Validation("Image is too large for LINE"));
            }
            Ok(UploadedImage { bytes, mime })
        };
        Ok((
            decode(&self.original, ORIGINAL_MAX_BYTES)?,
            decode(&self.preview, PREVIEW_MAX_BYTES)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(text: Option<&str>, sticker: Option<(&str, &str)>, image: Option<&str>) -> SendInput {
        SendInput {
            request_id: "3F2504E0-4F89-41D3-9A0C-0305E82C3301".into(),
            text: text.map(String::from),
            sticker: sticker.map(|(p, s)| StickerRef {
                package_id: p.into(),
                sticker_id: s.into(),
            }),
            image_id: image.map(String::from),
        }
    }

    #[test]
    fn reply_input_requires_uuid_and_exactly_one_valid_kind() {
        let (id, text) = input(Some("  สวัสดีค่ะ \n"), None, None).validate().unwrap();
        assert_eq!(id, "3f2504e0-4f89-41d3-9a0c-0305e82c3301");
        assert_eq!(text, Outgoing::Text("สวัสดีค่ะ".into()));
        let mut bad_id = input(Some("hi"), None, None);
        bad_id.request_id = "not-a-uuid".into();
        assert!(bad_id.validate().is_err());
        for text in ["   ".to_string(), "ก".repeat(5001), "a\u{7}b".to_string()] {
            assert!(input(Some(&text), None, None).validate().is_err());
        }
        // Sticker IDs must be in LINE's sendable list.
        for (p, s) in [
            ("446", "1988"),
            ("446", "2027"),
            ("11539", "52114149"),
            ("8525", "16581313"),
        ] {
            assert!(
                input(None, Some((p, s)), None).validate().is_ok(),
                "{p}/{s}"
            );
        }
        for (p, s) in [
            ("446", "2028"),
            ("446", "10855"),
            ("999", "1"),
            ("446", "x"),
        ] {
            assert!(
                input(None, Some((p, s)), None).validate().is_err(),
                "{p}/{s}"
            );
        }
        assert!(matches!(
            input(None, None, Some("64b7f0c2a1b2c3d4e5f60718"))
                .validate()
                .unwrap()
                .1,
            Outgoing::Image(_)
        ));
        assert!(input(None, None, None).validate().is_err());
        assert!(
            input(Some("hi"), Some(("446", "1988")), None)
                .validate()
                .is_err()
        );
    }

    #[test]
    fn uploads_accept_only_small_enough_jpeg_or_png() {
        let jpeg = STANDARD.encode([0xFF, 0xD8, 0xFF, 0xE0, 1, 2, 3]);
        let png = STANDARD.encode([0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0]);
        let (original, preview) = UploadInput {
            original: jpeg.clone(),
            preview: png,
        }
        .validate()
        .unwrap();
        assert_eq!((original.mime, preview.mime), ("image/jpeg", "image/png"));
        let gif = STANDARD.encode(b"GIF89a....");
        assert!(
            UploadInput {
                original: gif,
                preview: jpeg.clone()
            }
            .validate()
            .is_err()
        );
        let big_preview =
            STANDARD.encode([&[0xFF, 0xD8, 0xFF][..], &vec![0; PREVIEW_MAX_BYTES]].concat());
        assert!(
            UploadInput {
                original: jpeg,
                preview: big_preview
            }
            .validate()
            .is_err()
        );
    }
}
