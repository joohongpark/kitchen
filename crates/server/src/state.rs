use std::sync::Arc;

use common::Db;
use media::MediaService;
use ntfy::NtfyClient;
use pantry::PantryService;
use queue::{Kv, Queue};
use recipes::RecipesService;

use crate::auth::AuthService;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pantry: PantryService,
    pub recipes: RecipesService,
    pub media: MediaService,
    pub queue: Queue,
    pub kv: Kv,
    pub auth: AuthService,
    pub ntfy: Option<NtfyClient>,
}

impl AppState {
    pub async fn new(config: Config, db: Db) -> anyhow::Result<Self> {
        let pantry = PantryService::new(db.clone());
        let recipes = RecipesService::new(db.clone(), pantry.clone());
        let media = MediaService::new(db.clone(), config.photos_dir()).await?;
        let http = reqwest_client()?;
        let ntfy = config
            .ntfy_topic
            .as_ref()
            .map(|topic| NtfyClient::new(http, &config.ntfy_url, topic, config.ntfy_token.clone()));
        Ok(Self {
            auth: AuthService::new(db.clone(), config.password.clone()),
            queue: Queue::new(db.clone()),
            kv: Kv::new(db.clone()),
            config: Arc::new(config),
            pantry,
            recipes,
            media,
            ntfy,
        })
    }
}

fn reqwest_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).user_agent("kitchen/0.1").build()?)
}
