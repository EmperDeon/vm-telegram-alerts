#[derive(Debug, Default, Clone)]
pub struct Config {
  pub alerts: crate::alerts::config::Config,
  pub bots: crate::bot::config::Config,
  pub db: crate::db::config::Config,
}

pub fn init_config() -> anyhow::Result<Config> {
  let mut config = Config::default();

  if dotenv::from_filename(".env").is_ok() {
    log::info!("Loaded .env")
  }

  config.alerts = crate::alerts::config::init_config(&config).unwrap();
  config.bots = crate::bot::config::init_config(&config).unwrap();
  config.db = crate::db::config::init_config(&config).unwrap();

  Ok(config)
}
