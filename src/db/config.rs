#[derive(Debug, Default, Clone)]
pub struct Config {
  /// Mongo DB URL
  pub url: String,
  /// Mongo DB Name
  pub name: String
}

pub fn init_config(_: &crate::config::Config) -> anyhow::Result<Config> {
  let mut config = Config::default();

  config.url = std::env::var("MONGODB_URL").unwrap_or("mongodb://localhost:27017/vm-telegram".to_owned());
  config.name = std::env::var("MONGODB_NAME").unwrap_or("vm-telegram".to_owned());

  Ok(config)
}
