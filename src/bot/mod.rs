pub mod config;

use teloxide::{prelude::*, utils::command::BotCommand};
use teloxide::prelude::AutoSend;
use std::error::Error;
use crate::db::user::{set_authorized};

type Context = UpdateWithCx<AutoSend<Bot>, Message>;

pub fn create_bot() -> AutoSend<Bot> {
  Bot::from_env().auto_send()
}

pub async fn launch() {
  teloxide::commands_repl(create_bot(), crate::CONFIG.bots.name.clone(), answer).await;
}

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
  #[command(description = "Display help")]
  Help,
  #[command(description = "Authorize and Subscribe to notifications, example: /auth code")]
  Auth(String),
  #[command(description = "Unsubscribe from notifications")]
  Stop,
}

async fn answer(cx: Context, command: Command) -> Result<(), Box<dyn Error + Send + Sync>> {
  match command {
    Command::Help => cx.answer(Command::descriptions()).await?,
    Command::Auth(token) => authorize(&cx, token).await?,
    Command::Stop => stop(&cx).await?,
  };

  Ok(())
}

async fn authorize(cx: &Context, token: String) -> anyhow::Result<teloxide::prelude::Message> {
  if token == crate::CONFIG.bots.auth_token {
    set_authorized(cx.update.chat_id(), true).await?;

    Ok(cx.answer("Successfully authorized").await?)
  } else {
    Ok(cx.answer("Unauthorized").await?)
  }
}
async fn stop(cx: &Context) -> anyhow::Result<teloxide::prelude::Message> {
  set_authorized(cx.update.chat_id(), false).await?;
  Ok(cx.answer("Stopped").await?)
}
