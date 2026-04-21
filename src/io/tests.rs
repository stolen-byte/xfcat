// SPDX-License-Identifier: GPL-3.0-or-later
use std::io::{self, Cursor};

use super::*;

// =============================================================================
#[test]
fn stream_copier() {
	const DATASIZE: usize = 1024;
	let mut r = Cursor::new(vec![0xffu8; DATASIZE]);
	let mut w = Vec::with_capacity(DATASIZE);
	let mut copier = StreamCopier::with_capacity(DATASIZE / 2);

	let size = copier.copy(&mut r, &mut w).unwrap();

	assert_eq!(size, DATASIZE as u64);
	assert_eq!(w.as_slice(), r.get_ref());
}

#[test]
fn tee_writer() {
	const DATA: &[u8] = b"The quick brown fox jumps over the lazy dog";
	let mut r = Cursor::new(DATA);
	let mut buf1 = Vec::with_capacity(DATA.len());
	let mut buf2 = Vec::with_capacity(DATA.len());
	let mut writer = TeeWriter::new(buf1.by_ref(), buf2.by_ref());

	let copied = io::copy(&mut r, &mut writer).unwrap();

	assert_eq!(copied, DATA.len() as u64);
	assert_eq!(buf1.as_slice(), DATA);
	assert_eq!(buf2.as_slice(), DATA);
}
