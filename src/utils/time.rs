use chrono::NaiveDate;

pub fn get_today_date() -> NaiveDate {
    return chrono::prelude::Local::now().date_naive();
}
