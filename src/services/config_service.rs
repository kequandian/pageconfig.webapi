use crate::repositories::mongo_repo::MongoRepository;
use crate::repositories::redis_repo::RedisRepository;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::sync::Arc;

pub struct ConfigService {
    pub redis_repo: Arc<RedisRepository>,
    pub mongo_repo: Arc<MongoRepository>,
}

impl ConfigService {
    pub fn new(redis_repo: Arc<RedisRepository>, mongo_repo: Arc<MongoRepository>) -> Self {
        Self {
            redis_repo,
            mongo_repo,
        }
    }

    /// Retrieve configuration: Redis first, then MongoDB
    pub async fn get_config(&self, page_id: &str) -> Result<Value> {
        if let Some(config) = self.redis_repo.get_config(page_id).await? {
            return Ok(config);
        }

        if let Some(config) = self.mongo_repo.get_config(page_id).await? {
            // Update cache
            self.redis_repo.set_config(page_id, &config).await?;
            return Ok(config);
        }

        Err(anyhow::anyhow!("Configuration for {} not found", page_id))
    }

    /// Sync Redis cache to MongoDB
    pub async fn sync_to_mongo(&self, page_id: &str) -> Result<()> {
        let config = self.redis_repo
            .get_config(page_id)
            .await?
            .context(format!("No configuration in cache for {}", page_id))?;
            
        self.mongo_repo.save_config(page_id, config).await?;
        Ok(())
    }

    /// Add a field to a specific range (view, create, update, list)
    pub async fn add_field(&self, page_id: &str, range_code: &str, field: Value) -> Result<()> {
        let mut config = self.get_config(page_id).await?;
        let field_key = match range_code {
            "view" => "viewConfig",
            "create" => "createFields",
            "update" => "updateFields",
            "list" => "tableFields",
            _ => return Err(anyhow::anyhow!("Invalid range code: {}", range_code)),
        };

        if let Some(fields) = config.get_mut(field_key) {
            if let Some(arr) = fields.as_array_mut() {
                // Simplified duplicate check by 'field' key
                let new_field_name = field.get("field").and_then(|v| v.as_str()).unwrap_or("");
                if !arr.iter().any(|f| f.get("field").and_then(|v| v.as_str()) == Some(new_field_name)) {
                    arr.push(field);
                }
            }
        } else {
            // If the field key doesn't exist, initialize it
            config[field_key] = json!([field]);
        }

        self.redis_repo.set_config(page_id, &config).await?;
        Ok(())
    }

    /// Edit an existing field
    pub async fn edit_field(&self, page_id: &str, range_code: &str, field: Value) -> Result<()> {
        let mut config = self.get_config(page_id).await?;
        let field_key = match range_code {
            "view" => "viewConfig",
            "create" => "createFields",
            "update" => "updateFields",
            "list" => "tableFields",
            _ => return Err(anyhow::anyhow!("Invalid range code: {}", range_code)),
        };

        if let Some(arr) = config.get_mut(field_key).and_then(|v| v.as_array_mut()) {
            let field_name = field.get("field").and_then(|v| v.as_str()).context("Missing field name")?;
            if let Some(pos) = arr.iter().position(|f| f.get("field").and_then(|v| v.as_str()) == Some(field_name)) {
                arr[pos] = field;
            } else {
                return Err(anyhow::anyhow!("Field {} not found in {}", field_name, range_code));
            }
        }

        self.redis_repo.set_config(page_id, &config).await?;
        Ok(())
    }

    /// Delete a field
    pub async fn delete_field(&self, page_id: &str, range_code: &str, field_name: &str) -> Result<()> {
        let mut config = self.get_config(page_id).await?;
        let field_key = match range_code {
            "view" => "viewConfig",
            "create" => "createFields",
            "update" => "updateFields",
            "list" => "tableFields",
            _ => return Err(anyhow::anyhow!("Invalid range code: {}", range_code)),
        };

        if let Some(arr) = config.get_mut(field_key).and_then(|v| v.as_array_mut()) {
            arr.retain(|f| f.get("field").and_then(|v| v.as_str()) != Some(field_name));
        }

        self.redis_repo.set_config(page_id, &config).await?;
        Ok(())
    }

    // Additional mutation methods (add_action, edit_action, etc.) can be added similarly
}
