
use chrono::prelude::*;

fn main() {
    let start_time = Utc::now();
    println!("{}", start_time.timestamp_millis());
    let total_time = start_time.timestamp_millis() - Utc::now().timestamp_millis();
    println!("{}", total_time);

    println!();
    println!();
    println!("{}", Local::now());
    let real_date = Utc::now().naive_local().checked_add_offset(FixedOffset::east_opt(3*3600).unwrap()).unwrap();
    println!("{}", real_date);
    println!("{}", Utc::now());
    println!("{}", Utc::now().naive_local());
    println!("{}", Utc::now().naive_utc());
    println!();
    println!();
    
    let date_1 = NaiveDateTime::new(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), 
                                                NaiveTime::from_hms_milli_opt(0, 1, 2, 3).unwrap());
    let date_2 = Utc::now().naive_local();
    let time_delta = date_2 - date_1;
    println!("{}", time_delta);
    println!("{}", time_delta.num_milliseconds());
}
