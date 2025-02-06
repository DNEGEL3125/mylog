use chrono::{NaiveDate, NaiveDateTime};

pub fn get_today_date() -> NaiveDate {
    chrono::prelude::Local::now().date_naive()
}

pub fn date_time_now() -> NaiveDateTime {
    chrono::prelude::Local::now().naive_local()
}
