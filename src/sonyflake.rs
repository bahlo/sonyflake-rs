use chrono::prelude::*;
use std::{
    fmt,
    ops::Deref,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{builder::Builder, error::Error};

/// bit length of time
pub(crate) const BIT_LEN_TIME: u64 = 39;
/// bit length of sequence number
pub(crate) const BIT_LEN_SEQUENCE: u64 = 8;
/// bit length of machine id
pub(crate) const BIT_LEN_MACHINE_ID: u64 = 63 - BIT_LEN_TIME - BIT_LEN_SEQUENCE;

const GENERATE_MASK_SEQUENCE: u16 = (1 << BIT_LEN_SEQUENCE) - 1;

/// A generated Sonyflake id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl Id {
    /// Returns the inner `u64` for this id.
    #[must_use]
    pub fn to_u64(self) -> u64 {
        self.0
    }
}

impl Deref for Id {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
pub(crate) struct Internals {
    pub(crate) elapsed_time: i64,
    pub(crate) sequence: u16,
}

#[derive(Debug)]
pub(crate) struct SharedSonyflake {
    pub(crate) start_time: i64,
    pub(crate) machine_id: u16,
    pub(crate) internals: Mutex<Internals>,
}

/// Sonyflake is a distributed unique ID generator.
#[derive(Debug, Clone)]
pub struct Sonyflake(pub(crate) Arc<SharedSonyflake>);

impl Sonyflake {
    /// Create a new Sonyflake with the default configuration.
    /// For custom configuration see [`builder`].
    ///
    /// # Errors
    ///
    /// Returns an error if the finalization failed.
    ///
    /// [`builder`]: struct.Sonyflake.html#method.builder
    pub fn new() -> Result<Self, Error> {
        Builder::new().finalize()
    }

    /// Create a new [`Builder`] to construct a Sonyflake.
    ///
    /// [`Builder`]: struct.Builder.html
    #[must_use]
    pub fn builder<'a>() -> Builder<'a> {
        Builder::new()
    }

    pub(crate) fn new_inner(shared: Arc<SharedSonyflake>) -> Self {
        Self(shared)
    }

    /// Generate the next unique id.
    /// After the Sonyflake time overflows, `next_id` returns an error.
    ///
    /// # Errors
    ///
    /// This function returns an error if it can't determin the elapsed time.
    /// It also returns an error if it's over the time limit.
    #[allow(clippy::cast_sign_loss)]
    pub fn next_id(&self) -> Result<Id, Error> {
        let mut internals = self.0.internals.lock().map_err(|_| Error::MutexPoisoned)?;

        let current = current_elapsed_time(self.0.start_time)?;
        if internals.elapsed_time < current {
            internals.elapsed_time = current;
            internals.sequence = 0;
        } else {
            // self.elapsed_time >= current
            internals.sequence = (internals.sequence + 1) & GENERATE_MASK_SEQUENCE;
            if internals.sequence == 0 {
                internals.elapsed_time += 1;
                let overtime = internals.elapsed_time - current;
                thread::sleep(sleep_time(overtime)?);
            }
        }

        if internals.elapsed_time >= 1 << BIT_LEN_TIME {
            return Err(Error::OverTimeLimit);
        }

        Ok(Id((internals.elapsed_time as u64)
            << (BIT_LEN_SEQUENCE + BIT_LEN_MACHINE_ID)
            | u64::from(internals.sequence) << BIT_LEN_MACHINE_ID
            | u64::from(self.0.machine_id)))
    }
}

const SONYFLAKE_TIME_UNIT: i64 = 10_000_000; // nanoseconds, i.e. 10msec

pub(crate) fn to_sonyflake_time(time: DateTime<Utc>) -> Result<i64, Error> {
    Ok(time
        .timestamp_nanos_opt()
        .ok_or(Error::FailedToGetCurrentTime)?
        / SONYFLAKE_TIME_UNIT)
}

fn current_elapsed_time(start_time: i64) -> Result<i64, Error> {
    Ok(to_sonyflake_time(Utc::now())? - start_time)
}

#[allow(clippy::cast_sign_loss)]
fn sleep_time(overtime: i64) -> Result<Duration, Error> {
    Ok(Duration::from_millis(overtime as u64 * 10)
        - Duration::from_nanos(
            (Utc::now()
                .timestamp_nanos_opt()
                .ok_or(Error::FailedToGetCurrentTime)?
                % SONYFLAKE_TIME_UNIT) as u64,
        ))
}

/// A decomposed Sonyflake.
pub struct DecomposedSonyflake {
    /// The ID.
    pub id: u64,
    /// The MSB.
    pub msb: u64,
    /// The time.
    pub time: u64,
    /// The sequence number.
    pub sequence: u64,
    /// The machine id.
    pub machine_id: u64,
}

impl DecomposedSonyflake {
    /// Returns the timestamp in nanoseconds without epoch.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn nanos_time(&self) -> i64 {
        (self.time as i64) * SONYFLAKE_TIME_UNIT
    }
}

const DECOMPOSE_MASK_SEQUENCE: u64 = ((1 << BIT_LEN_SEQUENCE) - 1) << BIT_LEN_MACHINE_ID;

const MASK_MACHINE_ID: u64 = (1 << BIT_LEN_MACHINE_ID) - 1;

/// Break a Sonyflake ID up into its parts.
#[must_use]
pub fn decompose(id: Id) -> DecomposedSonyflake {
    let id = id.to_u64();
    DecomposedSonyflake {
        id,
        msb: id >> 63,
        time: id >> (BIT_LEN_SEQUENCE + BIT_LEN_MACHINE_ID),
        sequence: (id & DECOMPOSE_MASK_SEQUENCE) >> BIT_LEN_MACHINE_ID,
        machine_id: id & MASK_MACHINE_ID,
    }
}
