use crate::alerts::config::{Alert, AlertCondition, AlertStatus, Condition};
use crate::db::alert_state::{get_alert_state, update_alert_state};
use std::collections::HashMap;
use std::time::Duration;
use tokio::task::JoinHandle;

mod chart;
pub mod config;
mod notifier;

type Values = Vec<(u64, f32)>;

pub fn launch_loop() -> JoinHandle<()> {
  tokio::spawn(async {
    let duration = crate::CONFIG.alerts.fetch_interval_ms;
    let mut interval = tokio::time::interval(Duration::from_millis(duration));

    loop {
      interval.tick().await;

      if let Err(err) = process_alerts().await {
        log::error!("Could not process alerts: {}", err)
      }
    }
  })
}

pub async fn process_alerts() -> anyhow::Result<()> {
  let config = crate::CONFIG.alerts.clone();

  let mut trigger_notifier = false;
  for alert in config.alerts {
    let mut state = get_alert_state(&alert).await?;

    let new_statuses = if state.counter >= alert.interval {
      state.counter = 1;
      trigger_notifier = true;

      calculate_status(&alert).await?
    } else {
      state.counter += 1;
      state.status.clone()
    };

    if state.status != new_statuses {
      for (label, new_status) in new_statuses.clone() {
        if state.status.contains_key(&label) && state.status.get(&label).unwrap() == new_statuses.get(&label).unwrap() {
          // Skip if status is not changed
          continue
        }

        let alert_name = alert.name.clone();
        match notifier::send_alert(alert.clone(), label.clone(), &state, new_status.clone()).await {
          Ok(_) => {}
          Err(e) => {
            log::error!("Failed to send notification for {}: {:?}", alert_name, e)
          }
        }

        state.update_status(label.clone(), new_status);
      }
    }

    update_alert_state(state).await?;
  }

  if trigger_notifier {
    notifier::refresh_pinned().await?;
  }

  Ok(())
}

async fn calculate_status(alert: &Alert) -> anyhow::Result<HashMap<String, AlertStatus>> {
  let end = chrono::Utc::now().timestamp();
  let start = end - (alert.condition_range_s as i64);
  let values = match request_values(alert, start, end).await {
    Ok(val) => val,
    Err(err) => {
      log::error!("Failed to request metrics for {}: {:?}", alert.name, err);
      HashMap::new()
    }
  };

  if values.is_empty() {
    return Ok(HashMap::new());
  }

  let mut firing: HashMap<String, AlertStatus> = HashMap::new();
  for (label, values) in values {
    let result = match alert.condition.clone() {
      AlertCondition::Avg { condition, value } => {
        let average = values.iter().map(|(_, v)| *v).reduce(|a, b| a + b).unwrap_or(0.0) / values.len() as f32;

        match condition {
          Condition::Less => average < value,
          Condition::Greater => average > value,
        }
      }
    };

    firing.insert(label.clone(), if result { AlertStatus::Err } else { AlertStatus::Ok });
  }

  Ok(firing)
}

pub async fn request_values(alert: &Alert, start: i64, end: i64) -> anyhow::Result<HashMap<String, Values>> {
  let mut result: HashMap<String, Values> = HashMap::new();
  let datasource = alert.datasource_instance();

  let response = datasource
    .fetch(format!(
      "api/v1/query_range?query={}&start={}&end={}&step={}",
      alert.query, start, end, alert.step
    ))
    .await?;
  let response = &response["data"]["result"];

  for response_result in response.members() {
    let mut values = Values::new();

    for value in response_result["values"].members() {
      let val: f32 = value[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
      values.push((value[0].as_u64().unwrap_or(0), val));
    }

    result.insert(alert.format_label(&response_result["metric"]), values);
  }

  Ok(result)
}
