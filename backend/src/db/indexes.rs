use crate::{error::AppError, state::AppState};
use mongodb::{IndexModel, bson::doc, options::IndexOptions};

pub async fn ensure(state: &AppState) -> Result<(), AppError> {
    for (collection, keys, unique) in [
        (
            "oa_messages",
            doc! {"shop_id":1,"channel_id":1,"bot_id":1,"external_message_id":1},
            true,
        ),
        (
            "oa_messages",
            doc! {"shop_id":1,"channel_id":1,"inquiry_id":1,"event_at":1,"event_id":1},
            false,
        ),
        (
            "connector_statuses",
            doc! {"shop_id":1,"channel_id":1,"name":1},
            true,
        ),
        (
            "draft_requests",
            doc! {"shop_id":1,"channel_id":1,"inquiry_id":1,"request_id":1},
            true,
        ),
        (
            "draft_requests",
            doc! {"shop_id":1,"channel_id":1,"status":1,"available_at":1},
            false,
        ),
        (
            "order_drafts",
            doc! {"shop_id":1,"channel_id":1,"inquiry_id":1,"draft_version":-1},
            false,
        ),
        (
            "admin_summaries",
            doc! {"shop_id":1,"channel_id":1,"inquiry_id":1,"created_at":-1},
            false,
        ),
        (
            "line_users",
            doc! {"shop_id":1, "channel_id":1, "line_user_id":1},
            true,
        ),
        (
            "line_users",
            doc! {"shop_id":1, "channel_id":1, "_id":-1},
            false,
        ),
        (
            "line_events",
            doc! {"shop_id":1, "channel_id":1, "payload.source.userId":1, "event_at":1, "event_id":1},
            false,
        ),
        (
            "line_events",
            doc! {"shop_id":1, "channel_id":1, "event_id":1},
            true,
        ),
        (
            "line_events",
            doc! {"shop_id":1, "channel_id":1, "status":1, "available_at":1},
            false,
        ),
        (
            "line_events",
            doc! {"shop_id":1, "channel_id":1, "inquiry_id":1, "event_at":1, "event_id":1},
            false,
        ),
        (
            "line_events",
            doc! {"shop_id":1, "channel_id":1, "message_id":1},
            false,
        ),
        (
            "conversations",
            doc! {"shop_id":1, "channel_id":1, "conversation_key":1},
            true,
        ),
        (
            "inquiries",
            doc! {"shop_id":1, "channel_id":1, "conversation_id":1, "status":1},
            false,
        ),
        (
            "analysis_jobs",
            doc! {"shop_id":1, "channel_id":1, "inquiry_id":1},
            true,
        ),
        (
            "withdrawn_messages",
            doc! {"shop_id":1, "channel_id":1, "message_id":1},
            true,
        ),
    ] {
        state
            .collection(collection)
            .create_index(
                IndexModel::builder()
                    .keys(keys)
                    .options(IndexOptions::builder().unique(unique).build())
                    .build(),
            )
            .await?;
    }
    Ok(())
}
