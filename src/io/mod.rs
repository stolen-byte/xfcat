// SPDX-License-Identifier: GPL-3.0-or-later
use std::io::{Result, Write};

// =============================================================================
pub const DEFAULT_BUFSIZE: usize = 1024 * 32; // 32KB

// =============================================================================
mod copier;
pub use copier::*;

#[cfg(test)]
mod tests;

// =============================================================================
pub struct TeeWriter<T: Write, U: Write>(T, U);

impl<T: Write, U: Write> TeeWriter<T, U> {
	pub fn new(a: T, b: U) -> Self {
		Self(a, b)
	}
}

impl<T: Write, U: Write> Write for TeeWriter<T, U> {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		Ok(std::cmp::min(self.0.write(buf)?, self.1.write(buf)?))
	}

	fn flush(&mut self) -> Result<()> {
		self.0.flush()?;
		self.1.flush()
	}
}
