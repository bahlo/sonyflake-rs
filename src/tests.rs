use crate::{
    builder::lower_16_bit_private_ip,
    error::*,
    sonyflake::{decompose, to_sonyflake_time, Sonyflake, BIT_LEN_SEQUENCE},
};
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

#[test]
fn test_next_id() -> Result<(), Box<dyn std::error::Error>> {
    let mut sf = Sonyflake::new()?;
    assert!(sf.next_id().is_ok());
    Ok(())
}

#[test]
fn test_once() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now();
    let mut sf = Sonyflake::builder().start_time(now).finalize()?;

    let sleep_time = 50;
    thread::sleep(Duration::from_millis(10 * sleep_time));

    let id = sf.next_id()?;
    let parts = decompose(id);

    let actual_msb = *parts.get("msb").expect("No msb part");
    assert_eq!(0, actual_msb, "Unexpected msb");

    let actual_time = *parts.get("time").expect("No time part");
    if actual_time < sleep_time || actual_time > sleep_time + 1 {
        panic!("Unexpected time {}", actual_time)
    }

    let machine_id = lower_16_bit_private_ip()? as u64;
    let actual_machine_id = *parts.get("machine-id").expect("No machine id part");
    assert_eq!(machine_id, actual_machine_id, "Unexpected machine id");

    Ok(())
}

#[test]
fn test_run_for_10s() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now();
    let start_time = to_sonyflake_time(now);
    let mut sf = Sonyflake::builder().start_time(now).finalize()?;

    let mut last_id: u64 = 0;
    let mut max_sequence: u64 = 0;

    let machine_id = lower_16_bit_private_ip()? as u64;

    let initial = to_sonyflake_time(Utc::now());
    let mut current = initial.clone();
    while current - initial < 1000 {
        let id = sf.next_id()?;
        let parts = decompose(id);

        if id <= last_id {
            panic!("duplicated id (id: {}, last_id: {})", id, last_id);
        }
        last_id = id;

        current = to_sonyflake_time(Utc::now());

        let actual_msb = *parts.get("msb").unwrap();
        if actual_msb != 0 {
            panic!("unexpected msb: {}", actual_msb);
        }

        let actual_time = *parts.get("time").unwrap() as i64;
        let overtime = start_time + actual_time - current;
        if overtime > 0 {
            panic!("unexpected overtime: {}", overtime)
        }

        let actual_sequence = *parts.get("sequence").unwrap();
        dbg!(actual_sequence);
        if max_sequence < actual_sequence {
            max_sequence = actual_sequence;
        }

        let actual_machine_id = *parts.get("machine-id").unwrap();
        if actual_machine_id != machine_id {
            panic!("unexpected machine id: {}", actual_machine_id)
        }
    }

    assert_eq!(
        max_sequence,
        (1 << BIT_LEN_SEQUENCE) - 1,
        "unexpected max sequence"
    );

    Ok(())
}

#[test]
fn test_threads() -> Result<(), Box<dyn std::error::Error>> {
    let sf = Sonyflake::new()?;

    let (tx, rx): (Sender<u64>, Receiver<u64>) = mpsc::channel();

    let mut children = Vec::new();
    for _ in 0..10 {
        let mut thread_sf = sf.clone();
        let thread_tx = tx.clone();
        children.push(thread::spawn(move || {
            for _ in 0..1000 {
                thread_tx.send(thread_sf.next_id().unwrap()).unwrap();
            }
        }));
    }

    let mut ids = HashSet::new();
    for _ in 0..10_000 {
        let id = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert!(!ids.contains(&id), "duplicate id: {}", id);
        ids.insert(id);
    }

    for child in children {
        child.join().expect("Child thread panicked");
    }

    Ok(())
}

#[test]
fn test_generate_10_ids() -> Result<(), Box<dyn std::error::Error>> {
    let mut sf = Sonyflake::builder().machine_id(&|| Ok(42)).finalize()?;
    let mut ids = vec![];
    for _ in 0..10 {
        let id = sf.next_id()?;
        if ids.iter().find(|vec_id| **vec_id == id).is_some() {
            panic!("duplicated id: {}", id)
        }
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
    };

    match Sonyflake::builder()
        .machine_id(&|| Err(Box::new(TestError::SomeError)))
        .finalize()
    {
        Err(Error::MachineIdFailed(_)) => {} // ok
        _ => panic!("Expected error failing machine_id closure"),
    };

    match Sonyflake::builder().check_machine_id(&|_| false).finalize() {
        Err(Error::CheckMachineIdFailed) => {}
        _ => panic!("Expected error on check_machine_id closure returning false"),
    }
}
