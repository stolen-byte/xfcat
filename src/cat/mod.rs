// SPDX-License-Identifier: GPL-3.0-or-later
use crate::md5::Digest;
use crate::utils::Timestamp;

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
}

// =============================================================================
mod reader;
pub use reader::*;

#[cfg(test)]
mod tests;
