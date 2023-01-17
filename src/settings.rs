use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(my_settings_reader::SettingsModel, Serialize, Deserialize, Debug, Clone)]
pub struct SettingsModel {
    #[serde(rename = "SeqConnString")]
    pub seq_conn_string: String,
    #[serde(rename = "WsSettingsBaseUrl")]
    pub ws_settings_base_url: String,
    #[serde(rename = "PolygonToken")]
    pub polygon_token: String,
    #[serde(rename = "InstrumentsMapping")]
    pub instruments_mapping: HashMap<String, String>,
}


