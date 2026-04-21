// SPDX-License-Identifier: GPL-3.0-or-later
use std::io::Write;

use anstream::stderr;

use super::Logger;

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
