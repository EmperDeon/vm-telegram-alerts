use json::JsonValue;
use reqwest::header::HeaderMap;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::ops::Add;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub repeat_interval_secs: u64,

  #[serde(default)]
  pub no_data_message: String,

  #[serde(default)]
  pub datasources: HashMap<String, Datasource>,

  #[serde(default)]
  pub alerts: Vec<Alert>,
}

pub fn init_config(_: &crate::config::Config) -> anyhow::Result<Config> {
  let config = read_config()?;

  // Validate
  for alert in &config.alerts {
    if !config.datasources.contains_key(&alert.datasource) {
      return Err(anyhow::anyhow!("Could not find datasource {}", alert.datasource));
    }
  }

  Ok(config)
}

fn read_config() -> anyhow::Result<Config> {
  let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "/vm_telegram.yml".to_owned());
  let config_path = std::path::Path::new(&config_path)
    .to_path_buf()
    .to_str()
    .unwrap()
    .to_owned();

  match File::open(config_path) {
    Ok(input) => match serde_yaml::from_reader(input) {
      Ok(config) => Ok(config),
      Err(err) => {
        log::error!("Failed to read config: {}", err);
        Ok(Config::default())
      }
    },
    Err(_) => Ok(Config::default()),
  }
}

////
// Datasources
////

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Datasource {
  url: String,

  #[serde(default = "DatasourceAuth::default")]
  auth: DatasourceAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DatasourceAuth {
  None,
  AuthorizationHeader(String),
}

impl Default for DatasourceAuth {
  fn default() -> DatasourceAuth {
    DatasourceAuth::None
  }
}

impl Datasource {
  pub async fn fetch(&self, url: String) -> anyhow::Result<JsonValue> {
    let mut headers = HeaderMap::new();
    match self.auth.clone() {
      DatasourceAuth::None => {}
      DatasourceAuth::AuthorizationHeader(value) => {
        headers.insert("Authorization", value.parse().unwrap());
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
  pub interval_s: u32,

  pub query: String,
  pub condition: AlertCondition,
  pub condition_range_s: u64,
  pub graph_range_s: u64,
  #[serde(default = "Alert::default_graph_min")]
  pub graph_min: f32,
  #[serde(default = "Alert::default_graph_max")]
  pub graph_max: f32,

  #[serde(default = "Alert::default_step")]
  pub step: String,
  pub label: String,

  pub description: String,
  #[serde(default)]
  pub statuses: HashMap<AlertStatus, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
  Less,
  Greater,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
  Avg {
    condition: Condition,
    value: f32,
    value_ok: f32,
  },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AlertStatus {
  Ok,
  Err,
  NoData,
}

impl Default for AlertCondition {
  fn default() -> AlertCondition {
    AlertCondition::Avg {
      condition: Condition::Less,
      value: 0.0,
      value_ok: 0.0,
    }
  }
}

impl Alert {
  pub fn format_label(&self, json: &JsonValue) -> String {
    let mut label = self.label.clone();

    for (key, value) in json.entries() {
      // Replace {{key}} with value
      label = label.replace(format!("{{{{{}}}}}", key).as_str(), value.as_str().unwrap_or(""));
    }

    label
  }

  pub fn datasource_instance(&self) -> Datasource {
    // Datasource is validated in init_config, safe to unwrap
    crate::CONFIG.alerts.datasources.get(&self.datasource).unwrap().clone()
  }

  pub fn status_description(&self, status: &AlertStatus) -> String {
    match self.statuses.get(status) {
      None => "".to_owned(),
      Some(v) => v.clone(),
    }
  }

  fn default_step() -> String {
    "10s".to_owned()
  }
  // Used for calculating values range, these values guarantee that any value from storage would result in correct range
  fn default_graph_min() -> f32 {
    f32::MAX
  }
  fn default_graph_max() -> f32 {
    f32::MIN
  }
}

impl AlertStatus {
  pub fn emoji(&self) -> &'static str {
    match self {
      AlertStatus::Ok => "✅",
      AlertStatus::Err => "‼",
      AlertStatus::NoData => "️⚠️",
    }
  }
}

impl Default for AlertStatus {
  fn default() -> Self {
    AlertStatus::Ok
  }
}

impl Display for AlertStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      AlertStatus::Ok => {
        write!(f, "Ok")
      }
      AlertStatus::Err => {
        write!(f, "Firing")
      }
      AlertStatus::NoData => {
        write!(f, "No data")
      }
    }
  }
}
