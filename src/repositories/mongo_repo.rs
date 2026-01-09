use mongodb::{Client, Collection, Database};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct MongoPageConfig {
    #[serde(rename = "_id")]
    pub page_id: String,
    pub config: Value, // Store as raw JSON value for flexibility
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct MongoRepository {
    collection: Collection<MongoPageConfig>,
}

impl MongoRepository {
    pub async fn new(uri: &str, db_name: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database(db_name);
        let collection = db.collection::<MongoPageConfig>("page_configs");
        Ok(Self { collection })
    }

    pub async fn get_config(&self, page_id: &str) -> Result<Option<Value>> {
        let filter = mongodb::bson::doc! { "_id": page_id };
        let result = self.collection.find_one(filter, None).await?;
        Ok(result.map(|doc| doc.config))
    }

    pub async fn save_config(&self, page_id: &str, config: Value) -> Result<()> {
        let options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        let query = mongodb::bson::doc! { "_id": page_id };
        let update = mongodb::bson::doc! {
            "$set": {
                "config": mongodb::bson::to_bson(&config)?,
                "updated_at": mongodb::bson::DateTime::now(),
            }
        };
        self.collection.update_one(query, update, options).await?;
        Ok(())
    }
}
