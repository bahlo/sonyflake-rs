use chrono::prelude::*;
use std::{
    collections::HashSet,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
    },
    thread,
    time::Duration,
};
use thiserror::Error;

use crate::{
    builder::lower_16_bit_private_ip,
    error::*,
    sonyflake::{BIT_LEN_SEQUENCE, BIT_LEN_TIME, Sonyflake, decompose, to_sonyflake_time},
};

#[test]
fn test_next_id() -> Result<(), BoxDynError> {
    let sf = Sonyflake::new()?;
    assert!(sf.next_id().is_ok());
    Ok(())
}

#[test]
fn test_once() -> Result<(), BoxDynError> {
    let now = Utc::now();
    let sf = Sonyflake::builder().start_time(now).finalize()?;

    let sleep_time = 50;
    thread::sleep(Duration::from_millis(10 * sleep_time));

    let id = sf.next_id()?;
    let parts = decompose(id);

    let actual_msb = parts.msb;
    assert_eq!(0, actual_msb, "Unexpected msb");

    let actual_time = parts.time;
    assert!(
        !(actual_time < sleep_time || actual_time > sleep_time + 1),
        "Unexpected time {actual_time}",
    );

    let machine_id = u64::from(lower_16_bit_private_ip()?);
    let actual_machine_id = parts.machine_id;
    assert_eq!(machine_id, actual_machine_id, "Unexpected machine id");

    Ok(())
}

#[test]
#[allow(clippy::cast_possible_wrap)]
fn test_run_for_10s() -> Result<(), BoxDynError> {
    let now = Utc::now();
    let start_time = to_sonyflake_time(now)?;
    let sf = Sonyflake::builder().start_time(now).finalize()?;

    let mut last_id: u64 = 0;
    let mut max_sequence: u64 = 0;

    let machine_id = u64::from(lower_16_bit_private_ip()?);

    let initial = to_sonyflake_time(Utc::now())?;
    let mut current = initial;
    while current - initial < 1000 {
        let id = sf.next_id()?;
        let parts = decompose(id);

        assert!(id > last_id, "duplicated id (id: {id}, last_id: {last_id})",);
        last_id = id;

        current = to_sonyflake_time(Utc::now())?;

        let actual_msb = parts.msb;
        assert!(actual_msb == 0, "unexpected msb: {actual_msb}");

        let actual_time = parts.time as i64;
        let overtime = start_time + actual_time - current;
        assert!(overtime <= 0, "unexpected overtime: {overtime}");

        let actual_sequence = parts.sequence;
        if max_sequence < actual_sequence {
            max_sequence = actual_sequence;
        }

        let actual_machine_id = parts.machine_id;
        assert!(
            actual_machine_id == machine_id,
            "unexpected machine id: {actual_machine_id}",
        );
    }

    assert_eq!(
        max_sequence,
        (1 << BIT_LEN_SEQUENCE) - 1,
        "unexpected max sequence"
    );

    Ok(())
}

#[test]
fn test_threads() -> Result<(), BoxDynError> {
    let sf = Sonyflake::new()?;

    let (tx, rx): (Sender<u64>, Receiver<u64>) = mpsc::channel();

    let mut children = Vec::new();
    for _ in 0..10 {
        let thread_sf = sf.clone();
        let thread_tx = tx.clone();
        children.push(thread::spawn(move || {
            for _ in 0..1000 {
                thread_tx
                    .send(thread_sf.next_id().expect("failed to get next id"))
                    .expect("failed to send");
            }
        }));
    }

    let mut ids = HashSet::new();
    for _ in 0..10_000 {
        let id = rx
            .recv_timeout(Duration::from_millis(100))
            .expect("failed to receive");
        assert!(!ids.contains(&id), "duplicate id: {id}");
        ids.insert(id);
    }

    for child in children {
        child.join().expect("Child thread panicked");
    }

    Ok(())
}

#[test]
fn test_generate_10_ids() -> Result<(), BoxDynError> {
    let sf = Sonyflake::builder().machine_id(&|| Ok(42)).finalize()?;
    let mut ids = vec![];
    for _ in 0..10 {
        let id = sf.next_id()?;
        assert!(
            !ids.iter().any(|vec_id: &u64| *vec_id == id),
            "duplicated id: {id}",
        );
        ids.push(id);
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum TestError {
    #[error("some error")]
    SomeError,
}

#[test]
fn test_builder_errors() {
    let start_time = Utc::now() + chrono::Duration::seconds(1);
    match Sonyflake::builder().start_time(start_time).finalize() {
        Err(Error::StartTimeAheadOfCurrentTime(_)) => {} // ok
        _ => panic!("Expected error on start time ahead of current time"),
    }

    match Sonyflake::builder()
        .machine_id(&|| Err(Box::new(TestError::SomeError)))
        .finalize()
    {
        Err(Error::MachineIdFailed(_)) => {} // ok
        _ => panic!("Expected error failing machine_id closure"),
    }

    match Sonyflake::builder().check_machine_id(&|_| false).finalize() {
        Err(Error::CheckMachineIdFailed) => {}
        _ => panic!("Expected error on check_machine_id closure returning false"),
    }
}

#[test]
fn test_error_send_sync() {
    let res = Sonyflake::new();
    thread::spawn(move || {
        let _ = res.is_ok();
    })
    .join()
    .expect("failed to join thread");
}

#[test]
fn test_over_time_limit() -> Result<(), BoxDynError> {
    let sf = Sonyflake::new()?;
    let mut internals = sf.0.internals.lock().expect("failed to aquire lock");
    internals.elapsed_time = 1 << BIT_LEN_TIME;
    drop(internals);
    assert!(sf.next_id().is_err());
    Ok(())
}
