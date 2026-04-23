// SPDX-License-Identifier: GPL-3.0-or-later
use std::{fmt, time::Duration};

// =============================================================================
#[derive(Debug)]
pub struct FormattedDuration(pub Duration);

impl fmt::Display for FormattedDuration {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut t = self.0.as_secs();
		let secs = t % 60;
		t /= 60;
		let mins = t % 60;
		t /= 60;
		if t > 0 {
			let hours = t % 24;
			t /= 24;
			if t > 0 {
				write!(f, "{t}d ")?;
			}
			write!(f, "{hours:02}:")?;
		}
		write!(f, "{mins:02}:{secs:02}")
	}
}
