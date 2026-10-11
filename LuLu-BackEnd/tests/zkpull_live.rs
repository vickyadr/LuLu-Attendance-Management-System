use lulu_attendance_server::utility::zkpull::{ZkPull, decode_device_time, decode_records};
use std::time::Duration;

#[test]
fn pull_from_the_real_device_and_check_the_rows() {
    let Ok(addr) = std::env::var("ZK_TEST_ADDR") else {
        eprintln!("ZK_TEST_ADDR not set, skipping live pull");
        return;
    };
    let addr = addr.parse().expect("ZK_TEST_ADDR must be host:port");
    let mut zk = ZkPull::connect(&addr, Duration::from_secs(20)).expect("handshake");
    eprintln!("handshake ok, pulling");
    let (payload, records) = zk.pull_attendance().expect("pull");
    eprintln!("pull returned");

    println!("payload {} bytes, device claims {records} records", payload.len());
    assert!(!payload.is_empty(), "empty payload");
    assert_eq!(
        payload.len() / records as usize,
        40,
        "expected the 40-byte layout"
    );

    let rows = decode_records(&payload, 40);
    println!("decoded {} rows", rows.len());
    assert!(rows.len() > 9000, "expected the historical table, got {}", rows.len());

    for row in rows.iter().take(3).chain(rows.iter().rev().take(3)) {
        let (y, mo, d, h, mi, s) = decode_device_time(row.device_time);
        println!("uid={} time={y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}", row.user_id);
        assert!(
            (2020..=2030).contains(&y),
            "implausible year {y} for uid {}",
            row.user_id
        );
    }

    // Every decoded id must be a positive integer: the push path skips anything
    // else, so a non-numeric id here means the two paths disagree.
    let bad = rows
        .iter()
        .filter(|r| r.user_id.parse::<i32>().unwrap_or(0) <= 0)
        .count();
    assert_eq!(bad, 0, "{bad} rows carry an unusable user id");
    println!("live pull OK");
}