// SPDX-License-Identifier: GPL-3.0-or-later
use crate::md5::Digest;
use crate::utils::Timestamp;

use std::io::{Read, Result as IoResult, Seek, SeekFrom};

// =============================================================================
#[derive(Default, Debug, PartialEq)]
pub struct Entry {
	pub path: String,
	pub hash: Digest,
	pub timestamp: Timestamp,
	pub size: u64,
	offset: u64,
}

impl Entry {
	pub fn with_capacity(cap: usize) -> Self {
		let mut r = Self::default();
		r.path.reserve_exact(cap);
		r
	}

	pub fn reader<R: Read + Seek>(&self, source: &mut R) -> IoResult<impl Read> {
		source.seek(SeekFrom::Start(self.offset))?;
		Ok(source.take(self.size))
	}
}

// =============================================================================
mod reader;
pub use reader::*;

mod writer;
pub use writer::*;

#[cfg(test)]
mod tests;
