use chrono::{DateTime, Utc};
use std::error::Error as StdError;
use thiserror::Error;

/// Convenience type alias for usage within sonyflake.
pub(crate) type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

/// The error type for this crate.
#[derive(Error, Debug)]
pub enum Error {
    /// Start time is ahead of current time
    #[error("start_time `{0}` is ahead of current time")]
    StartTimeAheadOfCurrentTime(DateTime<Utc>),
    /// Maching ID fn failed.
    #[error("machine_id returned an error: {0}")]
    MachineIdFailed(#[source] BoxDynError),
    /// Maching ID check failed.
    #[error("check_machine_id returned false")]
    CheckMachineIdFailed,
    /// Over time limit.
    #[error("over the time limit")]
    OverTimeLimit,
    /// No prive IPv4.
    #[error("could not find any private ipv4 address")]
    NoPrivateIPv4,
    /// Mutex is poisoned.
    #[error("mutex is poisoned (i.e. a panic happened while it was locked)")]
    MutexPoisoned,
    /// Failed to get current time.
    #[error("failed to get current time")]
    FailedToGetCurrentTime,
    /// Returned if the pnet features is deactivated an no custom machine id
    /// function was provided.
    #[error("no machine id function provided")]
    NoMachineIdFn,
}
