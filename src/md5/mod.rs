// SPDX-License-Identifier: GPL-3.0-or-later
mod digest;
mod platform;

use std::io::Write;

#[cfg(test)]
mod tests;

// =============================================================================
pub use digest::*;

// =============================================================================
// based on ietf.org/rfc/rfc1321.txt, modified for 64bit
const DEFAULT_STATE: [u32; 4] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];
const BLOCKSIZE: usize = 64;

#[allow(clippy::zero_prefixed_literal, reason = "nice alignment")]
#[rustfmt::skip]
const S: [u32; BLOCKSIZE] = [
	07, 12, 17, 22, 07, 12, 17, 22, 07, 12, 17, 22, 07, 12, 17, 22,
	05, 09, 14, 20, 05, 09, 14, 20, 05, 09, 14, 20, 05, 09, 14, 20,
	04, 11, 16, 23, 04, 11, 16, 23, 04, 11, 16, 23, 04, 11, 16, 23,
	06, 10, 15, 21, 06, 10, 15, 21, 06, 10, 15, 21, 06, 10, 15, 21,
];

const K: [u32; BLOCKSIZE] = [
	0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
	0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
	0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
	0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
	0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
	0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
	0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
	0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

// pre-computed g-index table
const G: [usize; BLOCKSIZE] = {
	let mut table = [0usize; BLOCKSIZE];
	let mut i = 0usize;
	// still can't use for loop over ranges in const contexts :(
	// and `iter_mut` isn't stable either (const)
	while i < BLOCKSIZE {
		table[i] = match i {
			0..16 => i,
			16..32 => (5 * i + 1) % 16,
			32..48 => (3 * i + 5) % 16,
			_ /* 48..64 */ => (7 * i) % 16,
		};
		i += 1;
	}
	table
};

const PADDING: [u8; BLOCKSIZE] = {
	let mut data = [0; BLOCKSIZE];
	data[0] = 0x80;
	data
};

pub struct Context {
	state: [u32; 4],
	buf: [u8; BLOCKSIZE],
	cursor: usize,
	len: u64,
}

impl Context {
	#[inline]
	pub fn new() -> Self {
		Self {
			state: DEFAULT_STATE,
			buf: [0u8; BLOCKSIZE],
			cursor: 0,
			len: 0,
		}
	}

	pub fn update<T: AsRef<[u8]>>(&mut self, data: T) {
		// weird:
		// destructuring like this results in a ~3x speed increase over
		// just passing `self` and updating in-place...
		(self.state, self.buf, self.cursor, self.len) =
			update(self.state, self.buf, self.cursor, self.len, data.as_ref());
	}

	pub fn finalize(self) -> Digest {
		finalize(self.state, self.buf, self.cursor, self.len)
	}
}

impl Default for Context {
	fn default() -> Self {
		Self::new()
	}
}

impl Write for Context {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.update(buf);
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

// =============================================================================
#[inline(always)]
fn update(
	mut state: [u32; 4],
	mut buf: [u8; BLOCKSIZE],
	mut cursor: usize,
	mut len: u64,
	mut data: &[u8],
) -> ([u32; 4], [u8; BLOCKSIZE], usize, u64) {
	// fill partial buffer
	if cursor != 0 {
		let needed = BLOCKSIZE - cursor;
		let take = needed.min(data.len());
		buf[cursor..cursor + take].copy_from_slice(&data[..take]);
		cursor += take;
		data = &data[take..];
		if cursor == BLOCKSIZE {
			transform(&mut state, &buf);
			cursor = 0;
		}
	}

	// process full blocks
	while data.len() >= BLOCKSIZE {
		let block = &data[..BLOCKSIZE];
		transform(&mut state, block.try_into().unwrap());
		data = &data[BLOCKSIZE..];
		len = len.wrapping_add(BLOCKSIZE as u64);
	}

	// process remaining
	if !data.is_empty() {
		buf[..data.len()].copy_from_slice(data);
		cursor = data.len();
	}

	len = len.wrapping_add(data.len() as u64);
	(state, buf, cursor, len)
}

#[inline(always)]
fn finalize(mut state: [u32; 4], mut buf: [u8; BLOCKSIZE], cursor: usize, len: u64) -> Digest {
	if cursor > 55 {
		buf[cursor..BLOCKSIZE].copy_from_slice(&PADDING[..BLOCKSIZE - cursor]);
		transform(&mut state, &buf);
		buf[0..56].copy_from_slice(&PADDING[1..57]);
	} else {
		buf[cursor..56].copy_from_slice(&PADDING[..56 - cursor]);
	}

	buf[56..].copy_from_slice(&(len << 3).to_le_bytes());
	transform(&mut state, &buf);

	Digest({
		let mut r = [0u8; 16];
		for (i, chunk) in r.chunks_exact_mut(4).enumerate() {
			chunk.copy_from_slice(&state[i].to_le_bytes());
		}
		r
	})
}

#[inline(always)]
fn transform(state: &mut [u32; 4], data: &[u8; BLOCKSIZE]) {
	let mut m = [0u32; 16];

	// due to const data, and const-sized arrays, bounds-checking *should* be eliminated in here
	for (i, chunk) in data.chunks_exact(4).enumerate() {
		m[i] = u32::from_le_bytes(chunk.try_into().unwrap());
	}

	let mut a = state[0];
	let mut b = state[1];
	let mut c = state[2];
	let mut d = state[3];

	macro_rules! ROUND0 {
		($i:literal) => {
			let f = (b & c) | (!b & d);
			rotate(&mut a, &mut b, &mut c, &mut d, f, m[G[$i]], $i);
		};
	}

	macro_rules! ROUND1 {
		($i:literal) => {
			let f = (d & b) | (!d & c);
			rotate(&mut a, &mut b, &mut c, &mut d, f, m[G[$i]], $i);
		};
	}

	macro_rules! ROUND2 {
		($i:literal) => {
			let f = b ^ c ^ d;
			rotate(&mut a, &mut b, &mut c, &mut d, f, m[G[$i]], $i);
		};
	}

	macro_rules! ROUND3 {
		($i:literal) => {
			let f = c ^ (b | !d);
			rotate(&mut a, &mut b, &mut c, &mut d, f, m[G[$i]], $i);
		};
	}

	// fully unrolled rounds
	// 0..16
	ROUND0!(0);
	ROUND0!(1);
	ROUND0!(2);
	ROUND0!(3);
	ROUND0!(4);
	ROUND0!(5);
	ROUND0!(6);
	ROUND0!(7);
	ROUND0!(8);
	ROUND0!(9);
	ROUND0!(10);
	ROUND0!(11);
	ROUND0!(12);
	ROUND0!(13);
	ROUND0!(14);
	ROUND0!(15);

	// 16..31
	ROUND1!(16);
	ROUND1!(17);
	ROUND1!(18);
	ROUND1!(19);
	ROUND1!(20);
	ROUND1!(21);
	ROUND1!(22);
	ROUND1!(23);
	ROUND1!(24);
	ROUND1!(25);
	ROUND1!(26);
	ROUND1!(27);
	ROUND1!(28);
	ROUND1!(29);
	ROUND1!(30);
	ROUND1!(31);

	// 32..47
	ROUND2!(32);
	ROUND2!(33);
	ROUND2!(34);
	ROUND2!(35);
	ROUND2!(36);
	ROUND2!(37);
	ROUND2!(38);
	ROUND2!(39);
	ROUND2!(40);
	ROUND2!(41);
	ROUND2!(42);
	ROUND2!(43);
	ROUND2!(44);
	ROUND2!(45);
	ROUND2!(46);
	ROUND2!(47);

	// 48..63
	ROUND3!(48);
	ROUND3!(49);
	ROUND3!(50);
	ROUND3!(51);
	ROUND3!(52);
	ROUND3!(53);
	ROUND3!(54);
	ROUND3!(55);
	ROUND3!(56);
	ROUND3!(57);
	ROUND3!(58);
	ROUND3!(59);
	ROUND3!(60);
	ROUND3!(61);
	ROUND3!(62);
	ROUND3!(63);

	state[0] = state[0].wrapping_add(a);
	state[1] = state[1].wrapping_add(b);
	state[2] = state[2].wrapping_add(c);
	state[3] = state[3].wrapping_add(d);
}

#[inline(always)]
fn rotate(a: &mut u32, b: &mut u32, c: &mut u32, d: &mut u32, mut f: u32, g: u32, i: usize) {
	f = f.wrapping_add(*a).wrapping_add(K[i]).wrapping_add(g);
	*a = *d;
	*d = *c;
	*c = *b;
	*b = f.rotate_left(S[i]).wrapping_add(*b);
}
