#[derive(Debug, Default, Clone)]
pub struct Config {
  pub alerts: crate::alerts::config::Config,
  pub bots: crate::bot::config::Config,
  pub db: crate::db::config::Config,
}

pub fn init_config() -> anyhow::Result<Config> {
  let mut config = Config::default();

  if let Ok(_) = dotenv::from_filename(".env") {
    log::info!("Loaded .env")
  }

  config.alerts = crate::alerts::config::init_config(&config).unwrap();
  config.bots = crate::bot::config::init_config(&config).unwrap();
  config.db = crate::db::config::init_config(&config).unwrap();

  return Ok(config)
}
