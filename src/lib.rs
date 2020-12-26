use chrono::prelude::*;
use pnet::datalink;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    thread,
    time::Duration,
};
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
    #[error("could not find any private ipv4 address")]
    NoPrivateIPv4,
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

impl<'a> Settings<'a> {
    pub fn empty() -> Self {
        Self {
            start_time: None,
            machine_id: None,
            check_machine_id: None,
        }
    }
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
            lower_16_bit_private_ip()?
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

fn private_ipv4() -> Option<Ipv4Addr> {
    datalink::interfaces()
        .iter()
        .find(|interface| interface.is_up() && !interface.is_loopback())
        .and_then(|interface| {
            interface
                .ips
                .iter()
                .map(|ip_addr| ip_addr.ip()) // convert to std
                .find(|ip_addr| match ip_addr {
                    IpAddr::V4(ipv4) => is_private_ipv4(*ipv4),
                    IpAddr::V6(_) => false,
                })
                .and_then(|ip_addr| match ip_addr {
                    IpAddr::V4(ipv4) => Some(ipv4), // make sure the return type is Ipv4Addr
                    _ => None,
                })
        })
}

fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 10
        || octets[0] == 172 && (octets[1] >= 16 && octets[1] < 32)
        || octets[0] == 192 && octets[1] == 168
}

fn lower_16_bit_private_ip() -> Result<u16, Error> {
    match private_ipv4() {
        Some(ip) => {
            let octets = ip.octets();
            Ok((((octets[2] as u16) << 8) + (octets[3] as u16)).into())
        }
        None => Err(Error::NoPrivateIPv4),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_id() {
        let mut sf = Sonyflake::new(Settings::empty()).expect("Could not construct Sonyflake");
        assert!(sf.next_id().is_ok());
    }
}
