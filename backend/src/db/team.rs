use crate::{
    alex,
    error::AppError,
    models::{
        customer::Actor,
        team::{self, Access, EditInput, InviteInput, Role},
    },
    state::AppState,
};
use futures_util::TryStreamExt;
use mongodb::{
    bson::{Bson, DateTime, Document, doc, oid::ObjectId},
    options::ReturnDocument,
};
use std::sync::atomic::Ordering;

pub fn enabled(state: &AppState) -> bool {
    state.team_enabled.load(Ordering::Acquire)
}
fn keyed(state: &AppState, id: ObjectId) -> Document {
    let mut filter = state.scope();
    filter.insert("_id", id);
    filter
}
fn history(action: &str, actor: &Actor, version: i64, changes: Document) -> Document {
    doc! {"action":action,"actor_id":&actor.id,"actor_name":&actor.name,"at":DateTime::now(),"version":version,"changes":changes}
}

// Capture the pre-existing allowlist once. Later env changes never resurrect a revoked member.
pub async fn bootstrap(state: &AppState) -> Result<(), AppError> {
    let settings = state
        .collection("team_settings")
        .find_one(state.scope())
        .await?;
    let settings = match settings {
        Some(value) => value,
        None if state.config.admin_line_user_ids.is_empty()
            || !state.config.line_login_enabled() =>
        {
            return Ok(());
        }
        None => {
            let mut initial = state.scope();
            initial.extend(doc! {"_id":ObjectId::new(),"bootstrap_ids":&state.config.admin_line_user_ids,"enabled":false,"created_at":DateTime::now(),"updated_at":DateTime::now()});
            match state
                .collection("team_settings")
                .find_one_and_update(state.scope(), doc! {"$setOnInsert":initial})
                .upsert(true)
                .return_document(ReturnDocument::After)
                .await
            {
                Ok(Some(value)) => value,
                Err(error) if super::events::duplicate(&error) => state
                    .collection("team_settings")
                    .find_one(state.scope())
                    .await?
                    .ok_or(AppError::Data)?,
                Ok(None) => return Err(AppError::Data),
                Err(error) => return Err(error.into()),
            }
        }
    };
    if !settings.get_bool("enabled").unwrap_or(false) {
        let ids = settings
            .get_array("bootstrap_ids")
            .map_err(|_| AppError::Data)?;
        if ids.is_empty() {
            return Err(AppError::Data);
        }
        for value in ids {
            let user = value
                .as_str()
                .filter(|id| crate::models::customer::valid_user_id(id))
                .ok_or(AppError::Data)?;
            let account = account(state, user, None, None, false).await?;
            let mut filter = state.scope();
            filter.insert("bootstrap_user_id", user);
            let mut initial = filter.clone();
            let actor = Actor {
                id: "system:team-migration".into(),
                name: "ระบบย้ายสิทธิ์เดิม".into(),
            };
            initial.extend(doc! {"_id":ObjectId::new(),"account_id":account.get_object_id("_id").map_err(|_|AppError::Data)?,"line_user_id":user,
                "name":"","role":"OWNER","branch_ids":[],"branches":[],"status":"ACTIVE","live_member":true,"version":1_i64,"note":"",
                "created_at":DateTime::now(),"updated_at":DateTime::now(),"approved_at":DateTime::now(),"history":[history("MIGRATED_OWNER",&actor,1,doc! {})]});
            match state
                .collection("team_memberships")
                .update_one(filter, doc! {"$setOnInsert":initial})
                .upsert(true)
                .await
            {
                Ok(_) => {}
                Err(error) if super::events::duplicate(&error) => {}
                Err(error) => return Err(error.into()),
            }
        }
        state
            .collection("team_settings")
            .update_one(
                state.scope(),
                doc! {"$set":{"enabled":true,"updated_at":DateTime::now()}},
            )
            .await?;
    }
    state.team_enabled.store(true, Ordering::Release);
    Ok(())
}

