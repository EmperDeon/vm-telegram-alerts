use crate::config::init_config;

mod alerts;
mod bot;
mod db;
mod config;

lazy_static::lazy_static! {
  pub static ref CONFIG: crate::config::Config = init_config().unwrap();
}

#[tokio::main]
async fn main() {
  launch().await.unwrap()
}

async fn launch() -> anyhow::Result<()> {
  init_config()?;
  teloxide::enable_logging!();

  alerts::launch_loop();
  bot::launch().await;

  Ok(())
}
