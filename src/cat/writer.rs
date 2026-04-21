// SPDX-License-Identifier: GPL-3.0-or-later
use std::io::{BufWriter, Result, Write};

use super::*;
use crate::{io::DEFAULT_BUFSIZE, md5::Digest};

// =============================================================================
pub struct Writer<W: Write> {
	inner: BufWriter<W>,
}

impl<W: Write> Writer<W> {
	pub fn new(inner: W) -> Self {
		Self::with_capacity(DEFAULT_BUFSIZE, inner)
	}

	pub fn with_capacity(cap: usize, inner: W) -> Self {
		Self {
			inner: BufWriter::with_capacity(cap, inner),
		}
	}

	pub fn write(&mut self, path: &str, size: u64, stamp: Timestamp, hash: &Digest) -> Result<()> {
		writeln!(self.inner, "{path} {size} {stamp} {hash}")
	}

	pub fn write_entry(&mut self, entry: &Entry) -> Result<()> {
		self.write(&entry.path, entry.size, entry.timestamp, &entry.hash)
	}
}
