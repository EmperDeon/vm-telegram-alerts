pub mod config;

use crate::alerts::config::AlertStatus;
use crate::db::{alert_state, user};
use crate::db::user::set_authorized;
use crate::util::formatted_elapsed;
use std::collections::HashMap;
use std::error::Error;
use teloxide::prelude::AutoSend;
use teloxide::{prelude::*, utils::command::BotCommand};

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
  #[command(description = "Get status of alerts: /status")]
  Status,
  #[command(description = "Unsubscribe from notifications")]
  Stop,
}

async fn answer(cx: Context, command: Command) -> Result<(), Box<dyn Error + Send + Sync>> {
  match command {
    Command::Help => cx.answer(Command::descriptions()).await?,
    Command::Status => status(&cx).await?,
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

async fn status(cx: &Context) -> anyhow::Result<teloxide::prelude::Message> {
  let user = user::get_user(cx.update.chat_id()).await?;

  if !user.authorized {
    return Ok(cx.answer("Unauthorized").await?)
  }

  let mut response = "Alerts status:".to_owned();
  let config = crate::CONFIG.alerts.clone();

  for alert in config.alerts {
    let state = alert_state::get_alert_state(&alert).await?;

    let mut statuses: Vec<_> = state.status.iter().collect();
    statuses.sort_by_key(|v| state.status_last_changed(v.0.clone()));
    statuses.reverse();

    // Count statuses for quick overview
    let mut status_counts: HashMap<&'static str, usize> = HashMap::new();
    for (_, status) in &statuses {
      status_counts.insert(status.emoji(), status_counts.get(status.emoji()).unwrap_or(&0) + 1);
    }
    let mut status_counts = status_counts
      .iter()
      .map(|(key, num)| format!("{key}: {num}"))
      .collect::<Vec<_>>();
    status_counts.sort();
    let status_counts = status_counts.join(", ");

    response.push_str(format!("\n\n{} ({})", alert.name, status_counts).as_str());

    for (name, status) in statuses {
      if status.eq(&AlertStatus::Ok) {
        continue;
      }

      let duration = formatted_elapsed(state.status_last_changed(name.clone()));
      let message = format!("\n{} {name}: for {}", status.emoji(), duration);

      if response.len() + message.len() > 4096 {
        cx.answer(response.trim()).await?;
        response = "".to_owned();
      }

      response.push_str(message.as_str())
    }
  }

  Ok(cx.answer(response.trim()).await?)
}