pub async fn account(
    state: &AppState,
    user: &str,
    name: Option<&str>,
    picture: Option<&str>,
    login: bool,
) -> Result<Document, AppError> {
    let channel = state
        .config
        .line_login_channel_id
        .as_deref()
        .ok_or(AppError::AdminUnavailable)?;
    let filter = doc! {"login_channel_id":channel,"line_user_id":user};
    let mut set = doc! {"updated_at":DateTime::now()};
    if let Some(name) = name {
        set.insert("line_name", name.chars().take(100).collect::<String>());
    }
    if let Some(picture) = picture.filter(|url| url.len() <= 2000 && url.starts_with("https://")) {
        set.insert("picture_url", picture);
    }
    if login {
        set.insert("last_login_at", DateTime::now());
    }
    let result = state
        .collection("admin_accounts")
        .find_one_and_update(
            filter.clone(),
            doc! {"$set":set,"$setOnInsert":{"_id":ObjectId::new(),"created_at":DateTime::now()}},
        )
        .upsert(true)
        .return_document(ReturnDocument::After)
        .await;
    match result {
        Ok(Some(record)) => Ok(record),
        Err(error) if super::events::duplicate(&error) => state
            .collection("admin_accounts")
            .find_one(filter)
            .await?
            .ok_or(AppError::Data),
        Ok(None) => Err(AppError::Data),
        Err(error) => Err(error.into()),
    }
}
pub async fn membership(state: &AppState, user: &str) -> Result<Option<Document>, AppError> {
    let mut filter = state.scope();
    filter.extend(doc! {"line_user_id":user,"status":{"$in":["ACTIVE","PENDING","SUSPENDED"]}});
    if let Some(record) = state
        .collection("team_memberships")
        .find_one(filter.clone())
        .await?
    {
        return Ok(Some(record));
    }
    filter.insert("status", "REJECTED");
    Ok(state
        .collection("team_memberships")
        .find_one(filter)
        .sort(doc! {"updated_at":-1,"_id":-1})
        .await?)
}
pub async fn access(state: &AppState, user: &str, fallback: &str) -> Result<Access, AppError> {
    let record = membership(state, user)
        .await?
        .filter(|value| value.get_str("status").ok() == Some("ACTIVE"))
        .ok_or(AppError::Forbidden)?;
    Access::from_document(&record, fallback)
}
pub fn require_owner(state: &AppState) -> Result<&Access, AppError> {
    state
        .access
        .as_ref()
        .filter(|access| enabled(state) && access.role == Role::Owner)
        .ok_or(AppError::Forbidden)
}
pub fn actor(access: &Access) -> Actor {
    Actor {
        id: format!("line:{}", access.line_user_id),
        name: access.name.clone(),
    }
}
pub fn check_branch(state: &AppState, id: &str) -> Result<(), AppError> {
    if state
        .access
        .as_ref()
        .is_none_or(|access| access.allows_branch(id))
    {
        Ok(())
    } else {
        Err(AppError::AlexScope)
    }
}
pub async fn revalidate(state: &AppState) -> Result<(), AppError> {
    if let Some(access) = &state.access {
        let record = by_id(state, access.member_id).await?;
        if record.get_str("status").ok() != Some("ACTIVE")
            || record.get_i64("version").ok() != Some(access.version)
        {
            return Err(AppError::Forbidden);
        }
    }
    Ok(())
}
pub async fn job_state(state: &AppState, job: &Document) -> Result<AppState, AppError> {
    let mut scoped = state.clone();
    if let Ok(id) = job.get_object_id("requested_by_member") {
        let record = by_id(state, id).await?;
        if record.get_str("status").ok() != Some("ACTIVE")
            || record.get_i64("version").ok() != job.get_i64("requested_by_member_version").ok()
        {
            return Err(AppError::Forbidden);
        }
        scoped.access = Some(Access::from_document(&record, "พนักงาน")?);
    }
    Ok(scoped)
}

