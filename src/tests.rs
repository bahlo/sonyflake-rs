use crate::builder::lower_16_bit_private_ip;
use crate::sonyflake::{decompose, to_sonyflake_time, Sonyflake, BIT_LEN_SEQUENCE};
use chrono::prelude::*;
use std::{thread, time::Duration};

#[test]
fn next_id() -> Result<(), Box<dyn std::error::Error>> {
    let mut sf = Sonyflake::new()?;
    assert!(sf.next_id().is_ok());
    Ok(())
}

#[test]
fn once() -> Result<(), Box<dyn std::error::Error>> {
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
fn run_for_10s() -> Result<(), Box<dyn std::error::Error>> {
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
fn threads() -> Result<(), Box<dyn std::error::Error>> {
    let sf = Sonyflake::new()?;

    let mut handles = vec![];
    for _ in 0..100 {
        let mut sfc = sf.clone();
        handles.push(thread::spawn(move || {
            sfc.next_id().unwrap();
        }));
    }

    for handle in handles {
        handle.join().expect("Could not join handle");
    }

    Ok(())
}

#[test]
fn generate_10_ids() -> Result<(), Box<dyn std::error::Error>> {
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
