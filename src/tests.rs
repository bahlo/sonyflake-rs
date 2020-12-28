use crate::builder::lower_16_bit_private_ip;
use crate::sonyflake::{decompose, Sonyflake};
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
