use crate::{error::AppError, models::outbound::valid_request_id};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use mongodb::bson::{Document, oid::ObjectId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    Owner,
    Manager,
    Staff,
}
impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "OWNER",
            Self::Manager => "MANAGER",
            Self::Staff => "STAFF",
        }
    }
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "OWNER" => Ok(Self::Owner),
            "MANAGER" => Ok(Self::Manager),
            "STAFF" => Ok(Self::Staff),
            _ => Err(AppError::Data),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Access {
    pub member_id: ObjectId,
    pub line_user_id: String,
    pub name: String,
    pub role: Role,
    pub branch_ids: Vec<String>,
    pub version: i64,
}
impl Access {
    pub fn from_document(record: &Document, fallback_name: &str) -> Result<Self, AppError> {
        let name = record
            .get_str("name")
            .ok()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(fallback_name)
            .to_string();
        Ok(Self {
            member_id: record.get_object_id("_id").map_err(|_| AppError::Data)?,
            line_user_id: record
                .get_str("line_user_id")
                .map_err(|_| AppError::Data)?
                .into(),
            name,
            role: Role::parse(record.get_str("role").map_err(|_| AppError::Data)?)?,
            branch_ids: record
                .get_array("branch_ids")
                .map_err(|_| AppError::Data)?
                .iter()
                .map(|value| value.as_str().map(String::from).ok_or(AppError::Data))
                .collect::<Result<_, _>>()?,
            version: record.get_i64("version").map_err(|_| AppError::Data)?,
        })
    }
    pub fn can_submit(&self) -> bool {
        self.role != Role::Staff
    }
    pub fn allows_branch(&self, id: &str) -> bool {
        self.role == Role::Owner || self.branch_ids.iter().any(|branch| branch == id)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InviteInput {
    pub request_id: String,
    pub name: String,
    pub role: Role,
    pub branch_ids: Vec<String>,
    #[serde(default)]
    pub note: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionInput {
    pub expected_version: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditInput {
    pub expected_version: i64,
    pub name: String,
    pub role: Role,
    pub branch_ids: Vec<String>,
    pub status: String,
    #[serde(default)]
    pub note: String,
}

pub fn validate_fields(
    name: &str,
    role: Role,
    branches: &[String],
    note: &str,
) -> Result<(), AppError> {
    if name.trim().is_empty()
        || name.chars().count() > 100
        || name.chars().any(char::is_control)
        || role == Role::Owner
        || branches.len() > 30
        || branches
            .iter()
            .any(|id| !crate::models::product::valid_id(id))
        || branches
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            != branches.len()
        || note.chars().count() > 1000
        || note
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
    {
        return Err(AppError::Validation("Invalid team member fields"));
    }
    Ok(())
}
impl InviteInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if !valid_request_id(&self.request_id) {
            return Err(AppError::BadRequest);
        }
        validate_fields(&self.name, self.role, &self.branch_ids, &self.note)
    }
}
impl EditInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.expected_version < 1
            || !["ACTIVE", "SUSPENDED", "PENDING"].contains(&self.status.as_str())
        {
            return Err(AppError::BadRequest);
        }
        validate_fields(&self.name, self.role, &self.branch_ids, &self.note)
    }
}
pub fn token(secret: &str, shop: &str, channel: &str, id: ObjectId) -> Result<String, AppError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| AppError::Data)?;
    mac.update(b"alex-team-invitation-v1:");
    mac.update(&serde_json::to_vec(&(shop, channel, id.to_hex())).map_err(|_| AppError::Data)?);
    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}
pub fn token_hash(value: &str) -> Result<String, AppError> {
    if value.len() != 43
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
    {
        return Err(AppError::Team("INVITE_INVALID"));
    }
    Ok(format!("{:x}", Sha256::digest(value.as_bytes())))
}
