use crate::builder::lower_16_bit_private_ip;
use crate::sonyflake::{decompose, Sonyflake};
use chrono::prelude::*;
use std::{thread, time::Duration};

#[test]
fn next_id() {
    let mut sf = Sonyflake::builder()
        .finalize()
        .expect("Could not construct Sonyflake");
    assert!(sf.next_id().is_ok());
}

#[test]
fn sonyflake_once() -> Result<(), Box<dyn std::error::Error>> {
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
