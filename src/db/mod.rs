use mongodb::options::{ClientOptions, FindOneAndUpdateOptions};
use mongodb::{Client, Database};
use crate::config::Config;

pub mod config;
pub mod alert_state;
pub mod user;

pub async fn init_db(config: Config) -> anyhow::Result<Database> {
  let mut client_options = ClientOptions::parse(config.db.url).await?;

  client_options.app_name = Some("vm-telegram.rust".to_string());

  let client = Client::with_options(client_options)?;
  let db = client.database(&config.db.name);

  Ok(db)
}

pub fn upsert() -> Option<FindOneAndUpdateOptions> {
  Some(bson::from_document::<FindOneAndUpdateOptions>(bson::doc!{ "upsert": true }).unwrap())
}
