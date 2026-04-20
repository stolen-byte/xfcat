// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;

use std::io::Cursor;

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
