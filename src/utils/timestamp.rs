// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
	fmt,
	fs::FileTimes,
	ops,
	time::{Duration, SystemTime},
};

use time::UtcDateTime;

// =============================================================================
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Timestamp(pub i64);

impl Timestamp {
	pub fn as_utc(self) -> UtcDateTime {
		UtcDateTime::from_unix_timestamp(self.0).expect("timestamp value out of range")
	}

	pub fn as_file_time(self) -> FileTimes {
		let st = self.as_system_time();
		FileTimes::new().set_accessed(st).set_modified(st)
	}

	pub fn as_system_time(self) -> SystemTime {
		let d = Duration::from_secs(self.0.unsigned_abs());
		if d.is_zero() {
			SystemTime::UNIX_EPOCH
		} else if self.0.is_positive() {
			SystemTime::UNIX_EPOCH + d
		} else {
			SystemTime::UNIX_EPOCH - d
		}
	}
}

impl ops::Deref for Timestamp {
	type Target = i64;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ops::DerefMut for Timestamp {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl From<i64> for Timestamp {
	#[inline]
	fn from(value: i64) -> Self {
		Self(value)
	}
}

impl From<SystemTime> for Timestamp {
	#[inline]
	fn from(value: SystemTime) -> Self {
		From::from(&value)
	}
}

impl From<&SystemTime> for Timestamp {
	#[inline]
	fn from(value: &SystemTime) -> Self {
		let ts = match value.duration_since(SystemTime::UNIX_EPOCH) {
			Ok(d) => d.as_secs(),
			Err(e) => e.duration().as_secs(),
		};
		Self(ts.cast_signed())
	}
}

impl From<Timestamp> for FileTimes {
	fn from(value: Timestamp) -> Self {
		value.as_file_time()
	}
}

impl std::str::FromStr for Timestamp {
	type Err = std::num::ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self(s.parse()?))
	}
}

impl fmt::Display for Timestamp {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			let utc = self.as_utc();
			let (year, month, day) = utc.to_calendar_date();
			let (h, m, _) = utc.as_hms();
			let month = month_as_str(month);
			write!(f, "{month} {day:>2} {year} {h:0>2}:{m:0>2}")
		} else {
			self.0.fmt(f)
		}
	}
}

// =============================================================================
#[inline]
fn month_as_str(m: time::Month) -> &'static str {
	use time::Month;

	match m {
		Month::January => "Jan",
		Month::February => "Feb",
		Month::March => "Mar",
		Month::April => "Apr",
		Month::May => "May",
		Month::June => "Jun",
		Month::July => "Jul",
		Month::August => "Aug",
		Month::September => "Sep",
		Month::October => "Oct",
		Month::November => "Nov",
		Month::December => "Dec",
	}
}
