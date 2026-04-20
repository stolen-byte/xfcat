// SPDX-License-Identifier: GPL-3.0-or-later
use std::io::{Read, Result, Write};
use std::mem::MaybeUninit;
use std::slice;

// =============================================================================
/// a customizable & reusable version of `io::copy`
pub struct StreamCopier {
	buf: Box<[MaybeUninit<u8>]>,
}

impl StreamCopier {
	pub fn new() -> Self {
		Self::with_capacity(super::DEFAULT_BUFSIZE)
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self {
			buf: Box::new_uninit_slice(cap),
		}
	}

	pub fn copy<R, W>(&mut self, reader: &mut R, writer: &mut W) -> Result<u64>
	where
		R: Read,
		W: Write,
	{
		let mut copied = 0;

		loop {
			let nread = reader.read(self.buffer())?;
			if nread == 0 {
				break;
			}

			writer.write_all(unsafe { self.as_used(nread) })?;
			copied += nread as u64;
		}

		Ok(copied)
	}

	#[inline]
	fn buffer(&mut self) -> &mut [u8] {
		unsafe { slice::from_raw_parts_mut(self.buf.as_mut_ptr().cast(), self.buf.len()) }
	}

	/// caller must make sure that `0..n` bytes are *actually* initialized
	#[inline]
	unsafe fn as_used(&self, n: usize) -> &[u8] {
		unsafe { self.buf.get_unchecked(..n).assume_init_ref() }
	}
}

impl Default for StreamCopier {
	fn default() -> Self {
		Self::new()
	}
}
