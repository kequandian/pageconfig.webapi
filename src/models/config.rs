use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageConfig {
    pub page_name: PageName,
    pub list_api: String,
    pub create_api: Option<String>,
    pub get_api: Option<String>,
    pub update_api: Option<String>,
    pub delete_api: Option<String>,
    pub columns: Option<i32>,
    pub page_type: Option<String>, // Renamed from 'type' to avoid keyword conflict
    pub create_fields: Option<Vec<FieldConfig>>,
    pub update_fields: Option<Vec<FieldConfig>>,
    pub search_fields: Option<Vec<FieldConfig>>,
    pub table_fields: Option<Vec<FieldConfig>>,
    pub form_fields: Option<Vec<FieldConfig>>,
    pub table_actions: Option<Vec<ActionConfig>>,
    pub table_operation: Option<Vec<ActionConfig>>,
    pub view_config: Option<Value>, // Can be object or array
    pub map: Option<HashMap<String, MapConfig>>,
    pub layout: Option<Value>, 
    pub search_type: Option<String>,
    pub search_button_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageName {
    pub table: Option<String>,
    pub new: Option<String>,
    pub edit: Option<String>,
    pub view: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldConfig {
    pub label: String,
    pub field: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub props: Option<Value>,
    pub rules: Option<Vec<Value>>,
    pub options: Option<Value>,
    pub span: Option<i32>,
    pub width: Option<String>,
    pub toptips: Option<String>,
    pub default_value: Option<Value>,
    pub expect: Option<Value>,
    pub value_type: Option<String>,
    pub align: Option<String>,
    pub theme: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionConfig {
    pub title: String,
    #[serde(rename = "type")]
    pub action_type: String,
    pub options: Option<Value>,
    pub expect: Option<Value>,
    pub other_props: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapConfig {
    pub map: Option<HashMap<String, String>>,
    pub options: Option<Vec<Value>>,
    pub color: Option<Value>,
}
