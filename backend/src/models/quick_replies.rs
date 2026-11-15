use crate::{
    error::AppError,
    models::outbound::{UploadInput, UploadedImage, valid_request_id},
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateInput {
    pub request_id: String,
    pub title: String,
    pub kind: String,
    pub text: Option<String>,
    pub image: Option<UploadInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditInput {
    pub expected_version: i64,
    pub title: String,
    pub kind: String,
    pub text: Option<String>,
    pub image: Option<UploadInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionInput {
    pub expected_version: i64,
}
impl VersionInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.expected_version < 1 {
            return Err(AppError::BadRequest);
        }
        Ok(())
    }
}

pub struct Content {
    pub title: String,
    pub kind: String,
    pub text: String,
    pub image: Option<(UploadedImage, UploadedImage)>,
}

fn content(
    title: String,
    kind: String,
    text: Option<String>,
    image: Option<UploadInput>,
    creating: bool,
) -> Result<Content, AppError> {
    let title = title.trim().to_string();
    let text = text.unwrap_or_default().trim().to_string();
    if title.is_empty()
        || title.chars().count() > 80
        || title.starts_with('/')
        || title.chars().any(char::is_control)
        || text.chars().count() > 5000
        || text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
    {
        return Err(AppError::Validation("Invalid quick reply title or text"));
    }
    match kind.as_str() {
        "text" if !text.is_empty() && image.is_none() => (),
        "image" if text.is_empty() && (!creating || image.is_some()) => (),
        _ => {
            return Err(AppError::Validation(
                "A quick reply needs text or one image",
            ));
        }
    }
    Ok(Content {
        title,
        kind,
        text,
        image: image.map(UploadInput::validate).transpose()?,
    })
}
impl CreateInput {
    pub fn validate(self) -> Result<(String, Content), AppError> {
        if !valid_request_id(&self.request_id) {
            return Err(AppError::BadRequest);
        }
        Ok((
            self.request_id.to_ascii_lowercase(),
            content(self.title, self.kind, self.text, self.image, true)?,
        ))
    }
}
impl EditInput {
    pub fn validate(self) -> Result<(i64, Content), AppError> {
        if self.expected_version < 1 {
            return Err(AppError::BadRequest);
        }
        Ok((
            self.expected_version,
            content(self.title, self.kind, self.text, self.image, false)?,
        ))
    }
}
