// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;

use std::cmp::Ord;
use std::path::Path;
use std::time::{Duration, SystemTime};

// =============================================================================
#[test]
fn timestamp_conversions() {
	let t = SystemTime::UNIX_EPOCH + Duration::from_secs(12345);

	assert_eq!(*Timestamp(12345), 12345i64, "Timestamp <-> i64");
	assert_eq!(Timestamp::from(t), Timestamp(12345), "SystemTime -> Timestamp");
	assert_eq!(Timestamp::from(t).as_system_time(), t, "Timestamp -> SystemTime");
}

#[test]
fn timestamp_parse() {
	assert_eq!(Timestamp(12345), "12345".parse().unwrap());
}

#[test]
fn timestamp_formatting() {
	assert_eq!("12345", Timestamp(12345).to_string(), "bare timestamp");
	assert_eq!("Apr 11 2025 19:50", format!("{:#}", Timestamp(1744401000)), "utc time/date");
}

#[test]
fn timestamp_ordering() {
	assert!(Ord::cmp(&Timestamp(1000), &Timestamp(2000)).is_lt());
	assert!(Ord::cmp(&Timestamp(2000), &Timestamp(2000)).is_eq());
}

#[test]
fn as_context() {
	assert_eq!("/some/file.txt".to_string(), Path::new("/some/file.txt").as_context());
}

#[test]
fn size_formatting() {
	assert_eq!("123456", SizeFormat::Raw(123456).to_string());

	assert_eq!("123B", SizeFormat::Human(123).to_string());
	assert_eq!("123K", SizeFormat::Human(125952).to_string());
	assert_eq!("123.5K", SizeFormat::Human(126464).to_string());
	assert_eq!("123M", SizeFormat::Human(128974848).to_string());
	assert_eq!("123.5M", SizeFormat::Human(129499136).to_string());
	assert_eq!("123G", SizeFormat::Human(132070244352).to_string());
	assert_eq!("123.5G", SizeFormat::Human(132607115264).to_string());

	// alignment & width
	assert_eq!(format!("{:|>10}", SizeFormat::Human(132607115264)), "||||123.5G", "align right");
	assert_eq!(format!("{:|<10}", SizeFormat::Human(132607115264)), "123.5G||||", "align left");
	assert_eq!(format!("{:|^10}", SizeFormat::Human(132607115264)), "||123.5G||", "align center");
}
