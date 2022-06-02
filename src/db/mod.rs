use crate::config::Config;
use mongodb::options::{ClientOptions, Credential, FindOneAndUpdateOptions};
use mongodb::{Client, Database};

pub mod alert_state;
pub mod config;
pub mod user;

pub async fn init_db(config: Config) -> anyhow::Result<Database> {
  let mut client_options = ClientOptions::parse(config.db.url).await?;

  client_options.app_name = Some("vm-telegram.rust".to_string());

  if !config.db.user.is_empty() {
    let mut credential = Credential::default();

    credential.username = Some(config.db.user);
    credential.password = Some(config.db.pass);
    credential.source = Some("admin".to_owned());

    client_options.credential = Some(credential);
  }

  let client = Client::with_options(client_options)?;
  let db = client.database(&config.db.name);

  Ok(db)
}

pub fn upsert() -> Option<FindOneAndUpdateOptions> {
  Some(bson::from_document::<FindOneAndUpdateOptions>(bson::doc! { "upsert": true }).unwrap())
}
