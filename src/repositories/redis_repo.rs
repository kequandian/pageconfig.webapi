use redis::{AsyncCommands, Client};
use anyhow::Result;
use serde_json::Value;

pub struct RedisRepository {
    client: Client,
}

impl RedisRepository {
    pub fn new(uri: &str) -> Result<Self> {
        let client = Client::open(uri)?;
        Ok(Self { client })
    }

    fn get_key(page_id: &str) -> String {
        format!("pageconfig:{}", page_id)
    }

    pub async fn get_config(&self, page_id: &str) -> Result<Option<Value>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = Self::get_key(page_id);
        let data: Option<String> = conn.get(key).await?;
        
        match data {
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
            None => Ok(None),
        }
    }

    pub async fn set_config(&self, page_id: &str, config: &Value) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = Self::get_key(page_id);
        let s = serde_json::to_string(config)?;
        conn.set::<_, _, ()>(key, s).await?;
        Ok(())
    }
}
