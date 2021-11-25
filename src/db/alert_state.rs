use crate::alerts::config::{Alert, AlertStatus};
use crate::db::{init_db, upsert};
use serde_derive::{Deserialize, Serialize};
use mongodb::Collection;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AlertState {
  #[serde(rename = "_id")]
  pub id: String,

  #[serde(default)]
  pub status: AlertStatus,

  #[serde(default)]
  pub status_last_changed: i64,

  #[serde(default)]
  pub counter: u32
}

impl AlertState {
  pub fn update_status(&mut self, new_status: AlertStatus) {
    self.status = new_status;
    self.status_last_changed = chrono::Utc::now().timestamp();
  }

  pub fn status_last_changed(&self) -> chrono::DateTime<chrono::Utc> {
    let naive = chrono::NaiveDateTime::from_timestamp(self.status_last_changed, 0);

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
  let state = states.find_one(bson::doc!{ "_id": &alert.name }, None).await?;

  match state {
    Some(state) => Ok(state),
    None => {
      let mut state = AlertState::default();
      state.id = alert.name.clone();

      Ok(state)
    }
  }
}

pub async fn update_alert_state(state: AlertState) -> anyhow::Result<()> {
  let states = collection().await?;
  states.find_one_and_update(bson::doc!{ "_id": &state.id }, bson::doc!{ "$set": bson::to_document(&state)? }, upsert()).await?;

  Ok(())
}
