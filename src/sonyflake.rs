use chrono::prelude::*;
use std::{collections::HashMap, thread, time::Duration};

use crate::error::*;

/// bit length of time
pub(crate) const BIT_LEN_TIME: u64 = 39;
/// bit length of sequence number
pub(crate) const BIT_LEN_SEQUENCE: u64 = 8;
/// bit length of machine id
pub(crate) const BIT_LEN_MACHINE_ID: u64 = 63 - BIT_LEN_TIME - BIT_LEN_SEQUENCE;

/// Sonyflake is a distributed unique ID generator.
pub struct Sonyflake {
    pub(crate) start_time: i64,
    pub(crate) elapsed_time: i64,
    pub(crate) sequence: u16,
    pub(crate) machine_id: u16,
}

impl Sonyflake {
    /// Generate the next unique id.
    /// After the Sonyflake time overflows, next_id returns an error.
    pub fn next_id(&mut self) -> Result<u64, Error> {
        let mask_sequence = 1 << BIT_LEN_SEQUENCE - 1;

        let current = current_elapsed_time(self.start_time);
        if self.elapsed_time < current {
            self.elapsed_time = current;
            self.sequence = 0;
        } else {
            // self.elapsed_time >= current
            self.sequence = (self.sequence + 1) & mask_sequence;
            if self.sequence == 0 {
                self.elapsed_time += 1;
                let overtime = self.elapsed_time - current;
                thread::sleep(sleep_time(overtime));
            }
        }

        self.to_id()
    }

    fn to_id(&self) -> Result<u64, Error> {
        if self.elapsed_time >= 1 << BIT_LEN_TIME {
            return Err(Error::OverTimeLimit);
        }

        Ok(
            (self.elapsed_time as u64) << (BIT_LEN_SEQUENCE + BIT_LEN_MACHINE_ID)
                | (self.sequence as u64) << BIT_LEN_MACHINE_ID
                | (self.machine_id as u64),
        )
    }
}

const SONYFLAKE_TIME_UNIT: i64 = 10_000_000; // nanoseconds, i.e. 10msec

pub(crate) fn to_sonyflake_time(time: DateTime<Utc>) -> i64 {
    time.timestamp_nanos() / SONYFLAKE_TIME_UNIT
}

fn current_elapsed_time(start_time: i64) -> i64 {
    to_sonyflake_time(Utc::now()) - start_time
}

fn sleep_time(overtime: i64) -> Duration {
    Duration::from_millis(overtime as u64 * 10)
        - Duration::from_nanos((Utc::now().timestamp_nanos() % SONYFLAKE_TIME_UNIT) as u64)
}

/// Break a Sonyflake ID up into its parts.
pub fn decompose(id: u64) -> HashMap<String, u64> {
    let mut map = HashMap::new();

    let mask_sequence = (1 << BIT_LEN_SEQUENCE - 1) << BIT_LEN_MACHINE_ID;
    let mask_machine_id = 1 << BIT_LEN_MACHINE_ID - 1;

    map.insert("id".into(), id);
    map.insert("msb".into(), id >> 63);
    map.insert("time".into(), id >> (BIT_LEN_SEQUENCE + BIT_LEN_MACHINE_ID));
    map.insert("sequence".into(), id & mask_sequence >> BIT_LEN_MACHINE_ID);
    map.insert("machine-id".into(), id & mask_machine_id);

    map
}
