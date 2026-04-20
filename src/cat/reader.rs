// SPDX-License-Identifier: GPL-3.0-or-later
use super::Entry;
use crate::md5::{Digest, DigestError};
use crate::io::DEFAULT_BUFSIZE;

use std::fmt;
use std::io::{self, BufRead, BufReader, Read};

// =============================================================================
#[derive(Debug)]
pub enum Error {
	IoError(io::Error),
	ParseError(usize, String),
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Error::IoError(value)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::IoError(inner) => inner.fmt(f),
			Error::ParseError(line, msg) => write!(f, "parse error at line #{line}: {msg}"),
		}
	}
}

pub type Result<T> = std::result::Result<T, Error>;

// =============================================================================
pub struct Reader<R: Read> {
	inner: BufReader<R>,
	line: usize,
	offset: u64,
}

impl<R: Read> Reader<R> {
	pub fn new(inner: R) -> Self {
		Self::with_capacity(DEFAULT_BUFSIZE, inner)
	}

	pub fn with_capacity(cap: usize, inner: R) -> Self {
		Self {
			inner: BufReader::with_capacity(cap, inner),
			line: 0,
			offset: 0,
		}
	}

	pub fn read_entry(&mut self, out: &mut Entry) -> Result<bool> {
		out.path.clear();

		if self.inner.read_line(&mut out.path)? == 0 {
			return Ok(false);
		}

		let mut it = out.path.trim_end().rsplitn(4, ' ');

		let hash = it.next().ok_or_else(|| self.format_error(0))?;
		Digest::decode_into(hash, &mut out.hash).map_err(|e| self.hash_error(hash, e))?;

		let stamp = it.next().ok_or_else(|| self.format_error(1))?;
		out.timestamp = stamp.parse().map_err(|e| self.num_error(stamp, e))?;

		let size = it.next().ok_or_else(|| self.format_error(2))?;
		out.size = size.parse().map_err(|e| self.num_error(size, e))?;

		let path = it.next().ok_or_else(|| self.format_error(3))?;
		out.path.truncate(path.len());

		out.offset = self.offset;
		self.offset += out.size;
		self.line += 1;

		Ok(true)
	}

	fn format_error(&self, actual: usize) -> Error {
		Error::ParseError(self.line, format!("expected 4 fields, found {actual}"))
	}

	#[allow(clippy::needless_pass_by_value)]
	fn num_error(&self, source: &str, inner: std::num::ParseIntError) -> Error {
		Error::ParseError(self.line, format!("{inner} - '{source}'"))
	}

	#[allow(clippy::needless_pass_by_value)]
	fn hash_error(&self, source: &str, inner: DigestError) -> Error {
		Error::ParseError(self.line, format!("{inner} - '{source}'"))
	}
}

impl<R: Read> Iterator for Reader<R> {
	type Item = Result<Entry>;

	fn next(&mut self) -> Option<Self::Item> {
		let mut r = Entry::with_capacity(256); // wc -L /path/to/game/**/*.cat gives 248
		match self.read_entry(&mut r) {
			Ok(true) => Some(Ok(r)),
			Ok(false) => None,
			Err(e) => Some(Err(e)),
		}
	}
}
