use chrono::{DateTime, Datelike, Timelike};

pub fn ts_at_zero(seconds:i64, tz: i32) -> i64 {
    ts_at_hour(seconds, tz, 0)
}

pub fn ts_at_hour(seconds:i64, tz: i32, hour: i32) -> i64{
    let parser = DateTime::from_timestamp(seconds, 0);
    let h:i64 = (parser.unwrap().time().hour() * 3600) as i64;
    let m:i64 = (parser.unwrap().time().minute() * 60) as i64;
    let s:i64 = (parser.unwrap().time().second()) as i64;
    parser.unwrap().timestamp() - h - m - s + ((tz+hour) * 3600) as i64
}

pub fn ts_dow(seconds:i64, tz:i32) -> u32 {
    let parser = DateTime::from_timestamp(seconds + (tz*3600) as i64, 0);
    parser.unwrap().weekday().number_from_monday()
}

pub fn ts_week(seconds:i64, tz:i32) -> u32 {
    let parser = DateTime::from_timestamp(seconds + (tz*3600) as i64, 0);
    parser.unwrap().iso_week().week()
}