pub async fn branches(state: &AppState, ids: &[String]) -> Result<Vec<Document>, AppError> {
    if state.config.alex.is_none() {
        return if ids.is_empty() {
            Ok(vec![])
        } else {
            Err(AppError::AlexScope)
        };
    }
    if ids.is_empty() {
        return Err(AppError::Validation("Select at least one branch"));
    }
    let session = alex::login(state).await?;
    ids.iter()
        .map(|id| {
            session
                .branches
                .iter()
                .find(|branch| branch.id == *id)
                .map(|branch| doc! {"id":&branch.id,"name":&branch.name})
                .ok_or(AppError::AlexScope)
        })
        .collect()
}
pub async fn create_invitation(state: &AppState, input: InviteInput) -> Result<Document, AppError> {
    let access = require_owner(state)?;
    input.validate()?;
    let actor = actor(access);
    let mut filter = state.scope();
    filter.extend(doc! {"request_id":&input.request_id,"created_by_id":&actor.id});
    if let Some(existing) = state
        .collection("team_memberships")
        .find_one(filter.clone())
        .await?
    {
        if existing.get_str("name").ok() != Some(input.name.trim())
            || existing.get_str("role").ok() != Some(input.role.as_str())
            || existing.get_array("branch_ids").ok()
                != Some(
                    &input
                        .branch_ids
                        .iter()
                        .map(|id| Bson::String(id.clone()))
                        .collect::<Vec<_>>(),
                )
            || existing.get_str("note").ok() != Some(input.note.trim())
        {
            return Err(AppError::RecordConflict);
        }
        return Ok(existing);
    }
    let branch_names = branches(state, &input.branch_ids).await?;
    let id = ObjectId::new();
    let token = team::token(
        &state.config.admin_token,
        &state.config.shop_id,
        &state.config.channel_id,
        id,
    )?;
    let now = DateTime::now();
    let mut record = filter;
    let initial_changes = doc! {"name":input.name.trim(),"role":input.role.as_str(),"branch_ids":&input.branch_ids,"branches":&branch_names,"note":input.note.trim(),"status":"INVITED"};
    record.extend(doc! {"_id":id,"name":input.name.trim(),"role":input.role.as_str(),"branch_ids":input.branch_ids,"branches":branch_names,"note":input.note.trim(),
        "status":"INVITED","version":1_i64,"created_by":&actor.name,"expires_at":DateTime::from_millis(now.timestamp_millis()+86_400_000),
        "token_hash":team::token_hash(&token)?,"created_at":now,"updated_at":now,"history":[history("INVITED",&actor,1,initial_changes)]});
    match state
        .collection("team_memberships")
        .insert_one(record.clone())
        .await
    {
        Ok(_) => Ok(record),
        Err(error) if super::events::duplicate(&error) => Err(AppError::RecordConflict),
        Err(error) => Err(error.into()),
    }
}
pub async fn by_id(state: &AppState, id: ObjectId) -> Result<Document, AppError> {
    state
        .collection("team_memberships")
        .find_one(keyed(state, id))
        .await?
        .ok_or(AppError::NotFound)
}
pub async fn invitation(state: &AppState, token: &str) -> Result<Document, AppError> {
    if !enabled(state) {
        return Err(AppError::Team("TEAM_NOT_READY"));
    }
    let mut filter = state.scope();
    filter.insert("token_hash", team::token_hash(token)?);
    state
        .collection("team_memberships")
        .find_one(filter)
        .await?
        .ok_or(AppError::Team("INVITE_INVALID"))
}
pub fn invitation_live(record: &Document) -> Result<(), AppError> {
    if record.get_str("status").ok() != Some("INVITED") {
        return Err(AppError::Team("INVITE_USED"));
    }
    if record
        .get_datetime("expires_at")
        .ok()
        .is_none_or(|expiry| *expiry <= DateTime::now())
    {
        return Err(AppError::Team("INVITE_EXPIRED"));
    }
    Ok(())
}
pub fn invitation_token(state: &AppState, record: &Document) -> Result<String, AppError> {
    invitation_live(record)?;
    let token = team::token(
        &state.config.admin_token,
        &state.config.shop_id,
        &state.config.channel_id,
        record.get_object_id("_id").map_err(|_| AppError::Data)?,
    )?;
    if record.get_str("token_hash").ok() != Some(team::token_hash(&token)?.as_str()) {
        return Err(AppError::Team("INVITE_INVALID"));
    }
    Ok(token)
}

