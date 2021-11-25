use chrono::Timelike;

type DateTime = chrono::DateTime<chrono::Utc>;

pub fn formatted_elapsed(to: DateTime) -> String {
  let from = chrono::Utc::now().with_nanosecond(0).unwrap_or(chrono::Utc::now());
  let mut duration = from - to;

  let mut result = "".to_owned();

  result += &localize(duration.num_weeks(), "","1 week ", "weeks ");
  duration = duration.checked_sub(&chrono::Duration::weeks(duration.num_weeks())).unwrap();

  result += &localize(duration.num_days(), "","1 day ", "days ");
  duration = duration.checked_sub(&chrono::Duration::days(duration.num_days())).unwrap();

  result += &localize(duration.num_hours(), "","1 hour ", "hours ");
  duration = duration.checked_sub(&chrono::Duration::hours(duration.num_hours())).unwrap();

  result += &localize(duration.num_minutes(), "","1 minute ", "minutes ");
  duration = duration.checked_sub(&chrono::Duration::minutes(duration.num_minutes())).unwrap();

  if result.len() == 0 {
    result += &localize(duration.num_seconds(), "", "1 second ", "seconds ");
  }

  result
}

pub fn localize<'a>(num: i64, zero: &'a str, one: &'a str, many: &'a str) -> String {
  if num == 0 {
    zero.to_owned()
  } else if num == 1 {
    one.to_owned()
  } else {
    format!("{} {}", num, many)
  }
}
