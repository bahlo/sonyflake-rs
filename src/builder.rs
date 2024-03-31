use chrono::prelude::*;
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, Mutex},
};

use crate::{
    error::{BoxDynError, Error},
    sonyflake::{to_sonyflake_time, Internals, SharedSonyflake, Sonyflake, BIT_LEN_SEQUENCE},
};

/// A builder to build a [`Sonyflake`] generator.
///
/// [`Sonyflake`]: struct.Sonyflake.html
pub struct Builder<'a> {
    start_time: Option<DateTime<Utc>>,
    machine_id: Option<&'a dyn Fn() -> Result<u16, BoxDynError>>,
    check_machine_id: Option<&'a dyn Fn(u16) -> bool>,
}

impl<'a> Default for Builder<'a> {
    fn default() -> Self {
        Builder::new()
    }
}

impl<'a> Builder<'a> {
    /// Construct a new builder to call methods on for the [`Sonyflake`] construction.
    ///
    /// [`Sonyflake`]: struct.Sonyflake.html
    pub fn new() -> Self {
        Self {
            start_time: None,
            machine_id: None,
            check_machine_id: None,
        }
    }

    /// Sets the start time.
    /// If the time is ahead of current time, finalize will fail.
    pub fn start_time(mut self, start_time: DateTime<Utc>) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Sets the machine id.
    /// If the fn returns an error, finalize will fail.
    pub fn machine_id(mut self, machine_id: &'a dyn Fn() -> Result<u16, BoxDynError>) -> Self {
        self.machine_id = Some(machine_id);
        self
    }

    /// Set a function to check the machine id.
    /// If the fn returns false, finalize will fail.
    pub fn check_machine_id(mut self, check_machine_id: &'a dyn Fn(u16) -> bool) -> Self {
        self.check_machine_id = Some(check_machine_id);
        self
    }

    /// Finalize the builder to create a Sonyflake.
    pub fn finalize(self) -> Result<Sonyflake, Error> {
        let sequence = 1 << (BIT_LEN_SEQUENCE - 1);

        let start_time = if let Some(start_time) = self.start_time {
            if start_time > Utc::now() {
                return Err(Error::StartTimeAheadOfCurrentTime(start_time));
            }

            to_sonyflake_time(start_time)
        } else {
            to_sonyflake_time(Utc.with_ymd_and_hms(2014, 9, 1, 0, 0, 0).unwrap())
        };

        let machine_id = if let Some(machine_id) = self.machine_id {
            match machine_id() {
                Ok(machine_id) => machine_id,
                Err(e) => return Err(Error::MachineIdFailed(e)),
            }
        } else {
            lower_16_bit_private_ip()?
        };

        if let Some(check_machine_id) = self.check_machine_id {
            if !check_machine_id(machine_id) {
                return Err(Error::CheckMachineIdFailed);
            }
        }

        let shared = Arc::new(SharedSonyflake {
            internals: Mutex::new(Internals {
                sequence,
                elapsed_time: 0,
            }),
            start_time,
            machine_id,
        });
        Ok(Sonyflake::new_inner(shared))
    }
}

fn private_ipv4() -> Option<Ipv4Addr> {
    pnet_datalink::interfaces()
        .iter()
        .filter(|interface| interface.is_up() && !interface.is_loopback())
        .flat_map(|interface| interface.ips.iter())
        .filter_map(|network| match network.ip() {
            IpAddr::V4(ipv4) => Some(ipv4),
            IpAddr::V6(_) => None,
        })
        .find(is_private_ipv4)
}

fn is_private_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 10
        || octets[0] == 172 && (octets[1] >= 16 && octets[1] < 32)
        || octets[0] == 192 && octets[1] == 168
}

pub(crate) fn lower_16_bit_private_ip() -> Result<u16, Error> {
    match private_ipv4() {
        Some(ip) => {
            let octets = ip.octets();
            Ok(((octets[2] as u16) << 8) + (octets[3] as u16))
        }
        None => Err(Error::NoPrivateIPv4),
    }
}