// An invite and its pending membership are one aggregate: claiming and consuming the link is one CAS.
pub async fn accept(
    state: &AppState,
    token: &str,
    user: &str,
    line_name: &str,
) -> Result<Document, AppError> {
    let record = invitation(state, token).await?;
    if record.get_str("line_user_id").ok() == Some(user)
        && ["PENDING", "ACTIVE", "SUSPENDED"].contains(&record.get_str("status").unwrap_or(""))
    {
        return Ok(record);
    }
    invitation_live(&record)?;
    if membership(state, user)
        .await?
        .is_some_and(|member| member.get_str("status").ok() != Some("REJECTED"))
    {
        return Err(AppError::Team("TEAM_MEMBER_EXISTS"));
    }
    let account = account(state, user, Some(line_name), None, false).await?;
    let id = record.get_object_id("_id").map_err(|_| AppError::Data)?;
    let actor = Actor {
        id: format!("line:{user}"),
        name: line_name.into(),
    };
    let mut filter = keyed(state, id);
    filter.extend(doc! {"status":"INVITED","token_hash":team::token_hash(token)?,"expires_at":{"$gt":DateTime::now()}});
    let result = state.collection("team_memberships").find_one_and_update(filter,doc! {"$set":{"status":"PENDING","live_member":true,"line_user_id":user,"account_id":account.get_object_id("_id").map_err(|_|AppError::Data)?,"accepted_at":DateTime::now(),"updated_at":DateTime::now()},"$inc":{"version":1_i64},"$push":{"history":history("ACCEPTED",&actor,2,doc! {})}}).return_document(ReturnDocument::After).await;
    match result {
        Ok(Some(member)) => Ok(member),
        Ok(None) => {
            let latest = by_id(state, id).await?;
            if latest.get_str("line_user_id").ok() == Some(user)
                && ["PENDING", "ACTIVE"].contains(&latest.get_str("status").unwrap_or(""))
            {
                Ok(latest)
            } else {
                Err(AppError::Team("INVITE_USED"))
            }
        }
        Err(error) if super::events::duplicate(&error) => Err(AppError::Team("TEAM_MEMBER_EXISTS")),
        Err(error) => Err(error.into()),
    }
}

