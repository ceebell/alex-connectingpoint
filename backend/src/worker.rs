use crate::{
    db::{events, inquiries, media, users},
    error::AppError,
    state::AppState,
};
use mongodb::bson::{DateTime, Document, doc};
use tokio::sync::watch;

pub async fn process(state: &AppState, record: &Document) -> Result<(), AppError> {
    if record.get_bool("withdrawn").unwrap_or(false) {
        return events::finish(state, record, "DONE").await;
    }
    let event = media::payload(record)?;
    if event.kind == "unsend" {
        let message = event
            .unsend
            .as_ref()
            .and_then(|u| u["messageId"].as_str())
            .ok_or(AppError::Data)?;
        media::tombstone(state, message).await?;
        media::purge_message(state, message).await?;
        return events::finish(state, record, "DONE").await;
    }
    if !matches!(event.kind.as_str(), "message" | "postback") {
        return events::finish(state, record, "IGNORED").await;
    }
    if let Some(id) = event.message.as_ref().and_then(|m| m["id"].as_str())
        && media::withdrawn(state, id).await?
    {
        media::purge_message(state, id).await?;
        return events::finish(state, record, "DONE").await;
    }
    let linked = inquiries::link(state, &event).await?;
    if let Some(user) = event.source.as_ref().and_then(|s| s.user_id.as_deref())
        && users::refresh_profile(state, user).await.is_err()
    {
        tracing::warn!("LINE profile refresh failed; will retry on a later message");
    }
    // A retry keeps its original inquiry even if another inquiry has since opened.
    let inquiry = record
        .get_object_id("inquiry_id")
        .ok()
        .or(linked.and_then(|(_, selected)| selected));
    let mut fields = doc! {"updated_at":DateTime::now()};
    if let Some((conversation, _)) = linked {
        fields.insert("conversation_id", conversation);
    }
    if let Some(id) = inquiry {
        fields.insert("inquiry_id", id);
    }
    state
        .collection("line_events")
        .update_one(events::lease_filter(state, record)?, doc! {"$set":fields})
        .await?;
    if let Some(message) = event.message.as_ref().filter(|m| m["type"] == "image") {
        let id = message["id"].as_str().ok_or(AppError::Data)?;
        if message["contentProvider"]["type"]
            .as_str()
            .is_some_and(|p| p != "line")
        {
            state
                .collection("line_events")
                .update_one(
                    events::lease_filter(state, record)?,
                    doc! {"$set":{"media_status":"NEEDS_ADMIN", "updated_at":DateTime::now()}},
                )
                .await?;
        } else {
            let media_id = media::download(state, record, id).await?;
            state.collection("line_events").update_one(events::lease_filter(state, record)?,
                doc! {"$set":{"media_id":media_id, "media_status":"STORED", "updated_at":DateTime::now()}}).await?;
        }
        if media::withdrawn(state, id).await? {
            media::purge_message(state, id).await?;
        }
    }
    let Some(inquiry) = inquiry else {
        return events::finish(state, record, "NEEDS_ADMIN").await;
    };
    inquiries::schedule(state, record, inquiry).await?;
    events::finish(state, record, "DONE").await
}

pub async fn run(state: AppState, mut stop: watch::Receiver<bool>) {
    let mut cleanup_at = tokio::time::Instant::now();
    loop {
        if *stop.borrow() {
            break;
        }
        if tokio::time::Instant::now() >= cleanup_at {
            if media::cleanup(&state).await.is_err() {
                tracing::error!("media cleanup failed");
            }
            cleanup_at = tokio::time::Instant::now() + std::time::Duration::from_secs(60);
        }
        match events::claim(&state).await {
            Ok(Some(event)) => {
                if let Err(error) = process_with_heartbeat(&state, &event).await
                    && events::retry(&state, &event, error.code()).await.is_err()
                {
                    tracing::error!("could not release event; lease will expire");
                }
                continue;
            }
            Ok(None) => (),
            Err(_) => tracing::error!("inquiry queue unavailable"),
        }
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_millis(250)) => (),
            result = stop.changed() => if result.is_err() { break; },
        }
    }
}

async fn process_with_heartbeat(state: &AppState, event: &Document) -> Result<(), AppError> {
    let (finished, mut finished_rx) = watch::channel(false);
    let heartbeat_state = state.clone();
    let heartbeat_event = event.clone();
    // Keep renewing during long downloads/storage operations without cancelling Mongo futures.
    let heartbeat = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => (),
                _ = finished_rx.changed() => break,
            }
            match events::renew(&heartbeat_state, &heartbeat_event).await {
                Ok(true) => (),
                Ok(false) => break,
                Err(_) => tracing::warn!("event lease renewal failed"),
            }
        }
    });
    let result = process(state, event).await;
    let _ = finished.send(true);
    let _ = heartbeat.await;
    result
}
