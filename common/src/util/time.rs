use std::time::{SystemTime, UNIX_EPOCH};

// use chrono::format::strftime::StrftimeItems;
// use lazy_static::lazy_static;

// lazy_static! {
//     static ref DATETIME_FORMAT: StrftimeItems<'static> = StrftimeItems::new("%Y-%m-%d %H:%M:%S");
// }

pub fn unix_epoch_sec() -> u64 {
    let now = SystemTime::now();
    let d = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    d.as_secs()
}