pub async fn transition(
    state: &AppState,
    id: ObjectId,
    version: i64,
    action: &str,
) -> Result<Document, AppError> {
    let access = require_owner(state)?;
    if version < 1 {
        return Err(AppError::BadRequest);
    }
    let (from, to) = match action {
        "APPROVED" => ("PENDING", "ACTIVE"),
        "REJECTED" => ("PENDING", "REJECTED"),
        "CANCELLED" => ("INVITED", "CANCELLED"),
        _ => return Err(AppError::BadRequest),
    };
    let record = by_id(state, id).await?;
    if record.get_str("role").ok() == Some("OWNER") {
        return Err(AppError::Forbidden);
    }
    if action == "APPROVED" {
        let ids = record
            .get_array("branch_ids")
            .map_err(|_| AppError::Data)?
            .iter()
            .map(|id| id.as_str().map(String::from).ok_or(AppError::Data))
            .collect::<Result<Vec<_>, _>>()?;
        branches(state, &ids).await?;
    }
    let actor = actor(access);
    let mut filter = keyed(state, id);
    filter.extend(doc! {"version":version,"status":from,"role":{"$ne":"OWNER"}});
    let mut set = doc! {"status":to,"updated_at":DateTime::now(),"updated_by":&actor.name,"updated_by_id":&actor.id};
    if action == "REJECTED" {
        set.insert("live_member", false);
    }
    if action == "APPROVED" {
        set.insert("approved_at", DateTime::now());
        set.insert("approved_by_id", &actor.id);
    }
    state.collection("team_memberships").find_one_and_update(filter,doc! {"$set":set,"$inc":{"version":1_i64},"$push":{"history":history(action,&actor,version+1,doc! {"status":to})}}).return_document(ReturnDocument::After).await?.ok_or(AppError::RecordConflict)
}
pub async fn edit(state: &AppState, id: ObjectId, input: EditInput) -> Result<Document, AppError> {
    let access = require_owner(state)?;
    input.validate()?;
    let record = by_id(state, id).await?;
    if record.get_str("role").ok() == Some("OWNER") {
        return Err(AppError::Forbidden);
    }
    let previous = record.get_str("status").map_err(|_| AppError::Data)?;
    if !["PENDING", "ACTIVE", "SUSPENDED"].contains(&previous)
        || (previous == "PENDING") != (input.status == "PENDING")
    {
        return Err(AppError::BadRequest);
    }
    let names = branches(state, &input.branch_ids).await?;
    let changes = doc! {"name":input.name.trim(),"role":input.role.as_str(),"branch_ids":input.branch_ids,"branches":names,"status":&input.status,"note":input.note.trim()};
    let mut set = changes.clone();
    let actor = actor(access);
    set.extend(
        doc! {"updated_at":DateTime::now(),"updated_by":&actor.name,"updated_by_id":&actor.id},
    );
    let mut filter = keyed(state, id);
    filter.extend(doc! {"version":input.expected_version,"status":previous,"role":{"$ne":"OWNER"}});
    state.collection("team_memberships").find_one_and_update(filter,doc! {"$set":set,"$inc":{"version":1_i64},"$push":{"history":history("UPDATED",&actor,input.expected_version+1,changes)}}).return_document(ReturnDocument::After).await?.ok_or(AppError::RecordConflict)
}

pub async fn list(state: &AppState, before: Option<ObjectId>) -> Result<Vec<Document>, AppError> {
    require_owner(state)?;
    let mut filter = state.scope();
    if let Some(id) = before {
        filter.insert("_id", doc! {"$lt":id});
    }
    let records: Vec<Document> = state
        .collection("team_memberships")
        .find(filter)
        .sort(doc! {"_id":-1})
        .limit(100)
        .projection(doc! {"token_hash":0,"history":0})
        .await?
        .try_collect()
        .await?;
    Ok(records)
}
pub async fn profile(state: &AppState, record: &Document) -> Result<Option<Document>, AppError> {
    let Ok(id) = record.get_object_id("account_id") else {
        return Ok(None);
    };
    let channel = state
        .config
        .line_login_channel_id
        .as_deref()
        .ok_or(AppError::AdminUnavailable)?;
    Ok(state
        .collection("admin_accounts")
        .find_one(doc! {"_id":id,"login_channel_id":channel})
        .projection(doc! {"line_name":1,"picture_url":1,"last_login_at":1})
        .await?)
}
pub async fn identity_profile(state: &AppState, user: &str) -> Result<Option<Document>, AppError> {
    let channel = state
        .config
        .line_login_channel_id
        .as_deref()
        .ok_or(AppError::AdminUnavailable)?;
    Ok(state
        .collection("admin_accounts")
        .find_one(doc! {"login_channel_id":channel,"line_user_id":user})
        .projection(doc! {"line_name":1,"picture_url":1})
        .await?)
}
