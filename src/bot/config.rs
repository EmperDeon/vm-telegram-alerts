#[derive(Debug, Default, Clone)]
pub struct Config {
  pub name: String,
  pub auth_token: String
}

pub fn init_config(_: &crate::config::Config) -> anyhow::Result<Config> {
  let mut config = Config::default();

  std::env::set_var("TELOXIDE_TOKEN", std::env::var("BOT_TOKEN")?);

  config.name = std::env::var("BOT_NAME").unwrap();
  config.auth_token = std::env::var("BOT_AUTH_TOKEN").unwrap_or("Auth".to_owned());

  Ok(config)
}
