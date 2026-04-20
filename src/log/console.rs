// SPDX-License-Identifier: GPL-3.0-or-later
use super::Logger;

use std::io::Write;

use anstream::stderr;

// =============================================================================
pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
	fn write(&self, level: super::Level, args: std::fmt::Arguments) {
		let mut s = stderr().lock();
		_ = write!(s, "{level}: ");
		_ = s.write_fmt(args);
		_ = writeln!(s);
	}
}
