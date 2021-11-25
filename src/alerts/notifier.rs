use crate::alerts::config::{Alert, AlertStatus};
use crate::bot::create_bot;
use crate::db::user;
use crate::util::formatted_elapsed;
use teloxide::prelude::*;
use teloxide::types::InputFile;
use std::borrow::Cow;
use crate::alerts::chart;
use crate::db::alert_state::{AlertState};

pub async fn send_alert(alert: Alert, state: &AlertState, new_status: AlertStatus) -> anyhow::Result<()> {
  let bot = create_bot();
  let users = user::get_users().await?;

  let end = chrono::Utc::now().timestamp();
  let graph_start = end - (alert.graph_range_s as i64);
  let png_data: Vec<u8> = chart::generate_chart(&alert, graph_start, end).await?;
  let image = Cow::from(png_data);

  let duration = formatted_elapsed(state.status_last_changed());
  let message = format!("{} {} ({})\n\n{}\n\nWas: {} for {}", new_status.emoji(), new_status, alert.status_description(&new_status), alert.description, state.status, duration);

  for user in users {
    bot.send_photo(user.id.clone(), InputFile::Memory{ data: image.clone(), file_name: "alert.png".to_owned() }).caption(message.clone()).await?;
  }

  Ok(())
}
