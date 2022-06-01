use crate::config::init_config;
use std::env;
use std::str::FromStr;

mod alerts;
mod bot;
mod config;
mod db;
mod util;

lazy_static::lazy_static! {
  pub static ref CONFIG: crate::config::Config = init_config().unwrap();
}

#[tokio::main]
async fn main() {
  launch().await.unwrap()
}

async fn launch() -> anyhow::Result<()> {
  init_config()?;

  pretty_env_logger::formatted_timed_builder()
    .filter(
      Some(&env!("CARGO_PKG_NAME").replace('-', "_")),
      log::LevelFilter::from_str(&env::var("RUST_LOG").unwrap_or_else(|_| String::from("info"))).unwrap(),
    )
    .init();

  alerts::launch_loop();
  bot::launch().await;

  Ok(())
}
