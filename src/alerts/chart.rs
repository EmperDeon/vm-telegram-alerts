use crate::alerts::config::{Alert, AlertCondition, Condition};
use crate::alerts::request_values;
use chrono::{Utc};
use plotters::prelude::{ChartBuilder, IntoFont, BitMapBackend, IntoDrawingArea, LineSeries, RGBColor, PathElement, IntoTextStyle, TRANSPARENT, Polygon, Color};
use tempfile::tempdir;
use std::fs::File;
use std::io::{BufReader, Read};

const BLACK: RGBColor = RGBColor(23, 23, 27);
const WHITE: RGBColor = RGBColor(255, 255, 255);
const ERROR_POLYGON: RGBColor = RGBColor(255, 0, 0);
const COLORS: [RGBColor; 18] = [
  RGBColor(115, 191, 105), // Green
  RGBColor(87, 148, 242), // Blue
  RGBColor(242, 73, 92), // Red
  RGBColor(255, 152, 48), // Orange
  RGBColor(250, 222, 42), // Yellow
  RGBColor(184, 119, 217), // Magenta
  RGBColor(55, 135, 45),
  RGBColor(31, 96, 196),
  RGBColor(196, 22, 42),
  RGBColor(250, 100, 0),
  RGBColor(224, 180, 0),
  RGBColor(143, 59, 184),
  RGBColor(86, 166, 75),
  RGBColor(50, 116, 217),
  RGBColor(224, 47, 68),
  RGBColor(255, 120, 10),
  RGBColor(242, 204, 12),
  RGBColor(163, 82, 204)
];

pub async fn generate_chart(alert: &Alert, start: i64, end: i64) -> anyhow::Result<Vec<u8>> {
  let values = request_values(&alert, start, end).await;

  ////
  // Get constraints
  ////

  let end = parse_time(end);
  let start = parse_time(start);

  let mut min = alert.graph_min;
  let mut max = alert.graph_max;

  let name = match &values {
    Ok(values) => {
      for values in values.values().clone() {
        for (_, value) in values {
          min = min.min(*value);
          max = max.max(*value);
        }
      }

      min *= 0.95;
      max *= 1.05;

      alert.name.clone()
    }
    Err(err) => {
      log::error!("Failed to request metrics for {}: {:?}", alert.name, err);

      min = 0.0;
      max = 0.0;

      alert.name.clone() + " (Could not request data from storage)"
    }
  };

  ////
  // Render
  ////

  let tempdir = tempdir()?;
  let path = tempdir.path().join("chart.png");

  let root = BitMapBackend::new(path.to_str().unwrap(), (1280, 720)).into_drawing_area();
  root.fill(&BLACK)?;

  let mut chart = ChartBuilder::on(&root)
    .margin(10)
    .x_label_area_size(40)
    .y_label_area_size(40)
    .caption(name, ("sans-serif", 30.0).into_font().with_color(WHITE))
    .build_cartesian_2d(start..end, min..max)?;

  chart.configure_mesh().light_line_style(&TRANSPARENT).bold_line_style(&TRANSPARENT).axis_style(&WHITE).label_style(("sans-serif", 16).into_font().color(&WHITE)).draw()?;

  match values {
    Ok(values) => {
      let mut keys = values.keys().map(|v| v.clone()).collect::<Vec<String>>();
      keys.sort();

      for label in keys.clone() {
        let metric_values = values.get(&label).unwrap();
        let color = label_color(keys.iter().position(|v| v.eq(&label)).unwrap_or(0));

        chart.draw_series(
          LineSeries::new(metric_values.iter().map(|(timestamp, value)| (parse_time(timestamp.clone() as i64), value.clone())), &color),
        )?.label(&label).legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
      }

      let error_polygon = match alert.condition.clone() {
        AlertCondition::Avg { condition, value } => {
          match condition {
            Condition::Less => { vec![(start, min), (end, min), (end, value), (start, value)] }
            Condition::Greater => { vec![(start, value), (end, value), (end, max), (start, max)] }
          }
        }
      };
      chart.draw_series(std::iter::once(Polygon::new(error_polygon, &ERROR_POLYGON.mix(0.09))))?;
    }
    Err(_) => {}
  }

  chart.configure_series_labels().border_style(&WHITE).label_font(("sans-serif", 16).into_font().color(&WHITE)).draw()?;

  // To avoid the IO failure being ignored silently, we manually call the present function
  root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");

  ////
  // Read file
  ////

  let mut png_data: Vec<u8> = Vec::new();
  let file = File::open(path.clone())?;
  let mut reader = BufReader::new(file);

  reader.read_to_end(&mut png_data)?;

  Ok(png_data)
}

fn parse_time(t: i64) -> chrono::DateTime<Utc> {
  let naive = chrono::NaiveDateTime::from_timestamp(t, 0);

  chrono::DateTime::from_utc(naive, Utc)
}

fn label_color(index: usize) -> RGBColor {
  return COLORS[index % 18];
}
