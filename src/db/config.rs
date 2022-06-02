#[derive(Debug, Default, Clone)]
pub struct Config {
  /// Mongo DB URL
  pub url: String,
  /// Mongo DB Name
  pub name: String,

  pub user: String,
  pub pass: String
}

pub fn init_config(_: &crate::config::Config) -> anyhow::Result<Config> {
  Ok(Config {
    url: std::env::var("MONGODB_URL").unwrap_or_else(|_| "mongodb://localhost:27017/vm-telegram".to_owned()),
    name: std::env::var("MONGODB_NAME").unwrap_or_else(|_| "vm-telegram".to_owned()),
    user: std::env::var("MONGODB_USER").unwrap_or_else(|_| "".to_owned()),
    pass: std::env::var("MONGODB_PASS").unwrap_or_else(|_| "".to_owned()),
  })
}
