use crate::{
    error::AppError,
    models::outbound::{image_type, valid_request_id},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const IMAGE_MAX: usize = 2 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AliasInput {
    pub expected_version: i64,
    pub custom_name: Option<String>,
}

impl AliasInput {
    pub fn validate(self) -> Result<Self, AppError> {
        let custom_name = self
            .custom_name
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty());
        if self.expected_version < 0
            || custom_name.as_ref().is_some_and(|name| {
                name.chars().count() > 200 || name.chars().any(char::is_control)
            })
        {
            return Err(AppError::Validation("Invalid customer alias"));
        }
        Ok(Self {
            expected_version: self.expected_version,
            custom_name,
        })
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImageInput {
    pub id: Option<String>,
    pub name: Option<String>,
    pub image_base64: Option<String>,
}

pub enum NoteImage {
    Existing(ObjectId),
    New {
        name: String,
        bytes: Vec<u8>,
        mime: &'static str,
    },
}

pub struct NoteContent {
    pub text: String,
    pub images: Vec<NoteImage>,
}

fn content(text: &str, images: &[ImageInput]) -> Result<NoteContent, AppError> {
    let text = text.trim().to_string();
    if text.chars().count() > 5000
        || text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        || images.len() > 5
        || (text.is_empty() && images.is_empty())
    {
        return Err(AppError::Validation(
            "A note needs text or up to five images",
        ));
    }
    let mut ids = HashSet::new();
    let images = images
        .iter()
        .map(
            |image| match (&image.id, &image.name, &image.image_base64) {
                (Some(id), None, None) => {
                    let id = ObjectId::parse_str(id).map_err(|_| AppError::BadRequest)?;
                    if !ids.insert(id) {
                        return Err(AppError::Validation("Duplicate note image"));
                    }
                    Ok(NoteImage::Existing(id))
                }
                (None, Some(name), Some(data))
                    if !name.trim().is_empty()
                        && name.chars().count() <= 200
                        && !name.chars().any(char::is_control)
                        && data.len() <= IMAGE_MAX / 3 * 4 + 8 =>
                {
                    let bytes = STANDARD
                        .decode(data)
                        .map_err(|_| AppError::Validation("Invalid note image"))?;
                    let mime = image_type(&bytes)
                        .ok_or(AppError::Validation("Note images must be JPEG or PNG"))?;
                    if bytes.len() > IMAGE_MAX {
                        return Err(AppError::Validation("Note image exceeds 2 MB"));
                    }
                    Ok(NoteImage::New {
                        name: name.trim().to_string(),
                        bytes,
                        mime,
                    })
                }
                _ => Err(AppError::Validation("Invalid note image")),
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NoteContent { text, images })
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreateInput {
    pub request_id: String,
    pub text: String,
    #[serde(default)]
    pub images: Vec<ImageInput>,
}

impl CreateInput {
    pub fn validate(&self) -> Result<NoteContent, AppError> {
        if !valid_request_id(&self.request_id) {
            return Err(AppError::BadRequest);
        }
        let value = content(&self.text, &self.images)?;
        if value
            .images
            .iter()
            .any(|image| matches!(image, NoteImage::Existing(_)))
        {
            return Err(AppError::Validation(
                "A new note must upload its own images",
            ));
        }
        Ok(value)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditInput {
    pub expected_version: i64,
    pub text: String,
    #[serde(default)]
    pub images: Vec<ImageInput>,
}

impl EditInput {
    pub fn validate(&self) -> Result<NoteContent, AppError> {
        if self.expected_version < 1 {
            return Err(AppError::BadRequest);
        }
        content(&self.text, &self.images)
    }
}
