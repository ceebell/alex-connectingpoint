use crate::{config::Config, error::AppError};
use mongodb::{
    Database,
    bson::{Document, doc},
    options::{Acknowledgment, DatabaseOptions, WriteConcern},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Database,
    pub http: reqwest::Client,
}

impl AppState {
    pub async fn connect(config: Config) -> Result<Self, AppError> {
        let mut options = mongodb::options::ClientOptions::parse(&config.mongodb_uri).await?;
        options.server_selection_timeout = Some(std::time::Duration::from_secs(5));
        options.connect_timeout = Some(std::time::Duration::from_secs(5));
        let client = mongodb::Client::with_options(options)?;
        let db = client.database_with_options(
            &config.mongodb_db,
            DatabaseOptions::builder()
                .write_concern(
                    WriteConcern::builder()
                        .w(Acknowledgment::Majority)
                        .journal(true)
                        .w_timeout(std::time::Duration::from_secs(10))
                        .build(),
                )
                .build(),
        );
        db.run_command(doc! {"ping": 1}).await?;
        let http = reqwest::Client::builder()
            .timeout(config.http_timeout())
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            config: Arc::new(config),
            db,
            http,
        })
    }

    pub fn collection(&self, name: &str) -> mongodb::Collection<Document> {
        self.db.collection(name)
    }

    pub fn scope(&self) -> Document {
        doc! {"shop_id": &self.config.shop_id, "channel_id": &self.config.channel_id}
    }
}
