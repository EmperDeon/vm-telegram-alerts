use crate::alerts::chart;
use crate::alerts::config::{Alert, AlertStatus};
use crate::bot::create_bot;
use crate::db::alert_state::AlertState;
use crate::db::{alert_state, user};
use crate::util::formatted_elapsed;
use crate::CONFIG;
use chrono::SecondsFormat;
use std::borrow::Cow;
use std::collections::HashMap;
use teloxide::prelude::*;
use teloxide::types::InputFile;
use teloxide::{ApiError, RequestError};

pub async fn send_alert(
  alert: Alert,
  label: String,
  state: &AlertState,
  new_status: AlertStatus,
) -> anyhow::Result<()> {
  let bot = create_bot();
  let users = user::get_users().await?;

  let end = chrono::Utc::now().timestamp();
  let graph_start = end - (alert.graph_range_s as i64);
  let png_data: Vec<u8> = chart::generate_chart(&alert, graph_start, end, Some(label.clone())).await?;
  let image = Cow::from(png_data);

  let duration = formatted_elapsed(state.status_last_changed(label.clone()));
  let message = format!(
    "{} {}({}): {} ({})\n{}\n\nWas: {} for {}",
    new_status.emoji(),
    alert.name,
    label,
    new_status,
    alert.status_description(&new_status),
    alert.description,
    state.status.get(&label).unwrap_or(&AlertStatus::NoData),
    duration
  );

  for user in users {
    bot
      .send_photo(
        user.id.clone(),
        InputFile::Memory {
          data: image.clone(),
          file_name: "alert.png".to_owned(),
        },
      )
      .caption(message.clone())
      .await?;
  }

  Ok(())
}

pub async fn send_no_data_alert(alert: Alert) -> anyhow::Result<()> {
  let bot = create_bot();
  let users = user::get_users().await?;

  let message = format!(
    "{} {}: {} {}",
    AlertStatus::NoData.emoji(),
    alert.name,
    AlertStatus::NoData,
    CONFIG.alerts.no_data_message
  );

  for user in users {
    bot.send_message(user.id.clone(), message.clone()).await?;
  }

  Ok(())
}

pub async fn refresh_pinned() -> anyhow::Result<()> {
  let bot = create_bot();
  let users = user::get_users().await?;

  let mut statuses: HashMap<&'static str, usize> = HashMap::new();
  let config = crate::CONFIG.alerts.clone();

  for alert in config.alerts {
    let state = alert_state::get_alert_state(&alert).await?;
    for status in state.status.values() {
      statuses.insert(status.emoji(), statuses.get(status.emoji()).unwrap_or(&0) + 1);
    }
  }
  let mut status = statuses
    .iter()
    .map(|(key, num)| format!("{key}: {num}"))
    .collect::<Vec<_>>();
  status.sort();
  let status = status.join(", ");

  for mut user in users {
    if user.pinned_message_id == 0 {
      bot.unpin_all_chat_messages(user.id.clone()).await?;
      let message = bot.send_message(user.id.clone(), "Pinned").await?;
      bot.pin_chat_message(user.id.clone(), message.id).await?;

      user::set_pinned(&mut user, message.id).await?;
    }

    let message = format!(
      "Status at {}: {}",
      chrono::Utc::now()
        .to_rfc3339_opts(SecondsFormat::Secs, false)
        .replace("+00:00", ""),
      status.clone()
    );
    let result = bot
      .edit_message_text(user.id.clone(), user.pinned_message_id, message)
      .await;
    if let Err(err) = result {
      match err {
        RequestError::ApiError { kind, status_code } => {
          if ApiError::MessageNotModified != kind {
            return Err(RequestError::ApiError { kind, status_code }.into());
          }
        }
        err => return Err(err.into()),
      }
    }
  }

  Ok(())
}
