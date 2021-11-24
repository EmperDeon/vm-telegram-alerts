use serde_derive::{Serialize, Deserialize};
use std::fs::File;
use std::collections::HashMap;
use json::JsonValue;
use reqwest::header::HeaderMap;
use std::ops::Add;
use crate::db::alert_state::AlertStatus;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub fetch_interval_ms: u64,

  #[serde(default)]
  pub datasources: HashMap<String, Datasource>,

  #[serde(default)]
  pub alerts: Vec<Alert>
}

pub fn init_config(_: &crate::config::Config) -> anyhow::Result<Config> {
  let mut config = read_config()?;

  let val = std::env::var("FETCH_INTERVAL_MS").unwrap_or("".to_owned());
  if val.trim().len() > 0 {
    config.fetch_interval_ms = val.parse().unwrap_or(1000);
  } else if config.fetch_interval_ms == u64::default() {
    config.fetch_interval_ms = 1000;
  }

  // Validate
  for alert in &config.alerts {
    if !config.datasources.contains_key(&alert.datasource) {
      return Err(anyhow::anyhow!("Could not find datasource {}", alert.datasource))
    }
  }

  Ok(config)
}

fn read_config() -> anyhow::Result<Config> {
  let config_path = std::env::var("CONFIG_PATH").unwrap_or("/vm_telegram.yml".to_owned());
  let config_path = std::path::Path::new(&config_path).to_path_buf().to_str().unwrap().to_owned();

  match File::open(config_path) {
    Ok(input) => {
      match serde_yaml::from_reader(input) {
        Ok(config) => Ok(config),
        Err(_) => Ok(Config::default())
      }
    }
    Err(_) => Ok(Config::default())
  }
}


////
// Datasources
////

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Datasource {
  url: String,

  #[serde(default = "DatasourceAuth::default")]
  auth: DatasourceAuth
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DatasourceAuth {
  None,
  AuthorizationHeader(String)
}

impl Default for DatasourceAuth {
  fn default() -> DatasourceAuth { DatasourceAuth::None }
}

impl Datasource {
  pub async fn fetch(&self, url: String) -> anyhow::Result<JsonValue> {
    let mut headers = HeaderMap::new();
    match self.auth.clone() {
      DatasourceAuth::None => {}
      DatasourceAuth::AuthorizationHeader(value) => {
        headers.insert("Authorization", value.clone().parse().unwrap());
      }
    }

    let client = reqwest::Client::builder().default_headers(headers).build()?;
    let response = client.get(self.url.clone().add("/").add(&url)).send().await?;

    Ok(json::parse(response.text().await?.as_str())?)
  }
}

////
// Alerts
////

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Alert {
  pub name: String,
  pub datasource: String,

  // Call alert once in `interval` times
  #[serde(default)]
  pub interval: u32,

  pub query: String,
  pub condition: AlertCondition,
  pub condition_range_s: u64,
  pub graph_range_s: u64,

  #[serde(default = "Alert::default_step")]
  pub step: String,
  pub label: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
  Less,
  Greater
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
  Avg { condition: Condition, value: f32 }
}

impl Default for AlertCondition {
  fn default() -> AlertCondition { AlertCondition::Avg { condition: Condition::Less, value: 0.0 } }
}

impl Alert {
  pub fn format_label(&self, json: &JsonValue) -> String {
    let mut label = self.label.clone();

    for (key, value) in json.entries() {
      // Replace {{key}} with value
      label = label.replace(format!("{{{{{}}}}}", key).as_str(), value.as_str().unwrap_or(""));
    }

    return label;
  }

  pub fn datasource_instance(&self) -> Datasource {
    // Datasource is validated in init_config, safe to unwrap
    crate::CONFIG.alerts.datasources.get(&self.datasource).unwrap().clone()
  }

  pub fn description_for(&self, status: &AlertStatus) -> String {
    "Description".to_owned()
  }

  fn default_step() -> String { "10s".to_owned() }
}
