use parking_lot::Mutex;

use blog_common::util::time;

static LAST_TIMESTAMP: Mutex<u64> = Mutex::new(0);

const START_TIME_MILLIS: u64 = 1643212800;

pub(crate) fn gen_id() -> u64 {
    loop {
        let current_timestamp = time::unix_epoch_sec();
        loop {
            let mut last_timestamp = LAST_TIMESTAMP.lock();
            let mut sequence = 0u8;
            if *last_timestamp == current_timestamp {
                // u16::MAX;
                if sequence == u8::MAX {
                    break;
                } else {
                    sequence += 1;
                }
            } else {
                *last_timestamp = current_timestamp;
            }
            let id = (current_timestamp - START_TIME_MILLIS) << 8;
            if sequence == 0 {
                return id;
            } else {
                return id | sequence as u64;
            }
        }
    }
}
