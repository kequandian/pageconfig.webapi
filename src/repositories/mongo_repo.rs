use common_mongo_db::{JsonRepository, MongoDb};
use std::sync::Arc;
use anyhow::Result;
use serde_json::Value;

pub struct MongoRepository {
    repo: Arc<JsonRepository>,
}

impl MongoRepository {
    pub async fn new(uri: &str, db_name: &str) -> Result<Self> {
        let mongo = MongoDb::new(uri, db_name).await?;
        // 集合名 "pageconfigs"，数据字段 "config"
        let repo = Arc::new(JsonRepository::new(mongo, "pageconfigs", "config"));
        Ok(Self { repo })
    }

    pub async fn get_config(&self, page_id: &str) -> Result<Option<Value>> {
        self.repo.get(page_id).await.map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn save_config(&self, page_id: &str, config: Value) -> Result<()> {
        self.repo.upsert(page_id.to_string(), config).await.map_err(|e| anyhow::anyhow!(e))
    }
}
