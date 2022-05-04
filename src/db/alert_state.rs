use std::collections::HashMap;
use crate::alerts::config::{Alert, AlertStatus};
use crate::db::{init_db, upsert};
use mongodb::Collection;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AlertState {
  #[serde(rename = "_id")]
  pub id: String,

  #[serde(default)]
  pub status: HashMap<String, AlertStatus>,

  #[serde(default)]
  pub status_last_changed: HashMap<String, i64>,

  #[serde(default)]
  pub counter: u32,
}

impl AlertState {
  pub fn update_status(&mut self, label: String, new_status: AlertStatus) {
    self.status.insert(label.clone(), new_status);
    self.status_last_changed.insert(label, chrono::Utc::now().timestamp());
  }

  pub fn status_last_changed(&self, label: String) -> chrono::DateTime<chrono::Utc> {
    let naive = chrono::NaiveDateTime::from_timestamp(*self.status_last_changed.get(&label).unwrap_or(&0), 0);

    chrono::DateTime::from_utc(naive, chrono::Utc)
  }
}

////
// Operations
////

async fn collection() -> anyhow::Result<Collection<AlertState>> {
  let db = init_db(crate::CONFIG.clone()).await.unwrap();

  Ok(db.collection("alert_states"))
}

pub async fn get_alert_state(alert: &Alert) -> anyhow::Result<AlertState> {
  let states = collection().await?;
  let state = states.find_one(bson::doc! { "_id": &alert.name }, None).await?;

  match state {
    Some(state) => Ok(state),
    None => Ok(AlertState {
      id: alert.name.clone(),
      ..Default::default()
    }),
  }
}

pub async fn update_alert_state(state: AlertState) -> anyhow::Result<()> {
  let states = collection().await?;
  states
    .find_one_and_update(
      bson::doc! { "_id": &state.id },
      bson::doc! { "$set": bson::to_document(&state)? },
      upsert(),
    )
    .await?;

  Ok(())
}
