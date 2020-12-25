use chrono::prelude::*;
use std::{thread, time::Duration};
use thiserror::Error;

/// bit length of time
const BIT_LEN_TIME: u64 = 39;
/// bit length of sequence number
const BIT_LEN_SEQUENCE: u64 = 8;
/// bit length of machine id
const BIT_LEN_MACHINE_ID: u64 = 63 - BIT_LEN_TIME - BIT_LEN_SEQUENCE;

#[derive(Error, Debug)]
pub enum Error {
    #[error("start_time `{0}` is ahead of current time")]
    StartTimeAheadOfCurrentTime(DateTime<Utc>),
    #[error("machine_id returned an error: {0}")]
    MachineIdFailed(Box<dyn std::error::Error>),
    #[error("check_machine_id returned false")]
    CheckMachineIdFailed,
    #[error("over the time limit")]
    OverTimeLimit,
}

/// Settings configures Sonyflake:
///
/// start_time is the time since which the Sonyflake time is defined as the elapsed time.
/// If start_time is 0, the start time of the Sonyflake is set to "2014-09-01 00:00:00 +0000 UTC".
/// machine_id returns the unique ID of the Sonyflake instance.
/// Default machine_id returns the lower 16 bits of the private IP address.
/// check_machine_id validates the uniqueness of the machine ID.
pub struct Settings<'a> {
    pub start_time: Option<DateTime<Utc>>,
    pub machine_id: Option<&'a dyn Fn() -> Result<u16, Box<dyn std::error::Error>>>,
    pub check_machine_id: Option<&'a dyn Fn(u16) -> bool>,
}

/// Sonyflake is a distributed unique ID generator.
pub struct Sonyflake {
    start_time: i64,
    elapsed_time: i64,
    sequence: u16,
    machine_id: u16,
}

impl Sonyflake {
    /// Create a new Sonyflake configured with the given Settings.
    /// Returns an error in the following cases:
    /// - Settings.start_time is ahead of the current time.
    /// - Settings.machine_id returns an error.
    /// - Settings.check_machine_id returns false.
    pub fn new(settings: Settings) -> Result<Self, Error> {
        let sequence = 1 << BIT_LEN_SEQUENCE - 1;

        let start_time = if let Some(start_time) = settings.start_time {
            if start_time > Utc::now() {
                return Err(Error::StartTimeAheadOfCurrentTime(start_time));
            }

            to_sonyflake_time(start_time)
        } else {
            to_sonyflake_time(Utc.ymd(2014, 9, 1).and_hms(0, 0, 0))
        };

        let machine_id = if let Some(machine_id) = settings.machine_id {
            match machine_id() {
                Ok(machine_id) => machine_id,
                Err(e) => return Err(Error::MachineIdFailed(e)),
            }
        } else {
            lower_16_bit_private_ip()
        };

        if let Some(check_machine_id) = settings.check_machine_id {
            if !check_machine_id(machine_id) {
                return Err(Error::CheckMachineIdFailed);
            }
        }

        Ok(Self {
            sequence,
            start_time,
            machine_id,
            elapsed_time: 0,
        })
    }

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

/// nanoseconds, i.e. 10msec
const SONYFLAKE_TIME_UNIT: i64 = 10_000_000;

fn to_sonyflake_time(time: DateTime<Utc>) -> i64 {
    time.timestamp_nanos() / SONYFLAKE_TIME_UNIT
}

fn current_elapsed_time(start_time: i64) -> i64 {
    to_sonyflake_time(Utc::now()) - start_time
}

fn sleep_time(overtime: i64) -> Duration {
    Duration::from_millis(overtime as u64 * 10)
        - Duration::from_nanos((Utc::now().timestamp_nanos() % SONYFLAKE_TIME_UNIT) as u64)
}

fn lower_16_bit_private_ip() -> u16 {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
